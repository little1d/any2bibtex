# Development Guide

This document covers how to set up, test, package, and release any2bibtex.

## Prerequisites

- **Node.js** 20+ ([download](https://nodejs.org/))
- **Rust** stable ([rustup](https://rustup.rs/))
- **Platform build dependencies** for Tauri 2

On macOS:

```bash
xcode-select --install
```

On Linux, install the Tauri system dependencies for your distribution. For Ubuntu:

```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf
```

## Quick Start

Install JavaScript dependencies:

```bash
npm install
```

Ensure Rust is available in the current shell:

```bash
source "$HOME/.cargo/env"
rustc --version
cargo --version
```

Run the app in development mode:

```bash
npm run dev
```

## Architecture

any2bibtex now uses a Tauri shell with a Rust resolver backend.

```
any2bibtex/
├── src/               # Vue 3 renderer
│   ├── App.vue
│   ├── components/
│   ├── services/      # Tauri command bridge
│   └── utils/
├── src-tauri/         # Tauri 2 + Rust backend
│   ├── src/lib.rs     # Tauri commands, tray, shortcuts
│   ├── src/resolver.rs
│   ├── src/settings.rs
│   ├── icons/
│   └── tauri.conf.json
├── assets/            # README assets such as logo and demo gif
├── build/             # App icon sources
└── dist/              # Built frontend output (gitignored)
```

There is no Python/FastAPI service and no local HTTP backend. The renderer calls Rust commands through Tauri IPC.

## Semantic Scholar API Key

Title search uses Semantic Scholar. Without an API key, title search still works with shared unauthenticated rate limits, but it may return `429 Too Many Requests` during busy periods.

Users can configure a Semantic Scholar API key inside the app. The key is stored locally by the Rust settings layer and used directly by the resolver.

Apply here: <https://www.semanticscholar.org/product/api#api-key-form>

Semantic Scholar API keys currently have a limit of `1 request/second`, cumulative across all endpoints.

## Useful Commands

```bash
npm run build      # Build the Vue frontend
cargo check        # Check the Rust backend from src-tauri/
npm run build:app  # Build the Tauri desktop app
npm run build:release  # Build installers plus signed updater artifacts
```

If `npm run build:app` fails because `cargo` is not on `PATH`, run:

```bash
source "$HOME/.cargo/env"
```

## Packaging

From the project root:

```bash
npm run build:app
```

Typical outputs:

- macOS: `src-tauri/target/release/bundle/dmg/*.dmg`
- Windows: `src-tauri/target/release/bundle/nsis/*.exe`
- Linux: `src-tauri/target/release/bundle/appimage/*.AppImage` and `src-tauri/target/release/bundle/deb/*.deb`

macOS release builds use ad-hoc signing via `bundle.macOS.signingIdentity = "-"`. This keeps downloaded Apple Silicon builds code-signed, but it is not a substitute for Developer ID signing and notarization.

## Automatic Updates

any2bibtex uses the Tauri updater plugin. Release builds publish `latest.json` plus signed updater artifacts to GitHub Releases.

Updater signing uses a long-lived key pair:

- Public key: committed in `src-tauri/tauri.conf.json`.
- Private key: keep secret and store in GitHub Actions as `TAURI_SIGNING_PRIVATE_KEY`.
- Password: optional; this project currently uses an empty password, so `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` can be empty.

The local private key generated for this project is stored outside the repository at:

```bash
~/.tauri/any2bibtex.key
```

To configure GitHub Actions, add the private key content as a repository secret:

```bash
cat ~/.tauri/any2bibtex.key
```

The normal local package command remains `npm run build:app`. Use `npm run build:release` only when `TAURI_SIGNING_PRIVATE_KEY` is configured, because updater artifacts cannot be generated without the private signing key.

On macOS, if the final DMG script fails in a sandboxed terminal but `.app` is generated, rerun the generated script locally:

```bash
cd src-tauri/target/release/bundle/dmg
./bundle_dmg.sh any2bibtex_0.0.6_aarch64.dmg ../macos/any2bibtex.app
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

- Launch with `npm run dev`.
- Search DOI: `10.1038/nphys1170`.
- Search arXiv ID: `2205.15019`.
- Search title: `Attention Is All You Need`.
- Copy BibTeX.
- Configure and remove a Semantic Scholar API key.
- Toggle dark/light mode from both the in-app button and tray menu.
- Verify tray actions: `Show`, `Hide`, `Dark Mode`, `Light Mode`, `Quit`.
- Verify the macOS transparent window has no white background outside the rounded app container.

## Release Process

This repository publishes GitHub Releases from version tags that match `v*`.

Before creating a release:

1. Ensure `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and `CHANGELOG.md` are up to date.
2. Run `npm run build`.
3. Run `cd src-tauri && cargo check`.
4. Ensure the GitHub repository secret `TAURI_SIGNING_PRIVATE_KEY` is configured.
5. Run `npm run build:app` on at least one local platform.

Create and push a release tag:

```bash
git pull origin main
git tag v0.0.6
git push origin main
git push origin v0.0.6
```

After pushing the tag:

1. Open the GitHub `Actions` page.
2. Wait for `Build any2bibtex` to finish on Windows, macOS, and Linux.
3. Open the GitHub `Releases` page and verify the new release artifacts were attached successfully.

If the tag already exists and you intentionally want to retarget it:

```bash
git tag -d v0.0.6
git push origin :refs/tags/v0.0.6
git tag v0.0.6
git push origin v0.0.6
```
