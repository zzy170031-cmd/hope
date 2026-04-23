# Hope Desktop UI

React + TypeScript desktop shell for the Hope MVP internal trial.

The UI must not bypass Rust commands to touch SQLite directly.

## Current desktop shell scope

- Four packaged entry surfaces remain in place: Projects, Script Workbench, Storyboard Review, and Export Center.
- The shell prefers packaged desktop IPC when it is available and falls back to fixture data only in browser preview mode.
- Writer readiness captures a local input package only; it does not call Qwen, Seedance, or write export payloads.
- UI alignment can improve naming, layout, and workflow clarity, but it must not widen backend contracts.
