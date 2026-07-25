import {
  FileText,
  FolderPlus,
  FolderSimpleDashed,
  Pause,
  Play,
  ShieldCheck,
  Stop,
  Warning,
} from '@phosphor-icons/react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useAppData } from '../app/AppDataContext';
import { EmptyState, ErrorState, LoadingState } from '../components/states/ContentState';
import type { IndexedFolder, NestedFolderWarning } from '../models';
import './IndexedFoldersPage.css';

interface ContentEditingState {
  extensions: string;
  maxBytes: string;
  exclusions: string;
  expanded: boolean;
}

const LARGE_FOLDER_WARNING_FILES = 10_000;
const MAX_CONTENT_INDEX_BYTES = 8_388_608;

const formatTime = (value: string | null, locale: string, never: string) =>
  value
    ? new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeStyle: 'short' }).format(
        new Date(value),
      )
    : never;

export const IndexedFoldersPage = () => {
  const { t, i18n } = useTranslation();
  const {
    indexedFolders,
    folderActionError,
    scans,
    watcherStatuses,
    addIndexedFolder,
    removeIndexedFolder,
    startFolderScan,
    cancelFolderScan,
    enableFolderMonitoring,
    disableFolderMonitoring,
    pauseFolderMonitoring,
    resumeFolderMonitoring,
    enableFolderContentIndexing,
    disableFolderContentIndexing,
    reindexFolderContent,
    clearFolderActionError,
    reload,
  } = useAppData();
  const [adding, setAdding] = useState(false);
  const [removeTarget, setRemoveTarget] = useState<IndexedFolder | null>(null);
  const [nestedWarnings, setNestedWarnings] = useState<NestedFolderWarning[]>([]);
  const [contentEditing, setContentEditing] = useState<Record<number, ContentEditingState>>({});

  const addFolder = async () => {
    setAdding(true);
    const result = await addIndexedFolder();
    setNestedWarnings(result?.nestedWarnings ?? []);
    setAdding(false);
  };

  const confirmRemoval = async () => {
    if (!removeTarget) return;
    const removed = await removeIndexedFolder(removeTarget.id);
    if (removed) setRemoveTarget(null);
  };

  const ensureEditingState = (folder: IndexedFolder): ContentEditingState =>
    contentEditing[folder.id] ?? {
      extensions: folder.contentIndexingExtensions,
      maxBytes: String(folder.contentIndexingMaxBytes),
      exclusions: folder.contentIndexingExclusionPatterns,
      expanded: false,
    };

  const handleEnableContentIndexing = async (folder: IndexedFolder) => {
    const state = ensureEditingState(folder);
    await enableFolderContentIndexing({
      folderId: folder.id,
      extensions: state.extensions,
      maxBytes: Number(state.maxBytes),
      exclusionPatterns: state.exclusions,
    });
    setContentEditing((current) => {
      const next = { ...current };
      delete next[folder.id];
      return next;
    });
  };

  const handleDisableContentIndexing = async (folder: IndexedFolder) => {
    await disableFolderContentIndexing({ folderId: folder.id });
    setContentEditing((current) => {
      const next = { ...current };
      delete next[folder.id];
      return next;
    });
  };

  const handleReindexContent = async (folder: IndexedFolder) => {
    await reindexFolderContent({ folderId: folder.id });
  };

  const updateEditingState = (folder: IndexedFolder, patch: Partial<ContentEditingState>) => {
    setContentEditing((current) => ({
      ...current,
      [folder.id]: { ...ensureEditingState(folder), ...patch },
    }));
  };

  return (
    <div className="page-view folders-page">
      <header className="page-header folders-header">
        <div>
          <h2>{t('folders.title')}</h2>
          <p>{t('folders.description')}</p>
        </div>
        <button
          className="button button--primary"
          type="button"
          onClick={() => void addFolder()}
          disabled={adding}
        >
          <FolderPlus size={18} aria-hidden="true" />
          {adding ? t('folders.choosing') : t('folders.add')}
        </button>
      </header>

      <aside className="privacy-note">
        <ShieldCheck size={20} aria-hidden="true" />
        <span>{t('folders.metadataOnly')}</span>
      </aside>

      {folderActionError ? (
        <div className="inline-alert inline-alert--error" role="alert">
          <Warning size={18} aria-hidden="true" />
          <span>{t(folderActionError.messageKey)}</span>
          <button type="button" onClick={clearFolderActionError}>
            {t('folders.dismiss')}
          </button>
        </div>
      ) : null}
      {nestedWarnings.length > 0 ? (
        <div className="inline-alert" role="status">
          <Warning size={18} aria-hidden="true" />
          <span>{t('folders.nestedWarning', { path: nestedWarnings[0]?.existingPath })}</span>
          <button type="button" onClick={() => setNestedWarnings([])}>
            {t('folders.dismiss')}
          </button>
        </div>
      ) : null}

      {indexedFolders.state === 'loading' ? <LoadingState /> : null}
      {indexedFolders.state === 'error' ? (
        <ErrorState error={indexedFolders.error} onRetry={reload} />
      ) : null}
      {indexedFolders.state === 'ready' && indexedFolders.data.length === 0 ? (
        <EmptyState
          icon={<FolderSimpleDashed size={28} />}
          title={t('folders.emptyTitle')}
          body={t('folders.emptyBody')}
        />
      ) : null}
      {indexedFolders.state === 'ready' && indexedFolders.data.length > 0 ? (
        <div className="folder-grid">
          {indexedFolders.data.map((folder) => {
            const scan = scans[folder.id];
            const running = scan?.status === 'running';
            const watcher = watcherStatuses[folder.id];
            const monitoringState = watcher?.desiredState ?? 'disabled';
            const runtimeState = watcher?.runtimeState ?? 'stopped';
            const monitoringActive = monitoringState !== 'disabled';
            return (
              <article className="folder-card" key={folder.id}>
                <div className="folder-card__title">
                  <div>
                    <h3>{folder.displayName}</h3>
                    <p title={folder.normalizedPath}>{folder.normalizedPath}</p>
                  </div>
                  <span className="availability" data-status={folder.availabilityStatus}>
                    {t(`folders.availability.${folder.availabilityStatus}`)}
                  </span>
                </div>
                <dl className="folder-facts">
                  <div>
                    <dt>{t('folders.added')}</dt>
                    <dd>{formatTime(folder.addedAt, i18n.language, t('folders.never'))}</dd>
                  </div>
                  <div>
                    <dt>{t('folders.lastScan')}</dt>
                    <dd>
                      {formatTime(folder.lastSuccessfulScanAt, i18n.language, t('folders.never'))}
                    </dd>
                  </div>
                  <div>
                    <dt>{t('folders.monitoring')}</dt>
                    <dd>{t(`folders.monitoringStates.${monitoringState}`)}</dd>
                  </div>
                </dl>
                {watcher ? (
                  <div className="monitoring-status" data-status={runtimeState}>
                    <div>
                      <strong>{t(`folders.watcher.${runtimeState}`)}</strong>
                      <span>
                        {t('folders.watcher.counts', {
                          events: watcher.eventsRecorded,
                          dropped: watcher.eventsDropped,
                        })}
                      </span>
                    </div>
                    {watcher.lastErrorKind ? (
                      <p>{t('folders.watcher.lastError', { kind: watcher.lastErrorKind })}</p>
                    ) : null}
                  </div>
                ) : null}
                {scan ? (
                  <div className="scan-status" data-status={scan.status}>
                    <div>
                      <strong>{t(`folders.scan.${scan.status}`)}</strong>
                      <span>
                        {t('folders.scan.counts', {
                          files: scan.filesSeen,
                          warnings: scan.warningCount,
                          errors: scan.errorCount,
                        })}
                      </span>
                    </div>
                    {running ? <progress aria-label={t('folders.scan.running')} /> : null}
                    {scan.filesSeen >= LARGE_FOLDER_WARNING_FILES ? (
                      <p className="large-folder-warning" role="status">
                        <Warning size={16} aria-hidden="true" />
                        {t('folders.scan.largeFolderWarning', {
                          count: scan.filesSeen.toLocaleString(i18n.language),
                        })}
                      </p>
                    ) : null}
                  </div>
                ) : null}
                <div className="folder-actions">
                  {!monitoringActive ? (
                    <button
                      className="button"
                      type="button"
                      disabled={folder.availabilityStatus !== 'available'}
                      onClick={() => void enableFolderMonitoring(folder.id)}
                    >
                      <Play size={16} aria-hidden="true" />
                      {t('folders.enableMonitoring')}
                    </button>
                  ) : monitoringState === 'paused' ? (
                    <button
                      className="button"
                      type="button"
                      disabled={folder.availabilityStatus !== 'available'}
                      onClick={() => void resumeFolderMonitoring(folder.id)}
                    >
                      <Play size={16} aria-hidden="true" />
                      {t('folders.resumeMonitoring')}
                    </button>
                  ) : (
                    <button
                      className="button"
                      type="button"
                      onClick={() => void pauseFolderMonitoring(folder.id)}
                    >
                      <Pause size={16} aria-hidden="true" />
                      {t('folders.pauseMonitoring')}
                    </button>
                  )}
                  {monitoringActive ? (
                    <button
                      className="button"
                      type="button"
                      onClick={() => void disableFolderMonitoring(folder.id)}
                    >
                      <Stop size={16} aria-hidden="true" />
                      {t('folders.disableMonitoring')}
                    </button>
                  ) : null}
                  {running ? (
                    <button
                      className="button"
                      type="button"
                      onClick={() => void cancelFolderScan(scan.scanRunId)}
                    >
                      {t('folders.cancel')}
                    </button>
                  ) : (
                    <button
                      className="button"
                      type="button"
                      disabled={folder.availabilityStatus !== 'available'}
                      onClick={() => void startFolderScan(folder.id)}
                    >
                      {t('folders.scanNow')}
                    </button>
                  )}
                  <button
                    className="button button--danger"
                    type="button"
                    disabled={running}
                    onClick={() => setRemoveTarget(folder)}
                  >
                    {t('folders.remove')}
                  </button>
                </div>
                <div className="content-indexing-panel">
                  <div className="content-indexing-header">
                    <FileText size={16} aria-hidden="true" />
                    <strong>
                      {folder.contentIndexingEnabled
                        ? t('folders.contentIndexing.enabled')
                        : t('folders.contentIndexing.disabled')}
                    </strong>
                    {folder.contentIndexingEnabled ? (
                      <>
                        <button
                          className="button"
                          type="button"
                          onClick={() => void handleReindexContent(folder)}
                        >
                          {t('folders.contentIndexing.reindex')}
                        </button>
                        <button
                          className="button"
                          type="button"
                          onClick={() =>
                            updateEditingState(folder, {
                              expanded: !ensureEditingState(folder).expanded,
                            })
                          }
                        >
                          {ensureEditingState(folder).expanded
                            ? t('folders.contentIndexing.hide')
                            : t('folders.contentIndexing.edit')}
                        </button>
                        <button
                          className="button button--danger"
                          type="button"
                          onClick={() => void handleDisableContentIndexing(folder)}
                        >
                          {t('folders.contentIndexing.disable')}
                        </button>
                      </>
                    ) : (
                      <button
                        className="button"
                        type="button"
                        onClick={() => void handleEnableContentIndexing(folder)}
                      >
                        {t('folders.contentIndexing.enable')}
                      </button>
                    )}
                  </div>
                  {folder.contentIndexingEnabled ? (
                    <p className="content-indexing-note">
                      {t('folders.contentIndexing.localOnly')}
                    </p>
                  ) : (
                    <p className="content-indexing-note">
                      {t('folders.contentIndexing.disabledNote')}
                    </p>
                  )}
                  {folder.contentIndexingEnabled && ensureEditingState(folder).expanded ? (
                    <div className="content-indexing-form">
                      <label>
                        <span>{t('folders.contentIndexing.extensions')}</span>
                        <input
                          type="text"
                          value={ensureEditingState(folder).extensions}
                          onChange={(event) =>
                            updateEditingState(folder, { extensions: event.target.value })
                          }
                        />
                      </label>
                      <label>
                        <span>{t('folders.contentIndexing.maxBytes')}</span>
                        <input
                          type="number"
                          min={1024}
                          max={MAX_CONTENT_INDEX_BYTES}
                          step={1024}
                          value={ensureEditingState(folder).maxBytes}
                          onChange={(event) =>
                            updateEditingState(folder, { maxBytes: event.target.value })
                          }
                        />
                        <small>{t('folders.contentIndexing.maxBytesLimit')}</small>
                      </label>
                      <label>
                        <span>{t('folders.contentIndexing.exclusions')}</span>
                        <input
                          type="text"
                          value={ensureEditingState(folder).exclusions}
                          onChange={(event) =>
                            updateEditingState(folder, { exclusions: event.target.value })
                          }
                        />
                      </label>
                      <button
                        className="button"
                        type="button"
                        onClick={() => void handleEnableContentIndexing(folder)}
                      >
                        {t('folders.contentIndexing.save')}
                      </button>
                    </div>
                  ) : null}
                </div>
              </article>
            );
          })}
        </div>
      ) : null}

      {removeTarget ? (
        <div className="modal-backdrop" role="presentation">
          <section
            className="confirmation-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="remove-title"
          >
            <h3 id="remove-title">
              {t('folders.removeTitle', { name: removeTarget.displayName })}
            </h3>
            <p>{t('folders.removeBody')}</p>
            <div className="confirmation-dialog__actions">
              <button className="button" type="button" onClick={() => setRemoveTarget(null)}>
                {t('folders.keep')}
              </button>
              <button
                className="button button--danger-solid"
                type="button"
                onClick={() => void confirmRemoval()}
              >
                {t('folders.confirmRemove')}
              </button>
            </div>
          </section>
        </div>
      ) : null}
    </div>
  );
};
