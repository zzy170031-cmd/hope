# Seedance2 V108 V3 Docs Alignment Refresh 2026-04-23

## Route Metadata

| Item | Value |
| --- | --- |
| repo | `E:\codex\hope` |
| proposal branch | `codex/v3-field-overlay-proposal` |
| V3 starting anchor | `69c645c` (`docs: promote golden sample overlay to freeze candidate`) |
| KB source anchor reviewed | `E:\codex\hope-kb @ 6f210f0` (`docs: align Seedance2 V108 fields with V3 proposal`) |
| Hope main-control anchor reviewed | `142225e3e6d029c2ed83e382c292d3a43cc9ddb0` |
| handoff type | docs-only V3 proposal / freeze-candidate alignment refresh |
| implementation scope | no product code, no exporter, no IPC, no Rust DTO, no validator, no desktop, no intake, no workbook, no Qwen/Seedance integration, no hope-kb mutation |

## Canonical Source Format Decision

The local Seedance2.0 V108 XLSX / DOCX table format accepted by main control on
2026-04-23 is now the latest canonical source format for this V3 docs refresh.

Historical V3 documents at `69c645c` remain useful as proposal and
freeze-candidate reference material, but they are no longer the current field
format truth when they conflict with V108.

This refresh does not import V108 rows into runtime few-shot, does not change
the v0.1 workbook, does not update KB, and does not authorize implementation.

## Source Materials Read

| Source | Location |
| --- | --- |
| Hope main-control acceptance | `origin/codex/contracts-freeze:docs/seedance2-v108-kb-sample-update-review-2026-04-23.md` |
| KB field mapping review | `E:\codex\hope-kb\docs\seedance2-v108-field-mapping-review-2026-04-23.md` |
| KB V3 alignment report | `E:\codex\hope-kb\docs\seedance2-v108-v3-alignment-report-2026-04-23.md` |
| KB sample update readiness | `E:\codex\hope-kb\docs\seedance2-v108-sample-update-readiness-2026-04-23.md` |
| KB normalized staging | `E:\codex\hope-kb\docs\normalized-staging\seedance2-v108-fused-golden-sample.normalized.json` |
| Existing V3 field overlay | `docs/contracts/export-overlays/v3-field-overlay-proposal-2026-04-22.md` |
| Existing golden sample proposal | `docs/contracts/export-overlays/golden-sample-v0.2-schema-overlay-proposal-2026-04-22.md` |
| Existing freeze candidate | `docs/contracts/export-overlays/golden-sample-v0.2-contract-freeze-candidate-2026-04-22.md` |

## V108 Source Package Facts

| Item | Value |
| --- | --- |
| XLSX source hash | `2c7f7a0b37f5d7a6f90f18c0080a78fa568fcb4528b1b4e6fbdbfb959cfe9e34` |
| DOCX source hash | `a2335102154ab88cb5c01517b2b20bbb15efdbdd25f0d32eeb9715d42e8d7ed3` |
| normalized staging hash | `a9ace9098ae60ce942ee5101b39329dc5b9ee70ab2d73e20d058d6db5938c167` |
| V108 field count | `23` |
| total source rows | `115` |
| official rows | `108` |
| reserve rows | `7` |
| official `single_shot` rows | `72` |
| official `sequence_shot` rows | `36` |
| official source few-shot `Yes` | `97` |
| official source few-shot `No` | `11` |
| rows promoted to positive few-shot by this gate | `0` |

Current v0.2 KB sample records remain unchanged:

```text
golden_sample_library = 40
golden_sample_field_coverage_rules = 5
golden_sample_failure_mapping = 40
golden_sample_repair_mapping = 40
```

## V108 Field Set

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

| V108 field | historical V3 / old golden sample field | current decision | status | notes |
| --- | --- | --- | --- | --- |
| `shot_id` | `sample_id`, `shot_id` | Preserve V108 source value; future import may map to row/sample IDs. | rename | Do not rewrite existing 40-row IDs in this docs refresh. |
| `library_status` | none exact | New source governance field. | new | `reserve` rows are excluded from positive few-shot. |
| `reserve_reason` | `repair_hint`, `negative_boundary_marker` analogues | Reserve-only / conditional metadata. | new | Blank official cells are not missing content. |
| `sample_type` | old golden sample `sample_type=positive/negative/repair_before/repair_after`; V3 `structure_mode` | V108 means `single_shot` or `sequence_shot`. | rename + v3_alignment_gap | The old sample-category meaning must not be reused for V108. |
| `sequence_id` | `shot_beats_link`, `sequence_no`, `shot_beats` | Future sequence grouping surface. | future_model_fill | Blank single-shot cells are expected. |
| `shot_order` | `beat_order`, `sequence_no`, `shot_beats` | Future sequence ordering surface. | future_model_fill | Blank single-shot cells are expected. |
| `sample_title` | `sample_title` | Preserve as title / retrieval label. | exact | Directly compatible. |
| `style_cluster` | `style_profile_id`, `retrieval_tags` | New retrieval taxonomy candidate. | new | Internal only; not imitation language. |
| `scene_category` | `kb_scene_type`, `shot_function_code`, `retrieval_tags` | Scene taxonomy input. | rename | Needs future taxonomy review. |
| `scene_tag` | `scene_tag`, `retrieval_tags` | Preserve as source tag. | exact | Current KB v0.2 has analogous scene tags. |
| `quality_grade` | `tier` | Preserve source grade. | rename | Replaces old `tier` naming for V108 source rows. |
| `usable_for_fewshot` | `usable_for_fewshot`, `fewshot.eligible` | Preserve as source gate only. | exact | Still blocked by reserve, negative, placeholder, and reference gates. |
| `technical_profile` | `target_clip_duration_sec`, `shot_type`, `transition_note`, `lens_feel`, model capability fields | Split into technical subfields only after a future schema gate. | split | Contains future-fill placeholders in 41 rows. |
| `scene_performance_core` | `visual_scene_core` + `motion_performance_core` | V108 canonical source fuses these old axes. | merge + v3_alignment_gap | Do not hard-split without a future accepted rule. |
| `camera_directing_core` | `camera_directing_core` | Preserve name as source content field. | exact | Current KB v0.2 stores evidence rows, not top-level prompt content. |
| `audio_directing_core` | `audio_directing_core` | Preserve name as source content field. | exact | Current KB v0.2 stores evidence rows, not top-level prompt content. |
| `continuity_negative_core` | `continuity_lock_core`, `blocking_failure_codes`, `negative_boundary_marker` | V108 combines continuity and negative constraints. | split + v3_alignment_gap | Future schema must decide the split. |
| `reference_bundle` | `reference_control_core`, `asset_registry`, `image_ref_assets`, `video_ref_assets`, `audio_ref_assets` | Normalize only to `external_reference_handles`. | rename + v3_alignment_gap | Not paths, URLs, asset IDs, or completed reference-control coverage. |
| `ip_abstraction_note` | `rights_or_ip_risk_flag`, `copyright_risk_flag`, `negative_boundary_marker` | Compliance / abstraction evidence. | new | Not product prompt expansion. |
| `covered_points` | `covered_points`, `covered_core_fields`, `covered_atomic_fields` | Future validator evidence. | exact | No implementation in this gate. |
| `missed_points` | `missed_points`, `missing_items` | Future validator / repair evidence. | exact | No implementation in this gate. |
| `teaching_note` | `teaching_note`, `repair_hint` | Future repair rationale / reviewer trace. | exact | Not runtime behavior now. |
| `prompt_body` | `sample_text`; compiled Seedance prompt fields | Preserve as source prompt body / candidate text only. | rename + v3_alignment_gap | Not adapter output truth. Placeholder-bearing rows are blocked. |

## Deprecated Or Downscoped Historical Assumptions

| Historical item | V108-aligned disposition |
| --- | --- |
| old golden sample `sample_type` as positive/negative/repair category | Do not use for V108 `sample_type`; if sample role is needed later, name it separately in a future gate. |
| direct `reference_control_core` backfill | Still not covered. V108 `reference_bundle` only produces external handle candidates. |
| `image_ref_assets`, `video_ref_assets`, `audio_ref_assets` as source fields | Adapter/export bindings only after a future gate; not V108 source truth. |
| `compiled_prompt_seedance` / `compiled_payload_seedance` | Derived adapter output only; V108 `prompt_body` is source candidate text. |
| old separate `visual_scene_core` and `motion_performance_core` as V108 source columns | Superseded by V108 `scene_performance_core` for this source package. |
| old standalone `continuity_lock_core` as the only continuity source | Superseded by V108 `continuity_negative_core`, which needs a future split rule. |
| importing V108 rows into current v0.2 sample rows | Closed. V108 remains raw source and normalized staging evidence. |
| positive few-shot promotion from V108 | Closed. This gate promotes zero V108 rows. |

## v3_alignment_gap List

The following gaps are intentionally preserved and must not be hard-merged:

1. `sample_type`: V108 uses shot-structure semantics, while old V3 golden-sample
   material used sample-category semantics.
2. `scene_performance_core`: V108 merges old `visual_scene_core` and
   `motion_performance_core`.
3. `continuity_negative_core`: V108 combines continuity locks, prohibitions,
   empty-word blacklist, and risk evidence.
4. `reference_bundle`: V108 maps only toward `external_reference_handles`, not
   asset binding, concrete media paths, or completed `reference_control_core`.
5. `prompt_body`: V108 source prompt text is not compiled Seedance adapter output.
6. Current v0.2 stores one main `classification.core` per sample, while V108
   carries multiple fused core fields per row.
7. Current v0.2 has no top-level fields for `external_reference_handles`,
   `technical_profile`, `scene_performance_core`, `prompt_body_candidate`, or
   sequence grouping.

## future_model_fill_surface List

| Surface | Count | Required handling |
| --- | ---: | --- |
| blank `reserve_reason` on official rows | 108 | Conditional metadata; not missing sample content. |
| blank `sequence_id` on single-shot rows | 79 | Future sequence surface; do not count as sequence records. |
| blank `shot_order` on single-shot rows | 79 | Future sequence surface; do not count as sequence records. |
| placeholder-like `reference_bundle` | 102 | Preserve as gap; do not invent paths, URLs, asset IDs, or handles. |
| placeholder-like `prompt_body` | 102 | Block `prompt_body_candidate`; do not guess-fill. |
| placeholder-like `technical_profile` | 41 | Future model/schema fill only. |
| placeholder-like `scene_performance_core` | 41 | Future model/schema fill only. |
| placeholder-like `camera_directing_core` | 42 | Future model/schema fill only. |
| placeholder-like `audio_directing_core` | 67 | Future model/schema fill only. |
| placeholder-like `reserve_reason` | 1 | Preserve as source gap; do not promote. |

No empty V108 worksheet was observed in the reviewed package. If a later source
package introduces empty worksheets, empty tables, or additional blank template
areas, they should be recorded under the same `future_model_fill_surface`
handling rule until main control opens a fill/import gate.

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

Disallowed in this V3 docs refresh:

```text
image paths
URLs
asset IDs
real media bindings
invented character appearance
invented scene design
completed reference_control_core coverage
```

Prompt wording may describe action, expression, tone, camera, light, rhythm, and
continuity relationships. Character and scene references should use clear names
directly after a future gate confirms canonical external object names.

## Positive Few-Shot Boundary

V108 source facts:

```text
official few-shot Yes rows = 97
placeholder-clean positive few-shot candidates after staging = 13
rows ready for positive few-shot promotion in this package = 0
```

No V108 row is promoted into runtime positive few-shot by this refresh. Reserve,
negative, unusable, placeholder-bearing, and alignment-gap rows remain only
validator, repair, provenance, and gap evidence until future gates.

## Required Future Gates

Further work still requires explicit KB or main-control gates:

1. KB review of canonical external object names for `external_reference_handles`.
2. KB or vNext schema decision for V108's 23-field source format.
3. V3/V4 schema decision for splitting or preserving fused fields.
4. Hope validator planning for V108 evidence and failure codes.
5. Hope repair planning for V108 failure-to-repair behavior.
6. Export/debug metadata planning, if V108 evidence needs surfaced outside KB.
7. Desktop readonly provenance/readiness planning.
8. Intake/Qwen retrieval boundary planning.
9. Seedance adapter gate only after structured Hope output and validator
   readiness are separately accepted.

## Explicit Prohibitions

This docs-only refresh does not authorize:

- changing Hope product code
- changing the frozen v0.1 contract or workbook
- changing Rust DTOs, validators, exporters, IPC, desktop, or intake
- editing `E:\codex\hope-kb`
- connecting Qwen or Seedance
- importing V108 rows into current v0.2 sample rows
- promoting V108 rows into runtime few-shot
- inventing `reference_control_core`
- inventing reference image paths, URLs, asset IDs, character appearance, or
  scene designs

## Verification Commands

```powershell
git -C E:\codex\hope status --short --branch
git -C E:\codex\hope log -1 --oneline --decorate
```
