import { describe, expect, it } from 'vitest';

import { timelineDateGroup } from './timelineGrouping';

describe('timeline grouping', () => {
  it('groups events into today, yesterday, this week, and earlier', () => {
    const now = new Date('2026-06-24T12:00:00.000Z');
    expect(timelineDateGroup('2026-06-24T01:00:00.000Z', now)).toBe('today');
    expect(timelineDateGroup('2026-06-23T12:00:00.000Z', now)).toBe('yesterday');
    expect(timelineDateGroup('2026-06-22T12:00:00.000Z', now)).toBe('thisWeek');
    expect(timelineDateGroup('2026-06-14T12:00:00.000Z', now)).toBe('earlier');
  });
});
