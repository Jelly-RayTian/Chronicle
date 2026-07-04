export type PageId =
  | 'timeline'
  | 'indexed-folders'
  | 'versions'
  | 'projects'
  | 'sessions'
  | 'settings';

export const pageTranslationKey: Record<PageId, string> = {
  timeline: 'navigation.timeline',
  'indexed-folders': 'navigation.indexedFolders',
  versions: 'navigation.versions',
  projects: 'navigation.projects',
  sessions: 'navigation.sessions',
  settings: 'navigation.settings',
};
