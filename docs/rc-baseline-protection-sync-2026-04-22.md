# RC Baseline Protection Sync 2026-04-22

## Route

This memo records the post-thread-sync state after the desktop Writer readiness stable stop and KB live-progress alignment.

- main repo: `E:\codex\hope`
- main branch: `codex/contracts-freeze`
- main branch anchor before this memo: `c82160f` (`docs: record desktop writer readiness stable stop`)
- release route: `RC_READY` plus scope freeze
- review type: release confirmation / baseline protection sync only

This memo is docs-only. It does not reopen implementation scope and does not merge side threads into the Hope main repo.

## Decision

`RC_BASELINE_PROTECTION_CONTINUE_NO_SCOPE_REOPEN`

The current Hope route remains:

- keep the RC baseline protected
- keep side threads separate
- keep V3/V4 field material as proposal-only planning input
- allow only release confirmation, validation / benchmark recovery, and non-blocking polish

## Current Anchors

Main Hope:

- branch: `codex/contracts-freeze`
- current anchor: `c82160f` before this memo
- latest control decisions:
  - `1ffe416`: V3/V4 field overlay accepted as docs-only v0.2 planning input
  - `c82160f`: desktop WriterReadiness stable-stop accepted and kept separate

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

KB:

- repo: `E:\codex\hope-kb`
- branch: `codex/contracts-freeze`
- anchor: `6617434` (`docs: align KB live progress after desktop stable stop`)
- state: independent hardening / standby, live-progress aligned, no merge with `hope`

V3/V4 field proposal:

- reviewed remote branch: `origin/codex/v3-field-overlay-proposal`
- anchor: `13f8d7d`
- state: docs-only v0.2 overlay proposal planning input, not implementation scope

## Protected Baseline Meaning

The RC baseline still traces to the existing release confirmation packet and frozen benchmark ladder.

No source-code files changed in this sync round. The only new work after the RC packet is governance / boundary documentation:

- V3/V4 proposal governance review
- desktop WriterReadiness stable-stop memo
- KB live-progress alignment
- this RC baseline protection sync memo

Because no runtime, exporter, validator, contract, fixture, or workbook files changed in this sync round, the RC benchmark ladder is not re-opened by this memo.

## Still Prohibited

- do not add new frozen contracts
- do not modify the v0.1 17-sheet workbook
- do not modify exporter behavior
- do not modify IPC
- do not modify Rust DTOs or validators
- do not connect live Qwen
- do not connect Seedance
- do not merge `hope-kb` into `hope`
- do not promote the V3/V4 field proposal to implementation
- do not open another desktop package automatically

## Allowed Next Work

Allowed without reopening product scope:

- release confirmation notes
- baseline protection notes
- validation / benchmark recovery if a regression appears
- non-blocking polish that stays inside the existing RC boundary
- docs-only handoff or status alignment for side threads

Anything beyond this list requires an explicit scope gate.

## Next Step

Move the main Hope thread to RC publication follow-through:

- keep the current branches clean
- keep side threads waiting
- keep V3/V4 as proposal-only
- do not begin engineering implementation until the user explicitly reopens scope after RC publication follow-through
