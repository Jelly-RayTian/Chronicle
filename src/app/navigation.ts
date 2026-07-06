export type PageId =
  | 'timeline'
  | 'indexed-folders'
  | 'versions'
  | 'projects'
  | 'sessions'
  | 'search'
  | 'settings';

export const pageTranslationKey: Record<PageId, string> = {
  timeline: 'navigation.timeline',
  'indexed-folders': 'navigation.indexedFolders',
  versions: 'navigation.versions',
  projects: 'navigation.projects',
  sessions: 'navigation.sessions',
  search: 'navigation.search',
  settings: 'navigation.settings',
};
