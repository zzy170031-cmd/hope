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

## Repo-Local Launcher

Use the repo-local launcher from a checkout of `codex/desktop-shell`:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\start-hope-release-cdp.ps1 -Provider qwen -Model qwen-max -StopExisting -StopOnCdpFailure -WaitSeconds 20
```

Cleanup command:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\start-hope-release-cdp.ps1 -StopOnly
```

The legacy local launcher path may still exist on the original machine:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File E:\codex\tools\start-hope-release-cdp.ps1 -Provider qwen -Model qwen-max -StopExisting -StopOnCdpFailure -WaitSeconds 20
```

For new machines, prefer the repo-local launcher.

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

- Current QA live default model is `qwen-max`.
- Qwen API model candidate order is `qwen-max -> qvq-max-2025-03-25 -> qwen-math-turbo`.
- `qwen-plus` is retained only as a historical compatibility model, not the
  current default gate.
- Every QA run must explicitly confirm the active model in both the release
  shell startup command and the runner `--expected-model` argument.
- Do not silently switch models inside the UI-driven trace.
- Do not pass QA through fallback-only behavior or fallback pseudo-success.

Before running business QA, confirm provider status from the real release shell:

```text
provider=qwen
model=qwen-max
enabled=true
base_url_present=true
api_key_present=true
live_ready=true
status=enabled
storage=session-only
```

If `live_ready` is not true, stop the shell and do not run fallback-only QA.

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
node .\scripts\hope-ui-driven-trace-runner.mjs --case A_hot_blood_battle --scene 热血战斗 --source "废墟之上，主角单膝跪地，敌人缓步逼近。" --duration 15 --expected-provider qwen --expected-model qwen-max --compact 1
```

Each case must:

- start the real release shell through CDP
- drive the visible WebView2 UI
- select scene type
- select 15 seconds
- enter source text
- run expand story or expand script
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

- response rows hash equals UI rows hash
- `rows_match=true`
- `row_diffs=[]`
- duration is 15 seconds as 2 rows: `10 + 5`
- no `model_config_disabled`
- no `text_model_live_call_closed`
- provider is live-ready

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
- total: 403 cases

Long-text sample path on the original machine:

```text
C:\Users\Administrator\Desktop\九州剧本文字版2.docx
```

Do not run the 403-case matrix until the four-group trace gate, scene-type
rewrite gate, and cross rewrite drift smoke gate pass.

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

- four-group qwen trace passes
- 403-case matrix passes or the controller explicitly accepts a reduced gate
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
