const { app, BrowserWindow, globalShortcut, ipcMain, clipboard, Tray, Menu, nativeImage, dialog } = require('electron');
const path = require('path');
const fs = require('fs');
const { spawn } = require('child_process');
const http = require('http');

let mainWindow;
let pythonProcess;
let tray;

// Development mode flag - only use app.isPackaged
const isDev = !app.isPackaged;
const BACKEND_PORT = 8765;
const BACKEND_URL = `http://127.0.0.1:${BACKEND_PORT}`;

/**
 * Get the path to the Python backend executable
 */
function getBackendPath() {
  if (isDev) {
    return null; // In dev mode, backend is run manually
  }
  
  const platform = process.platform;
  let binaryName = 'any2bibtex-backend';
  
  if (platform === 'win32') {
    binaryName += '.exe';
  }
  
  // In production, the backend is in resources/backend
  const backendPath = path.join(process.resourcesPath, 'backend', binaryName);
  
  console.log('[DEBUG] resourcesPath:', process.resourcesPath);
  console.log('[DEBUG] backendPath:', backendPath);
  console.log('[DEBUG] exists:', fs.existsSync(backendPath));
  
  return backendPath;
}

/**
 * Check if the backend is already running
 */
function checkBackendHealth() {
  return new Promise((resolve) => {
    const req = http.get(BACKEND_URL, (res) => {
      resolve(res.statusCode === 200);
    });
    req.on('error', () => resolve(false));
    req.setTimeout(1000, () => {
      req.destroy();
      resolve(false);
    });
  });
}

/**
 * Wait for backend to be ready
 */
async function waitForBackend(maxAttempts = 30) {
  for (let i = 0; i < maxAttempts; i++) {
    const isReady = await checkBackendHealth();
    if (isReady) {
      console.log('Backend is ready!');
      return true;
    }
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  console.error('Backend failed to start after', maxAttempts * 0.5, 'seconds');
  return false;
}

/**
 * Start the Python backend process
 */
async function startPythonBackend() {
  console.log('[DEBUG] isDev:', isDev);
  console.log('[DEBUG] app.isPackaged:', app.isPackaged);
  
  if (isDev) {
    console.log('Development mode: expecting backend to be running at', BACKEND_URL);
    const isRunning = await checkBackendHealth();
    if (!isRunning) {
      console.warn('Backend not running! Please start it with: npm run start:backend');
    }
    return;
  }
  
  const backendPath = getBackendPath();
  if (!backendPath) {
    console.error('Could not determine backend path');
    return;
  }
  
  // Check if backend exists
  if (!fs.existsSync(backendPath)) {
    console.error('Backend executable not found at:', backendPath);
    dialog.showErrorBox('Backend Error', `Backend not found at: ${backendPath}`);
    return;
  }
  
  console.log('Starting backend from:', backendPath);
  
  pythonProcess = spawn(backendPath, [], {
    stdio: ['pipe', 'pipe', 'pipe'],
    env: {
      ...process.env,
      PORT: BACKEND_PORT.toString()
    }
  });
  
  pythonProcess.stdout.on('data', (data) => {
    console.log(`[Backend] ${data.toString().trim()}`);
  });

  pythonProcess.stderr.on('data', (data) => {
    console.error(`[Backend Error] ${data.toString().trim()}`);
  });

  pythonProcess.on('error', (err) => {
    console.error('Failed to start backend:', err);
    dialog.showErrorBox('Backend Error', `Failed to start backend: ${err.message}`);
  });

  pythonProcess.on('exit', (code) => {
    console.log('Backend exited with code:', code);
    pythonProcess = null;
  });
  
  // Wait for backend to be ready
  const ready = await waitForBackend();
  if (!ready) {
    console.error('Backend did not become ready in time');
  }
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 700,
    height: 500,
    frame: false,
    transparent: true,
    resizable: false,
    alwaysOnTop: true,
    skipTaskbar: false, // Show in taskbar/dock
    show: false,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js')
    }
  });

  if (isDev) {
    mainWindow.loadURL('http://localhost:5173');
    mainWindow.webContents.openDevTools({ mode: 'detach' });
  } else {
    // Load from the dist folder (asar packaged)
    const htmlPath = path.join(__dirname, '../dist/index.html');
    console.log('[DEBUG] Loading HTML from:', htmlPath);
    mainWindow.loadFile(htmlPath);
  }

  mainWindow.on('blur', () => {
    // Auto-hide when clicking outside (production only)
    if (!isDev) {
      mainWindow.hide();
    }
  });

  mainWindow.once('ready-to-show', () => {
    mainWindow.show();
  });
  
  // Show window when clicked on dock icon (macOS)
  mainWindow.on('show', () => {
    mainWindow.focus();
  });
}

function createTray() {
  // Create a simple tray icon (16x16 or 22x22 for macOS)
  const iconPath = isDev 
    ? path.join(__dirname, '../build/tray-icon.png')
    : path.join(process.resourcesPath, 'tray-icon.png');
  
  // Use a template image for macOS to respect dark/light mode
  let icon;
  if (fs.existsSync(iconPath)) {
    icon = nativeImage.createFromPath(iconPath);
    if (process.platform === 'darwin') {
      icon.setTemplateImage(true);
    }
  } else {
    // Create a simple colored icon if no custom icon exists
    icon = nativeImage.createEmpty();
  }
  
  tray = new Tray(icon);
  tray.setToolTip('any2bibtex');
  
  const contextMenu = Menu.buildFromTemplate([
    { 
      label: 'Show any2bibtex', 
      click: () => {
        if (mainWindow) {
          mainWindow.show();
          mainWindow.focus();
        }
      }
    },
    { type: 'separator' },
    { 
      label: 'Quit', 
      click: () => {
        app.quit();
      }
    }
  ]);
  
  tray.setContextMenu(contextMenu);
  
  // Click on tray icon to show window
  tray.on('click', () => {
    if (mainWindow) {
      if (mainWindow.isVisible()) {
        mainWindow.hide();
      } else {
        mainWindow.show();
        mainWindow.focus();
      }
    }
  });
}

function registerGlobalShortcut() {
  const shortcut = process.platform === 'darwin' ? 'Option+Space' : 'Alt+Space';
  
  globalShortcut.register(shortcut, () => {
    if (mainWindow.isVisible()) {
      mainWindow.hide();
    } else {
      mainWindow.show();
      mainWindow.focus();
    }
  });
}

// IPC handlers
ipcMain.handle('copy-to-clipboard', (event, text) => {
  clipboard.writeText(text);
  return true;
});

ipcMain.handle('hide-window', () => {
  mainWindow.hide();
  return true;
});

// App lifecycle
app.whenReady().then(async () => {
  await startPythonBackend();
  createWindow();
  // createTray(); // Disabled per user request
  registerGlobalShortcut();
});

app.on('will-quit', () => {
  globalShortcut.unregisterAll();
  if (pythonProcess) {
    console.log('Stopping backend process...');
    pythonProcess.kill();
  }
});

app.on('window-all-closed', () => {
  // Don't quit on macOS when window is closed
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  // Re-show window when dock icon is clicked (macOS)
  if (mainWindow) {
    mainWindow.show();
    mainWindow.focus();
  } else if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});
