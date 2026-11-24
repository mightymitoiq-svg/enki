# Contributing to ENKI

Private project; the bar is engineering quality, not velocity.

## Workflow

1. Branch from `main` (`git checkout -b feat/xyz`).
2. Small, complete vertical changes — a feature should land with its tests
   in the same commit series.
3. All checks green before merging:

```bash
# Rust backend
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings

# Frontend
cd ui
npm run build     # tsc -b && vite build
npm test          # vitest
npm run lint
```

4. Update `DECISIONS.md` when you make a non-obvious trade-off, and
   `ARCHITECTURE.md` when module boundaries change.

## Conventions

- **Rust**: module per concern under `src/`; service functions take
  `&Connection` and return `Result<T, AppError>`; commands in `commands.rs`
  stay thin. New tables go through a new migration — never edit an old one.
- **Frontend**: every `invoke` result parsed with a Zod schema in
  `lib/schemas.ts`; add/adjust contract tests there when the backend shape
  changes. Pages stay presentation-focused; data access goes through
  `lib/api.ts`.
- **Honesty over polish**: never add a control that doesn't do real work, and
  never report a capability that isn't verified. Unsupported is rendered as
  unsupported.
- **Security**: any new filesystem/process/network access from agent context
  must pass the existing gates (`path_is_within`, audit log) or introduce an
  explicit, reviewed gate.

## Testing philosophy

- Unit tests next to the code they cover (both languages).
- Contract tests in `ui/src/lib/schemas.test.ts` pin the bridge types.
- Anything user-visible and stateful deserves a persistence round-trip test
  (see `schema_migrations_persist` for the pattern).
