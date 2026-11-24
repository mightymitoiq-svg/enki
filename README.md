# ENKI

Private, local-first AI development platform for Windows (Tauri 2 + Rust + React).

## Status: Milestone 1 — desktop foundation (in progress)

- [x] Tauri 2 shell with Rust backend (commands, state, SQLite, migrations)
- [x] Provider/model manager backend (custom OpenAI-compatible providers, OS keyring credentials)
- [x] Agent runtime adapter interface + OpenCode discovery (real `opencode --version` probe)
- [x] Project-scoped filesystem security boundary + audit log
- [ ] React UI (dark-first design system, navigation, Home/Settings/Providers/Agents)
- [ ] Windows installer bundle

## Structure

```
enki/
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

Prerequisites: Rust 1.82+, Node 20+.

```bash
# frontend
cd ui && npm install

# run the desktop app in dev mode
cd src-tauri && cargo tauri dev   # or: npx tauri dev from ui/

# tests
cd src-tauri && cargo test
cd ui && npm test
```

See ARCHITECTURE.md, DECISIONS.md, ROADMAP.md.
