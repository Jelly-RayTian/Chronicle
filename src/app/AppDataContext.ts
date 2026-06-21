import { createContext, useContext } from 'react';

import type {
  ApplicationError,
  ApplicationInfo,
  DatabaseStatus,
  IndexedFolder,
  TimelinePage,
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
