# Track E Storyboard Cuts Contract

## Scope

This contract freezes the minimal runtime shape for Track E:

- `RenderSegment` planning
- cut-level storyboard generation
- committee runtime assembly
- director assignment aggregation
- handoff zone generation
- `layout_prompt` / `render_prompt` two-layer payload pairing

It does not add new schema tables, new columns, or new trait signatures. It only composes the frozen Track A contracts and the single-source trait / IPC definitions already owned by other modules.

## Single Sources

- Trait names and signatures come from `crates/core-domain/src/traits.rs`
- IPC command names come from `app/src/ipc.rs`
- Excel sheet names and frozen columns come from `contracts/track-a/excel-17-sheet.md`

Track E must not define a second copy of `PromptRenderer`, `LLMProvider`, `Validator`, or `Exporter`.

## Frozen Runtime Types

The minimal Track E runtime uses the following shapes:

- `StoryboardPlanRequest`
- `RenderSegmentPlan`
- `CutPlan`
- `HandoffZonePlan`
- `PromptLayers`
- `DirectorAssignment`
- `CommitteeRuntimePlan`
- `StoryboardPlan`

These are runtime wrappers only. They do not change schema shape.

## Frozen Rules

- `RenderSegment` target duration is `30s-90s`
- `RenderSegment` cannot cross `NarrativeScene`
- one `CutPlan` belongs to one `RenderSegment`
- one `handoff_zone` belongs to one `render_segment`
- `handoff_zone` references `render_segment` boundaries only and does not live inside the segment
- `committee runtime` may derive prompts and assignments, but it must not mutate `hard_locks`
- `primary_scene_director_id` and `primary_action_director_id` are runtime aggregate fields
- if one director id is missing, the present one can be duplicated into both primary slots
- if both director ids are missing, the plan is invalid

## Field Freeze

The Track A column set is reused as-is:

- `render_segment`: `render_segment_id`, `narrative_scene_id`, `序号`, `起始镜头序号`, `结束镜头序号`, `目标时长秒`, `实际时长秒`
- `cut`: `cut_id`, `render_segment_id`, `序号`, `镜头描述`, `对白`, `时长秒`
- `handoff_zone`: `handoff_zone_id`, `render_segment_id`, `起始边界`, `结束边界`, `边界类型`

No extra storyboard fields are allowed in the contract freeze.

## Workbook Mapping

Track E feeds the frozen Excel sheets by reference only:

- `render_segment` -> sheet 4
- `cut` -> sheet 5
- `prompt_package` -> sheet 6
- `handoff_zone` -> sheet 7
- `validation_report` -> sheet 10

Track E must not rename these sheets or introduce new workbook columns.

## Shared Fixture Source

The only shared storyboard fixture source is:

- [`contracts/fixtures/week3-shared-fixture.json`](../fixtures/week3-shared-fixture.json)

This fixture is the canonical sample source for Track E, Track F, and Track G. No private sample, inline sample, or alternate fixture is maintained for this track.

## Fixture Shape

The fixture contains:

- one `render_segment`
- three `cuts`
- one `handoff_zone`
- `layout_prompt`
- `render_prompt`
- `traceability`
- frozen prompt package trace fields
- committee runtime director assignment fields

## Out Of Scope

- image generation
- video generation
- schema edits
- extra prompt trait signatures
- `hard_locks` mutation
- exporter-led schema changes
