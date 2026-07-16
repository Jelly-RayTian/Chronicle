export type DateGroup = 'today' | 'yesterday' | 'thisWeek' | 'thisMonth' | 'older';

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
  if (event >= weekStart) return 'thisWeek';
  const monthStart = new Date(today.getFullYear(), today.getMonth(), 1);
  if (event >= monthStart) return 'thisMonth';
  return 'older';
};
