import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

import type {
  ApplicationError,
  ApplicationInfo,
  DatabaseStatus,
  FolderRegistration,
  IndexedFolder,
  FileEvent,
  FileRecord,
  PathHistoryItem,
  ScanRun,
  TimelinePage,
  TimelineRequest,
  ScanTaskSnapshot,
  WatcherStatus,
  VersionFamilySummary,
  VersionFamilyDetail,
  ListVersionFamiliesRequest,
  SuggestVersionFamiliesRequest,
  SuggestVersionFamiliesResponse,
  VersionFamilyRequest,
  VersionFamily,
  RenameVersionFamilyRequest,
  SplitVersionFamilyRequest,
  MergeVersionFamiliesRequest,
  AddVersionFamilyMemberRequest,
  RemoveVersionFamilyMemberRequest,
} from '../../models';

export type InvokeFunction = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;
export type SelectDirectoryFunction = () => Promise<string | null>;

export interface TauriClient {
  getApplicationInfo(): Promise<ApplicationInfo>;
  getDatabaseStatus(): Promise<DatabaseStatus>;
  listIndexedFolders(): Promise<IndexedFolder[]>;
  selectIndexedFolder(): Promise<string | null>;
  registerIndexedFolder(path: string): Promise<FolderRegistration>;
  removeIndexedFolder(folderId: number): Promise<void>;
  startFolderScan(folderId: number): Promise<ScanTaskSnapshot>;
  getScanTask(scanRunId: number): Promise<ScanTaskSnapshot>;
  cancelFolderScan(scanRunId: number): Promise<void>;
  listAllFiles(): Promise<FileRecord[]>;
  listMonitoringStatuses(): Promise<WatcherStatus[]>;
  enableFolderMonitoring(folderId: number, coalescingWindowMs?: number): Promise<WatcherStatus>;
  disableFolderMonitoring(folderId: number): Promise<WatcherStatus>;
  pauseFolderMonitoring(folderId: number): Promise<WatcherStatus>;
  resumeFolderMonitoring(folderId: number): Promise<WatcherStatus>;
  queryTimelinePage(request: TimelineRequest): Promise<TimelinePage>;
  getFileEventHistory(fileId: number): Promise<FileEvent[]>;
  getScanHistory(folderId: number): Promise<ScanRun[]>;
  getFilePathHistory(fileId: number): Promise<PathHistoryItem[]>;
  confirmEvent(eventId: number): Promise<void>;
  rejectEvent(eventId: number): Promise<void>;
  openTimelineFile(fileId: number): Promise<void>;
  revealTimelineFile(fileId: number): Promise<void>;
  listVersionFamilies(request: ListVersionFamiliesRequest): Promise<VersionFamilySummary[]>;
  getVersionFamily(request: VersionFamilyRequest): Promise<VersionFamilyDetail>;
  suggestVersionFamilies(
    request: SuggestVersionFamiliesRequest,
  ): Promise<SuggestVersionFamiliesResponse>;
  acceptVersionFamily(request: VersionFamilyRequest): Promise<VersionFamily>;
  rejectVersionFamily(request: VersionFamilyRequest): Promise<VersionFamily>;
  renameVersionFamily(request: RenameVersionFamilyRequest): Promise<VersionFamily>;
  splitVersionFamily(request: SplitVersionFamilyRequest): Promise<VersionFamilyDetail>;
  mergeVersionFamilies(request: MergeVersionFamiliesRequest): Promise<VersionFamilyDetail>;
  addVersionFamilyMember(request: AddVersionFamilyMemberRequest): Promise<VersionFamilyDetail>;
  removeVersionFamilyMember(
    request: RemoveVersionFamilyMemberRequest,
  ): Promise<VersionFamilyDetail>;
}

const isApplicationError = (value: unknown): value is ApplicationError => {
  if (typeof value !== 'object' || value === null) {
    return false;
  }

  const candidate = value as Record<string, unknown>;
  return (
    typeof candidate.code === 'string' &&
    typeof candidate.messageKey === 'string' &&
    typeof candidate.retryable === 'boolean'
  );
};

export const toApplicationError = (value: unknown): ApplicationError => {
  if (isApplicationError(value)) {
    return value;
  }

  return {
    code: 'unexpected_error',
    messageKey: 'errors.unexpected',
    retryable: false,
  };
};

const selectDirectory: SelectDirectoryFunction = async () => {
  const result = await open({
    directory: true,
    multiple: false,
    title: 'Choose a folder to index',
  });
  return typeof result === 'string' ? result : null;
};

export const createTauriClient = (
  invokeFunction: InvokeFunction,
  selectDirectoryFunction: SelectDirectoryFunction = selectDirectory,
): TauriClient => ({
  getApplicationInfo: () => invokeFunction<ApplicationInfo>('get_application_info'),
  getDatabaseStatus: () => invokeFunction<DatabaseStatus>('get_database_status'),
  listIndexedFolders: () => invokeFunction<IndexedFolder[]>('list_indexed_folders'),
  selectIndexedFolder: selectDirectoryFunction,
  registerIndexedFolder: (path) =>
    invokeFunction<FolderRegistration>('register_indexed_folder', { path }),
  removeIndexedFolder: (folderId) =>
    invokeFunction<void>('remove_indexed_folder', {
      request: { folderId, confirmOriginalFilesUntouched: true },
    }),
  startFolderScan: (folderId) =>
    invokeFunction<ScanTaskSnapshot>('start_folder_scan', { folderId }),
  getScanTask: (scanRunId) => invokeFunction<ScanTaskSnapshot>('get_scan_task', { scanRunId }),
  cancelFolderScan: (scanRunId) => invokeFunction<void>('cancel_folder_scan', { scanRunId }),
  listAllFiles: () => invokeFunction<FileRecord[]>('list_all_files'),
  listMonitoringStatuses: () => invokeFunction<WatcherStatus[]>('list_monitoring_statuses'),
  enableFolderMonitoring: (folderId, coalescingWindowMs = 750) =>
    invokeFunction<WatcherStatus>('enable_folder_monitoring', {
      request: { folderId, coalescingWindowMs },
    }),
  disableFolderMonitoring: (folderId) =>
    invokeFunction<WatcherStatus>('disable_folder_monitoring', {
      request: { folderId, coalescingWindowMs: null },
    }),
  pauseFolderMonitoring: (folderId) =>
    invokeFunction<WatcherStatus>('pause_folder_monitoring', {
      request: { folderId, coalescingWindowMs: null },
    }),
  resumeFolderMonitoring: (folderId) =>
    invokeFunction<WatcherStatus>('resume_folder_monitoring', {
      request: { folderId, coalescingWindowMs: null },
    }),
  queryTimelinePage: (request) => invokeFunction<TimelinePage>('query_timeline_page', { request }),
  getFileEventHistory: (fileId) =>
    invokeFunction<FileEvent[]>('get_file_event_history', { request: { fileId, limit: 100 } }),
  getScanHistory: (folderId) =>
    invokeFunction<ScanRun[]>('get_scan_history', { request: { folderId, limit: 50 } }),
  getFilePathHistory: (fileId) =>
    invokeFunction<PathHistoryItem[]>('get_file_path_history', { request: { fileId, limit: 100 } }),
  confirmEvent: (eventId) => invokeFunction<void>('confirm_event', { request: { eventId } }),
  rejectEvent: (eventId) => invokeFunction<void>('reject_event', { request: { eventId } }),
  openTimelineFile: (fileId) => invokeFunction<void>('open_timeline_file', { request: { fileId } }),
  revealTimelineFile: (fileId) =>
    invokeFunction<void>('reveal_timeline_file', { request: { fileId } }),
  listVersionFamilies: (request) =>
    invokeFunction<VersionFamilySummary[]>('list_version_families', { request }),
  getVersionFamily: (request) =>
    invokeFunction<VersionFamilyDetail>('get_version_family', { request }),
  suggestVersionFamilies: (request) =>
    invokeFunction<SuggestVersionFamiliesResponse>('suggest_version_families', { request }),
  acceptVersionFamily: (request) =>
    invokeFunction<VersionFamily>('accept_version_family', { request }),
  rejectVersionFamily: (request) =>
    invokeFunction<VersionFamily>('reject_version_family', { request }),
  renameVersionFamily: (request) =>
    invokeFunction<VersionFamily>('rename_version_family', { request }),
  splitVersionFamily: (request) =>
    invokeFunction<VersionFamilyDetail>('split_version_family', { request }),
  mergeVersionFamilies: (request) =>
    invokeFunction<VersionFamilyDetail>('merge_version_families', { request }),
  addVersionFamilyMember: (request) =>
    invokeFunction<VersionFamilyDetail>('add_version_family_member', { request }),
  removeVersionFamilyMember: (request) =>
    invokeFunction<VersionFamilyDetail>('remove_version_family_member', { request }),
});

export const tauriClient = createTauriClient(invoke);
