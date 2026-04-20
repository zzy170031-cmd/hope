# Hope RC Release Confirmation 2026-04-20

## Route

- Repo: `E:\codex\hope`
- Branch: `codex/contracts-freeze`
- Baseline: `RC_READY`
- Scope: keep main thread frozen; allow RC confirmation, baseline protection / benchmark recovery, and exporter-only non-blocking polish

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

## Baseline Artifacts Refreshed

The 10min RC benchmark rewrites tracked baseline artifacts as part of fixture / export confirmation:

- `contracts/fixtures/week3-shared-fixture.json`
- `contracts/fixtures/week3-validation-report.json`
- `contracts/fixtures/exports/week3-export.json`
- `contracts/fixtures/exports/week3-export.md`
- `contracts/fixtures/exports/week3-export.xlsx`

These outputs are part of the RC confirmation surface for the current main-thread route.

## Current Progress Snapshot

Compared against `docs/nightly-handoff-2026-04-20.md`, the main thread is still on the required Git route:

- branch stays on `codex/contracts-freeze`
- `HEAD` stays aligned to `origin/codex/contracts-freeze` at `ca78b43`
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

If this round is staged later, the main-thread boundary should include the tracked deliverables above and exclude the temporary target directories.

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
