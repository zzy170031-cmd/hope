# Hope RC Release Packet 2026-04-20

## Release State

- Repo: `E:\codex\hope`
- Branch: `codex/contracts-freeze`
- Release state: `RC_READY`
- Main thread state: current RC baseline is locked, no feature reopening, and the main thread remains scope-frozen

## Baseline Commit

- Baseline commit: `f75c847`
- Commit subject: `Confirm Hope RC recovery baseline`
- Release reference point: all current release-confirmation materials trace back to this commit

## What Is Included

The current RC baseline includes only:

1. storyboard runtime baseline protection
2. exporter-only face-detail cleanup polish
3. refreshed fixtures / validation / exports
4. RC release confirmation materials

The face-detail cleanup work is already included in `f75c847` and must not be repeated as a new patch in this stage.

## Benchmark Ladder

The frozen benchmark ladder remains green against the current RC baseline:

- `cargo test -p storyboard-pipeline`
- `cargo test -p export-engine --test benchmark_45s_clip`
- `cargo test -p export-engine benchmark_10min_single_episode_runs_end_to_end -- --exact`
- `cargo test -p export-engine --test benchmark_30min_single_episode`
- `cargo test -p export-engine --test benchmark_60min_project`
- `cargo test -p export-engine --lib engine::tests::render_face_detail_clause_is_stronger_than_layout_clause -- --exact`
- `cargo test -p export-engine --lib engine::tests::appearance_constraint_appends_face_cleanup_defaults -- --exact`

Confirmed results:

- storyboard-pipeline: `4 passed`
- export-engine 45s benchmark: `passed`
- export-engine 10min benchmark: `passed`
- export-engine 30min benchmark: `passed`
- export-engine 60min benchmark: `passed`
- exporter focused unit tests: `2 passed`

## Baseline Artifacts

The committed snapshot at `f75c847` is the canonical RC artifact baseline.

Confirmed artifact set:

- `contracts/fixtures/week3-shared-fixture.json`
- `contracts/fixtures/week3-validation-report.json`
- `contracts/fixtures/exports/week3-export.json`
- `contracts/fixtures/exports/week3-export.md`
- `contracts/fixtures/exports/week3-export.xlsx`

These files are part of the current release confirmation surface.

## Frozen Boundary

The following remain explicitly out of scope for this RC line:

- no new features
- no new contracts
- no KB scope expansion
- no deeper runtime integration
- no repeat face-detail patch
- no merge with `hope-kb` yet

## Post-RC Handling

From this point, Hope main-thread work is limited to release confirmation, follow-up scheduling, and merge-readiness observation. Follow-up remains scheduled work, and merge-readiness is observation-only at this stage.
