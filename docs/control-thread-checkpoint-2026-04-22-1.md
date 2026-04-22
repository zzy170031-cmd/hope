# Control Thread Checkpoint 2026-04-22-1

## Checkpoint Reason

The new Hope control thread has finished Git-first startup verification and the user requested advancing the next operations for each thread according to current project progress.

This checkpoint is docs-only. It records the next dispatch package without reopening product scope.

## Route

- date: 2026-04-22 17:58:08 +08:00
- timezone: Asia/Shanghai
- main repo: `E:\codex\hope`
- main branch: `codex/contracts-freeze`
- route: `RC_READY` plus scope freeze
- decision: `CONTROL_THREAD_DISPATCH_RC_PUBLICATION_BASELINE_PROTECTION`

The active route remains:

- publish / confirm the existing RC baseline
- protect the current baseline
- recover validation or benchmarks only if a regression appears
- allow non-blocking polish only inside the existing RC boundary
- keep side threads separate and waiting unless an explicit scope gate is opened

## Current Anchors

Hope main:

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor: `a79787b` (`docs: add control thread handoff for context rotation`)
- state: clean and aligned with `origin/codex/contracts-freeze`

Hope KB:

- repo: `E:\codex\hope-kb`
- branch: `codex/contracts-freeze`
- anchor: `6617434` (`docs: align KB live progress after desktop stable stop`)
- state: clean and aligned with `origin/codex/contracts-freeze`
- route: independent KB hardening / standby; no merge with `hope`

Desktop:

- repo: `E:\codex\hope-desktop-shell`
- branch: `codex/desktop-shell`
- anchor: `66d687d` (`desktop-shell: add writer qwen input readiness boundary`)
- state: clean and aligned with `origin/codex/desktop-shell`
- route: WriterReadiness / InputBoundary package complete and waiting

Controlled intake:

- repo: `E:\codex\hope-intake-app`
- branch: `codex/controlled-intake-snapshot-bootstrap-app`
- anchor: `d74bd11` (`qwen-storyboard: tighten contract boundary`)
- state: clean and aligned with `origin/codex/controlled-intake-snapshot-bootstrap-app`
- route: Qwen/storyboard contract shell complete and waiting

V3/V4 field proposal:

- branch: `origin/codex/v3-field-overlay-proposal`
- anchor: `13f8d7d` (`docs: add v3 field overlay proposal handoff`)
- state: v0.2 overlay proposal planning input only; no engineering implementation

## Dispatch Plan

### 1. Hope Main / Control Thread

Next operation:

- move to RC publication follow-through and baseline protection
- keep the current RC release packet and benchmark evidence as the release baseline
- write release confirmation / baseline notes only when they are needed for publication or handoff clarity
- rerun validation or benchmark ladders only if a product-code change or regression signal appears

Main controller responsibilities:

- remain the only milestone owner and boundary judge
- keep side-thread outputs as waiting packets unless a scope gate is explicitly opened
- keep allowed / prohibited actions Git-visible
- apply `docs/control-thread-context-supervision-2026-04-22.md`; write another checkpoint at yellow threshold and a handoff at red threshold

### 2. Hope KB Thread

Next operation:

- stay in independent hardening / standby
- do not merge into `hope`
- do not start a new KB hardening package from desktop WriterReadiness or V3/V4 proposal work
- if the main control thread explicitly opens a KB package, use the existing 1 KB integration owner plus 4 lane model:
  - Lane 1: source register / source policy / provenance review
  - Lane 2: scene taxonomy / director mappings / alias normalization
  - Lane 3: failure patterns / repair mappings / prompt hardening / degraded-input coverage
  - Lane 4: classic examples / export templates / validation / snapshot / runtime handoff
- update `E:\codex\hope-kb\docs\live-progress.md` after each 10-20 minute KB work package

### 3. Desktop Thread

Next operation:

- remain waiting at WriterReadiness / InputBoundary stable stop
- do not connect live Qwen
- do not open another desktop package automatically
- if a future scope gate opens, treat the current desktop branch as a bounded readiness package only, not as permission to alter Hope RC behavior

### 4. Controlled Intake Thread

Next operation:

- remain waiting at Qwen/storyboard contract shell stable stop
- do not expand controlled intake beyond the existing contract shell
- do not treat intake DTO or UI placeholders as a live runtime integration gate

### 5. V3/V4 Field Overlay Proposal

Next operation:

- keep `origin/codex/v3-field-overlay-proposal` as v0.2 overlay proposal planning input only
- do not promote any V3/V4 field to frozen contract, workbook change, exporter behavior, validator behavior, IPC, or Rust DTO work
- evaluate it only after RC publication follow-through and an explicit scope reopen

## Allowed Work

Allowed without reopening scope:

- release publication follow-through
- release confirmation notes
- baseline protection notes
- validation / benchmark recovery if a regression appears
- non-blocking polish inside the existing RC boundary
- docs-only side-thread status alignment
- docs-only checkpoint or handoff records

## Prohibited Work

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

No product-code files changed in this dispatch checkpoint.

No benchmark or validation ladder is reopened by this checkpoint because it is docs-only and records scheduling / boundary decisions only.

Latest known verification evidence remains the existing RC chain plus the handoff-recorded desktop checks:

- RC chain: 45s, 10min, 30min, 60min benchmark, regression ladder stability, real-project regression, external-consumption verification
- desktop `npm run typecheck`: pass
- desktop `npm run build`: pass

## Side Thread Messages

Use these short messages when updating active side threads.

Hope main:

```text
Continue from RC_READY plus scope freeze. Next operation is RC publication follow-through and baseline protection only. Do not reopen product scope. Rerun validation or benchmarks only if a regression signal or product-code change appears.
```

Hope KB:

```text
Remain on independent KB hardening / standby. Do not merge into hope and do not start a new hardening package unless the control thread explicitly opens that scope gate. Keep live-progress current for any future 10-20 minute KB work package.
```

Desktop:

```text
WriterReadiness / InputBoundary is accepted as complete and waiting. Do not connect live Qwen or open another desktop package unless the control thread explicitly opens a scope gate.
```

Controlled intake:

```text
Qwen/storyboard contract shell is accepted as complete and waiting. Do not expand intake beyond the current contract shell unless the control thread explicitly opens a scope gate.
```

V3/V4 proposal:

```text
Remain proposal-only planning input for v0.2 overlay. Do not implement fields, contracts, workbook changes, exporter behavior, validators, IPC, or DTOs until RC publication follow-through is complete and scope is explicitly reopened.
```

## Exact Next Action

Proceed with RC publication follow-through / baseline protection from `E:\codex\hope` while keeping all side packages waiting.

If the user asks for the next concrete artifact, produce a docs-only RC publication confirmation note. If the user asks to reopen scope, first write a scope-gate memo that states what is being opened and what remains closed.
