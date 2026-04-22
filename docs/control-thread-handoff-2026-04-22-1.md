# Control Thread Handoff 2026-04-22-1

## Handoff Reason

This handoff is created because the current Hope control thread has reached the red threshold defined by:

- `docs/control-thread-context-supervision-2026-04-22.md`

Observed trigger:

- the current control thread has already gone through context compaction / summary recovery

Required action:

- stop opening new work in this thread
- push this handoff
- continue formal work in a new Hope control thread

## Route

- date: 2026-04-22
- timezone: Asia/Shanghai
- main repo: `E:\codex\hope`
- main branch: `codex/contracts-freeze`
- route: `RC_READY` plus scope freeze
- handoff type: control-thread rotation / context protection

The source of truth for the next thread is Git, not this chat.

## Current Anchors

Hope main:

- branch: `codex/contracts-freeze`
- anchor before this handoff: `0db2a43` (`docs: add control thread context supervision`)
- status before this handoff: clean and aligned with `origin/codex/contracts-freeze`

Hope KB:

- repo: `E:\codex\hope-kb`
- branch: `codex/contracts-freeze`
- anchor: `6617434` (`docs: align KB live progress after desktop stable stop`)
- status: clean and aligned with `origin/codex/contracts-freeze`

Desktop:

- repo: `E:\codex\hope-desktop-shell`
- branch: `codex/desktop-shell`
- anchor: `66d687d` (`desktop-shell: add writer qwen input readiness boundary`)
- state: WriterReadiness / InputBoundary complete and waiting

Controlled intake:

- repo: `E:\codex\hope-intake-app`
- branch: `codex/controlled-intake-snapshot-bootstrap-app`
- anchor: `d74bd11` (`qwen-storyboard: tighten contract boundary`)
- state: Qwen/storyboard contract shell only, clean and waiting

V3/V4 field proposal:

- branch: `origin/codex/v3-field-overlay-proposal`
- anchor: `13f8d7d` (`docs: add v3 field overlay proposal handoff`)
- state: v0.2 overlay proposal planning input only, no engineering implementation

## Accepted Decisions Since Previous Handoff

- `1ffe416`: V3/V4 field overlay governance review accepted as docs-only v0.2 planning input.
- `c82160f`: desktop WriterReadiness stable-stop accepted and kept separate.
- `6617434` in `hope-kb`: KB live-progress aligned to independent hardening / standby.
- `baebf39`: RC baseline protection sync recorded all side threads as waiting.
- `0db2a43`: control-thread context supervision activated.

## Allowed Work For Next Thread

Allowed without reopening product scope:

- release publication follow-through
- release confirmation notes
- baseline protection notes
- validation / benchmark recovery if a regression appears
- non-blocking polish that stays inside the current RC boundary
- docs-only handoff, checkpoint, or side-thread status alignment

## Prohibited Work For Next Thread

- do not add new frozen contracts
- do not modify the v0.1 17-sheet workbook
- do not modify exporter behavior
- do not modify IPC
- do not modify Rust DTOs or validators
- do not connect live Qwen
- do not connect Seedance
- do not merge `hope-kb` into `hope`
- do not promote V3/V4 field proposal to engineering implementation
- do not open another desktop package automatically
- do not expand controlled intake beyond its contract shell

## Validation / Benchmark Evidence

No product-code files changed after the RC release packet in this control-thread rotation round.

Recent verification rerun:

- desktop `npm run typecheck`: pass
- desktop `npm run build`: pass

The RC benchmark ladder is not reopened by this handoff because this handoff is docs-only.

## Open Risks / Blockers

- no active implementation blocker
- current risk is context quality only; this handoff closes that risk by rotating the control thread
- merge-readiness remains closed
- V3/V4 implementation remains closed until explicit scope reopen

## Exact Next Action

Open a new Hope control thread before formal next work.

The new thread must read Git state first, then continue from `RC_READY` publication follow-through / baseline protection.

## Startup Prompt For New Control Thread

```text
This thread replaces the previous Hope control thread. Do not inherit old chat context. Treat the current Git state as the only fact source.

Read first:
- E:\codex\hope current branch, git status, latest control-thread-handoff, latest control-thread-checkpoint if any, nightly handoff, project-thread-startup, parallel-execution-appendix, control-thread-context-supervision
- E:\codex\hope-kb current branch, git status, latest live-progress, nightly handoff, project-thread-startup, parallel-execution-appendix
- E:\codex\hope-desktop-shell current branch, git status, latest commit
- E:\codex\hope-intake-app current branch, git status, latest commit

Expected route:
- hope: RC_READY + scope freeze; allow only release confirmation, baseline protection, benchmark / validation recovery, and non-blocking polish
- hope-kb: independent KB hardening / standby; do not merge into hope yet
- V3/V4 fields: v0.2 overlay proposal planning input only; no engineering implementation yet
- desktop and intake: completed packages stay waiting unless the control thread explicitly opens a scope gate

Current expected anchors:
- hope: codex/contracts-freeze at the latest pushed commit after control-thread-handoff-2026-04-22-1
- hope-kb: codex/contracts-freeze at 6617434
- desktop: codex/desktop-shell at 66d687d
- intake: codex/controlled-intake-snapshot-bootstrap-app at d74bd11
- V3/V4 proposal: origin/codex/v3-field-overlay-proposal at 13f8d7d

Control-thread duties:
- review, schedule, judge boundaries, and write Git-visible handoff records only
- do not directly expand product scope
- follow control-thread-context-supervision thresholds; at the red threshold, write a handoff and rotate to a new thread before continuing
```
