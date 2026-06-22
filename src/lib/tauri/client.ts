import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

import type {
  ApplicationError,
  ApplicationInfo,
  DatabaseStatus,
  FolderRegistration,
  IndexedFolder,
  TimelinePage,
  TimelineRequest,
  ScanTaskSnapshot,
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
  queryTimelinePage(request: TimelineRequest): Promise<TimelinePage>;
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
  queryTimelinePage: (request) => invokeFunction<TimelinePage>('query_timeline_page', { request }),
});

export const tauriClient = createTauriClient(invoke);
