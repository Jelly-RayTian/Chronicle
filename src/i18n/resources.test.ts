import { describe, expect, it } from 'vitest';

import { detectLocale } from './index';
import { en } from './en';
import { zhCN } from './zh-CN';

const leafKeys = (value: Record<string, unknown>, prefix = ''): string[] =>
  Object.entries(value).flatMap(([key, child]) => {
    const path = prefix ? `${prefix}.${key}` : key;
    return typeof child === 'object' && child !== null
      ? leafKeys(child as Record<string, unknown>, path)
      : [path];
  });

describe('localization resources', () => {
  it('keeps Chinese and English resource keys in sync', () => {
    expect(leafKeys(zhCN).sort()).toEqual(leafKeys(en).sort());
  });

  it('uses Simplified Chinese for Chinese system locales', () => {
    expect(detectLocale('zh-CN')).toBe('zh-CN');
    expect(detectLocale('zh-Hans')).toBe('zh-CN');
  });

  it('falls back to English for other system locales', () => {
    expect(detectLocale('en-US')).toBe('en');
    expect(detectLocale('fr-FR')).toBe('en');
  });
});
