# RC Publication Follow-Through 2026-04-22

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- current anchor: `b8f200a` (`Fix RC validation drift`)
- route: `RC_READY` plus scope freeze
- decision: `RC_PUBLICATION_FOLLOW_THROUGH_CONTINUE`

This memo is docs-only. It records the current control-thread alignment after
reviewing the active side-thread sessions and the latest Git state.

It does not reopen product scope, does not modify the frozen v0.1 workbook, and
does not merge any side thread into `hope`.

## Current Conclusion

Hope remains in RC publication follow-through and baseline protection.

The latest `b8f200a` change is validation recovery inside the existing RC
boundary. It does not authorize feature expansion. The current next work is
publication confirmation, baseline protection, and bounded recovery only if a
new validation or benchmark signal appears.

## Current Git Anchors

Hope main:

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor: `b8f200a` (`Fix RC validation drift`)
- state: clean and aligned with `origin/codex/contracts-freeze`

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
- state: clean and aligned with `origin/codex/controlled-intake-snapshot-bootstrap-app`
- role: Qwen/storyboard contract shell complete and waiting

V3/V4 proposal:

- branch: `origin/codex/v3-field-overlay-proposal`
- anchor: `13f8d7d` (`docs: add v3 field overlay proposal handoff`)
- state: v0.2 overlay proposal planning input only

## Side-Thread Session Alignment

Reviewed side-thread sessions:

- V3 field thread: `019db40b-4299-75c3-8ea7-d6b1ff3a8175`
- KB thread: `019da89f-49a2-7e11-bd2f-c0138165fdd9`
- controlled intake thread: `019daa9c-1955-7852-92ea-e9a2139acaef`
- desktop thread: `019daa78-87fb-7380-a8be-c19a7bf168b7`

Confirmed operating pattern:

- the control thread schedules, reviews, gates, and records handoff state
- child threads own bounded packages and report back
- implementation packages require explicit scope gate approval
- accepted packages stop at stable-stop and wait
- Git state is the fact source, not chat memory

## V3/V4 Future Integration Node

V3/V4 is not a permanent side path.

Current state:

- proposal-only planning input
- no exporter, IPC, Rust DTO, validator, desktop UI, or workbook implementation
- no Seedance integration

Future integration may be reopened only after:

- RC publication follow-through is complete
- the control thread writes an explicit v0.2 scope-gate memo
- field ownership is confirmed across `hope`, desktop, exporter, validator, and KB
- the v0.1 17-sheet workbook remains protected unless a new contract is explicitly opened

Future model boundary remains:

- `content_generation_model`: Qwen / qwen-compatible text generation
- `target_video_model`: Seedance 2.0 as downstream execution target
- `adapter_profile`: Seedance adapter / payload compiler profile

## KB Classic Sample Intake Status

The user has provided a Notion source for additional classic samples intended
for `hope-kb`.

The control-thread decision is:

- this can become a bounded `hope-kb`-only package
- it must not merge into `hope`
- it must not alter Hope RC criteria
- it must not invent sample content
- it requires readable Notion rows or an exported Markdown / CSV source before ingestion

Expected KB package shape, once source rows are available:

- append new records to `seed/v0.1/classic_case_examples.json`
- update `seed/v0.1/source_register.json`
- update manifest counts and bundle hash
- refresh `docs/live-progress.md`
- run seed bundle validation and snapshot build
- push a separate `hope-kb` commit

## Allowed Next Work

Allowed now:

- RC publication follow-through notes
- release confirmation notes
- baseline protection notes
- validation / benchmark recovery if a new signal appears
- non-blocking polish inside the current RC boundary
- docs-only side-thread status alignment
- KB classic sample intake only if the control thread explicitly opens the KB-only package and the source rows are readable

## Prohibited Work

- do not add new frozen contracts
- do not modify the v0.1 17-sheet workbook
- do not modify exporter behavior
- do not modify IPC
- do not modify Rust DTOs or validators
- do not connect live Qwen
- do not connect Seedance
- do not merge `hope-kb` into `hope`
- do not promote V3/V4 proposal fields to engineering implementation
- do not open another desktop package automatically
- do not expand controlled intake beyond its contract shell

## Validation Evidence

Recent RC recovery:

- `b8f200a` fixed RC validation drift in `app/src/runtime.rs` and `crates/export-engine/src/engine.rs`
- the post-fix workspace validation passed before this memo

No validation ladder is reopened by this memo because this memo is docs-only.

Current command evidence:

```text
git -C E:\codex\hope status --short --branch
## codex/contracts-freeze...origin/codex/contracts-freeze

git -C E:\codex\hope log -1 --oneline --decorate
b8f200a (HEAD -> codex/contracts-freeze, origin/codex/contracts-freeze) Fix RC validation drift
```

## Dispatch Messages

Hope main:

```text
Continue RC publication follow-through and baseline protection from `b8f200a`.
Do not reopen product scope. Validation or benchmark work is allowed only if a
new regression signal appears.
```

Hope KB:

```text
Remain independent KB hardening / standby. A bounded classic-sample intake
package may be opened only after the Notion rows or exported source rows are
readable. Do not merge into hope.
```

Desktop:

```text
WriterReadiness / InputBoundary remains accepted and waiting at `66d687d`.
Do not open another package without a new control-thread scope gate.
```

Controlled intake:

```text
Qwen/storyboard contract shell remains accepted and waiting at `d74bd11`.
Do not expand into live Qwen, prompt package generation, segment/cut handoff, or
runtime integration.
```

V3/V4:

```text
Remain v0.2 overlay proposal planning input at `13f8d7d`. Future integration is
expected, but only after RC publication follow-through and an explicit v0.2
scope gate.
```

## Exact Next Action

Proceed from `E:\codex\hope` with RC publication follow-through and baseline
protection.

Keep all side threads waiting unless the control thread explicitly opens one of
these gates:

- KB-only classic sample intake from readable Notion/exported source rows
- v0.2 overlay scope gate after RC publication follow-through
- targeted validation / benchmark recovery after a new regression signal
