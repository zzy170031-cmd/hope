# Day 1-3 Contract Freeze Checklist

Freeze these before parallel feature work:

## Data and storage

- [ ] `hope-kb` SQLite schema
- [ ] `hope` project SQLite schema
- [ ] `render_segments` contract
- [ ] `project_handoff_zones` contract
- [ ] stale propagation contract
- [ ] foreign key and WAL bootstrap policy

## Runtime traits

- [ ] `PromptRenderer`
- [ ] `LLMProvider`
- [ ] `Validator`
- [ ] `Exporter`

## App contracts

- [ ] Tauri IPC command list
- [ ] TypeScript mirror types
- [ ] JSON schemas

## Workbook contracts

- [ ] 17-sheet workbook structure
- [ ] sheet names as machine identifiers
- [ ] Chinese headers
- [ ] `PromptPackage` traceability columns

## Chinese pipeline freeze

- [ ] all token fields Chinese-only
- [ ] camera vocabulary Chinese-only with aliases
- [ ] no Chinese-to-English translation stage

## Duration rules

- [ ] `Episode` default 22-24 minutes
- [ ] `RenderSegment` target 30-90 seconds
- [ ] empty segment blocked
- [ ] single-cut segment warning
