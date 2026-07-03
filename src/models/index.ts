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
  eventType: 'created' | 'modified' | 'deleted' | 'renamed' | 'moved' | 'likely_renamed' | 'possible_move' | null;
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
