# Desktop Single-Page Workbench 2026-04-23

## Package Intent

- repo: `E:\codex\hope-desktop-shell`
- branch: `codex/desktop-shell`
- base checkpoint: `f359f99`
- control dispatch: `E:\codex\hope\docs\desktop-single-page-workbench-dispatch-2026-04-23.md`

This package replaces the old multi-route desktop shell with a single-page Chinese workbench.

## UI Truth Rules

- top bar exposes product-level controls only
- script area keeps story input and local script expansion honest
- task strip and storyboard output run from front-end working state
- export is front-end only from the current workbench state

## Frozen Boundary Kept

- no live Qwen
- no Seedance
- no KB runtime growth
- no V108 import
- no new IPC
- no bridge widening
- no `app/src` or `crates/**` changes
