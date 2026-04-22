# Control Thread Context Supervision 2026-04-22

## Route

This memo adds a docs-only supervision rule for the Hope control thread.

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor before this memo: `baebf39` (`docs: record RC baseline protection sync`)
- route: `RC_READY` plus scope freeze
- purpose: keep the main control thread fast, reconstructable, and Git-backed

This memo does not reopen product scope, does not authorize implementation, and does not merge any side thread.

## Decision

`CONTROL_THREAD_CONTEXT_SUPERVISION_ACTIVE`

The Hope main control thread must actively monitor its own context size and rotate before forced compaction or slow recall starts to damage development speed.

The source of truth remains Git, not chat history.

## Supervision Rule

At the start and end of each control-thread work block, and after each side-thread report, the main controller should check:

- can the current route be restated from Git docs without relying on chat scrollback?
- are the current anchors for `hope`, `hope-kb`, desktop, intake, and V3/V4 proposal Git-visible?
- are current allowed / prohibited actions written in a committed memo?
- has the thread accumulated enough screenshots, long reports, or decisions that the next model turn may become sluggish?
- has a system summary, forced compaction, or visible loss of recent detail occurred?

If any answer suggests context risk, use the thresholds below.

## Yellow Threshold: Write Checkpoint

Trigger a control-thread checkpoint when any of these happens:

- more than two meaningful control decisions have occurred since the latest control sync memo
- two or more side threads reported back after the last memo
- a new scope gate is about to be opened
- the user reports lag, slowness, or repeated context confusion
- the controller needs old chat scrollback to answer a basic status question
- the current next step cannot be explained in under one minute from committed docs and `git status`

Required action:

- stop new scope work temporarily
- write a docs-only checkpoint memo under `docs/`
- include current branches, commits, clean/dirty state, active route, side-thread labels, allowed work, prohibited work, and the next intended action
- push the checkpoint before continuing

Suggested file name:

```text
docs/control-thread-checkpoint-YYYY-MM-DD-N.md
```

## Red Threshold: Rotate Thread

Rotate to a new control thread when any of these happens:

- a forced summary / compaction has already occurred
- the model sees a summary instead of the full recent thread
- the thread becomes materially slow during normal control work
- current facts exist only in chat and not in Git docs
- the controller cannot confidently distinguish old instructions from the newest request
- a new engineering phase is about to start after a long supervision phase
- the user explicitly asks to refresh, rotate, or start a new main control thread

Required action:

- stop opening new work in the current thread
- write a docs-only handoff memo
- push it
- give the user a startup prompt for the new control thread
- mark the old thread as read-only / archived by practice

Suggested file name:

```text
docs/control-thread-handoff-YYYY-MM-DD-N.md
```

## Required Handoff Contents

Every control-thread handoff must include:

- current date and local timezone
- current route and release state
- latest `hope` branch, clean/dirty state, and commit
- latest `hope-kb` branch, clean/dirty state, and commit
- latest desktop branch, clean/dirty state, and commit if relevant
- latest intake branch, clean/dirty state, and commit if relevant
- latest proposal branch anchors if any proposal is being supervised
- accepted decisions since the previous handoff
- explicit allowed work
- explicit prohibited work
- validation / benchmark evidence and whether it was rerun
- open risks and blockers
- exact next action for the new control thread
- exact message to send to active side threads, if any

## New Control Thread Startup Prompt

Use this prompt when rotating:

```text
This thread replaces the previous Hope control thread. Do not inherit old chat context. Treat the current Git state as the only fact source.

Read first:
- E:\codex\hope current branch, git status, latest control-thread-handoff / control-thread-checkpoint, nightly handoff, project-thread-startup, parallel-execution-appendix, control-thread-context-supervision
- E:\codex\hope-kb current branch, git status, latest live-progress, nightly handoff, project-thread-startup, parallel-execution-appendix

Expected route:
- hope: RC_READY + scope freeze; allow only release confirmation, baseline protection, benchmark / validation recovery, and non-blocking polish
- hope-kb: independent KB hardening / standby; do not merge into hope yet
- V3/V4 fields: v0.2 overlay proposal planning input only; no engineering implementation yet
- desktop and intake: completed packages stay waiting unless the control thread explicitly opens a scope gate

Control-thread duties:
- review, schedule, judge boundaries, and write Git-visible handoff records only
- do not directly expand product scope
- follow control-thread-context-supervision thresholds; at the red threshold, write a handoff and rotate to a new thread before continuing
```

## Current Anchors At Activation

- `hope`: `baebf39` (`docs: record RC baseline protection sync`)
- `hope-kb`: `6617434` (`docs: align KB live progress after desktop stable stop`)
- desktop: `66d687d` (`desktop-shell: add writer qwen input readiness boundary`)
- intake: `d74bd11` (`qwen-storyboard: tighten contract boundary`)
- V3/V4 proposal: `13f8d7d` (`docs: add v3 field overlay proposal handoff`)

## Current Conclusion

The context supervision mechanism is active immediately after this memo.

Before any formal scope reopening, the controller must either:

- confirm the current thread is still under the yellow threshold, or
- write a checkpoint / handoff and rotate before starting the next work package.
