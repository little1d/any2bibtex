# any2bibtex

## ✨ Features

- 🔍 **Smart Input** — Automatically detects DOI, arXiv ID, or paper title
- ⚡ **Instant Results** — Get BibTeX in seconds
- 📋 **One-Click Copy** — Copy to clipboard instantly
- 🎨 **Spotlight UI** — Minimal, keyboard-friendly interface
- ⌨️ **Global Shortcut** — `Option+Space` (macOS) / `Alt+Space` (Windows/Linux)

![any2bibtex demo](demo.gif)

## 📥 Installation

| Platform       | Download                                                                       | Install                        |
| -------------- | ------------------------------------------------------------------------------ | ------------------------------ |
| 🍎 **macOS**   | [any2bibtex.dmg](https://github.com/little1d/any2bibtex/releases/latest)       | Open DMG, drag to Applications |
| 🪟 **Windows** | [any2bibtex-Setup.exe](https://github.com/little1d/any2bibtex/releases/latest) | Run installer                  |
| 🐧 **Linux**   | [any2bibtex.AppImage](https://github.com/little1d/any2bibtex/releases/latest)  | `chmod +x` and run             |

## 🚀 Usage

1. Press `Option+Space` (macOS) or `Alt+Space` (Windows) to open
2. Enter a DOI (`10.1038/nphys1170`), arXiv ID (`2205.15019`), or paper title (`Attention Is All You Need`)
3. Press **Enter**
4. Click **Copy BibTeX**

### Title Search Notes

- Title search uses Semantic Scholar.
- Without an API key, title search still works, but it uses shared rate limits and may occasionally return rate-limit errors during busy periods.
- You can configure a Semantic Scholar API key inside the app for more reliable title search.
- Apply for a Semantic Scholar API key: <https://www.semanticscholar.org/product/api#api-key-form>

## 🛠 Tech Stack

| Layer       | Technology                                                                                                              |
| ----------- | ----------------------------------------------------------------------------------------------------------------------- |
| 🖥 Frontend | [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) + [Tailwind CSS](https://tailwindcss.com/)  |
| 💻 Desktop  | [Electron](https://www.electronjs.org/)                                                                                 |
| ⚙️ Backend  | [Python](https://www.python.org/) + [FastAPI](https://fastapi.tiangolo.com/)                                            |
| 📦 Build    | [Vite](https://vitejs.dev/) + [electron-builder](https://www.electron.build/) + [PyInstaller](https://pyinstaller.org/) |

## 🗺 Roadmap

- [-] Improve title search matching quality and fallback behavior
- [ ] Similarity-based reference search (input abstract → find related papers)
- [ ] Evaluate a Tauri 2 migration to reduce package size and startup overhead

## ⭐ Star History

[![Star History Chart](https://api.star-history.com/svg?repos=little1d/any2bibtex&type=Date)](https://star-history.com/#little1d/any2bibtex&Date)
