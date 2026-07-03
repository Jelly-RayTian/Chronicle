export type DateGroup = 'today' | 'yesterday' | 'thisWeek' | 'earlier';

const startOfDay = (value: Date) =>
  new Date(value.getFullYear(), value.getMonth(), value.getDate());

export const timelineDateGroup = (value: string, now = new Date()): DateGroup => {
  const event = startOfDay(new Date(value));
  const today = startOfDay(now);
  const days = Math.floor((today.getTime() - event.getTime()) / 86_400_000);
  if (days <= 0) return 'today';
  if (days === 1) return 'yesterday';
  const weekStart = new Date(today);
  weekStart.setDate(today.getDate() - ((today.getDay() + 6) % 7));
  return event >= weekStart ? 'thisWeek' : 'earlier';
};
