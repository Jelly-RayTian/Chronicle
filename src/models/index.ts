export type Locale = 'zh-CN' | 'en';
export type AvailabilityStatus =
  | 'available'
  | 'missing_or_moved'
  | 'permission_denied'
  | 'inaccessible';
export type TaskStatus = 'idle' | 'running' | 'completed' | 'failed' | 'cancelled';
export type WatcherDesiredState = 'disabled' | 'enabled' | 'paused';
export type WatcherRuntimeState = 'stopped' | 'running' | 'paused' | 'unavailable' | 'error';
export type DatabaseState = 'ready' | 'error';

export interface IndexedFolder {
  id: number;
  normalizedPath: string;
  displayName: string;
  addedAt: string;
  lastSuccessfulScanAt: string | null;
  monitoringEnabled: boolean;
  availabilityStatus: AvailabilityStatus;
  lastCheckedAt: string | null;
  contentIndexingEnabled: boolean;
  contentIndexingExtensions: string;
  contentIndexingMaxBytes: number;
  contentIndexingExclusionPatterns: string;
}

export type NestingRelationship = 'inside_existing' | 'contains_existing';

export interface NestedFolderWarning {
  existingFolderId: number;
  existingPath: string;
  relationship: NestingRelationship;
}

export interface FolderRegistration {
  folder: IndexedFolder;
  nestedWarnings: NestedFolderWarning[];
}

export interface FileRecord {
  id: number;
  indexedFolderId: number;
  normalizedPath: string;
  name: string;
  parentPath: string;
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
  userConfirmation: string | null;
  userConfirmedAt: string | null;
}

export interface PathHistoryItem {
  id: number;
  fileId: number;
  oldPath: string;
  newPath: string;
  validFrom: string;
  validUntil: string | null;
  confidence: number;
  evidence: string;
  eventSource: string;
  detectedAt: string;
}

export type VersionFamilyStatus = 'suggested' | 'confirmed' | 'rejected' | 'superseded';
export type VersionFamilyDecision = 'accepted' | 'rejected' | 'split' | 'merged';

export interface VersionFamily {
  id: number;
  displayName: string;
  status: VersionFamilyStatus;
  userDecision: VersionFamilyDecision | null;
  userDecidedAt: string | null;
  mergedIntoFamilyId: number | null;
  createdAt: string;
  updatedAt: string;
}

export interface VersionFamilyMember {
  id: number;
  versionFamilyId: number;
  fileId: number;
  sortOrder: number;
  addedAt: string;
}

export interface VersionFamilyMemberWithFile {
  member: VersionFamilyMember;
  file: FileRecord;
}

export interface VersionFamilySuggestion {
  id: number;
  versionFamilyId: number;
  confidence: number;
  evidence: string;
  detectedAt: string;
  source: string;
}

export interface VersionFamilyDetail {
  family: VersionFamily;
  members: VersionFamilyMemberWithFile[];
  suggestions: VersionFamilySuggestion[];
}

export interface VersionFamilySummary {
  family: VersionFamily;
  memberCount: number;
  fileIds: number[];
}

export interface TimelineItem {
  event: FileEvent;
  file: FileRecord;
  folderName: string;
  folderPath: string;
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
  failureKind: string | null;
}

export interface ScanTaskSnapshot {
  scanRunId: number;
  indexedFolderId: number;
  status: TaskStatus;
  filesSeen: number;
  warningCount: number;
  errorCount: number;
  cancellable: boolean;
}

export interface WatcherStatus {
  folderId: number;
  desiredState: WatcherDesiredState;
  runtimeState: WatcherRuntimeState;
  coalescingWindowMs: number;
  lastStartedAt: string | null;
  lastStoppedAt: string | null;
  lastEventAt: string | null;
  lastErrorAt: string | null;
  lastErrorKind: string | null;
  lastErrorMessage: string | null;
  eventsRecorded: number;
  eventsDropped: number;
}

export interface TimelinePage {
  items: TimelineItem[];
  nextCursor: number | null;
  hasMore: boolean;
}

export interface TimelineRequest {
  cursor: number | null;
  pageSize: number;
  filename: string | null;
  extension: string | null;
  eventType:
    | 'created'
    | 'modified'
    | 'deleted'
    | 'renamed'
    | 'moved'
    | 'likely_renamed'
    | 'possible_move'
    | null;
  folderId: number | null;
  dateFrom: string | null;
  dateTo: string | null;
  presence: 'present' | 'deleted' | null;
}

export interface PathHistoryRequest {
  fileId: number;
  limit: number;
}

export interface ConfirmEventRequest {
  eventId: number;
}

export interface ListVersionFamiliesRequest {
  status: VersionFamilyStatus | null;
  folderId: number | null;
}

export interface VersionFamilyRequest {
  familyId: number;
}

export interface SuggestVersionFamiliesRequest {
  folderId: number | null;
}

export interface SuggestVersionFamiliesResponse {
  familiesCreated: number;
}

export interface RenameVersionFamilyRequest {
  familyId: number;
  displayName: string;
}

export interface SplitVersionFamilyRequest {
  familyId: number;
  fileIds: number[];
  displayName: string | null;
}

export interface MergeVersionFamiliesRequest {
  targetFamilyId: number;
  sourceFamilyIds: number[];
}

export interface AddVersionFamilyMemberRequest {
  familyId: number;
  fileId: number;
}

export interface RemoveVersionFamilyMemberRequest {
  familyId: number;
  fileId: number;
}

export interface ApplicationInfo {
  name: string;
  version: string;
  platform: string;
}

export interface DatabaseStatus {
  state: DatabaseState;
  schemaVersion: number;
}

export interface ApplicationError {
  code: string;
  messageKey: string;
  retryable: boolean;
}

export type ProjectStatus = 'suggested' | 'active' | 'archived' | 'rejected';
export type ProjectDecision = 'accepted' | 'rejected';
export type ProjectMembershipType = 'manual' | 'suggested';
export type ActivitySessionStatus = 'auto' | 'edited' | 'accepted' | 'rejected';

export interface Project {
  id: number;
  name: string;
  description: string;
  status: ProjectStatus;
  decision: ProjectDecision | null;
  createdAt: string;
  updatedAt: string;
}

export interface ProjectMember {
  id: number;
  projectId: number;
  fileId: number;
  membershipType: ProjectMembershipType;
  addedAt: string;
}

export interface ProjectMemberWithFile {
  member: ProjectMember;
  file: FileRecord;
}

export interface ProjectSuggestion {
  id: number;
  projectId: number;
  fileId: number;
  confidence: number;
  evidence: string;
  source: string;
  handled: boolean;
  suggestedAt: string;
}

export interface ProjectDetail {
  project: Project;
  members: ProjectMemberWithFile[];
  suggestions: ProjectSuggestion[];
}

export interface ProjectSummary {
  project: Project;
  memberCount: number;
  fileIds: number[];
}

export interface ActivitySession {
  id: number;
  projectId: number | null;
  title: string;
  startedAt: string;
  endedAt: string;
  eventSummary: string;
  status: ActivitySessionStatus;
  createdAt: string;
  updatedAt: string;
}

export interface ActivitySessionDetail {
  session: ActivitySession;
  project: Project | null;
  files: FileRecord[];
  events: FileEvent[];
}

export interface ActivitySessionSummary {
  session: ActivitySession;
  eventCount: number;
  fileCount: number;
  projectName: string | null;
}

export interface ProjectRequest {
  projectId: number;
}

export interface CreateProjectRequest {
  name: string;
  description: string | null;
  fileIds: number[];
}

export interface UpdateProjectRequest {
  projectId: number;
  name: string | null;
  description: string | null;
}

export interface ListProjectsRequest {
  status: ProjectStatus | null;
}

export interface AcceptProjectRequest {
  projectId: number;
}

export interface RejectProjectRequest {
  projectId: number;
}

export interface AddProjectMemberRequest {
  projectId: number;
  fileId: number;
}

export interface RemoveProjectMemberRequest {
  projectId: number;
  fileId: number;
}

export interface SuggestProjectsRequest {
  folderId: number | null;
}

export interface SuggestProjectsResponse {
  projectsCreated: number;
}

export interface SessionRequest {
  sessionId: number;
}

export interface UpdateSessionRequest {
  sessionId: number;
  title: string | null;
  projectId: number | null;
}

export interface ListSessionsRequest {
  projectId: number | null;
}

export interface MergeSessionsRequest {
  targetSessionId: number;
  sourceSessionId: number;
}

export interface GenerateSessionsRequest {
  gapMinutes: number | null;
}

export interface GenerateSessionsResponse {
  sessionsCreated: number;
}

export type ContentSearchMode = 'filename' | 'content';

export interface ContentSearchResult {
  file: FileRecord;
  snippet: string;
  rank: number;
}

export interface SearchFilesRequest {
  query: string;
  mode: ContentSearchMode;
  folderId: number | null;
  limit: number;
}

export interface EnableFolderContentIndexingRequest {
  folderId: number;
  extensions: string | null;
  maxBytes: number | null;
  exclusionPatterns: string | null;
}

export interface FolderContentIndexingRequest {
  folderId: number;
}

export type ClearContentIndexRequest = Record<string, never>;

export interface ReindexFolderContentResponse {
  filesIndexed: number;
  filesRemoved: number;
}
