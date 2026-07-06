import { ChartLineDown, CloudSlash, FileLock, FileText } from '@phosphor-icons/react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { setLocale } from '../i18n';
import { tauriClient } from '../lib/tauri/client';
import type { Locale } from '../models';

const privacyItems = [
  ['localOnly', 'localOnlyBody', CloudSlash],
  ['contentsUnread', 'contentsUnreadBody', FileLock],
  ['noAnalytics', 'noAnalyticsBody', ChartLineDown],
] as const;

export const SettingsPage = () => {
  const { t, i18n } = useTranslation();
  const currentLocale: Locale = i18n.resolvedLanguage === 'zh-CN' ? 'zh-CN' : 'en';
  const [exportStatus, setExportStatus] = useState<'idle' | 'exporting' | 'done' | 'error'>('idle');

  const handleExportDiagnostics = async () => {
    setExportStatus('exporting');
    try {
      const path = await tauriClient.exportDiagnostics();
      setExportStatus(path ? 'done' : 'idle');
    } catch {
      setExportStatus('error');
    }
  };

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
      <section className="settings-section" aria-labelledby="diagnostics-heading">
        <div className="settings-section__heading">
          <h3 id="diagnostics-heading">{t('settings.diagnosticsTitle')}</h3>
          <p>{t('settings.diagnosticsDescription')}</p>
        </div>
        <button
          type="button"
          className="diagnostics-export-button"
          onClick={() => void handleExportDiagnostics()}
          disabled={exportStatus === 'exporting'}
          aria-busy={exportStatus === 'exporting'}
        >
          <FileText size={18} weight="regular" aria-hidden="true" />
          {exportStatus === 'exporting'
            ? t('settings.diagnosticsExporting')
            : t('settings.diagnosticsExport')}
        </button>
        {exportStatus === 'done' && (
          <p className="diagnostics-export-status">{t('settings.diagnosticsDone')}</p>
        )}
        {exportStatus === 'error' && (
          <p className="diagnostics-export-status diagnostics-export-status--error">
            {t('settings.diagnosticsError')}
          </p>
        )}
      </section>
    </div>
  );
};
