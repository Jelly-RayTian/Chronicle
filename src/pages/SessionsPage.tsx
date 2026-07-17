import './SessionsPage.css';

import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { GitMerge, Hourglass, X } from '@phosphor-icons/react';

import { useAppData } from '../app/AppDataContext';
import { EmptyState, ErrorState, LoadingState } from '../components/states/ContentState';
import { toApplicationError } from '../lib/tauri/client';
import type {
  ActivitySessionDetail,
  ActivitySessionSummary,
  ApplicationError,
  Project,
} from '../models';

const parseEventBreakdown = (summary: string) => {
  if (!summary) return {};
  const result: Record<string, number> = {};
  for (const part of summary.split(',')) {
    const match = part.trim().match(/^(\d+)\s+(.+)$/);
    if (match?.[2]) {
      result[match[2]] = Number(match[1]);
    }
  }
  return result;
};

const formatDuration = (startedAt: string, endedAt: string) => {
  const start = new Date(startedAt).getTime();
  const end = new Date(endedAt).getTime();
  const minutes = Math.round((end - start) / 60_000);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  const remain = minutes % 60;
  return remain > 0 ? `${hours}h ${remain}m` : `${hours}h`;
};

export const SessionsPage = () => {
  const { t } = useTranslation();
  const { sessions, generateSessions, mergeSessions, folderActionError, clearFolderActionError } =
    useAppData();
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [mergeSourceId, setMergeSourceId] = useState<number | null>(null);

  const handleGenerate = async () => {
    clearFolderActionError();
    try {
      await generateSessions({ gapMinutes: null });
    } catch {
      /* handled in context */
    }
  };

  const handleMerge = async (targetId: number) => {
    if (mergeSourceId == null) return;
    clearFolderActionError();
    try {
      await mergeSessions({ targetSessionId: targetId, sourceSessionId: mergeSourceId });
      setMergeSourceId(null);
    } catch {
      /* handled in context */
    }
  };

  if (selectedId != null) {
    return (
      <SessionDetails key={selectedId} sessionId={selectedId} onBack={() => setSelectedId(null)} />
    );
  }

  return (
    <div className="sessions-page">
      <header className="sessions-header">
        <div>
          <h1>{t('sessions.title')}</h1>
          <p>{t('sessions.description')}</p>
        </div>
        <button
          type="button"
          className="button primary"
          onClick={() => {
            void handleGenerate();
          }}
        >
          {t('sessions.generate')}
        </button>
      </header>

      {folderActionError ? (
        <div className="inline-error" role="alert">
          {t(`errors.${folderActionError.messageKey.replace(/^errors\./, '')}` as never)}
        </div>
      ) : null}

      <p className="privacy-note">{t('sessions.noProductivityScore')}</p>

      {mergeSourceId != null && (
        <div className="merge-banner">
          <GitMerge size={16} />
          <span>{t('sessions.mergeHint')}</span>
          <button
            type="button"
            className="button secondary small"
            onClick={() => setMergeSourceId(null)}
          >
            <X size={14} />
          </button>
        </div>
      )}

      {sessions.state === 'loading' ? <LoadingState /> : null}
      {sessions.state === 'error' ? <ErrorState error={sessions.error} /> : null}

      {sessions.state === 'ready' ? (
        <div className="session-list">
          {sessions.data.length === 0 ? (
            <EmptyState
              icon={<Hourglass size={28} />}
              title={t('sessions.emptyTitle')}
              body={t('sessions.emptyBody')}
            />
          ) : (
            sessions.data.map((summary) => (
              <SessionCard
                key={summary.session.id}
                summary={summary}
                merging={mergeSourceId === summary.session.id}
                showMergeTarget={mergeSourceId != null}
                onSelect={() => setSelectedId(summary.session.id)}
                onMergeSelect={() => {
                  if (mergeSourceId == null) {
                    setMergeSourceId(summary.session.id);
                  } else {
                    void handleMerge(summary.session.id);
                  }
                }}
              />
            ))
          )}
        </div>
      ) : null}
    </div>
  );
};

const SessionCard = ({
  summary,
  merging,
  showMergeTarget,
  onSelect,
  onMergeSelect,
}: {
  summary: ActivitySessionSummary;
  merging: boolean;
  showMergeTarget: boolean;
  onSelect: () => void;
  onMergeSelect: () => void;
}) => {
  const { t } = useTranslation();
  const breakdown = useMemo(
    () => parseEventBreakdown(summary.session.eventSummary),
    [summary.session.eventSummary],
  );
  const duration = formatDuration(summary.session.startedAt, summary.session.endedAt);

  return (
    <div className={`session-card${merging ? ' is-merging' : ''}`}>
      <button type="button" className="session-card-main" onClick={onSelect}>
        <div className="session-card-title">{summary.session.title}</div>
        <div className="session-card-meta">
          {duration && <span className="meta-chip">{duration}</span>}
          <span className="meta-chip">
            {t('sessions.eventCount', { count: summary.eventCount })}
          </span>
          <span className="meta-chip">{t('sessions.fileCount', { count: summary.fileCount })}</span>
          {summary.projectName && <span className="meta-chip project">{summary.projectName}</span>}
        </div>
        <div className="session-card-summary">
          {Object.entries(breakdown).length > 0
            ? Object.entries(breakdown).map(([kind, count]) => (
                <span key={kind} className="event-tag" data-event={kind}>
                  {count} {t(`timeline.events.${kind}` as never)}
                </span>
              ))
            : summary.session.eventSummary}
        </div>
        <div className="session-card-time">
          {new Date(summary.session.startedAt).toLocaleDateString()}{' '}
          {new Date(summary.session.startedAt).toLocaleTimeString([], {
            hour: '2-digit',
            minute: '2-digit',
          })}
          {' — '}
          {new Date(summary.session.endedAt).toLocaleTimeString([], {
            hour: '2-digit',
            minute: '2-digit',
          })}
        </div>
      </button>
      <button
        type="button"
        className={`merge-button${merging ? ' active' : ''}`}
        onClick={(e) => {
          e.stopPropagation();
          onMergeSelect();
        }}
        title={showMergeTarget ? t('sessions.mergeTarget') : t('sessions.merge')}
      >
        <GitMerge size={15} />
        {merging ? t('sessions.mergeTarget') : showMergeTarget ? t('sessions.merge') : ''}
      </button>
    </div>
  );
};

const SessionDetails = ({ sessionId, onBack }: { sessionId: number; onBack: () => void }) => {
  const { t } = useTranslation();
  const {
    getSession,
    updateSession,
    acceptSession,
    rejectSession,
    listProjects,
    folderActionError,
    clearFolderActionError,
  } = useAppData();
  const [detail, setDetail] = useState<ActivitySessionDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<ApplicationError | null>(null);
  const [editing, setEditing] = useState(false);
  const [title, setTitle] = useState('');
  const [projectId, setProjectId] = useState<number | null>(null);
  const [projects, setProjects] = useState<Project[]>([]);

  useEffect(() => {
    let active = true;
    getSession({ sessionId })
      .then((next) => {
        if (!active) return;
        setDetail(next);
        setTitle(next.session.title);
        setProjectId(next.session.projectId);
        return listProjects({ status: 'active' });
      })
      .then((summaries) => {
        if (active) setProjects((summaries ?? []).map((summary) => summary.project));
      })
      .catch((err: unknown) => {
        if (active) setError(toApplicationError(err));
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [sessionId, getSession, listProjects]);

  const handleSave = async () => {
    if (!detail) return;
    clearFolderActionError();
    setLoading(true);
    try {
      await updateSession({
        sessionId,
        title: title.trim() || null,
        projectId,
      });
      setEditing(false);
      const next = await getSession({ sessionId });
      setDetail(next);
      setTitle(next.session.title);
      setProjectId(next.session.projectId);
    } catch {
      /* handled */
    } finally {
      setLoading(false);
    }
  };

  const handleAccept = async () => {
    clearFolderActionError();
    setLoading(true);
    try {
      await acceptSession({ sessionId });
      const next = await getSession({ sessionId });
      setDetail(next);
    } catch {
      /* handled */
    } finally {
      setLoading(false);
    }
  };

  const handleReject = async () => {
    clearFolderActionError();
    setLoading(true);
    try {
      await rejectSession({ sessionId });
      const next = await getSession({ sessionId });
      setDetail(next);
    } catch {
      /* handled */
    } finally {
      setLoading(false);
    }
  };

  if (loading) return <LoadingState />;
  if (error || !detail)
    return (
      <ErrorState
        error={
          error ?? { code: 'not_found', messageKey: 'errors.sessionNotFound', retryable: false }
        }
      />
    );

  const duration = formatDuration(detail.session.startedAt, detail.session.endedAt);
  const breakdown = parseEventBreakdown(detail.session.eventSummary);

  return (
    <div className="session-details">
      <button type="button" className="button secondary" onClick={onBack}>
        ← {t('versions.close')}
      </button>

      {folderActionError ? (
        <div className="inline-error" role="alert">
          {t(`errors.${folderActionError.messageKey.replace(/^errors\./, '')}` as never)}
        </div>
      ) : null}

      <div className="detail-header">
        {editing ? (
          <div className="inline-edit">
            <input
              type="text"
              value={title}
              onChange={(event) => setTitle(event.target.value)}
              placeholder={t('sessions.titlePlaceholder')}
              className="input"
            />
            <select
              value={projectId ?? ''}
              onChange={(event) =>
                setProjectId(event.target.value ? Number(event.target.value) : null)
              }
              className="select"
            >
              <option value="">{t('sessions.noProject')}</option>
              {projects.map((project) => (
                <option key={project.id} value={project.id}>
                  {project.name}
                </option>
              ))}
            </select>
            <button
              type="button"
              className="button primary"
              onClick={() => {
                void handleSave();
              }}
            >
              {t('projects.save')}
            </button>
          </div>
        ) : (
          <div>
            <h1>{detail.session.title}</h1>
            <p>
              {new Date(detail.session.startedAt).toLocaleString()} —{' '}
              {new Date(detail.session.endedAt).toLocaleString()} {duration ? `(${duration})` : ''}
            </p>
            {detail.project ? <p>{detail.project.name}</p> : null}

            <div className="session-breakdown">
              {Object.entries(breakdown).map(([kind, count]) => (
                <span key={kind} className="event-tag" data-event={kind}>
                  {count} {t(`timeline.events.${kind}` as never)}
                </span>
              ))}
            </div>

            <button type="button" className="button secondary" onClick={() => setEditing(true)}>
              {t('sessions.editTitle')}
            </button>
          </div>
        )}

        <div className="detail-actions">
          <button
            type="button"
            className="button primary"
            onClick={() => {
              void handleAccept();
            }}
          >
            {t('sessions.accept')}
          </button>
          <button
            type="button"
            className="button danger"
            onClick={() => {
              void handleReject();
            }}
          >
            {t('sessions.reject')}
          </button>
        </div>
      </div>

      <section className="detail-section">
        <h2>{t('sessions.relatedFiles')}</h2>
        <ul className="file-list">
          {detail.files.map((file) => (
            <li key={file.id}>{file.name}</li>
          ))}
        </ul>
      </section>

      <section className="detail-section">
        <h2>{t('sessions.events')}</h2>
        <ul className="timeline-mini">
          {detail.events.slice(0, 100).map((event) => (
            <li key={event.id}>
              <span className="event-type">{event.eventType}</span>
              <span className="event-time">{new Date(event.detectedAt).toLocaleString()}</span>
            </li>
          ))}
          {detail.events.length > 100 && (
            <li className="more-events">… {detail.events.length - 100} more events</li>
          )}
        </ul>
      </section>
    </div>
  );
};
