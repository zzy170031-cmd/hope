# Seedance2 V108 KB Reference Handles And Headers Review 2026-04-23

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- review type: main-control acceptance of KB-only V108 reference-handle and
  canonical-header follow-up
- reviewed KB repo: `E:\codex\hope-kb`
- reviewed KB branch: `codex/contracts-freeze`
- reviewed KB commits:
  - `d9decf145c66b0f15fe0a890fd5a3395d8a6ec12`
    (`docs: review Seedance2 V108 external reference handles`)
  - `6e5a150c39e3e1c7abba1da08fb1d13bf8e199ee`
    (`docs: pin Seedance2 V108 canonical headers`)
- previous Hope main-control anchor:
  `f72a75542426cbefbc3b3ea3f4b66ded5c0770e4`
- previous V108 KB acceptance:
  `142225e3e6d029c2ed83e382c292d3a43cc9ddb0`
- previous V108 V3 acceptance:
  `f72a75542426cbefbc3b3ea3f4b66ded5c0770e4`

This review is docs-only. It does not authorize Hope product implementation,
schema import, V3 updates, Qwen calls, Seedance calls, desktop work, intake
work, or merging `hope-kb` into `hope`.

## Reviewed KB Result

Changed KB files across the reviewed commits:

- `docs/seedance2-v108-external-reference-handles-canonical-name-review-2026-04-23.md`
- `docs/live-progress.md`

No KB seed rows, snapshot source rows, Hope product code, V3 branch files,
desktop, intake, Qwen, Seedance, Rust DTO, validator, exporter, workbook, or IPC
files were changed by these commits.

KB validation reported:

- seed bundle validation: passed
- `hope-kb` worktree: clean and up to date with
  `origin/codex/contracts-freeze`

## Acceptance Decision

`SEEDANCE2_V108_KB_REFERENCE_HANDLES_AND_CANONICAL_HEADERS_ACCEPTED`

Main control accepts KB commit
`6e5a150c39e3e1c7abba1da08fb1d13bf8e199ee` as the latest KB-only boundary
record for V108 canonical headers and external reference handle readiness.

Accepted points:

- The V108 XLSX / DOCX table package remains the canonical source format for
  this gate.
- The V108 worksheet header row is accepted as the canonical field surface.
- All downstream mapping, schema, handle, validator, repair, and prompt-boundary
  decisions must derive from the V108 header strings, not from old V3 field
  names or free-text blocks inside cell values.
- The cells below the headers are source values, evidence, examples, gaps, or
  `future_model_fill_surface` content. They do not create additional fields.
- Text inside `prompt_body`, including any reference-material block, remains
  content under the `prompt_body` header. It does not create a separate
  reference field.
- `reference_bundle` is the only V108 header currently reviewed for external
  reference naming.
- Media labels and media-style descriptions inside `reference_bundle` are
  values under that header. They are not final external reference handles.
- Current product-ready `external_reference_handles` count remains `0`.
- The 37 extracted candidate stems are retained as non-canonical evidence only.
- `reference_control_core` remains absent and must not be invented from V108.
- Runtime positive few-shot promotion remains `0`.

## Canonical V108 Headers

Main control accepts the following 23 V108 headers as the field source for this
gate:

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

Any future exact / rename / split / merge / new / deprecated / gap decision must
cite one or more of these headers.

## External Reference Boundary

Accepted future product-facing handle categories remain:

```text
character
scene
prop
style
continuity_object
```

But this acceptance does not create actual canonical names. Future names must
come from story setup, user-provided character and scene settings, or an
external reference system / registry.

Directly rejected as final `external_reference_handles`:

- image paths
- URLs
- asset IDs
- real media bindings
- image / video / audio media labels
- media counts
- generic labels such as protagonist face or character standee
- invented character appearance
- invented scene design

If a required object name is missing, later work must emit
`unresolved_reference_handle` or preserve `future_model_fill_surface`.

## Remaining Closed Gates

This acceptance does not authorize:

- Hope product implementation
- vNext schema import
- Rust DTO changes
- validator implementation
- repair implementation
- writer or orchestrator implementation
- exporter behavior changes
- workbook changes
- IPC changes
- desktop changes
- intake changes
- Qwen calls or prompt assembly implementation
- Seedance calls or adapter implementation
- V3 branch edits
- `hope-kb` edits from the Hope main repo
- merging `hope-kb` into `hope`
- importing V108 rows into current v0.2 sample rows
- promoting V108 rows into runtime positive few-shot
- inventing `reference_control_core`

## Next Control State

The five-thread state after this review is:

- Hope main: `codex/contracts-freeze`, V108 KB sample update, V3 docs alignment,
  and KB reference/header follow-up accepted; implementation still closed.
- KB: `codex/contracts-freeze @ 6e5a150`, V108 23 headers pinned and
  `reference_bundle` kept as non-canonical source evidence; standby.
- V3: `codex/v3-field-overlay-proposal @ bc745a4`, V108 docs alignment
  accepted; archive / standby.
- Desktop: waiting.
- Intake: waiting.
- Qwen and Seedance: closed.

Next eligible gate depends on available source material:

1. If the operator provides project/story character, scene, prop, style, or
   continuity object names, open a KB-only canonical object registry package.
2. If no object registry is available, open a docs-only V108 vNext schema
   decision package. That package must preserve this review's rule that
   product-ready `external_reference_handles = 0` until canonical names exist.

Do not proceed directly to implementation from this acceptance.

Recommended next label:

```text
Hope主线-Seedance2V108KBHeaders【已接收·待下轮Gate】
```

关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。
