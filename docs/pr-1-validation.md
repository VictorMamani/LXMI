# Frontend rewrite validation

This change fixes runtime updates with an active profile, recovery after a saved
profile fails activation, and keyboard interaction in menus. Maintained UI,
diagnostics, comments, documentation, and demo fixtures now use English.
User-created names, paths, and persisted formats remain compatible.

## Regression coverage

- The runtime update service reads the latest selection revision without trying
  to resolve the previous selection against a new base, then reapplies the profile.
- The demo rejects incompatible runtime/package combinations, checks revisions,
  and keeps applied profile snapshots separate from the editable library.
- A saved profile and its applied runtime are compared by mod identity, including
  replacements that keep the same mod count. Native IPC reads this identity from
  the immutable runtime record and verifies its content hash.
- Saving and activating remain separate operations; the UI refreshes after either
  outcome so a failed activation can be retried without another stale save.
- Menus focus their items on opening, handle arrows, Home/End and Escape, and
  restore the trigger focus. Global shortcuts are suspended while a menu is open.

## Checks

Validated on 2026-10-19:

- Frontend regression suite: 3 passing tests.
- TypeScript, ESLint, Prettier, and production build: passed.
- Rust formatting, workspace Clippy with warnings denied, and workspace tests:
  passed; 413 tests passed and 13 explicit opt-in tests remained ignored.
- Browser demo: active-profile update, package rollback, controlled activation
  failure, retry after navigation, and menu keyboard focus passed.
- Home and Mods layouts inspected at 1366×768 and 1024×640; no page overflow at
  those sizes. Fonts loaded locally. README screenshot uses demo data.
- The opt-in native smoke check passed in the application's WebKitGTK view using
  real Tauri IPC: English document/controls, applied profile identity, and menu
  keyboard focus. It did not mutate the library, execute helpers, or launch games.

Native game launch and sustained gameplay were not exercised by this review.
Browser failure injection does not constitute native mutation testing. The native
smoke check does not operate window controls or certify video decoding.

The historical environment qualification matrix translates authored LXMI warning
text and explicitly labels that translation. Its measurements, recorded byte
hashes, qualification decisions, and outcomes remain unchanged; the summary file
receipt was refreshed to match the translated report. This is not new host evidence.
