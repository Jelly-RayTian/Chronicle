import './SearchPage.css';

import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { MagnifyingGlass } from '@phosphor-icons/react';

import { useAppData } from '../app/AppDataContext';
import { EmptyState, ErrorState, LoadingState } from '../components/states/ContentState';
import { toApplicationError } from '../lib/tauri/client';
import type { ContentSearchMode, ContentSearchResult } from '../models';

export const SearchPage = () => {
  const { t } = useTranslation();
  const { indexedFolders, folderActionError, clearFolderActionError, searchFiles } = useAppData();
  const [query, setQuery] = useState('');
  const [mode, setMode] = useState<ContentSearchMode>('filename');
  const [folderId, setFolderId] = useState<number | null>(null);
  const [results, setResults] = useState<ContentSearchResult[]>([]);
  const [loading, setLoading] = useState(false);

  const readyFolders =
    indexedFolders.state === 'ready'
      ? indexedFolders.data.filter((folder) => folder.availabilityStatus === 'available')
      : [];

  const handleSearch = async () => {
    clearFolderActionError();
    const trimmed = query.trim();
    if (!trimmed) {
      setResults([]);
      return;
    }
    setLoading(true);
    try {
      const hits = await searchFiles({
        query: trimmed,
        mode,
        folderId,
        limit: 50,
      });
      setResults(hits);
    } catch (error: unknown) {
      setResults([]);
      clearFolderActionError();
      toApplicationError(error);
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      void handleSearch();
    }
  };

  return (
    <div className="page-view search-page">
      <header className="page-header">
        <h2>{t('search.title')}</h2>
        <p>{t('search.description')}</p>
      </header>
      <section className="search-form" aria-label={t('search.formLabel')}>
        <div className="search-input-row">
          <input
            type="text"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t('search.placeholder')}
            aria-label={t('search.query')}
          />
          <button
            type="button"
            onClick={() => void handleSearch()}
            disabled={loading || !query.trim()}
          >
            <MagnifyingGlass size={18} weight="bold" aria-hidden="true" />
            {t('search.search')}
          </button>
        </div>
        <div className="search-filters">
          <label htmlFor="search-folder">{t('search.folder')}</label>
          <select
            id="search-folder"
            value={folderId ?? ''}
            onChange={(event) =>
              setFolderId(event.target.value === '' ? null : Number(event.target.value))
            }
          >
            <option value="">{t('search.allFolders')}</option>
            {readyFolders.map((folder) => (
              <option key={folder.id} value={folder.id}>
                {folder.displayName}
              </option>
            ))}
          </select>
          <div className="search-mode-group" role="group" aria-label={t('search.mode')}>
            <button
              type="button"
              className={mode === 'filename' ? 'is-active' : ''}
              onClick={() => setMode('filename')}
              aria-pressed={mode === 'filename'}
            >
              {t('search.filenameMode')}
            </button>
            <button
              type="button"
              className={mode === 'content' ? 'is-active' : ''}
              onClick={() => setMode('content')}
              aria-pressed={mode === 'content'}
            >
              {t('search.contentMode')}
            </button>
          </div>
          {mode === 'content' ? <small>{t('search.contentModeHint')}</small> : null}
        </div>
      </section>
      {folderActionError ? (
        <ErrorState error={folderActionError} onRetry={() => void handleSearch()} />
      ) : null}
      {loading ? <LoadingState /> : null}
      {!loading && results.length > 0 ? (
        <>
          <p className="search-results-meta">{t('search.results', { count: results.length })}</p>
          <ul className="search-results-list" aria-label={t('search.resultsLabel')}>
            {results.map((result) => (
              <li key={result.file.id} className="search-result-card">
                <h4>{result.file.name}</h4>
                <p className="result-path">{result.file.normalizedPath}</p>
                {mode === 'content' && result.snippet ? (
                  <p className="result-snippet">{result.snippet}</p>
                ) : null}
              </li>
            ))}
          </ul>
        </>
      ) : null}
      {!loading && query.trim() && results.length === 0 && !folderActionError ? (
        <EmptyState
          icon={<MagnifyingGlass size={28} />}
          title={t('search.noMatchesTitle')}
          body={t('search.noMatchesBody')}
        />
      ) : null}
      {!query.trim() && !loading && !folderActionError ? (
        <EmptyState
          icon={<MagnifyingGlass size={28} />}
          title={t('search.emptyTitle')}
          body={t('search.emptyBody')}
        />
      ) : null}
    </div>
  );
};
