import './SessionsPage.css';

import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Hourglass } from '@phosphor-icons/react';

import { useAppData } from '../app/AppDataContext';
import { EmptyState, ErrorState, LoadingState } from '../components/states/ContentState';
import { toApplicationError } from '../lib/tauri/client';
import type {
  ActivitySessionDetail,
  ActivitySessionSummary,
  ApplicationError,
  Project,
} from '../models';

export const SessionsPage = () => {
  const { t } = useTranslation();
  const { sessions, generateSessions, folderActionError, clearFolderActionError } = useAppData();
  const [selectedId, setSelectedId] = useState<number | null>(null);

  const handleGenerate = async () => {
    clearFolderActionError();
    try {
      await generateSessions({ gapMinutes: null });
    } catch {
      // handled in context
    }
  };

  if (selectedId != null) {
    return (
      <SessionDetails key={selectedId} sessionId={selectedId} onBack={() => setSelectedId(null)} />
    );
  }

  return (
    <div className="sessions-page">
      <header className="page-header">
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
                onSelect={() => setSelectedId(summary.session.id)}
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
  onSelect,
}: {
  summary: ActivitySessionSummary;
  onSelect: () => void;
}) => {
  const { t } = useTranslation();
  return (
    <button type="button" className="session-card" onClick={onSelect}>
      <div className="session-card-title">{summary.session.title}</div>
      <div className="session-card-meta">
        {t('sessions.eventCount', { count: summary.eventCount })} ·{' '}
        {t('sessions.fileCount', { count: summary.fileCount })}
        {summary.projectName ? ` · ${summary.projectName}` : null}
      </div>
      <div className="session-card-summary">{summary.session.eventSummary}</div>
    </button>
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

  const load = async () => {
    setError(null);
    try {
      const next = await getSession({ sessionId });
      setDetail(next);
      setTitle(next.session.title);
      setProjectId(next.session.projectId);
      const projectList = await listProjects({ status: 'active' });
      setProjects(projectList.map((summary) => summary.project));
    } catch (err: unknown) {
      setError(toApplicationError(err));
    } finally {
      setLoading(false);
    }
  };

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
      await load();
    } catch {
      // handled
    } finally {
      setLoading(false);
    }
  };

  const handleAccept = async () => {
    clearFolderActionError();
    setLoading(true);
    try {
      await acceptSession({ sessionId });
      await load();
    } catch {
      // handled
    } finally {
      setLoading(false);
    }
  };

  const handleReject = async () => {
    clearFolderActionError();
    setLoading(true);
    try {
      await rejectSession({ sessionId });
      await load();
    } catch {
      // handled
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
              {detail.session.startedAt} — {detail.session.endedAt}
            </p>
            {detail.project ? <p>{detail.project.name}</p> : null}
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
          {detail.events.map((event) => (
            <li key={event.id}>
              <span className="event-type">{event.eventType}</span>
              <span className="event-time">{event.detectedAt}</span>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
};
