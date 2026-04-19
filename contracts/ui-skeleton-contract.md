# Hope UI Skeleton Contract

Scope: Track B only.

## View entry points

- `#/projects` for project create / switch
- `#/writer` for Writer four-layer view
- `#/preview` for Storyboard / RenderSegment / Cuts preview
- `#/export` for Export / Validation panel

## Tauri command placeholders

These names are stable placeholders only and must be aligned with Track A before real IPC wiring.

- `hope_project_list`
- `hope_project_create`
- `hope_project_switch`
- `hope_writer_snapshot`
- `hope_storyboard_preview`
- `hope_export_validation_snapshot`

## Current UI contract rules

- UI must not access SQLite directly.
- UI may use mock state until Track A finalizes IPC shapes.
- UI must keep all labels and page text in Chinese.
- UI must not invent new business fields beyond the skeleton needs.
- Any field shape shown here is waiting for Track A alignment.

