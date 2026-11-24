# ENKI Decision Log

Architecture decision records, newest last. Each entry: context → decision →
consequences. Trade-offs are stated honestly, including the ones we may regret.

## 1. Tauri 2 over Electron

**Context.** The brief mandates "no Electron" and Windows 11 first; the app is
local-first with a Rust backend requirement.
**Decision.** Tauri 2 with the system WebView (WebView2 on Windows).
**Consequences.** Much smaller binaries than Electron, native Rust backend in
the same process, no Node runtime shipped to users. Cost: WebView
inconsistencies across OSes are Tauri's problem to absorb; on Linux dev boxes
the WebKitGTK dependency chain is heavy (see #7).

## 2. SQLite via rusqlite with hand-rolled ordered migrations

**Context.** We need local persistence that survives restarts; no server.
**Decision.** `rusqlite` (bundled SQLite) with an append-only `MIGRATIONS`
array applied in order, tracked in a `schema_migrations` table. No ORM.
**Consequences.** Zero-magic schema evolution; SQL is reviewable in one file.
Cost: hand-written mapping functions; no compile-time query checking. At the
current scale (7 tables) this is the right trade.

## 3. Credentials in the OS keyring, never in SQLite

**Context.** API keys are the crown jewels; the brief forbids secrets in
frontend storage, logs, or source control.
**Decision.** `keyring` crate (Windows Credential Manager; Secret Service on
Linux) behind a `CredentialStore` trait. The DB stores only
provider metadata; the UI only ever sees a `has_api_key: bool`.
**Consequences.** Keys survive reinstalls of the app data dir, and a leaked
`enki.db` leaks nothing. Cost: keyring backends can fail (locked keyring,
missing service) — handled as typed errors. Orphaned entries are avoided by
deleting the secret before deleting the provider row.

## 4. `ureq` for gateway HTTP

**Context.** Connection tests need one blocking JSON GET per provider; the
full streaming gateway comes later.
**Decision.** `ureq` (rustls, no native TLS) rather than `reqwest`.
**Consequences.** Small dependency tree, synchronous code that is trivial to
test. Cost: when milestone 3 adds SSE streaming we will need async or
chunked reads; ureq 2.x supports reading the body stream, but if it proves
insufficient we will revisit — recorded here so the revisit is a conscious
decision, not drift.

## 5. Model catalog contains endpoints, not model lists

**Context.** The brief forbids inventing model IDs, pricing, or free-tier
availability. Model lineups change monthly; shipping a static list guarantees
staleness.
**Decision.** Built-in providers ship only name/protocol/base URL (publicly
documented, stable). Models are user-added or discovered via the provider's
`/models` listing during connection tests. Capability flags (tools, vision,
JSON schema) default to false and are user-set per model.
**Consequences.** Honest data at the cost of some manual setup. The model
import-from-listing flow (milestone 3) removes most of that cost.

## 6. Engine adapters report capabilities; the UI renders gaps honestly

**Context.** Engines (OpenCode now; Codex, Claude Code later) differ wildly;
the brief says to expose unsupported features honestly rather than simulate.
**Decision.** `AgentEngineAdapter::discover()` returns verified
`EngineCapabilities` plus human notes. Capability flags are only true when the
engine is actually installed and the adapter verified support. The Agents page
strikes through unsupported capabilities.
**Consequences.** The OpenCode adapter currently reports discovery-only (its
notes say session streaming is milestone 2) rather than pretending to run
chat. No fake buttons anywhere in the UI.

## 7. Local sysroot for Linux CI builds (environment-specific)

**Context.** This build environment has no root; WebKitGTK/GTK dev packages
are needed to compile Tauri for Linux at all. Windows remains the target.
**Decision.** Downloaded the required `*-dev`/runtime debs into
`/home/z/enki-sysroot/sysroot`, repaired broken dev symlinks (dev packages
ship `libX.so -> libX.so.N` links whose targets live in runtime packages),
and build with `PKG_CONFIG_PATH`/`PKG_CONFIG_SYSROOT_DIR`/`LD_LIBRARY_PATH`
pointing there.
**Consequences.** Full `cargo test`/`cargo build` works without root. Cost:
Linux packaging in this environment is not representative of the Windows
target; Windows builds use the standard MSVC toolchain and none of this
sysroot machinery.

## 8. Frontend Zod-validates every backend response

**Context.** The Tauri bridge crosses a serialization boundary; Rust types and
TS types can drift silently.
**Decision.** Every `invoke` wrapper parses its result with a Zod schema that
mirrors the Rust serde type (snake_case preserved). Schema tests pin the
contract.
**Consequences.** Drift fails loudly at the boundary instead of corrupting UI
state. Cost: duplicated type definitions in Rust and TS — accepted because the
alternative (codegen) adds a build step for a contract that changes slowly.

## 9. Dark-first design tokens in Tailwind 4 `@theme`

**Context.** The brief demands a premium dark-first identity: charcoal
surfaces, restrained electric blue/violet, thin borders, no glassmorphism.
**Decision.** CSS-first custom properties in `@theme` (`--color-base` …
`--color-violet`), semantic names (`base/surface/line/ink/accent`), no
component library heavyweight — plain Tailwind utilities with a few shared
`cn()` helpers. shadcn/ui components will be added selectively in milestone 2
where forms multiply.
**Consequences.** Total control of the visual language; no fighting a
component theme. Cost: we hand-build more UI primitives than a shadcn-first
approach would; the trade favors the custom identity requirement.
