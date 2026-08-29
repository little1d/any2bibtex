# Development Guide

This document covers how to set up, test, package, and release any2bibtex.

## Prerequisites

- **Node.js** 22+ ([download](https://nodejs.org/))
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
npm test               # Test release manifest generation
npm run typecheck      # Check Vue and TypeScript types
npm run build          # Build the Vue frontend
npm run release:check  # Verify package, Tauri, and Cargo versions match
cargo check            # Check the Rust backend from src-tauri/
npm run build:app      # Build the Tauri desktop app
npm run build:release  # Build installers plus signed updater artifacts
```

If `npm run build:app` fails because `cargo` is not on `PATH`, run:

```bash
source "$HOME/.cargo/env"
```

## Packaging

Local builds use the current machine architecture:

```bash
npm run build:app
```

The release workflow builds:

- macOS: Apple Silicon and Intel `.dmg` files
- Windows: `src-tauri/target/release/bundle/nsis/*.exe`
- Linux: x64 and ARM64 `.AppImage` and `.deb` files
- Updater artifacts and signatures for every supported release architecture
- `latest.json`, `release-notes.md`, and `SHA256SUMS.txt`

### Platform signing

macOS CI uses Developer ID signing when these repository secrets are set:

- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`

Notarization can use either App Store Connect secrets (`APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PRIVATE_KEY`) or Apple ID secrets (`APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`). Without a Developer ID certificate, CI falls back to ad-hoc signing and prints a warning.

Windows CI supports Azure Artifact Signing with:

- `AZURE_CLIENT_ID`
- `AZURE_CLIENT_SECRET`
- `AZURE_TENANT_ID`
- `AZURE_SIGNING_ENDPOINT`
- `AZURE_SIGNING_ACCOUNT`
- `AZURE_SIGNING_PROFILE`

All six Azure secrets must be set together. Without them, CI produces an unsigned Windows installer and prints a warning. Linux updater artifacts are signed by the Tauri updater key and all release downloads are covered by `SHA256SUMS.txt`.

## Automatic Updates

any2bibtex checks for updates shortly after launch. Available releases open the in-app update view, which reports byte and percentage progress, prompts for restart, and confirms the installed version after relaunch.

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

## Shortcuts

| Shortcut                    | Action        |
| --------------------------- | ------------- |
| `Option+Space` (macOS)      | Toggle window |
| `Alt+Space` (Windows/Linux) | Toggle window |
| `Command+,` (macOS)         | Open settings |
| `Enter`                     | Search        |
| `Escape`                    | Hide window   |

The tray menu provides `Open any2bibtex`, `Hide Window`, `Settings`, `Check for Updates`, `Launch at Login`, appearance controls, repository and About links, and `Quit`.

## Manual Checks

Recommended smoke tests before packaging:

- Launch with `npm run dev`.
- Search DOI: `10.1038/nphys1170`.
- Search arXiv ID: `2205.15019`.
- Search title: `Attention Is All You Need`.
- Copy BibTeX.
- Configure and remove a Semantic Scholar API key.
- Toggle dark/light mode from both the in-app button and tray menu.
- Open the update view and verify the latest-version state.
- Test an older signed build against a newer draft release before publishing.
- Verify the app menu and tray actions, including updates and launch at login.
- Verify the macOS transparent window has no white background outside the rounded app container.

## Release Process

This repository publishes GitHub Releases from version tags that match `v*`.

Before creating a release:

1. Ensure `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and `CHANGELOG.md` are up to date.
2. Run `npm test && npm run build && npm run release:check`.
3. Run `cd src-tauri && cargo check --locked`.
4. Ensure the GitHub repository secret `TAURI_SIGNING_PRIVATE_KEY` is configured.
5. Configure the Apple and Azure signing secrets when signed public installers are required.
6. Run `npm run build:app` on at least one local platform.

Create and push a release tag:

```bash
git pull origin main
git tag v0.0.7
git push origin main
git push origin v0.0.7
```

After pushing the tag:

1. Open the GitHub `Actions` page.
2. Wait for `Release any2bibtex` to finish on Windows, macOS, and Linux.
3. Verify both macOS architectures, both Linux architectures, the Windows installer, `latest.json`, and `SHA256SUMS.txt`.
4. Verify the installed app reports the new version after an in-app update and restart.

If the tag already exists and you intentionally want to retarget it:

```bash
git tag -d v0.0.7
git push origin :refs/tags/v0.0.7
git tag v0.0.7
git push origin v0.0.7
```
