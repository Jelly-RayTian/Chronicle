import { render, screen } from '@testing-library/react';
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
  queryTimelinePage: vi.fn().mockResolvedValue({
    items: [],
    nextCursor: null,
    hasMore: false,
  }),
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

  it('navigates between all three pages', async () => {
    const user = userEvent.setup();
    render(<App client={readyClient()} />);

    await user.click(screen.getByRole('button', { name: 'Indexed folders' }));
    expect(screen.getByRole('heading', { name: 'Indexed folders' })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Settings' }));
    expect(screen.getByRole('heading', { name: 'Settings' })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Timeline' }));
    expect(screen.getByRole('heading', { name: 'Timeline' })).toBeInTheDocument();
  });

  it('shows the empty timeline returned by the native client', async () => {
    render(<App client={readyClient()} />);

    expect(await screen.findByText('Your timeline is ready')).toBeInTheDocument();
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
      queryTimelinePage: () => pending,
    };

    render(<App client={client} />);

    expect(screen.getByText('Loading local data')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Settings' })).toBeEnabled();
  });
});
