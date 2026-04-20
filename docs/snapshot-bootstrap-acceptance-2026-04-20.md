# Hope Snapshot Bootstrap Acceptance Memo 2026-04-20

## Route

This memo is a docs-only main-thread acceptance record on `codex/contracts-freeze`.

It does not merge branches, it does not reopen merge-readiness, and it does not authorize implementation on the frozen main thread.

## Main-Thread Baseline

- frozen main-thread branch: `codex/contracts-freeze`
- frozen main-thread baseline: `ecff1da`
- reviewed controlled-intake checkpoint branch: `codex/controlled-intake-snapshot-bootstrap`
- accepted checkpoint commit: `2490055` (`Add checkpoint-driven snapshot bootstrap context`)

## Acceptance Decision

`2490055` is accepted as the stable `snapshot_bootstrap` checkpoint inside `crates/project-store`.

This acceptance is limited to the first runtime consume surface only.

This acceptance does not mean:

- merge of `codex/controlled-intake-snapshot-bootstrap` into `codex/contracts-freeze`
- reopening merge-readiness
- expansion into `validation_feedback_projection`
- expansion into `segment_and_cut_projection`
- expansion into `handoff_projection`
- expansion into `prompt_package_projection`
- any change in `hope-kb`

## Accepted Checkpoint Scope

The accepted checkpoint content is:

- `VerifiedKbContext`
- `SnapshotBootstrapCheckpointArtifacts`
- `bootstrap_verified_kb_context_from_checkpoint(...)`

The acceptance basis is that this checkpoint gives `project-store` a bounded reviewed-checkpoint entry point for `snapshot_bootstrap` without widening the intake surface set.

## Accepted Boundary

This acceptance is specifically for a stable checkpoint in `project-store`.

It is not:

- an app-side intake implementation
- a desktop-side hookup
- a product-wide runtime consume rollout
- a merge of the controlled-intake branch back into the frozen main thread

## Authorized Next Phase

The next implementation phase is authorized on a new branch only:

- create from: `2490055`
- new branch: `codex/controlled-intake-snapshot-bootstrap-app`

That next phase is limited to app-side bounded intake only.

## Next-Phase Allowed Scope

The next branch may:

- touch `app/src/runtime.rs` only
- consume `SnapshotBootstrapCheckpointArtifacts`
- consume `bootstrap_verified_kb_context_from_checkpoint(...)`

## Next-Phase Explicitly Blocked

The next branch may not:

- expand into `validation_feedback_projection`
- expand into `segment_and_cut_projection`
- expand into `handoff_projection`
- expand into `prompt_package_projection`
- touch desktop-side code
- touch `hope-kb`
- write implementation directly on `codex/contracts-freeze`

## Main-Thread Position After Acceptance

`codex/contracts-freeze` remains frozen.

Main-thread work after this memo stays in waiting mode until the bounded app-side intake branch reports back with its own controlled-intake results.
