import './SearchPage.css';

import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { ArrowClockwise, File, MagnifyingGlass } from '@phosphor-icons/react';

import { useAppData } from '../app/AppDataContext';
import { EmptyState, LoadingState } from '../components/states/ContentState';
import type { ContentSearchMode, ContentSearchResult } from '../models';

const bytes = (value: number) =>
  value < 1024
    ? `${value} B`
    : value < 1024 ** 2
      ? `${(value / 1024).toFixed(1)} KB`
      : `${(value / 1024 ** 2).toFixed(1)} MB`;

export const SearchPage = () => {
  const { t } = useTranslation();
  const { indexedFolders, searchFiles } = useAppData();
  const [query, setQuery] = useState('');
  const [mode, setMode] = useState<ContentSearchMode>('filename');
  const [folderId, setFolderId] = useState<number | null>(null);
  const [extension, setExtension] = useState('');
  const [dateFrom, setDateFrom] = useState('');
  const [dateTo, setDateTo] = useState('');
  const [presence, setPresence] = useState('');
  const [results, setResults] = useState<ContentSearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasSearched, setHasSearched] = useState(false);

  const readyFolders =
    indexedFolders.state === 'ready'
      ? indexedFolders.data.filter(
          (folder) =>
            folder.availabilityStatus === 'available' &&
            (mode === 'filename' || folder.contentIndexingEnabled),
        )
      : [];

  const handleSearch = async () => {
    setError(null);
    const trimmed = query.trim();
    if (!trimmed) {
      setResults([]);
      return;
    }
    setLoading(true);
    setHasSearched(true);
    try {
      const hits = await searchFiles({
        query: trimmed,
        mode,
        folderId,
        extension: extension.trim() || null,
        dateFrom: dateFrom || null,
        dateTo: dateTo || null,
        presence: (presence || null) as 'present' | 'deleted' | null,
        eventType: null,
        limit: 50,
      });
      setResults(hits);
    } catch (err: unknown) {
      setError(
        typeof err === 'object' && err && 'messageKey' in err
          ? (err as { messageKey: string }).messageKey
          : 'errors.unexpected',
      );
      setResults([]);
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      void handleSearch();
    }
  };

  const reset = () => {
    setQuery('');
    setExtension('');
    setDateFrom('');
    setDateTo('');
    setPresence('');
    setResults([]);
    setHasSearched(false);
    setError(null);
  };

  const activeFilters = [
    extension && `${t('timeline.filters.extension')}: ${extension}`,
    dateFrom && `${t('timeline.filters.from')}: ${dateFrom}`,
    dateTo && `${t('timeline.filters.to')}: ${dateTo}`,
    presence && t(`timeline.presence.${presence}`),
    folderId &&
      indexedFolders.state === 'ready' &&
      indexedFolders.data.find((f) => f.id === folderId)?.displayName,
  ].filter(Boolean) as string[];

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
            className="button button--primary"
            onClick={() => void handleSearch()}
            disabled={loading || !query.trim()}
          >
            <MagnifyingGlass size={18} />
            {t('search.search')}
          </button>
        </div>
        <div className="search-filters">
          <div className="search-mode-group" role="group" aria-label={t('search.mode')}>
            <button
              type="button"
              className={mode === 'filename' ? 'is-active' : ''}
              onClick={() => {
                setMode('filename');
                setFolderId(null);
              }}
              aria-pressed={mode === 'filename'}
            >
              {t('search.filenameMode')}
            </button>
            <button
              type="button"
              className={mode === 'content' ? 'is-active' : ''}
              onClick={() => {
                setMode('content');
                setFolderId(null);
              }}
              aria-pressed={mode === 'content'}
            >
              {t('search.contentMode')}
            </button>
          </div>

          <label>
            <span>{t('search.folder')}</span>
            <select
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
          </label>

          <label>
            <span>{t('timeline.filters.extension')}</span>
            <input
              value={extension}
              onChange={(e) => setExtension(e.target.value)}
              placeholder="pdf"
            />
          </label>

          <label>
            <span>{t('timeline.filters.from')}</span>
            <input type="date" value={dateFrom} onChange={(e) => setDateFrom(e.target.value)} />
          </label>

          <label>
            <span>{t('timeline.filters.to')}</span>
            <input type="date" value={dateTo} onChange={(e) => setDateTo(e.target.value)} />
          </label>

          <label>
            <span>{t('timeline.filters.status')}</span>
            <select value={presence} onChange={(e) => setPresence(e.target.value)}>
              <option value="">{t('timeline.filters.all')}</option>
              <option value="present">{t('timeline.presence.present')}</option>
              <option value="deleted">{t('timeline.presence.deleted')}</option>
            </select>
          </label>
        </div>

        {mode === 'content' && <small>{t('search.contentModeHint')}</small>}
      </section>

      {activeFilters.length > 0 && (
        <div className="active-filters">
          {activeFilters.map((filter) => (
            <span key={filter}>{filter}</span>
          ))}
          <button type="button" onClick={reset}>
            <ArrowClockwise size={15} />
            {t('timeline.filters.reset')}
          </button>
        </div>
      )}

      {error ? (
        <p className="search-error" role="alert">
          {t(error as never)}
        </p>
      ) : null}

      {loading ? <LoadingState /> : null}

      {!loading && results.length > 0 && (
        <>
          <p className="search-results-meta">{t('search.results', { count: results.length })}</p>
          <ul className="search-results-list" aria-label={t('search.resultsLabel')}>
            {results.map((result) => (
              <li key={result.file.id} className="search-result-card">
                <div className="result-head">
                  <File size={17} />
                  <h4>{result.file.name}</h4>
                  <span className="result-size">{bytes(result.file.sizeBytes)}</span>
                </div>
                <p className="result-path">{result.file.normalizedPath}</p>
                <p className="result-meta">
                  <span>{new Date(result.file.filesystemModifiedAt).toLocaleDateString()}</span>
                  {result.rank > 0 && (
                    <span className="result-rank">{Math.round(result.rank * 100)}% match</span>
                  )}
                  {!result.file.isPresent && (
                    <span className="result-deleted">{t('timeline.presence.deleted')}</span>
                  )}
                </p>
                {mode === 'content' && result.snippet && (
                  <p className="result-snippet">{result.snippet}</p>
                )}
              </li>
            ))}
          </ul>
        </>
      )}

      {!loading && hasSearched && query.trim() && results.length === 0 && !error && (
        <EmptyState
          icon={<MagnifyingGlass size={28} />}
          title={t('search.noMatchesTitle')}
          body={t('search.noMatchesBody')}
        />
      )}

      {!hasSearched && !loading && (
        <EmptyState
          icon={<MagnifyingGlass size={28} />}
          title={t('search.emptyTitle')}
          body={t('search.emptyBody')}
        />
      )}
    </div>
  );
};
