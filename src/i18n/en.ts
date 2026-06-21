export const en = {
  app: {
    name: 'Chronicle',
    tagline: 'Your local file activity, remembered in context.',
  },
  navigation: {
    timeline: 'Timeline',
    indexedFolders: 'Indexed folders',
    settings: 'Settings',
  },
  status: {
    system: 'System status',
    native: 'Native connection',
    database: 'Local database',
    ready: 'Ready',
    unavailable: 'Unavailable',
    schemaVersion: 'Schema version {{version}}',
  },
  timeline: {
    title: 'Timeline',
    description: 'Return to files through the time and context around your work.',
    emptyTitle: 'Your timeline is ready',
    emptyBody:
      'There is no indexed file activity yet. Folder selection and metadata scanning arrive in Milestone 1.',
  },
  folders: {
    title: 'Indexed folders',
    description: 'Only folders you explicitly choose can be indexed by Chronicle.',
    emptyTitle: 'No indexed folders',
    emptyBody:
      'Chronicle has not been granted access to any folders. Clearing Chronicle data will never delete your original files.',
    count: '{{count}} folders',
  },
  settings: {
    title: 'Settings',
    description: 'Control the local Chronicle experience.',
    language: 'Language',
    languageDescription: 'Choose the language used throughout Chronicle.',
    chinese: '简体中文',
    english: 'English',
    privacyTitle: 'Privacy baseline',
    localOnly: 'Local only',
    localOnlyBody: 'Chronicle does not upload file paths or metadata.',
    contentsUnread: 'File contents are not read',
    contentsUnreadBody: 'Milestone 0 creates the foundation without opening your files.',
    noAnalytics: 'No analytics',
    noAnalyticsBody: 'Chronicle does not include behavioral tracking or telemetry.',
  },
  state: {
    loading: 'Loading local data',
    loadingBody: 'Chronicle is connecting to its native services.',
    retry: 'Try again',
  },
  errors: {
    unexpected: 'Chronicle could not complete this local request.',
    databaseUnavailable:
      'Chronicle could not open its local database. Your original files were not changed.',
    nativeUnavailable:
      'Chronicle could not connect to its native service. Your original files were not changed.',
  },
} as const;
