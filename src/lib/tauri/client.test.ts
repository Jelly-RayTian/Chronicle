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

    await client.queryTimelinePage({
      cursor: null,
      pageSize: 50,
      filename: null,
      extension: null,
      eventType: null,
      folderId: null,
      dateFrom: null,
      dateTo: null,
      presence: null,
    });

    expect(invoke).toHaveBeenCalledWith('query_timeline_page', {
      request: {
        cursor: null,
        pageSize: 50,
        filename: null,
        extension: null,
        eventType: null,
        folderId: null,
        dateFrom: null,
        dateTo: null,
        presence: null,
      },
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
    await client.listMonitoringStatuses();
    await client.enableFolderMonitoring(7, 500);
    await client.pauseFolderMonitoring(7);
    await client.resumeFolderMonitoring(7);
    await client.disableFolderMonitoring(7);

    expect(invoke).toHaveBeenNthCalledWith(1, 'register_indexed_folder', { path: 'C:\\Work' });
    expect(invoke).toHaveBeenNthCalledWith(2, 'remove_indexed_folder', {
      request: { folderId: 7, confirmOriginalFilesUntouched: true },
    });
    expect(invoke).toHaveBeenNthCalledWith(3, 'start_folder_scan', { folderId: 7 });
    expect(invoke).toHaveBeenNthCalledWith(4, 'get_scan_task', { scanRunId: 9 });
    expect(invoke).toHaveBeenNthCalledWith(5, 'cancel_folder_scan', { scanRunId: 9 });
    expect(invoke).toHaveBeenNthCalledWith(6, 'list_monitoring_statuses');
    expect(invoke).toHaveBeenNthCalledWith(7, 'enable_folder_monitoring', {
      request: { folderId: 7, coalescingWindowMs: 500 },
    });
    expect(invoke).toHaveBeenNthCalledWith(8, 'pause_folder_monitoring', {
      request: { folderId: 7, coalescingWindowMs: null },
    });
    expect(invoke).toHaveBeenNthCalledWith(9, 'resume_folder_monitoring', {
      request: { folderId: 7, coalescingWindowMs: null },
    });
    expect(invoke).toHaveBeenNthCalledWith(10, 'disable_folder_monitoring', {
      request: { folderId: 7, coalescingWindowMs: null },
    });
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

  it('uses typed history and file-action payloads', async () => {
    const invoke = vi.fn().mockResolvedValue(undefined);
    const client = createTauriClient(invoke as InvokeFunction);

    await client.getFileEventHistory(5);
    await client.getScanHistory(7);
    await client.openTimelineFile(5);
    await client.revealTimelineFile(5);

    expect(invoke).toHaveBeenNthCalledWith(1, 'get_file_event_history', {
      request: { fileId: 5, limit: 100 },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, 'get_scan_history', {
      request: { folderId: 7, limit: 50 },
    });
    expect(invoke).toHaveBeenNthCalledWith(3, 'open_timeline_file', {
      request: { fileId: 5 },
    });
    expect(invoke).toHaveBeenNthCalledWith(4, 'reveal_timeline_file', {
      request: { fileId: 5 },
    });
  });
});
