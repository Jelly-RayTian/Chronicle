export type Locale = 'zh-CN' | 'en';
export type AvailabilityStatus = 'available' | 'missing' | 'unavailable';
export type TaskStatus = 'idle' | 'running' | 'completed' | 'failed' | 'cancelled';
export type DatabaseState = 'ready' | 'error';

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

export interface TimelineRequest {
  cursor: number | null;
  pageSize: number;
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
