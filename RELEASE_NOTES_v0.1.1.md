# ENKI v0.1.1 — Desktop Foundation Update

This update improves setup and provider management while keeping the current milestone's scope clear. Agent session execution, conversation, editor, and terminal features are still not available.

## What's improved

- Labeled, collapsible navigation with keyboard shortcuts and clearer session availability status.
- Light theme now applies when selected and saved.
- Project registration uses the native folder picker. Project and provider removal use in-app dialogs that explain what is removed.
- Provider API keys use a masked input dialog, and save status is shown in the interface.
- Unsupported settings are clearly marked as inactive in this build.

## Security and reliability

- Added a restrictive production content security policy and a separate development policy.
- Remote provider endpoints require HTTPS; plain HTTP is limited to loopback services. URLs containing embedded usernames or passwords are rejected.
- Google provider keys use the `x-goog-api-key` header. Provider connection requests have a timeout and errors avoid exposing credentials or endpoint details.
- Provider network requests run outside the database lock.
- OpenCode detection no longer implies that ENKI can run or control agent sessions.

## Downloads

- Windows: MSI and NSIS setup installers
- Linux: deb, rpm, and AppImage
- Installers are unsigned and may show an operating-system trust warning.
