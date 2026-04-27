# Changelog

All notable changes to this project will be documented in this file.

## [0.0.5] - 2026-04-27

### Added

- **Tauri Runtime** - Migrated the desktop shell from Electron to Tauri 2.
- **Rust Resolver** - Reimplemented DOI, arXiv, and Semantic Scholar title resolution in Rust Tauri commands.
- **Rust Shell Integrations** - Moved tray menu, global shortcut, settings, clipboard, and external-link handling into the Tauri/Rust layer.
- **In-App Updates** - Added a Tauri updater flow with update detection, download progress, restart prompt, and post-restart success confirmation.

### Changed

- **Package Size** - Removed the bundled Python backend and Electron runtime path, reducing the packaged app from 120+ MB to under 10 MB, roughly a 92%+ size reduction.
- **Development Workflow** - Replaced the separate FastAPI backend workflow with a single `npm run dev` Tauri workflow.
- **Window Transparency** - Enabled macOS transparent-window support through Tauri `macOSPrivateApi`.

### Removed

- **Electron Main Process** - Removed Electron main/preload code from the runtime path.
- **Python FastAPI Backend** - Removed the Python backend source and PyInstaller build flow.

### Fixed

- **Windows Packaging** - Added the required Windows `.ico` app icon so Tauri can generate the Windows resource file during release builds.
- **macOS Packaging** - Added ad-hoc code signing for macOS release builds to avoid invalid-signature app damage warnings on downloaded DMG artifacts.

## [0.0.4] - 2026-04-27

### Added

- **Menu Bar Tray** - Restored the macOS tray entry with Show, Hide, Quit, and theme selection actions.
- **Theme Switching** - Added persistent dark/light theme support from both the tray menu and in-app icon button.
- **API Key Management** - Added a standalone Semantic Scholar API key panel with configured-state display, removal confirmation, and direct API key application link.

### Changed

- **Tray Icon** - Replaced the menu bar icon with a cleaner template image optimized for macOS tinting.
- **Window Styling** - Simplified the main app container, removed the old glass frame, and aligned component colors with theme variables.
- **API Key UX** - Updated the configured-key state so users can manage or remove the stored key instead of seeing the initial setup flow again.

### Fixed

- **Tray Interaction** - Left-clicking the tray icon now only shows and focuses the app instead of opening the context menu or toggling visibility.
- **Development Focus Behavior** - The tray and global shortcut paths now share the same show/hide logic, reducing inconsistent window state updates.

## [0.0.3] - 2026-04-20

### Added

- **Paper Title Search** - Added Semantic Scholar-based title lookup alongside DOI and arXiv resolution.
- **API Key Support** - Added local Semantic Scholar API key storage and backend injection for more reliable title search.
- **In-App API Key Setup** - Added an in-app configuration panel with a direct link to the Semantic Scholar API key application form.
- **Demo GIF** - Replaced the old embedded demo video with a repository-friendly GIF preview.

### Changed

- **Input Detection** - Updated the app and documentation to present DOI, arXiv ID, and paper title as first-class input types.
- **Loading and Error Copy** - Improved title-search loading states and rate-limit guidance, especially when no API key is configured.
- **Window and Layout Polish** - Refined the spotlight window sizing, result empty-state layout, and API key setup presentation.
- **Development Docs** - Updated local development steps, smoke tests, and packaging notes for the current Electron + backend workflow.

### Fixed

- **Title Match Ranking** - Tightened title-match selection to avoid weak Semantic Scholar matches.
- **API Key UX Regression** - Reworked the API key flow from an overlay modal into an inline settings panel to avoid clipped content in the fixed-size window.
- **Icon Packaging** - Restored packaged app icon handling for the Electron build output.

## [0.0.2] - 2026-01-29

### Fixed

- 🐞 **Tray Icon Removed** - Disabled menu bar tray icon that caused black boxes on macOS.
- 🏗 **CI/CD Improvements** - Removed custom icon configurations that were breaking Windows and Linux builds.

## [0.0.1] - 2026-01-29

### Added

- 🚀 **Initial Release MVP**
- 🎨 **Spotlight UI** - A clean, dark-themed interface inspired by Raycast/Spotlight.
- ⌨️ **Global Shortcut** - Toggle window with `Option+Space` (macOS) or `Alt+Space` (Windows/Linux).
- 🔩 **BibTeX Resolvers** - Supports DOI, arXiv ID, and Paper Title (via Semantic Scholar).
- 📦 **Standalone Packaging** - Bundled Python backend with Electron for a seamless desktop experience.
- 🎥 **Demo Video** - Added demonstration video to README.
- 🛠 **Dev Tools** - Added `npm run dev:all` for full stack hot-reload development.

### Fixed

- 🐞 **arXiv Title Parsing** - Correctly extracts paper titles from entry nodes instead of feed metadata.
- 🖱 **Window Dragging** - Dedicated drag handle at the top of the window for better reliability.
- ⌨️ **Interaction Fixes** - ESC key and clicking outside the window now correctly hide the application.
- 📜 **BibTeX Formatting** - Unified formatting utility ensures consistent indentation for display and clipboard.

### Refactored

- 🏗 **Componentization** - Refactored `App.vue` into modular components (`SearchBar`, `ResultCard`).
- 🧹 **Style Cleanup** - Removed problematic blur effects for a more robust and clean UI.
