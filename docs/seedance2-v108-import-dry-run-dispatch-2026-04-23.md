# Seedance2 V108 Import Dry-Run Dispatch 2026-04-23

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- dispatch type: main-control dispatch to KB-only V108 import dry-run report
- Hope anchor before dispatch:
  `c2d57c67b01389320904f5e46c46a2e2f9648a43`
  (`docs: accept Seedance2 V108 import contract`)
- target KB repo: `E:\codex\hope-kb`
- target KB branch: `codex/contracts-freeze`
- target KB anchor:
  `6e5a150c39e3e1c7abba1da08fb1d13bf8e199ee`
  (`docs: pin Seedance2 V108 canonical headers`)
- V3 archived anchor:
  `bc745a4d8b48ddc259c8b1c94ab907f8946797c0`
  (`docs: align V3 proposal with Seedance2 V108`)

This dispatch is docs-only. It sends concrete work to the KB thread. Hope main
does not perform the dry-run itself.

## Thread Assignment

Target thread:

```text
Hope-KB-Seedance2V108ImportDryRun【待执行】
```

Target repo / branch:

```text
E:\codex\hope-kb / codex/contracts-freeze
```

Starting anchor:

```text
6e5a150c39e3e1c7abba1da08fb1d13bf8e199ee
```

## Required Read List

The KB thread must read:

- `E:\codex\hope\docs\seedance2-v108-import-contract-review-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-import-contract-candidate-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-vnext-schema-contract-review-2026-04-23.md`
- `E:\codex\hope-kb\docs\seedance2-v108-external-reference-handles-canonical-name-review-2026-04-23.md`
- `E:\codex\hope-kb\docs\seedance2-v108-field-mapping-review-2026-04-23.md`
- `E:\codex\hope-kb\docs\normalized-staging\seedance2-v108-fused-golden-sample.normalized.json`
- raw V108 XLSX / DOCX source package under
  `E:\codex\hope-kb\golden-samples\seedance2-v108\`

## Task

Produce a KB-only V108 import dry-run report.

Required output document:

```text
E:\codex\hope-kb\docs\seedance2-v108-import-dry-run-report-2026-04-23.md
```

The report must include:

- `batch_id`
- source commit and source artifact hashes
- source hash check
- header check against the 23 canonical V108 headers
- row count check
- official/reserve count check
- placeholder summary
- blank conditional surface summary
- reference handle summary
- sequence summary
- prompt candidate summary
- validator evidence summary
- promotion summary
- blocked row summary
- changed files
- `no_runtime_import_assertion`

## Required Assertions

The KB thread must preserve these assertions:

```text
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
```

The dry-run report may classify blockers, evidence candidates, and future
derived-view availability. It must not write product rows.

## Required Validation

The KB thread should run:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\validate-seed-bundle.ps1
```

If it rebuilds snapshots only for verification, snapshot outputs must not be
treated as product import unless an existing KB workflow already tracks them.

## Boundaries

Allowed:

- KB docs-only dry-run report
- read-only analysis of normalized staging JSON
- read-only analysis of raw V108 XLSX / DOCX
- update `docs/live-progress.md`
- commit and push KB-only documentation changes

Forbidden:

- modifying `E:\codex\hope`
- modifying V3 branch files
- modifying desktop or intake
- connecting Qwen or Seedance
- changing Rust DTOs, validators, repair, exporter, workbook, or IPC
- importing V108 rows into product structures
- changing KB seed rows as an import
- promoting positive few-shot rows
- creating `reference_control_core`
- inventing reference images, URLs, asset IDs, character appearance, or scene
  design

## Completion Report

After completion, the KB thread must return:

- final commit hash
- changed files
- validation result
- source/hash/header/row-count summary
- blocked row summary
- dry-run conclusion
- git status

Then main control will perform a separate acceptance / rejection review.

## Next State

Hope main remains:

```text
Hope主线-Seedance2V108ImportContract【已接收·调度KB DryRun】
```

KB should use:

```text
Hope-KB-Seedance2V108ImportDryRun【执行中】
```

关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。
