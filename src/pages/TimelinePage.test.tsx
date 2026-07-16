import { describe, expect, it } from 'vitest';

import { timelineDateGroup } from './timelineGrouping';

describe('timeline grouping', () => {
  const saturday = new Date('2026-07-11T12:00:00.000Z');

  it('groups events into today, yesterday, thisWeek, thisMonth, and older', () => {
    expect(timelineDateGroup('2026-07-11T01:00:00.000Z', saturday)).toBe('today');
    expect(timelineDateGroup('2026-07-10T12:00:00.000Z', saturday)).toBe('yesterday');
    expect(timelineDateGroup('2026-07-09T12:00:00.000Z', saturday)).toBe('thisWeek');
    expect(timelineDateGroup('2026-07-02T12:00:00.000Z', saturday)).toBe('thisMonth');
    expect(timelineDateGroup('2026-06-14T12:00:00.000Z', saturday)).toBe('older');
  });

  it('groups Wednesday in same week as a Thursday reference point', () => {
    const thursday = new Date('2026-07-09T12:00:00.000Z');
    expect(timelineDateGroup('2026-07-08T12:00:00.000Z', thursday)).toBe('yesterday');
    expect(timelineDateGroup('2026-07-06T12:00:00.000Z', thursday)).toBe('thisWeek');
  });
});
