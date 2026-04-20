# Hope Runtime Consume Intake Schedule 2026-04-20

## Route

This memo is a docs-only intake schedule for the `hope` main thread under the current `RC_READY` freeze on `codex/contracts-freeze`.

It does not authorize implementation, it does not reopen merge-readiness, and it does not imply a merge with `hope-kb`.

## Scheduling Goal

Define the future controlled intake order for the validated `hope-kb` snapshot contract after review checkpoint `0d534a4`.

Current `hope` status is still:

- main-thread release baseline stays frozen
- most runtime consume surfaces are not yet formally productized
- any future intake must happen as bounded `hope`-owned work, not as cross-repo blending

## Planned Intake Order

1. `snapshot_bootstrap`
2. `validation_feedback_projection`
3. `segment_and_cut_projection`
4. `handoff_projection`
5. `prompt_package_projection`

This order keeps the future intake path conservative:

- trust the validated snapshot before deeper projection work
- preserve validator semantics before consuming richer runtime payloads
- delay prompt-package adoption until the latest stage because it has the highest risk of local default bleed-through

## Global Guardrails

Every surface below inherits the same blocked local behaviors:

- no local sheet alias fallback
- no local blocking inference
- no local `render_segment` backfill
- no local handoff safe default
- no local negative/default prompt fill

These guardrails are future intake boundaries. They do not mean the current RC branch has already productized the wrong behavior.

## Surface Plan

### 1. `snapshot_bootstrap`

**Current status**

`hope` still bootstraps KB runtime state through a lightweight local path and is not yet a full validated-snapshot consumer.

**Guardrails**

- no local sheet alias fallback
- no local blocking inference
- no local `render_segment` backfill
- no local handoff safe default
- no local negative/default prompt fill

**Entry conditions**

- reviewed bundle hash stays pinned to `bundle-sha256:5f042c10ada3726bdbd71d5f4cbad2187b3895dd85e1e5195d6938a3a22d20b6` or a later explicitly re-reviewed checkpoint
- `manifest.json`, validator result, and snapshot result remain mutually consistent
- `hope` implementation plan explicitly names `snapshot_meta`, `export_template`, `failure_pattern`, `degraded_input_example`, and `runtime_consume_contract` as trusted bootstrap inputs

**Exit conditions**

- bootstrap reads from the validated snapshot contract instead of repo-local guess paths
- bootstrap does not substitute Validation-sheet aliases for the declared contract
- bootstrap preserves the reviewed bundle identity as an explicit runtime input

### 2. `validation_feedback_projection`

**Current status**

`hope` currently derives block state from `severity` and exports a slim validation row, so the richer replayable payload is not yet formally consumed.

**Guardrails**

- no local sheet alias fallback
- no local blocking inference
- no local `render_segment` backfill
- no local handoff safe default
- no local negative/default prompt fill

**Entry conditions**

- bootstrap intake path is already defined and trusted
- implementation plan explicitly names `status`, `severity`, `message`, `related_cut_or_segment`, and `is_blocking` as preserved fields
- no shortcut proposal is allowed that recreates validator semantics from `severity` alone

**Exit conditions**

- validation feedback preserves blocking-aware payload fields without collapsing them into a local severity-only decision
- replayable validation context remains intact for downstream runtime consumption
- `hope` does not infer blocking semantics that contradict the snapshot contract

### 3. `segment_and_cut_projection`

**Current status**

`hope` focused tests already show runtime-authoritative behavior against knowledge-side overlap, but the snapshot-backed projection itself is not yet implemented.

**Guardrails**

- no local sheet alias fallback
- no local blocking inference
- no local `render_segment` backfill
- no local handoff safe default
- no local negative/default prompt fill

**Entry conditions**

- bootstrap and validation feedback intake order is already fixed
- projection plan names `RenderSegments` and `Cuts` as explicit snapshot-driven inputs
- continuity and column-source expectations are traced back to the reviewed contract instead of exporter-local helpers

**Exit conditions**

- runtime consumption does not fabricate missing `render_segment` links locally
- runtime-authoritative duration and shot-window behavior remain unchanged
- taxonomy continues to affect metadata visibility only, not runtime-authoritative output

### 4. `handoff_projection`

**Current status**

`hope` does not yet expose a fully productized snapshot-backed handoff payload with transition, buffer, and continuity context.

**Guardrails**

- no local sheet alias fallback
- no local blocking inference
- no local `render_segment` backfill
- no local handoff safe default
- no local negative/default prompt fill

**Entry conditions**

- segment and cut projection boundaries are already defined
- handoff plan explicitly preserves pair, director, transition, buffer, and continuity fields from the reviewed contract
- no proposal is allowed that downgrades empty `buffer_cut_ids` into a safe default

**Exit conditions**

- handoff payload remains explicit and snapshot-driven
- empty or partial handoff data does not silently rewrite the boundary contract
- runtime handoff remains stable without introducing local safety guesses

### 5. `prompt_package_projection`

**Current status**

This is the highest-risk intake step because `hope` still has exporter-local prompt assembly and negative/default prompt fill in the current RC branch.

**Guardrails**

- no local sheet alias fallback
- no local blocking inference
- no local `render_segment` backfill
- no local handoff safe default
- no local negative/default prompt fill

**Entry conditions**

- the first four surfaces already have an approved intake design
- prompt package plan explicitly treats snapshot `PromptPackage` as authoritative
- intentionally blank `negative_prompt` payloads are supported without local refill logic

**Exit conditions**

- runtime consumption reads the prompt package from the snapshot contract instead of rebuilding it from local defaults
- negative/default prompt injection is not used as a substitute for the reviewed payload
- runtime-authoritative prompt layers remain stable while taxonomy remains metadata-only

## Next Action

Keep `hope` in `RC_READY` freeze.

Use this memo only as scheduling guidance for later controlled intake planning.

Do not implement product hookup, do not reopen merge-readiness, and do not merge with `hope-kb` from this stage.
