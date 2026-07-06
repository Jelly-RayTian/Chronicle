import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { i18n } from '../i18n';
import type { TauriClient } from '../lib/tauri/client';
import type { IndexedFolder } from '../models';
import { App } from './App';

const readyClient = (): TauriClient => ({
  getApplicationInfo: vi.fn().mockResolvedValue({
    name: 'Chronicle',
    version: '0.1.0',
    platform: 'windows',
  }),
  getDatabaseStatus: vi.fn().mockResolvedValue({
    state: 'ready',
    schemaVersion: 1,
  }),
  listIndexedFolders: vi.fn().mockResolvedValue([]),
  selectIndexedFolder: vi.fn().mockResolvedValue(null),
  registerIndexedFolder: vi.fn(),
  removeIndexedFolder: vi.fn(),
  startFolderScan: vi.fn(),
  getScanTask: vi.fn(),
  cancelFolderScan: vi.fn(),
  listMonitoringStatuses: vi.fn().mockResolvedValue([]),
  enableFolderMonitoring: vi.fn(),
  disableFolderMonitoring: vi.fn(),
  pauseFolderMonitoring: vi.fn(),
  resumeFolderMonitoring: vi.fn(),
  queryTimelinePage: vi.fn().mockResolvedValue({
    items: [],
    nextCursor: null,
    hasMore: false,
  }),
  getFileEventHistory: vi.fn().mockResolvedValue([]),
  getScanHistory: vi.fn().mockResolvedValue([]),
  getFilePathHistory: vi.fn().mockResolvedValue([]),
  confirmEvent: vi.fn(),
  rejectEvent: vi.fn(),
  openTimelineFile: vi.fn(),
  revealTimelineFile: vi.fn(),
  listVersionFamilies: vi.fn().mockResolvedValue([]),
  getVersionFamily: vi.fn(),
  suggestVersionFamilies: vi.fn().mockResolvedValue({ familiesCreated: 0 }),
  acceptVersionFamily: vi.fn(),
  rejectVersionFamily: vi.fn(),
  renameVersionFamily: vi.fn(),
  splitVersionFamily: vi.fn(),
  mergeVersionFamilies: vi.fn(),
  addVersionFamilyMember: vi.fn(),
  removeVersionFamilyMember: vi.fn(),
  listAllFiles: vi.fn().mockResolvedValue([]),
  listProjects: vi.fn().mockResolvedValue([]),
  getProject: vi.fn(),
  createProject: vi.fn(),
  updateProject: vi.fn(),
  acceptProject: vi.fn(),
  rejectProject: vi.fn(),
  addProjectMember: vi.fn(),
  removeProjectMember: vi.fn(),
  suggestProjects: vi.fn().mockResolvedValue({ projectsCreated: 0 }),
  getProjectTimeline: vi.fn().mockResolvedValue([]),
  listSessions: vi.fn().mockResolvedValue([]),
  getSession: vi.fn(),
  generateSessions: vi.fn().mockResolvedValue({ sessionsCreated: 0 }),
  updateSession: vi.fn(),
  acceptSession: vi.fn(),
  rejectSession: vi.fn(),
  enableFolderContentIndexing: vi.fn(),
  disableFolderContentIndexing: vi.fn(),
  reindexFolderContent: vi.fn(),
  clearAllContentIndex: vi.fn(),
  searchFiles: vi.fn().mockResolvedValue([]),
});

const folder: IndexedFolder = {
  id: 7,
  normalizedPath: 'C:\\Users\\Guodong\\Documents',
  displayName: 'Documents',
  addedAt: '2026-06-22T05:00:00.000Z',
  lastSuccessfulScanAt: null,
  monitoringEnabled: false,
  availabilityStatus: 'available',
  lastCheckedAt: '2026-06-22T05:00:00.000Z',
  contentIndexingEnabled: false,
  contentIndexingExtensions: 'txt,md',
  contentIndexingMaxBytes: 1048576,
  contentIndexingExclusionPatterns: '.env,.env.*',
};

const watcherStatus = {
  folderId: 7,
  desiredState: 'disabled' as const,
  runtimeState: 'stopped' as const,
  coalescingWindowMs: 750,
  lastStartedAt: null,
  lastStoppedAt: null,
  lastEventAt: null,
  lastErrorAt: null,
  lastErrorKind: null,
  lastErrorMessage: null,
  eventsRecorded: 0,
  eventsDropped: 0,
};

describe('Chronicle application', () => {
  beforeEach(async () => {
    globalThis.localStorage.clear();
    await i18n.changeLanguage('en');
  });

  it('renders the application shell and real native status', async () => {
    render(<App client={readyClient()} />);

    expect(screen.getByRole('heading', { name: 'Chronicle' })).toBeInTheDocument();
    expect(await screen.findByText('Native connection')).toBeInTheDocument();
    expect(screen.getAllByText('Ready')).toHaveLength(2);
  });

  it('navigates between all seven pages', async () => {
    const user = userEvent.setup();
    render(<App client={readyClient()} />);

    await user.click(screen.getByRole('button', { name: 'Indexed folders' }));
    expect(screen.getByRole('heading', { name: 'Indexed folders' })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Versions' }));
    expect(screen.getByRole('heading', { name: 'Versions' })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Projects' }));
    expect(screen.getByRole('heading', { name: 'Projects' })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Sessions' }));
    expect(screen.getByRole('heading', { name: 'Sessions' })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Search' }));
    expect(screen.getByRole('heading', { name: 'Search' })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Settings' }));
    expect(screen.getByRole('heading', { name: 'Settings' })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Timeline' }));
    expect(screen.getByRole('heading', { name: 'Timeline' })).toBeInTheDocument();
  });

  it('shows the empty timeline returned by the native client', async () => {
    render(<App client={readyClient()} />);

    expect(await screen.findByText('No file activity yet')).toBeInTheDocument();
    expect(screen.queryByText(/sample|demo event/i)).not.toBeInTheDocument();
  });

  it('shows the empty indexed-folder state', async () => {
    const user = userEvent.setup();
    render(<App client={readyClient()} />);

    await user.click(screen.getByRole('button', { name: 'Indexed folders' }));

    expect(await screen.findByText('No indexed folders')).toBeInTheDocument();
    expect(screen.getByText(/never delete your original files/i)).toBeInTheDocument();
  });

  it('registers a selected folder and reports nested-root overlap', async () => {
    const user = userEvent.setup();
    const client = readyClient();
    const registerIndexedFolder = vi.fn().mockResolvedValue({
      folder,
      nestedWarnings: [
        {
          existingFolderId: 3,
          existingPath: 'C:\\Users\\Guodong',
          relationship: 'inside_existing',
        },
      ],
    });
    client.selectIndexedFolder = vi.fn().mockResolvedValue(folder.normalizedPath);
    client.registerIndexedFolder = registerIndexedFolder;
    client.listIndexedFolders = vi.fn().mockResolvedValueOnce([]).mockResolvedValue([folder]);
    render(<App client={client} />);
    await user.click(screen.getByRole('button', { name: 'Indexed folders' }));
    await user.click(screen.getByRole('button', { name: 'Add folder' }));

    expect(await screen.findByText('Documents')).toBeInTheDocument();
    expect(screen.getByText(/overlaps an existing indexed root/i)).toBeInTheDocument();
    expect(registerIndexedFolder).toHaveBeenCalledWith(folder.normalizedPath);
  });

  it('requires an explicit confirmation before removing only Chronicle metadata', async () => {
    const user = userEvent.setup();
    const client = readyClient();
    const removeIndexedFolder = vi.fn().mockResolvedValue(undefined);
    client.listIndexedFolders = vi.fn().mockResolvedValue([folder]);
    client.removeIndexedFolder = removeIndexedFolder;
    render(<App client={client} />);
    await user.click(screen.getByRole('button', { name: 'Indexed folders' }));
    await user.click(await screen.findByRole('button', { name: 'Remove index' }));

    expect(screen.getByRole('dialog')).toHaveTextContent(/original files and folders will remain/i);
    expect(removeIndexedFolder).not.toHaveBeenCalled();
    await user.click(screen.getByRole('button', { name: 'Remove Chronicle index' }));
    expect(removeIndexedFolder).toHaveBeenCalledWith(7);
  });

  it('enables, pauses, resumes, and disables explicit folder monitoring', async () => {
    const user = userEvent.setup();
    const client = readyClient();
    client.listIndexedFolders = vi.fn().mockResolvedValue([folder]);
    client.listMonitoringStatuses = vi.fn().mockResolvedValue([watcherStatus]);
    const enableFolderMonitoring = vi.fn().mockResolvedValue({
      ...watcherStatus,
      desiredState: 'enabled',
      runtimeState: 'running',
    });
    const pauseFolderMonitoring = vi.fn().mockResolvedValue({
      ...watcherStatus,
      desiredState: 'paused',
      runtimeState: 'paused',
    });
    const resumeFolderMonitoring = vi.fn().mockResolvedValue({
      ...watcherStatus,
      desiredState: 'enabled',
      runtimeState: 'running',
    });
    const disableFolderMonitoring = vi.fn().mockResolvedValue(watcherStatus);
    client.enableFolderMonitoring = enableFolderMonitoring;
    client.pauseFolderMonitoring = pauseFolderMonitoring;
    client.resumeFolderMonitoring = resumeFolderMonitoring;
    client.disableFolderMonitoring = disableFolderMonitoring;

    render(<App client={client} />);
    await user.click(screen.getByRole('button', { name: 'Indexed folders' }));
    await user.click(await screen.findByRole('button', { name: 'Enable monitoring' }));
    expect(enableFolderMonitoring).toHaveBeenCalledWith(7);
    await user.click(await screen.findByRole('button', { name: 'Pause' }));
    expect(pauseFolderMonitoring).toHaveBeenCalledWith(7);
    await user.click(await screen.findByRole('button', { name: 'Resume' }));
    expect(resumeFolderMonitoring).toHaveBeenCalledWith(7);
    await user.click(await screen.findByRole('button', { name: 'Disable' }));
    expect(disableFolderMonitoring).toHaveBeenCalledWith(7);
  });

  it('shows an understandable database failure without exposing details', async () => {
    const client = readyClient();
    client.getDatabaseStatus = vi.fn().mockRejectedValue({
      code: 'database_unavailable',
      messageKey: 'errors.databaseUnavailable',
      retryable: true,
    });
    render(<App client={client} />);

    expect(
      await screen.findByText(
        'Chronicle could not open its local database. Your original files were not changed.',
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText(/chronicle\.sqlite/i)).not.toBeInTheDocument();
  });

  it('switches the complete application to Simplified Chinese', async () => {
    const user = userEvent.setup();
    render(<App client={readyClient()} />);

    await user.click(screen.getByRole('button', { name: 'Settings' }));
    await user.click(screen.getByRole('button', { name: '简体中文' }));

    expect(await screen.findByRole('heading', { name: '设置' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '时间线' })).toBeInTheDocument();
    expect(globalThis.localStorage.getItem('chronicle.locale')).toBe('zh-CN');
  });

  it('shows a stable loading state while native data resolves', () => {
    const pending = new Promise<never>(() => undefined);
    const client: TauriClient = {
      getApplicationInfo: () => pending,
      getDatabaseStatus: () => pending,
      listIndexedFolders: () => pending,
      selectIndexedFolder: () => pending,
      registerIndexedFolder: () => pending,
      removeIndexedFolder: () => pending,
      startFolderScan: () => pending,
      getScanTask: () => pending,
      cancelFolderScan: () => pending,
      listMonitoringStatuses: () => pending,
      enableFolderMonitoring: () => pending,
      disableFolderMonitoring: () => pending,
      pauseFolderMonitoring: () => pending,
      resumeFolderMonitoring: () => pending,
      queryTimelinePage: () => pending,
      getFileEventHistory: () => pending,
      getScanHistory: () => pending,
      getFilePathHistory: () => pending,
      confirmEvent: () => pending,
      rejectEvent: () => pending,
      openTimelineFile: () => pending,
      revealTimelineFile: () => pending,
      listVersionFamilies: () => pending,
      getVersionFamily: () => pending,
      suggestVersionFamilies: () => pending,
      acceptVersionFamily: () => pending,
      rejectVersionFamily: () => pending,
      renameVersionFamily: () => pending,
      splitVersionFamily: () => pending,
      mergeVersionFamilies: () => pending,
      addVersionFamilyMember: () => pending,
      removeVersionFamilyMember: () => pending,
      listAllFiles: () => pending,
      listProjects: () => pending,
      getProject: () => pending,
      createProject: () => pending,
      updateProject: () => pending,
      acceptProject: () => pending,
      rejectProject: () => pending,
      addProjectMember: () => pending,
      removeProjectMember: () => pending,
      suggestProjects: () => pending,
      getProjectTimeline: () => pending,
      listSessions: () => pending,
      getSession: () => pending,
      generateSessions: () => pending,
      updateSession: () => pending,
      acceptSession: () => pending,
      rejectSession: () => pending,
      enableFolderContentIndexing: () => pending,
      disableFolderContentIndexing: () => pending,
      reindexFolderContent: () => pending,
      clearAllContentIndex: () => pending,
      searchFiles: () => pending,
    };

    render(<App client={client} />);

    expect(screen.getByText('Loading local data')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Settings' })).toBeEnabled();
  });

  it('groups real events, debounces filename search, and disables deleted file actions', async () => {
    const user = userEvent.setup();
    const client = readyClient();
    const now = new Date().toISOString();
    const deletedItem = {
      event: {
        id: 12,
        fileId: 5,
        indexedFolderId: 7,
        eventType: 'deleted',
        detectedAt: now,
        filesystemTime: null,
        oldPath: 'C:\\Work\\gone.pdf',
        newPath: null,
        confidence: 1,
        eventSource: 'reconciliation',
      },
      file: {
        id: 5,
        indexedFolderId: 7,
        normalizedPath: 'C:\\Work\\gone.pdf',
        name: 'gone.pdf',
        parentPath: 'C:\\Work',
        extension: 'pdf',
        sizeBytes: 42,
        filesystemCreatedAt: null,
        filesystemModifiedAt: now,
        firstIndexedAt: now,
        lastSeenAt: now,
        isPresent: false,
      },
      folderName: 'Work',
      folderPath: 'C:\\Work',
    };
    client.listIndexedFolders = vi
      .fn()
      .mockResolvedValue([{ ...folder, id: 7, displayName: 'Work' }]);
    const queryTimelinePage = vi
      .fn()
      .mockResolvedValue({ items: [deletedItem], nextCursor: null, hasMore: false });
    client.queryTimelinePage = queryTimelinePage;
    client.getFileEventHistory = vi.fn().mockResolvedValue([deletedItem.event]);
    render(<App client={client} />);

    expect(await screen.findByRole('heading', { name: 'Today' })).toBeInTheDocument();
    await user.type(screen.getByPlaceholderText('Search filenames'), 'gone');
    await waitFor(
      () =>
        expect(queryTimelinePage).toHaveBeenCalledWith(
          expect.objectContaining({ filename: 'gone' }),
        ),
      { timeout: 1000 },
    );
    await user.click(screen.getByRole('button', { name: 'Reset filters' }));
    await waitFor(() =>
      expect(queryTimelinePage).toHaveBeenCalledWith(expect.objectContaining({ filename: null })),
    );
    await user.click(screen.getByRole('button', { name: /gone\.pdf/i }));
    expect(screen.getByRole('button', { name: 'Open file' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Show in folder' })).toBeDisabled();
    expect(screen.getByText(/no longer present/i)).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Event history' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Scan history' })).toBeInTheDocument();
  });
});
