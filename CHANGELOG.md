# Changelog

All notable changes to this project will be documented in this file.

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
