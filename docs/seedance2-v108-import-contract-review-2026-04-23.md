# Seedance2 V108 Import Contract Review 2026-04-23

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- review type: main-control acceptance of docs-only V108 import contract
  candidate
- reviewed candidate commit:
  `b69c7edb4f2096604f9af37bef1156b89c2e48c4`
  (`docs: add Seedance2 V108 import contract candidate`)
- reviewed candidate document:
  `docs/seedance2-v108-import-contract-candidate-2026-04-23.md`
- accepted schema contract anchor:
  `198b31aed84f22908e40e55f7ff56673b88c049b`
  (`docs: accept Seedance2 V108 schema contract`)
- KB source anchor:
  `6e5a150c39e3e1c7abba1da08fb1d13bf8e199ee`
  (`docs: pin Seedance2 V108 canonical headers`)
- V3 archived anchor:
  `bc745a4d8b48ddc259c8b1c94ab907f8946797c0`
  (`docs: align V3 proposal with Seedance2 V108`)

This review accepts or rejects the candidate only. It does not import V108 rows
or open product implementation.

## Reviewed Candidate

The candidate defines these planning contract objects:

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

Scope check:

- Only documentation was added by the candidate.
- No KB seed files, Hope product code, Rust DTO, validator, repair, exporter,
  workbook, IPC, desktop, intake, Qwen, Seedance, or V3 branch file was changed.
- Hope, KB, and V3 anchors remain Git-visible.

## Acceptance Decision

`SEEDANCE2_V108_IMPORT_CONTRACT_CANDIDATE_ACCEPTED`

Main control accepts the candidate as the docs-only planning contract for a
future V108 import dry-run and import readiness review.

Accepted points:

- The candidate preserves the accepted V108 23-header source surface.
- It records the accepted source facts: `115` data rows, `108` official rows,
  `7` reserve rows, `102` placeholder rows, `13` placeholder-clean prompt
  candidates, and `0` rows ready for positive few-shot.
- It keeps raw source row handling separate from product import.
- It defines preflight checks for source hashes, sheet name, header row, row
  counts, normalized staging hash, source anchors, no positive few-shot
  promotion, and no `reference_control_core`.
- It defines row candidates as raw evidence only.
- It defines blockers without deleting rows or promoting blocked rows.
- It sets the current accepted import outcome to:
  `raw_source_rows_ready_for_future_dry_run = 115`,
  `rows_ready_for_product_import = 0`, and
  `rows_ready_for_positive_fewshot = 0`.
- It constrains derived views:
  - `V108SequenceView`: planning only
  - `V108PromptCandidateView`: candidate text only, no runtime use
  - `V108ReferenceHandleView`: blocked, product-ready handles remain `0`
  - `V108ValidatorEvidenceView`: planning only
- It requires a future dry-run report before implementation.
- It keeps import implementation closed until main control accepts all required
  gates.

## No Rejection Findings

No blocking rejection finding was found.

The candidate is intentionally conservative: it defines how a future dry-run
should reason about V108 rows while preserving the current state that no product
rows are imported.

## Remaining Closed Gates

This acceptance does not authorize:

- writing imported rows
- changing KB seed files
- changing Hope product code
- Rust DTO changes
- validator implementation
- repair implementation
- writer/orchestrator implementation
- exporter behavior changes
- workbook changes
- IPC changes
- desktop changes
- intake changes
- Qwen calls or prompt assembly implementation
- Seedance calls or adapter implementation
- V3 branch edits
- runtime positive few-shot promotion
- creating `reference_control_core`
- inventing reference images, URLs, asset IDs, character appearance, or scene
  design

## Next Control State

The five-thread state after this review is:

- Hope main: `codex/contracts-freeze`, V108 schema and import contracts
  accepted docs-only; implementation still closed.
- KB: `codex/contracts-freeze @ 6e5a150`, V108 headers and reference boundaries
  accepted; standby.
- V3: `codex/v3-field-overlay-proposal @ bc745a4`, archived / standby.
- Desktop: waiting.
- Intake: waiting.
- Qwen and Seedance: closed.

Next eligible gate:

1. `V108 import dry-run report`, docs-only, producing preflight and blocked-row
   evidence without writing product rows.
2. `KB canonical object registry package`, if story/character/scene/prop/style
   names become available.
3. `V108 validator evidence contract candidate`, after import dry-run evidence
   is accepted and still docs-only.

Do not proceed directly to implementation from this acceptance.

Recommended next label:

```text
Hope主线-Seedance2V108ImportContract【已接收·待下轮Gate】
```

关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。
