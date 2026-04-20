# Development Guide

This document covers how to set up the development environment and test any2bibtex locally.

## Prerequisites

- **Node.js** 20+ ([download](https://nodejs.org/))
- **Python** 3.10+ ([download](https://www.python.org/) or use [Conda](https://docs.conda.io/))

## Quick Start

### 1. Install Dependencies

```bash
# Frontend
npm install

# Backend (using Conda)
conda create -n any2bibtex python=3.10 -y
conda activate any2bibtex
pip install -r backend/requirements.txt
```

Optional for more stable title search:

```bash
export SEMANTIC_SCHOLAR_API_KEY=your_key_here
```

Apply here: <https://www.semanticscholar.org/product/api#api-key-form>

Without an API key, Semantic Scholar title search still works on shared
unauthenticated limits, but it may return `429 Too Many Requests` during busy
periods.

### 2. Run Development Mode

**Terminal 1 - Backend:**

```bash
conda activate any2bibtex
cd backend && uvicorn main:app --port 8765 --reload
```

**Terminal 2 - Frontend + Electron shell:**

```bash
npm run dev
```

Note:

- In development, Electron expects the backend to already be running on `127.0.0.1:8765`.
- Saving a Semantic Scholar API key in the packaged app restarts the bundled backend automatically.
- In development mode, if you want the backend to use an API key, you still need to export `SEMANTIC_SCHOLAR_API_KEY` in the backend terminal before starting `uvicorn`.

## Building for Production

### Step 1: Build Python Backend

```bash
cd backend
python build.py
```

This creates `resources/backend/any2bibtex-backend`.

### Step 2: Build Electron App

```bash
# From project root
npm run build

# Build for your platform
npx electron-builder --mac     # macOS
npx electron-builder --win     # Windows
npx electron-builder --linux   # Linux
```

Output: `dist-electron/any2bibtex-*.dmg` (or `.exe`, `.AppImage`)

### Step 3: Test the App

```bash
# macOS
open dist-electron/any2bibtex-*.dmg

# Windows
./dist-electron/any2bibtex-Setup-*.exe

# Linux
chmod +x dist-electron/any2bibtex-*.AppImage
./dist-electron/any2bibtex-*.AppImage
```

## Project Structure

```
any2bibtex/
├── backend/           # Python FastAPI backend
│   ├── main.py        # API entry point
│   ├── resolvers.py   # DOI/arXiv/title resolution
│   ├── semantic_scholar.py
│   ├── build.py       # PyInstaller script
│   └── requirements.txt
├── electron/          # Electron main + preload bridge
│   ├── main.js
│   └── preload.js
├── src/               # Vue 3 renderer
│   ├── App.vue
│   ├── main.ts
│   └── style.css
├── build/             # Icons and build resources
├── resources/         # Demo gif and built backend output
└── dist-electron/     # Built app (gitignored)
```

## Shortcuts

| Shortcut                    | Action        |
| --------------------------- | ------------- |
| `Option+Space` (macOS)      | Toggle window |
| `Alt+Space` (Windows/Linux) | Toggle window |
| `Enter`                     | Search        |
| `Escape`                    | Hide window   |

## Manual Checks

Recommended smoke tests before packaging:

```bash
curl 'http://127.0.0.1:8765/resolve?q=10.1038/nphys1170'
curl 'http://127.0.0.1:8765/resolve?q=2205.15019'
curl 'http://127.0.0.1:8765/resolve?q=Deep%20Residual%20Learning%20for%20Image%20Recognition'
curl 'http://127.0.0.1:8765/resolve?q=Attention%20Is%20All%20You%20Need'
```
