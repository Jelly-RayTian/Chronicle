import './ProjectsPage.css';

import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Briefcase } from '@phosphor-icons/react';

import { useAppData } from '../app/AppDataContext';
import { EmptyState, ErrorState, LoadingState } from '../components/states/ContentState';
import { toApplicationError } from '../lib/tauri/client';
import type {
  ApplicationError,
  FileRecord,
  ProjectDetail,
  ProjectSummary,
  TimelineItem,
} from '../models';

export const ProjectsPage = () => {
  const { t } = useTranslation();
  const {
    projects,
    folderActionError,
    clearFolderActionError,
    suggestProjects,
    createProject,
    listAllFiles,
  } = useAppData();
  const [filter, setFilter] = useState<'all' | 'suggested' | 'active' | 'rejected'>('all');
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState('');
  const [newDescription, setNewDescription] = useState('');
  const [selectedFileIds, setSelectedFileIds] = useState<Set<number>>(new Set());
  const [allFiles, setAllFiles] = useState<FileRecord[]>([]);

  useEffect(() => {
    let active = true;
    void listAllFiles().then((files) => {
      if (active) setAllFiles(files);
    });
    return () => {
      active = false;
    };
  }, [listAllFiles]);

  const filtered = useMemo(() => {
    if (projects.state !== 'ready') return [];
    if (filter === 'all') return projects.data;
    return projects.data.filter((summary) => summary.project.status === filter);
  }, [projects, filter]);

  const grouped = useMemo(() => {
    const suggested: ProjectSummary[] = [];
    const active: ProjectSummary[] = [];
    const rejected: ProjectSummary[] = [];
    for (const summary of filtered) {
      if (summary.project.status === 'suggested') suggested.push(summary);
      else if (summary.project.status === 'rejected') rejected.push(summary);
      else active.push(summary);
    }
    return { suggested, active, rejected };
  }, [filtered]);

  const handleSuggest = async () => {
    clearFolderActionError();
    try {
      await suggestProjects({ folderId: null });
    } catch {
      // error is already set in context
    }
  };

  const handleCreate = async () => {
    if (!newName.trim()) return;
    clearFolderActionError();
    try {
      const detail = await createProject({
        name: newName.trim(),
        description: newDescription.trim() || null,
        fileIds: Array.from(selectedFileIds),
      });
      setNewName('');
      setNewDescription('');
      setSelectedFileIds(new Set());
      setCreating(false);
      setSelectedId(detail.project.id);
    } catch {
      // error handled in context
    }
  };

  if (selectedId != null) {
    return (
      <ProjectDetails
        key={selectedId}
        projectId={selectedId}
        onBack={() => setSelectedId(null)}
        allFiles={allFiles}
      />
    );
  }

  return (
    <div className="projects-page">
      <header className="page-header">
        <div>
          <h1>{t('projects.title')}</h1>
          <p>{t('projects.description')}</p>
        </div>
        <div className="page-actions">
          <button
            type="button"
            className="button secondary"
            onClick={() => {
              void handleSuggest();
            }}
          >
            {t('projects.suggest')}
          </button>
          <button
            type="button"
            className="button primary"
            onClick={() => {
              setCreating((value) => !value);
              setSelectedFileIds(new Set());
            }}
          >
            {creating ? t('versions.close') : t('projects.create')}
          </button>
        </div>
      </header>

      {folderActionError ? (
        <div className="inline-error" role="alert">
          {t(`errors.${folderActionError.messageKey.replace(/^errors\./, '')}` as never)}
        </div>
      ) : null}

      {creating ? (
        <section className="create-panel">
          <input
            type="text"
            value={newName}
            onChange={(event) => setNewName(event.target.value)}
            placeholder={t('projects.createPlaceholder')}
            aria-label={t('projects.createPlaceholder')}
            className="input"
          />
          <input
            type="text"
            value={newDescription}
            onChange={(event) => setNewDescription(event.target.value)}
            placeholder={t('projects.descriptionPlaceholder')}
            aria-label={t('projects.descriptionPlaceholder')}
            className="input"
          />
          <div className="file-picker">
            <span>{t('projects.addMember')}</span>
            <div className="file-list compact">
              {allFiles.map((file) => (
                <label key={file.id} className="file-option">
                  <input
                    type="checkbox"
                    checked={selectedFileIds.has(file.id)}
                    onChange={(event) => {
                      const next = new Set(selectedFileIds);
                      if (event.target.checked) next.add(file.id);
                      else next.delete(file.id);
                      setSelectedFileIds(next);
                    }}
                  />
                  <span>{file.name}</span>
                </label>
              ))}
            </div>
          </div>
          <div className="form-actions">
            <button
              type="button"
              className="button primary"
              onClick={() => {
                void handleCreate();
              }}
            >
              {t('projects.create')}
            </button>
            <button
              type="button"
              className="button secondary"
              onClick={() => {
                setCreating(false);
                setSelectedFileIds(new Set());
              }}
            >
              {t('versions.close')}
            </button>
          </div>
        </section>
      ) : null}

      <div className="filter-bar">
        {(['all', 'suggested', 'active', 'rejected'] as const).map((status) => (
          <button
            key={status}
            type="button"
            className={filter === status ? 'filter-chip is-active' : 'filter-chip'}
            onClick={() => setFilter(status)}
          >
            {t(`projects.status.${status}`)}
          </button>
        ))}
      </div>

      {projects.state === 'loading' ? <LoadingState /> : null}
      {projects.state === 'error' ? <ErrorState error={projects.error} /> : null}

      {projects.state === 'ready' ? (
        <div className="project-lists">
          {grouped.suggested.length > 0 ? (
            <section>
              <h2>{t('projects.suggestedTitle')}</h2>
              <div className="card-list">
                {grouped.suggested.map((summary) => (
                  <ProjectCard
                    key={summary.project.id}
                    summary={summary}
                    onSelect={() => setSelectedId(summary.project.id)}
                  />
                ))}
              </div>
            </section>
          ) : null}
          {grouped.active.length > 0 ? (
            <section>
              <h2>{t('projects.activeTitle')}</h2>
              <div className="card-list">
                {grouped.active.map((summary) => (
                  <ProjectCard
                    key={summary.project.id}
                    summary={summary}
                    onSelect={() => setSelectedId(summary.project.id)}
                  />
                ))}
              </div>
            </section>
          ) : null}
          {grouped.rejected.length > 0 ? (
            <section>
              <h2>{t('projects.rejectedTitle')}</h2>
              <div className="card-list">
                {grouped.rejected.map((summary) => (
                  <ProjectCard
                    key={summary.project.id}
                    summary={summary}
                    onSelect={() => setSelectedId(summary.project.id)}
                  />
                ))}
              </div>
            </section>
          ) : null}
          {filtered.length === 0 ? (
            <EmptyState
              icon={<Briefcase size={28} />}
              title={t('projects.emptyTitle')}
              body={t('projects.emptyBody')}
            />
          ) : null}
        </div>
      ) : null}
    </div>
  );
};

const ProjectCard = ({ summary, onSelect }: { summary: ProjectSummary; onSelect: () => void }) => {
  const { t } = useTranslation();
  return (
    <button type="button" className="project-card" onClick={onSelect}>
      <div className="project-card-name">{summary.project.name}</div>
      <div className="project-card-meta">
        {t('projects.memberCount', { count: summary.memberCount })}
      </div>
      {summary.project.status === 'suggested' ? (
        <span className="badge suggested">{t('projects.status.suggested')}</span>
      ) : null}
    </button>
  );
};

const ProjectDetails = ({
  projectId,
  onBack,
  allFiles,
}: {
  projectId: number;
  onBack: () => void;
  allFiles: FileRecord[];
}) => {
  const { t } = useTranslation();
  const {
    getProject,
    updateProject,
    acceptProject,
    rejectProject,
    addProjectMember,
    removeProjectMember,
    getProjectTimeline,
    folderActionError,
    clearFolderActionError,
  } = useAppData();
  const [detail, setDetail] = useState<ProjectDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<ApplicationError | null>(null);
  const [editingName, setEditingName] = useState(false);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [addFileId, setAddFileId] = useState('');
  const [timeline, setTimeline] = useState<TimelineItem[]>([]);

  const load = async () => {
    setError(null);
    try {
      const next = await getProject({ projectId });
      setDetail(next);
      setName(next.project.name);
      setDescription(next.project.description);
      const events = await getProjectTimeline({ projectId });
      setTimeline(events);
    } catch (err: unknown) {
      setError(toApplicationError(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    let active = true;
    getProject({ projectId })
      .then((next) => {
        if (!active) return;
        setDetail(next);
        setName(next.project.name);
        setDescription(next.project.description);
        return getProjectTimeline({ projectId });
      })
      .then((events) => {
        if (active) setTimeline(events ?? []);
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
  }, [projectId, getProject, getProjectTimeline]);

  const handleSaveName = async () => {
    if (!detail) return;
    setLoading(true);
    try {
      await updateProject({
        projectId,
        name: name.trim() || null,
        description: description.trim() || null,
      });
      setEditingName(false);
      await load();
    } catch {
      // handled in context
    } finally {
      setLoading(false);
    }
  };

  const handleAccept = async () => {
    clearFolderActionError();
    setLoading(true);
    try {
      await acceptProject({ projectId });
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
      await rejectProject({ projectId });
      await load();
    } catch {
      // handled
    } finally {
      setLoading(false);
    }
  };

  const handleAddMember = async () => {
    if (!addFileId) return;
    clearFolderActionError();
    setLoading(true);
    try {
      await addProjectMember({ projectId, fileId: Number(addFileId) });
      setAddFileId('');
      await load();
    } catch {
      // handled
    } finally {
      setLoading(false);
    }
  };

  const handleRemoveMember = async (fileId: number) => {
    clearFolderActionError();
    setLoading(true);
    try {
      await removeProjectMember({ projectId, fileId });
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
          error ?? { code: 'not_found', messageKey: 'errors.projectNotFound', retryable: false }
        }
      />
    );

  const availableFiles = allFiles.filter(
    (file) => !detail.members.some((member) => member.member.fileId === file.id),
  );

  return (
    <div className="project-details">
      <button type="button" className="button secondary" onClick={onBack}>
        ← {t('versions.close')}
      </button>

      {folderActionError ? (
        <div className="inline-error" role="alert">
          {t(`errors.${folderActionError.messageKey.replace(/^errors\./, '')}` as never)}
        </div>
      ) : null}

      <div className="detail-header">
        {editingName ? (
          <div className="inline-edit">
            <input
              type="text"
              value={name}
              onChange={(event) => setName(event.target.value)}
              aria-label={t('projects.createPlaceholder')}
              className="input"
            />
            <input
              type="text"
              value={description}
              onChange={(event) => setDescription(event.target.value)}
              placeholder={t('projects.descriptionPlaceholder')}
              aria-label={t('projects.descriptionPlaceholder')}
              className="input"
            />
            <button
              type="button"
              className="button primary"
              onClick={() => {
                void handleSaveName();
              }}
            >
              {t('projects.save')}
            </button>
          </div>
        ) : (
          <div>
            <h1>{detail.project.name}</h1>
            {detail.project.description ? <p>{detail.project.description}</p> : null}
            <button type="button" className="button secondary" onClick={() => setEditingName(true)}>
              {t('projects.editName')}
            </button>
          </div>
        )}

        {detail.project.status === 'suggested' ? (
          <div className="detail-actions">
            <button
              type="button"
              className="button primary"
              onClick={() => {
                void handleAccept();
              }}
            >
              {t('projects.accept')}
            </button>
            <button
              type="button"
              className="button danger"
              onClick={() => {
                void handleReject();
              }}
            >
              {t('projects.reject')}
            </button>
          </div>
        ) : null}
      </div>

      <section className="detail-section">
        <h2>{t('projects.members')}</h2>
        <div className="member-toolbar">
          <select
            value={addFileId}
            onChange={(event) => setAddFileId(event.target.value)}
            className="select"
          >
            <option value="">{t('projects.selectFile')}</option>
            {availableFiles.map((file) => (
              <option key={file.id} value={file.id}>
                {file.name}
              </option>
            ))}
          </select>
          <button
            type="button"
            className="button secondary"
            onClick={() => {
              void handleAddMember();
            }}
            disabled={!addFileId}
          >
            {t('projects.add')}
          </button>
        </div>
        <ul className="member-list">
          {detail.members.map((member) => (
            <li key={member.member.id} className="member-row">
              <span>{member.file.name}</span>
              <button
                type="button"
                className="button ghost"
                onClick={() => {
                  void handleRemoveMember(member.member.fileId);
                }}
              >
                {t('projects.removeMember')}
              </button>
            </li>
          ))}
        </ul>
      </section>

      <section className="detail-section">
        <h2>{t('projects.timeline')}</h2>
        {timeline.length === 0 ? (
          <p>{t('projects.noTimeline')}</p>
        ) : (
          <ul className="timeline-mini">
            {timeline.map((item) => (
              <li key={item.event.id}>
                <span className="event-type">{item.event.eventType}</span>
                <span className="event-file">{item.file.name}</span>
                <span className="event-time">{item.event.detectedAt}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
};
