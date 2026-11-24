# ENKI v0.1.0 — Milestone 1: Desktop Foundation

First tagged build of ENKI, a private, local-first AI development platform
(Tauri 2 + Rust + React + SQLite).

## Included

- **Desktop foundation**: Tauri 2 shell, dark-first React/TypeScript UI
  (charcoal surfaces, electric blue/violet accents, thin borders), compact
  keyboard-navigable navigation (Alt+1–4).
- **Persistence**: SQLite (WAL) with ordered migrations — settings, projects,
  sessions, providers, models, audit log. Verified to survive restarts.
- **Provider manager**: 8 built-in provider definitions (documented public
  endpoints only), custom OpenAI-compatible providers with protocol/URL/header
  configuration, per-provider models with capability flags, default model
  selection.
- **Credentials**: API keys stored exclusively in the OS credential store
  (Windows Credential Manager); only a key-presence boolean reaches the UI.
- **Connection testing**: real HTTP probes of each provider's model-listing
  endpoint, with honest success/failure reporting.
- **Agent runtime**: engine-neutral adapter interface; OpenCode discovery via
  real `opencode --version` probe; unsupported capabilities rendered as
  unsupported — nothing simulated.
- **Security**: project-scoped filesystem boundary (traversal-safe), outside
  writes always rejected, outside reads opt-in and audited, security events
  written to the audit log.

## Verification

- 24 Rust tests (`cargo test`), clippy clean
- 9 frontend tests (vitest, incl. bridge-contract and shell smoke tests)
- `tsc -b && vite build` production build
- CI (this tag) re-runs all tests on Windows and Ubuntu before bundling

## Artifacts

- Windows: `ENKI_0.1.0_x64_en-US.msi`, `ENKI_0.1.0_x64-setup.exe` (NSIS)
- Linux: `.deb`, `.rpm`, `.AppImage`

## Known limitations

- Installers are unsigned — Windows SmartScreen / macOS Gatekeeper will warn.
- The coding workspace (conversation view, Monaco editor, terminal, diff
  approval) is Milestone 2; agent session streaming is not in this release —
  the Agents page reports discovery only.
- No macOS bundles yet.
