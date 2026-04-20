const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronAPI', {
    copyToClipboard: (text) => ipcRenderer.invoke('copy-to-clipboard', text),
    hideWindow: () => ipcRenderer.invoke('hide-window'),
    getSemanticScholarConfig: () => ipcRenderer.invoke('get-semantic-scholar-config'),
    saveSemanticScholarConfig: (apiKey) => ipcRenderer.invoke('save-semantic-scholar-config', apiKey),
    openExternalUrl: (url) => ipcRenderer.invoke('open-external-url', url)
});
