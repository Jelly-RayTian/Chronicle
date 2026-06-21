# Chronicle Milestone 0 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建一个可安装、隐私优先、本地运行、支持中英文的 Chronicle Tauri 桌面应用基础，并以真实 SQLite 空数据驱动三个核心页面。

**Architecture:** React 负责桌面外壳、本地化和异步状态；唯一的 typed Tauri client 连接薄 command wrapper；Rust service core 负责 SQLite、migration、模型、错误和未来平台能力边界。所有数据库与文件系统能力留在 Rust，Milestone 0 不扫描文件或读取文件内容。

**Tech Stack:** Tauri 2、React 19、TypeScript 6 strict、Vite 8、Rust 1.96、SQLite/rusqlite、Vitest 4、React Testing Library、ESLint、Prettier、i18next、Phosphor Icons。

---

## 文件结构

```text
Chronicle/
  .github/workflows/ci.yml
  .gitignore
  .prettierignore
  .prettierrc.json
  AGENTS.md
  CHANGELOG.md
  CONTRIBUTING.md
  README.md
  SECURITY.md
  eslint.config.js
  index.html
  package-lock.json
  package.json
  tsconfig.app.json
  tsconfig.json
  tsconfig.node.json
  vite.config.ts
  src/
    app/App.tsx
    app/App.test.tsx
    app/AppContext.tsx
    app/navigation.ts
    components/layout/AppShell.tsx
    components/layout/AppShell.css
    components/states/ContentState.tsx
    components/states/ContentState.css
    i18n/en.ts
    i18n/index.ts
    i18n/resources.test.ts
    i18n/zh-CN.ts
    lib/tauri/client.ts
    lib/tauri/client.test.ts
    models/index.ts
    pages/IndexedFoldersPage.tsx
    pages/SettingsPage.tsx
    pages/TimelinePage.tsx
    styles/global.css
    styles/tokens.css
    test/setup.ts
    main.tsx
    vite-env.d.ts
  src-tauri/
    build.rs
    capabilities/default.json
    Cargo.lock
    Cargo.toml
    migrations/V1__initial.sql
    src/commands/application.rs
    src/commands/database.rs
    src/commands/folders.rs
    src/commands/mod.rs
    src/commands/timeline.rs
    src/database/migrations.rs
    src/database/mod.rs
    src/database/repository.rs
    src/errors.rs
    src/events.rs
    src/lib.rs
    src/main.rs
    src/models.rs
    src/platform.rs
    src/scanner.rs
    src/tasks.rs
    tauri.conf.json
  docs/
    architecture.md
    database.md
    event-model.md
    platform-limitations.md
    privacy.md
    product.md
    roadmap.md
    testing.md
```

## Task 1: 工具链门禁与仓库基础

**Files:**

- Create: `.gitignore`
- Create: `.prettierignore`
- Create: `.prettierrc.json`

- [ ] **Step 1: 记录并验证工具链**

Run:

```powershell
node --version
npm --version
rustc --version
cargo --version
rustup show
where.exe cl
where.exe link
```

Expected: Node 24、npm 11、Rust/Cargo 1.96 可用；Rust target 为 `x86_64-pc-windows-msvc`。若 `cl.exe` 不存在，先安装 Visual Studio Build Tools 的 `Desktop development with C++` workload 和 Windows SDK，再继续 Rust build。

- [ ] **Step 2: 建立忽略与格式化规则**

`.gitignore`:

```gitignore
node_modules/
dist/
coverage/
src-tauri/target/
.superpowers/
*.log
.DS_Store
Thumbs.db
```

`.prettierignore`:

```text
dist
coverage
node_modules
src-tauri/target
src-tauri/Cargo.lock
package-lock.json
```

`.prettierrc.json`:

```json
{
  "semi": true,
  "singleQuote": true,
  "trailingComma": "all",
  "printWidth": 100
}
```

- [ ] **Step 3: 验证 UTF-8 与 Git 状态**

Run:

```powershell
Get-Content -Raw -Encoding UTF8 .prettierrc.json | ConvertFrom-Json | Out-Null
git status --short
```

Expected: JSON 可解析；`.superpowers/` 不再出现在 Git 状态中。

- [ ] **Step 4: 提交仓库基础**

```powershell
git add .gitignore .prettierignore .prettierrc.json
git -c user.name=Codex -c user.email=codex@local commit -m "chore: add repository hygiene"
```

## Task 2: 初始化 npm、Vite、React 与质量工具

**Files:**

- Create: `package.json`
- Create: `package-lock.json`
- Create: `index.html`
- Create: `tsconfig.json`
- Create: `tsconfig.app.json`
- Create: `tsconfig.node.json`
- Create: `vite.config.ts`
- Create: `eslint.config.js`
- Create: `src/vite-env.d.ts`
- Create: `src/test/setup.ts`
- Create: `src/main.tsx`

- [ ] **Step 1: 创建明确的 package manifest**

`package.json` scripts:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc -b && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "typecheck": "tsc -b --pretty false",
    "lint": "eslint . --max-warnings 0",
    "format": "prettier --write .",
    "format:check": "prettier --check .",
    "test": "vitest run",
    "test:watch": "vitest"
  }
}
```

Production dependencies: React/React DOM 提供 UI；`@tauri-apps/api` 提供 native invoke；`i18next` 与 `react-i18next` 提供双语；`@phosphor-icons/react` 提供统一图标。

Development dependencies: 本地 `@tauri-apps/cli`、Vite React plugin、TypeScript、Vitest、jsdom、Testing Library、user-event、ESLint、typescript-eslint、React hooks/refresh plugins、Prettier。

- [ ] **Step 2: 安装项目依赖并锁定版本**

Run:

```powershell
npm install react react-dom @tauri-apps/api i18next react-i18next @phosphor-icons/react
npm install --save-dev @tauri-apps/cli @vitejs/plugin-react vite typescript vitest jsdom @testing-library/react @testing-library/jest-dom @testing-library/user-event eslint @eslint/js typescript-eslint eslint-plugin-react-hooks eslint-plugin-react-refresh prettier @types/react @types/react-dom
```

Expected: `package-lock.json` 创建；没有全局 package 安装。

- [ ] **Step 3: 启用 strict TypeScript**

`tsconfig.app.json` 必须包含：

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "useDefineForClassFields": true,
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "allowJs": false,
    "skipLibCheck": true,
    "esModuleInterop": true,
    "allowSyntheticDefaultImports": true,
    "strict": true,
    "forceConsistentCasingInFileNames": true,
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true
  },
  "include": ["src"]
}
```

- [ ] **Step 4: 配置 Vite 与 Vitest**

`vite.config.ts` 使用 React plugin，Vitest 设置为 `jsdom`、加载 `src/test/setup.ts`、启用 CSS，并把测试文件从 production build 输入中隔离。`src/test/setup.ts` 导入 `@testing-library/jest-dom/vitest`。

- [ ] **Step 5: 配置 ESLint flat config**

`eslint.config.js` 应组合 `@eslint/js`、`typescript-eslint`、React hooks 与 refresh rules，忽略 `dist`、`coverage`、`src-tauri/target`，并对 TypeScript 文件禁止显式 `any`。

- [ ] **Step 6: 运行空项目质量门禁**

Run:

```powershell
npm run typecheck
npm run lint
npm run format:check
```

Expected: 全部 exit 0。

- [ ] **Step 7: 提交前端工具链**

```powershell
git add package.json package-lock.json index.html tsconfig*.json vite.config.ts eslint.config.js src/main.tsx src/vite-env.d.ts src/test/setup.ts
git -c user.name=Codex -c user.email=codex@local commit -m "build: configure React and frontend quality tools"
```

## Task 3: 先定义 TypeScript 模型与 typed Tauri client

**Files:**

- Create: `src/models/index.ts`
- Create: `src/lib/tauri/client.ts`
- Create: `src/lib/tauri/client.test.ts`

- [ ] **Step 1: 写 client 的失败测试**

测试注入假的 invoke function，并断言：

```ts
expect(invoke).toHaveBeenCalledWith('query_timeline_page', {
  request: { cursor: null, pageSize: 50 },
});
```

另一个测试让 invoke reject typed payload，断言 `toApplicationError` 产生稳定的 `ApplicationError`，而不是抛出未知对象。

- [ ] **Step 2: 运行测试确认失败**

Run:

```powershell
npm test -- src/lib/tauri/client.test.ts
```

Expected: FAIL，因为 client 与模型尚不存在。

- [ ] **Step 3: 创建无 any 的共享模型**

`src/models/index.ts` 定义并导出：

```ts
export type Locale = 'zh-CN' | 'en';
export type AvailabilityStatus = 'available' | 'missing' | 'unavailable';
export type TaskStatus = 'idle' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface IndexedFolder {
  id: number;
  normalizedPath: string;
  displayName: string;
  addedAt: string;
  lastSuccessfulScanAt: string | null;
  monitoringEnabled: boolean;
  availabilityStatus: AvailabilityStatus;
}
export interface FileRecord {
  id: number;
  indexedFolderId: number;
  normalizedPath: string;
  name: string;
  extension: string | null;
  sizeBytes: number;
  filesystemCreatedAt: string | null;
  filesystemModifiedAt: string;
  firstIndexedAt: string;
  lastSeenAt: string;
  isPresent: boolean;
}
export interface FileEvent {
  id: number;
  fileId: number | null;
  indexedFolderId: number;
  eventType: string;
  detectedAt: string;
  filesystemTime: string | null;
  oldPath: string | null;
  newPath: string | null;
  confidence: number | null;
  eventSource: string;
}
export interface ScanRun {
  id: number;
  indexedFolderId: number;
  startedAt: string;
  completedAt: string | null;
  status: TaskStatus;
  filesSeen: number;
  warningCount: number;
  errorCount: number;
}
export interface TimelinePage {
  items: FileEvent[];
  nextCursor: number | null;
  hasMore: boolean;
}
export interface ApplicationInfo {
  name: string;
  version: string;
  platform: string;
}
export interface DatabaseStatus {
  state: 'ready' | 'error';
  schemaVersion: number;
}
export interface ApplicationError {
  code: string;
  messageKey: string;
  retryable: boolean;
}
```

- [ ] **Step 4: 实现唯一 invoke 边界**

`TauriClient` 暴露 `getApplicationInfo`、`getDatabaseStatus`、`listIndexedFolders`、`queryTimelinePage`。默认实现只在本文件导入 `invoke`。测试通过 `createTauriClient(invokeFn)` 注入 mock。

- [ ] **Step 5: 运行 client 测试与 typecheck**

Run:

```powershell
npm test -- src/lib/tauri/client.test.ts
npm run typecheck
```

Expected: PASS。

- [ ] **Step 6: 提交 typed boundary**

```powershell
git add src/models src/lib/tauri
git -c user.name=Codex -c user.email=codex@local commit -m "feat: add typed Tauri client boundary"
```

## Task 4: 双语资源与本地语言偏好

**Files:**

- Create: `src/i18n/en.ts`
- Create: `src/i18n/zh-CN.ts`
- Create: `src/i18n/index.ts`
- Create: `src/i18n/resources.test.ts`

- [ ] **Step 1: 写资源完整性失败测试**

测试递归收集 `en` 与 `zh-CN` 的 leaf key，断言两个排序后的 key 列表完全相等；再断言系统中文映射为 `zh-CN`，其他语言映射为 `en`。

- [ ] **Step 2: 运行测试确认失败**

Run: `npm test -- src/i18n/resources.test.ts`

Expected: FAIL，因为资源文件不存在。

- [ ] **Step 3: 建立完整中英文资源**

资源至少包含：app name、三个导航标签、native/database status、Timeline/Indexed folders/Settings 标题、loading/empty/error 文案、隐私说明、语言名称与数据库失败消息。禁止用英文句子作为 key。

- [ ] **Step 4: 初始化 i18next**

使用 `localStorage` key `chronicle.locale`；无偏好时 `navigator.language.startsWith('zh')` 选择中文，否则英文。设置 `fallbackLng: 'en'`、`interpolation.escapeValue: false`。

- [ ] **Step 5: 运行资源测试**

Run: `npm test -- src/i18n/resources.test.ts`

Expected: PASS，资源 key 集合相同。

- [ ] **Step 6: 提交双语基础**

```powershell
git add src/i18n
git -c user.name=Codex -c user.email=codex@local commit -m "feat: add Chinese and English localization"
```

## Task 5: 用测试驱动 React 应用外壳与完整状态

**Files:**

- Create: `src/app/App.test.tsx`
- Create: `src/app/App.tsx`
- Create: `src/app/AppContext.tsx`
- Create: `src/app/navigation.ts`
- Create: `src/components/layout/AppShell.tsx`
- Create: `src/components/states/ContentState.tsx`
- Create: `src/pages/TimelinePage.tsx`
- Create: `src/pages/IndexedFoldersPage.tsx`
- Create: `src/pages/SettingsPage.tsx`

- [ ] **Step 1: 写五个必需 frontend 失败测试**

使用注入的 fake `TauriClient`，分别验证：

```ts
it('renders the application shell');
it('navigates between all three pages');
it('shows the real empty timeline state');
it('shows the empty indexed-folder state');
it('shows an understandable database error without implying original-file damage');
```

再增加语言测试：点击 English 后 Timeline、Indexed folders、Settings 使用英文；切换简体中文后显示中文。

- [ ] **Step 2: 运行 App 测试确认失败**

Run: `npm test -- src/app/App.test.tsx`

Expected: FAIL，因为 App 尚未实现。

- [ ] **Step 3: 实现独立启动状态**

`AppContext` 使用 `Promise.allSettled` 或独立请求状态加载 application info、database status、folders 与 timeline。一个请求失败不得让窗口白屏。状态 union 必须为 `loading | ready | error`，不使用多个互相矛盾的 boolean。

- [ ] **Step 4: 实现页面与导航**

导航使用明确的 button/tab semantics 与 `aria-current`。Milestone 0 不渲染无功能的 Add folder、Scan 或 Search 按钮。Settings 中语言切换必须真实工作。

- [ ] **Step 5: 实现 reusable states**

`ContentState` 提供同形 skeleton、contextual empty state 与 inline error。数据库错误文字必须包含“原始文件未被修改”及英文等价文案。

- [ ] **Step 6: 运行 App 测试**

Run:

```powershell
npm test -- src/app/App.test.tsx
npm run typecheck
```

Expected: 全部 PASS。

- [ ] **Step 7: 提交应用行为**

```powershell
git add src/app src/components src/pages src/main.tsx
git -c user.name=Codex -c user.email=codex@local commit -m "feat: add bilingual Chronicle application shell"
```

## Task 6: 实现原创桌面视觉系统

**Files:**

- Create: `src/styles/tokens.css`
- Create: `src/styles/global.css`
- Create: `src/components/layout/AppShell.css`
- Create: `src/components/states/ContentState.css`

- [ ] **Step 1: 锁定设计参数**

Design read: calm local desktop product for privacy-conscious users. `DESIGN_VARIANCE=4`、`MOTION_INTENSITY=3`、`VISUAL_DENSITY=5`。

定义 light/dark semantic tokens：canvas、surface、surface-muted、text、text-muted、border、accent、accent-contrast、danger、focus-ring、shadow。只使用一个深绿色 accent，不使用 AI purple、glow 或渐变文字。

- [ ] **Step 2: 构建 desktop-first shell**

使用固定左侧导航与自适应主区；最小窗口下缩为紧凑 rail，不模仿 Finder/Explorer。容器圆角 14px，控件 8px，状态 pill 999px。只在真实层级处使用阴影。

- [ ] **Step 3: 补齐交互与可访问性**

所有 button 提供 hover、active、focus-visible、disabled；命中区域至少 36px；文字对比达到 WCAG AA；motion 只使用 opacity/transform，并在 `prefers-reduced-motion` 下关闭。

- [ ] **Step 4: 运行 design-taste pre-flight 的适用项目**

检查：单一 accent、主题一致、圆角一致、按钮与表单对比、没有手绘 SVG、没有 fake data、没有装饰性 dashboard、loading/empty/error 齐全、浅色/深色均可读。

- [ ] **Step 5: 运行前端检查并提交**

```powershell
npm run format
npm run lint
npm run typecheck
npm test
git add src/styles src/components
git -c user.name=Codex -c user.email=codex@local commit -m "style: create Chronicle desktop design system"
```

## Task 7: 初始化 Tauri 2 与 Rust 模块边界

**Files:**

- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/errors.rs`
- Create: `src-tauri/src/models.rs`
- Create: `src-tauri/src/scanner.rs`
- Create: `src-tauri/src/events.rs`
- Create: `src-tauri/src/tasks.rs`
- Create: `src-tauri/src/platform.rs`

- [ ] **Step 1: 创建 Rust manifest**

Dependencies:

```toml
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
rusqlite = { version = "0.37", features = ["bundled"] }
rusqlite_migration = "2"
```

Dev dependency: `tempfile = "3"`。Build dependency: `tauri-build = { version = "2", features = [] }`。

- [ ] **Step 2: 写模型序列化失败测试**

在 `models.rs` 测试 `TimelinePage::empty()` 序列化为 camelCase，包含 `items: []`、`nextCursor: null`、`hasMore: false`。

- [ ] **Step 3: 实现 Rust models 与 typed errors**

定义与 TypeScript 一致的 `IndexedFolder`、`FileRecord`、`FileEvent`、`ScanRun`、`TaskStatus`、`ApplicationError`、`TimelinePage`、`ApplicationInfo`、`DatabaseStatus`。所有 DTO 使用 `#[serde(rename_all = "camelCase")]`。

`ApplicationError` 只序列化 `code`、`message_key`、`retryable`，内部 source 不发送到 UI。

- [ ] **Step 4: 创建空行为模块而不是空文件**

`scanner.rs` 定义 future `MetadataScanner` trait；`tasks.rs` 定义 task status contract；`platform.rs` 定义 path normalization contract；`events.rs` 定义 event type enum。不得在 Milestone 0 实际读取文件系统。

- [ ] **Step 5: 运行 Rust 模型测试**

Run: `cargo test --manifest-path src-tauri/Cargo.toml models`

Expected: PASS。

- [ ] **Step 6: 提交 Rust foundation**

```powershell
git add src-tauri
git -c user.name=Codex -c user.email=codex@local commit -m "build: initialize Tauri Rust foundation"
```

## Task 8: 用 migration 与 repository 测试驱动 SQLite

**Files:**

- Create: `src-tauri/migrations/V1__initial.sql`
- Create: `src-tauri/src/database/mod.rs`
- Create: `src-tauri/src/database/migrations.rs`
- Create: `src-tauri/src/database/repository.rs`

- [ ] **Step 1: 写三个失败测试**

```rust
#[test] fn initial_migration_creates_all_tables();
#[test] fn new_database_lists_no_indexed_folders();
#[test] fn new_database_returns_empty_timeline_page();
```

使用 `tempfile::tempdir()`，不得使用用户真实 app-data 数据库。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml database`

Expected: FAIL，因为 migration/repository 尚不存在。

- [ ] **Step 3: 写 V1 migration**

SQL 创建 `indexed_folders`、`files`、`file_events`、`scan_runs`、`app_settings`，包含 foreign keys、non-negative checks、confidence 0..1 check、bool checks 与规格定义的四组索引。时间保存 UTC RFC3339 TEXT。

- [ ] **Step 4: 实现 Database service**

`Database::open(path)` 创建父目录、打开连接、启用 foreign keys、执行 embedded migrations。`Database::open_in_memory()` 只供测试。连接封装在 `Mutex<Connection>`，lock poisoning 转换为 typed internal error。

- [ ] **Step 5: 实现 repository 查询**

`list_indexed_folders()` 使用显式 column list 并按 `added_at, id` 排序。`query_timeline_page(cursor, page_size)` 把 page size 限制为 1..100，按 event id 倒序 keyset pagination，读取 `page_size + 1` 决定 `hasMore`。

- [ ] **Step 6: 运行数据库测试与 clippy**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml database
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Expected: PASS，无 recoverable-path unwrap/expect。

- [ ] **Step 7: 提交数据库基础**

```powershell
git add src-tauri/migrations src-tauri/src/database
git -c user.name=Codex -c user.email=codex@local commit -m "feat: add versioned SQLite foundation"
```

## Task 9: 实现并测试 Tauri commands

**Files:**

- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/application.rs`
- Create: `src-tauri/src/commands/database.rs`
- Create: `src-tauri/src/commands/folders.rs`
- Create: `src-tauri/src/commands/timeline.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 写 service-level command 测试**

在 command modules 中测试新临时数据库的 `list_indexed_folders_impl` 与 `query_timeline_page_impl`。thin Tauri wrapper 只提取 State 并调用 impl。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test --manifest-path src-tauri/Cargo.toml commands`

Expected: FAIL，因为 commands 尚未实现。

- [ ] **Step 3: 实现四个命令**

- `get_application_info`
- `get_database_status`
- `list_indexed_folders`
- `query_timeline_page`

绝对数据库路径不得进入返回 DTO。`query_timeline_page` 接受 camelCase `TimelineRequest`。

- [ ] **Step 4: 在 Tauri builder 注册 state 与 handler**

setup 阶段使用 `app.path().app_data_dir()` 创建 `chronicle.sqlite3`，打开并 migration，然后 `app.manage(database)`。注册四个 invoke handlers。setup 失败时返回带上下文但不泄密到 UI 的错误。

- [ ] **Step 5: 运行全部 Rust 检查**

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: 全部 exit 0。

- [ ] **Step 6: 提交 native commands**

```powershell
git add src-tauri/src
git -c user.name=Codex -c user.email=codex@local commit -m "feat: expose Chronicle native status commands"
```

## Task 10: 编写项目文档

**Files:**

- Create: `README.md`
- Create: `AGENTS.md`
- Create: `CONTRIBUTING.md`
- Create: `CHANGELOG.md`
- Create: `SECURITY.md`
- Create: `docs/product.md`
- Create: `docs/architecture.md`
- Create: `docs/database.md`
- Create: `docs/event-model.md`
- Create: `docs/privacy.md`
- Create: `docs/platform-limitations.md`
- Create: `docs/roadmap.md`
- Create: `docs/testing.md`

- [ ] **Step 1: 写 README 与贡献入口**

README 必须包含产品定位、Milestone 0 功能、明确不做的功能、prerequisites、npm/cargo 命令、privacy guarantees、文档索引。CONTRIBUTING 包含 branch、format/lint/test、commit 与 PR checklist。AGENTS 说明模块边界、禁用 fake data、禁止 React 访问 DB/FS、Windows UTF-8 与视觉验证要求。

- [ ] **Step 2: 写安全与变更记录**

SECURITY 说明私下报告渠道占位不得伪造邮箱，因此要求通过 repository security advisory；列出敏感路径、日志与依赖风险。CHANGELOG 使用 Keep a Changelog 结构并记录 `Unreleased / Added` 的 Milestone 0 foundation。

- [ ] **Step 3: 写八份架构与产品文档**

每份文档必须明确：

- `product.md`: 用户问题、目标用户、非目标、Milestone 0 acceptance。
- `architecture.md`: React/Tauri/Rust/SQLite 边界与数据流。
- `database.md`: schema、类型、索引、migration、清除数据不删原文件。
- `event-model.md`: created/modified/deleted 的未来语义与本里程碑不检测事件。
- `privacy.md`: explicit folder consent、local-only、no analytics/cloud/content reads。
- `platform-limitations.md`: Windows priority、path semantics、permissions、offline/removable drives。
- `roadmap.md`: M0 完成内容与仅建议的 M1，不自动实现后续里程碑。
- `testing.md`: frontend/Rust/visual/CI commands 与 temporary database policy。

- [ ] **Step 4: 检查隐私用语**

Run:

```powershell
rg -n "explicit|selected folder|local|original files|file contents|analytics|cloud" README.md SECURITY.md docs
```

Expected: 明确覆盖只扫描用户选择文件夹、清除 Chronicle 数据不删除原文件、Milestone 0 不读文件内容、无 analytics/cloud。

- [ ] **Step 5: 提交文档**

```powershell
git add README.md AGENTS.md CONTRIBUTING.md CHANGELOG.md SECURITY.md docs
git -c user.name=Codex -c user.email=codex@local commit -m "docs: document Chronicle foundation and privacy model"
```

## Task 11: GitHub Actions CI

**Files:**

- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: 创建 frontend job**

在 `windows-latest` 上使用 Node 24 与 `npm ci`，依次运行：

```text
npm run format:check
npm run lint
npm run typecheck
npm test
npm run build
```

- [ ] **Step 2: 创建 Rust job**

在 `windows-latest` 安装 stable Rust，并运行：

```text
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 3: 本地解析 workflow 并提交**

使用 PowerShell 读取 UTF-8，确认无 tab 与无未替换 placeholder，然后：

```powershell
git add .github/workflows/ci.yml
git -c user.name=Codex -c user.email=codex@local commit -m "ci: verify frontend and Rust foundation"
```

## Task 12: 全量质量、运行与视觉验收

**Files:**

- Modify: only files required by failures found below
- Create: `outputs/screenshots/chronicle-zh-light.png`
- Create: `outputs/screenshots/chronicle-en-dark.png`
- Create: `outputs/screenshots/chronicle-error.png`

- [ ] **Step 1: 运行 frontend 全量检查**

```powershell
npm run format:check
npm run lint
npm run typecheck
npm test
npm run build
```

Expected: 全部 exit 0。

- [ ] **Step 2: 运行 Rust 全量检查**

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: 全部 exit 0。

- [ ] **Step 3: 启动真实 Tauri 应用**

Run: `npm run tauri dev`

Expected: Windows 桌面窗口启动，database/native 状态来自 Rust commands；新数据库显示真实空 timeline 与空 folders。

- [ ] **Step 4: 视觉与交互检查**

逐项验证：三页导航、中英文立即切换、浅色/深色、常规和窄窗口、键盘 focus、loading、empty、database error。截图保存在 `outputs/screenshots`。不得使用浏览器静态页面替代 Tauri 窗口。

- [ ] **Step 5: 构建可安装包**

Run: `npm run tauri build`

Expected: production frontend 与 Rust binary 构建成功，Windows installer/bundle 位于 `src-tauri/target/release/bundle/`。

- [ ] **Step 6: 最终需求审计**

逐项对照原始 15 组要求与设计规格。每一项必须由文件、测试输出、真实窗口或构建产物证明。若证据缺失，继续修复，不以“无明显错误”作为完成依据。

- [ ] **Step 7: 最终提交**

```powershell
git add -A
git -c user.name=Codex -c user.email=codex@local commit -m "chore: complete Chronicle milestone 0 verification"
git status --short --branch
```

Expected: worktree clean。

## 交付报告结构

最终回复按以下顺序提供：

1. 创建或修改的文件。
2. 新增依赖与用途。
3. 最终架构。
4. 数据库 schema 与改进原因。
5. 实际执行命令。
6. 测试、build 与安装包结果。
7. 剩余 warning 或环境问题。
8. 建议用户手动审阅的内容。
9. 不实施的 Milestone 1 建议。
10. 用初学者语言解释五个最重要文件。
