import { WarningCircle } from '@phosphor-icons/react';
import { useTranslation } from 'react-i18next';

import type { ApplicationError } from '../../models';

export const LoadingState = () => {
  const { t } = useTranslation();
  return (
    <section className="content-state content-state--loading" role="status" aria-live="polite">
      <div className="skeleton-mark" aria-hidden="true" />
      <div className="content-state__copy">
        <strong>{t('state.loading')}</strong>
        <span>{t('state.loadingBody')}</span>
      </div>
      <div className="skeleton-lines" aria-hidden="true">
        <span />
        <span />
      </div>
    </section>
  );
};

interface EmptyStateProps {
  icon: React.ReactNode;
  title: string;
  body: string;
}

export const EmptyState = ({ icon, title, body }: EmptyStateProps) => (
  <section className="content-state content-state--empty">
    <div className="content-state__icon" aria-hidden="true">
      {icon}
    </div>
    <div className="content-state__copy">
      <strong>{title}</strong>
      <span>{body}</span>
    </div>
  </section>
);

interface ErrorStateProps {
  error: ApplicationError;
  onRetry?: () => void;
}

export const ErrorState = ({ error, onRetry }: ErrorStateProps) => {
  const { t } = useTranslation();
  return (
    <section className="content-state content-state--error" role="alert">
      <WarningCircle size={22} weight="regular" aria-hidden="true" />
      <div className="content-state__copy">
        <strong>{t(error.messageKey)}</strong>
        <span>{error.code}</span>
      </div>
      {error.retryable && onRetry ? (
        <button className="button button--secondary" type="button" onClick={onRetry}>
          {t('state.retry')}
        </button>
      ) : null}
    </section>
  );
};
