# Codex Usage Core Rules

This repo adopts the global Codex constitution from `E:\codex\AGENTS.md`.
This file is the repo-local non-secret rule anchor for device handoff. If the
global constitution changes, sync this file as a docs-only change.

## Source Of Truth

- Live Git state and actual files override handoff packets.
- Do not revert user or previous-thread changes unless the user explicitly asks.
- Do not put API keys, tokens, secrets, raw env files, raw prompt bodies,
  `source_register`, overlay JSON, or full raw KB rows in chat, repo docs, logs,
  or exported QA artifacts.
- Hope / Codex coordination defaults to Chinese. Code identifiers, commit
  messages, commands, and raw error text may stay in English.

## Thread Handoff Rule

When the user says the thread is near its context/background limit, at or above
7.5/10, 75%, 70%, or uses terms like `接力`, `切线程`, `上下文满了`,
`背景信息满了`, `额度满了`, or `新线程继续`, enter handoff mode.

Handoff mode:

- Do not start new large work.
- Finish or report any command already running.
- Generate a copy-ready section titled `给下一线程的接力指令`.
- Include current goal, completed work, unfinished work, workspace path, branch,
  key files, recently modified files, commands/tests and results, decisions,
  constraints, risks, and next steps.
- Tell the next thread to trust live file state over the packet if they differ.

## Copy-Ready Communication Rule

Cross-thread instructions, controller recaps, worker reports, handoff packets,
acceptance checklists, and archive packets should be directly pasteable.

Rules:

- Use a fenced `text` block by default.
- Keep prose outside the block to one short sentence when possible.
- Commands must be listed one per line.
- Forbidden items must be explicit under `Forbidden:` or an equivalent section.
- If instructions conflict with live Git, live file state, or live process
  state, trust live state first and report the mismatch.

## Controller / Worker Directive Label Rule

Controller-to-worker instruction blocks must start with:

```text
分线程指令：
线程动作：<新开 / 沿用 / 重启 / 待命 / 结束 / 归档>
指令发给：原名线程---><这段指令应粘贴到哪条线程；新开/重启时用 新线程>
沿用线程：对应线程---><继续使用的现有线程；没有则写 无>
更名线程：更名线程---><执行后应显示的线程名；不改名则写 不更名>
替代线程：<被替代或退休的旧线程；没有则写 无>
```

Worker-to-controller reports must start with:

```text
总控回报：
目标线程：总控线程
来源线程：<reporting worker thread>
```

Controller recaps use:

```text
总控复盘：
...
```

Rules:

- `线程动作` is mandatory and must be one of `新开`, `沿用`, `重启`, `待命`,
  `结束`, or `归档`.
- Put the thread that needs renaming first on `指令发给`.
- Put the action relationship on `沿用线程`.
- Put the post-action visible name on `更名线程`.
- Do not use vague routing labels such as `对应线程` or `目标线程` in place of
  the mandatory routing lines.

## Thread Action Declaration Rule

- `新开`: send to `新线程`; `沿用线程=无`; `替代线程=无`; `更名线程` is the new
  visible thread name.
- `沿用`: `指令发给` and `沿用线程` both name the existing thread;
  `更名线程` is the new visible name or `不更名`.
- `重启`: send to `新线程`; `沿用线程=无`; `替代线程` names the old thread being
  replaced; include what conclusions carry forward and what stale state to
  ignore.
- `待命`: send to the current thread; `沿用线程` names it; `更名线程` should carry
  the `（待命）` prefix.
- `结束`: send to the current thread; `沿用线程` names it; `更名线程` should carry
  the `（结束）` prefix.
- `归档`: send to the current thread; `沿用线程` names it; `更名线程` should carry
  the `（归档）` prefix.

## Thread Naming Rule

Use status-prefixed, workstream-aware names:

```text
（status）Project-WorkstreamThread-【Chinese target action】
```

Allowed prefixes:

- `（运行）`
- `（待命）`
- `（结束）`
- `（归档）`

Examples:

- `（运行）Hope桌面端-QA线程-【场景化四组trace复验】`
- `（运行）Hope桌面端-功能线程-【场景化改写与分镜收口】`
- `（待命）Hope桌面端-打包线程-【等待真实验收通过】`

Do not collapse distinct workstreams into a generic desktop thread name.

## Action-Only Dispatch Rule

When replying with worker-thread instructions, include only threads that need
immediate action, such as `（运行）` or `（归档）` threads.

Omit `（待命）` and `（结束）` threads unless their state changed or the user asks
for a full status review.

## Controller Cross-Thread Coordination Rule

Every controller decision must ask whether it affects other workstreams:

- runtime / response fields / validator / warning changes can affect contract,
  desktop consumption, QA, and packaging
- UI display changes affect UI and QA
- icon, installer, release, or auto-update changes affect packaging/release
- taxonomy, scene types, or routing changes affect QA and KB/routing planning

Only dispatch to actually affected threads. If a workstream does not need
action, mention the reason briefly in the recap rather than sending no-op
standby instructions.

## Core Challenger Rule

Every Hope / Codex project keeps one persistent `Core Challenger` role.

Role intent:

- challenge plans, outputs, milestones, and release claims
- try to falsify conclusions with live evidence
- trust actual Git state, files, artifacts, tests, and user-visible behavior
  over reports or verbal conclusions

Required behavior:

- ask what evidence could disprove the current conclusion
- identify the weakest link in the narrative
- identify the least self-consistent part of the evidence
- identify the point most likely to fail if the team keeps moving without a gate
- use a default five-round cadence across distinct issues
- if one issue needs more debate, allow two to three extra rounds, then record
  consensus, unresolved split, or downgraded conclusion and move on

Required output shape:

- question list
- synthesis / verdict
- executable next actions

Possible downgraded verdicts:

- direction not disproven
- evidence insufficient
- artifact loop not closed
- governance not aligned
- not ready to declare done

## Audit Specialist Rule

Every Hope / Codex project may use an independent `Audit Specialist` role.

Role intent:

- code-health auditing
- redundancy accumulation detection
- cleanup-candidate prioritization
- cleanup-gate proposals

Allowed by default:

- read-only scanning
- evidence listing
- risk and benefit ordering
- Top candidate lists
- retention lists
- one proposed follow-up gate

Forbidden by default:

- no deleting
- no archiving
- no editing runtime, UI, contracts, KB, fixtures, docs, or export artifacts
- no committing
- no pushing
- no long tests unless the controller opens an explicit gate
- no implementation work

First-round audit size:

- Top 10 candidates only.
- Each candidate should include path, type, evidence, recommendation, risk, and
  verification method.

Priority labels:

- `P0`: content that can mislead current execution
- `P1`: dead code, duplicate fallback, or old compatibility layers
- `P2`: historical handoffs, old docs, or old fixtures to archive before delete
- `P3`: readability or size issues only

Audit conclusions may propose only one cleanup gate at a time.

## Hope Desktop Release CDP QA Rule

Hope desktop-shell acceptance that requires the real WebView2 target must use
the real release shell. Browser preview, Vite, localhost mock, IPC-only, and
backend-only runs do not count as acceptance.

Default launch command:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\start-hope-release-cdp.ps1 -StopExisting -StopOnCdpFailure -WaitSeconds 20
```

Default cleanup command:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\start-hope-release-cdp.ps1 -StopOnly
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
shell lifecycle:

1. Start the real release shell with CDP.
2. Confirm CDP readiness.
3. Execute exactly one UI-driven test group.
4. Capture visible UI evidence, rows, screenshots or layout bounds, and
   sanitized diagnostics.
5. Immediately run `StopOnly`.
6. Confirm `hope-app.exe`, Hope-owned WebView2 processes, port `9224`, and the
   release exe lock are all clear.
7. Only then start the next group.

Do not run multiple QA groups in one long-lived release shell.

## Hope Desktop Fallback Trace Rule

When UI-driven QA needs to prove fallback behavior or response-vs-UI row
consistency, expose only sanitized diagnostics through stable QA trace surfaces:

- `window.__hopeQaTrace`
- `#hope-qa-trace[data-hope-qa-trace]`

Allowed trace fields:

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

Fallback and validator warnings must remain visible. Do not hide, rename, or
suppress `text_model_live_expand_fallback`,
`text_model_live_storyboard_fallback`, or `text_model_validator_failed`.

## Global Rules Multi-Project Sync Rule

When global Codex rules, controller coordination rules, thread naming rules,
report formats, or cross-thread rules change, the controller should consider
syncing them to:

- `E:\codex\hope`
- `E:\codex\hope-desktop-shell`
- `E:\codex\hope-kb`
- `E:\codex\hope-intake-app`

Sync rules:

- first do read-only Git verification
- do not directly modify dirty `AGENTS.md` unless the user explicitly confirms
  ownership
- prefer project-local docs such as `docs/codex-usage-core-rules.md`
- keep rules sync as docs-only commits
- never mix rules sync with runtime, UI, KB, fixture, package, or release changes
