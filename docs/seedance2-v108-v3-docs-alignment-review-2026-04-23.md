# Seedance2 V108 V3 Docs Alignment Review 2026-04-23

## Route

- repo: `E:\codex\hope`
- review branch: `codex/contracts-freeze`
- reviewed V3 branch: `codex/v3-field-overlay-proposal`
- reviewed V3 starting anchor: `69c645c`
- reviewed V3 commit: `bc745a4d8b48ddc259c8b1c94ab907f8946797c0`
  (`docs: align V3 proposal with Seedance2 V108`)
- required KB anchor: `E:\codex\hope-kb @ 6f210f0`
- required main-control anchor:
  `142225e3e6d029c2ed83e382c292d3a43cc9ddb0`
- review type: main-control acceptance of docs-only V3 alignment refresh

This review accepts or rejects the V3 documentation refresh only. It does not
open Hope product implementation.

## Reviewed V3 Result

Changed files in `bc745a4d8b48ddc259c8b1c94ab907f8946797c0`:

- `docs/contracts/export-overlays/seedance2-v108-v3-docs-alignment-refresh-2026-04-23.md`
- `docs/contracts/export-overlays/v3-field-overlay-proposal-2026-04-22.md`
- `docs/contracts/export-overlays/golden-sample-v0.2-schema-overlay-proposal-2026-04-22.md`
- `docs/contracts/export-overlays/golden-sample-v0.2-contract-freeze-candidate-2026-04-22.md`

Scope check:

- Only documentation files changed.
- No Rust DTO, validator, repair, exporter, workbook, IPC, desktop, intake,
  Qwen, Seedance, or `hope-kb` file changed.
- The V3 branch reports clean and aligned with
  `origin/codex/v3-field-overlay-proposal` at the reviewed commit.

## Acceptance Decision

`SEEDANCE2_V108_V3_DOCS_ALIGNMENT_REFRESH_ACCEPTED`

Main control accepts V3 commit
`bc745a4d8b48ddc259c8b1c94ab907f8946797c0` as the docs-only alignment refresh
for Seedance2 V108.

Accepted points:

- V108 XLSX / DOCX is correctly recorded as the latest canonical source format
  for this docs gate.
- Old V3 proposal and freeze-candidate documents are correctly downgraded to
  historical proposal / freeze-candidate reference where they conflict with
  V108.
- V3 records the 23 V108 fields, 115 total rows, 108 official rows, 7 reserve
  rows, 97 official source `usable_for_fewshot=Yes` rows, and zero runtime
  positive few-shot promotions.
- The `sample_type` semantic conflict is preserved as `v3_alignment_gap`
  instead of being silently reused.
- `scene_performance_core` is marked as the V108 fused source surface for the
  old `visual_scene_core` plus `motion_performance_core` axes.
- `continuity_negative_core` is marked as a fused continuity plus negative
  constraint surface that needs a future split or retention decision.
- `reference_bundle` is constrained to `external_reference_handles`, not image
  paths, URLs, asset IDs, or completed `reference_control_core`.
- `prompt_body` is constrained to source prompt body / candidate text, not
  compiled Seedance adapter output.
- Placeholder, blank, and pending-fill surfaces are preserved as
  `future_model_fill_surface`; they are not guessed, filled, or promoted.
- The positive few-shot boundary remains closed even where KB normalized
  staging records `positive_fewshot_candidate_after_placeholder_gate = 13`.

## No Rejection Findings

No blocking rejection finding was found in this V3 refresh.

The refresh is intentionally conservative: it records field mapping, deprecated
historical assumptions, alignment gaps, future-fill surfaces, and future gates
without changing runtime behavior.

## Remaining Closed Gates

This acceptance does not authorize:

- Hope product implementation
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
- `hope-kb` edits
- merging `hope-kb` into `hope`
- importing V108 rows into current v0.2 sample rows
- promoting V108 rows into runtime positive few-shot
- inventing `reference_control_core`
- inventing concrete reference assets, URLs, asset IDs, character appearance,
  or scene design

## Remaining Gaps

Known gaps after this acceptance:

- Canonical external object names for `external_reference_handles` still need a
  KB / main-control review gate.
- V108's 23-field source format still needs a vNext schema decision before any
  import into product-facing structures.
- The fused fields `scene_performance_core` and `continuity_negative_core` need
  a future split / preserve decision before DTO or validator planning.
- Placeholder surfaces remain unresolved and are not usable as positive
  few-shot material.
- Hope's generation field-rule contract still reflects the earlier five-core
  V3 planning language and must not be treated as updated implementation scope
  until a later main-control gate opens that work.

## Next Control State

The five-thread state after this review is:

- Hope main: `codex/contracts-freeze`, docs-only V108 KB and V3 alignment
  accepted; implementation still closed.
- KB: `codex/contracts-freeze @ 6f210f0`, V108 raw source and normalized staging
  accepted; waiting for next KB-only gate if external names or schema evidence
  are requested.
- V3: `codex/v3-field-overlay-proposal @ bc745a4`, docs-only V108 alignment
  accepted; archive / standby.
- Desktop: waiting.
- Intake: waiting.
- Qwen and Seedance: closed.

Next eligible gates must be explicitly opened by main control. The likely next
docs-only choices are:

1. KB external reference handle canonical-name review.
2. KB / V3 vNext schema decision for V108's 23-field source shape.
3. Hope validator evidence planning, only after the source/schema boundary is
   accepted.

Do not proceed directly to implementation from this acceptance.

Recommended next label:

```text
Hope主线-Seedance2V108V3Docs【已接收·待下轮Gate】
```

关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。
