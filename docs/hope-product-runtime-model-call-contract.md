# Hope Product Runtime Model Call Contract

Status: docs-only freeze candidate
Default runtime mode: Standard / MVP 3/4
Scope: product runtime target contract

## Product Boundary

Hope is not a chat-style multi-turn polishing tool.

Hope's product runtime goal is one user input, one complete delivery package:

- Complete story
- Script
- Scene duration plan
- Storyboard prompt package
- Excel-ready structure

The product runtime must not be treated as an open-ended conversation loop. It must produce a bounded, validated package from a single user request.

This contract is the product runtime target contract. The current code implementation is not equivalent to completion of this 3/4-call architecture.

Current engineering must first complete all of the following before entering 403-case:

- `qwen-max` storyboard grounding fix
- Four trace gates
- Scene-type-driven rewrite gate

Development-time GPT/Codex usage is separate from product runtime model usage. Developer assistance, code generation, QA investigation, and thread governance must never be counted as product runtime model calls and must never be mixed with the product runtime provider/model ledger.

## Frozen Contract Names

The following 10 contract names are frozen for product runtime planning, implementation, validation, and handoff language.

1. `HopeRuntimeModeContract`

   Defines the allowed runtime modes, their defaults, and the explicit-selection boundary. Standard / MVP is the default mode. Extended is future/explicit-choice only and must not become the implicit default.

2. `HopeRuntimeCallBudgetContract`

   Defines the product runtime model-call budget by mode. Standard / MVP is happy path 3 calls and max path 4 calls. Extended is happy path 3 calls and max path 6 calls. Validation, export, KB matching, ledger writes, token estimate, and other deterministic operations do not count as model calls.

3. `NarrativeTransformContract`

   Defines the first primary model call. It transforms the user's single input into the bounded narrative/script basis used by downstream planning. It must not become a recurring chat refinement loop.

4. `SceneDurationPlanContract`

   Defines the second primary model call. It produces the scene duration plan from the narrative basis and must preserve a structured, validator-checkable output surface.

5. `StoryboardPromptPackageContract`

   Defines the third primary model call. It produces the storyboard prompt package and Excel-ready structured package from the approved narrative and duration planning inputs.

6. `BoundedRepairContract`

   Defines the only repair call allowed in Standard / MVP. It is triggered only by validator failure, can run at most once, and may only repair validator-identified failed items. It must not rewrite the full package, perform a second polishing pass, or retry indefinitely.

7. `HopeProviderCapabilityGate`

   Defines the BYOK and provider/model capability classification gate. Each provider/model must be classified as `supported`, `extended-only`, or `unsupported` based on the required capability checks.

8. `HopeKBSnapshotSummaryContract`

   Defines how KB context enters runtime prompts. Runtime must use bounded KB matches, snapshot references, retrieved facts, summaries, or rule packages. It must not stuff large raw KB passages directly into prompts.

9. `HopeValidationExportContract`

   Defines deterministic validation and export behavior. Schema validation, duration validation, continuity reference checks, export, and Excel shaping are 0-model-call operations and must not silently invoke product runtime models.

10. `HopeRunLedgerContract`

    Defines the required run ledger fields and audit rules. Model identity, runtime mode, stage, call count, validator findings, repair reason, KB snapshot/summary identity, cost estimate, fallback use, and final status must be recorded.

## Standard / MVP Runtime Mode

Standard / MVP is the default mode.

Standard / MVP call budget:

- Happy path: 3 model calls
- Max path: 4 model calls

The three primary model calls are:

1. `NarrativeTransform`
2. `SceneDurationPlan`
3. `StoryboardPromptPackage`

The optional fourth call is:

4. `BoundedRepair`

`BoundedRepair` rules:

- Triggered only when validator findings fail the package.
- At most 1 repair call per run.
- May only repair the specific failed items identified by the validator.
- Must not perform a second polishing pass.
- Must not rewrite the full package.
- Must not create an unbounded retry loop.
- Must preserve the run's configured provider/model unless an explicit, ledgered model switch has been configured before the repair.

## Extended Runtime Mode

Extended mode is future/explicit-choice only.

Extended call budget:

- Happy path: 3 model calls
- Max path: 6 model calls

Extended rules:

- Each primary stage may have at most 1 repair.
- The user must explicitly select Extended mode.
- Extended mode is not the MVP default.
- Extended mode must be recorded in the run ledger as `runtime_mode`.
- Extended mode must not be entered silently from Standard / MVP because validation failed.

## Zero-Model-Call Operations

The following operations are always 0 product runtime model calls:

- KB matching
- KB snapshot reading
- KB summary / rule package selection
- Schema validation
- Duration validation
- Continuity reference check
- Export / Excel shaping
- Run ledger recording
- Token / cost estimate display
- Error report generation

These operations may use deterministic code, local data, recorded metadata, retrieved matches, prebuilt snapshots, or prebuilt summary packages. They must not silently invoke a product runtime model.

## Prohibited Runtime Behavior

The product runtime must not:

- Default to one model call per scene.
- Default to one model call per shot.
- Silently switch models inside a UI-driven trace.
- Perform hidden repair.
- Retry indefinitely.
- Treat fallback as a fake pass.
- Mix development-stage GPT/Codex usage with product runtime model calls.
- Hard-code Qwen as the only product runtime model.
- Put large raw KB passages directly into the prompt.
- Enter Extended mode without explicit user selection.
- Enter 403-case before the required pre-403 gates are complete.

Fallback behavior, if any, must be explicit, configured, ledgered, and unable to mark a failed run as passed without validator success.

## BYOK Capability Gate

Bring-your-own-key support must classify each provider/model as one of:

- `supported`
- `extended-only`
- `unsupported`

Capability checks must include at least:

- `context_length_ok`
- `output_length_ok`
- `structured_json_ok`
- `provider_model_identity_recordable`
- `sampling_params_controllable`
- `timeout_error_rate_limit_handled`
- `external_tool_dependency_absent`

A provider/model that does not satisfy the Standard / MVP gate must not be used as the default Standard / MVP runtime model. A provider/model that only satisfies the broader Extended gate may be marked `extended-only`, but Extended still requires explicit user selection.

## Runtime Model Priority

Default recommended model/configuration priority:

1. `qwen-max`
2. `qvq-max-2025-03-25`
3. `qwen-math-turbo`

This priority is a default recommendation and configuration order only. The runtime must not silently switch models during a run. Any model switch must be explicit configuration, must be visible to the operator/user flow where applicable, and must be recorded in the run ledger.

Qwen must not be hard-coded as the only valid product runtime model. The runtime contract must preserve provider/model identity as configuration and ledger data.

## Run Ledger Requirements

Every product runtime run must record:

- `runtime_mode`
- `task_id`
- `provider`
- `model`
- `call_count`
- `stage`
- Token estimate / usage if available
- `repair_reason`
- `validator_findings`
- KB snapshot id / summary package id
- Cost estimate
- `fallback_used`
- `final_status`

Ledger entries must make model calls auditable by stage. Repair calls must identify the validator finding that authorized them. Fallback usage must never be hidden and must never convert a failed validation result into a pass.

## KB Prompt Boundary

Runtime prompts must use bounded KB references, snapshots, retrieved facts, or summary packages.

Large raw KB passages must not be copied into prompts by default. Prompt construction must preserve a reviewable KB snapshot id or summary package id in the run ledger so the source context can be audited without inflating every model call.

## Implementation Readiness Boundary

This document freezes the target product runtime contract only. It does not assert that the current implementation already enforces:

- Standard / MVP 3/4 call orchestration
- Extended 3/6 explicit-choice orchestration
- Full provider capability gating
- Full zero-model-call validation/export/ledger behavior
- Full run ledger audit surface
- Full KB snapshot/summary prompt boundary

Before any 403-case work starts, the engineering sequence must complete:

1. `qwen-max` storyboard grounding fix
2. Four trace gates
3. Scene-type-driven rewrite gate

## Freeze Summary

This contract freezes Standard / MVP as the default 3/4-call product runtime path:

- 3 primary calls on the happy path.
- 1 bounded repair call only after validator failure.
- 0 model calls for KB matching, KB snapshot reading, KB summary/rule package selection, schema validation, duration validation, continuity reference checks, export/Excel shaping, run ledger recording, token/cost estimate display, and error report generation.
- No per-scene or per-shot default model-call expansion.
- No hidden repair, infinite retry, silent model switch, fake fallback pass, raw-KB prompt stuffing, or premature 403-case entry.

Extended 3/6 remains future/explicit-choice only and is not the MVP default.
