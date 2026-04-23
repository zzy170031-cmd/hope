# Seedance2 V120 V3 Docs Alignment Refresh 2026-04-23

## Route Metadata

| Item | Value |
| --- | --- |
| repo | `E:\codex\hope` |
| proposal branch | `codex/v3-field-overlay-proposal` |
| V3 starting anchor | `bc745a4` (`docs: align V3 proposal with Seedance2 V108`) |
| Hope main-control sources reviewed | `origin/codex/contracts-freeze:docs/seedance2-v120-full-kb-ingest-review-2026-04-23.md`; `origin/codex/contracts-freeze:docs/v120-v3-kb-full-ingest-dispatch-2026-04-23.md` |
| comparison baseline | `docs/contracts/export-overlays/seedance2-v108-v3-docs-alignment-refresh-2026-04-23.md` |
| handoff type | docs-only V120 primary-source alignment refresh |
| implementation scope | no product code, no exporter, no IPC, no Rust DTO, no validator, no desktop, no intake, no workbook, no Qwen/Seedance integration, no hope-kb mutation |

## Canonical Source Format Decision

V120 is now the latest primary source for KB-ingest-oriented V3 alignment work.
V108 remains the accepted comparison baseline.

This refresh updates only V3 docs meaning and source boundaries. It does not
import V120 rows into the current 40-row v0.2 sample package, does not change
Hope runtime truth, does not promote V120 rows into positive few-shot, and does
not authorize implementation.

## Source Materials Read

| Source | Location |
| --- | --- |
| V120 full-ingest review snapshot | `docs/contracts/export-overlays/source-materials/seedance2-v120-full-kb-ingest-review-2026-04-23.md` |
| V120 synchronized dispatch snapshot | `docs/contracts/export-overlays/source-materials/v120-v3-kb-full-ingest-dispatch-2026-04-23.md` |
| V108 comparison baseline addendum | `docs/contracts/export-overlays/seedance2-v108-v3-docs-alignment-refresh-2026-04-23.md` |
| Existing V3 field overlay | `docs/contracts/export-overlays/v3-field-overlay-proposal-2026-04-22.md` |
| Existing golden sample schema overlay | `docs/contracts/export-overlays/golden-sample-v0.2-schema-overlay-proposal-2026-04-22.md` |
| Existing golden sample freeze candidate | `docs/contracts/export-overlays/golden-sample-v0.2-contract-freeze-candidate-2026-04-22.md` |

## V120 Source Package Facts

| Item | Value |
| --- | --- |
| V120 working sheet row count | `120` |
| V120 field count | `23` |
| schema compatibility vs V108 | compatible with the accepted V108 23-column source shape |
| V120 status counts | `official=108`, `reserve=12` |
| V120 reviewed sample-type counts | `single_shot=72`, `sequence_shot=48` |
| V120 sequence groups | `12`, with `CNSEQ04`, `CNSEQ05`, `CNSEQ06` newly added |
| source priority | `V120 = primary`, `V108 = comparison baseline` |
| current live KB sample reality inside Hope | unchanged: `golden_sample_library=40`, `golden_sample_field_coverage_rules=5`, `golden_sample_failure_mapping=40`, `golden_sample_repair_mapping=40` |

## V120 vs V108 Delta Summary

Comparison uses the reviewed source-package facts published by the current V120
main-control memo and the accepted V108 alignment addendum.

| Item | V108 baseline | V120 primary | Delta / decision |
| --- | --- | --- | --- |
| source field count | `23` | `23` | no source-column delta |
| total source rows | `115` | `120` | `+5`; V120 expands total source coverage |
| `official` rows | `108` | `108` | no delta |
| `reserve` rows | `7` | `12` | `+5`; reserve-only surfaces expand |
| reviewed `single_shot` rows | `72` | `72` | no reviewed count delta |
| reviewed `sequence_shot` rows | `36` | `48` | `+12`; stronger sequence/chase/ensemble emphasis |
| reviewed sequence groups | `9` | `12` | `+3`; adds `CNSEQ04`, `CNSEQ05`, `CNSEQ06` |
| source priority | primary | comparison baseline | V120 supersedes V108 as the current source of truth |
| runtime positive few-shot promotion | closed | closed | still not authorized |

The source-schema conclusion is simple: V120 changes count, emphasis, and
source priority, but it does not change the 23-field workbook shape.

## V120 Field Set

```text
shot_id
library_status
reserve_reason
sample_type
sequence_id
shot_order
sample_title
style_cluster
scene_category
scene_tag
quality_grade
usable_for_fewshot
technical_profile
scene_performance_core
camera_directing_core
audio_directing_core
continuity_negative_core
reference_bundle
ip_abstraction_note
covered_points
missed_points
teaching_note
prompt_body
```

## Field Mapping Changes

| V120 field | historical V3 / old golden sample field | current decision | status | notes |
| --- | --- | --- | --- | --- |
| `shot_id` | `sample_id`, `shot_id` | Preserve V120 source value; future import may map to row/sample IDs. | rename | Do not rewrite existing 40-row IDs in this docs refresh. |
| `library_status` | none exact | Source governance field. | new | `reserve` rows remain excluded from positive few-shot; V120 expands reserve rows from `7` to `12`. |
| `reserve_reason` | `repair_hint`, `negative_boundary_marker` analogues | Reserve-only / conditional metadata. | new | Blank official cells are not missing content. |
| `sample_type` | old golden sample `sample_type=positive/negative/repair_before/repair_after`; V3 `structure_mode` | V120 still means `single_shot` or `sequence_shot`. | rename + v3_alignment_gap | The old sample-category meaning must not be reused here. |
| `sequence_id` | `shot_beats_link`, `sequence_no`, `shot_beats` | Future sequence grouping surface. | future_model_fill | V120 adds `CNSEQ04`, `CNSEQ05`, `CNSEQ06`; blank single-shot cells remain expected. |
| `shot_order` | `beat_order`, `sequence_no`, `shot_beats` | Future sequence ordering surface. | future_model_fill | Blank single-shot cells remain expected. |
| `sample_title` | `sample_title` | Preserve as title / retrieval label. | exact | Directly compatible. |
| `style_cluster` | `style_profile_id`, `retrieval_tags` | Retrieval taxonomy candidate. | new | Internal only; not imitation language. |
| `scene_category` | `kb_scene_type`, `shot_function_code`, `retrieval_tags` | Scene taxonomy input. | rename | Needs future taxonomy review. |
| `scene_tag` | `scene_tag`, `retrieval_tags` | Preserve as source tag. | exact | Current KB v0.2 has analogous scene tags. |
| `quality_grade` | `tier` | Preserve source grade. | rename | Replaces old `tier` naming for V120/V108 source rows. |
| `usable_for_fewshot` | `usable_for_fewshot`, `fewshot.eligible` | Preserve as source gate only. | exact | Still blocked by reserve, negative, placeholder, and reference gates. |
| `technical_profile` | `target_clip_duration_sec`, `shot_type`, `transition_note`, `lens_feel`, model capability fields | Split into technical subfields only after a future schema gate. | split | V120 keeps the same future-fill technical surface; exact placeholder counts must be recomputed from ingest outputs. |
| `scene_performance_core` | `visual_scene_core` + `motion_performance_core` | V120 canonical source still fuses these old axes. | merge + v3_alignment_gap | Do not hard-split without a future accepted rule. |
| `camera_directing_core` | `camera_directing_core` | Preserve name as source content field. | exact | Current KB v0.2 stores evidence rows, not top-level prompt content. |
| `audio_directing_core` | `audio_directing_core` | Preserve name as source content field. | exact | Current KB v0.2 stores evidence rows, not top-level prompt content. |
| `continuity_negative_core` | `continuity_lock_core`, `blocking_failure_codes`, `negative_boundary_marker` | V120 still combines continuity and negative constraints. | split + v3_alignment_gap | Future schema must decide the split. |
| `reference_bundle` | `reference_control_core`, `asset_registry`, `image_ref_assets`, `video_ref_assets`, `audio_ref_assets` | Normalize only to `external_reference_handles`. | rename + v3_alignment_gap | Not paths, URLs, asset IDs, or completed reference-control coverage. |
| `ip_abstraction_note` | `rights_or_ip_risk_flag`, `copyright_risk_flag`, `negative_boundary_marker` | Compliance / abstraction evidence. | new | Not product prompt expansion. |
| `covered_points` | `covered_points`, `covered_core_fields`, `covered_atomic_fields` | Future validator evidence. | exact | No implementation in this gate. |
| `missed_points` | `missed_points`, `missing_items` | Future validator / repair evidence. | exact | No implementation in this gate. |
| `teaching_note` | `teaching_note`, `repair_hint` | Future repair rationale / reviewer trace. | exact | Not runtime behavior now. |
| `prompt_body` | `sample_text`; compiled Seedance prompt fields | Preserve as source prompt body / candidate text only. | rename + v3_alignment_gap | Not adapter output truth. Placeholder-bearing rows remain blocked. |

## Deprecated Or Downscoped Historical Assumptions

| Historical item | V120-aligned disposition |
| --- | --- |
| V108 as the latest primary source format | superseded; V120 is now primary and V108 becomes the comparison baseline |
| old golden sample `sample_type` as positive/negative/repair category | do not use for V120 `sample_type`; if sample role is needed later, name it separately in a future gate |
| direct `reference_control_core` backfill | still not covered; V120 `reference_bundle` only produces external handle candidates |
| `image_ref_assets`, `video_ref_assets`, `audio_ref_assets` as source fields | adapter/export bindings only after a future gate; not V120 source truth |
| `compiled_prompt_seedance` / `compiled_payload_seedance` | derived adapter output only; V120 `prompt_body` is source candidate text |
| old separate `visual_scene_core` and `motion_performance_core` as V120 source columns | superseded by V120 `scene_performance_core` for this source package |
| old standalone `continuity_lock_core` as the only continuity source | superseded by V120 `continuity_negative_core`, which needs a future split rule |
| importing V120 rows into current v0.2 sample rows | closed; V120 remains raw source and future package-ingest input |
| positive few-shot promotion from V120 | closed; this gate promotes zero V120 rows into live Hope runtime |

## unresolved_gaps / v3_alignment_gap List

The following gaps remain intentionally preserved and must not be hard-merged:

1. `sample_type`: V120 uses shot-structure semantics, while old V3
   golden-sample material used sample-category semantics.
2. `scene_performance_core`: V120 still merges old `visual_scene_core` and
   `motion_performance_core`.
3. `continuity_negative_core`: V120 still combines continuity locks,
   prohibitions, empty-word blacklist, and risk evidence.
4. `reference_bundle`: V120 still maps only toward
   `external_reference_handles`, not asset binding, concrete media paths, or
   completed `reference_control_core`.
5. `prompt_body`: V120 source prompt text is not compiled Seedance adapter
   output.
6. Current v0.2 stores one main `classification.core` per sample, while V120
   carries multiple fused core fields per row.
7. Current v0.2 has no top-level fields for `external_reference_handles`,
   `technical_profile`, `scene_performance_core`, `prompt_body_candidate`, or
   sequence grouping.
8. The V120 review publishes row/count deltas but does not publish refreshed
   field-level placeholder counts or placeholder-clean few-shot counts; KB must
   compute those from actual ingest outputs instead of inheriting V108 counts.

## future_model_fill_surface List

The `future_model_fill_surface` concept remains active under V120. Because the
current V120 control review does not publish refreshed field-level placeholder
counts, the V108-reviewed counts below remain the comparison baseline only.
They must be recomputed during KB ingest and must not be assumed unchanged.

| Surface | V108 baseline count | V120 current status | Required handling |
| --- | ---: | --- | --- |
| blank `reserve_reason` on official rows | 108 | semantic rule unchanged; exact V120 count not restated | Conditional metadata; not missing sample content. |
| blank `sequence_id` on single-shot rows | 79 | recompute required from V120 package | Future sequence surface; do not count as sequence records. |
| blank `shot_order` on single-shot rows | 79 | recompute required from V120 package | Future sequence surface; do not count as sequence records. |
| placeholder-like `reference_bundle` | 102 | recompute required from V120 package | Preserve as gap; do not invent paths, URLs, asset IDs, or handles. |
| placeholder-like `prompt_body` | 102 | recompute required from V120 package | Block `prompt_body_candidate`; do not guess-fill. |
| placeholder-like `technical_profile` | 41 | recompute required from V120 package | Future model/schema fill only. |
| placeholder-like `scene_performance_core` | 41 | recompute required from V120 package | Future model/schema fill only. |
| placeholder-like `camera_directing_core` | 42 | recompute required from V120 package | Future model/schema fill only. |
| placeholder-like `audio_directing_core` | 67 | recompute required from V120 package | Future model/schema fill only. |
| placeholder-like `reserve_reason` | 1 | recompute required from V120 package | Preserve as source gap; do not promote. |

No empty V120 working sheet was observed in the reviewed control memo. If a
later V120 package introduces empty worksheets, empty tables, or additional
blank template areas, they should be recorded under the same
`future_model_fill_surface` rule until main control opens a fill/import gate.

These surfaces are not positive few-shot material and are not current validator
evidence.

## external_reference_handles Boundary

`reference_bundle` may only be interpreted as source input for
`external_reference_handles`.

Allowed handle categories:

```text
character names
scene names
prop names
style names
continuity object names
```

Disallowed in this V120 docs refresh:

```text
image paths
URLs
asset IDs
real media bindings
invented character appearance
invented scene design
completed reference_control_core coverage
```

Prompt wording may describe action, expression, tone, camera, light, rhythm,
and continuity relationships. Character and scene references should use clear
names only after a future gate confirms canonical external object names.

## Positive Few-Shot Boundary

V108 comparison-baseline facts remain:

```text
official few-shot Yes rows = 97
official few-shot No rows = 11
placeholder-clean positive few-shot candidates after staging = 13
rows promoted to positive few-shot in live Hope runtime = 0
```

V120 changes the source priority, not the runtime-promotion boundary. The V120
review does not reopen positive few-shot promotion, and this V3 refresh still
promotes zero V120 rows into live Hope runtime use.

Any refreshed V120 few-shot-clean counts must come from the KB ingest package,
not from this docs-only alignment thread.

## KB Handoff Note

KB should ingest against V120 as the primary 23-field source shape and keep
V108 only as the comparison baseline.

During KB ingest:

- preserve the V120 field names and meanings exactly at the source layer
- carry forward the V3 mapping decisions for `sample_type`,
  `scene_performance_core`, `continuity_negative_core`, `reference_bundle`, and
  `prompt_body`
- recompute exact V120 placeholder, few-shot-clean, and sequence-surface counts
  from actual package outputs instead of assuming V108 counts still apply
- record unresolved gaps rather than silently normalizing them
- do not invent `reference_control_core`, asset bindings, media paths, or live
  runtime readiness claims

## Required Future Gates

Further work still requires explicit KB or main-control gates:

1. KB ingest / manifest / validation / snapshot completion for the V120 package.
2. KB review of canonical external object names for `external_reference_handles`.
3. KB or vNext schema decision for the V120/V108 23-field source format.
4. V3/V4 schema decision for splitting or preserving fused fields.
5. Hope validator planning for V120 evidence and failure codes.
6. Hope repair planning for V120 failure-to-repair behavior.
7. Export/debug metadata planning, if V120 evidence needs surfaced outside KB.
8. Desktop readonly provenance/readiness planning.
9. Intake/Qwen retrieval boundary planning.
10. Seedance adapter gate only after structured Hope output and validator
    readiness are separately accepted.

## Explicit Prohibitions

This docs-only refresh does not authorize:

- changing Hope product code
- changing the frozen v0.1 contract or workbook
- changing Rust DTOs, validators, exporters, IPC, desktop, or intake
- editing `E:\codex\hope-kb`
- connecting Qwen or Seedance
- importing V120 rows into current v0.2 sample rows
- promoting V120 rows into runtime few-shot
- inventing `reference_control_core`
- inventing reference image paths, URLs, asset IDs, character appearance, or
  scene designs

## Verification Commands

```powershell
git -C E:\codex\hope status --short --branch
git -C E:\codex\hope log -1 --oneline --decorate
```
