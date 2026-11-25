---
version: alpha
name: ENKI
description: A focused local development workbench, using quiet charcoal surfaces and precise blue-violet signals.
colors:
  primary: "#5b8cff"
  background: "#0a0b0f"
  surface: "#101218"
  text: "#e8eaf2"
  muted: "#8992a7"
  danger: "#f06272"
typography:
  sans:
    fontFamily: "Segoe UI Variable, Segoe UI, Inter, system-ui, sans-serif"
  mono:
    fontFamily: "Cascadia Code, JetBrains Mono, Consolas, monospace"
rounded:
  sm: "0.25rem"
  DEFAULT: "0.375rem"
  md: "0.5rem"
  lg: "0.75rem"
spacing:
  section-gap: "2rem"
  page-max: "72rem"
components:
  button: {}
  card: {}
  dialog: {}
---

# ENKI Design System

## Overview

### Creative North Star

ENKI should feel like a well-kept developer workbench: dark matte surfaces, readable labels, and a narrow electric-blue signal that marks the current task. The product is a utility for setting up local coding work, not a promotional dashboard.

### Product context and register

- **Audience and primary job:** Developers preparing local projects, AI providers, and installed agent engines.
- **Target market and locale:** English desktop application; no country-specific workflows are implemented.
- **Usage scene:** Windows desktop first, with enough room for project paths and dense provider configuration.
- **Register:** Product interface.
- **Memorable signature:** A labeled, collapsible workspace rail with a small blue active marker.
- **Restraint:** Keep cards flat and technical. Use gradients only for the home introduction and brand mark.
- **Anti-references:** No glass panels, oversized metrics, or controls that imply agent execution is already available.
- **Token ownership:** `ui/src/index.css` is the runtime owner. This file documents its accepted values and rationale; CSS variables in `@theme` feed Tailwind utilities, and the light theme overrides those same semantic variables on `:root[data-theme="light"]`.

## Colors

Dark mode uses `background` `#0a0b0f`, `surface` `#101218`, elevated surfaces `#161923` and `#1d2130`, `text` `#e8eaf2`, and `muted` `#8992a7`. The primary signal is blue `#5b8cff`; violet `#9d7bff` is a secondary brand accent. Danger is `#f06272`. Light mode remaps the same semantic tokens to cool paper-white surfaces and dark text; actions, focus, and semantic states retain their meaning. Keep text readable without relying on color alone.

## Typography

Use Segoe UI Variable first for interface text, with Segoe UI, Inter, and system fallbacks. Use Cascadia Code first for paths, model IDs, and versions. Keep headings sentence case and reserve uppercase mono text for small section markers.

## Layout

The app is a desktop shell with a 240px expanded navigation rail and a compact 72px collapsed rail. The rail remembers its state. Content uses a 72rem maximum width, natural page scrolling, and stable controls. On narrower windows, forms stack rather than compressing labels or paths.

## Elevation & Depth

Use tonal differences and thin borders for hierarchy. Do not add drop shadows to ordinary cards. Dialogs may use a clear overlay and shadow so they read above the app.

## Shapes

Use 4px control corners, 6px standard corners, and 12px for larger surfaces. Prefer rectangular work surfaces with restrained rounding; reserve pill shapes for small state labels.

## Components

### Foundational visual states

Hover changes surface or border, keyboard focus uses a visible 2px blue outline, and disabled controls retain legible text while looking inactive. Busy buttons keep their dimensions. Errors use both an icon or label and text.

### Buttons and actions

Solid blue is the primary action, outlined buttons are neutral, and red is reserved for delete. Every icon-only action has an accessible label and tooltip.

### Navigation and data display

Navigation is labeled when expanded and uses tooltips plus accessible names when collapsed. Provider/model lists keep technical identifiers in mono type and expose a clear empty state.

### Forms and overlays

Use visible labels and inline validation. App-owned modal dialogs use the native HTML dialog element for focus containment, Escape, and focus restoration. Native OS dialogs are reserved for selecting a local folder.

### Iconography

Use Lucide React with a consistent 16–18px outline stroke. Icons supplement labels; they do not replace labels where space permits.

### Motion

Use short transitions for navigation and state changes. Respect `prefers-reduced-motion`; motion does not decorate routine content.

### Content

Use direct, plain language. Label only capabilities the current build actually performs. State that live coding sessions, file editing, and terminal execution are not yet available.

## Do's and Don'ts

- **Do:** Keep the existing charcoal and blue-violet identity while improving hierarchy and contrast.
- **Do:** Tell users when a setting is stored but not active in this build.
- **Don't:** Show a detected engine as if ENKI can already use every engine capability.
- **Don't:** Hide actions until pointer hover or rely on tiny low-contrast helper text.
