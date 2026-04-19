# Track F Validators Contract

## Anchor

- `E:\codex\hope\crates\core-domain\src\traits.rs` -> `Validator` / `Exporter`
- `E:\codex\hope\contracts\track-a\excel-17-sheet.md` -> `validation_report`

## Shared summary contract

- `pass` means zero findings.
- `warn` means only warning-level findings, no blocking findings.
- `block` means at least one blocking finding.
- `validation_report` sheet stays fixed to the existing five columns: `校验报告标识`、`项目标识`、`通过`、`问题数`、`更新时间戳`.
- Track F does not add new sheet columns and does not rename the existing sheet fields.

## Shared runtime types

- `ValidationEnvelope` carries the sheet anchor: `validation_report_id`, `project_id`, `updated_at_timestamp`.
- `ValidationFinding` carries internal validator detail: `code`, `severity`, `subject`, `message`, `failure_sample`.
- `ValidationReport` carries the validator findings plus the `ValidationEnvelope`.
- `ValidationReportRow` mirrors the Validation sheet columns exactly.

## Shared fixture

- Input is fixed to `E:\codex\hope\contracts\fixtures\week3-shared-fixture.json`.
- Output is fixed to `E:\codex\hope\contracts\fixtures\week3-validation-report.json`.
- Track F reads only the shared fixture and does not keep private sample inputs.
- `validation_report_id` is the trace envelope and must embed the shared fixture trace, at minimum one of `render_segment_id`, `cut_id`, or `handoff_zone_id`, plus the validator target.

## Validator matrix

| Validator | Input contract | Output contract | pass | warn | block | Minimal failure sample | Validation sheet mapping |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `HardLockInjectorGuard` | `ValidationEnvelope` + `hard_lock_id` + `lock_name` + `lock_value` + `scope` + frozen `allowed_lock_names` | `ValidationReport` | `scope = project`, lock name belongs to the frozen hard-lock set, and the value is non-empty | placeholder markers appear in the record but the hard-lock shape is still valid | scope is not `project`, the lock name is not in the frozen set, or a required field is empty | `scope = episode` | One summary row in `validation_report`; detail stays inside the report object, not in new sheet columns |
| `StyleUnityValidator` | `ValidationEnvelope` + `prompt_package_id` + `director_profile_id` + `director_cut_sample_id` + `committee_template_id` + `prompt_text` + frozen `visual_terms` + frozen `cinematography_terms` | `ValidationReport` | prompt text matches at least one visual term and one cinematography term | only one vocabulary family matched, or placeholder markers remain in the prompt | any anchor ID is missing, the frozen vocab sets are missing, or the prompt fails to match both vocab families | prompt text contains no style language and no cinematography language | One summary row in `validation_report`; source sheets remain `director_profile`, `director_cut_sample`, `committee_template`, `visual_term`, `cinematography_term`, `prompt_package` |
| `NegativeGlobalGuard` | `ValidationEnvelope` + `prompt_package_id` + `prompt_text` + frozen `blocked_phrases` | `ValidationReport` | prompt text contains none of the frozen blocked phrases | placeholder markers remain in the prompt or blocked-phrase list, but no frozen phrase is hit | any frozen blocked phrase is present in the prompt, or the blocked-phrase set is missing | prompt text contains a frozen blocked phrase | One summary row in `validation_report`; the blocked phrase source stays outside the sheet, in the frozen hard-lock set |
| `ContinuityValidator` | `ValidationEnvelope` + `stale_event_id` + `source_level` + `source_id` + `target_level` + `target_id` + `trace_id` | `ValidationReport` | source and target identifiers are present, levels stay inside the frozen hierarchy, and source does not equal target | placeholder markers remain in the trace path | any source/target field is missing, the levels escape the frozen hierarchy, or the event points back to itself | `source_level = target_level` and `source_id = target_id` | One summary row in `validation_report`; the detailed stale event stays in the `stale_event` sheet |
| `SegmentDurationValidator` | `ValidationEnvelope` + `render_segment_id` + `narrative_scene_id` + `target_duration_seconds` + optional `actual_duration_seconds` | `ValidationReport` | target duration stays within `30s-90s` and the identity fields are present | actual duration is still missing, but the segment target is in bounds | target duration escapes `30s-90s`, actual duration escapes the frozen bound, or segment identity is missing | `target_duration_seconds = 120` | One summary row in `validation_report`; the source entities remain `render_segment` and `cut` |
| `LayerTraceabilityValidator` | `ValidationEnvelope` + `trace_id` + `source_level` + `source_id` + `target_level` + `target_id` + `render_segment_id?` + `cut_id?` | `ValidationReport` | trace identity is present, the hierarchy levels stay inside the frozen set, and at least one entity anchor exists | only one of `render_segment_id` or `cut_id` is present; traceability is partial but recoverable | trace identity is missing, both entity anchors are missing, or the levels escape the frozen hierarchy | `trace_id = ""` | One summary row in `validation_report`; detail links back through the trace fields, not through new workbook columns |
| `HandoffCoverageValidator` | `ValidationEnvelope` + `handoff_zone_id` + `render_segment_id` + `start_boundary` + `end_boundary` + `boundary_type` | `ValidationReport` | the handoff zone resolves to exactly one render segment and the boundaries are both present and distinct | placeholder markers remain in the boundary text | any identity field is missing, the boundaries collapse to the same value, or the boundary type is missing | `start_boundary = end_boundary` | One summary row in `validation_report`; the detailed zone remains in `handoff_zone` |

## Minimal failure samples

- `HardLockInjectorGuard`: `scope = episode`
- `StyleUnityValidator`: prompt text has no visual term and no cinematography term
- `NegativeGlobalGuard`: prompt text contains a frozen blocked phrase
- `ContinuityValidator`: `source_level == target_level` and `source_id == target_id`
- `SegmentDurationValidator`: `target_duration_seconds = 120`
- `LayerTraceabilityValidator`: `trace_id` is blank
- `HandoffCoverageValidator`: `start_boundary == end_boundary`

## Track dependencies

- Track E supplies the render-segment, handoff-zone, and stale-trace structure that Track F validates.
- Track G consumes the `validation_report` summary row and uses `problem_count` plus `passed` as the exported status gate.
- The main thread owns the frozen hard-lock vocabulary and any blocked-phrase list that Track F needs to consume.
