# Chronicle Milestone 0 设计规格

日期：2026-06-21

## 目标与范围

Milestone 0 建立可安装、可测试、可持续扩展的 Chronicle 桌面应用基础。应用使用 Tauri、React、TypeScript、Vite、Rust 与 SQLite，启动后展示真实的本地数据库状态、空时间线、空索引文件夹列表与设置页面。

本里程碑不选择文件夹、不扫描文件、不监听文件系统、不生成文件事件、不读取文件内容，也不提供任何破坏性文件操作。所有空状态都来自真实数据库，不使用演示数据。

新增产品要求：界面同时支持简体中文与英文。首次启动按系统语言选择，用户可在设置页切换语言，偏好保存在本机浏览器存储中。该偏好不包含文件信息，不离开设备。

## 已确认的产品方向

Chronicle 是本地优先的文件活动记忆工具，不是云盘、员工监控工具、静态网页、AI 聊天包装器或生产力统计面板。React 只负责界面与交互，文件系统、数据库和后台任务边界属于 Rust。

完整产品流程为：启动 Chronicle、明确选择索引文件夹、执行元数据扫描、浏览时间线、查看文件详情。Milestone 0 只实现第一个环节，并为后续环节建立类型和模块边界。

## 方案比较与选择

### 方案 A：React 直接访问 SQLite

通过 SQL 插件把查询能力暴露给前端。开发速度快，但数据库结构泄漏到界面层，文件系统与业务规则难以集中测试，也会削弱隐私边界。

### 方案 B：Tauri command 包含业务逻辑

每个 Tauri command 直接查询数据库。初始代码少，但命令会逐渐承担校验、查询、扫描与错误转换，后续难以脱离窗口测试。

### 方案 C：薄命令层加 Rust 服务核心

React 通过类型化客户端调用少量命令。命令只校验输入、调用服务并序列化结果。数据库、扫描器、事件与任务逻辑位于独立 Rust 模块。

选择方案 C。它最符合隐私、测试性、跨平台与长期演进要求。

## 高层架构

```text
React application shell
  -> typed TypeScript Tauri client
    -> thin Tauri commands
      -> Rust application services
        -> database service -> SQLite + migrations
        -> task service -> background task contracts
        -> scanner contract -> future metadata scanning
        -> platform contract -> Windows-first adapters
```

### React 边界

- `app`：应用启动、路由、全局错误边界与语言初始化。
- `components/layout`：桌面外壳、侧栏、顶部区域与状态区域。
- `pages`：Timeline、Indexed Folders、Settings 页面。
- `components/states`：与最终内容同形的加载骨架、上下文空状态、可恢复错误状态。
- `lib/tauri`：唯一允许调用 Tauri invoke 的前端模块。
- `models`：不含 `any` 的共享 TypeScript DTO。
- `i18n`：中英文资源、语言检测与切换。

React 组件不得包含 SQL、文件系统路径处理或扫描逻辑。

### Rust 边界

- `commands`：薄 Tauri wrapper，负责参数与返回值边界。
- `database`：连接、迁移、repository 与数据库状态。
- `scanner`：仅定义未来扫描契约，不读取文件系统。
- `events`：事件类型与未来事件处理契约。
- `tasks`：后台任务状态模型与未来执行契约，Milestone 0 保持 idle。
- `platform`：跨平台接口和 Windows 优先的实现位置。
- `models`：序列化 DTO 与领域模型。
- `errors`：带稳定错误码和安全消息的 typed error。

核心服务可在没有 Tauri 窗口的测试中运行。生产路径上的可恢复错误不使用 `unwrap` 或 `expect`。

## 数据流与启动流程

1. Tauri 解析平台应用数据目录并创建 Chronicle 数据目录。
2. Rust database service 打开 SQLite，启用 foreign keys，并按顺序执行版本化 migration。
3. Tauri 将数据库服务放入托管状态。
4. React 启动后并行请求 application info、database status、indexed folders 与第一页 timeline。
5. 每个页面明确呈现 loading、ready、empty 或 error 状态。
6. 数据库失败时，应用外壳与设置页仍可显示，错误文案明确说明原始文件未被修改。

## SQLite 设计

选择 `rusqlite` 与 `rusqlite_migration`，数据库只在 Rust 中访问。使用 bundled SQLite 以减少 Windows 环境差异。测试通过 `tempfile` 创建隔离数据库。

### 通用约定

- 主键使用 SQLite `INTEGER PRIMARY KEY`，Rust 类型为 `i64`。
- 时间使用 UTC RFC 3339 `TEXT`。
- 布尔值使用带 `CHECK (value IN (0, 1))` 的 `INTEGER`。
- 规范化路径保存为 `TEXT`，比较规则由未来 platform 模块决定，不在本里程碑假设所有系统大小写不敏感。
- 启用 `PRAGMA foreign_keys = ON`。
- migration 嵌入二进制并按版本顺序执行。

### `indexed_folders`

- `id`
- `normalized_path`，唯一且非空
- `display_name`
- `added_at`
- `last_successful_scan_at`，可空
- `monitoring_enabled`，默认 false
- `availability_status`

### `files`

- `id`
- `indexed_folder_id`，引用 indexed folders
- `normalized_path`
- `name`
- `extension`，可空
- `size_bytes`
- `filesystem_created_at`，可空
- `filesystem_modified_at`
- `first_indexed_at`
- `last_seen_at`
- `is_present`
- 同一 indexed folder 内 normalized path 唯一

### `file_events`

- `id`
- `file_id`，可空，删除事件仍可保留上下文
- `indexed_folder_id`
- `event_type`
- `detected_at`
- `filesystem_time`，可空
- `old_path`，可空
- `new_path`，可空
- `confidence`，可空且限制在 0 到 1
- `event_source`

### `scan_runs`

- `id`
- `indexed_folder_id`
- `started_at`
- `completed_at`，可空
- `status`
- `files_seen`、`warning_count`、`error_count`，默认 0 且非负

### `app_settings`

- `key`，主键
- `value`

Milestone 0 的语言偏好暂存于前端本机存储。`app_settings` 为后续原生设置保留，但不会伪造记录。

### 索引

- files 按 indexed folder 与 normalized path 的唯一索引。
- file events 按 detected at 与 id 的倒序分页索引。
- file events 按 indexed folder 与 detected at 的查询索引。
- scan runs 按 indexed folder 与 started at 的查询索引。

## Tauri 命令契约

### `get_application_info`

返回应用名称、版本、Tauri 运行环境与当前平台，不返回敏感路径。

### `get_database_status`

返回状态、schema version 与安全的状态说明。内部数据库绝对路径不发送到 React。

### `list_indexed_folders`

从 repository 返回真实记录。新数据库返回空数组。

### `query_timeline_page`

接收可选 cursor 与受上限约束的 page size。Milestone 0 返回真实空页：`items: []`、`nextCursor: null`、`hasMore: false`。

所有错误返回 `ApplicationError`，包含稳定 code、用户可理解的双语 message key 与可选 retryable 标记。底层 SQL 或绝对路径不得直接暴露给界面。

## 双语策略

- 使用 `i18next` 与 `react-i18next`。
- 资源键使用语义命名，例如 `timeline.empty.title`，不以英文句子作为键。
- `zh-CN` 与 `en` 资源必须具有相同键集合，并由测试验证。
- 日期、数字与文件大小通过 `Intl` 按当前 locale 格式化。
- 初始语言规则：系统为中文时使用简体中文，否则使用英文。
- 设置页语言切换立即生效，并通过 localStorage 在本机持久化。
- 自动化测试同时覆盖至少一个中文渲染与一个英文切换场景。

## 界面设计系统

设计读取：本地桌面生产力应用，面向重视隐私与工作上下文的用户，视觉应冷静、专业、清晰，不模仿 Finder、Explorer、Notion 或营销网站。

- `DESIGN_VARIANCE: 4`：稳定网格中保留少量非对称留白。
- `MOTION_INTENSITY: 3`：仅用于状态转换、按压与焦点反馈。
- `VISUAL_DENSITY: 5`：适合日常桌面使用，不做空旷展示页或密集驾驶舱。
- 使用原生 CSS、语义 design tokens 与 CSS variables，不引入 Tailwind 或大型设计系统。
- 图标统一使用 `@phosphor-icons/react`，统一笔画粗细，不手绘 SVG。
- 使用系统 sans 字体栈，避免在线字体请求。
- 只使用一套偏冷中性色与低饱和深绿色 accent。
- 固定圆角层级：容器 14px，控件 8px，状态标签可使用 pill。
- 卡片只用于确有层级的状态区域，普通内容优先使用留白与分隔。
- 默认支持浅色与深色主题，并遵循系统偏好。
- 所有动画尊重 `prefers-reduced-motion`。
- 键盘焦点、文本对比度与控件命中区域满足桌面可访问性要求。

`design-taste-frontend` 的营销页、hero、滚动叙事模式不适用于本产品界面。后续实现只采用其中的反模板审美、视觉令牌、交互状态、对比度、动效与验收纪律。

## 页面与状态

### Application shell

左侧固定导航包含 Timeline、Indexed folders、Settings。底部仅显示有语义的 native 与 database 状态。主内容区保持稳定尺寸，切换页面时不跳动。

### Timeline

Milestone 0 显示真实空 timeline。页面解释当前尚无索引活动，并明确 folder selection 将在 Milestone 1 提供。不展示假的搜索结果、事件、统计或图表。未实现的搜索控件不以可点击按钮伪装。

### Indexed folders

显示真实空列表，解释 Chronicle 只会扫描用户明确选择的文件夹，以及清除 Chronicle 数据不会删除原始文件。Milestone 0 不展示可点击但无功能的 Add folder 按钮。

### Settings

提供可工作的中英文切换，展示 local-only、file contents not read 与数据库状态。不得暗示尚未实现的监控或扫描设置可用。

### Loading

使用与最终布局同形的 skeleton，避免只显示居中 spinner。加载期间导航保持可用。

### Empty

说明为什么为空、未来什么操作会产生数据，并严格区分当前里程碑能力。

### Error

错误靠近失败内容显示。数据库失败时明确说明 Chronicle 数据不可用，但原始文件没有被修改。只在确实可重试时显示 retry。

## 错误处理

Rust 将 migration、connection、query、serialization 与 platform 错误转换为稳定 error code。详细错误写入本机开发日志，UI 只显示安全的本地化消息。React 启动请求独立失败，database status 失败不会导致整个窗口白屏。

## 测试策略

### Frontend

- 应用外壳渲染。
- 三个导航入口可切换。
- 新数据库结果显示 timeline 空状态。
- indexed folders 空状态显示。
- 数据库错误显示可理解的本地化信息。
- 中英文资源键一致，语言切换可工作。
- mock 只位于 typed Tauri client 边界，不在组件中伪造业务数据。

### Rust

- migration 在临时 SQLite 数据库成功执行。
- migration 可重复检查且 schema version 正确。
- 新数据库的 indexed folders 查询为空。
- timeline 查询返回真实空分页结果。
- 关键模型可序列化为前端约定的 camelCase JSON。
- typed error 不泄漏数据库路径。

### 视觉验证

- 实际启动 Tauri 窗口并检查 Timeline、Indexed folders、Settings。
- 分别截图中文与英文界面。
- 检查浅色、深色、窄窗口和常规桌面窗口。
- 检查 loading、empty 与 error 状态，不仅检查成功状态。
- 运行 design-taste pre-flight 中适用于产品 UI 的对比度、主题、圆角、动效、图标与 AI tell 检查。

## 文档与自动化

项目创建 README、AGENTS、CONTRIBUTING、CHANGELOG、SECURITY 以及 brief 指定的 docs 文件。文档使用英文作为仓库规范语言，产品可见界面保持中英文双语。

GitHub Actions 分离 frontend 与 Rust 检查。前端执行 formatting、lint、type-check、tests 与 production build。Rust 执行 fmt、clippy、test 与 check。Windows runner 作为首要验证环境。

## 环境评估

已检测：

- `D:\Chronicle` 当前为空，不是 Git repository。
- Node.js `v24.11.1`，npm `11.6.2`，Corepack `0.34.2`。
- pnpm、Yarn 与 Bun 未安装，选择 npm，避免无必要安装 package manager。
- Rust `1.96.0`、Cargo `1.96.0`、rustup `1.29.0`。
- 默认 Rust target 为 `x86_64-pc-windows-msvc`。
- Tauri CLI 尚未安装。
- WebView2 Runtime 已安装，检测到 `149.0.4022.80`。
- 未检测到 Visual Studio C++ Build Tools，当前 PATH 中的 `link.exe` 来自 MSYS2，不是 MSVC linker。

主要风险是缺少 MSVC C++ build tools 与 Windows SDK，Tauri Rust 构建预计会在链接阶段失败。实现阶段应先安装或确认 Visual Studio Build Tools 的 Desktop development with C++ workload，再执行脚手架和构建。Tauri CLI 使用项目本地 npm devDependency，不进行不必要的全局安装。

## Milestone 0 完成标准

- 可启动并可构建的 Tauri 桌面应用。
- SQLite migration 与四个必需命令真实工作。
- 三个页面、中英文切换、系统状态与完整异步状态可见。
- 没有 fake data、analytics、cloud、file-content reads 或越界功能。
- 所有可用质量检查通过。
- Windows 桌面界面经过实际运行、截图与交互检查。

## 明确留给 Milestone 1 的内容

Milestone 1 可实现用户明确授权的 native folder selection、一次性 metadata scan、scan progress、取消与错误恢复，并将真实文件记录写入现有 schema。它不应在 Milestone 0 自动开始。
