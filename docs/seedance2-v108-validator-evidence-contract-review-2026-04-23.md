# Seedance2 V108 Validator Evidence Contract Review 2026-04-23

## Review Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- reviewed candidate commit:
  `641cdb5` (`docs: add Seedance2 V108 validator evidence contract`)
- candidate memo:
  `docs/seedance2-v108-validator-evidence-contract-candidate-2026-04-23.md`
- review scope: main-control docs-only acceptance / rejection

This review does not implement validators, import V108 rows, change Rust DTOs,
change exporter or workbook behavior, create reference handles, modify
`hope-kb`, modify V3, wake desktop or intake, or call Qwen / Seedance.

## Decision

`SEEDANCE2_V108_VALIDATOR_EVIDENCE_CONTRACT_CANDIDATE_ACCEPTED`

Main control accepts the candidate as the current planning contract for V108
validator evidence readiness.

The accepted contract is planning-only. It may be used to name future evidence
families, blocker families, and gate order. It may not be treated as runtime
validator behavior, import authorization, repair authorization, exporter
authorization, product prompt assembly, positive few-shot promotion, or
`reference_control_core` creation.

## Acceptance Findings

The candidate satisfies the dispatch requirements:

- preserves the accepted dry-run assertions
- defines planning objects for V108 validator evidence
- scopes canonical raw evidence fields:
  `covered_points`, `missed_points`, `teaching_note`,
  `ip_abstraction_note`, and `continuity_negative_core`
- describes future evidence use without turning source fields into runtime
  validator behavior
- preserves blocker families for placeholder, unresolved reference, V3
  alignment gap, missing schema field, unopened promotion gate, and undefined
  validator evidence
- keeps all V108 rows outside product import, positive few-shot, exporter,
  workbook, prompt assembly, and runtime validator behavior
- keeps `reference_control_core` closed
- gives an explicit future gate order

## Required Assertions Remain Frozen

```text
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
```

Additional unchanged boundaries:

```text
v108_rows_imported_into_seed = 0
v108_rows_imported_into_product_structure = 0
positive_fewshot_promotions = 0
qwen_calls = 0
seedance_calls = 0
v3_branch_edits = 0
hope_product_code_changes = 0
desktop_or_intake_changes = 0
rust_dto_validator_exporter_workbook_ipc_changes = 0
```

## Accepted Planning Objects

The following names are accepted as planning names only:

- `V108ValidatorEvidenceContract`
- `V108ValidatorEvidenceSource`
- `V108ValidatorEvidenceFamily`
- `V108ValidatorEvidenceBlocker`
- `V108ValidatorEvidenceReadiness`
- `V108ValidatorEvidenceNoRuntimeImportAssertion`
- `V108ValidatorEvidenceFutureGateOrder`

They are not accepted as Rust types, database tables, IPC payloads, workbook
sheets, exporter fields, runtime enums, failure codes, severity levels,
thresholds, or blocking logic.

## Still Blocked

The accepted contract does not resolve these blockers:

- `placeholder_present`
- `prompt_body_blocked_by_placeholder`
- `reference_handle_unresolved`
- `v3_alignment_gap`
- `schema_field_missing`
- `promotion_gate_not_accepted`
- `validator_evidence_not_defined`
- `reserve_row`

These blockers continue to prevent product import, runtime positive few-shot,
product-ready external reference handles, runtime validator behavior, repair
runtime behavior, exporter use, and workbook changes.

## Next Gate Order

Main control accepts the candidate and keeps the next gates ordered as follows:

1. KB-only canonical object registry readiness gate before any product-facing
   reference handles are considered.
2. V108 shot-language / retrieval-selection planning gate, if main control
   chooses to turn `style_cluster`, `scene_category`, and
   `camera_directing_core` into bounded retrieval planning rather than runtime
   generation.
3. V108 validator evidence implementation scope gate naming exact files and
   forbidden files before any DTO, validator, fixture, or debug metadata
   implementation.
4. Repair evidence contract gate before any repair behavior or repair mapping
   is created.
5. V108 import implementation scope gate only after import, validator,
   reference, and repair boundaries are accepted.
6. Exporter/workbook/Seedance overlay gate separately; current exporter and
   workbook behavior remain frozen until opened by main control.
7. Qwen, Seedance, desktop, and intake gates separately; this review does not
   wake those threads.

No gate may skip from raw V108 rows directly to product import, positive
few-shot, runtime prompts, exporter output, or `reference_control_core`.

## Validation

This was a docs-only review. No code tests were required or run.

## Recommended Thread Label

```text
Hope主线-Seedance2V108ValidatorEvidence【Contract已接收·等待下一Gate】
```
