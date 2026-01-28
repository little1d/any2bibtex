# Changelog

All notable changes to this project will be documented in this file.

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
