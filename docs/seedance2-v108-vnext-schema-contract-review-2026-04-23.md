# Seedance2 V108 vNext Schema Contract Review 2026-04-23

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- review type: main-control acceptance of docs-only V108 vNext schema contract
  candidate
- reviewed candidate commit:
  `b4dbf531699d7f638dfd3a575a3c35c75d6fa408`
  (`docs: add Seedance2 V108 schema contract candidate`)
- reviewed candidate document:
  `docs/seedance2-v108-vnext-schema-contract-candidate-2026-04-23.md`
- KB source anchor:
  `6e5a150c39e3e1c7abba1da08fb1d13bf8e199ee`
  (`docs: pin Seedance2 V108 canonical headers`)
- V3 archived anchor:
  `bc745a4d8b48ddc259c8b1c94ab907f8946797c0`
  (`docs: align V3 proposal with Seedance2 V108`)

This review accepts or rejects the candidate only. It does not open product
implementation.

## Reviewed Candidate

The candidate defines these planning contract objects:

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

Scope check:

- Only documentation was added by the candidate.
- No Rust DTO, validator, repair, exporter, workbook, IPC, desktop, intake,
  Qwen, Seedance, V3 branch, or `hope-kb` file was changed.
- Hope, KB, and V3 anchors remain Git-visible.

## Acceptance Decision

`SEEDANCE2_V108_VNEXT_SCHEMA_CONTRACT_CANDIDATE_ACCEPTED`

Main control accepts the candidate as the docs-only planning contract for future
V108 import, validator, repair, retrieval, prompt-candidate, and adapter
planning.

Accepted points:

- The V108 23-header worksheet shape is preserved as the canonical source field
  surface.
- `V108SourceRow` preserves raw source values and provenance without rewriting
  source content or compiling Seedance output.
- `V108ReviewFlags` records official/reserve status, placeholder surfaces,
  alignment gaps, unresolved references, and promotion blockers without making
  product behavior decisions.
- `V108SequenceView` derives only from `sample_type`, `sequence_id`,
  `shot_order`, and `shot_id`.
- `V108PromptCandidateView` treats `prompt_body` as source prompt candidate
  text, not compiled Seedance output.
- `V108ReferenceHandleView` remains future-gated with
  `product_ready_external_reference_handles = 0`.
- `reference_control_core_coverage = 0` remains explicit.
- `V108ValidatorEvidenceView` names evidence families without implementing
  validators, severities, thresholds, or failure-code enums.
- `scene_performance_core` and `continuity_negative_core` remain fused at source
  level.
- `V108PromotionGate` keeps runtime positive few-shot promotion at `0`.
- `V108ImportGateOrder` correctly prevents V108 rows from skipping KB and
  main-control review into runtime prompts or product structures.

## No Rejection Findings

No blocking rejection finding was found.

The candidate is conservative and keeps all implementation gates closed. It is
therefore accepted for controlled v0.2 / vNext readiness planning.

## Remaining Closed Gates

This acceptance does not authorize:

- importing V108 rows into current KB v0.2 rows
- changing the v0.1 workbook
- Rust DTO changes
- validator implementation
- repair implementation
- writer/orchestrator implementation
- exporter behavior changes
- IPC changes
- desktop changes
- intake changes
- Qwen calls or prompt assembly implementation
- Seedance calls or adapter implementation
- V3 branch edits
- `hope-kb` edits
- positive few-shot promotion
- inventing `reference_control_core`
- inventing reference images, URLs, asset IDs, character appearance, or scene
  design

## Next Control State

The five-thread state after this review is:

- Hope main: `codex/contracts-freeze`, V108 vNext schema contract candidate
  accepted; implementation still closed.
- KB: `codex/contracts-freeze @ 6e5a150`, V108 headers and reference boundaries
  accepted; standby.
- V3: `codex/v3-field-overlay-proposal @ bc745a4`, archived / standby.
- Desktop: waiting.
- Intake: waiting.
- Qwen and Seedance: closed.

Next eligible gate:

1. `V108 import contract candidate`, if main control wants to define how raw
   V108 source rows would enter a future schema without implementation.
2. `KB canonical object registry package`, if story/character/scene/prop/style
   names become available.
3. `V108 validator evidence contract candidate`, after import boundaries are
   accepted and still docs-only.

Do not proceed directly to implementation from this acceptance.

Recommended next label:

```text
Hope主线-Seedance2V108SchemaContract【已接收·待下轮Gate】
```

关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。
