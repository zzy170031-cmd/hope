# V3 Golden Sample Overlay Review 2026-04-22

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor before this memo: `fad0c2b` (`docs: accept KB golden sample staging`)
- reviewed proposal branch: `origin/codex/v3-field-overlay-proposal`
- reviewed proposal commit: `7a08cd6` (`docs: add golden sample v0.2 overlay proposal`)
- route: `RC_READY` plus scope freeze
- review type: docs-only proposal acceptance

This memo does not reopen implementation scope. It records the main control
review of the V3 field thread's golden-sample v0.2 overlay proposal.

## Reviewed Files

The reviewed V3 proposal commit adds only docs/source materials:

- `docs/contracts/export-overlays/golden-sample-v0.2-schema-overlay-proposal-2026-04-22.md`
- `docs/contracts/export-overlays/source-materials/golden-sample-library-642bd7876e0d4649814f35fb133b67f3-2026-04-22.normalized.json`
- `docs/contracts/export-overlays/source-materials/golden-sample-v0.2-schema-note-2026-04-22.md`
- `docs/contracts/export-overlays/source-materials/kb-golden-sample-intake-review-2026-04-22.md`

No product code, exporter, IPC, Rust DTO, validator, desktop UI, workbook,
Qwen integration, Seedance integration, or `hope-kb` seed file is changed by
this proposal.

## Acceptance Decision

`V3_GOLDEN_SAMPLE_OVERLAY_PROPOSAL_ACCEPTED_FOR_V0_2_PLANNING`

The proposal is accepted as v0.2 planning input only.

Accepted points:

- `golden_sample_library` should be a dedicated v0.2 schema overlay.
- The 40-row golden sample intake must not be flattened into v0.1
  `classic_case_examples`.
- The proposal preserves the 17 source fields, source CSV hash, row index,
  normalized artifact ID, provenance, and staging decision.
- It correctly separates future Qwen few-shot use, validator evidence, repair
  mapping, and negative-sample signals.
- It keeps `director_voice` as internal retrieval metadata and does not
  authorize prompt text that imitates a specific director.
- It records the current coverage gap: no `reference_control_core` rows exist
  in this intake batch, so that coverage must not be invented.

## Proposed Schema Fields Accepted For Planning

Source fields that must remain first-class in v0.2 planning:

- `sample_title`
- `core`
- `covered_points`
- `created_at`
- `director_voice`
- `empty_words_detected`
- `genre`
- `missed_points`
- `sample_id`
- `scene_tag`
- `shot_type`
- `source_cut`
- `teaching_note`
- `tier`
- `updated_at`
- `usable_for_fewshot`
- `word_count`

Provenance / governance fields accepted for planning:

- `source_csv`
- `source_sha256`
- `normalized_artifact_id`
- `normalized_at`
- `row_index`
- `stable_sample_id`
- `kb_staging_status`
- `v0_1_seed_mapping`
- `imported_into_v0_1_seed`
- `preservation_contract`

Future derived / implementation-gated fields accepted for planning:

- `sample_text`
- `usable_for_validator`
- `usable_for_repair`
- `sample_type`
- `sample_usage`
- `negative_sample_signal`
- `v3_core_coverage_profile`
- `generation_stage`
- `v4_profile`
- `retrieval_tags`
- `validator_expected_result`
- `expected_failure_codes`
- `repair_template_id`
- `repair_hint`
- `negative_boundary_marker`

These fields are planning input. They are not frozen runtime contracts.

## Current Prohibitions

Still prohibited in the current RC line:

- do not modify the frozen v0.1 17-sheet workbook
- do not add `golden_sample_library` to current v0.1 seed/import/snapshot
- do not change exporter behavior
- do not change IPC
- do not change Rust DTOs or validators
- do not change desktop UI
- do not connect live Qwen
- do not connect Seedance
- do not merge `hope-kb` into `hope`
- do not treat `7a08cd6` as a frozen implementation contract

## Next Control Step

The V3 field thread should stop after this accepted proposal and wait.

The next useful main-control action is a v0.2 scope-gate readiness memo that
orders future work across:

1. `hope-kb` v0.2 schema/staging promotion
2. Hope domain / validator planning
3. export overlay planning
4. desktop read-only provenance planning
5. intake / Qwen boundary planning

That memo may prepare an implementation order, but implementation should not
start until the control thread explicitly opens a v0.2 scope gate.

## Message To V3 Field Thread

```text
Your docs-only golden-sample v0.2 overlay proposal at `7a08cd6` is accepted by
main control as planning input only.

Stop further proposal expansion for now and wait for the main control thread's
v0.2 scope-gate readiness decision.

Do not modify product code, exporter, IPC, Rust DTOs, validators, desktop UI,
workbook, Qwen integration, Seedance integration, or hope-kb seed files.
```
