export type PageId = 'timeline' | 'indexed-folders' | 'versions' | 'settings';

export const pageTranslationKey: Record<PageId, string> = {
  timeline: 'navigation.timeline',
  'indexed-folders': 'navigation.indexedFolders',
  versions: 'navigation.versions',
  settings: 'navigation.settings',
};
