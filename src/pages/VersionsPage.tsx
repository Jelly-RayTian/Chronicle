import { ArrowsMerge, ArrowsSplit, File, FloppyDisk, Plus, Trash, X } from '@phosphor-icons/react';
import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useAppData } from '../app/AppDataContext';
import { EmptyState, ErrorState, LoadingState } from '../components/states/ContentState';
import type {
  ApplicationError,
  FileRecord,
  VersionFamilyDetail,
  VersionFamilyMemberWithFile,
  VersionFamilyStatus,
} from '../models';
import './VersionsPage.css';

const statusOrder: VersionFamilyStatus[] = ['suggested', 'confirmed', 'rejected', 'superseded'];

const formatEvidence = (evidence: string) =>
  evidence
    .split(';')
    .map((part) => part.trim())
    .filter(Boolean);

export const VersionsPage = () => {
  const { t } = useTranslation();
  const {
    versionFamilies,
    suggestVersionFamilies,
    getVersionFamily,
    acceptVersionFamily,
    rejectVersionFamily,
    renameVersionFamily,
    splitVersionFamily,
    mergeVersionFamilies,
    addVersionFamilyMember,
    removeVersionFamilyMember,
    listAllFiles,
    reload,
  } = useAppData();

  const [filter, setFilter] = useState<VersionFamilyStatus | 'all'>('all');
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [detail, setDetail] = useState<VersionFamilyDetail | null>(null);
  const [loadingDetail, setLoadingDetail] = useState(false);
  const [detailError, setDetailError] = useState<ApplicationError | null>(null);
  const [actionError, setActionError] = useState<ApplicationError | null>(null);
  const [allFiles, setAllFiles] = useState<FileRecord[]>([]);

  const [renameValue, setRenameValue] = useState('');
  const [splitSelected, setSplitSelected] = useState<Set<number>>(new Set());
  const [splitName, setSplitName] = useState('');
  const [mergeTarget, setMergeTarget] = useState<number | null>(null);
  const [mergeSources, setMergeSources] = useState<Set<number>>(new Set());
  const [addFileId, setAddFileId] = useState<number | ''>('');

  const families = useMemo(() => {
    if (versionFamilies.state !== 'ready') return [];
    if (filter === 'all') return versionFamilies.data;
    return versionFamilies.data.filter((summary) => summary.family.status === filter);
  }, [filter, versionFamilies]);

  useEffect(() => {
    let active = true;
    listAllFiles().then(
      (files) => {
        if (active) setAllFiles(files);
      },
      () => {},
    );
    return () => {
      active = false;
    };
  }, [listAllFiles]);

  useEffect(() => {
    if (selectedId == null) {
      return undefined;
    }
    let active = true;
    void getVersionFamily({ familyId: selectedId }).then(
      (next) => {
        if (active) {
          setDetail(next);
          setRenameValue(next.family.displayName);
          setLoadingDetail(false);
        }
      },
      (reason: ApplicationError) => {
        if (active) {
          setDetailError(reason);
          setLoadingDetail(false);
        }
      },
    );
    return () => {
      active = false;
    };
  }, [getVersionFamily, selectedId]);

  const runAction = async (action: () => Promise<void>) => {
    setActionError(null);
    try {
      await action();
    } catch (reason: unknown) {
      setActionError(reason as ApplicationError);
    }
  };

  const closeDetail = () => {
    setSelectedId(null);
    setDetail(null);
    setDetailError(null);
    setActionError(null);
    setSplitSelected(new Set());
    setSplitName('');
    setMergeTarget(null);
    setMergeSources(new Set());
    setAddFileId('');
  };

  const selectFamily = (id: number) => {
    setSelectedId(id);
    setLoadingDetail(true);
    setDetailError(null);
    setSplitSelected(new Set());
    setSplitName('');
    setMergeTarget(null);
    setMergeSources(new Set());
    setAddFileId('');
  };

  const toggleSplit = (fileId: number) => {
    setSplitSelected((current) => {
      const next = new Set(current);
      if (next.has(fileId)) {
        next.delete(fileId);
      } else {
        next.add(fileId);
      }
      return next;
    });
  };

  const otherFamilies = useMemo(() => {
    if (versionFamilies.state !== 'ready' || selectedId == null) return [];
    return versionFamilies.data.filter(
      (summary) =>
        summary.family.id !== selectedId &&
        !['rejected', 'superseded'].includes(summary.family.status),
    );
  }, [versionFamilies, selectedId]);

  const bytes = (value: number) =>
    value < 1024
      ? `${value} B`
      : value < 1024 ** 2
        ? `${(value / 1024).toFixed(1)} KB`
        : `${(value / 1024 ** 2).toFixed(1)} MB`;

  return (
    <div className="page-view versions-page">
      <header className="page-header versions-header">
        <div>
          <h2>{t('versions.title')}</h2>
          <p>{t('versions.description')}</p>
        </div>
        <button
          type="button"
          className="button"
          onClick={() =>
            void runAction(async () => {
              await suggestVersionFamilies({ folderId: null });
            })
          }
          disabled={versionFamilies.state === 'loading'}
        >
          {t('versions.suggest')}
        </button>
      </header>

      <section className="versions-filter" aria-label={t('versions.filterLabel')}>
        {(['all', ...statusOrder] as const).map((key) => (
          <button
            key={key}
            type="button"
            className={filter === key ? 'button button--small is-active' : 'button button--small'}
            onClick={() => setFilter(key)}
          >
            {t(`versions.status.${key}`)}
          </button>
        ))}
      </section>

      {versionFamilies.state === 'loading' ? <LoadingState /> : null}
      {versionFamilies.state === 'error' ? (
        <ErrorState error={versionFamilies.error} onRetry={reload} />
      ) : null}
      {versionFamilies.state === 'ready' && families.length === 0 ? (
        <EmptyState
          icon={<File size={28} />}
          title={t('versions.emptyTitle')}
          body={t('versions.emptyBody')}
        />
      ) : null}
      {versionFamilies.state === 'ready' && families.length > 0 ? (
        <ul className="versions-list">
          {families.map((summary) => (
            <li key={summary.family.id}>
              <button
                type="button"
                className="version-family-card"
                onClick={() => selectFamily(summary.family.id)}
              >
                <span className="version-family-icon">
                  <File size={19} weight="duotone" />
                </span>
                <span className="version-family-main">
                  <strong>{summary.family.displayName}</strong>
                  <small>
                    {t('versions.memberCount', { count: summary.memberCount })} ·{' '}
                    {t(`versions.status.${summary.family.status}`)}
                  </small>
                </span>
                {summary.family.status === 'suggested' ? (
                  <span className="version-family-badge suggested">
                    {t('versions.status.suggested')}
                  </span>
                ) : null}
              </button>
            </li>
          ))}
        </ul>
      ) : null}

      {selectedId != null ? (
        <div className="detail-backdrop" role="presentation" onMouseDown={closeDetail}>
          <aside
            className="file-detail versions-detail"
            role="dialog"
            aria-modal="true"
            aria-labelledby="version-detail-title"
            onMouseDown={(event) => event.stopPropagation()}
          >
            {loadingDetail ? <LoadingState /> : null}
            {detailError ? <ErrorState error={detailError} onRetry={reload} /> : null}
            {detail ? (
              <>
                <header>
                  <div>
                    <span>{t(`versions.status.${detail.family.status}`)}</span>
                    <h3 id="version-detail-title">{detail.family.displayName}</h3>
                  </div>
                  <button type="button" aria-label={t('versions.close')} onClick={closeDetail}>
                    <X size={20} />
                  </button>
                </header>

                <section className="version-members">
                  <h4>{t('versions.members')}</h4>
                  <ol>
                    {detail.members.map((member, index) => (
                      <VersionMemberRow
                        key={member.member.id}
                        index={index}
                        member={member}
                        bytes={bytes}
                        canSplit={detail.family.status !== 'superseded'}
                        splitSelected={splitSelected.has(member.file.id)}
                        onToggleSplit={() => toggleSplit(member.file.id)}
                        onRemove={() =>
                          void runAction(async () => {
                            await removeVersionFamilyMember({
                              familyId: detail.family.id,
                              fileId: member.file.id,
                            });
                          })
                        }
                      />
                    ))}
                  </ol>
                </section>

                {detail.suggestions.length > 0 ? (
                  <section className="version-evidence">
                    <h4>{t('versions.evidence')}</h4>
                    <ul>
                      {formatEvidence(detail.suggestions[0]?.evidence ?? '').map((line) => (
                        <li key={line}>{line}</li>
                      ))}
                    </ul>
                    <small>
                      {t('versions.confidence', {
                        value: Math.round((detail.suggestions[0]?.confidence ?? 0) * 100),
                      })}
                    </small>
                  </section>
                ) : null}

                {detail.family.status === 'suggested' ? (
                  <div className="version-actions">
                    <button
                      type="button"
                      className="button button--small button--primary"
                      onClick={() =>
                        void runAction(async () => {
                          await acceptVersionFamily({ familyId: detail.family.id });
                        })
                      }
                    >
                      {t('versions.accept')}
                    </button>
                    <button
                      type="button"
                      className="button button--small"
                      onClick={() =>
                        void runAction(async () => {
                          await rejectVersionFamily({ familyId: detail.family.id });
                        })
                      }
                    >
                      {t('versions.reject')}
                    </button>
                  </div>
                ) : null}

                {detail.family.status !== 'superseded' ? (
                  <section className="version-section">
                    <h4>{t('versions.rename')}</h4>
                    <div className="version-inline">
                      <input
                        value={renameValue}
                        onChange={(event) => setRenameValue(event.target.value)}
                        placeholder={t('versions.renamePlaceholder')}
                      />
                      <button
                        type="button"
                        className="button button--small"
                        onClick={() =>
                          void runAction(async () => {
                            await renameVersionFamily({
                              familyId: detail.family.id,
                              displayName: renameValue,
                            });
                          })
                        }
                      >
                        <FloppyDisk size={15} />
                        {t('versions.save')}
                      </button>
                    </div>
                  </section>
                ) : null}

                {detail.family.status !== 'superseded' && detail.members.length >= 2 ? (
                  <section className="version-section">
                    <h4>{t('versions.split')}</h4>
                    <p className="version-hint">{t('versions.splitHint')}</p>
                    <input
                      value={splitName}
                      onChange={(event) => setSplitName(event.target.value)}
                      placeholder={t('versions.splitNamePlaceholder')}
                    />
                    <button
                      type="button"
                      className="button button--small"
                      disabled={splitSelected.size === 0 || !splitName.trim()}
                      onClick={() =>
                        void runAction(async () => {
                          await splitVersionFamily({
                            familyId: detail.family.id,
                            fileIds: Array.from(splitSelected),
                            displayName: splitName.trim() || null,
                          });
                          setSplitSelected(new Set());
                          setSplitName('');
                        })
                      }
                    >
                      <ArrowsSplit size={15} />
                      {t('versions.split')}
                    </button>
                  </section>
                ) : null}

                {detail.family.status !== 'superseded' && otherFamilies.length > 0 ? (
                  <section className="version-section">
                    <h4>{t('versions.merge')}</h4>
                    <p className="version-hint">{t('versions.mergeHint')}</p>
                    <select
                      value={mergeTarget ?? ''}
                      onChange={(event) =>
                        setMergeTarget(event.target.value ? Number(event.target.value) : null)
                      }
                    >
                      <option value="">{t('versions.mergeTarget')}</option>
                      {otherFamilies.map((summary) => (
                        <option key={summary.family.id} value={summary.family.id}>
                          {summary.family.displayName}
                        </option>
                      ))}
                    </select>
                    <div className="version-checkbox-list">
                      {otherFamilies.map((summary) => (
                        <label key={summary.family.id}>
                          <input
                            type="checkbox"
                            checked={mergeSources.has(summary.family.id)}
                            onChange={() => {
                              setMergeSources((current) => {
                                const next = new Set(current);
                                if (next.has(summary.family.id)) {
                                  next.delete(summary.family.id);
                                } else {
                                  next.add(summary.family.id);
                                }
                                return next;
                              });
                            }}
                          />
                          {summary.family.displayName}
                        </label>
                      ))}
                    </div>
                    <button
                      type="button"
                      className="button button--small"
                      disabled={mergeTarget == null || mergeSources.size === 0}
                      onClick={() =>
                        void runAction(async () => {
                          if (mergeTarget == null) return;
                          await mergeVersionFamilies({
                            targetFamilyId: mergeTarget,
                            sourceFamilyIds: Array.from(mergeSources),
                          });
                          setMergeTarget(null);
                          setMergeSources(new Set());
                        })
                      }
                    >
                      <ArrowsMerge size={15} />
                      {t('versions.merge')}
                    </button>
                  </section>
                ) : null}

                {detail.family.status !== 'superseded' ? (
                  <section className="version-section">
                    <h4>{t('versions.addMember')}</h4>
                    <div className="version-inline">
                      <select
                        value={addFileId}
                        onChange={(event) =>
                          setAddFileId(event.target.value ? Number(event.target.value) : '')
                        }
                      >
                        <option value="">{t('versions.selectFile')}</option>
                        {allFiles.map((file) => (
                          <option key={file.id} value={file.id}>
                            {file.name} — {file.parentPath}
                          </option>
                        ))}
                      </select>
                      <button
                        type="button"
                        className="button button--small"
                        disabled={addFileId === ''}
                        onClick={() =>
                          void runAction(async () => {
                            if (addFileId === '') return;
                            await addVersionFamilyMember({
                              familyId: detail.family.id,
                              fileId: Number(addFileId),
                            });
                            setAddFileId('');
                          })
                        }
                      >
                        <Plus size={15} />
                        {t('versions.add')}
                      </button>
                    </div>
                  </section>
                ) : null}

                {actionError ? (
                  <p className="deleted-explanation" role="alert">
                    {t(actionError.messageKey)}
                  </p>
                ) : null}
              </>
            ) : null}
          </aside>
        </div>
      ) : null}
    </div>
  );
};

interface VersionMemberRowProps {
  index: number;
  member: VersionFamilyMemberWithFile;
  bytes: (value: number) => string;
  canSplit: boolean;
  splitSelected: boolean;
  onToggleSplit: () => void;
  onRemove: () => void;
}

const VersionMemberRow = ({
  index,
  member,
  bytes,
  canSplit,
  splitSelected,
  onToggleSplit,
  onRemove,
}: VersionMemberRowProps) => {
  const { t } = useTranslation();
  return (
    <li className="version-member-row">
      {canSplit ? (
        <input
          type="checkbox"
          checked={splitSelected}
          onChange={onToggleSplit}
          aria-label={t('versions.selectForSplit')}
        />
      ) : null}
      <span className="version-member-index">{index + 1}</span>
      <span className="version-member-main">
        <strong>{member.file.name}</strong>
        <small title={member.file.normalizedPath}>{member.file.parentPath}</small>
      </span>
      <span className="version-member-meta">
        {bytes(member.file.sizeBytes)} ·{' '}
        {member.file.isPresent ? t('timeline.presence.present') : t('timeline.presence.deleted')}
      </span>
      {canSplit ? (
        <button
          type="button"
          className="button button--small button--danger"
          onClick={onRemove}
          aria-label={t('versions.removeMember')}
        >
          <Trash size={15} />
        </button>
      ) : null}
    </li>
  );
};
