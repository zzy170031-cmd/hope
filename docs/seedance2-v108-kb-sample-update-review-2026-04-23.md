# Seedance2 V108 KB Sample Update Review 2026-04-23

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- review type: main-control acceptance of KB-only Seedance2 V108 sample update gate
- reviewed KB repo: `E:\codex\hope-kb`
- reviewed KB branch: `codex/contracts-freeze`
- reviewed KB commit: `6f210f0` (`docs: align Seedance2 V108 fields with V3 proposal`)
- raw artifact receipt commit: `e056e7d` (`add Seedance2.0 V108 fused golden sample library`)
- previous KB readiness anchor: `818c098` (`Add golden sample v0.2 snapshot import readiness`)
- V3 historical reference anchor: `origin/codex/v3-field-overlay-proposal @ 69c645c`

This review is docs-only. It does not authorize Hope product implementation,
V3 implementation, Qwen calls, Seedance calls, desktop work, intake work, or
merging `hope-kb` into `hope`.

## Reviewed KB Artifacts

Accepted reviewed files and artifacts:

- `golden-samples/seedance2-v108/Seedance2.0黄金样本库V108_单表融合字段版.xlsx`
- `golden-samples/seedance2-v108/Seedance2.0黄金样本库V108_单表融合字段版说明.docx`
- `docs/normalized-staging/seedance2-v108-fused-golden-sample.normalized.json`
- `docs/seedance2-v108-field-mapping-review-2026-04-23.md`
- `docs/seedance2-v108-v3-alignment-report-2026-04-23.md`
- `docs/seedance2-v108-sample-update-readiness-2026-04-23.md`
- `docs/live-progress.md`
- `seed/v0.2/manifest.json`
- `seed/v0.2/source_register.json`

Raw artifact integrity recorded by KB:

- XLSX SHA-256:
  `2c7f7a0b37f5d7a6f90f18c0080a78fa568fcb4528b1b4e6fbdbfb959cfe9e34`
- DOCX SHA-256:
  `a2335102154ab88cb5c01517b2b20bbb15efdbdd25f0d32eeb9715d42e8d7ed3`
- normalized staging SHA-256:
  `a9ace9098ae60ce942ee5101b39329dc5b9ee70ab2d73e20d058d6db5938c167`

## Main-Control Verification

Local verification confirms:

- `hope-kb` worktree is clean and up to date with
  `origin/codex/contracts-freeze`.
- `hope` worktree was clean before this review file was added.
- V108 source sheet: `V108融合字段库`.
- V108 source field count: `23`.
- V108 total source rows: `115`.
- V108 official rows: `108`.
- V108 reserve rows: `7`.
- normalized staging record count: `115`.
- normalized staging status counts: `official=108`, `reserve=7`.
- v0.1 no-regression validation: passed.
- v0.1 snapshot build: passed.
- v0.2 snapshot build: passed.
- v0.2 SQLite `quick_check`: `ok`.
- v0.2 row counts:
  - `golden_sample_library=40`
  - `golden_sample_field_coverage_rule=5`
  - `golden_sample_failure_mapping=40`
  - `golden_sample_repair_mapping=40`
  - `golden_sample_source=9`
  - `golden_sample_provenance=2`
- v0.2 snapshot hash:
  `bundle-sha256:3a813e8417cc23f9dbbb2115c40be5a811ece21e76f2d743f05df9ab038c9c0c`

## Acceptance Decision

`SEEDANCE2_V108_KB_SAMPLE_UPDATE_READINESS_ACCEPTED`

Main control accepts KB commit `6f210f0` as the completed KB-only V108 sample
update readiness package.

Accepted points:

- The local V108 XLSX and DOCX are now the canonical source format for this
  gate.
- The old V3 proposal material remains historical reference only until a
  separate V3 docs-only refresh is dispatched and accepted.
- V108 field conflicts against the old V3 proposal are correctly marked as
  `v3_alignment_gap` instead of being silently mapped.
- Blank or placeholder surfaces are preserved as `future_model_fill_surface`.
  They are not guessed, promoted, or used as validator evidence.
- The `reference_bundle` concept is accepted only as
  `external_reference_handles`: named external character, scene, prop, style,
  or continuity handles that downstream tools may already bind to concrete
  assets outside Hope.
- No image path, URL, asset ID, real media asset, or completed
  `reference_control_core` was invented.
- V108 source rows are staged as raw source and normalized staging evidence
  only. They are not imported into current v0.2 sample rows.
- Current v0.2 sample row counts remain unchanged:
  `library=40`, `coverage=5`, `failure=40`, `repair=40`.
- Positive few-shot promotion remains closed. No V108 row is promoted into the
  runtime positive few-shot set by this gate.

## Remaining Closed Gates

This acceptance does not authorize:

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
- `hope-kb` edits from the Hope main repo
- merging `hope-kb` into `hope`
- importing V108 rows into current v0.2 sample rows
- using V108 negative, reserve, unusable, or placeholder rows as positive
  few-shot examples
- inventing concrete reference assets or `reference_control_core` coverage

## Remaining Gaps

Known gaps that remain outside this acceptance:

- Current v0.2 does not yet have top-level
  `external_reference_handles`, `technical_profile`, `scene_performance_core`,
  `prompt_body_candidate`, or sequence grouping fields.
- 102 V108 rows contain `待补` placeholder-like content in at least one field.
- Canonical external object names still need content review before any future
  sample import or prompt-use gate.
- V3 docs still reflect the pre-V108 proposal shape and need a separate
  docs-only refresh against KB commit `6f210f0`.

## Next Control State

V3 may now be dispatched for a docs-only alignment refresh, with these bounds:

- target repo: `E:\codex\hope`
- target branch: `codex/v3-field-overlay-proposal`
- starting anchor: `69c645c`
- required KB anchor: `E:\codex\hope-kb @ 6f210f0`
- required control anchor: this review document
- allowed work: update V3 proposal/freeze-candidate documentation so old V3
  fields are aligned to the V108 canonical source format and KB mapping
  reports.
- required output: mapping changes, deprecated fields, renamed fields, new
  gaps, unresolved `future_model_fill_surface` entries, and final docs-only
  commit.

V3 must not update Hope product code, Rust DTOs, validators, exporters,
workbooks, IPC, desktop, intake, Qwen, Seedance, or `hope-kb`.

Recommended next label:

```text
Hope主线-Seedance2V108KBGate【已接收·待V3文档刷新】
```

关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。
