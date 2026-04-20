export {};

declare global {
  interface Window {
    electronAPI: {
      copyToClipboard: (text: string) => Promise<boolean>;
      hideWindow: () => Promise<boolean>;
      getSemanticScholarConfig: () => Promise<{
        hasApiKey: boolean;
      }>;
      saveSemanticScholarConfig: (apiKey: string) => Promise<{
        hasApiKey: boolean;
      }>;
      openExternalUrl: (url: string) => Promise<boolean>;
    };
  }
}
