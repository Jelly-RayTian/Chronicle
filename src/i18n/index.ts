import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import type { Locale } from '../models';
import { en } from './en';
import { zhCN } from './zh-CN';

const localeStorageKey = 'chronicle.locale';

export const detectLocale = (systemLocale: string): Locale =>
  systemLocale.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en';

const storedLocale = (): Locale | null => {
  const value = globalThis.localStorage?.getItem(localeStorageKey);
  return value === 'zh-CN' || value === 'en' ? value : null;
};

export const initialLocale = (): Locale =>
  storedLocale() ?? detectLocale(globalThis.navigator?.language ?? 'en');

void i18n.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    'zh-CN': { translation: zhCN },
  },
  lng: initialLocale(),
  fallbackLng: 'en',
  interpolation: { escapeValue: false },
  returnNull: false,
});

export const setLocale = async (locale: Locale): Promise<void> => {
  globalThis.localStorage?.setItem(localeStorageKey, locale);
  await i18n.changeLanguage(locale);
};

export { i18n };
