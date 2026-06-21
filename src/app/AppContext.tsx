import {
  type ReactNode,
  useCallback,
  useEffect,
  useState,
} from 'react';

import { tauriClient, toApplicationError, type TauriClient } from '../lib/tauri/client';
import type { ApplicationInfo, DatabaseStatus, IndexedFolder, TimelinePage } from '../models';
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

  const reload = useCallback(() => {
    setApplicationInfo(loading);
    setDatabaseStatus(loading);
    setIndexedFolders(loading);
    setTimeline(loading);
    setReloadToken((token) => token + 1);
  }, []);

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

  const value = { applicationInfo, databaseStatus, indexedFolders, timeline, reload };

  return <AppDataContext.Provider value={value}>{children}</AppDataContext.Provider>;
};
