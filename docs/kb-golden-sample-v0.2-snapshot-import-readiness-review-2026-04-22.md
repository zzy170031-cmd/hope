# KB Golden Sample v0.2 Snapshot Import Readiness Review 2026-04-22

## Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- anchor before this memo: `ead8765` (`docs: add cross-device restart instructions`)
- reviewed repo: `E:\codex\hope-kb`
- reviewed branch: `codex/contracts-freeze`
- reviewed commit: `818c098` (`Add golden sample v0.2 snapshot import readiness`)
- route: `RC_READY` plus controlled v0.2 readiness planning
- review type: main-control acceptance of a KB-only snapshot-import readiness package

This memo is docs-only. It does not open Hope product implementation and does
not merge `hope-kb` into `hope`.

## Thread Labels

- `Hope总控-CrossDeviceRestart【KB SnapshotGate已接收】` | path: `E:\codex\hope` | branch: `codex/contracts-freeze` | anchor: `ead8765` before this memo | worktree: dirty only for this memo | boundary: control review and Git-visible handoff only
- `Hope-KB-V0.2SnapshotImportReadiness【主控已接收·待命】` | path: `E:\codex\hope-kb` | branch: `codex/contracts-freeze` | anchor: `818c098` | worktree: clean | boundary: KB-only snapshot/import assets, no Hope implementation
- `Hope-V3字段线程-GoldenSampleContract【归档待命】` | path: `E:\codex\hope` | branch: `origin/codex/v3-field-overlay-proposal` | anchor: `69c645c` | worktree: not checked out | boundary: freeze candidate only, no implementation
- `Hope桌面端-WriterReadiness【InputBoundary待命】` | path: `E:\codex\hope-desktop-shell` | branch: `codex/desktop-shell` | anchor: `66d687d` | worktree: clean | boundary: waiting, no live Qwen
- `Hope接入线程-Qwen分镜生成Contract【Contract边界待命】` | path: `E:\codex\hope-intake-app` | branch: `codex/controlled-intake-snapshot-bootstrap-app` | anchor: `d74bd11` | worktree: clean | boundary: waiting, no prompt package or runtime integration

## Reviewed KB Result

Reviewed commit `818c098` adds the snapshot-import readiness layer on top of
the accepted v0.2 schema package.

Changed files in `818c098`:

- `docs/golden-sample-v0.2-snapshot-import-readiness-2026-04-22.md`
- `docs/live-progress.md`
- `migrations/0002_golden_sample_v0_2.sql`
- `scripts/build-kb-snapshot.py`
- `seed/v0.2/import_map.json`
- `seed/v0.2/manifest.json`

The reviewed package keeps the builder default on `v0.1` and requires explicit
selection of `--version v0.2` for the new golden-sample snapshot.

## Local Verification

Verification was rerun on this machine after fast-forwarding the repos to the
remote anchors. The system `python` alias was unavailable, so the Codex bundled
Python runtime was used for snapshot commands.

Passed:

- v0.1 no-regression seed validation:
  `powershell -ExecutionPolicy Bypass -File E:\codex\hope-kb\scripts\validate-seed-bundle.ps1`
- v0.1 default snapshot build:
  `snapshots/hope-kb-v0.1.sqlite3`
  - snapshot version: `v0.1`
  - director cut samples: `35`
- v0.2 explicit snapshot build:
  `snapshots/hope-kb-v0.2.sqlite3`
  - snapshot version: `v0.2`
  - golden sample library: `40`
- v0.2 SQLite `quick_check`: `ok`

Verified v0.2 row counts:

- `golden_sample_library = 40`
- `golden_sample_field_coverage_rule = 5`
- `golden_sample_failure_mapping = 40`
- `golden_sample_repair_mapping = 40`
- `golden_sample_source = 3`
- `golden_sample_provenance = 1`

Manifest hash:

- `bundle-sha256:b62c565b7c048bb5e0d2623b438eb83e269471283e2b10fcdd7646ef06c83bf2`

## Acceptance Decision

`KB_GOLDEN_SAMPLE_V0_2_SNAPSHOT_IMPORT_READINESS_ACCEPTED`

The KB v0.2 snapshot-import readiness package is accepted by main control.

Accepted points:

- v0.2 import map has six bounded entries for golden sample library, coverage
  rules, failure mappings, repair mappings, sources, and provenance.
- v0.2 manifest records the expected counts and content hash.
- v0.2 migration defines the required snapshot tables without inventing
  `reference_control_core`.
- versioned snapshot builder support preserves the v0.1 default behavior.
- validation and local rebuild evidence match the KB readiness memo.
- no Hope product code, desktop code, intake code, Qwen, Seedance, v0.1 seed,
  v0.1 import map, or v0.1 manifest is opened by this package.

## Remaining Closed Gates

This acceptance does not authorize:

- Rust DTO changes
- validator implementation
- repair implementation
- exporter/debug metadata changes
- workbook or IPC changes
- desktop read-only provenance
- intake / Qwen retrieval work
- live Qwen or Seedance integration
- merging `hope-kb` into `hope`
- V3 implementation or direct promotion into Hope product code

`reference_control_core` remains absent and must not be invented from the
current golden-sample rows.

## Next Control State

The current five-thread state is standby with one accepted KB readiness packet:

- Hope stays `RC_READY` plus controlled v0.2 readiness planning.
- KB returns to checkpoint standby after the accepted v0.2 snapshot-import
  readiness package.
- V3 remains archived / waiting at `69c645c`.
- Desktop and intake remain waiting.
- Qwen and Seedance remain closed.

If the user supplies additional golden sample cases next, main control must open
a new bounded KB-only sample update gate first. Do not route new samples
directly into Hope product implementation.

Only after a future KB sample update is complete and accepted by main control
may V3 be rescheduled to update the proposal or freeze candidate.

关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。
