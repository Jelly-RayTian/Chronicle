import { ChartLineDown, CloudSlash, FileLock } from '@phosphor-icons/react';
import { useTranslation } from 'react-i18next';

import { setLocale } from '../i18n';
import type { Locale } from '../models';

const privacyItems = [
  ['localOnly', 'localOnlyBody', CloudSlash],
  ['contentsUnread', 'contentsUnreadBody', FileLock],
  ['noAnalytics', 'noAnalyticsBody', ChartLineDown],
] as const;

export const SettingsPage = () => {
  const { t, i18n } = useTranslation();
  const currentLocale: Locale = i18n.resolvedLanguage === 'zh-CN' ? 'zh-CN' : 'en';

  return (
    <div className="page-view">
      <header className="page-header">
        <h2>{t('settings.title')}</h2>
        <p>{t('settings.description')}</p>
      </header>
      <section className="settings-section" aria-labelledby="language-heading">
        <div className="settings-section__heading">
          <h3 id="language-heading">{t('settings.language')}</h3>
          <p>{t('settings.languageDescription')}</p>
        </div>
        <div className="segmented-control" role="group" aria-label={t('settings.language')}>
          {(['zh-CN', 'en'] as const).map((locale) => (
            <button
              key={locale}
              type="button"
              aria-pressed={currentLocale === locale}
              onClick={() => void setLocale(locale)}
            >
              {locale === 'zh-CN' ? t('settings.chinese') : t('settings.english')}
            </button>
          ))}
        </div>
      </section>
      <section className="settings-section" aria-labelledby="privacy-heading">
        <div className="settings-section__heading">
          <h3 id="privacy-heading">{t('settings.privacyTitle')}</h3>
        </div>
        <div className="privacy-list">
          {privacyItems.map(([title, body, Icon]) => (
            <article className="privacy-item" key={title}>
              <Icon size={22} weight="regular" aria-hidden="true" />
              <div>
                <strong>{t(`settings.${title}`)}</strong>
                <p>{t(`settings.${body}`)}</p>
              </div>
            </article>
          ))}
        </div>
      </section>
    </div>
  );
};
