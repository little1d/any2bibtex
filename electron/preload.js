const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronAPI', {
    copyToClipboard: (text) => ipcRenderer.invoke('copy-to-clipboard', text),
    hideWindow: () => ipcRenderer.invoke('hide-window'),
    getAppTheme: () => ipcRenderer.invoke('get-app-theme'),
    setAppTheme: (theme) => ipcRenderer.invoke('set-app-theme', theme),
    toggleAppTheme: () => ipcRenderer.invoke('toggle-app-theme'),
    onThemeChanged: (callback) => {
        const listener = (event, theme) => callback(theme);
        ipcRenderer.on('theme-changed', listener);
        return () => ipcRenderer.removeListener('theme-changed', listener);
    },
    getSemanticScholarConfig: () => ipcRenderer.invoke('get-semantic-scholar-config'),
    saveSemanticScholarConfig: (apiKey) => ipcRenderer.invoke('save-semantic-scholar-config', apiKey),
    openExternalUrl: (url) => ipcRenderer.invoke('open-external-url', url)
});
