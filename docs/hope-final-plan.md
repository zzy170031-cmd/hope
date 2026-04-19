# Hope Final Plan

This repo follows the frozen Hope v0.1 / v1 direction:

- desktop-first product
- all-Chinese pipeline
- Qwen-compatible primary text route
- `jimeng` / `kling` as preferred downstream verification targets
- no in-product image or video generation
- Excel as the final delivery artifact
- `Project -> Episode -> NarrativeScene -> RenderSegment -> Cut`

## Canonical v0.1 journey

1. input synopsis or script
2. generate Story layer
3. generate Screenplay layer
4. plan RenderSegments
5. generate Cuts
6. create handoff zones
7. create layout prompts
8. create render prompts
9. validate
10. export Excel

## Key hard rules

- `RenderSegment` cannot cross `NarrativeScene`
- project `hard_locks` are global across all episodes
- `actual_duration_seconds` is derived from cuts
- all token fields are Chinese
- Excel is the main delivery artifact
