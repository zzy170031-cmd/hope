# Seedance2 V108 vNext Schema Contract Candidate 2026-04-23

## Route

This is a docs-only contract candidate for the Hope main control thread.

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor before this memo:
  `ba9db6ce32a932b5734e6d79ba04fb8cad05fa4a`
  (`docs: decide Seedance2 V108 vNext schema boundary`)
- KB source anchor:
  `6e5a150c39e3e1c7abba1da08fb1d13bf8e199ee`
  (`docs: pin Seedance2 V108 canonical headers`)
- V3 archived anchor:
  `bc745a4d8b48ddc259c8b1c94ab907f8946797c0`
  (`docs: align V3 proposal with Seedance2 V108`)
- package type: docs-only vNext schema contract candidate
- route: `RC_READY` plus controlled v0.2 / vNext readiness planning

This candidate does not open product implementation. It does not modify Rust
DTOs, validators, exporter behavior, workbook shape, IPC, desktop, intake,
Qwen, Seedance, V3 proposal branches, or `hope-kb`.

## Reviewed Inputs

Hope-side inputs:

- `docs/seedance2-v108-vnext-schema-decision-2026-04-23.md`
- `docs/seedance2-v108-kb-reference-handles-and-headers-review-2026-04-23.md`
- `docs/seedance2-v108-v3-docs-alignment-review-2026-04-23.md`
- `docs/seedance2-v108-kb-sample-update-review-2026-04-23.md`
- `docs/generation-field-rule-contract-candidate-2026-04-22.md`

KB-side inputs, read only from `E:\codex\hope-kb`:

- `docs/seedance2-v108-external-reference-handles-canonical-name-review-2026-04-23.md`
- `docs/seedance2-v108-field-mapping-review-2026-04-23.md`
- `docs/normalized-staging/seedance2-v108-fused-golden-sample.normalized.json`
- raw V108 XLSX / DOCX source package under `golden-samples/seedance2-v108/`

## Candidate Decision

`SEEDANCE2_V108_VNEXT_SCHEMA_CONTRACT_CANDIDATE_FROZEN_DOCS_ONLY`

Hope may use this candidate as the planning contract for how the V108 source
package can later become import evidence, validator evidence, retrieval
metadata, repair evidence, and prompt-candidate evidence.

This candidate is not an implementation authorization. It must be accepted by
main control before any import contract, DTO/schema design, validator planning,
repair planning, Qwen retrieval planning, exporter/debug planning, or Seedance
adapter planning can rely on it.

## Contract Objects

Future implementation-facing work should preserve these contract object names:

- `V108SourceSchemaContract`
- `V108SourceRow`
- `V108HeaderSet`
- `V108ReviewFlags`
- `V108SequenceView`
- `V108PromptCandidateView`
- `V108ReferenceHandleView`
- `V108ValidatorEvidenceView`
- `V108PromotionGate`
- `V108ImportGateOrder`

These names are planning contract names only. Rust types, database tables,
validators, and IPC payloads remain closed until later explicit gates.

## Canonical Header Set

`V108HeaderSet` contains exactly these 23 source headers:

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

Contract rules:

- The header strings above are the canonical source field surface.
- Cell content below the headers is value/evidence/gap content only.
- Free-text sections inside `prompt_body` do not create additional fields.
- Old V3 field names may be cited only as historical mapping references.
- Any future rename, split, merge, derived view, deprecation, or gap must cite
  one or more of these 23 headers.

## V108SourceRow

`V108SourceRow` preserves one raw source row from the V108 source package.

Required planning fields:

```text
source_package_id
source_artifact_hash
source_sheet_name
source_row_number
source_headers_version
source_values
source_status
source_provenance
```

`source_values` must preserve the 23 V108 source header values without rewriting
or expanding them.

`source_status` may record:

```text
official
reserve
placeholder_present
blank_conditional_surface
v3_alignment_gap
future_model_fill_surface
unresolved_reference_handle
blocked_from_positive_fewshot
candidate_for_validator_evidence
candidate_for_repair_evidence
```

This source row contract must not compile Seedance prompt or payload data.

## V108ReviewFlags

`V108ReviewFlags` records why a row can or cannot enter later views.

Candidate flags:

```text
is_official
is_reserve
is_single_shot
is_sequence_shot
has_sequence_identity
has_placeholder_text
has_blank_conditional_surface
has_v3_alignment_gap
has_reference_bundle_source
needs_reference_handle_registry
has_prompt_body_candidate
prompt_body_blocked_by_placeholder
fewshot_source_yes
fewshot_source_no
ready_for_positive_fewshot
validator_or_repair_evidence_candidate
```

Current accepted default:

```text
ready_for_positive_fewshot = false
```

No V108 row is currently ready for runtime positive few-shot promotion.

## V108SequenceView

`V108SequenceView` may be derived only from:

```text
sample_type
sequence_id
shot_order
shot_id
```

Rules:

- `sample_type=single_shot` is a complete structure state, not a missing
  sequence.
- Blank `sequence_id` and `shot_order` on single-shot rows are expected
  conditional blanks.
- `sequence_shot` rows require future sequence validation before import.
- Sequence grouping must not reorder source rows without preserving provenance.

## V108PromptCandidateView

`V108PromptCandidateView` reads only:

```text
prompt_body
usable_for_fewshot
library_status
placeholder flags
reference flags
quality_grade
covered_points
missed_points
teaching_note
```

Rules:

- `prompt_body` is source prompt text / candidate text.
- `prompt_body` is not compiled Seedance output.
- Internal headings inside `prompt_body` remain text under `prompt_body`.
- `prompt_body` may not bypass `reference_bundle`.
- Rows with placeholder-bearing prompt bodies remain blocked.

## V108ReferenceHandleView

`V108ReferenceHandleView` is a future derived view, not a current product view.

Current accepted values:

```text
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
```

Future product-facing handles may use only canonical names from a separate
object registry:

```text
character
scene
prop
style
continuity_object
```

Disallowed direct sources for final handles:

- image labels
- video labels
- audio labels
- media counts
- file paths
- URLs
- asset IDs
- real media bindings
- generic labels such as protagonist face or character standee
- invented character appearance
- invented scene design

If the registry is missing, the row must retain
`unresolved_reference_handle`.

## V108ValidatorEvidenceView

`V108ValidatorEvidenceView` may later read:

```text
covered_points
missed_points
technical_profile
scene_performance_core
camera_directing_core
audio_directing_core
continuity_negative_core
ip_abstraction_note
teaching_note
quality_grade
usable_for_fewshot
```

Candidate evidence families:

```text
v108_header_presence
v108_placeholder_surface
v108_future_model_fill_surface
v108_sequence_consistency
v108_reference_handle_unresolved
v108_prompt_candidate_blocked
v108_fused_core_gap
v108_continuity_negative_gap
v108_ip_abstraction_evidence
v108_fewshot_promotion_blocked
```

These are planning families only. No validator implementation, severity,
threshold, or failure-code enum is authorized here.

## Fused Core Rule

The following source fields remain fused:

```text
scene_performance_core
continuity_negative_core
```

Derived read views may later inspect subparts, but the source contract must
preserve the fused value.

Closed until a future rule:

- hard-splitting `scene_performance_core` into old V3 visual/motion fields
- hard-splitting `continuity_negative_core` into continuity, negative prompt,
  blacklist, and risk fields
- dropping any part of the fused source text because it does not fit old V3
  names

## V108PromotionGate

No V108 row may enter runtime positive few-shot until all of these gates pass:

1. source row is `official`
2. source `usable_for_fewshot` is `Yes`
3. row is not reserve, negative, or otherwise blocked
4. row has no blocking placeholder or unresolved future-fill surface
5. required reference handles have accepted canonical object names
6. prompt body is preserved as source candidate and not compiled output
7. validator / repair evidence boundary is accepted
8. main control accepts the promotion package

Current accepted count:

```text
runtime_positive_fewshot_promoted_rows = 0
```

## V108ImportGateOrder

Future work must follow this order:

1. Accept this contract candidate in main control.
2. If available, receive a KB-only canonical object registry package.
3. Produce a formal import contract from this candidate.
4. Define validator evidence and repair evidence contracts.
5. Only then consider DTO/schema implementation.
6. Only after DTO/schema acceptance consider retrieval or prompt assembly.
7. Only after structured output and validation gates consider export or
   Seedance adapter planning.

The order prevents V108 rows from skipping KB and main-control review to enter
runtime prompts or product structures.

## Explicitly Closed

This candidate does not authorize:

- importing V108 rows into current KB v0.2 rows
- changing the v0.1 workbook
- changing Rust DTOs
- implementing validators
- implementing repair
- implementing writer/orchestrator logic
- changing exporter behavior
- changing IPC
- changing desktop or intake
- connecting Qwen or Seedance
- editing the V3 branch
- editing `hope-kb`
- promoting positive few-shot rows
- inventing `reference_control_core`
- inventing reference images, URLs, asset IDs, character appearance, or scene
  design

## Candidate Completion

This candidate is complete when committed to `codex/contracts-freeze`. The next
main-control step should be an acceptance / rejection review.

Recommended next label:

```text
Hope主线-Seedance2V108SchemaContract【候选已完成·待接收】
```

关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。
