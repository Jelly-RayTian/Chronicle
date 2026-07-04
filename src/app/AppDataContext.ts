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
  FileRecord,
  PathHistoryItem,
  ScanRun,
  WatcherStatus,
  VersionFamilySummary,
  VersionFamilyDetail,
  ListVersionFamiliesRequest,
  SuggestVersionFamiliesRequest,
  VersionFamilyRequest,
  VersionFamily,
  RenameVersionFamilyRequest,
  SplitVersionFamilyRequest,
  MergeVersionFamiliesRequest,
  AddVersionFamilyMemberRequest,
  RemoveVersionFamilyMemberRequest,
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
  versionFamilies: Loadable<VersionFamilySummary[]>;
  folderActionError: ApplicationError | null;
  scans: Readonly<Record<number, ScanTaskSnapshot>>;
  watcherStatuses: Readonly<Record<number, WatcherStatus>>;
  addIndexedFolder: () => Promise<FolderRegistration | null>;
  removeIndexedFolder: (folderId: number) => Promise<boolean>;
  startFolderScan: (folderId: number) => Promise<void>;
  cancelFolderScan: (scanRunId: number) => Promise<void>;
  listAllFiles: () => Promise<FileRecord[]>;
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
  listVersionFamilies: (request: ListVersionFamiliesRequest) => Promise<VersionFamilySummary[]>;
  getVersionFamily: (request: VersionFamilyRequest) => Promise<VersionFamilyDetail>;
  suggestVersionFamilies: (request: SuggestVersionFamiliesRequest) => Promise<number>;
  acceptVersionFamily: (request: VersionFamilyRequest) => Promise<VersionFamily>;
  rejectVersionFamily: (request: VersionFamilyRequest) => Promise<VersionFamily>;
  renameVersionFamily: (request: RenameVersionFamilyRequest) => Promise<VersionFamily>;
  splitVersionFamily: (request: SplitVersionFamilyRequest) => Promise<VersionFamilyDetail>;
  mergeVersionFamilies: (request: MergeVersionFamiliesRequest) => Promise<VersionFamilyDetail>;
  addVersionFamilyMember: (request: AddVersionFamilyMemberRequest) => Promise<VersionFamilyDetail>;
  removeVersionFamilyMember: (
    request: RemoveVersionFamilyMemberRequest,
  ) => Promise<VersionFamilyDetail>;
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
