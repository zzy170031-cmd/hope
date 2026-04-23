# Seedance2 V108 Validator Evidence Contract Candidate 2026-04-23

## Route

This is a docs-only validator evidence contract candidate for the Hope main
control thread.

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor before this memo:
  `6aad29a` (`docs: dispatch Seedance2 V108 validator evidence contract`)
- accepted dry-run review anchor:
  `cd56773e5f31cd989634e66b310468a4f25dff24`
  (`docs: accept Seedance2 V108 import dry-run`)
- KB dry-run report anchor:
  `b9660be7f44904ec56ba78462632db4251e88de3`
  (`docs: add Seedance2 V108 import dry-run report`)
- V3 archived alignment anchor:
  `bc745a4d8b48ddc259c8b1c94ab907f8946797c0`
  (`docs: align V3 proposal with Seedance2 V108`)
- package type: docs-only V108 validator evidence planning contract candidate

This candidate does not implement validators. It does not modify Rust DTOs,
validator code, repair logic, exporter behavior, workbook shape, IPC, desktop,
intake, V3 branch files, `hope-kb`, Qwen, or Seedance.

## Reviewed Inputs

Hope-side inputs:

- `docs/seedance2-v108-validator-evidence-contract-dispatch-2026-04-23.md`
- `docs/seedance2-v108-import-dry-run-review-2026-04-23.md`
- `docs/seedance2-v108-import-contract-review-2026-04-23.md`
- `docs/seedance2-v108-import-contract-candidate-2026-04-23.md`
- `docs/seedance2-v108-vnext-schema-contract-review-2026-04-23.md`
- `docs/seedance2-v108-vnext-schema-contract-candidate-2026-04-23.md`
- `docs/seedance2-v108-vnext-schema-decision-2026-04-23.md`
- `docs/seedance2-v108-v3-docs-alignment-review-2026-04-23.md`

KB-side inputs, read only from `E:\codex\hope-kb`:

- `docs/seedance2-v108-import-dry-run-report-2026-04-23.md`
- `docs/seedance2-v108-v3-alignment-report-2026-04-23.md`

## Candidate Decision

`SEEDANCE2_V108_VALIDATOR_EVIDENCE_CONTRACT_CANDIDATE_FROZEN_DOCS_ONLY`

Hope may use this memo as the planning candidate for how V108 raw source
evidence could be reasoned about by a future validator layer.

This candidate is not a validator implementation authorization. It is not an
import authorization, not a repair authorization, not a positive few-shot
promotion, not an exporter change, and not a `reference_control_core` creation.

## Required Assertions

The accepted dry-run assertions remain unchanged:

```text
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
```

Additional closed-scope assertions remain unchanged:

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

All `115` V108 rows remain raw source evidence for future dry-run reasoning
only. They do not enter product import, runtime positive few-shot, validator
runtime evidence, repair runtime behavior, product-ready external reference
handles, exporter output, workbook sheets, or `reference_control_core`.

## Planning Contract Objects

Future planning should preserve these object names:

- `V108ValidatorEvidenceContract`
- `V108ValidatorEvidenceSource`
- `V108ValidatorEvidenceFamily`
- `V108ValidatorEvidenceBlocker`
- `V108ValidatorEvidenceReadiness`
- `V108ValidatorEvidenceNoRuntimeImportAssertion`
- `V108ValidatorEvidenceFutureGateOrder`

These names are planning names only. They are not Rust types, database tables,
IPC payloads, workbook sheets, exporter fields, runtime enums, validator
failure codes, severity enums, thresholds, or blocking logic.

## Evidence Source Boundary

`V108ValidatorEvidenceSource` may reason about the following canonical source
fields only as raw evidence:

- `covered_points`
- `missed_points`
- `teaching_note`
- `ip_abstraction_note`
- `continuity_negative_core`

Supporting context may be read from these V108 source fields:

- `scene_performance_core`
- `camera_directing_core`
- `audio_directing_core`
- `technical_profile`
- `prompt_body`
- `reference_bundle`
- `quality_grade`
- `usable_for_fewshot`
- `library_status`
- `sample_type`
- `sequence_id`
- `shot_order`

Supporting context does not make a row product-ready. It only helps a future
validator evidence review explain why a source row is covered, missing, blocked,
or still raw.

## Future Evidence Use

`covered_points` is future positive coverage evidence. It may later explain
which required directing, scene, camera, audio, continuity, or prompt-source
points appear to be covered by the raw row. It must not become a validator pass
by itself.

`missed_points` is future gap evidence. It may later map to missing coverage,
repair planning, or debug metadata. It must not create repair behavior or
runtime blocking semantics without a later implementation gate.

`teaching_note` is future reviewer rationale evidence. It may explain why a row
is useful, incomplete, risky, or educational for validator planning. It is not
runtime generation content and must not be inserted into prompts as final
storyboard text.

`ip_abstraction_note` is future compliance and abstraction evidence. It may
support checks that source examples have been abstracted away from concrete IP
or reference dependency. If unresolved, it remains a blocker for product import
and positive few-shot promotion.

`continuity_negative_core` is future continuity and negative-boundary evidence.
It may later support continuity lock checks, prohibited drift checks,
empty-word/risk checks, and negative-sample planning. It must remain fused at
source level until a later gate accepts any split. It must not be promoted as a
positive few-shot example.

## Evidence Families

Future validator planning may group evidence into these families:

- `v108_coverage_evidence`
- `v108_gap_evidence`
- `v108_teaching_rationale_evidence`
- `v108_ip_abstraction_evidence`
- `v108_continuity_negative_evidence`
- `v108_prompt_candidate_evidence`
- `v108_reference_unresolved_evidence`
- `v108_placeholder_blocked_evidence`
- `v108_v3_alignment_gap_evidence`
- `v108_schema_missing_evidence`
- `v108_promotion_gate_blocked_evidence`

These families are planning signal groups only. They do not create failure-code
enums, severity levels, thresholds, repair mappings, validator rows, product
rows, or exporter/debug metadata fields in this gate.

## Blocker Semantics

The accepted dry-run blockers continue to control readiness:

- `validator_evidence_not_defined`: blocks validator runtime use until a later
  accepted implementation gate defines actual validator evidence DTOs and
  behavior.
- `placeholder_present`: blocks product import and positive few-shot whenever a
  row still contains placeholder-like source content.
- `prompt_body_blocked_by_placeholder`: blocks prompt-candidate runtime use
  when `prompt_body` contains placeholder content.
- `reference_handle_unresolved`: blocks product-ready external reference
  handles because no canonical object registry or accepted handles exist.
- `v3_alignment_gap`: blocks reuse of historical V3 meanings when V108 source
  semantics conflict with old proposal language.
- `schema_field_missing`: blocks product import while the vNext schema/import
  implementation surface remains absent.
- `promotion_gate_not_accepted`: blocks product import, positive few-shot, and
  runtime use until main control accepts a later promotion package.
- `reserve_row`: blocks positive few-shot promotion and keeps the row as
  reserve governance evidence.

Blockers do not delete source rows. They preserve rows as raw evidence while
preventing product import, runtime positive few-shot, product-ready reference
handles, validator runtime behavior, repair runtime behavior, and exporter use.

## Reference And Registry Boundary

`reference_bundle` remains raw reference evidence only.

Current accepted state:

```text
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
```

This candidate does not create `reference_control_core`. It does not invent
reference images, URLs, asset IDs, character appearances, scene designs,
canonical object names, or registry data.

Future reference use requires a separate canonical object registry gate and a
separate main-control acceptance. Until then, every V108 row with reference
needs remains under `reference_handle_unresolved`.

## Product Import And Positive Few-Shot Boundary

V108 source facts remain:

```text
total_source_rows = 115
official_rows = 108
reserve_rows = 7
placeholder_like_rows = 102
source_official_usable_for_fewshot_yes = 97
placeholder_clean_prompt_candidates_after_staging = 13
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
```

`covered_points`, `missed_points`, `teaching_note`, `ip_abstraction_note`, and
`continuity_negative_core` may help a future review understand evidence quality,
but none of them overrides placeholder blockers, unresolved references, V3
alignment gaps, missing schema fields, reserve status, or an unopened promotion
gate.

## Fixture, Repair, Exporter, And Debug Metadata Impact

Fixture impact is planning-only. Future fixtures may cite row identity, source
field presence, evidence families, and blocker families, but this gate does not
create fixtures, snapshot rows, or product validator data.

Repair impact is planning-only. `missed_points`, `teaching_note`, and
`continuity_negative_core` may later inform repair candidate reasoning, but this
gate does not implement repair recommendations, repair priority, repair scope,
validator hints, or prompt template selection.

Exporter impact is closed. The v0.1 workbook remains unchanged. No V108 row,
evidence family, blocker, or debug field enters workbook sheets, exporter
payloads, Seedance overlay output, or Excel behavior in this gate.

Debug metadata impact is planning-only. A later gate may decide whether
validator debug output can include source row IDs, evidence family names, and
blocker names. This gate does not add debug metadata fields.

## Readiness Matrix

| planning surface | current readiness | reason |
| --- | --- | --- |
| raw source evidence preservation | ready for future dry-run | All `115` rows are preserved as raw evidence. |
| validator evidence planning | candidate only | This memo defines planning families but no validator behavior. |
| product import | blocked | `rows_ready_for_product_import = 0`. |
| positive few-shot | blocked | `rows_ready_for_positive_fewshot = 0`. |
| external reference handles | blocked | `product_ready_external_reference_handles = 0`. |
| `reference_control_core` | closed | `reference_control_core_coverage = 0`; no creation authorized. |
| exporter/workbook use | closed | No exporter or workbook gate is open. |
| Qwen/Seedance use | closed | No model call, prompt assembly, or adapter gate is open. |

## Future Gate Order

Future work must proceed in this order unless main control opens a separate
docs-only gate:

1. Accept or reject this validator evidence contract candidate.
2. If accepted, run a KB-only canonical object registry gate before any
   product-facing reference handles are considered.
3. Run a V108 validator evidence implementation scope gate naming exact files
   and forbidden files before any DTO, validator, fixture, or debug metadata
   implementation.
4. Run a repair evidence contract gate before any repair behavior, repair
   recommendation, repair priority, or repair mapping is created.
5. Run a V108 import implementation scope gate only after import, validator,
   reference, and repair boundaries are accepted.
6. Run any exporter/workbook/Seedance overlay gate separately; v0.1 workbook
   and current exporter behavior remain frozen until then.
7. Run any Qwen, Seedance, desktop, or intake gate separately; this memo does
   not wake those threads.

No future gate may skip directly from raw V108 rows to product import,
positive few-shot, runtime prompts, exporter output, or `reference_control_core`.

## Explicitly Closed

This candidate does not authorize:

- Rust DTO changes
- validator implementation
- failure-code enum, severity enum, threshold, blocking, or repair behavior
- exporter behavior changes
- workbook shape changes
- IPC changes
- desktop changes
- intake changes
- `hope-kb` edits
- V3 branch edits
- Qwen calls
- Seedance calls
- runtime prompt assembly
- V108 row import into seed or product structures
- positive few-shot promotion
- `reference_control_core` creation
- invented external references, character appearance, scene design, or registry
  data

## Candidate Completion

This candidate is complete when committed and pushed to
`codex/contracts-freeze`.

Recommended next label:

```text
Hope主线-Seedance2V108ValidatorEvidence【候选已完成·待接收】
```

Validation for this package is docs-only. No code tests are required unless a
later gate opens implementation.
