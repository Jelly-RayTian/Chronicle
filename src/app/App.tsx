import { useState } from 'react';

import { AppShell } from '../components/layout/AppShell';
import type { TauriClient } from '../lib/tauri/client';
import { IndexedFoldersPage } from '../pages/IndexedFoldersPage';
import { SettingsPage } from '../pages/SettingsPage';
import { TimelinePage } from '../pages/TimelinePage';
import { VersionsPage } from '../pages/VersionsPage';
import { AppProvider } from './AppContext';
import type { PageId } from './navigation';

interface AppProps {
  client?: TauriClient;
}

const CurrentPage = ({ page }: { page: PageId }) => {
  if (page === 'indexed-folders') return <IndexedFoldersPage />;
  if (page === 'versions') return <VersionsPage />;
  if (page === 'settings') return <SettingsPage />;
  return <TimelinePage />;
};

export const App = ({ client }: AppProps) => {
  const [page, setPage] = useState<PageId>('timeline');
  return (
    <AppProvider client={client}>
      <AppShell activePage={page} onNavigate={setPage}>
        <CurrentPage page={page} />
      </AppShell>
    </AppProvider>
  );
};
