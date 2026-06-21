export type PageId = 'timeline' | 'indexed-folders' | 'settings';

export const pageTranslationKey: Record<PageId, string> = {
  timeline: 'navigation.timeline',
  'indexed-folders': 'navigation.indexedFolders',
  settings: 'navigation.settings',
};
