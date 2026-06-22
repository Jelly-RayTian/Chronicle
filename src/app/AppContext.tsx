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

  const reload = useCallback(() => {
    setApplicationInfo(loading);
    setDatabaseStatus(loading);
    setIndexedFolders(loading);
    setTimeline(loading);
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

  const addIndexedFolder = useCallback(async (): Promise<FolderRegistration | null> => {
    setFolderActionError(null);
    try {
      const path = await client.selectIndexedFolder();
      if (path === null) return null;
      const registration = await client.registerIndexedFolder(path);
      await refreshFolders();
      return registration;
    } catch (error: unknown) {
      setFolderActionError(toApplicationError(error));
      return null;
    }
  }, [client, refreshFolders]);

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
        await refreshFolders();
        return true;
      } catch (error: unknown) {
        setFolderActionError(toApplicationError(error));
        return false;
      }
    },
    [client, refreshFolders],
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
    settle(client.queryTimelinePage({ cursor: null, pageSize: 50 }), setTimeline);

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
            if (updated.status !== 'running') await refreshFolders();
          } catch (error: unknown) {
            setFolderActionError(toApplicationError(error));
          }
        }),
      );
    }, 400);
    return () => globalThis.clearInterval(timer);
  }, [client, refreshFolders, scans]);

  const value = {
    applicationInfo,
    databaseStatus,
    indexedFolders,
    timeline,
    folderActionError,
    scans,
    addIndexedFolder,
    removeIndexedFolder,
    startFolderScan,
    cancelFolderScan,
    clearFolderActionError: () => setFolderActionError(null),
    reload,
  };

  return <AppDataContext.Provider value={value}>{children}</AppDataContext.Provider>;
};
