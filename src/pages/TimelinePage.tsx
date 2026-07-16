import {
  ArrowClockwise,
  ClockCounterClockwise,
  Copy,
  File,
  FolderOpen,
  MagnifyingGlass,
  Star,
  X,
} from '@phosphor-icons/react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useAppData } from '../app/AppDataContext';
import { EmptyState, ErrorState, LoadingState } from '../components/states/ContentState';
import type {
  ApplicationError,
  FileEvent,
  PathHistoryItem,
  ScanRun,
  TimelineItem,
  TimelinePage as TimelinePageData,
  TimelineRequest,
} from '../models';
import { timelineDateGroup, type DateGroup } from './timelineGrouping';
import './TimelinePage.css';

const PAGE_SIZE = 30;

const boundary = (value: string, end: boolean) => {
  if (!value) return null;
  const date = new Date(`${value}T00:00:00`);
  if (end) date.setDate(date.getDate() + 1);
  return date.toISOString();
};
const bytes = (value: number) =>
  value < 1024
    ? `${value} B`
    : value < 1024 ** 2
      ? `${(value / 1024).toFixed(1)} KB`
      : `${(value / 1024 ** 2).toFixed(1)} MB`;

const DATE_GROUP_ORDER: DateGroup[] = ['today', 'yesterday', 'thisWeek', 'thisMonth', 'older'];

export const TimelinePage = () => {
  const { t, i18n } = useTranslation();
  const {
    indexedFolders,
    timeline,
    reload,
    queryTimeline,
    getFileEventHistory,
    getScanHistory,
    getFilePathHistory,
    confirmEvent,
    rejectEvent,
    openTimelineFile,
    revealTimelineFile,
    toggleEventFavorite,
    listFavoriteEventIds,
  } = useAppData();
  const searchInputRef = useRef<HTMLInputElement>(null);
  const [search, setSearch] = useState('');
  const [debounced, setDebounced] = useState('');
  const [extension, setExtension] = useState('');
  const [eventType, setEventType] = useState('');
  const [folderId, setFolderId] = useState('');
  const [dateFrom, setDateFrom] = useState('');
  const [dateTo, setDateTo] = useState('');
  const [presence, setPresence] = useState('');
  const [page, setPage] = useState<TimelinePageData | null>(
    timeline.state === 'ready' ? timeline.data : null,
  );
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<ApplicationError | null>(null);
  const [selected, setSelected] = useState<TimelineItem | null>(null);
  const [focusedIndex, setFocusedIndex] = useState(-1);
  const [favorites, setFavorites] = useState<Set<number>>(new Set());
  const [eventHistory, setEventHistory] = useState<FileEvent[]>([]);
  const [scanHistory, setScanHistory] = useState<ScanRun[]>([]);
  const [pathHistory, setPathHistory] = useState<PathHistoryItem[]>([]);
  const [actionError, setActionError] = useState<ApplicationError | null>(null);
  const [pathCopied, setPathCopied] = useState(false);

  useEffect(() => {
    const timer = globalThis.setTimeout(() => setDebounced(search.trim()), 300);
    return () => globalThis.clearTimeout(timer);
  }, [search]);

  useEffect(
    () => {
      void listFavoriteEventIds().then(
        (ids) => setFavorites(new Set(ids)),
        () => {},
      );
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  const request = useMemo<TimelineRequest>(
    () => ({
      cursor: null,
      pageSize: PAGE_SIZE,
      filename: debounced || null,
      extension: extension.trim() || null,
      eventType: (eventType || null) as TimelineRequest['eventType'],
      folderId: folderId ? Number(folderId) : null,
      dateFrom: boundary(dateFrom, false),
      dateTo: boundary(dateTo, true),
      presence: (presence || null) as TimelineRequest['presence'],
    }),
    [dateFrom, dateTo, debounced, eventType, extension, folderId, presence],
  );

  useEffect(() => {
    let active = true;
    void queryTimeline(request).then(
      (next) => {
        if (active) {
          setPage(next);
          setLoading(false);
          setFocusedIndex(-1);
        }
      },
      (reason: ApplicationError) => {
        if (active) {
          setError(reason);
          setLoading(false);
        }
      },
    );
    return () => {
      active = false;
    };
  }, [queryTimeline, request]);

  useEffect(() => {
    if (!selected) return;
    let active = true;
    void Promise.all([
      getFileEventHistory(selected.file.id),
      getScanHistory(selected.event.indexedFolderId),
      getFilePathHistory(selected.file.id),
    ]).then(([events, scans, paths]) => {
      if (active) {
        setEventHistory(events);
        setScanHistory(scans);
        setPathHistory(paths);
      }
    });
    return () => {
      active = false;
    };
  }, [getFileEventHistory, getScanHistory, getFilePathHistory, selected]);

  const active = [
    debounced && `${t('timeline.filters.filename')}: ${debounced}`,
    extension && `${t('timeline.filters.extension')}: ${extension}`,
    eventType && t(`timeline.events.${eventType}`),
    folderId &&
      indexedFolders.state === 'ready' &&
      indexedFolders.data.find((folder) => folder.id === Number(folderId))?.displayName,
    dateFrom && `${t('timeline.filters.from')}: ${dateFrom}`,
    dateTo && `${t('timeline.filters.to')}: ${dateTo}`,
    presence && t(`timeline.presence.${presence}`),
  ].filter(Boolean) as string[];

  const selectItem = (item: TimelineItem) => {
    setEventHistory([]);
    setScanHistory([]);
    setPathHistory([]);
    setActionError(null);
    setPathCopied(false);
    setSelected(item);
  };

  const closeDetails = () => {
    setSelected(null);
    setEventHistory([]);
    setScanHistory([]);
    setPathHistory([]);
    setActionError(null);
  };

  const reset = () => {
    setSearch('');
    setDebounced('');
    setExtension('');
    setEventType('');
    setFolderId('');
    setDateFrom('');
    setDateTo('');
    setPresence('');
    setFocusedIndex(-1);
  };

  const groups = useMemo(() => {
    const result = new Map<DateGroup, TimelineItem[]>();
    page?.items.forEach((item) => {
      const key = timelineDateGroup(item.event.detectedAt);
      result.set(key, [...(result.get(key) ?? []), item]);
    });
    return result;
  }, [page]);

  const allItems = useMemo(
    () => DATE_GROUP_ORDER.flatMap((group) => groups.get(group) ?? []),
    [groups],
  );

  const setQuickDate = useCallback((label: 'today' | 'yesterday' | 'last7' | 'last30') => {
    const now = new Date();
    const fmt = (d: Date) => d.toISOString().split('T')[0] ?? '';
    if (label === 'today') {
      setDateFrom(fmt(now));
      setDateTo('');
    } else if (label === 'yesterday') {
      const y = new Date(now);
      y.setDate(y.getDate() - 1);
      setDateFrom(fmt(y));
      setDateTo('');
    } else if (label === 'last7') {
      const d = new Date(now);
      d.setDate(d.getDate() - 6);
      setDateFrom(fmt(d));
      setDateTo('');
    } else {
      const d = new Date(now);
      d.setDate(d.getDate() - 29);
      setDateFrom(fmt(d));
      setDateTo('');
    }
  }, []);

  const loadMore = async () => {
    if (!page?.nextCursor) return;
    setLoadingMore(true);
    try {
      const next = await queryTimeline({ ...request, cursor: page.nextCursor });
      setError(null);
      setPage({
        items: [...page.items, ...next.items],
        nextCursor: next.nextCursor,
        hasMore: next.hasMore,
      });
    } catch (reason: unknown) {
      setError(reason as ApplicationError);
    } finally {
      setLoadingMore(false);
    }
  };

  const runAction = async (action: () => Promise<void>) => {
    setActionError(null);
    try {
      await action();
    } catch (reason: unknown) {
      setActionError(reason as ApplicationError);
    }
  };

  const handleToggleFavorite = async (eventId: number) => {
    try {
      const added = await toggleEventFavorite(eventId);
      setFavorites((prev) => {
        const next = new Set(prev);
        if (added) next.add(eventId);
        else next.delete(eventId);
        return next;
      });
    } catch {
      /* ignore */
    }
  };

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === '/' && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        searchInputRef.current?.focus();
        return;
      }
      if (selected) {
        if (e.key === 'Escape') {
          e.preventDefault();
          closeDetails();
          return;
        }
        return;
      }
      if (e.key === 'Escape') {
        e.preventDefault();
        reset();
        return;
      }
      if ((e.key === 'j' || e.key === 'ArrowDown') && allItems.length > 0) {
        e.preventDefault();
        setFocusedIndex((prev) => (prev + 1) % allItems.length);
        return;
      }
      if ((e.key === 'k' || e.key === 'ArrowUp') && allItems.length > 0) {
        e.preventDefault();
        setFocusedIndex((prev) => (prev <= 0 ? allItems.length - 1 : prev - 1));
        return;
      }
      if (e.key === 'Enter' && focusedIndex >= 0 && focusedIndex < allItems.length) {
        e.preventDefault();
        selectItem(allItems[focusedIndex]!);
      }
    },
    [allItems, focusedIndex, selected],
  );

  return (
    <div className="page-view timeline-page" onKeyDown={handleKeyDown} tabIndex={-1}>
      <header className="page-header timeline-header">
        <div>
          <h2>{t('timeline.title')}</h2>
          <p>{t('timeline.description')}</p>
        </div>
        <span className="timeline-count">
          {t('timeline.results', { count: page?.items.length ?? 0 })}
        </span>
      </header>

      <section className="timeline-quick-filters">
        <button type="button" onClick={() => setQuickDate('today')}>
          {t('timeline.quickFilters.today')}
        </button>
        <button type="button" onClick={() => setQuickDate('yesterday')}>
          {t('timeline.quickFilters.yesterday')}
        </button>
        <button type="button" onClick={() => setQuickDate('last7')}>
          {t('timeline.quickFilters.last7Days')}
        </button>
        <button type="button" onClick={() => setQuickDate('last30')}>
          {t('timeline.quickFilters.last30Days')}
        </button>
        <label className="quick-date-jump">
          <span>{t('timeline.quickFilters.jumpToDate')}</span>
          <input
            type="date"
            value=""
            onChange={(e) => {
              if (e.target.value) {
                setDateFrom(e.target.value);
                setDateTo(e.target.value);
              }
            }}
          />
        </label>
      </section>

      <section className="timeline-filters" aria-label={t('timeline.filters.label')}>
        <label className="search-control">
          <span>{t('timeline.filters.filename')}</span>
          <MagnifyingGlass size={18} />
          <input
            ref={searchInputRef}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={t('timeline.filters.searchPlaceholder')}
          />
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
          <span>{t('timeline.filters.event')}</span>
          <select value={eventType} onChange={(e) => setEventType(e.target.value)}>
            <option value="">{t('timeline.filters.all')}</option>
            <option value="created">{t('timeline.events.created')}</option>
            <option value="modified">{t('timeline.events.modified')}</option>
            <option value="deleted">{t('timeline.events.deleted')}</option>
            <option value="renamed">{t('timeline.events.renamed')}</option>
            <option value="moved">{t('timeline.events.moved')}</option>
            <option value="likely_renamed">{t('timeline.events.likelyRenamed')}</option>
            <option value="possible_move">{t('timeline.events.possibleMove')}</option>
          </select>
        </label>
        <label>
          <span>{t('timeline.filters.folder')}</span>
          <select value={folderId} onChange={(e) => setFolderId(e.target.value)}>
            <option value="">{t('timeline.filters.all')}</option>
            {indexedFolders.state === 'ready'
              ? indexedFolders.data.map((folder) => (
                  <option key={folder.id} value={folder.id}>
                    {folder.displayName}
                  </option>
                ))
              : null}
          </select>
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
      </section>

      {active.length > 0 && (
        <div className="active-filters">
          {active.map((filter) => (
            <span key={filter}>{filter}</span>
          ))}
          <button type="button" onClick={reset}>
            <ArrowClockwise size={15} />
            {t('timeline.filters.reset')}
          </button>
        </div>
      )}

      {loading && <LoadingState />}
      {error && <ErrorState error={error} onRetry={reload} />}
      {!loading && !error && page?.items.length === 0 && (
        <EmptyState
          icon={<ClockCounterClockwise size={28} />}
          title={active.length ? t('timeline.noMatchesTitle') : t('timeline.emptyTitle')}
          body={active.length ? t('timeline.noMatchesBody') : t('timeline.emptyBody')}
        />
      )}

      {!loading && !error && (page?.items.length ?? 0) > 0 && (
        <div className="timeline-stream">
          {DATE_GROUP_ORDER.map((group) => {
            const items = groups.get(group);
            if (!items?.length) return null;
            let runningIndex = 0;
            for (let i = 0; i < DATE_GROUP_ORDER.indexOf(group); i++) {
              runningIndex += (groups.get(DATE_GROUP_ORDER[i]!) ?? []).length;
            }
            return (
              <section className="timeline-group" key={group}>
                <h3>{t(`timeline.groups.${group}`)}</h3>
                <ol>
                  {items.map((item, idx) => {
                    const globalIdx = runningIndex + idx;
                    const isFav = favorites.has(item.event.id);
                    const confidence =
                      item.event.confidence != null &&
                      item.event.confidence < 1.0 &&
                      item.event.eventSource !== 'scan';
                    return (
                      <li key={item.event.id}>
                        <button
                          type="button"
                          className={`event-card${globalIdx === focusedIndex ? ' is-focused' : ''}`}
                          onClick={() => selectItem(item)}
                        >
                          <span className="event-icon" data-event={item.event.eventType}>
                            <File size={19} weight="duotone" />
                          </span>
                          <span className="event-main">
                            <strong>{item.file.name}</strong>
                            <small title={item.file.parentPath}>{item.file.parentPath}</small>
                          </span>
                          <span className="event-folder">{item.folderName}</span>
                          <span className="event-kind" data-event={item.event.eventType}>
                            {t(`timeline.events.${item.event.eventType}`)}
                          </span>
                          {confidence && (
                            <span className="event-confidence">
                              {Math.round(item.event.confidence! * 100)}%
                            </span>
                          )}
                          <time>
                            {new Intl.DateTimeFormat(i18n.language, {
                              hour: '2-digit',
                              minute: '2-digit',
                            }).format(new Date(item.event.detectedAt))}
                          </time>
                          <span
                            className={`event-favorite${isFav ? ' is-active' : ''}`}
                            onClick={(e) => {
                              e.stopPropagation();
                              void handleToggleFavorite(item.event.id);
                            }}
                            title={
                              isFav
                                ? t('timeline.details.unfavorite')
                                : t('timeline.details.favorite')
                            }
                            role="button"
                            tabIndex={0}
                            onKeyDown={(e) => {
                              if (e.key === 'Enter' || e.key === ' ') {
                                e.stopPropagation();
                                void handleToggleFavorite(item.event.id);
                              }
                            }}
                          >
                            <Star size={16} weight={isFav ? 'fill' : 'regular'} />
                          </span>
                        </button>
                      </li>
                    );
                  })}
                </ol>
              </section>
            );
          })}
          {page?.hasMore && (
            <button
              className="button load-more"
              disabled={loadingMore}
              type="button"
              onClick={() => void loadMore()}
            >
              {loadingMore ? t('timeline.loadingMore') : t('timeline.loadMore')}
            </button>
          )}
        </div>
      )}

      {selected && (
        <div className="detail-backdrop" role="presentation" onMouseDown={closeDetails}>
          <aside
            className="file-detail"
            role="dialog"
            aria-modal="true"
            aria-labelledby="file-detail-title"
            onMouseDown={(e) => e.stopPropagation()}
          >
            <header>
              <div>
                <span>{t(`timeline.events.${selected.event.eventType}`)}</span>
                <h3 id="file-detail-title">{selected.file.name}</h3>
              </div>
              <button type="button" aria-label={t('timeline.details.close')} onClick={closeDetails}>
                <X size={20} />
              </button>
            </header>
            <dl className="file-facts">
              <div>
                <dt>{t('timeline.details.path')}</dt>
                <dd>
                  {selected.file.normalizedPath}
                  <button
                    type="button"
                    className="copy-button"
                    title={t('timeline.details.copyPath')}
                    onClick={() => {
                      void navigator.clipboard.writeText(selected.file.normalizedPath).then(() => {
                        setPathCopied(true);
                        globalThis.setTimeout(() => setPathCopied(false), 2000);
                      });
                    }}
                  >
                    <Copy size={14} />
                    {pathCopied ? t('timeline.details.pathCopied') : null}
                  </button>
                </dd>
              </div>
              <div>
                <dt>{t('timeline.details.folder')}</dt>
                <dd>{selected.folderName}</dd>
              </div>
              <div>
                <dt>{t('timeline.details.size')}</dt>
                <dd>{bytes(selected.file.sizeBytes)}</dd>
              </div>
              <div>
                <dt>{t('timeline.details.status')}</dt>
                <dd>{t(`timeline.presence.${selected.file.isPresent ? 'present' : 'deleted'}`)}</dd>
              </div>
            </dl>
            <div className="file-actions">
              <button
                className="button button--primary"
                type="button"
                disabled={!selected.file.isPresent}
                onClick={() => void runAction(() => openTimelineFile(selected.file.id))}
              >
                <File size={17} />
                {t('timeline.details.open')}
              </button>
              <button
                className="button"
                type="button"
                disabled={!selected.file.isPresent}
                onClick={() => void runAction(() => revealTimelineFile(selected.file.id))}
              >
                <FolderOpen size={17} />
                {t('timeline.details.reveal')}
              </button>
              <button
                className={`button${favorites.has(selected.event.id) ? ' button--active' : ''}`}
                type="button"
                onClick={() => void handleToggleFavorite(selected.event.id)}
              >
                <Star size={17} weight={favorites.has(selected.event.id) ? 'fill' : 'regular'} />
                {favorites.has(selected.event.id)
                  ? t('timeline.details.unfavorite')
                  : t('timeline.details.favorite')}
              </button>
            </div>
            {!selected.file.isPresent && (
              <p className="deleted-explanation">{t('timeline.details.deletedExplanation')}</p>
            )}
            {selected.event.confidence != null && selected.event.confidence !== 1.0 && (
              <p className="confidence-note">
                {t('timeline.details.confidenceNote', {
                  level: t(
                    `timeline.confidence.${selected.event.confidence >= 0.8 ? 'strong' : 'weak'}`,
                  ),
                })}
              </p>
            )}
            {selected.event.userConfirmation && (
              <p className="confidence-note">
                {t(`timeline.confirmation.${selected.event.userConfirmation}`)}
              </p>
            )}
            {(['likely_renamed', 'possible_move'].includes(selected.event.eventType) ||
              (selected.event.confidence != null && selected.event.confidence < 1.0)) &&
              !selected.event.userConfirmation && (
                <div className="confirmation-actions">
                  <button
                    className="button button--small"
                    type="button"
                    onClick={() =>
                      void runAction(async () => {
                        await confirmEvent(selected.event.id);
                        setSelected({
                          ...selected,
                          event: {
                            ...selected.event,
                            userConfirmation: 'confirmed',
                            userConfirmedAt: new Date().toISOString(),
                          },
                        });
                      })
                    }
                  >
                    {t('timeline.details.confirm')}
                  </button>
                  <button
                    className="button button--small"
                    type="button"
                    onClick={() =>
                      void runAction(async () => {
                        await rejectEvent(selected.event.id);
                        setSelected({
                          ...selected,
                          event: {
                            ...selected.event,
                            userConfirmation: 'rejected',
                            userConfirmedAt: new Date().toISOString(),
                          },
                        });
                      })
                    }
                  >
                    {t('timeline.details.reject')}
                  </button>
                </div>
              )}
            {actionError && (
              <p className="deleted-explanation" role="alert">
                {t(actionError.messageKey)}
              </p>
            )}
            <section className="history-section">
              <h4>{t('timeline.details.eventHistory')}</h4>
              <ol>
                {eventHistory.map((event) => (
                  <li key={event.id}>
                    <span>{t(`timeline.events.${event.eventType}`)}</span>
                    <time>
                      {new Intl.DateTimeFormat(i18n.language, {
                        dateStyle: 'medium',
                        timeStyle: 'short',
                      }).format(new Date(event.detectedAt))}
                    </time>
                  </li>
                ))}
              </ol>
            </section>
            <section className="history-section">
              <h4>{t('timeline.details.scanHistory')}</h4>
              <ol>
                {scanHistory.map((scan) => (
                  <li key={scan.id}>
                    <span>{t(`folders.scan.${scan.status}`)}</span>
                    <time>
                      {new Intl.DateTimeFormat(i18n.language, {
                        dateStyle: 'medium',
                        timeStyle: 'short',
                      }).format(new Date(scan.startedAt))}
                    </time>
                  </li>
                ))}
              </ol>
            </section>
            {pathHistory.length > 0 && (
              <section className="history-section">
                <h4>{t('timeline.details.pathHistory')}</h4>
                <ol>
                  {pathHistory.map((entry) => (
                    <li key={entry.id}>
                      <span>
                        {entry.oldPath} &rarr; {entry.newPath}
                      </span>
                      <small>
                        {t(`timeline.confidence.${entry.confidence >= 0.8 ? 'strong' : 'weak'}`)}
                      </small>
                    </li>
                  ))}
                </ol>
              </section>
            )}
          </aside>
        </div>
      )}
    </div>
  );
};
