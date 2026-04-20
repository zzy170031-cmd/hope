# Hope Snapshot Bootstrap Package 2026-04-20

## Package Goal

Deliver the first controlled intake package for `snapshot_bootstrap` only.

This package should move Hope away from the current bootstrap-era local path in `crates/project-store/src/kb_runtime.rs` and toward an explicit, reviewed snapshot-contract bootstrap boundary.

This package is for controlled intake preparation only. It does not deliver a finished product hookup, and it must not bring `hope-kb` repo logic directly into `hope`.

## In Scope

- `snapshot_bootstrap` only
- controlled intake preparation only
- runtime bootstrap metadata and bundle identity handling
- reviewed bundle hash pinning
- `manifest.json` / validator result / snapshot result consistency checks
- loading trusted bootstrap inputs from the reviewed snapshot contract
- targeted tests for bootstrap behavior and guardrails
- narrow interface changes only when required to carry explicit snapshot identity

## Entry Conditions

This package should proceed only if all of the following remain true:

1. the working branch is a dedicated `snapshot_bootstrap` branch and is not `codex/contracts-freeze`
2. the branch still traces back to `ecff1da`
3. the reviewed bundle hash is pinned to `bundle-sha256:5f042c10ada3726bdbd71d5f4cbad2187b3895dd85e1e5195d6938a3a22d20b6` or a later explicitly re-reviewed checkpoint
4. `manifest.json`, validator result, and snapshot result are mutually consistent at the selected checkpoint
5. the package plan names only these trusted bootstrap inputs:
   - `snapshot_meta`
   - `export_template`
   - `failure_pattern`
   - `degraded_input_example`
   - `runtime_consume_contract`
6. no code path in this package depends on reopening merge-readiness or editing `hope-kb`

## Trusted Inputs

The implementation plan for this package must treat these as named bootstrap inputs:

- `snapshot_meta`
- `export_template`
- `failure_pattern`
- `degraded_input_example`
- `runtime_consume_contract`

The reviewed bundle hash remains pinned to:

- `bundle-sha256:5f042c10ada3726bdbd71d5f4cbad2187b3895dd85e1e5195d6938a3a22d20b6`

or a later checkpoint that has been explicitly re-reviewed.

## Current Gap

Current Hope bootstrap behavior is still a lightweight local path:

- `load_kb_runtime` stamps `snapshot_hash = "runtime-unverified"`
- knowledge loading derives from repo-local seed paths
- reviewed bundle identity is not carried as an explicit runtime input

That behavior is acceptable for the frozen RC branch, but this package should not preserve it as the long-term intake boundary.

## Explicitly Blocked

Do not include any work for:

- `validation_feedback_projection`
- `segment_and_cut_projection`
- `handoff_projection`
- `prompt_package_projection`
- exporter-local prompt assembly changes
- validator blocking semantics changes
- merge-readiness reopening
- repo merge with `hope-kb`
- copying `hope-kb` repo logic into `hope`
- desktop-shell, UI shell, or unrelated UI file changes
- any new local fallback that substitutes for the reviewed contract

## Guardrails

The package must preserve these intake guardrails:

- no local sheet alias fallback
- no local blocking inference
- no local `render_segment` backfill
- no local handoff safe default
- no local negative/default prompt fill

## Verification Lens

Each work package must explicitly report:

- branch detection result
- scope detection result
- route detection result
- Git detection result
- entry condition result
- exit condition result
- conclusion: continue, pause, or bounded feedback needed

## Exit Criteria

This package is complete only when all of the following are true:

1. bootstrap reads from the validated snapshot contract instead of repo-local guess paths
2. bootstrap preserves reviewed bundle identity as an explicit runtime input
3. `manifest.json`, validator result, and snapshot result remain mutually consistent at the chosen checkpoint
4. bootstrap does not substitute Validation-sheet aliases for the declared contract
5. other four consume surfaces remain behaviorally untouched

## Suggested Verification

- unit tests for missing and valid snapshot bootstrap inputs
- a focused test proving bundle identity is preserved instead of stamped as `runtime-unverified`
- a focused test proving bootstrap does not require repo-local seed guessing to succeed
- regression check that exporter and validator packages remain outside this change set

## Likely Landing Zone

Primary code landing zone:

- `crates/project-store/src/kb_runtime.rs`

Primary review question:

- does this change create a real snapshot bootstrap boundary without quietly pulling in the next four consume surfaces?
