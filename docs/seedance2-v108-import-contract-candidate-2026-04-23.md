# Seedance2 V108 Import Contract Candidate 2026-04-23

## Route

This is a docs-only import contract candidate for the Hope main control thread.

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor before this memo:
  `198b31aed84f22908e40e55f7ff56673b88c049b`
  (`docs: accept Seedance2 V108 schema contract`)
- accepted schema contract:
  `docs/seedance2-v108-vnext-schema-contract-review-2026-04-23.md`
- KB source anchor:
  `6e5a150c39e3e1c7abba1da08fb1d13bf8e199ee`
  (`docs: pin Seedance2 V108 canonical headers`)
- V3 archived anchor:
  `bc745a4d8b48ddc259c8b1c94ab907f8946797c0`
  (`docs: align V3 proposal with Seedance2 V108`)
- package type: docs-only V108 import contract candidate

This candidate does not import data. It does not modify Rust DTOs, validators,
exporter behavior, workbook shape, IPC, desktop, intake, Qwen, Seedance, V3
proposal branches, or `hope-kb`.

## Reviewed Inputs

Hope-side inputs:

- `docs/seedance2-v108-vnext-schema-contract-review-2026-04-23.md`
- `docs/seedance2-v108-vnext-schema-contract-candidate-2026-04-23.md`
- `docs/seedance2-v108-vnext-schema-decision-2026-04-23.md`
- `docs/seedance2-v108-kb-reference-handles-and-headers-review-2026-04-23.md`

KB-side inputs, read only from `E:\codex\hope-kb`:

- `docs/normalized-staging/seedance2-v108-fused-golden-sample.normalized.json`
- `docs/seedance2-v108-field-mapping-review-2026-04-23.md`
- `docs/seedance2-v108-external-reference-handles-canonical-name-review-2026-04-23.md`
- raw V108 XLSX / DOCX source package under `golden-samples/seedance2-v108/`

## Candidate Decision

`SEEDANCE2_V108_IMPORT_CONTRACT_CANDIDATE_FROZEN_DOCS_ONLY`

Hope may use this candidate as the planning contract for a future V108 import
dry-run and later import implementation.

This candidate is not an import authorization. It must be accepted by main
control before any implementation package can create tables, DTOs, validators,
or persisted imported rows.

## Source Facts

Accepted source facts from KB normalized staging:

```text
source_package_id = seedance2-v108-fused-golden-sample
schema_version = hope-kb-seedance2-v108-normalized-staging-v0.1
primary_sheet = V108融合字段库
header_row = 1
field_count = 23
data_rows = 115
official_rows = 108
reserve_rows = 7
official_usable_for_fewshot_yes = 97
official_usable_for_fewshot_no = 11
placeholder_rows = 102
placeholder_clean_positive_candidates_after_staging = 13
ready_for_positive_fewshot = 0
reference_handle_normalization_needed = 115
product_ready_external_reference_handles = 0
```

This candidate must preserve these facts as import preflight evidence.

## Import Contract Objects

Future import work should preserve these planning object names:

- `V108ImportContract`
- `V108ImportBatch`
- `V108ImportSourceArtifact`
- `V108ImportPreflight`
- `V108ImportRowCandidate`
- `V108ImportBlocker`
- `V108ImportDerivedViewPlan`
- `V108ImportPromotionState`
- `V108ImportDryRunReport`
- `V108ImportAcceptanceGate`

These are planning object names only. They are not implementation names, Rust
types, database tables, or IPC payloads.

## V108ImportBatch

`V108ImportBatch` represents one immutable import review package.

Required planning fields:

```text
batch_id
source_package_id
source_artifact_hashes
normalized_staging_hash
source_repo
source_commit
source_sheet_name
source_header_set
source_row_count
official_row_count
reserve_row_count
preflight_status
dry_run_status
accepted_by_main_control
```

Current status for this package:

```text
preflight_status = review_ready
dry_run_status = not_started
accepted_by_main_control = false
```

## V108ImportPreflight

Future import preflight must check:

1. raw XLSX / DOCX artifacts exist and hashes match KB source register
2. primary sheet is `V108融合字段库`
3. header row contains exactly the accepted 23 V108 headers
4. row count is `115`
5. official/reserve counts are `108/7`
6. normalized staging hash matches the accepted KB artifact
7. `hope-kb` and Hope source anchors are recorded
8. no old V3 field names are used as source truth
9. no row is promoted to runtime positive few-shot
10. no `reference_control_core` is created

Failure of any preflight item blocks import.

## V108ImportRowCandidate

Every source row may become an import row candidate only as raw source evidence.

Required candidate fields:

```text
source_row_number
shot_id
library_status
sample_type
sequence_id
shot_order
source_values
review_flags
blockers
derived_view_plan
provenance
```

The `source_values` map must preserve all 23 source headers without rewriting or
expanding cell content.

## V108ImportBlocker

Candidate blockers:

```text
reserve_row
placeholder_present
blank_conditional_surface
sequence_identity_missing_for_sequence_shot
reference_handle_unresolved
prompt_body_blocked_by_placeholder
v3_alignment_gap
schema_field_missing
validator_evidence_not_defined
promotion_gate_not_accepted
source_hash_mismatch
source_row_count_mismatch
```

Blocking semantics:

- Blockers do not delete rows.
- Blockers decide which derived views are unavailable.
- Blocked rows may still remain validator / repair / provenance evidence after
  a later accepted evidence contract.
- Blocked rows do not enter runtime positive few-shot.

## Import Eligibility Matrix

| source row class | raw source import | prompt candidate view | reference handle view | sequence view | positive few-shot |
| --- | --- | --- | --- | --- | --- |
| official row with placeholders | allowed as raw evidence only | blocked if `prompt_body` has placeholder | blocked until registry | allowed only for source structure evidence | blocked |
| official placeholder-clean row | allowed as raw evidence only | candidate only, not runtime | blocked until registry | allowed only for source structure evidence | blocked |
| reserve row | allowed as reserve evidence only | blocked from positive use | blocked until registry | allowed only for source structure evidence | blocked |
| sequence_shot row | allowed as raw evidence only | candidate only if prompt not blocked | blocked until registry | requires sequence validation | blocked |
| row with unresolved reference | allowed as raw evidence only | blocked if prompt depends on reference | blocked | allowed only for source structure evidence | blocked |

Current accepted import outcome:

```text
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
```

## Derived View Plan

Future import dry-run may create a plan for these derived views:

```text
V108SequenceView
V108PromptCandidateView
V108ReferenceHandleView
V108ValidatorEvidenceView
```

But current view availability is constrained:

- `V108SequenceView`: planning only; sequence validation not accepted yet.
- `V108PromptCandidateView`: 13 source candidates may be preserved as candidate
  text, but no runtime use is accepted.
- `V108ReferenceHandleView`: blocked; product-ready handles are `0`.
- `V108ValidatorEvidenceView`: planning only; validator evidence contract not
  accepted yet.

## Source-To-Contract Mapping

| V108 header | import role |
| --- | --- |
| `shot_id` | source row identity |
| `library_status` | official/reserve governance |
| `reserve_reason` | reserve evidence / conditional blank for official rows |
| `sample_type` | single/sequence structure |
| `sequence_id` | sequence grouping candidate |
| `shot_order` | sequence ordering candidate |
| `sample_title` | review/retrieval label |
| `style_cluster` | taxonomy evidence candidate |
| `scene_category` | taxonomy evidence candidate |
| `scene_tag` | scene tag evidence |
| `quality_grade` | quality metadata |
| `usable_for_fewshot` | source eligibility flag only |
| `technical_profile` | future technical parser source |
| `scene_performance_core` | fused core source evidence |
| `camera_directing_core` | camera core source evidence |
| `audio_directing_core` | audio core source evidence |
| `continuity_negative_core` | fused continuity/negative source evidence |
| `reference_bundle` | raw reference evidence only; blocked for product handles |
| `ip_abstraction_note` | compliance / abstraction evidence |
| `covered_points` | future validator evidence |
| `missed_points` | future validator / repair evidence |
| `teaching_note` | reviewer / repair rationale evidence |
| `prompt_body` | source prompt candidate text only |

## V108ImportDryRunReport

A later dry-run report must include:

```text
batch_id
source_commit
source_hash_check
header_check
row_count_check
official_reserve_count_check
placeholder_summary
blank_conditional_surface_summary
reference_handle_summary
sequence_summary
prompt_candidate_summary
validator_evidence_summary
promotion_summary
blocked_row_summary
changed_files
no_runtime_import_assertion
```

The dry-run report must explicitly state that no product rows were imported
unless main control opens a separate implementation gate.

## V108ImportAcceptanceGate

Before implementation, main control must accept:

1. this import contract candidate
2. an import dry-run report
3. a validator evidence contract candidate
4. a canonical object registry if reference handles are required
5. a final implementation scope memo naming exact files and forbidden files

Without all required acceptances, import implementation remains closed.

## Explicitly Closed

This candidate does not authorize:

- writing imported rows
- changing KB seed files
- changing Hope product code
- changing Rust DTOs
- implementing validators
- implementing repair
- implementing writer/orchestrator logic
- changing exporter behavior
- changing workbook shape
- changing IPC
- changing desktop or intake
- connecting Qwen or Seedance
- editing the V3 branch
- promoting runtime positive few-shot
- creating `reference_control_core`
- inventing reference images, URLs, asset IDs, character appearance, or scene
  design

## Candidate Completion

This candidate is complete when committed to `codex/contracts-freeze`. The next
main-control step should be an acceptance / rejection review.

Recommended next label:

```text
Hope主线-Seedance2V108ImportContract【候选已完成·待接收】
```

关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。
