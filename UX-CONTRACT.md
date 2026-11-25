# UX Contract

## Product context

- **Audience:** Developers setting up local projects, provider credentials, and coding engines.
- **Primary jobs:** Register a project folder, configure providers/models, test provider connectivity, and discover installed engines.
- **Target market:** Desktop use, Windows first; English interface.
- **Accessibility target:** Keyboard access, visible focus, semantic controls, and WCAG AA contrast intent.
- **Current scope:** The coding session, terminal, editor, and file-editing flows are not available in this build.

## Business and architecture sources

| Scope | Source | Reviewed |
|---|---|---|
| Current milestone and non-shipped features | `ROADMAP.md`, `ARCHITECTURE.md` | 2026-11-25 |
| Filesystem and provider behavior | `ARCHITECTURE.md`, `DECISIONS.md` | 2026-11-25 |
| Persistence and deletion | `src-tauri/src/storage/mod.rs`, `src-tauri/src/workspace/mod.rs`, `src-tauri/src/ai_gateway/providers.rs` | 2026-11-25 |

## Visual and navigation owners

- Visual intent: [`DESIGN.md`](DESIGN.md).
- Runtime tokens: `ui/src/index.css`; CSS `@theme` owns the semantic values and the light selector remaps the same values.
- Navigation: `ui/src/App.tsx` owns the page list and global `Alt+1` through `Alt+4` shortcuts.
- Dialog: `ui/src/components/Dialog.tsx` owns app confirmation and secret-entry modal behavior.
- Project folder selection: Tauri dialog plugin, gated by `dialog:allow-open`.
- Forms and bridge validation: screen-level labels and errors; `ui/src/lib/api.ts` and `schemas.ts` validate backend responses.
- Select/Listbox owner: native HTML `<select>` for fixed, short option sets (`protocol`, `theme`); platform keyboard behavior is accepted and must remain fully labeled.

## Canonical UI Map

| Capability | Canonical owner | Source of truth | Allowed variants | Verification |
|---|---|---|---|---|
| Select/Listbox | Native HTML `<select>` | `DESIGN.md`, `premium-ui.json` | Fixed, short option sets | Keyboard and visible focus |
| Form | Owning page with Rust service validation and Zod response parsing | `UX-CONTRACT.md` | Create / settings save | Production build and focused interaction review |
| Scrollbar | Global application stylesheet | `DESIGN.md` | Shared page scroller | Standards-based and WebKit rules |
| CRUD | Owning list/page and shared `Dialog` | `UX-CONTRACT.md` | Confirm destructive changes | Inline error and return to current list |

## Canonical behavior

| Operation | Trigger | Pending | Success | Failure recovery |
|---|---|---|---|---|
| Add project | New project → Add | Add disabled, folder chooser shows progress | Project appears in list with status message | Keep entered name/path and show inline error |
| Remove project | Remove action → Remove project | Dialog action shows Removing | Remove project and saved session records from ENKI; keep files on disk | Keep dialog open and show the error |
| Save API key | Set key → Save key | Save disabled and labeled Saving | Key saved to OS credential store; field cleared | Keep the dialog open, retain entry only until close, show inline error |
| Delete custom provider | Delete → Delete provider | Dialog action shows Deleting | Provider, its model records, and saved key are removed | Keep dialog open and show the error |
| Test provider | Test | Disable only that provider's button | Show reachability and discovered model count | Show a sanitized network/status message; never display the API key |
| Save settings | Save settings | Save disabled and status says Saving | Persisted values and theme are applied | Keep edited values and show inline error |

## Permission and capability states

- Provider API keys are entered in a masked field and stored by the Rust keyring service. Never put a key in a toast, audit detail, URL, or log.
- Remote HTTP requests with a saved API key are rejected; keyed requests require HTTPS except for loopback providers. Provider connection checks use a bounded timeout and must not hold the database lock during network I/O.
- Project deletion removes ENKI metadata and session records only; it never deletes the project folder.
- Terminal confirmation, file write confirmation, and agent reasoning settings are visibly disabled because those features are not implemented yet. Outside-read settings affect path checks but no project file reads are available in this build.
- Installed engine status is separate from ENKI integration support. The Agents page must not imply that detection means ENKI can run an engine.

## Validation and accessibility

- Forms use associated visible labels, inline errors, and no duplicate submissions.
- Destructive actions name the object and consequence in an app-owned dialog. Cancel is the least destructive initial action.
- Dialogs have an accessible title and description, support Escape, contain focus, and restore focus on close.
- Icon-only actions require accessible names; hover must not be the only way to reveal an action.
- Light and dark themes use the same semantic token names. Reduced-motion and forced-colors settings remain respected.

## Verification status

- Static review covers the current source contract; Windows installer execution is not part of this change.
- Build, lint, keyboard/focus, narrow-window, and folder-picker behavior should be checked before publishing a refreshed installer.
