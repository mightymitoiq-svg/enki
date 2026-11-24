# ENKI Architecture

ENKI is a private, local-first AI development platform built with Tauri 2
(Rust backend, React/TypeScript frontend). This document describes the module
boundaries as implemented today and the direction for the parts not yet built.

## Layer map

```
┌─────────────────────────────────────────────────────────┐
│ Desktop layer (Tauri shell)                             │
│   window management, app lifecycle, native integration  │
├─────────────────────────────────────────────────────────┤
│ Core layer            src-tauri/src/core                │
│   AppError, AppPaths, path boundary checks              │
├─────────────────────────────────────────────────────────┤
│ Storage layer         src-tauri/src/storage             │
│   SQLite open + ordered migrations                      │
├──────────────┬──────────────────┬───────────────────────┤
│ Workspace    │ AI gateway       │ Agent runtime         │
│ workspace/   │ ai_gateway/      │ agent_runtime/        │
│ projects,    │ providers,       │ adapter trait,        │
│ sessions,    │ models, creds,   │ registry, engine      │
│ fs security  │ catalog, tests   │ discovery             │
├──────────────┴──────────────────┴───────────────────────┤
│ Cross-cutting: settings/, audit/, commands.rs           │
└─────────────────────────────────────────────────────────┘
```

## Module responsibilities

### core
- `AppError`: typed error enum serialized to the frontend as
  `{error: code, message}` — no panics cross the boundary.
- `AppPaths`: canonical data layout (`<data_dir>/enki/`): `enki.db`, `logs/`.
- `path_is_within`: the filesystem security primitive. Lexical normalization
  first (so `..` cannot survive), then canonicalization of the deepest
  existing ancestor. Every agent/app file access on a project must pass it.

### storage
SQLite with ordered, append-only migrations (`MIGRATIONS` array). `open()`
enables WAL and foreign keys. Tests reset via `reset_for_test`. Never edit a
shipped migration; add a new one.

### settings
`AppSettings` — typed struct persisted as one JSON row. `validate()` enforces
invariants (theme enum, font range, non-negative budget). Frontend Zod schema
mirrors the same rules (`ui/src/lib/schemas.ts`); both sides validate.

### workspace
Projects (registered folders) and sessions. `check_project_path` is the
security gate: inside-root always allowed; outside reads only when the user
enabled `allow_reads_outside_workspace`; outside writes never. Denials are
audited.

### ai_gateway
- `providers`: CRUD + validation for providers and models. `Protocol` enum
  drives request building. Model entries are user-supplied (or discovered via
  `/models` listing) — the catalog never invents model IDs.
- `credentials`: `CredentialStore` trait. Production: OS keyring
  (Windows Credential Manager). Keys are referenced by provider id; only a
  boolean `has_api_key` ever reaches the UI.
- `catalog`: built-in provider definitions (documented public endpoints only),
  seeded idempotently at startup.
- `connection_test`: real HTTP probe of the provider's model-listing endpoint
  per protocol (OpenAI `/models`, Anthropic `/models` + version header,
  Google `?key=`, Ollama `/api/tags`). Reports reachability and model count
  honestly; failures surface as errors, not fake successes.

### agent_runtime
- `AgentEngineAdapter` trait: `id`, `display_name`, `adapter_version`,
  `discover()`. Discovery is a real probe (`opencode --version` on PATH).
- `EngineCapabilities` describes what an engine supports; `EngineInfo`
  carries status + notes. The UI renders unsupported capabilities as
  unsupported (struck through) — nothing simulated.
- `OpenCodeAdapter` is the first engine. Codex / Claude Code / custom adapters
  implement the same trait. Deep session integration (run, stream, interrupt)
  is milestone 2; the adapter says so in its notes.

### audit
Append-only `audit_log` table with categories `security | agent | gateway | app`.
Security-relevant events (key stored/removed, path denials, connection tests)
are recorded.

### commands.rs
The Tauri command layer: one thin command per service function, shared
`AppState` (SQLite connection behind a `Mutex`, credential store, engine
registry). Commands validate inputs at the service layer and return typed
`AppError`s.

## Frontend structure

```
ui/src/
  App.tsx            shell: icon nav (Alt+1..4), page routing
  index.css          design tokens (Tailwind 4 @theme)
  lib/schemas.ts     Zod schemas mirroring Rust types (contract tests)
  lib/api.ts         typed invoke wrappers; every response Zod-validated
  pages/             HomePage, ProvidersPage, AgentsPage, SettingsPage
```

The frontend treats backend responses as untrusted data: every `invoke` result
is parsed with Zod before rendering.

## Data flow example: adding a model

1. User submits model form → `api.upsertModel` (typed wrapper).
2. Tauri `invoke` → `upsert_model` command → `providers::upsert_model`.
3. Validation (model_id required, context ≥ 1024) → SQLite upert.
4. Row returned → Zod parse in frontend → table re-renders.

## Security model (current state)

- Filesystem: project-scoped via `path_is_within`; traversal-safe; outside
  writes rejected; outside reads opt-in and audited.
- Credentials: OS keyring only; never in SQLite, logs, or frontend state.
- Network: outbound only for provider connection tests (user-initiated).
- Audit: security events recorded with details.
- Planned (milestone 2+): agent tool execution permissions, terminal command
  approval flow, process lifecycle sandboxing.

## What is deliberately NOT built yet

- Coding workspace (conversation, Monaco editor, terminal, diffs) — milestone 2.
- Agent session running/streaming — milestone 2 (OpenCode adapter currently
  discovery-only, and says so).
- Extension system (plugins, skills, MCP) — milestone 4.
- Local intelligence (memory, context management) — milestone 5.
- Usage tracking (budget value is stored; counters land with the gateway
  router in milestone 3).
