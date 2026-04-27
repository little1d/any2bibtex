export {};

declare global {
  interface Window {
    electronAPI: {
      copyToClipboard: (text: string) => Promise<boolean>;
      hideWindow: () => Promise<boolean>;
      getAppTheme: () => Promise<"dark" | "light">;
      setAppTheme: (theme: "dark" | "light") => Promise<"dark" | "light">;
      toggleAppTheme: () => Promise<"dark" | "light">;
      onThemeChanged: (callback: (theme: "dark" | "light") => void) => () => void;
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
