import { FolderSimpleDashed } from '@phosphor-icons/react';
import { useTranslation } from 'react-i18next';

import { useAppData } from '../app/AppDataContext';
import { EmptyState, ErrorState, LoadingState } from '../components/states/ContentState';

export const IndexedFoldersPage = () => {
  const { t } = useTranslation();
  const { indexedFolders, reload } = useAppData();

  return (
    <div className="page-view">
      <header className="page-header">
        <div>
          <h2>{t('folders.title')}</h2>
          <p>{t('folders.description')}</p>
        </div>
        {indexedFolders.state === 'ready' ? (
          <span className="count-label">
            {t('folders.count', { count: indexedFolders.data.length })}
          </span>
        ) : null}
      </header>
      {indexedFolders.state === 'loading' ? <LoadingState /> : null}
      {indexedFolders.state === 'error' ? (
        <ErrorState error={indexedFolders.error} onRetry={reload} />
      ) : null}
      {indexedFolders.state === 'ready' && indexedFolders.data.length === 0 ? (
        <EmptyState
          icon={<FolderSimpleDashed size={28} weight="regular" />}
          title={t('folders.emptyTitle')}
          body={t('folders.emptyBody')}
        />
      ) : null}
      {indexedFolders.state === 'ready' && indexedFolders.data.length > 0 ? (
        <ul className="folder-list">
          {indexedFolders.data.map((folder) => (
            <li key={folder.id}>
              <strong>{folder.displayName}</strong>
              <span>{folder.normalizedPath}</span>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
};
