# Hope Desktop Release QA And Handoff

This document records the non-secret handoff surface needed to continue Hope
desktop-shell QA, matrix validation, and release preparation from another
machine.

## Current Rule

- Use the real release shell for desktop QA.
- Do not use browser preview, Vite, localhost mock, IPC-only, or backend-only
  runs as acceptance.
- Run one QA group per shell lifecycle.
- Clean the shell with `StopOnly` after every group.
- Do not require the user to take screenshots or click system error dialogs.

## Contract Pack

The following docs are now required companion contracts for environment and QA
continuation:

- `docs/desktop-shell-webview2-cdp-startup-contract.md`
- `docs/desktop-validator-source-grounding-contract.md`
- `docs/desktop-qa-preflight-ownership-contract.md`
- `docs/desktop-gate-evidence-ladder-contract.md`
- `docs/hope-story-fact-frame-storyboard-binding-contract.md`
- `docs/hope-field-aware-story-contract.md`
- `docs/hope-provider-failover-contract.md`
- `docs/hope-qa-evidence-freshness-contract.md`

## Repo-Local Launcher

Use the repo-local launcher from a checkout of `codex/desktop-shell`:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\start-hope-release-cdp.ps1 -Provider qwen -Model qwen-plus-2025-07-28 -NoProxy -QaProviderHardFail -WebView2ArgumentMode AppDefault -StopExisting -StopOnCdpFailure -WaitSeconds 20
```

Cleanup command:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\start-hope-release-cdp.ps1 -StopOnly
```

WebView2 / CDP environment-only diagnostic command:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\start-hope-release-cdp.ps1 -LaunchDiagnosticOnly -WebView2ArgumentMode AppDefault -StopExisting -StopOnCdpFailure -WaitSeconds 20
```

This mode does not launch provider runner work. It is only for confirming how far
the release shell and WebView2 startup progressed when `hope-app` starts but
`cdp_ready=false` and `target_count=0`.

The legacy local launcher path may still exist on the original machine:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File E:\codex\tools\start-hope-release-cdp.ps1 -Provider qwen -Model qwen-plus-2025-07-28 -StopExisting -StopOnCdpFailure -WaitSeconds 20
```

For new machines, prefer the repo-local launcher.

## Clean Workspace Shell Gate

Accepted Shell Gate evidence for a clean workspace / worktree must start the
repo-local launcher with an explicit `-RepoRoot` pointing at the absolute path
of the workspace under test. The accepted WebView2/CDP fix anchor is:

```text
fix_commit=9a8f4e0349fa56a7cc57fb2e3817fabaa75afb6e
accepted_clean_workspace=E:\codex\hope-desktop-shell-clean-9a8f4e0-r2
accepted_exe_sha256=E3BB9738851AF64FBE642517CA8DE2BEE82B23F67B4B19E9C8C817E8C6AD6D26
accepted_valid_runs=hope-cdp-20260505-193016-26016, hope-cdp-20260505-193110-27216
```

Example clean-worktree environment-only command:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\start-hope-release-cdp.ps1 -RepoRoot E:\codex\hope-desktop-shell-clean-9a8f4e0-r2 -LaunchDiagnosticOnly -WebView2ArgumentMode AppDefault -StopExisting -StopOnCdpFailure -WaitSeconds 20
```

A run that omits `-RepoRoot` and defaults to `E:\codex\hope-desktop-shell` is
`boundary-invalid` / `stale-reference` for clean-workspace Shell Gate
acceptance. It can explain historical behavior only; it cannot satisfy the
current gate.

Shell Gate acceptance must cite all of these fields together:

- workspace path and workspace `HEAD`
- launched process path / PID and `hope-app.exe` sha256
- saved `/json/list` artifact path and target URL/title summary
- saved post-setup probe artifact path
- saved launch result artifact path
- paired `StopOnly` cleanup artifact path and cleanup fields

Popup handling is not a visual-only gate. No visible popup is not enough: the
accepted run must also prove no `0x80000003`, no `HRESULT(0x8000FFFF)`, correct
`http://tauri.localhost/#/workbench` target URL/title, and cleanup with no
Hope-owned process, WebView2 process, CDP port, or exe-lock residue.

Hope-2-v0, `E:\codex\hope-local-stale-archive`, old `%TEMP%` launch artifacts,
old CDP profiles, and old no-`RepoRoot` launches are reference-only boundaries
for old Hope. They must not be used as current Shell Gate, provider, live3,
`full16`, `403-case`, or package evidence.

## WebView2 Environment-Blocked Evidence

When the release shell starts `hope-app` but no CDP target materializes, collect
launch-only evidence before retrying provider or validator work:

- the only formal shell startup gate is
  `-LaunchDiagnosticOnly -WebView2ArgumentMode AppDefault`
- `Minimal` and `FullDefault` are diagnostic comparison only and cannot unlock
  provider, runner, live3, required pool, `full16`, `403-case`, or package
- `AppDefault` uses the Hope-2 compatible hardened WebView2 browser arguments
  plus the diagnostic CDP port, but it must not use `--noerrdialogs`,
  `--disable-breakpad`, or `--disable-crash-reporter`; hiding a popup is not a
  startup pass
- if the diagnostic log stops at `before_main_window_build`, treat it as a
  host-window build blocker
- if a WebView2 popup shows `0x80000003`, route it to an environment thread,
  not provider or validator diagnosis
- if `target_url` resolves to `http://127.0.0.1:5173/`, treat it as a
  devUrl/stale-build blocker, not shell success
- use `-LaunchDiagnosticOnly` and do not pass provider / model / runner inputs
- for clean workspace / worktree Shell Gate runs, pass explicit `-RepoRoot`
  with the absolute path of the workspace under test; no-`RepoRoot` runs that
  hit the dirty main repo are `boundary-invalid` / `stale-reference`
- keep the per-launch diagnostic log under
  `%TEMP%\hope-webview2-cdp\<launch_id>\hope-shell-diagnostic.log`
- confirm launch anchors for the same launch:
  `process_start`, `before_builder_run`, `setup_enter`,
  `main_window=created_in_setup`, `setup_exit`, `builder_run_returned_ok` or
  `builder_run_error`, and any `panic=...`
- when the diagnostic reaches `setup_exit` but `target_count=0`, collect
  post-setup liveness anchors such as
  `post_setup_window_probe delay_ms=1000/5000 main_window_present=...`
- every formal shell run must persist immutable per-run CDP artifacts under
  `%TEMP%\hope-webview2-cdp\<launch_id>\`: `cdp-json-version-poll-*.json`,
  `cdp-json-list-poll-*.json`, `post-setup-probe-poll-*.json`, and
  `hope-launch-result.json`
- every `StopOnly` cleanup must persist an immutable
  `hope-stoponly-cleanup-result.json` artifact and report its
  `cleanup_artifact_path`
- `/json/list` acceptance must cite the saved artifact path, not only terminal
  summary output
- reviewer acceptance must come from saved JSON artifact paths such as
  `cdp_json_list_latest_artifact_path`, `post_setup_probe_latest_artifact_path`,
  and `result_artifact_path`; mutable fields like `last_targets` or console
  summaries are context only
- post-setup probes must record main-window presence, CDP port state, WebView2
  process count, expected profile usage, diagnostic last line, and the
  `target_materialization_stage`
- confirm runtime-entry lines for
  `webview2_additional_browser_args_runtime` and
  `webview2_user_data_folder_runtime`
- confirm process evidence for `webview2_hardened_args_seen=true`; confirm app
  diagnostic evidence for `webview2_app_popup_suppression_arg_present=false`
  and `webview2_popup_suppression_detected=false`
- record `webview2_runtime_noerrdialogs_seen` / `webview2_noerrdialogs_seen`
  separately when process inspection sees runtime-level flags; these fields are
  risk context, not proof that the app or launcher hid a popup
- record EBWebView profile stage evidence:
  `Default`, `Local State`, `Last Version`, `Crashpad`,
  `Edge-Local-State-Tmp*`, and lock-like entries
- record fallback evidence that does not depend on `Win32_Process` command-line
  access:
  port listening state, pid file, hope-app process state, stdout/stderr tail,
  profile stage evidence, and Application event-log summary
- always finish with `StopOnly` and report whether cleanup reached:
  `hope_app_remaining_pids=[]`, `webview2_remaining_pids=[]`,
  `port_released=true`, `port_listening=false`, and `exe_unlocked=true`

When this shell gate is blocked, do not start provider runner work, business
QA, live3, required pool, `full16`, `403-case`, or packaging.

## Release Build Provenance

Do not treat a plain workspace build such as:

```powershell
cargo build -p hope-app --release
```

as accepted Hope release-shell provenance.

For this repo, the accepted release-shell binary must come from the Tauri CLI /
formal release build path that uses `app/tauri.conf.json`, runs
`beforeBuildCommand`, and packages `frontendDist=../ui/dist` into the desktop
binary. Inference from local build outputs:

- the currently executed `hope-app.exe` must be shown as fresh enough for the
  current code / build-input state
- current-exe provenance may be proved with the rebuilt
  `target\release\hope-app.exe` hash / mtime / size and the launched process
  image path / hash / mtime / size matching that executable
- accepted release-like Tauri build output reports `cargo:dev=false`
- accepted release-like Tauri build output reports `cargo:rustc-cfg=custom_protocol`
- `ui/dist/index.html` and built assets must exist
- runtime CDP target must resolve to `http://tauri.localhost/#/workbench`

If a rebuilt shell opens `http://127.0.0.1:5173/`, treat that as a
stale/devUrl build blocker, not as validator/provider evidence. The repo-local
launcher now surfaces this as a release target mismatch blocker.

## Secret Setup

The launcher expects a local env file by default:

```text
C:\Users\Administrator\.codex\.sandbox-secrets\hope-qwen.env
```

The env file is not committed. It should provide the Qwen API key. The launcher
can provide the default DashScope compatible base URL for QA.

Never commit:

- API keys
- tokens
- secrets
- raw env files
- authorization headers
- raw prompt bodies
- raw `source_register`
- overlay JSON

## Provider Hard Gate

Non-secret Qwen model rule:

- Current QA live default model is `qwen-plus-2025-07-28`.
- Required text gate pool is exactly: `qwen-plus-2025-07-28`,
  `qwen3.6-plus`, `qwen3.6-plus-2026-04-02`,
  `qwen3.6-max-preview`, `qwen3-max-2026-01-23`,
  `qwen3.6-flash`, and `qwen-plus-2025-12-01`.
- `qwen-plus` is retained only as an
  `alias_compatibility_reference` / `not_required_gate` historical alias. It
  may be used for compatibility diagnostics, but it cannot satisfy live3,
  full16 restore, or required text gate evidence.
- Excluded from the text storyboard gate: `qvq-max-2025-03-25`, qwen-vl /
  qwen2.5-vl / qwen3-vl, HappyHorse and video generation models, qwen-math,
  and qwen-coder.
- `qwen-max` and older max aliases are not current gate evidence unless a
  controller dispatch explicitly adds them to `required_text_gate_models`.
- `qwen-plus` may still appear in app/UI/script compatibility allowlists. A run
  that starts or expects `qwen-plus` is non-gate even if the local provider
  status matches it.
- Every QA run must explicitly confirm the active model in both the release
  shell startup command and the runner `--expected-model` argument.
- Do not silently switch models inside the UI-driven trace.
- Do not pass QA through fallback-only behavior or fallback pseudo-success.
- For qwen/qwen-plus-2025-07-28 release-shell QA, start with `-NoProxy -QaProviderHardFail`
  unless the controller explicitly asks for proxy diagnostics.
- The launcher evidence is sanitized only: `process_env_proxy_present`,
  `proxy_env_before`, `proxy_env_after`, `qa_proxy_cleared_in_launcher`, and
  `qa_provider_hard_fail`. It must not include raw proxy values or secrets.
- QA hard-fail mode sets `HOPE_QA_NO_LOCAL_FALLBACK` for the launched app. If
  qwen retry is exhausted, runtime returns a blocked QA result with
  `text_model_qa_no_local_fallback_blocked`; it must not create or accept a
  local candidate as a live pass.
- QA hard-fail mode may also set a sanitized provider timeout override for
  release gates. This is QA-only stable transport, not a product default and
  not a user no-proxy requirement. Timeout exhaustion still fails the gate.

Multi-model output contract rule:

- The executable multi-model certification manifest is
  `tests/qa/desktop-model-certification-matrix.json`.
- Source role grounding must follow
  `docs/desktop-validator-source-grounding-contract.md`.
- The contract is a structural safety contract, not a creative style template.
  It verifies provider/model evidence, `script_goal`, `scene_type`, duration,
  accepted snapshot hash, StoryFactFrame hash, source text hash, rows schema,
  prompt_text boundary, no live fallback, rows_match, and validator status.
- The normalizer may unwrap markdown JSON, normalize row field names, normalize
  JSON/row shape, split visual/action/camera columns when labels are explicit,
  annotate source/forbidden fact refs, and record normalize notes. It must not
  add user source facts, invent characters/props/places/worldview, rewrite user
  facts to pass validators, force a fixed style template, use 21/27 sample
  content, or present fallback/local candidate output as live provider output.
- `hard_gate_failures[]` are reserved for source fact binding failure,
  forbidden drift, prompt_text pollution, fallback/local candidate,
  provider/model mismatch, scene_type or duration override, non-executable rows
  schema, rows_match=false, stale/missing StoryFactFrame or accepted snapshot,
  and cleanup failure.
- `quality_warnings[]` are for product-quality review only: template-like
  expression, weak rhythm, low visual specificity, plain prompt language, weak
  emotion, or scene-type expression that is not distinct enough but does not
  violate fact or boundary gates. A quality warning alone is not a hard-gate
  failure and cannot substitute for a hard-gate pass.
- Creative freedom must remain intact: no fixed three-row requirement, no fixed
  shot sentence style, no fixed action tempo, no fixed camera style, and no
  runtime use of 21/27 sample narrative structures, sample people, sample props,
  sample places, or sample worldview.
- Prompt text must not contain `sample_text`, `smoke_extracts`,
  `qa_reference`, `source_sample_id`, sample entity markers, raw KB rows,
  `source_register`, or overlay JSON. These markers are hard prompt-boundary
  failures, not quality warnings.
- The same contract is used for every callable model in
  `required_text_gate_models`. Do not add model-specific pass logic. The
  `qwen-plus` alias is compatibility-only and must not be counted as required
  gate evidence.

Live3 and full16 restore rule:

- Provider availability probes must be sanitized. Record only model,
  http_status, error_category, error_code, choices_present, and callable.
- Store the sanitized seven-model probe as
  `target/qa-live3-provider-probe/<timestamp>/qwen-text-model-availability.json`
  and validate it with:

```powershell
node .\scripts\hope-model-contract-certifier.mjs --matrix .\tests\qa\desktop-model-certification-matrix.json --availability <availability-json>
```

- Unavailable results such as model_not_found, entitlement, quota, http_403,
  and invalid_parameter must be recorded as unavailable reasons. Unavailable
  models are not silently removed.
- A required text model passes live3 only when `expected=3`, `executed=3`,
  `passed=3`, `failed=0`, `not_run=0`, `no_http_403=true`,
  `no_live_fallback=true`, `fallback_used=false`, `local_candidate=false`,
  `runner_assert_failures=[]`, StoryFactFrame is present, accepted snapshot is
  present, `prompt_text_boundary_passed=true`, `rows_match=true`, and
  `row_diffs=[]`.
- The text pool passes only when all callable required text gate models pass
  live3 3/3 and at least five callable text models pass.
- Full16 remains blocked until the text pool rule above is satisfied and Core
  Challenger plus Audit Specialist accept the evidence. One model passing
  live3 is not enough.
- Runner hard-fail stages remain distinct:
  `provider_retry_exhausted_hard_fail`,
  `qa_hard_fail_evidence_missing`, model unavailability categories
  (`model_not_found`, `entitlement`, `quota`, `http_403`,
  `invalid_parameter`), `validator_hard_gate_fail`, and
  `prompt_text_boundary`.
- Older `gate_model_order` fields in four-group, 16-case, cross-drift, and
  403 matrices are not authoritative for this live3 text-model pool. Until
  those manifests are separately opened, use
  `tests/qa/desktop-model-certification-matrix.json` as the only source of
  truth for required live3 text gate models.

Before running business QA, confirm provider status from the real release shell:

```text
provider=qwen
model=qwen-plus-2025-07-28
enabled=true
base_url_present=true
api_key_present=true
live_ready=true
status=enabled
storage=session-only
```

If `live_ready` is not true, stop the shell and do not run fallback-only QA.

When later entering `qwen3.6-plus` live3 or the required text model pool live3,
any shell lifecycle must still pass explicit `-RepoRoot` for the clean
workspace under test and must end with reported `StopOnly` cleanup. Provider or
runner evidence from a no-`RepoRoot` shell launch is not current gate evidence.

## Four-Group Trace Gate

The four required smoke cases are stored in:

```text
tests/qa/desktop-four-groups.json
```

The repo-local CDP runner used for the visible WebView2 workflow is:

```text
scripts/hope-ui-driven-trace-runner.mjs
```

Example invocation after the release shell CDP target is ready:

```powershell
node .\scripts\hope-ui-driven-trace-runner.mjs --case A_hot_blood_battle --script-goal expand --scene 热血战斗 --source "废墟之上，主角单膝跪地，敌人缓步逼近。" --duration 15 --expected-provider qwen --expected-model qwen-plus-2025-07-28 --assert-binding 1 --assert-no-proxy-env 1 --compact 1
```

Each case must:

- start the real release shell through CDP
- drive the visible WebView2 UI
- select scene type
- select 15 seconds
- enter source text
- run the selected script-goal command path; four-group smoke uses
  `--script-goal expand --assert-binding 1`
- confirm the expanded text
- create a shot task
- add it to the queue
- start generation
- read the visible "current shot result" table
- read `window.__hopeQaTrace`
- read `#hope-qa-trace[data-hope-qa-trace]`
- capture screenshot and layout bounds
- run `StopOnly`

Required trace checks:

- provider is `qwen`
- expected model is `qwen-plus-2025-07-28`
- runner passes `--script-goal expand --expected-provider qwen --expected-model qwen-plus-2025-07-28 --assert-binding 1`
- `/json/list` target count is recorded
- target URL/title is recorded
- release shell PID is recorded
- runner execution is recorded
- trace generation is recorded
- response rows hash equals UI rows hash
- `rows_match=true`
- `row_diffs=[]`
- duration is 15 seconds as 2 rows: `10 + 5`
- no `model_config_disabled`
- no `text_model_live_call_closed`
- provider is live-ready
- compact fallback evidence includes `retry_timeline`, `retry_exhausted_count`,
  `retry_recovered_count`, `attempts_observed`, `elapsed_buckets`,
  explicit missing-reason fields when compact evidence cannot recover
  attempt/elapsed metadata, `qa_proxy_evidence`, and `runner_env_proxy_evidence`
- `text_model_qa_no_local_fallback_blocked` proves QA hard-fail / no live
  fallback; it is not by itself provider transport retry exhaustion. Only
  runtime warning text containing `retry_exhausted=true` or observed
  attempts/elapsed metadata may set `provider_retry_exhausted_hard_fail`.
- When a validator hard gate triggers QA hard-fail without provider transport
  retry metadata, compact evidence must report provider attempt metadata as
  `not_applicable_*` and classify the provider retry edge as validator
  hard-gate QA hard-fail, not as a generic trace-missing provider field.
- Raw compact `provider_retry_evidence` must be self-contained for current
  runner artifacts: include `provider_retry_telemetry_format="current"` and
  `legacy_provider_retry_exhausted_hard_fail_edge=false` alongside
  `provider_retry_observed`, `provider_retry_classification`, and
  `provider_attempt_metadata_status`.
- compact model contract evidence includes `hard_gate_failures`,
  `provider_retry_evidence`, `quality_warnings`, `normalizer_actions`,
  `creative_freedom_preserved`, `template_overconstraint_risk`, `sample_leakage_risk`,
  `source_fact_binding_status`, `prompt_boundary_status`, and
  `fallback_status`
- `retry_recovered=true` is allowed only when `fallback_used=false` and no
  local candidate is reported
- `retry_exhausted=true` under QA hard-fail must become a blocked provider
  result with `text_model_qa_no_local_fallback_blocked`, not
  `text_model_live_storyboard_fallback`

## 16-Case Pre-403 Gate

The 16-case manifest is stored in:

```text
tests/qa/desktop-16-case-matrix.json
```

This is a P1-P4 pre-403 gate, not 403-case execution. It uses the four-group
manifest as the business carrier and varies:

- script goal: `rewrite` / `expand`
- scene: same / different
- duration: same / different
- source text: same / different

Current executable state:

- The runner supports explicit `--script-goal expand` and
  `--script-goal rewrite`.
- `expand` selects `ui.script_actions.handleExpandStory` through
  `.script-actions button:nth-of-type(2)`.
- `rewrite` selects `ui.script_actions.handleExpandScript` through
  `.script-actions button:nth-of-type(3)`.
- The app-emitted expand trace records observed `script_goal` and
  `command_path`; compact runner evidence records both selected and observed
  command evidence, qwen/qwen-plus-2025-07-28 provider/model, accepted snapshot evidence,
  StoryFactFrame binding evidence, marker-based prompt_text boundary evidence,
  rows_match/row_diffs, no_http_403, no live fallback, no validator
  pseudo-success, and `runner_assert_failures`. Runtime source-lineage proof is
  still a live trace review requirement and is not claimed by marker evidence
  alone.
- This is executable readiness only. Do not describe it as a real 16-case pass
  until all 16 live WebView2/CDP runs have completed with clean evidence.
- Harness readiness is not 403-case entry, packaging readiness, or release
  readiness.

Later real 16-case execution must run each manifest case with its own
`script_goal` value:

```powershell
node .\scripts\hope-ui-driven-trace-runner.mjs --case <case.id> --script-goal <case.script_goal> --scene <case scene label> --source "<case source text>" --duration <case.duration_seconds> --expected-provider qwen --expected-model qwen-plus-2025-07-28 --assert-binding 1 --assert-no-proxy-env 1 --compact 1
```

Every case must verify the storyboard task, storyboard `prompt_text`,
`scene_type`, `duration_seconds`, accepted snapshot, StoryFactFrame, forbidden
drift fields, and prompt boundary. `prompt_text` may use accepted facts,
StoryFactFrame, scene/duration, and rule tags only. It must not include
`sample_text`, `smoke_extracts`, `qa_reference`, `source_sample_id`, sample
entity markers, raw KB rows, raw prompt body, `source_register`, or overlay JSON.

Prompt boundary evidence must be sanitized and must not expose raw prompt
bodies. The runner records marker-based boundary evidence; source-lineage proof
still requires live trace review. Required fields are: `prompt_text_hash`, `prompt_text_source_policy`,
`prompt_text_forbidden_source_hits`, `prompt_text_raw_body_absent`,
`sample_text_absent`, `smoke_extracts_absent`, `raw_kb_rows_absent`,
`qa_reference_absent`, `source_sample_id_absent`,
`sample_entity_marker_absent`, `source_register_absent`, and
`overlay_json_absent`. The compact runner summary must include these fields for
every 16-case run.

Before any model becomes a full16 candidate, validate the certification manifest
and any compact evidence files with:

```powershell
node .\scripts\hope-model-contract-certifier.mjs --matrix .\tests\qa\desktop-model-certification-matrix.json
```

For live 3-case certification evidence, pass each compact runner artifact with
`--evidence <path>`. The certifier checks only whitelisted, redacted contract
fields and must not be given raw prompts, raw provider responses, raw env, or
sample-library source text.

## Scene-Type Rewrite Gate

The same source text must produce scene-specific expanded text when the scene
type changes.

The rewrite must preserve:

- original characters
- original events
- original pressure relationship
- original location or spatial facts

The rewrite must change:

- scene expression
- rhythm
- camera or viewpoint tendency
- atmosphere and scheduling language
- the selected scene type's visible writing features

The rewrite must not add:

- source-external characters
- source-external setting
- source-external props
- story facts invented only to fit the scene type

This gate is part of four-group QA first, then part of the later 403-case
matrix.

## 403-Case Matrix

The matrix definition is stored in:

```text
tests/qa/desktop-403-matrix.json
```

Current scope:

- fixed durations: `5`, `10`, `15`, `30`, `45`, `60` seconds
- scene types: 21
- fixed matrix: 3 texts x 21 scenes x 6 durations = 378 cases
- long text matrix: 1 docx long text x 21 scenes = 21 cases
- entry baseline guard: 4 cases from `tests/qa/desktop-four-groups.json`
- total: 403 cases (`378 + 21 + 4`)

The four entry baseline cases are formal 403 entry guards, not a substitute for
the fixed or long-text matrix: `A_hot_blood_battle`,
`A_chinese_war_formation`, `B_hot_blood_battle`, and `B_slg_sandbox_view`.
They must be rerun fresh with the same release WebView2/CDP shell lifecycle,
provider hard gate, prompt boundary, StoryFactFrame, KB rule-pack evidence, and
StopOnly cleanup required for every other formal 403 case.

The executable KB/golden boundary manifest is:

```text
tests/qa/desktop-403-kb-golden-mapping.json
```

Formal 403 evidence must prove `kb_rule_pack_ids` and `kb_snapshot_hash` from
the live trace. Golden/reference samples may be used only as sanitized
structure oracles: scene taxonomy, craft tags, director rule tags, camera or
motion anchors, negative drift tags, compact sample summaries, expected prompt
oracle, and expected row oracle. Raw `sample_text`, raw `smoke_extracts`, raw
KB rows, sample names, sample props, sample places, sample worldview,
`source_register`, and overlay JSON must not enter `prompt_text`, logs, chat, or
QA artifacts.

Long-text sample path on the original machine:

```text
C:\Users\Administrator\Desktop\九州剧本文字版2.docx
```

Do not run the 403-case matrix until the four-group trace gate, scene-type
rewrite gate, full 16-case pre-403 gate, and cross rewrite drift smoke gate
pass. Harness executable readiness alone does not authorize 403-case entry
unless a separate controller dispatch explicitly opens that reduced 403 gate.

## Story Fact Frame Binding Gate

The executable contract is stored in:

```text
docs/hope-story-fact-frame-storyboard-binding-contract.md
```

The cross rewrite drift smoke definition is stored in:

```text
tests/qa/desktop-cross-drift-smoke.json
```

Before 403-case, every live trace must expose sanitized binding evidence:

- `current_case_id`
- `source_text_hash`
- `accepted_rewrite_hash`
- `task_script_hash`
- `story_fact_frame_hash`
- `source_profile`
- `scene_type`
- `duration_seconds`
- `duration_plan_hash`
- `storyboard_rows_hash`
- `must_keep_facts`
- `missing_source_facts`
- `forbidden_facts`
- `forbidden_fact_hits`
- `stale_binding_detected`
- `kb_rule_pack_ids`
- `kb_snapshot_hash`

Passing rows require `stale_binding_detected=false`, `missing_source_facts=[]`,
and `forbidden_fact_hits=[]`. `rows_match=true` alone is not sufficient.

## 2026-04-30 Bridge State

This checkpoint was pushed so another machine can continue without depending on
local-only files from the original machine.

Latest known release-shell facts before the bridge:

- real release WebView2 launch and `StopOnly` cleanup were stable
- provider hard gate reported `live_ready=true`
- Qwen live calls returned HTTP 403 in all four groups
- scene-type rewrite text passed at the fallback text layer
- `generate_storyboard` still used fallback because of HTTP 403
- A + war formation still had a UI/response mismatch:
  response row 2 person was `敌人与废墟`, while UI displayed `/`
- do not open the 403-case matrix from the original machine

Next machine should continue from the pushed branch by first confirming the same
four-group gate, then resolving the 403 and A + war formation UI person
mismatch before entering the full 403-case matrix.

## Release Preparation Gate

Do not start packaging until:

- the shell startup gate is green under
  `docs/desktop-shell-webview2-cdp-startup-contract.md`
- four-group qwen trace passes
- 403-case matrix passes or the controller explicitly accepts a reduced gate
- any reduced gate is explicitly scoped by the controller and is not inferred
  from a degraded 16-case preparation record
- the gate layer is explicitly accepted under
  `docs/desktop-gate-evidence-ladder-contract.md`
- dirty ownership is resolved
- non-secret automation and resources are pushed
- release rebuild is repeatable
- WebView2/CDP has no blocker
- no secret is committed

## Dirty Ownership

Commit by explicit allowlist only. Do not use:

```text
git add .
git add -u
git commit -a
```

Known ownership buckets:

- shell/CDP/qwen enable: `app/src/main.rs`, `app/src/state.rs`,
  `app/tauri.conf.json`, `scripts/start-hope-release-cdp.ps1`
- runtime storyboard and scene rewrite quality: `app/src/runtime.rs`
- UI trace/status chain: `ui/src/App.tsx`, `ui/src/styles.css`
- icon and logo assets: `app/icons/**`
- Tauri generated schemas: `app/gen/**`
- QA samples and handoff docs: `tests/qa/**`, `docs/**`

Keep `app/Cargo.toml` separate unless the controller explicitly opens that
ownership gate.

Every QA or environment thread that can touch Tauri build paths must also
follow `docs/desktop-qa-preflight-ownership-contract.md`, including preflight
and postflight `app/Cargo.toml` diff recording and tracked-file ownership
reporting.

## Launcher Proxy Isolation Addendum

The `AppDefault + LaunchDiagnosticOnly` shell gate remains unchanged. The
2026-05-07 WebView2 popup recurrence only adds an environment-isolation rule to
the repo-local launcher; it does not replace the existing `HOPE-SHELL-004`
playbook.

For every release shell / WebView2 / CDP startup, the launcher must clear proxy
environment before starting `hope-app.exe`, regardless of whether `-NoProxy` is
passed:

```text
HTTP_PROXY
HTTPS_PROXY
ALL_PROXY
NO_PROXY
http_proxy
https_proxy
all_proxy
no_proxy
```

`-NoProxy` remains QA/provider semantics and evidence only. It must not be the
only trigger for WebView2 environment isolation.

Before any release shell launch, the launcher must also prove that WebView2
process command-line inspection is available. If
`webview2_process_query_capability.query_ok=false` or access is denied, stop
before starting `hope-app.exe`; otherwise the lifecycle cannot prove stale
WebView2 popup / crash cleanup.

Shell and UI-driven artifacts must record these non-secret fields:

```text
launcher_process_env_proxy_present_before
webview2_launch_env_proxy_cleared
app_process_env_proxy_present
no_proxy_loopback_only
webview2_process_query_capability.query_ok
webview2_process_query_capability.hope_process_count
webview2_process_query_capability.non_hope_process_count
webview2_process_query_capability.non_hope_webview_exe_names
```

The artifacts must not include raw proxy values, raw env files, API keys,
tokens, or secrets.

`StopOnly` cleanup scope is Hope-owned WebView2 only: command lines naming
`hope-app.exe` or the `hope-webview2-cdp` temp profile. System WebView2
processes owned by Windows surfaces such as `SearchHost.exe` or `Widgets.exe`
must be reported separately as non-Hope context and must not be killed or
counted as Hope release-shell residue.

The shell gate still forbids `--noerrdialogs`, `--disable-breakpad`, and
`--disable-crash-reporter`; popup suppression cannot be used as a pass. A
`StopOnly` clean artifact is required cleanup evidence, but it is not a shell
pass without a paired `AppDefault + LaunchDiagnosticOnly` launch artifact.

If `0x80000003` or `HRESULT(0x8000FFFF)` appears, stop targeted, `full16`,
`403-case`, provider, runner, and package work immediately and route the issue
back to shell/CDP blocker handling.

2026-05-07 validation evidence for the addendum:

```text
AppDefault LaunchDiagnosticOnly ok=true
cdp_ready=true
target_url=http://tauri.localhost/#/workbench
webview2_popup_suppression_detected=false
webview2_app_popup_suppression_arg_present=false
webview2_launch_env_proxy_cleared=true
app_process_env_proxy_present=false
StopOnly clean
```

## Scene Rewrite Narrative Contract Addendum

The executable rewrite contract is
`docs/hope-scene-type-rewrite-contract.md`.

All `扩写故事` and `改写剧本` UI-driven gates must prove this shared pipeline:

```text
current accepted story fact source
+ target scene_type
+ target duration
+ KB scene rule pack
= narrative story body
```

The main visible story text must be narrative prose. It must not be action
choreography, scene blocking notes, storyboard breakdown, strategy explanation,
duration-plan explanation, KB trace, prompt text, `source_register`, overlay
JSON, raw KB rows, raw sample text, or internal validator output.

The duration check applies to every product target duration, not only 15/30/60
probes. The current fixed duration set is `5`, `10`, `15`, `30`, `45`, and `60`
seconds, but any current or future positive fixed duration must map into the
same narrative-capacity rule before generation. A gate must fail if only
`duration_seconds` changes while the visible story prose keeps the same
capacity.

KB must be visible through better structure, not through explanatory prose.
`扩写故事` and `改写剧本` must use KB writing/director/golden structure rules to
shape situation setup, conflict progression, emotional turn, and closing beat.
The user-visible story body must not say "根据 KB", "知识库规则", or expose raw
KB/sample/source-register/overlay data.

Targeted cases may use `热血战斗 -> 场域追逐` and other strong-difference scene
switches, but implementation and certification must not special-case those
paths. Fresh full16 and formal403 inherit the same `story_body_gate`,
scene/duration binding evidence, KB leakage guard, and visual-description gate.
Desktop release UI review is mandatory: backend/runtime pass alone is
insufficient. The visible main editor text, confirmation dialog body, task
candidate text, generated storyboard rows, and expanded/edit dialogs must all
show the same scene/duration/KB narrative binding.

## 2026-05-08 Field-Aware Story / Failover / Freshness Addendum

The executable boundary for the current scene-body gate is now split into three
companion contracts:

```text
docs/hope-field-aware-story-contract.md
docs/hope-provider-failover-contract.md
docs/hope-qa-evidence-freshness-contract.md
```

Release QA must inherit all three before targeted, `full16`, or `formal403`
can resume.

Required inherited checks:

- `NarrativeStoryBody` is the first visible story block in the main editor and
  confirmation first screen.
- `FieldAwareEntityResolver` semantics are enforced so situation/title words
  such as `危局` do not become `person` facts.
- All 21 scene types resolve through one `SceneProfile` shape, not hot-path
  special cases.
- `target_duration` changes narrative capacity, not only stored seconds.
- KB oracle evidence is positive and structural while raw KB/sample/source
  content remains absent.
- Provider model failover is limited to quota, unavailable model, or
  entitlement/permission HTTP 403 and cannot hide validator, grounding, row,
  prompt, KB, visual, or CDP failures.
- Artifacts record workspace, branch, `HEAD`, origin head, dirty patch hash,
  release exe hash, runtime/runner/certifier/matrix/KB mapping hashes, gate
  level, case ID, parent summary hash, and timestamps.
- Any runtime, UI, runner, certifier, matrix, or KB mapping change makes older
  release exe and older gate evidence `stale` / `reference-only`.

This addendum does not authorize running targeted, `full16`, `formal403`,
packaging, commit, or push work.
