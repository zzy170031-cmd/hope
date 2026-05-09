# Hope Provider Failover Contract

Status: executable contract boundary for provider/model failover and QA
classification.

Scope: text-model calls used by `扩写故事`, `改写剧本`, storyboard generation,
UI-driven targeted QA, `full16`, and `formal403`.

This document is docs-only. It does not authorize runtime, UI, runner,
certifier, matrix, package, commit, or push work.

## 1. Model Order

The fixed provider model order for this contract is:

```text
primary_model=qwen3.6-plus
fallback_model=qwen3.6-plus-2026-04-02
```

No other model may be silently inserted into this two-model sequence.

Compatibility aliases, historical gate models, or local candidates may be
reported as reference-only context, but they cannot satisfy this failover gate
unless a controller dispatch explicitly opens a model-order change.

## 2. Allowed Failover Reasons

The runtime may switch from `primary_model` to `fallback_model` only for provider
availability failures:

- provider quota exhausted
- model unavailable
- entitlement / permission HTTP 403

Allowed evidence categories:

- `quota_exhausted`
- `model_unavailable`
- `entitlement_http_403`
- `permission_http_403`

The raw provider response body must not be written to docs, logs, chat, compact
QA evidence, or exported artifacts.

## 3. Forbidden Failover Success Reasons

Provider failover must not turn a content or contract failure into a pass.

The following reasons cannot be rescued by switching models:

- validator failed
- source grounding failed
- rows mismatch
- prompt boundary failed
- KB leakage
- visual_description abstract or placeholder-like
- CDP failure
- stale release exe
- stale QA artifact
- UI row mismatch
- missing StoryFactFrame
- missing accepted snapshot

If any of these happens, the result is a hard gate failure or blocked evidence,
not successful provider failover.

## 4. Trace Fields

Every provider-gate artifact must include these sanitized fields:

```text
primary_model
fallback_model
attempted_models
final_model
provider_failover_used
provider_failover_reason
raw_primary_http_403_seen
provider_http_403_handled_by_model_failover
no_unhandled_http_403
fallback_used
local_candidate
```

Field semantics:

- `primary_model` must equal `qwen3.6-plus`.
- `fallback_model` must equal `qwen3.6-plus-2026-04-02`.
- `attempted_models` must preserve actual order.
- `final_model` must be the model that produced the accepted provider output.
- `provider_failover_used=true` only when the provider switched from primary to
  fallback for an allowed provider availability reason.
- `provider_failover_reason` must be one of the allowed categories or empty
  when no provider failover occurred.
- `raw_primary_http_403_seen=true` is a sanitized boolean only.
- `provider_http_403_handled_by_model_failover=true` only when the primary
  failure was an entitlement/permission HTTP 403 and the fallback model produced
  valid provider output that still passes all non-provider validators.
- `no_unhandled_http_403=true` means no HTTP 403 remains outside the allowed
  provider-failover path.
- `fallback_used=false` is mandatory for live QA pass and means no local,
  deterministic, canned, or validator-repair fallback was accepted as provider
  success.
- `local_candidate=false` is mandatory for live QA pass.

Important distinction:

- `provider_failover_used` describes provider model switching.
- `fallback_used` describes local/content fallback.
- A valid provider model switch may have `provider_failover_used=true` while
  still requiring `fallback_used=false` and `local_candidate=false`.
- Source-bound repair of a provider response is not provider failover and must
  not be reported as local fallback when all of these are true: provider HTTP
  succeeded, the raw provider text is not exposed, repair is derived only from
  the current accepted source / scene / duration / KB oracle, the warning code
  is a `_repaired` code, and trace still reports `fallback_used=false`,
  `local_candidate=false`, and `no_live_fallback=true`.
- Prompt-like or storyboard-like expand output may be discarded only into a
  source-bound narrative repair. If no source-bound repair exists, the request
  is blocked; it must not use a local candidate and must not trigger provider
  model failover.

## 5. Pass / Block Classification

Provider failover pass requires all of these:

- allowed provider failover reason or no failover
- `final_model` is one of the two fixed models
- `fallback_used=false`
- `local_candidate=false`
- validator passes
- source grounding passes
- rows match
- prompt boundary passes
- KB leakage guards pass
- visual description gate passes
- shell/CDP gate is fresh when UI-driven QA is in scope
- evidence freshness gate passes

Blocked classifications:

- `provider_unavailable_blocked`
- `provider_failover_not_allowed`
- `validator_hard_gate_fail`
- `source_grounding_hard_gate_fail`
- `rows_mismatch_hard_gate_fail`
- `prompt_boundary_hard_gate_fail`
- `kb_leakage_hard_gate_fail`
- `visual_description_hard_gate_fail`
- `shell_cdp_blocked`
- `evidence_stale`

## 6. Runner And Certifier Assertions

Runner assertions:

- Must report `primary_model`, `fallback_model`, `attempted_models`,
  `final_model`, and failover fields.
- Must fail if `provider_failover_reason` is not allowlisted.
- Must fail if HTTP 403 is hidden by local fallback.
- Must fail if `fallback_used=true` or `local_candidate=true`.
- Must classify validator failures separately from provider failures.

Certifier assertions:

- Must reject missing failover trace fields.
- Must reject `provider_http_403_handled_by_model_failover=true` when
  `raw_primary_http_403_seen=false`.
- Must reject a provider failover pass when hard gate failures exist.
- Must preserve unavailable required models as unavailable; it must not silently
  remove them from the pool.

## 7. Reporting Boundary

Reports may include only sanitized provider status:

- provider name
- model name
- enabled status
- base URL presence
- API key presence
- live-ready status
- HTTP status category
- error category/code
- choices present
- callable boolean

Reports must not include:

- API keys
- tokens
- raw env file values
- authorization headers
- raw provider response body
- raw prompt body
- raw KB rows
- raw `source_register`
- overlay JSON

## 8. Provider And Validation Error Boundary

Provider errors and validation errors are different failure planes.

`ProviderErrorKind`:

```text
quota
model_unavailable
entitlement_403
timeout
network
unknown
```

Only these provider kinds may trigger model failover:

- `quota`
- `model_unavailable`
- `entitlement_403`

These provider kinds do not automatically trigger accepted failover:

- `timeout`
- `network`
- `unknown`

For `timeout`, `network`, and `unknown`, the run is blocked unless the current
controller dispatch explicitly authorizes retry/failover semantics and the
artifact records the classification.

`ValidationErrorKind`:

```text
field_semantics_violation
source_drift
narrative_body_invalid
visible_frame_invalid
prompt_leakage
kb_leakage
pseudo_success
```

No `ValidationErrorKind` may be converted into provider failover success.

Required separation:

- `provider_failover_used` means a live provider model switch was attempted for
  an allowed provider error.
- `fallback_used` means a local/content fallback was used and must be false for
  live QA pass.
- `local_candidate` means non-provider output was accepted or considered and
  must be false for live QA pass.
- `no_live_fallback=true` is required for all targeted/full16/formal403 passes.

Pseudo-success hard fail:

- If provider output is absent but local rows are accepted, fail.
- If rows are empty and `rows_match=true`, fail.
- If validator failure is hidden behind final model success, fail.
- If prompt/KB leakage is hidden behind final model success, fail.
