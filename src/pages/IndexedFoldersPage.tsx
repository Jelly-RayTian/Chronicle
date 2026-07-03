import {
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
    clearFolderActionError,
    reload,
  } = useAppData();
  const [adding, setAdding] = useState(false);
  const [removeTarget, setRemoveTarget] = useState<IndexedFolder | null>(null);
  const [nestedWarnings, setNestedWarnings] = useState<NestedFolderWarning[]>([]);

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
