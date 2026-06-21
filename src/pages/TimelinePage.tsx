import { ClockCounterClockwise } from '@phosphor-icons/react';
import { useTranslation } from 'react-i18next';

import { useAppData } from '../app/AppDataContext';
import { EmptyState, ErrorState, LoadingState } from '../components/states/ContentState';

export const TimelinePage = () => {
  const { t } = useTranslation();
  const { timeline, reload } = useAppData();

  return (
    <div className="page-view">
      <header className="page-header">
        <h2>{t('timeline.title')}</h2>
        <p>{t('timeline.description')}</p>
      </header>
      {timeline.state === 'loading' ? <LoadingState /> : null}
      {timeline.state === 'error' ? <ErrorState error={timeline.error} onRetry={reload} /> : null}
      {timeline.state === 'ready' && timeline.data.items.length === 0 ? (
        <EmptyState
          icon={<ClockCounterClockwise size={28} weight="regular" />}
          title={t('timeline.emptyTitle')}
          body={t('timeline.emptyBody')}
        />
      ) : null}
      {timeline.state === 'ready' && timeline.data.items.length > 0 ? (
        <ol className="timeline-list">
          {timeline.data.items.map((event) => (
            <li key={event.id}>{event.eventType}</li>
          ))}
        </ol>
      ) : null}
    </div>
  );
};
