import { invoke } from '@tauri-apps/api/core';

import type {
  ApplicationError,
  ApplicationInfo,
  DatabaseStatus,
  IndexedFolder,
  TimelinePage,
  TimelineRequest,
} from '../../models';

export type InvokeFunction = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

export interface TauriClient {
  getApplicationInfo(): Promise<ApplicationInfo>;
  getDatabaseStatus(): Promise<DatabaseStatus>;
  listIndexedFolders(): Promise<IndexedFolder[]>;
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

export const createTauriClient = (invokeFunction: InvokeFunction): TauriClient => ({
  getApplicationInfo: () => invokeFunction<ApplicationInfo>('get_application_info'),
  getDatabaseStatus: () => invokeFunction<DatabaseStatus>('get_database_status'),
  listIndexedFolders: () => invokeFunction<IndexedFolder[]>('list_indexed_folders'),
  queryTimelinePage: (request) => invokeFunction<TimelinePage>('query_timeline_page', { request }),
});

export const tauriClient = createTauriClient(invoke);
