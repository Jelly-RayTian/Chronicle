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
