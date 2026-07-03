import { createContext, useContext } from 'react';

import type {
  ApplicationError,
  ApplicationInfo,
  DatabaseStatus,
  IndexedFolder,
  FolderRegistration,
  ScanTaskSnapshot,
  TimelinePage,
  TimelineRequest,
  FileEvent,
  PathHistoryItem,
  ScanRun,
  WatcherStatus,
} from '../models';

export type Loadable<T> =
  | { state: 'loading' }
  | { state: 'ready'; data: T }
  | { state: 'error'; error: ApplicationError };

export interface AppData {
  applicationInfo: Loadable<ApplicationInfo>;
  databaseStatus: Loadable<DatabaseStatus>;
  indexedFolders: Loadable<IndexedFolder[]>;
  timeline: Loadable<TimelinePage>;
  folderActionError: ApplicationError | null;
  scans: Readonly<Record<number, ScanTaskSnapshot>>;
  watcherStatuses: Readonly<Record<number, WatcherStatus>>;
  addIndexedFolder: () => Promise<FolderRegistration | null>;
  removeIndexedFolder: (folderId: number) => Promise<boolean>;
  startFolderScan: (folderId: number) => Promise<void>;
  cancelFolderScan: (scanRunId: number) => Promise<void>;
  enableFolderMonitoring: (folderId: number) => Promise<void>;
  disableFolderMonitoring: (folderId: number) => Promise<void>;
  pauseFolderMonitoring: (folderId: number) => Promise<void>;
  resumeFolderMonitoring: (folderId: number) => Promise<void>;
  queryTimeline: (request: TimelineRequest) => Promise<TimelinePage>;
  getFileEventHistory: (fileId: number) => Promise<FileEvent[]>;
  getScanHistory: (folderId: number) => Promise<ScanRun[]>;
  getFilePathHistory: (fileId: number) => Promise<PathHistoryItem[]>;
  confirmEvent: (eventId: number) => Promise<void>;
  rejectEvent: (eventId: number) => Promise<void>;
  openTimelineFile: (fileId: number) => Promise<void>;
  revealTimelineFile: (fileId: number) => Promise<void>;
  clearFolderActionError: () => void;
  reload: () => void;
}

export const AppDataContext = createContext<AppData | null>(null);

export const useAppData = (): AppData => {
  const value = useContext(AppDataContext);
  if (!value) {
    throw new Error('useAppData must be used inside AppProvider.');
  }
  return value;
};
