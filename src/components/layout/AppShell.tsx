import {
  Briefcase,
  ClockCounterClockwise,
  FolderSimple,
  GearSix,
  Hourglass,
  MagnifyingGlass,
  Stack,
} from '@phosphor-icons/react';
import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import { useAppData } from '../../app/AppDataContext';
import { pageTranslationKey, type PageId } from '../../app/navigation';
import { ErrorState } from '../states/ContentState';

interface AppShellProps {
  activePage: PageId;
  onNavigate: (page: PageId) => void;
  children: ReactNode;
}

const navigation = [
  { id: 'timeline', icon: ClockCounterClockwise },
  { id: 'indexed-folders', icon: FolderSimple },
  { id: 'versions', icon: Stack },
  { id: 'projects', icon: Briefcase },
  { id: 'sessions', icon: Hourglass },
  { id: 'search', icon: MagnifyingGlass },
  { id: 'settings', icon: GearSix },
] as const;

export const AppShell = ({ activePage, onNavigate, children }: AppShellProps) => {
  const { t } = useTranslation();
  const { applicationInfo, databaseStatus, reload } = useAppData();
  const nativeReady = applicationInfo.state === 'ready';
  const databaseReady = databaseStatus.state === 'ready';

  return (
    <div className="app-shell">
      <a className="skip-link" href="#main-content">
        {t('app.skipToContent')}
      </a>
      <aside className="sidebar">
        <header className="brand-block">
          <div className="brand-mark" aria-hidden="true">
            C
          </div>
          <div>
            <h1>{t('app.name')}</h1>
            <p>{t('app.tagline')}</p>
          </div>
        </header>
        <nav className="primary-navigation" aria-label={t('app.name')}>
          {navigation.map(({ id, icon: Icon }) => (
            <button
              key={id}
              type="button"
              className={activePage === id ? 'navigation-item is-active' : 'navigation-item'}
              aria-current={activePage === id ? 'page' : undefined}
              aria-label={t(pageTranslationKey[id])}
              onClick={() => onNavigate(id)}
            >
              <Icon size={20} weight={activePage === id ? 'fill' : 'regular'} aria-hidden="true" />
              <span>{t(pageTranslationKey[id])}</span>
            </button>
          ))}
        </nav>
        <section className="system-status" aria-labelledby="system-status-title">
          <h2 id="system-status-title">{t('status.system')}</h2>
          <div className="status-row">
            <span>{t('status.native')}</span>
            <strong data-state={nativeReady ? 'ready' : 'unavailable'}>
              {t(nativeReady ? 'status.ready' : 'status.unavailable')}
            </strong>
          </div>
          <div className="status-row">
            <span>{t('status.database')}</span>
            <strong data-state={databaseReady ? 'ready' : 'unavailable'}>
              {t(databaseReady ? 'status.ready' : 'status.unavailable')}
            </strong>
          </div>
          {databaseStatus.state === 'ready' ? (
            <small>
              {t('status.schemaVersion', { version: databaseStatus.data.schemaVersion })}
            </small>
          ) : null}
        </section>
      </aside>
      <main id="main-content" className="main-content" tabIndex={-1}>
        {databaseStatus.state === 'error' ? (
          <ErrorState error={databaseStatus.error} onRetry={reload} />
        ) : null}
        {children}
      </main>
    </div>
  );
};
