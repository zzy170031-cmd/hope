# Hope Controlled Intake Thread Startup 2026-04-20

## Route

This branch is a bounded implementation thread for the first controlled intake surface only.

- repo: `E:\codex\hope`
- base commit: `ecff1da`
- branch: `codex/controlled-intake-snapshot-bootstrap`
- active surface: `snapshot_bootstrap`

The frozen `RC_READY` route on `codex/contracts-freeze` remains unchanged. This branch exists so intake work can start without reopening merge-readiness, without widening runtime integration, and without merging with `hope-kb`.

## Use This Prompt In The New Thread

```text
This thread is the Hope controlled-intake implementation thread for the first runtime consume surface only.

Repository:
- E:\codex\hope
- branch: codex/controlled-intake-snapshot-bootstrap
- base commit: ecff1da

Route:
- keep Hope main release baseline frozen
- do not reopen merge-readiness
- do not merge with hope-kb
- do not expand to the other four consume surfaces

Scope for this thread:
- implement only snapshot_bootstrap intake preparation and bounded product hookup
- keep the work inside crates/project-store and closely related tests unless a narrow interface adjustment is required
- preserve the reviewed bundle identity as an explicit runtime input
- stop using repo-local guess paths as the long-term bootstrap boundary

Explicit non-goals:
- validation_feedback_projection
- segment_and_cut_projection
- handoff_projection
- prompt_package_projection
- exporter behavior changes
- validator semantics changes
- KB schema growth

Acceptance direction:
- bootstrap consumes the reviewed snapshot contract as an explicit input
- no local sheet alias fallback
- no local blocking inference
- no local render_segment backfill
- no local handoff safe default
- no local negative/default prompt fill

Start by reading:
- docs/nightly-handoff-2026-04-20.md
- docs/runtime-consume-intake-schedule-2026-04-20.md
- docs/snapshot-bootstrap-package-2026-04-20.md
- crates/project-store/src/kb_runtime.rs
```

## First Thread Check

Before implementation claims, confirm:

1. branch is still `codex/controlled-intake-snapshot-bootstrap`
2. scope is still limited to `snapshot_bootstrap`
3. `hope` and `hope-kb` remain separate
4. current work does not mutate the frozen RC baseline route
