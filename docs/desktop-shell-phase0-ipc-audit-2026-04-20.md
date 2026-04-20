# Hope Desktop Shell Phase 0 IPC Audit 2026-04-20

## Purpose

This audit freezes the current desktop-shell bridge baseline before more panel work continues.

It exists to satisfy the Phase 0 requirement added in Notion:

- inventory the UI needs against the current IPC surface
- state which calls are already available, partial, or missing
- define the allowed Phase 1 bridge work
- prevent Track B from expanding into ad-hoc runtime or contract work

## Current Baseline

- Repo: `E:\codex\hope-desktop-shell`
- Branch: `codex/desktop-shell`
- Base commit: `ecff1da`
- Current shell status:
  - `ui/src/bridge/hopeBridge.ts` already moved toward `IPC-first + mock fallback`
  - `app/src/runtime.rs` already contains snapshot builders for the four current panels
  - `app/src/ipc.rs` defines request structs and command names
  - `app/src/main.rs` is still only a shell skeleton and does not yet register a real desktop invoke bridge

## Environment Baseline

- Rust baseline locked in `rust-toolchain.toml`: `1.94.1`
- Node baseline locked in `ui/.nvmrc`: `24.14.0`
- UI package manager remains `npm`

## IPC Audit Table

| UI panel | Required bridge call(s) | `app/src/ipc.rs` status | Current runtime/app status | Phase 1 strategy |
| --- | --- | --- | --- | --- |
| Projects | `project_create_or_switch` | Existing | Partial: request shape exists, runtime snapshot builder exists, desktop command registration is still missing | Real in Phase 1 after desktop invoke registration |
| Writer | `writer_entry_snapshot` | Existing | Partial: request shape exists, runtime snapshot builder exists, desktop command registration is still missing | Real in Phase 1 after Projects path is verified |
| Storyboard Preview | `storyboard_rendersegment_cut_preview_snapshot` | Existing | Partial: request shape exists, runtime snapshot builder exists, desktop command registration is still missing | Hold until Projects and Writer pass tests |
| Validation / Export panel | `validation_export_panel_snapshot` | Existing | Partial: request shape exists, runtime snapshot builder exists, desktop command registration is still missing | Hold until Preview path is verified |
| Export trigger | Not yet in current desktop scope | Missing by design | Not planned in current shell phase | Keep mock / deferred, do not add in this phase |
| Writer save / mutation flows | Not yet in current desktop scope | Missing by design | Not planned in current shell phase | Keep deferred, do not add in this phase |

## Findings

1. The current desktop branch has already moved beyond a pure mock UI, but it is not yet a real desktop IPC path.
2. All four current panel snapshot calls are `partial`, not `done`.
3. The main blocker is not UI rendering anymore; it is desktop-side invoke registration and the final wiring from `app` into the bridge.
4. Projects and Writer are the correct first real panels because they are lower-risk than Preview and Export.
5. Preview and Export should not continue to deepen until the first two panels are proven through tests.

## Allowed Phase 1 Work

Phase 1 may only do the following:

1. register a real desktop invoke path for the four existing snapshot commands
2. finish the Projects panel against the real IPC path
3. finish the Writer panel against the real IPC path
4. keep Preview and Export behind fallback until the first two panels are green
5. run tests after each completed panel step

## Explicitly Deferred

These items are not part of the current desktop-shell phase:

- new contract design
- `hope-kb` integration logic
- product-side runtime consume implementation
- export-trigger mutation flows
- writer save / edit mutation flows
- deeper Rust pipeline refactors

## Required Test Gates

After each panel step completes:

1. run the relevant Rust tests for the touched area
2. run `npm run typecheck`
3. run `npm run build`
4. only continue if all of the above are green

This gate applies in this order:

1. Projects
2. Writer
3. Preview
4. Validation / Export

## Next Action

The next desktop-shell implementation step is:

1. add the desktop invoke registration layer for the four existing snapshot commands
2. make `project_create_or_switch` real first
3. then make `writer_entry_snapshot` real
4. stop and test before touching Preview or Export again
