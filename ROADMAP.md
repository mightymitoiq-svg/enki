# ENKI Roadmap

Milestones are vertical: each one ends with a runnable app and verified
behavior. "Done" means built, tested, and honestly reported.

## Milestone 1 — Desktop foundation ✅ (this build)

- Tauri 2 shell, React + TS + Tailwind 4 UI, dark-first design system
- SQLite storage with migrations: settings, projects, sessions, providers,
  models, audit log — persists across restarts (verified by test)
- Settings service with validation on both sides of the bridge
- Project registration with path-existence validation
- Filesystem security boundary (`path_is_within`) + audit trail
- Provider manager: built-in catalog (endpoints only), custom providers,
  models with capability flags, default model selection
- Credentials in the OS keyring; only key-presence booleans reach the UI
- Real connection tests against provider model-listing endpoints
- Agent runtime: adapter trait, registry, OpenCode discovery (real probe,
  honest capability report)
- 24 Rust tests + 9 frontend contract tests, production builds pass

## Milestone 2 — Coding workspace + first live agent

- OpenCode session integration: run, stream events, interrupt (official
  `opencode` CLI/headless interfaces, no terminal scraping)
- Conversation view with streaming, reasoning phases, tool-call display
- File explorer + Monaco editor + diff viewer with change approval
- Terminal with risky-command approval flow
- Session resume/recovery; session list on Home
- shadcn/ui adopted selectively for the denser forms

## Milestone 3 — AI gateway routing + usage

- Request routing through the gateway: model resolution, protocol dialects
  (OpenAI Responses/Chat Completions, Anthropic, Google, Ollama)
- Streaming (SSE/chunked) with backpressure into the event system
- Usage tracking per request/provider; budget enforcement against settings
- Model import from provider `/models` listings during connection test

## Milestone 4 — Extension system

- MCP server management (register, start/stop lifecycle, tool exposure)
- Plugin and skill loading with permission declarations
- Extension-contributed UI surfaces (commands, settings sections)

## Milestone 5 — Local intelligence

- Project memory (embeddings-free first: structured notes + retrieval)
- Context assembly budgeting for long sessions
- Global memory across projects, user-controlled

## Later engines

- Codex adapter, Claude Code adapter, custom agent scaffolding — each gated
  on its adapter implementing honest capability discovery first.

## Windows packaging

- MSI + NSIS bundles (config already targets both) — verified on a Windows
  machine; this Linux sandbox cannot produce or test Windows installers.
