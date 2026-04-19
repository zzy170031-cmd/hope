# ui

React + TypeScript UI for Hope.

The UI must not bypass Rust commands to touch SQLite directly.

## Track B skeleton status

- Four entry views are wired as hash routes: projects, writer, preview, export.
- Tauri calls are currently mocked behind `src/bridge/hopeBridge.ts`.
- The placeholder command names are documented in `contracts/ui-skeleton-contract.md`.
- Any IPC payload, response shape, or field name still marked as `待 Track A 对齐` must stay as a placeholder until Track A freezes the contract.
