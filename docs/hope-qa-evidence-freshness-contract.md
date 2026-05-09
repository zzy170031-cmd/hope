# Hope QA Evidence Freshness Contract

Status: executable contract boundary for targeted, `full16`, `formal403`, and
release-shell QA evidence freshness.

Scope: artifact identity, source integrity, release executable freshness,
preflight, shell/CDP gate inheritance, runner/certifier/matrix freshness, and
next integration checklists.

This document is docs-only. It does not authorize running targeted, `full16`,
`formal403`, desktop shell, CDP, package, commit, or push work.

## 1. Artifact Identity

Every targeted, `full16`, and `formal403` artifact must record the complete
identity of the code, binary, runner, certifier, matrix, and KB mapping used to
produce it.

Required fields:

```text
workspace
branch
HEAD
origin_HEAD
dirty_patch_hash
release_exe_sha256
runtime_rs_sha256
runner_sha256
certifier_sha256
matrix_sha256
kb_mapping_sha256
gate_level
case_id
parent_gate_summary_hash
started_at
completed_at
```

Full case artifact schema:

```text
workspace
branch
head
origin_head
ahead_behind
git_status_summary
dirty_patch_hash
current_gate_allowed_dirty
previous_thread_dirty
build_side_effect_dirty
unknown_owner_dirty
runtime_rs_sha256
runtime_rs_utf8_ok
runtime_rs_sentinel_check_passed
ui_bundle_sha256
release_exe_path
release_exe_sha256
release_exe_mtime
runner_sha256
certifier_sha256
matrix_sha256
kb_mapping_sha256
gate_level
run_id
case_id
parent_gate_summary_hash
launch_artifact_path
runner_artifact_path
stoponly_artifact_path
cdp_ready
target_url
webview2_launch_env_proxy_cleared
app_process_env_proxy_present
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
no_live_fallback
seed_source_hash
accepted_narrative_hash
scene_profile_id
target_duration_seconds
duration_capacity_bucket
kb_rule_pack_ids
kb_snapshot_hash
kb_oracle_present
kb_oracle_affects_structure
main_body_first
main_body_is_complete_story
response_rows_non_empty
ui_rows_non_empty
rows_match
duration_sum_matches_target
visual_description_visible_frame_passed
prompt_text_boundary_passed
raw_sample_text_absent
raw_kb_rows_absent
source_register_absent
overlay_json_absent
prompt_body_absent
secret_absent
stoponly_ok
pseudo_success_detected
stale_evidence_detected
allowed_to_enter_next_gate
```

Schema rules:

- Field names are lower snake case in artifacts. Existing code may expose
  legacy aliases only if the certifier normalizes them into this schema.
- `current_gate_allowed_dirty`, `previous_thread_dirty`,
  `build_side_effect_dirty`, and `unknown_owner_dirty` must be separate lists.
- `dirty_patch_hash` must cover all dirty files that can affect the current
  gate; unknown ownership makes the gate blocked unless the controller
  explicitly allowlists it.
- `response_rows_non_empty=true` and `ui_rows_non_empty=true` are required
  before `rows_match=true` can contribute to a pass.
- `StopOnly` clean is represented by `stoponly_ok`; it never by itself sets
  `cdp_ready=true` or shell pass.
- `allowed_to_enter_next_gate=true` only when all inherited lower-gate
  assertions are fresh, non-stale, and non-pseudo-success.
- A blocker that repeats across adjacent cases must not remain a local patch.
  It must be promoted to a contract boundary, a regression test, and an
  error-ledger entry before the gate is rerun.

Field rules:

- `workspace` is the absolute repo/worktree path under test.
- `branch` is the current branch name.
- `HEAD` is `git rev-parse HEAD`.
- `origin_HEAD` is the upstream commit for the tested branch.
- `dirty_patch_hash` is the hash of relevant tracked/untracked patch content
  that can affect the run. Empty/clean must still be represented explicitly.
- `release_exe_sha256` is required for release-shell UI evidence.
- `runtime_rs_sha256` is required for any runtime-derived evidence.
- `runner_sha256` is required for UI-driven runner evidence.
- `certifier_sha256` is required for certifier acceptance.
- `matrix_sha256` is required for matrix evidence.
- `kb_mapping_sha256` is required when KB/golden mapping participates.
- `gate_level` must be one of `targeted`, `full16`, `formal403`, or a lower
  explicitly named diagnostic layer.
- `case_id` must identify the exact case, not only the group.
- `parent_gate_summary_hash` links the current run to the accepted lower gate.
- `started_at` and `completed_at` must be absolute timestamps.

## 2. Stale Evidence Rules

Old release exe and old gate evidence become stale if any of these changes:

- `app/src/runtime.rs`
- `ui/src/App.tsx`
- UI build input that affects the release shell
- release-shell launcher
- UI-driven runner
- model contract certifier
- targeted/full16/formal403 matrix
- KB/golden mapping
- scene profile mapping
- prompt boundary logic
- validator logic
- release build provenance

After any stale trigger:

- old artifacts are `stale` or `reference-only`
- old release exe cannot be used for new gate claims
- old shell/CDP proof cannot unlock provider, targeted, `full16`, or
  `formal403`
- old runner/certifier output cannot be merged into a new gate summary
- the next gate must rebuild or re-prove the relevant artifact identity

## 3. Source Integrity Preflight

Business gates must not start until source integrity preflight passes.

Required preflight:

```text
runtime.rs UTF-8 readable
Chinese sentinel 热血战斗 intact
Chinese sentinel 场域追逐 intact
Chinese sentinel 危局 intact
Chinese sentinel 林峰 intact
Chinese sentinel 苏瑶 intact
cargo test
npm build
node checks
git diff --check
rebuild release
AppDefault LaunchDiagnosticOnly
StopOnly
```

The UTF-8 check is a hard gate because prior non-ASCII mechanical replacement
damaged `runtime.rs`. If the runtime file cannot be read as UTF-8, or if any
sentinel is missing/mangled, stop before running business QA.

Shell/CDP hard stop:

- WebView2 `0x80000003`
- `HRESULT(0x8000FFFF)`
- `cdp_ready=false`
- wrong target URL
- missing `http://tauri.localhost/#/workbench`
- no paired `StopOnly`

If any appears, stop targeted, `full16`, `formal403`, provider, runner,
certifier, and package work. Do not substitute browser preview, Vite,
localhost mock, backend-only calls, or stale artifacts.

## 3.1 Repeated Blocker Promotion Rule

When a targeted or `full16` run fails on the same class of blocker more than
once, the next repair must first write the prevention boundary, then implement
the fix. The boundary must name:

```text
blocker_class
wrong_fix_path
correct_fix_path
required_regression_test
required_artifact_fields
stale_artifact_trigger
next_gate_rerun_scope
```

Repeated blocker classes include:

- StoryFactFrame narrative beat treated as verbatim storyboard text.
- Field-aware entity false positives caused by global string scanning.
- `rows_match=true` while runtime is blocked or rows are empty.
- `visible_rows_scene_missing` / `visible_rows_duration_missing` caused by
  blocked rows, pagination under-read, or stale UI extraction.
- `prompt_text` / `visual_description` carrying internal trace, source
  register, overlay JSON, strategy labels, or abstract evidence strings.
- provider failover swallowing validator, grounding, KB leakage, rows, or CDP
  failures.
- stale release exe or stale targeted/full16 artifact being read as current.

Correct gate behavior:

- Fix the current blocker before moving forward.
- Update the error ledger immediately after the blocker is understood, not at
  the final report.
- If runtime, UI, runner, certifier, matrix, KB mapping, or shell launcher
  changes, the release exe or evidence is stale and the flow restarts at
  rebuild, AppDefault, StopOnly, targeted A/B/C/D, then fresh `full16`.
- A lower gate pass must never be reported as a higher gate pass.

## 3.2 Recurring Blocker Closure Boundary

A recurring blocker is not closed by making the current case pass once. It is
closed only when the failure class has an executable boundary that prevents the
same mistake from reappearing under another case, scene type, duration, source
profile, shell lifecycle, or evidence layer.

Recurring blocker closure requires all of these, in order:

```text
1. live failure classified
2. wrong path named
3. prevention boundary written in the relevant contract
4. minimal regression test or runner/certifier assertion added
5. error-ledger card updated immediately
6. implementation fixed without weakening the gate
7. stale-trigger evaluated
8. release/AppDefault/targeted/full16 rerun scope reset as required
```

Blocking rule:

- If the failure is in the current allowed repair scope, fix it before moving to
  the next targeted case, next `full16` case, or higher gate.
- Do not park the blocker for later unless it is outside the allowed scope or
  hits an explicit stop condition.
- Do not continue with a known blocker while promising to document it at the
  final report.
- Do not convert the blocker into a looser allowlist, weaker assertion, broader
  provider failover, or stale artifact exception.

Required blocker-class contract mapping:

```text
field-aware false positive -> docs/hope-field-aware-story-contract.md
narrative-beat binding miss -> docs/hope-story-fact-frame-storyboard-binding-contract.md
scene/duration expression miss -> docs/hope-scene-type-rewrite-contract.md
provider/model retry confusion -> docs/hope-provider-failover-contract.md
stale evidence or pseudo-success -> docs/hope-qa-evidence-freshness-contract.md
WebView2/CDP target or popup issue -> docs/desktop-shell-webview2-cdp-startup-contract.md
prompt_text or visual_description contamination -> docs/hope-story-fact-frame-storyboard-binding-contract.md
UI stale state -> docs/hope-anime-script-confirmation-protocol.md
```

Each recurring blocker must have an error-ledger entry containing:

```text
problem symptom
wrong fix path
root cause
correct fix path
required evidence
regression or assertion
fresh rerun scope
remaining risk
```

Gate claim rule:

- `rows_match=true` cannot close a blocker unless rows are non-empty, runtime is
  not blocked, validator gates passed, prompt and visible-frame gates passed,
  and the artifact is fresh.
- A provider model switch cannot close a blocker unless the failure is a typed
  provider-layer failure and all non-provider gates still pass.
- A shell cleanup cannot close a blocker unless the shell gate itself also has
  fresh `cdp_ready=true`, the correct target URL, proxy isolation evidence, and
  paired `StopOnly`.
- A docs-only boundary cannot close an implementation blocker. It only allows
  the implementation repair to proceed with a stable rule.

## 3.3 Three-Strike Boundary Promotion Rule

If the same class of failure appears three times across targeted, `full16`,
shell, provider, runner, certifier, or validator work, it is no longer treated
as a local defect. The current rerun loop must pause before entering a larger
gate and the failure class must be promoted to a boundary.

Three-strike classification asks whether the repeated failures share one of
these roots:

```text
same_contract_boundary
same_field_semantics_boundary
same_runner_certifier_evidence_boundary
same_webview2_cdp_gate_boundary
same_provider_failover_fallback_boundary
same_stale_artifact_boundary
same_story_fact_binding_boundary
```

If classification is positive, the next repair must do all of this before the
next fresh rerun:

```text
1. name the three incidents
2. explain why they are the same boundary class
3. write or update the owning contract
4. update the error ledger
5. add a generic regression or assertion
6. repair the implementation through the generic boundary
7. prove the fix with a fresh lower gate before continuing upward
```

The owning contract must include:

```text
problem symptom
wrong fix path
correct boundary
implementation integration point
runner_certifier_acceptance_fields
counterexample_that_must_still_fail
fresh_rerun_scope
```

For field-aware entity failures, examples such as `危局`, `临界`, `林峰压`,
and `苏瑶并` are evidence of a field semantics boundary, not evidence for a
larger global word allowlist. The generic rule is:

- `person` remains strict.
- `shot_title` and `scene_title` may contain title, situation, and
  source-prefixed action/conjunction fragments.
- `visual_description` is judged as visible-frame prose, not person source.
- `character_action` may contain action fragments and conjunction tails.
- source-prefixed fragments are accepted only when the prefix is a current
  source character or accepted role and the tail is a bounded action, state, or
  conjunction fragment.
- true source-external names remain hard fail.

The three-strike rule does not stop implementation. It prevents repeated local
patches, then requires the thread to continue the current gate with the new
boundary and fresh evidence.

## 4. Gate Inheritance

Targeted A/B/C/D is a lower gate. It must not be reported as `full16`.

`full16` inherits:

- field-aware story body contract
- scene type contract
- duration capacity contract
- KB oracle positive/negative evidence
- provider failover contract
- shell/CDP freshness contract
- artifact identity contract
- source integrity preflight

`formal403` inherits:

- targeted assertions
- all `full16` assertions
- 4 entry baseline fresh rerun
- 378 fixed cases
- 21 long-text cases
- KB/golden mapping freshness
- parent `full16` summary hash

No layer may read upward:

- single case is not targeted pass
- targeted pass is not `full16`
- `full16` is not `formal403`
- `certifier_ok=true` is not a case pass
- executable readiness is not QA acceptance
- stale binary is not current evidence

## 5. Integration Point Checklist

Runtime integration points:

- `FieldAwareEntityResolver`
- stage-frame construction for `SeedSourceFrame`, `AcceptedNarrativeFrame`,
  `SceneRewritePlan`, `NarrativeStoryBody`, `SupportNotes`,
  `StoryboardInputFrame`
- unified `SceneProfile` resolver for all 21 scene types
- duration capacity resolver for fixed and future positive durations
- KB oracle positive/negative evidence builder
- provider failover classifier
- `VisibleFrame` builder for `visual_description`
- layered `prompt_text` builder
- artifact identity emitter
- source integrity preflight hooks

UI integration points:

- main editor first block must render `NarrativeStoryBody`
- confirmation first screen must start with `NarrativeStoryBody`
- support notes must be after the body or collapsed
- scene/duration switches must invalidate stale accepted expression
- task candidate must carry current accepted body, scene, duration, and KB
  binding
- visible QA trace must expose sanitized contract fields only
- UI rows must preserve backend trusted `person` values and hard fail `/`

Runner integration points:

- record artifact identity fields
- assert stage-frame presence
- assert story body first-screen presence
- assert current scene expression after switch
- assert duration capacity difference
- assert KB positive/negative evidence
- assert provider failover fields
- assert no local fallback/live fallback pseudo-success
- assert source integrity preflight result linkage
- assert StopOnly cleanup linkage

Certifier integration points:

- reject missing artifact identity fields
- reject stale parent gate summary
- reject missing KB positive/negative evidence
- reject missing provider failover fields
- reject `fallback_used=true` or `local_candidate=true`
- reject `person` unknowns and non-person fields promoted to `person`
- reject `visual_description` placeholder/internal terms
- reject prompt leakage markers
- reject upward gate reads

Formal403 runner integration points:

- verify 4 + 378 + 21 case count inheritance
- require fresh entry baseline rerun
- require `kb_mapping_sha256`
- require parent `full16` summary hash
- require per-case artifact identity
- require one shell lifecycle and `StopOnly` per case or explicitly approved
  equivalent lifecycle grouping

Rust unit test checklist:

- `危局` accepted in `shot_title`/`scene_title`, rejected in `person`
- unknown person hard fails
- `visual_description` token classes do not become person facts
- `character_action` fragments stay action fragments
- all 21 scene profiles resolve through the same resolver
- 5/10/15/30/45/60 duration capacity profiles differ
- future positive duration normalizes without special-case scene pairs
- KB positive/negative evidence is emitted and sanitized
- provider HTTP 403 can trigger only allowed model failover
- validator/source/prompt/KB failures do not trigger successful model failover
- UTF-8 sentinel check catches missing or damaged sentinels

UI-driven targeted assertion checklist:

- A/B/C/D body is complete story prose
- main editor body first
- confirmation body first screen
- scene switch changes expression and drops old scene residue
- duration switch changes capacity and drops old duration residue
- KB oracle present and structure-affecting
- no raw KB/sample/source-register/overlay/prompt leakage
- failover trace present
- no local fallback accepted as live pass
- UI rows and backend rows match
- `person=/` absent

`full16` / `formal403` assertion checklist:

- inherits every targeted assertion
- every case has artifact identity
- every case has parent gate linkage
- every case has fresh release exe identity when release shell is used
- every case has current runtime, runner, certifier, matrix, and KB mapping
  hashes
- every scene/duration/script-goal/source combination remains source-bound
- no stale artifact contributes to pass count

## 6. Runner Assertions

Runner must assert and record:

- full artifact schema fields listed in Section 1
- `main_body_first=true`
- `main_body_is_complete_story=true`
- confirmation first screen body hash equals main body hash
- candidate task `source_type`
- candidate task `narrative_body_hash`
- candidate task `scene_type`
- candidate task `target_duration`
- candidate task `kb_snapshot_hash`
- candidate task `sync_status`
- stale candidate task blocks queueing
- stale storyboard rows block generation
- `response_rows_non_empty=true`
- `ui_rows_non_empty=true`
- `rows_match=true`
- empty rows plus `rows_match=true` becomes `pseudo_success_detected=true`
- `visual_description_visible_frame_passed=true`
- `prompt_text_boundary_passed=true`
- KB positive evidence present
- KB negative leakage evidence clean
- provider failover fields present
- `fallback_used=false`
- `local_candidate=false`
- `no_live_fallback=true`
- `StopOnly` artifact path exists and `stoponly_ok=true`

Runner must fail closed when:

- artifact identity fields are missing
- source integrity preflight is missing or failed
- shell target is wrong
- CDP is not ready
- StopOnly is missing after a shell lifecycle
- rows are empty
- prompt/KB/source leakage appears
- stale candidate or stale rows are marked current
- old artifacts are reused without stale labeling

## 7. Certifier Assertions

Certifier must reject:

- missing full artifact schema fields
- missing `runtime_rs_utf8_ok`
- missing sentinel check
- stale release exe
- stale parent gate summary
- dirty ownership collapsed into one undifferentiated field
- missing `kb_oracle_affects_structure`
- `kb_rule_pack_ids` present but structure effect absent
- raw sample/KB/source/prompt leakage fields not explicitly true/false
- `response_rows_non_empty=false`
- `ui_rows_non_empty=false`
- `rows_match=true` when either row set is empty
- `fallback_used=true`
- `local_candidate=true`
- provider failover reason outside the allowlist
- validation failure hidden behind final model success
- `StopOnly` used as shell pass
- targeted evidence used as `full16`
- `full16` evidence used as `formal403`

Certifier may warn but not fail only for product-quality concerns that do not
violate source facts, field semantics, prompt boundary, KB leakage, visual-frame
validity, provider hard gate, or freshness.

## 8. Targeted / Full16 / Formal403 Inheritance

Targeted A/B/C/D:

- proves story-body, field semantics, scene switch, duration switch, KB oracle,
  provider failover, visible-frame, prompt-boundary, and freshness on the
  selected cases.
- cannot enter `full16` unless all targeted cases have fresh artifact schema
  fields and no pseudo-success.

`full16`:

- inherits every targeted assertion.
- runs the 16 source/scene/duration/script-goal combinations fresh.
- requires current runtime/UI/runner/certifier/matrix/KB mapping hashes.
- cannot enter `formal403` unless every case is fresh and the summary hash is
  accepted by Core Challenger and Audit Specialist.

`formal403`:

- inherits targeted and `full16`.
- requires fresh 4 entry baseline, 378 fixed cases, and 21 long-text cases.
- requires `kb_mapping_sha256`.
- requires parent `full16` summary hash.
- is not a throughput test. A run that only proves case count/execution speed
  without semantic, field, KB, prompt, provider, and freshness gates is blocked.

## 9. Stop Conditions

Stop immediately before business QA when:

- `runtime_rs_utf8_ok=false`
- sentinel check failed
- release exe stale
- CDP not ready
- wrong target URL
- WebView2 hard popup/error detected
- current dirty ownership unknown
- runner/certifier/matrix/KB mapping hash missing
- accepted narrative frame missing for rewrite
- candidate task stale
- storyboard rows stale
- prompt or KB leakage appears
- provider failover reason outside allowlist
- validator hard gate fails
- response/UI rows empty
- pseudo-success detected

## 10. Internal Governance Review

Core Challenger conclusion:

Question list:

1. Does the contract only cover known samples such as `危局`, `热血战斗`,
   `场域追逐`, or A/B?
   Verdict: no, only if P0-1 to P0-10 are implemented and tested; otherwise
   evidence insufficient.

2. Can the implementation still keep adding wordlist patches?
   Verdict: not accepted. The required mechanism is field-aware resolution by
   `FieldRole` and `EntityKind`.

3. Can SupportNotes pollute the body?
   Verdict: blocked. SupportNotes is support-only and cannot become accepted
   narrative source.

4. Can `scene_type` become source fact?
   Verdict: blocked. It is expression control only.

5. Can duration only change the numeric field?
   Verdict: blocked. Duration capacity must change body/beat/shot capacity.

6. Can KB be merely trace fields without structural influence?
   Verdict: blocked. `kb_oracle_affects_structure=true` is mandatory.

7. Can provider failover swallow business failures?
   Verdict: blocked. Validation, grounding, prompt, KB, visual, row, and CDP
   failures stay hard failures.

8. Can UI show stale state as current?
   Verdict: blocked. Scene/duration/body changes stale tasks and rows and
   disable generation.

9. Can artifacts reject stale evidence?
   Verdict: only if full artifact schema and stale triggers are implemented;
   until then artifact loop not closed.

10. Can `formal403` degrade into throughput testing?
    Verdict: blocked. It inherits semantic, field, KB, prompt, provider, CDP,
    and freshness gates.

11. Can source integrity catch encoding damage?
    Verdict: only if UTF-8 and sentinel preflight run before business QA.

12. Can dirty ownership be mixed into the current gate?
    Verdict: blocked. Dirty ownership must be split into allowed, previous
    thread, build side effect, and unknown owner buckets.

Synthesis / verdict:

- Direction not disproven, but implementation evidence is insufficient until
  the next thread wires the schema, stale triggers, UI invalidation, runner
  assertions, certifier assertions, and source-integrity preflight.
- Not ready to declare targeted, `full16`, `formal403`, or package readiness.

Executable next actions:

- open an implementation thread with exact file allowlist.
- implement type/state/error boundaries before business QA.
- update runner/certifier to enforce full artifact schema.
- run source-integrity preflight before any release shell QA.
- rerun targeted fresh before discussing `full16`.

Audit Specialist conclusion:

- Verdict: do not clean or refactor in this docs gate.
- Top risk: dirty worktree already contains runtime, UI, scripts, tests, and
  docs. A cleanup mixed into this contract thread would destroy ownership
  clarity.
- Recommended gate: implementation readiness gate only after controller
  approves exact file allowlist and validation commands.
- Forbidden in this thread: delete, archive, runtime edit, UI edit, runner edit,
  certifier edit, matrix edit, package, commit, push.

Audit findings:

- Docs-only scope respected by this contract thread.
- Existing dirty docs and non-doc dirty files predate this contract work and
  must not be silently claimed by it.
- Existing docs contain older gate language that is now superseded by the three
  companion contracts, but no direct contradiction requires cleanup inside this
  docs gate.
- Recommended follow-up gate: bounded implementation-readiness gate, not
  cleanup.

## 11. Next Integration Thread Draft

```text
分线程指令：
线程动作：新开
指令发给：新线程
沿用线程：无
更名线程：（运行）Hope桌面端-功能线程-【深层边界接入】
替代线程：无

workspace：
E:\codex\hope-desktop-shell-remote-current-66d3b75

目标：
按契约接入深层边界，不跑 targeted/full16/formal403，先完成类型、状态、错误、UI stale、artifact schema、runner/certifier 断言的最小实现与测试。

必须先读：
docs/hope-field-aware-story-contract.md
docs/hope-provider-failover-contract.md
docs/hope-qa-evidence-freshness-contract.md
docs/desktop-release-qa-handoff.md
docs/desktop-gate-evidence-ladder-contract.md

允许修改范围：
由总控另行明确 app/src/runtime.rs、ui/src/App.tsx、scripts/hope-ui-driven-trace-runner.mjs、scripts/hope-model-contract-certifier.mjs、tests/qa/* 的具体 allowlist。

禁止：
不要 package / commit / push。
不要跑 targeted/full16/formal403。
不要输出 API key、token、raw env、raw KB rows、raw sample_text、source_register、overlay JSON、prompt_body。
不要清理或回滚未知 dirty。

接入顺序：
1. 类型边界：FieldRole / EntityKind / SourceFrameKind / SceneProfileId / DurationCapacity / KBOracleKind / ProviderErrorKind / ValidationErrorKind。
2. 状态边界：seed_source_locked 到 artifact_stale。
3. 错误边界：provider failover 与 validation hard fail 分离。
4. UI stale：scene_type / target_duration / body change 后 stale task、stale rows、disable generation。
5. Artifact schema：补齐 full case artifact 字段。
6. Runner/certifier：拒绝 pseudo-success、stale evidence、KB/prompt leakage、fallback/local candidate。
7. Source integrity：runtime.rs UTF-8 与 sentinel preflight。

验证：
git diff --check -- <allowed files>
cargo test / npm build / node checks 按总控 allowlist 执行
不跑 targeted/full16/formal403

回报：
branch / HEAD / origin / ahead-behind
git status
修改文件清单
实现的类型/状态/错误边界
UI stale 行为
runner/certifier 断言
测试结果
未解决风险
是否允许 QA 线程恢复 targeted
```
