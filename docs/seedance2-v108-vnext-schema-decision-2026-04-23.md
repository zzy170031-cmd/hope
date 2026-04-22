# Seedance2 V108 vNext Schema Decision 2026-04-23

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- package type: docs-only vNext schema decision for Seedance2 V108 source shape
- Hope anchor before this package:
  `a14d1e3546af3b7fbf45b453d9bf1bbbe14e6d00`
  (`docs: accept Seedance2 V108 KB reference boundaries`)
- KB source anchor:
  `6e5a150c39e3e1c7abba1da08fb1d13bf8e199ee`
  (`docs: pin Seedance2 V108 canonical headers`)
- V3 archived anchor:
  `bc745a4d8b48ddc259c8b1c94ab907f8946797c0`
  (`docs: align V3 proposal with Seedance2 V108`)

This decision is docs-only. It does not implement a schema, import rows, change
Rust DTOs, change validators, change exporters, change workbooks, change IPC,
change desktop/intake, call Qwen, call Seedance, or merge `hope-kb` into
`hope`.

## Decision

`SEEDANCE2_V108_VNEXT_SCHEMA_DECISION_ACCEPTED_DOCS_ONLY`

Main control accepts the V108 23-header worksheet shape as the canonical source
surface for future vNext schema planning. The source schema must be designed
from the V108 headers, not from old V3 field names or from free-text sections
inside cells.

The current v0.1 workbook and current KB v0.2 sample rows remain unchanged.
This decision defines the future planning boundary only.

## Canonical Source Headers

The future source-row schema must preserve these 23 headers as the source
surface:

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

Cells below these headers are values, evidence, examples, gaps, or
`future_model_fill_surface`. They do not create new schema fields by
themselves.

## Proposed vNext Source Boundary

Future vNext work should use a source-first boundary with at least these
logical layers:

1. `v108_source_row`
   - Preserves the 23 raw V108 fields and source provenance.
   - Does not rewrite source values.
   - Does not compile Seedance payloads.
2. `v108_review_flags`
   - Records placeholder, blank conditional surfaces, v3 alignment gaps,
     unresolved reference handles, unusable rows, reserve rows, and promotion
     blockers.
3. `v108_sequence_view`
   - Derives sequence grouping only from `sample_type`, `sequence_id`, and
     `shot_order`.
   - Does not treat blank single-shot sequence cells as missing records.
4. `v108_prompt_candidate_view`
   - Reads `prompt_body` only as source prompt text / candidate text.
   - Does not treat `prompt_body` as compiled Seedance adapter output.
5. `v108_reference_handle_view`
   - May exist only as a derived, gated view.
   - Product-ready `external_reference_handles` remain `0` until a canonical
     object registry is accepted.

These are planning names only. They are not implementation names, Rust types, or
database tables.

## Field Disposition Table

| V108 header | vNext disposition | decision |
| --- | --- | --- |
| `shot_id` | source identity | Preserve as immutable V108 row / shot ID. Future imports may map to internal IDs, but must retain source ID. |
| `library_status` | source governance | Preserve `official/reserve`; reserve rows stay out of positive few-shot. |
| `reserve_reason` | conditional governance | Preserve for reserve rows. Blank official cells are conditional blanks, not missing content. |
| `sample_type` | structure enum | Preserve as `single_shot/sequence_shot`. Do not reuse old V3 positive/negative/repair semantics. |
| `sequence_id` | sequence key | Preserve as optional sequence grouping. Blank single-shot cells are expected. |
| `shot_order` | sequence order | Preserve as optional order inside sequence. Blank single-shot cells are expected. |
| `sample_title` | source label | Preserve as retrieval/review label, not a generated prompt field. |
| `style_cluster` | retrieval taxonomy candidate | Preserve as source taxonomy evidence. Do not emit as imitation language. |
| `scene_category` | retrieval taxonomy candidate | Preserve as source scene family. Future taxonomy review required before runtime use. |
| `scene_tag` | source tag | Preserve as source scene tag / retrieval evidence. |
| `quality_grade` | quality metadata | Preserve source grade. Do not equate it to few-shot eligibility by itself. |
| `usable_for_fewshot` | source eligibility flag | Preserve as a source gate only. Positive promotion still requires other gates. |
| `technical_profile` | structured candidate with raw text | Preserve raw text; future parser may derive duration, shot size, transition, lens, frame rate, and movement only after a schema/import gate. |
| `scene_performance_core` | fused core source field | Preserve fused source field. Do not hard-split into old `visual_scene_core` and `motion_performance_core` before a future rule accepts that split. |
| `camera_directing_core` | core source field | Preserve as source long text. Future validator may inspect camera purpose, composition, movement, axis, and adjacent-shot relation. |
| `audio_directing_core` | core source field | Preserve as source long text. Future validator may inspect sound layers, sync points, silence, music/SFX, and audio-picture relation. |
| `continuity_negative_core` | fused continuity / negative source field | Preserve fused source field. Future views may derive continuity locks, prohibitions, empty-word blacklist, and risk notes, but no hard split is accepted yet. |
| `reference_bundle` | raw reference evidence only | Preserve as raw source value. It is not final `external_reference_handles`, not asset binding, and not `reference_control_core`. |
| `ip_abstraction_note` | compliance evidence | Preserve as IP / abstraction / compliance evidence. Do not expand into prompt prose by default. |
| `covered_points` | validator evidence candidate | Preserve as future validator coverage evidence. No current validator implementation is authorized. |
| `missed_points` | validator / repair evidence candidate | Preserve as future gap, failure, and repair evidence. |
| `teaching_note` | reviewer / repair rationale evidence | Preserve as explanatory evidence. It is not runtime behavior by itself. |
| `prompt_body` | source prompt candidate | Preserve as source prompt body / candidate text. It is not compiled Seedance output and does not create extra fields from internal headings. |

## Fused Field Decision

The fused V108 fields stay fused at source level:

- `scene_performance_core`
- `continuity_negative_core`

Future implementations may define derived read views, but source truth remains
the V108 header value until a later gate accepts a split rule.

Prohibited shortcuts:

- splitting `scene_performance_core` back into old V3 fields without an accepted
  rule
- treating `continuity_negative_core` as only continuity lock evidence
- treating `continuity_negative_core` as only negative prompt text
- silently dropping negative constraints, empty-word blacklist, or risk notes

## Reference Handle Decision

`reference_bundle` remains raw source evidence only.

Current accepted counts:

```text
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
runtime_positive_fewshot_promotion = 0
```

Future product-facing handles may use only canonical object names from a
separate registry:

```text
character
scene
prop
style
continuity_object
```

No future schema may promote media labels, media counts, URLs, file paths, asset
IDs, or generic labels into product-ready handles.

## Few-Shot Decision

V108 source facts remain:

```text
total_rows = 115
official_rows = 108
reserve_rows = 7
official_usable_for_fewshot_yes = 97
placeholder_clean_positive_candidates_after_staging = 13
rows_ready_for_positive_fewshot_promotion = 0
```

The vNext schema may record source eligibility and review flags, but it must not
promote rows to runtime positive few-shot until later gates accept:

- official row status
- usable source flag
- no blocking placeholder / future-fill surface
- no reserve or negative blocker
- canonical external object names if references are needed
- validator / repair evidence boundaries
- main-control approval

## Prompt Body Decision

`prompt_body` is source prompt text / candidate text.

It must not be treated as:

- compiled Seedance prompt
- compiled Seedance payload
- exporter output
- a source of extra schema headers
- a way to bypass `reference_bundle`
- a place to invent character appearance or scene design

Internal sections inside `prompt_body`, including any reference-material section,
remain text under the `prompt_body` header.

## Closed Gates

This decision does not authorize:

- row import from V108 into current KB v0.2 rows
- runtime few-shot promotion
- Rust DTO changes
- validator implementation
- repair implementation
- writer/orchestrator implementation
- exporter or workbook changes
- IPC changes
- desktop or intake changes
- Qwen or Seedance calls
- V3 branch edits
- `hope-kb` edits
- `reference_control_core` invention

## Next Control State

The next eligible docs-only gate is one of:

1. `KB canonical object registry package`, if the operator provides story,
   character, scene, prop, style, or continuity-object names.
2. `V108 vNext schema contract candidate`, if main control wants the schema
   decision converted into a more formal contract candidate for later
   validator/import planning.
3. `Hope validator evidence planning`, only after main control accepts a schema
   contract candidate and still without implementation.

Recommended next label:

```text
Hope主线-Seedance2V108SchemaDecision【已完成·待下轮Gate】
```

关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。
