# ENKI

Private, local-first AI development platform for Windows (Tauri 2 + Rust + React).

## Status: Milestone 1 — desktop foundation ✅

- [x] Tauri 2 shell with Rust backend (commands, state, SQLite, migrations)
- [x] Provider/model manager backend (custom OpenAI-compatible providers, OS keyring credentials)
- [x] Agent runtime adapter interface + OpenCode discovery (real `opencode --version` probe)
- [x] Project-scoped filesystem security boundary + audit log
- [x] React UI (dark-first design system, navigation, Home/Providers/Agents/Settings)
- [x] CI release pipeline (Windows MSI/NSIS + Linux deb/AppImage via GitHub Actions)

## Structure

```
src-tauri/         Rust backend (Tauri 2)
  src/
    core/          errors, app paths, path boundary checks
    storage/       SQLite open + migrations
    settings/      typed app settings with validation
    workspace/     projects + sessions + security gates
    ai_gateway/    providers, models, credentials, catalog, connection tests
    agent_runtime/ engine adapter trait, registry, OpenCode adapter
    audit/         audit log
    commands.rs    Tauri command layer
ui/                React + TypeScript + Tailwind CSS 4 frontend
```

## Development

Prerequisites: Rust 1.82+ (MSVC toolchain on Windows), Node 20+.

```bash
# one-time: install dependencies (root = Tauri CLI, ui = frontend)
npm install && npm --prefix ui install

# run the desktop app in dev mode
npm run tauri dev

# production build (MSI/NSIS on Windows, deb/AppImage on Linux)
npm run tauri build

# tests
npm test                        # frontend (vitest)
cd src-tauri && cargo test      # backend (Rust)
```

## Releases

Tags `v*` trigger [.github/workflows/release.yml](.github/workflows/release.yml):
tests run on Windows and Ubuntu, installers are built, and a GitHub Release is
published with the artifacts.

See ARCHITECTURE.md, DECISIONS.md, ROADMAP.md, CONTRIBUTING.md.

