# Hope RC Release Confirmation 2026-04-20

See [rc-release-packet-2026-04-20.md](E:\codex\hope\docs\rc-release-packet-2026-04-20.md) for the canonical release-confirmation summary and frozen-boundary statement for the current RC line.

## Release State

- Repo: `E:\codex\hope`
- Branch: `codex/contracts-freeze`
- Release state: `RC_READY`
- Main thread state: current RC baseline is locked and the main thread remains scope-frozen

This document confirms the current Hope main-thread RC baseline. It is not a feature-progress report and does not reopen product scope.

## Baseline Commit

- Baseline commit: `f75c847`
- Commit subject: `Confirm Hope RC recovery baseline`
- Release reference point: all current RC confirmation materials trace back to this commit on `codex/contracts-freeze`
- `角色面部细节清洁度优化` is already included in the locked baseline and must not be patched again unless it becomes a new RC blocker

## RC Release Confirmation Materials

Current release-confirmation packet for the Hope main thread consists of:

- locked baseline commit
- frozen benchmark ladder results
- refreshed fixture / validation / export baseline artifacts
- documented freeze boundary and prohibited scope

This packet is intended to support RC publication confirmation, not a new implementation cycle.

## Confirmed Main-Thread Changes

### 1. Baseline protection for storyboard runtime inputs

`crates/storyboard-pipeline/src/lib.rs`

- keep `scene_taxonomy` as metadata when explicit runtime inputs are present
- stop deriving missing primary director ids from taxonomy defaults
- stop injecting taxonomy-derived prompt wording into runtime prompt layers
- keep handoff boundary on `render_segment_boundary`
- reject missing-director plans even when taxonomy metadata exists

This change is treated as baseline protection because the current `export-engine` 10min RC benchmark expects explicit runtime director ids and plain handoff boundaries to remain authoritative.

### 2. Non-blocking exporter polish

`crates/export-engine/src/engine.rs`

- add source-level face-detail cleanup wording to appearance constraints
- make render-level face-detail wording stronger than layout-level wording
- append default negative constraints for dirty / smeared / duplicated / low-clarity facial details

This remains inside the handoff-approved non-blocking scope:

- `prompt / wording`
- `exporter`
- `negative prompt defaults`

## Verification

Because the default repo `target` directory was locked on this device, verification was run with:

- `CARGO_TARGET_DIR=E:\codex\hope\target-isolated`

Commands confirmed in this round:

1. `cargo test -p storyboard-pipeline`
2. `cargo test -p export-engine benchmark_10min_single_episode_runs_end_to_end -- --exact`
3. `cargo test -p export-engine --test benchmark_30min_single_episode`
4. `cargo test -p export-engine --test benchmark_45s_clip`
5. `cargo test -p export-engine --test benchmark_60min_project`
6. `cargo test -p export-engine --lib engine::tests::render_face_detail_clause_is_stronger_than_layout_clause -- --exact`
7. `cargo test -p export-engine --lib engine::tests::appearance_constraint_appends_face_cleanup_defaults -- --exact`

Results:

- storyboard-pipeline: `4 passed`
- export-engine 10min RC benchmark: `passed`
- export-engine 30min RC benchmark: `passed`
- export-engine 45s RC benchmark: `passed`
- export-engine 60min RC benchmark: `passed`
- exporter focused unit tests: `2 passed`

This round confirms that the RC recovery is not limited to a single benchmark path; the frozen benchmark ladder remains green after the storyboard runtime protection change.

## Baseline Artifacts

The frozen benchmark ladder exercises the tracked baseline artifacts as part of fixture / validation / export confirmation:

- `contracts/fixtures/week3-shared-fixture.json`
- `contracts/fixtures/week3-validation-report.json`
- `contracts/fixtures/exports/week3-export.json`
- `contracts/fixtures/exports/week3-export.md`
- `contracts/fixtures/exports/week3-export.xlsx`

These outputs are part of the RC confirmation surface for the current main-thread route, and the committed snapshot at `f75c847` is the canonical baseline artifact set.

## Frozen Boundary

The Hope main thread remains frozen at the current RC baseline. Do not use this thread to:

- add new features
- add new contracts
- widen KB scope
- deepen runtime integration
- repeat the face-detail cleanup patch
- merge with `hope-kb`

Any post-release work that does not directly belong to RC confirmation must be scheduled into follow-up material instead of reopening implementation scope here.

## Current Progress Snapshot

Compared against `docs/nightly-handoff-2026-04-20.md`, the main thread is still on the required Git route:

- branch stays on `codex/contracts-freeze`
- `HEAD` stays aligned to `origin/codex/contracts-freeze` at `f75c847`
- release state stays at `RC_READY`
- current work remains inside RC confirmation, baseline protection / benchmark recovery, and exporter-only non-blocking polish

Tracked deliverables in the current worktree:

- `crates/storyboard-pipeline/src/lib.rs`
- `crates/export-engine/src/engine.rs`
- `contracts/fixtures/week3-shared-fixture.json`
- `contracts/fixtures/week3-validation-report.json`
- `contracts/fixtures/exports/week3-export.json`
- `contracts/fixtures/exports/week3-export.md`
- `contracts/fixtures/exports/week3-export.xlsx`
- this confirmation note

Local-only noise on this device that is not part of the release deliverable set:

- `target-*` temporary build directories created by isolated verification runs
- CRLF/index-only working-tree markers on `.gitignore`
- CRLF/index-only working-tree markers on `crates/writer-pipeline/src/lib.rs`
- CRLF/index-only working-tree markers on `docs/project-thread-startup.md`

The main-thread release boundary includes the tracked deliverables above and excludes the temporary target directories.

## Route Comparison

Compared with the current Git-backed requirements for `hope` main thread:

- `lock current RC baseline`: satisfied by the refreshed fixture / export artifacts and the green benchmark ladder
- `allow only non-blocking polish`: satisfied by the exporter face-detail cleanup wording change
- `do not add features`: respected
- `do not expand KB contract`: respected
- `do not widen runtime integration`: respected; storyboard change keeps taxonomy metadata visible but removes runtime fallback behavior
- `do not merge with hope-kb`: respected

The remaining next-step work should therefore stay on release confirmation and worktree cleanup, not feature growth.

## Freeze Reminder

- do not reopen feature growth on `hope`
- do not expand KB contract from this thread
- do not widen runtime integration beyond baseline protection
- do not merge or pull work from `hope-kb` in this thread
