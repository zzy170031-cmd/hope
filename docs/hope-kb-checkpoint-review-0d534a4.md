# hope Main-Thread Review Memo for hope-kb Checkpoint 0d534a4

## Route

This memo is a docs-only `hope` main-thread review under the `RC_READY` freeze on `codex/contracts-freeze`.

It does not reopen merge-readiness, it does not authorize implementation, and it does not imply a repo merge with `hope-kb`.

## Review Object

- reviewed checkpoint: `0d534a4` (`Harden exporter runtime payload edge cases`)
- reviewed branch: `codex/contracts-freeze`
- reviewed bundle hash: `bundle-sha256:5f042c10ada3726bdbd71d5f4cbad2187b3895dd85e1e5195d6938a3a22d20b6`
- reviewed inputs:
  - `E:\codex\hope-kb\docs\hope-main-thread-review-packet-0d534a4.md`
  - `E:\codex\hope-kb\docs\runtime-consume-integration-packet-v0.1.md`
  - `E:\codex\hope-kb\seed\v0.1\manifest.json`
  - validator result: passed
  - snapshot result: `snapshots/hope-kb-v0.1.sqlite3` matches the same bundle hash and expected record counts

## Reviewed Surfaces

This review covered these 5 runtime consume surfaces only:

1. `snapshot_bootstrap`
2. `segment_and_cut_projection`
3. `handoff_projection`
4. `prompt_package_projection`
5. `validation_feedback_projection`

## Current hope-Side Status

Current `hope` main-thread status remains compatible with future controlled intake of this snapshot contract, but most of the consume path is not yet formally productized.

- `snapshot_bootstrap`
  `hope` still has bootstrap-era shortcut behavior in `crates/project-store/src/kb_runtime.rs`. The current handle uses `snapshot_hash = "runtime-unverified"` and derives knowledge reads from the local seed bundle path. This is not yet a full validated-snapshot consumer.
- `segment_and_cut_projection`
  `hope` main-thread focused evidence already keeps runtime inputs authoritative, and taxonomy variants are constrained to metadata visibility. However, future intake must not treat exporter-local helpers as a substitute for explicit segment and cut mappings from the snapshot contract.
- `handoff_projection`
  Current `hope` runtime behavior does not show a productized snapshot-backed handoff projection yet. Future intake must preserve explicit handoff payloads and must not collapse empty `buffer_cut_ids` into a safe default.
- `prompt_package_projection`
  `crates/export-engine/src/engine.rs` still contains exporter-local prompt assembly, including negative/default prompt fill and face-cleanup negatives. That is acceptable as current RC exporter behavior, but it is not a compliant future snapshot-consume path.
- `validation_feedback_projection`
  `crates/validators/src/contract.rs` still derives decision state from `severity` and exports a slim `ValidationReportRow`. This means richer blocking-aware payload fields are not yet preserved as a formal runtime consume surface.

The important boundary is that these are future-integration guardrails, not evidence that the current RC main thread has already implemented the wrong snapshot consumer.

## Guardrails For Future Controlled Intake

When `hope` later schedules controlled intake, it should continue to prohibit these local fallback or default patterns:

- no local sheet alias fallback in place of the Validation sheet contract
- no local blocking inference from `severity` alone when richer validator payload exists
- no local `render_segment` backfill or ID derivation used as a substitute for explicit snapshot mappings
- no local handoff safe default that rewrites empty `buffer_cut_ids`
- no local negative/default prompt fill used as a substitute for a snapshot `PromptPackage`, including intentionally blank `negative_prompt` payloads

## Final Decision

`PASS_WITH_GUARDRAILS_KEEP_SEPARATE`

The `0d534a4` checkpoint remains acceptable as a future auxiliary knowledge snapshot input for `hope`, provided the intake stays controlled and the guardrails above remain explicit.

## Next Action

Keep `hope` and `hope-kb` separate.

Wait for later controlled intake scheduling under `hope` main-thread ownership.

Do not implement product hookup, do not reopen merge-readiness, and do not merge repos from this memo.
