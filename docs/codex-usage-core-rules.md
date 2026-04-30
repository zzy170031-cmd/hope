# Codex Usage Core Rules

This repo adopts the global Codex constitution from `E:\codex\AGENTS.md`.

Update strategy:

- `E:\codex\AGENTS.md` is the source of truth for global Hope / Codex rules.
- This repo-level file exists so desktop-shell threads have a local rule anchor
  inside the repo.
- When the global constitution changes, sync this file as a docs-only change or
  record a pending sync state if the repo is intentionally dirty or otherwise
  blocked.

## Core Challenger Rule

In addition to the five-agent execution model, this repo adopts one persistent
`Core Challenger` role.

Role intent:

- challenge plans, outputs, milestones, and release claims
- pressure the project forward through falsification instead of optimism
- trust live Git state, actual files, actual artifacts, actual tests, and
  actual user-visible behavior over prior reports or verbal conclusions

Required behaviors:

- ask what evidence could disprove the current conclusion
- identify the weakest link in the current narrative
- identify the least self-consistent part of the current evidence
- identify the point most likely to fail if the team keeps moving without
  another gate
- use a default cadence of five rounds on different issues
- if one issue does not reach consensus in one round, allow two to three
  additional rounds on that issue
- do not stay on one issue indefinitely; after the allowed extension, record a
  consensus, record the unresolved split, or downgrade the conclusion and move
  on
- optimize for breadth across distinct high-risk issues instead of repetitive
  pressure after the evidence stops improving

Required output shape:

- question list
- synthesis / verdict
- executable next actions

If evidence is insufficient, the core challenger may downgrade the conclusion
to:

- direction not disproven
- evidence insufficient
- artifact loop not closed
- governance not aligned
- not ready to declare done

Relationship to the five-agent model:

- the five-agent model optimizes bounded parallel execution
- the core challenger optimizes falsification, boundary checking, and release
  trust

## Controller Thread Instruction Format Rule

Hope / Codex coordination messages that are intended to be pasted into another
thread must use a Markdown `text` code block and begin with an explicit routing
header.

Controller-to-worker instruction blocks must start with:

```text
分线程指令：
线程动作：<新开 / 沿用 / 重启 / 待命 / 结束 / 归档>
指令发给：原名线程---><thread that receives this pasted instruction>
沿用线程：对应线程---><existing thread to continue, or 无>
更名线程：更名线程---><post-action visible thread name, or 不更名>
替代线程：<old thread replaced or retired, or 无>
```

Worker-to-controller reports must start with:

```text
总控回报：
目标线程：总控线程
来源线程：<reporting worker thread>
```

Rules:

- Put the thread that needs renaming first on the `指令发给` line.
- Put the action relationship on the `沿用线程` line.
- Put the post-action display name on the `更名线程` line.
- Keep `线程动作` one of: `新开`, `沿用`, `重启`, `待命`, `结束`, or `归档`.
- Do not use vague routing labels such as `对应线程` or `目标线程` without the
  explicit arrow format above.
- Keep thread names in the form:
  `（status）Project-WorkstreamThread-【Chinese target action】`.
- Use status prefixes consistently: `（运行）`, `（待命）`, `（结束）`,
  `（归档）`.
- Include only threads that need immediate action when dispatching controller
  instructions. Omit no-op standby or ended threads unless their state changed
  or the user asks for a full status review.

## Copy-Ready Communication Rule

Cross-thread instructions, controller recaps, worker reports, handoff packets,
acceptance checklists, and archive packets should be directly pasteable.

Rules:

- Use a fenced `text` block by default.
- Keep prose outside the block to one short sentence when possible.
- Commands must be listed one per line.
- Forbidden items must be explicit under `Forbidden:` or an equivalent section.
- Never include API keys, tokens, secrets, raw env files, full raw KB rows,
  raw `prompt_body`, `source_register`, or overlay JSON in chat, reports,
  exports, logs, or repo docs.
- If instructions conflict with live Git, live file state, or live process
  state, trust live state first and report the mismatch.
- Do not revert user or previous-thread changes unless the user explicitly
  asks.

## Hope Desktop Release CDP QA Rule

Hope desktop-shell acceptance that requires the real WebView2 target must use
the real release shell. Browser preview, Vite, localhost mock, IPC-only, and
backend-only runs do not count as acceptance.

Default launch command:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File E:\codex\tools\start-hope-release-cdp.ps1 -StopExisting -StopOnCdpFailure -WaitSeconds 20
```

Default cleanup command:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File E:\codex\tools\start-hope-release-cdp.ps1 -StopOnly
```

Acceptance requires:

- `ok=true`
- `cdp_ready=true`
- `target_url=http://tauri.localhost/#/workbench`
- `target_title=Hope UI Skeleton`
- `/json/list` is reachable while the shell is running
- `started_pid` still exists before cleanup

Cleanup requires:

- `ok=true`
- `hope_app_remaining_pids=[]` or `hope_app_remaining_count=0`
- `webview2_remaining_pids=[]` or `webview2_remaining_count=0`
- `port_released=true`
- `exe_unlocked=true`

## Hope Desktop One-Group-One-Shell QA Rule

For Hope desktop-shell UI-driven QA, each test group must use its own release
shell lifecycle.

Required per-group flow:

1. Start the real release shell with CDP.
2. Confirm CDP readiness.
3. Execute exactly one UI-driven test group.
4. Capture visible UI evidence, rows, screenshots or layout bounds, and
   sanitized diagnostics.
5. Immediately run `StopOnly`.
6. Confirm `hope-app.exe`, Hope-owned WebView2 processes, port `9224`, and the
   release exe lock are all clear.
7. Only then start the next group.

Do not run multiple QA groups in one long-lived release shell and clean up only
at the end.

If WebView2 or CDP startup fails:

- Stop subsequent groups immediately.
- Do not retry blindly.
- Do not bypass CDP.
- Report the launcher JSON fields, sanitized diagnostic log tail, WebView2
  profile state, `webview2_environment_not_created_or_crashed`, and Windows
  event delta where available.

## Hope Desktop Fallback Trace Rule

When UI-driven QA needs to prove fallback behavior or response-vs-UI row
consistency, the app should expose only sanitized diagnostics through a stable
QA trace surface such as `window.__hopeQaTrace` or
`#hope-qa-trace[data-hope-qa-trace]`.

Allowed trace fields include:

- command
- status
- warning codes and sanitized warning messages
- fallback reason
- validator reason
- response rows hash
- UI rows hash
- rows match flag
- row diffs
- normalized row fields needed for verification

Forbidden trace content:

- API keys, tokens, secrets, raw env file contents
- raw request payloads
- raw `prompt_body`
- raw `source_register`
- overlay JSON
- full raw KB rows

Fallback and validator warnings must remain visible as evidence. Do not hide,
rename, or suppress `text_model_live_expand_fallback`,
`text_model_live_storyboard_fallback`, or `text_model_validator_failed` to make
QA appear green.
