import { describe, expect, it, vi } from 'vitest';

import { createTauriClient, toApplicationError, type InvokeFunction } from './client';

describe('Tauri client', () => {
  it('sends the typed timeline request through the command boundary', async () => {
    const invoke = vi.fn().mockResolvedValue({
      items: [],
      nextCursor: null,
      hasMore: false,
    });
    const client = createTauriClient(invoke as InvokeFunction);

    await client.queryTimelinePage({ cursor: null, pageSize: 50 });

    expect(invoke).toHaveBeenCalledWith('query_timeline_page', {
      request: { cursor: null, pageSize: 50 },
    });
  });

  it('normalizes unknown rejections into a safe application error', () => {
    expect(toApplicationError(new Error('C:\\private\\chronicle.sqlite3'))).toEqual({
      code: 'unexpected_error',
      messageKey: 'errors.unexpected',
      retryable: false,
    });
  });

  it('uses typed folder and scan command payloads', async () => {
    const invoke = vi.fn().mockResolvedValue(undefined);
    const client = createTauriClient(invoke as InvokeFunction, () => Promise.resolve('C:\\Work'));

    expect(await client.selectIndexedFolder()).toBe('C:\\Work');
    await client.registerIndexedFolder('C:\\Work');
    await client.removeIndexedFolder(7);
    await client.startFolderScan(7);
    await client.getScanTask(9);
    await client.cancelFolderScan(9);

    expect(invoke).toHaveBeenNthCalledWith(1, 'register_indexed_folder', { path: 'C:\\Work' });
    expect(invoke).toHaveBeenNthCalledWith(2, 'remove_indexed_folder', {
      request: { folderId: 7, confirmOriginalFilesUntouched: true },
    });
    expect(invoke).toHaveBeenNthCalledWith(3, 'start_folder_scan', { folderId: 7 });
    expect(invoke).toHaveBeenNthCalledWith(4, 'get_scan_task', { scanRunId: 9 });
    expect(invoke).toHaveBeenNthCalledWith(5, 'cancel_folder_scan', { scanRunId: 9 });
  });

  it('preserves a typed application error payload', () => {
    expect(
      toApplicationError({
        code: 'database_unavailable',
        messageKey: 'errors.databaseUnavailable',
        retryable: true,
      }),
    ).toEqual({
      code: 'database_unavailable',
      messageKey: 'errors.databaseUnavailable',
      retryable: true,
    });
  });
});
