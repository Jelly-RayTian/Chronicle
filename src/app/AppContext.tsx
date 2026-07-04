import { type ReactNode, useCallback, useEffect, useState } from 'react';

import { tauriClient, toApplicationError, type TauriClient } from '../lib/tauri/client';
import type {
  ApplicationError,
  ApplicationInfo,
  DatabaseStatus,
  FolderRegistration,
  IndexedFolder,
  ScanTaskSnapshot,
  TimelinePage,
  TimelineRequest,
  VersionFamilySummary,
  WatcherStatus,
} from '../models';
import { AppDataContext, type Loadable } from './AppDataContext';

const loading = { state: 'loading' } as const;
interface AppProviderProps {
  children: ReactNode;
  client?: TauriClient | undefined;
}

export const AppProvider = ({ children, client = tauriClient }: AppProviderProps) => {
  const [applicationInfo, setApplicationInfo] = useState<Loadable<ApplicationInfo>>(loading);
  const [databaseStatus, setDatabaseStatus] = useState<Loadable<DatabaseStatus>>(loading);
  const [indexedFolders, setIndexedFolders] = useState<Loadable<IndexedFolder[]>>(loading);
  const [timeline, setTimeline] = useState<Loadable<TimelinePage>>(loading);
  const [reloadToken, setReloadToken] = useState(0);
  const [folderActionError, setFolderActionError] = useState<ApplicationError | null>(null);
  const [scans, setScans] = useState<Record<number, ScanTaskSnapshot>>({});
  const [watcherStatuses, setWatcherStatuses] = useState<Record<number, WatcherStatus>>({});
  const [timelineVersion, setTimelineVersion] = useState(0);
  const [versionFamilies, setVersionFamilies] = useState<Loadable<VersionFamilySummary[]>>(loading);

  const reload = useCallback(() => {
    setApplicationInfo(loading);
    setDatabaseStatus(loading);
    setIndexedFolders(loading);
    setTimeline(loading);
    setWatcherStatuses({});
    setVersionFamilies(loading);
    setReloadToken((token) => token + 1);
  }, []);

  const refreshFolders = useCallback(async () => {
    try {
      const data = await client.listIndexedFolders();
      setIndexedFolders({ state: 'ready', data });
    } catch (error: unknown) {
      setIndexedFolders({ state: 'error', error: toApplicationError(error) });
    }
  }, [client]);

  const refreshMonitoringStatuses = useCallback(async () => {
    const statuses = await client.listMonitoringStatuses();
    setWatcherStatuses(Object.fromEntries(statuses.map((status) => [status.folderId, status])));
    return statuses;
  }, [client]);

  const refreshVersionFamilies = useCallback(async () => {
    try {
      const families = await client.listVersionFamilies({ status: null, folderId: null });
      setVersionFamilies({ state: 'ready', data: families });
    } catch (error: unknown) {
      setVersionFamilies({ state: 'error', error: toApplicationError(error) });
    }
  }, [client]);

  const addIndexedFolder = useCallback(async (): Promise<FolderRegistration | null> => {
    setFolderActionError(null);
    try {
      const path = await client.selectIndexedFolder();
      if (path === null) return null;
      const registration = await client.registerIndexedFolder(path);
      await refreshFolders();
      await refreshMonitoringStatuses();
      return registration;
    } catch (error: unknown) {
      setFolderActionError(toApplicationError(error));
      return null;
    }
  }, [client, refreshFolders, refreshMonitoringStatuses]);

  const removeIndexedFolder = useCallback(
    async (folderId: number): Promise<boolean> => {
      setFolderActionError(null);
      try {
        await client.removeIndexedFolder(folderId);
        setScans((current) => {
          const next = { ...current };
          delete next[folderId];
          return next;
        });
        setWatcherStatuses((current) => {
          const next = { ...current };
          delete next[folderId];
          return next;
        });
        await refreshFolders();
        await refreshMonitoringStatuses();
        return true;
      } catch (error: unknown) {
        setFolderActionError(toApplicationError(error));
        return false;
      }
    },
    [client, refreshFolders, refreshMonitoringStatuses],
  );

  const startFolderScan = useCallback(
    async (folderId: number) => {
      setFolderActionError(null);
      try {
        const scan = await client.startFolderScan(folderId);
        setScans((current) => ({ ...current, [folderId]: scan }));
      } catch (error: unknown) {
        setFolderActionError(toApplicationError(error));
      }
    },
    [client],
  );

  const cancelFolderScan = useCallback(
    async (scanRunId: number) => {
      setFolderActionError(null);
      try {
        await client.cancelFolderScan(scanRunId);
      } catch (error: unknown) {
        setFolderActionError(toApplicationError(error));
      }
    },
    [client],
  );

  const listAllFiles = useCallback(() => client.listAllFiles(), [client]);

  const applyWatcherAction = useCallback(
    async (action: () => Promise<WatcherStatus>) => {
      setFolderActionError(null);
      try {
        const status = await action();
        setWatcherStatuses((current) => ({ ...current, [status.folderId]: status }));
        await refreshFolders();
      } catch (error: unknown) {
        setFolderActionError(toApplicationError(error));
      }
    },
    [refreshFolders],
  );

  const enableFolderMonitoring = useCallback(
    (folderId: number) => applyWatcherAction(() => client.enableFolderMonitoring(folderId)),
    [applyWatcherAction, client],
  );
  const disableFolderMonitoring = useCallback(
    (folderId: number) => applyWatcherAction(() => client.disableFolderMonitoring(folderId)),
    [applyWatcherAction, client],
  );
  const pauseFolderMonitoring = useCallback(
    (folderId: number) => applyWatcherAction(() => client.pauseFolderMonitoring(folderId)),
    [applyWatcherAction, client],
  );
  const resumeFolderMonitoring = useCallback(
    (folderId: number) => applyWatcherAction(() => client.resumeFolderMonitoring(folderId)),
    [applyWatcherAction, client],
  );

  useEffect(() => {
    let active = true;
    const settle = <T,>(promise: Promise<T>, setter: (value: Loadable<T>) => void) => {
      void promise.then(
        (data) => {
          if (active) setter({ state: 'ready', data });
        },
        (error: unknown) => {
          if (active) setter({ state: 'error', error: toApplicationError(error) });
        },
      );
    };

    settle(client.getApplicationInfo(), setApplicationInfo);
    settle(client.getDatabaseStatus(), setDatabaseStatus);
    settle(client.listIndexedFolders(), setIndexedFolders);
    void client.listMonitoringStatuses().then(
      (statuses) => {
        if (active) {
          setWatcherStatuses(
            Object.fromEntries(statuses.map((status) => [status.folderId, status])),
          );
        }
      },
      (error: unknown) => {
        if (active) setFolderActionError(toApplicationError(error));
      },
    );
    settle(
      client.queryTimelinePage({
        cursor: null,
        pageSize: 50,
        filename: null,
        extension: null,
        eventType: null,
        folderId: null,
        dateFrom: null,
        dateTo: null,
        presence: null,
      }),
      setTimeline,
    );
    settle(client.listVersionFamilies({ status: null, folderId: null }), setVersionFamilies);

    return () => {
      active = false;
    };
  }, [client, reloadToken]);

  useEffect(() => {
    const running = Object.values(scans).filter((scan) => scan.status === 'running');
    if (running.length === 0) return undefined;
    const timer = globalThis.setInterval(() => {
      void Promise.all(
        running.map(async (scan) => {
          try {
            const updated = await client.getScanTask(scan.scanRunId);
            setScans((current) => ({ ...current, [updated.indexedFolderId]: updated }));
            if (updated.status !== 'running') {
              await refreshFolders();
              setTimelineVersion((version) => version + 1);
            }
          } catch (error: unknown) {
            setFolderActionError(toApplicationError(error));
          }
        }),
      );
    }, 400);
    return () => globalThis.clearInterval(timer);
  }, [client, refreshFolders, scans]);

  useEffect(() => {
    const activeStatuses = Object.values(watcherStatuses).filter(
      (status) => status.desiredState !== 'disabled',
    );
    if (activeStatuses.length === 0) return undefined;
    const timer = globalThis.setInterval(() => {
      void client.listMonitoringStatuses().then(
        async (statuses) => {
          let changedEvents = false;
          setWatcherStatuses((current) => {
            const next = Object.fromEntries(statuses.map((status) => [status.folderId, status]));
            changedEvents = statuses.some(
              (status) => current[status.folderId]?.eventsRecorded !== status.eventsRecorded,
            );
            return next;
          });
          if (changedEvents) {
            await refreshFolders();
            setTimelineVersion((version) => version + 1);
          }
        },
        (error: unknown) => setFolderActionError(toApplicationError(error)),
      );
    }, 1000);
    return () => globalThis.clearInterval(timer);
  }, [client, refreshFolders, watcherStatuses]);

  const queryTimeline = useCallback(
    (request: TimelineRequest) => {
      void timelineVersion;
      return client.queryTimelinePage(request);
    },
    [client, timelineVersion],
  );
  const getFileEventHistory = useCallback(
    (fileId: number) => client.getFileEventHistory(fileId),
    [client],
  );
  const getScanHistory = useCallback(
    (folderId: number) => client.getScanHistory(folderId),
    [client],
  );
  const getFilePathHistory = useCallback(
    (fileId: number) => client.getFilePathHistory(fileId),
    [client],
  );
  const confirmEvent = useCallback((eventId: number) => client.confirmEvent(eventId), [client]);
  const rejectEvent = useCallback((eventId: number) => client.rejectEvent(eventId), [client]);
  const openTimelineFile = useCallback(
    (fileId: number) => client.openTimelineFile(fileId),
    [client],
  );
  const revealTimelineFile = useCallback(
    (fileId: number) => client.revealTimelineFile(fileId),
    [client],
  );

  const runVersionFamilyAction = useCallback(
    async <T,>(action: () => Promise<T>): Promise<T> => {
      setFolderActionError(null);
      try {
        const result = await action();
        await refreshVersionFamilies();
        return result;
      } catch (error: unknown) {
        setFolderActionError(toApplicationError(error));
        throw error;
      }
    },
    [refreshVersionFamilies],
  );

  const listVersionFamilies = useCallback(
    (request: import('../models').ListVersionFamiliesRequest) =>
      client.listVersionFamilies(request),
    [client],
  );
  const getVersionFamily = useCallback(
    (request: import('../models').VersionFamilyRequest) => client.getVersionFamily(request),
    [client],
  );
  const suggestVersionFamilies = useCallback(
    (request: import('../models').SuggestVersionFamiliesRequest) =>
      runVersionFamilyAction(() =>
        client.suggestVersionFamilies(request).then((response) => response.familiesCreated),
      ),
    [client, runVersionFamilyAction],
  );
  const acceptVersionFamily = useCallback(
    (request: import('../models').VersionFamilyRequest) =>
      runVersionFamilyAction(() => client.acceptVersionFamily(request)),
    [client, runVersionFamilyAction],
  );
  const rejectVersionFamily = useCallback(
    (request: import('../models').VersionFamilyRequest) =>
      runVersionFamilyAction(() => client.rejectVersionFamily(request)),
    [client, runVersionFamilyAction],
  );
  const renameVersionFamily = useCallback(
    (request: import('../models').RenameVersionFamilyRequest) =>
      runVersionFamilyAction(() => client.renameVersionFamily(request)),
    [client, runVersionFamilyAction],
  );
  const splitVersionFamily = useCallback(
    (request: import('../models').SplitVersionFamilyRequest) =>
      runVersionFamilyAction(() => client.splitVersionFamily(request)),
    [client, runVersionFamilyAction],
  );
  const mergeVersionFamilies = useCallback(
    (request: import('../models').MergeVersionFamiliesRequest) =>
      runVersionFamilyAction(() => client.mergeVersionFamilies(request)),
    [client, runVersionFamilyAction],
  );
  const addVersionFamilyMember = useCallback(
    (request: import('../models').AddVersionFamilyMemberRequest) =>
      runVersionFamilyAction(() => client.addVersionFamilyMember(request)),
    [client, runVersionFamilyAction],
  );
  const removeVersionFamilyMember = useCallback(
    (request: import('../models').RemoveVersionFamilyMemberRequest) =>
      runVersionFamilyAction(() => client.removeVersionFamilyMember(request)),
    [client, runVersionFamilyAction],
  );

  const value = {
    applicationInfo,
    databaseStatus,
    indexedFolders,
    timeline,
    versionFamilies,
    folderActionError,
    scans,
    watcherStatuses,
    addIndexedFolder,
    removeIndexedFolder,
    startFolderScan,
    cancelFolderScan,
    listAllFiles,
    enableFolderMonitoring,
    disableFolderMonitoring,
    pauseFolderMonitoring,
    resumeFolderMonitoring,
    queryTimeline,
    getFileEventHistory,
    getScanHistory,
    getFilePathHistory,
    confirmEvent,
    rejectEvent,
    openTimelineFile,
    revealTimelineFile,
    listVersionFamilies,
    getVersionFamily,
    suggestVersionFamilies,
    acceptVersionFamily,
    rejectVersionFamily,
    renameVersionFamily,
    splitVersionFamily,
    mergeVersionFamilies,
    addVersionFamilyMember,
    removeVersionFamilyMember,
    clearFolderActionError: () => setFolderActionError(null),
    reload,
  };

  return <AppDataContext.Provider value={value}>{children}</AppDataContext.Provider>;
};
