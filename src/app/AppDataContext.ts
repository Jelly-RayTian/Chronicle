import { createContext, useContext } from 'react';

import type {
  AcceptProjectRequest,
  ActivitySession,
  ActivitySessionDetail,
  ActivitySessionSummary,
  AddProjectMemberRequest,
  AddVersionFamilyMemberRequest,
  ApplicationError,
  ApplicationInfo,
  ClearContentIndexRequest,
  ContentSearchResult,
  CreateProjectRequest,
  DatabaseStatus,
  EnableFolderContentIndexingRequest,
  FileEvent,
  FileRecord,
  FolderContentIndexingRequest,
  FolderRegistration,
  GenerateSessionsRequest,
  IndexedFolder,
  ListProjectsRequest,
  ListSessionsRequest,
  ListVersionFamiliesRequest,
  MergeVersionFamiliesRequest,
  PathHistoryItem,
  Project,
  ProjectDetail,
  ProjectRequest,
  ProjectSummary,
  ReindexFolderContentResponse,
  RemoveProjectMemberRequest,
  RemoveVersionFamilyMemberRequest,
  RenameVersionFamilyRequest,
  RejectProjectRequest,
  ScanRun,
  ScanTaskSnapshot,
  SearchFilesRequest,
  SessionRequest,
  SplitVersionFamilyRequest,
  SuggestProjectsRequest,
  SuggestVersionFamiliesRequest,
  TimelineItem,
  TimelinePage,
  TimelineRequest,
  UpdateProjectRequest,
  UpdateSessionRequest,
  VersionFamily,
  VersionFamilyDetail,
  VersionFamilyRequest,
  VersionFamilySummary,
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
  versionFamilies: Loadable<VersionFamilySummary[]>;
  projects: Loadable<ProjectSummary[]>;
  sessions: Loadable<ActivitySessionSummary[]>;
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
  toggleEventFavorite: (eventId: number) => Promise<boolean>;
  listFavoriteEventIds: () => Promise<number[]>;
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
  listProjects: (request: ListProjectsRequest) => Promise<ProjectSummary[]>;
  getProject: (request: ProjectRequest) => Promise<ProjectDetail>;
  createProject: (request: CreateProjectRequest) => Promise<ProjectDetail>;
  updateProject: (request: UpdateProjectRequest) => Promise<Project>;
  acceptProject: (request: AcceptProjectRequest) => Promise<Project>;
  rejectProject: (request: RejectProjectRequest) => Promise<Project>;
  addProjectMember: (request: AddProjectMemberRequest) => Promise<ProjectDetail>;
  removeProjectMember: (request: RemoveProjectMemberRequest) => Promise<ProjectDetail>;
  suggestProjects: (request: SuggestProjectsRequest) => Promise<number>;
  getProjectTimeline: (request: ProjectRequest) => Promise<TimelineItem[]>;
  listSessions: (request: ListSessionsRequest) => Promise<ActivitySessionSummary[]>;
  getSession: (request: SessionRequest) => Promise<ActivitySessionDetail>;
  generateSessions: (request: GenerateSessionsRequest) => Promise<number>;
  updateSession: (request: UpdateSessionRequest) => Promise<ActivitySession>;
  acceptSession: (request: SessionRequest) => Promise<ActivitySession>;
  rejectSession: (request: SessionRequest) => Promise<ActivitySession>;
  enableFolderContentIndexing: (
    request: EnableFolderContentIndexingRequest,
  ) => Promise<IndexedFolder>;
  disableFolderContentIndexing: (request: FolderContentIndexingRequest) => Promise<IndexedFolder>;
  reindexFolderContent: (
    request: FolderContentIndexingRequest,
  ) => Promise<ReindexFolderContentResponse>;
  clearAllContentIndex: (request: ClearContentIndexRequest) => Promise<void>;
  searchFiles: (request: SearchFilesRequest) => Promise<ContentSearchResult[]>;
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
