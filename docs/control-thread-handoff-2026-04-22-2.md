# Control Thread Handoff 2026-04-22-2

## Handoff Reason

This handoff is created because the current Hope control thread has again
reached the red threshold defined by:

- `docs/control-thread-context-supervision-2026-04-22.md`

Observed trigger:

- the current controller now sees a compacted summary instead of the full recent
  thread
- the user explicitly asked to fold session
  `019db449-896d-7e32-b29e-3f8afbea2f5a` into control-thread governance

Required action:

- stop opening new scope in this thread
- push this handoff
- continue formal coordination from a fresh Hope control thread

The source of truth for the next thread is Git, not chat history.

## Current Route

- date: 2026-04-22
- timezone: Asia/Shanghai
- main repo: `E:\codex\hope`
- main branch: `codex/contracts-freeze`
- route: `RC_READY` plus scope freeze
- handoff type: control-thread rotation / context protection
- anchor before this handoff: `1cb9b22` (`docs: record RC publication follow-through`)
- worktree state before this handoff: clean and aligned with
  `origin/codex/contracts-freeze`

## Context Threshold Scheme Integrated

Session `019db449-896d-7e32-b29e-3f8afbea2f5a` has been reviewed. Its durable
output is already committed on the Hope branch:

- `0db2a43`: `docs: add control thread context supervision`
- `a79787b`: `docs: add control thread handoff for context rotation`

The active rule is:

- yellow threshold: write and push a docs-only checkpoint before continuing
- red threshold: write and push a docs-only handoff, then rotate to a new
  control thread before formal work continues

This handoff applies that rule to the current thread.

## Current Anchors

Hope main:

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor: `1cb9b22` (`docs: record RC publication follow-through`)
- state: clean and aligned with `origin/codex/contracts-freeze`
- role: RC publication follow-through and baseline protection

Hope KB:

- repo: `E:\codex\hope-kb`
- branch: `codex/contracts-freeze`
- anchor: `6617434` (`docs: align KB live progress after desktop stable stop`)
- state: clean and aligned with `origin/codex/contracts-freeze`
- role: independent KB hardening / standby

Desktop:

- repo: `E:\codex\hope-desktop-shell`
- branch: `codex/desktop-shell`
- anchor: `66d687d` (`desktop-shell: add writer qwen input readiness boundary`)
- state: clean and aligned with `origin/codex/desktop-shell`
- role: WriterReadiness / InputBoundary complete and waiting

Controlled intake:

- repo: `E:\codex\hope-intake-app`
- branch: `codex/controlled-intake-snapshot-bootstrap-app`
- anchor: `d74bd11` (`qwen-storyboard: tighten contract boundary`)
- state: clean and aligned with
  `origin/codex/controlled-intake-snapshot-bootstrap-app`
- role: Qwen/storyboard contract shell complete and waiting

V3/V4 field proposal:

- branch: `origin/codex/v3-field-overlay-proposal`
- anchor: `13f8d7d` (`docs: add v3 field overlay proposal handoff`)
- state: v0.2 overlay proposal planning input only

## Accepted Decisions Since Handoff 2026-04-22-1

- `feea505`: recorded a control-thread dispatch checkpoint.
- `b8f200a`: fixed RC validation drift inside the existing RC boundary.
- `1cb9b22`: recorded RC publication follow-through after side-thread review.
- Session `019db449-896d-7e32-b29e-3f8afbea2f5a` is accepted as the origin of
  the current context-threshold governance scheme.
- V3/V4 remains expected to enter mainline later as a v0.2 overlay only after
  RC publication follow-through and an explicit scope gate.
- The user's Notion classic samples can become a bounded `hope-kb`-only package
  only after readable rows or an exported source are available.

## Allowed Work For The Next Control Thread

Allowed without reopening product scope:

- RC publication follow-through
- release confirmation notes
- baseline protection notes
- validation / benchmark recovery if a regression appears
- non-blocking polish that stays inside the current RC boundary
- docs-only handoff, checkpoint, or side-thread status alignment
- a bounded `hope-kb` classic-sample intake package only if source rows are
  readable and the control thread explicitly opens that KB-only gate

## Prohibited Work For The Next Control Thread

- do not add new frozen contracts
- do not modify the v0.1 17-sheet workbook
- do not modify exporter behavior
- do not modify IPC
- do not modify Rust DTOs or validators
- do not connect live Qwen
- do not connect Seedance
- do not merge `hope-kb` into `hope`
- do not promote V3/V4 proposal fields to engineering implementation before a
  v0.2 scope gate
- do not open another desktop package automatically
- do not expand controlled intake beyond its contract shell

## Validation / Benchmark Evidence

Recent product validation:

- `b8f200a` fixed RC validation drift in `app/src/runtime.rs` and
  `crates/export-engine/src/engine.rs`
- the post-fix workspace validation passed before
  `docs/rc-publication-follow-through-2026-04-22.md`

This handoff is docs-only and does not reopen the benchmark ladder.

## Open Risks / Blockers

- no active implementation blocker is recorded
- current risk is context quality; this handoff closes that risk by rotating the
  control thread
- Notion classic sample ingestion is blocked until readable rows or exported
  source files are available
- V3/V4 implementation remains closed until RC publication follow-through and a
  written v0.2 scope gate

## Exact Next Action

Open a fresh Hope control thread before formal next work.

The next thread should read Git state first, then choose the next bounded action
from the RC publication follow-through route. It should not dispatch new
implementation work unless it writes an explicit scope gate.

## Startup Prompt For New Control Thread

```text
This thread replaces the previous Hope control thread. Do not inherit old chat context. Treat the current Git state as the only fact source.

Read first:
- E:\codex\hope current branch, git status, latest control-thread-handoff, latest control-thread-checkpoint if any, nightly handoff, project-thread-startup, parallel-execution-appendix, control-thread-context-supervision, rc-publication-follow-through
- E:\codex\hope-kb current branch, git status, latest live-progress, nightly handoff, project-thread-startup, parallel-execution-appendix
- E:\codex\hope-desktop-shell current branch, git status, latest commit
- E:\codex\hope-intake-app current branch, git status, latest commit

Expected route:
- hope: RC_READY + scope freeze; allow only release confirmation, baseline protection, benchmark / validation recovery, and non-blocking polish
- hope-kb: independent KB hardening / standby; do not merge into hope yet
- V3/V4 fields: v0.2 overlay proposal planning input only; future mainline integration is expected only after RC publication follow-through and an explicit v0.2 scope gate
- desktop and intake: completed packages stay waiting unless the control thread explicitly opens a scope gate

Current expected anchors:
- hope: codex/contracts-freeze at the latest pushed commit after control-thread-handoff-2026-04-22-2
- hope-kb: codex/contracts-freeze at 6617434
- desktop: codex/desktop-shell at 66d687d
- intake: codex/controlled-intake-snapshot-bootstrap-app at d74bd11
- V3/V4 proposal: origin/codex/v3-field-overlay-proposal at 13f8d7d

Control-thread duties:
- review, schedule, judge boundaries, and write Git-visible handoff records only
- coordinate the five-thread model: Hope main, KB, V3 field, desktop, and intake
- do not directly expand product scope
- follow control-thread-context-supervision thresholds; at the red threshold, write a handoff and rotate before continuing
```

## Side-Thread Messages To Carry Forward

Hope main:

```text
Continue RC publication follow-through and baseline protection from the latest
pushed `codex/contracts-freeze` handoff. Do not reopen product scope.
```

Hope KB:

```text
Remain independent KB hardening / standby. A bounded classic-sample intake
package may open only after readable Notion rows or exported source rows are
available. Do not merge into Hope.
```

V3/V4:

```text
Stay proposal-only at `13f8d7d`. V3/V4 is expected to enter mainline later as a
v0.2 overlay after RC publication follow-through and a written scope gate.
```

Desktop:

```text
WriterReadiness / InputBoundary remains accepted and waiting at `66d687d`.
Do not open another desktop package without a new control-thread scope gate.
```

Controlled intake:

```text
Qwen/storyboard contract shell remains accepted and waiting at `d74bd11`.
Do not expand into live Qwen, prompt package generation, segment/cut handoff,
or runtime integration without a new control-thread scope gate.
```
