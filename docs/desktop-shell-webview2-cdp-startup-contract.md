# Hope 桌面端 Shell / WebView2 / CDP 启动硬门契约
<small>Hope desktop shell / WebView2 / CDP startup hard gate contract</small>

状态：草案即执行。
<small>Status: draft but immediately executable.</small>

适用范围：`hope-desktop-shell` 的 release-like shell、WebView2、CDP 启动诊断、环境线程、后续 QA 线程。
<small>Scope: release-like shell, WebView2, CDP startup diagnostics, environment threads, and downstream QA threads in `hope-desktop-shell`.</small>

## 1. 契约目标
<small>1. Contract goal</small>

把 `HOPE-SHELL-004` 从“已文档化处理经验”升级成正式 hard gate。
<small>Upgrade `HOPE-SHELL-004` from informal handling notes into a formal hard gate.</small>

当 shell 启动 gate 不是绿色时，禁止把问题继续包装成 provider、validator、live3、pool、`full16`、`403-case` 或 package 讨论。
<small>When the shell startup gate is not green, the issue must not be repackaged as provider, validator, live3, pool, `full16`, `403-case`, or package discussion.</small>

## 2. Shell Gate 定义
<small>2. Shell gate definition</small>

正式 shell startup gate 只有一条：
<small>There is only one formal shell startup gate:</small>

- 启动命令必须是 `-LaunchDiagnosticOnly -WebView2ArgumentMode AppDefault`。
- 该 gate 只验证 shell / WebView2 / CDP 启动链路，不启动 provider / runner / 业务 QA。
- 只有 `AppDefault + LaunchDiagnosticOnly` 可以产出当前 shell gate 结论。

`Minimal` 与 `FullDefault` 只允许作为 diagnostic comparison。
<small>`Minimal` and `FullDefault` are diagnostic-comparison modes only.</small>

- 它们可用于对照 WebView2 参数差异。
- 它们不可用于开放 provider / runner / live3 / pool / `full16` / `403-case` / package。
- 它们的 artifact 默认是 `reference-only`，除非同一轮 `AppDefault` gate 已经通过且仅用于对照说明。

## 3. Gate 通过条件
<small>3. Gate pass conditions</small>

Shell gate 绿色必须同时满足：
<small>The shell gate is green only when all of the following hold:</small>

- clean workspace / worktree 运行必须显式传入 `-RepoRoot`，值为当前被测
  clean workspace 的绝对路径；当前已验收路径是
  `E:\codex\hope-desktop-shell-clean-9a8f4e0-r2`。
<small>Clean-workspace or worktree runs must explicitly pass `-RepoRoot` with the absolute path of the clean workspace under test; the currently accepted path is `E:\codex\hope-desktop-shell-clean-9a8f4e0-r2`.</small>
- 未传 `-RepoRoot` 且默认命中 `E:\codex\hope-desktop-shell` 的运行，只能标为
  `boundary-invalid` / `stale-reference`，不得作为当前 Shell Gate evidence。
<small>A run without `-RepoRoot` that defaults to `E:\codex\hope-desktop-shell` is `boundary-invalid` / `stale-reference` only and cannot be current Shell Gate evidence.</small>
- evidence 必须记录 workspace path、workspace `HEAD`、当前 `hope-app.exe`
  sha256、`/json/list` artifact、post-setup probe artifact、launch result
  artifact 与 paired `StopOnly` cleanup artifact。
<small>Evidence must record workspace path, workspace `HEAD`, current `hope-app.exe` sha256, `/json/list` artifact, post-setup probe artifact, launch result artifact, and paired `StopOnly` cleanup artifact.</small>
- fresh release-like binary 已被证明是当前运行源。
- 启动模式为 `AppDefault + LaunchDiagnosticOnly`。
- `ok=true`。
- `cdp_ready=true` 或至少 `/json/list` 可达且 target 已出现。
- `target_url=http://tauri.localhost/#/workbench`。
- `target_count>=1`，且 target title / url 与当前 shell 一致。
- 没有 WebView2 popup `0x80000003`，没有 `HRESULT(0x8000FFFF)`，且 popup 不显示不能单独算 pass。
<small>There must be no WebView2 popup `0x80000003`, no `HRESULT(0x8000FFFF)`, and merely hiding or not seeing a popup is not a pass.</small>
- 当前运行不依赖旧 profile、旧端口、旧 pid 或旧日志。
- 结束后 `StopOnly` cleanup 达到 `hope_app_remaining_pids=[]`、`webview2_remaining_pids=[]`、`port_released=true`、`exe_unlocked=true`。

## 4. Hard Blocker 分类
<small>4. Hard-blocker classification</small>

以下情况直接视为 shell / environment blocker：
<small>The following conditions are direct shell or environment blockers:</small>

- `target_count=0`。
- `cdp_ready=false`。
- target URL 落到 `http://127.0.0.1:5173/` 或其他 devUrl。
- diagnostic log 最后一条停在 `before_main_window_build`，且没有 `main_window_created`、`setup_exit`、`builder_run_returned_ok`。
- 观察到 WebView2 popup `0x80000003`。
- `before_main_window_build` 之后无 host-window 成功信号。
- 若已有 `main_window=created_in_setup`、`setup_exit`，但仍 `target_count=0` 且
  `cdp_ready=false`，视为 post-setup WebView2/CDP attach blocker；必须补
  post-setup sanitized diagnostics，再决定是否继续 provider / runner。

分类规则：
<small>Classification rules:</small>

- `before_main_window_build` 是 host-window build blocker。
- WebView2 popup `0x80000003` 必须转环境线程，不得继续转义成 provider 或 validator 根因。
- `127.0.0.1:5173` 是 devUrl / stale-build blocker，不是 provider 或 live3 evidence。
- `target_count=0` 时，后续 provider / runner 结论一律无效。

## 5. Freshness 与 Release-Like Binary 要求
<small>5. Freshness and release-like binary requirements</small>

以下任一条件成立时，旧 shell artifact 自动降级为 `stale` 或 `reference-only`：
<small>Any of the following automatically downgrades prior shell artifacts to `stale` or `reference-only`:</small>

- `app/src/main.rs`、release-shell 相关脚本、打包输入、`ui/dist` 或 Tauri build provenance 发生变化。
- 当前 `hope-app.exe` 无法证明匹配最新 release-like build 输出。
- `expected_target_url` 仍为 `tauri.localhost`，但实际落点是 devUrl。
- shell gate 使用了 `Minimal` / `FullDefault` 比较模式而非 `AppDefault` gate。

fresh binary 证明至少要包含：
<small>Fresh-binary proof must include at least:</small>

- 当前 `hope-app.exe` 路径与 LastWriteTime。
- 最新 release-like Tauri output 时间。
- `cargo:dev=false` 与 `custom_protocol` 证据。
- `ui/dist/index.html` 与 assets 存在性。

## 6. StopOnly Cleanup 硬要求
<small>6. StopOnly cleanup hard requirement</small>

每次 shell / CDP 诊断或 UI-driven QA 结束后都必须运行 `StopOnly`。
<small>Every shell/CDP diagnostic or UI-driven QA run must finish with `StopOnly`.</small>

以下情形不能报绿：
<small>The gate cannot be reported green under any of the following:</small>

- 未运行 `StopOnly`。
- 运行了 `StopOnly` 但没有回报 cleanup 结果。
- `hope-app` 或 Hope-owned WebView2 进程仍残留。
- 端口 `9224` 未释放。
- exe 仍被锁定。

## 7. Reference-Only Artifact 规则
<small>7. Reference-only artifact rules</small>

以下 artifact 只能作为 `reference-only`：
<small>The following artifacts are `reference-only` only:</small>

- `Minimal` 或 `FullDefault` 模式产物。
- 任何来自 stale binary、stale profile、stale port、stale logs 的 shell 诊断。
- 仅凭 popup 截图或肉眼观察、缺 `/json/list` / PID / cleanup 摘要的报告。
- `target_url=127.0.0.1:5173` 的 wrong-target artifact。
- 没有 paired `StopOnly` 结果的 launch artifact。
- Hope-2-v0、`E:\codex\hope-local-stale-archive`、旧 `%TEMP%` artifact、旧
  CDP profile、旧 no-`RepoRoot` run，均不得作为旧 Hope current gate evidence。
<small>Hope-2-v0, `E:\codex\hope-local-stale-archive`, old `%TEMP%` artifacts, old CDP profiles, and old no-`RepoRoot` runs must not be used as current gate evidence for old Hope.</small>

当前 2026-05-04 的 `launch-diagnostic-minimal-noproxy-reproof.json` 与同轮 `stoponly-final.json` 可用于说明第 5 次复发形态，但仍属于 comparison/reference-only，不得直接解锁 provider 或更高 gate。
<small>The 2026-05-04 `launch-diagnostic-minimal-noproxy-reproof.json` and its paired `stoponly-final.json` may explain the fifth recurrence pattern, but they remain comparison/reference-only and cannot unlock provider or higher gates.</small>

## 8. 环境线程与后续 QA 线程入口
<small>8. Entry conditions for environment and downstream QA threads</small>

环境线程必须先做：
<small>The environment thread must do this first:</small>

1. 用 `AppDefault + LaunchDiagnosticOnly` 重新取 fresh shell gate evidence。
2. 证明 binary freshness。
3. 证明 `target_url=http://tauri.localhost/#/workbench`。
4. 证明 `StopOnly` cleanup 完整。

只有 shell gate 绿色后，后续实现 QA 线程才允许继续：
<small>Only after the shell gate is green may downstream implementation-QA work continue into:</small>

- provider hard gate
- single case
- `qwen3.6-plus` live3 3-case
- required text model pool live3
- `full16`
- `403-case`
- package / release preflight

后续进入 `qwen3.6-plus` live3 或 required text model pool live3 时，只要需要启动
shell，就必须继续显式传入当前被测 clean workspace 的 `-RepoRoot`，并在每个
shell 生命周期后执行和回报 `StopOnly` cleanup。
<small>When later entering `qwen3.6-plus` live3 or required text model pool live3, any shell start must still pass `-RepoRoot` for the clean workspace under test and must execute and report `StopOnly` cleanup after every shell lifecycle.</small>

## 9. Launcher Proxy Isolation Addendum
<small>9. Launcher proxy isolation addendum</small>

The `AppDefault + LaunchDiagnosticOnly` shell gate remains the only formal
shell gate. The 2026-05-07 recurrence does not replace `HOPE-SHELL-004`; it
adds a launcher environment-isolation rule inside the same gate.

- Before starting `hope-app.exe`, every release shell / WebView2 / CDP launch
  path must clear `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `NO_PROXY`,
  `http_proxy`, `https_proxy`, `all_proxy`, and `no_proxy` from the launch
  process environment.
- `-NoProxy` is QA/provider semantics and evidence only. It must not be the
  only trigger for WebView2 environment isolation.
- Accepted artifacts must record only sanitized evidence:
  `launcher_process_env_proxy_present_before`,
  `webview2_launch_env_proxy_cleared`, `app_process_env_proxy_present`, and
  `no_proxy_loopback_only`.
- Before starting `hope-app.exe`, the launcher must be able to query WebView2
  process command lines. If `webview2_process_query_capability.query_ok=false`
  or the query reports access denied, the launcher must fail closed before
  starting the release shell. Otherwise `StopOnly` cannot prove that a WebView2
  popup / crash lifecycle was cleaned.
- WebView2 cleanup scope is Hope-owned only: `hope-app.exe` plus WebView2
  processes whose command line names `hope-app.exe` or the
  `hope-webview2-cdp` temp profile. System WebView2 processes owned by Windows
  surfaces such as `SearchHost.exe` or `Widgets.exe` must be recorded as
  `non_hope_process_count` / `non_hope_webview_exe_names`, not killed and not
  reported as Hope release-shell residue.
- Accepted artifacts must not include raw proxy values, raw env files, API
  keys, tokens, or secrets.
- `AppDefault` must not use `--noerrdialogs`, `--disable-breakpad`, or
  `--disable-crash-reporter`; hiding a popup is not a startup pass.
- `StopOnly` clean is cleanup evidence only. It is not a shell pass unless it
  is paired with a passing `AppDefault + LaunchDiagnosticOnly` launch artifact.
- If `0x80000003` or `HRESULT(0x8000FFFF)` appears, stop business gates and
  route the run to shell/CDP blocker handling. Do not continue targeted,
  `full16`, `403-case`, provider, runner, or package work from that lifecycle.

2026-05-07 verification evidence for this addendum:

- `AppDefault + LaunchDiagnosticOnly`
- `ok=true`
- `cdp_ready=true`
- `target_url=http://tauri.localhost/#/workbench`
- `webview2_popup_suppression_detected=false`
- `webview2_app_popup_suppression_arg_present=false`
- `webview2_launch_env_proxy_cleared=true`
- `app_process_env_proxy_present=false`
- paired `StopOnly` clean

## 10. 2026-05-08 Business Gate Preflight Link
<small>10. 2026-05-08 business gate preflight link</small>

The shell/CDP contract now inherits the source and artifact freshness rules in:
<small>The shell/CDP contract now inherits the source and artifact freshness rules in:</small>

```text
docs/hope-qa-evidence-freshness-contract.md
```

Before targeted, `full16`, or `formal403` starts, the launcher-side shell gate
must be paired with current artifact identity:
<small>Before targeted, `full16`, or `formal403` starts, the launcher-side shell gate must be paired with current artifact identity:</small>

- current workspace
- current branch / `HEAD`
- current release exe SHA256
- current `runtime_rs_sha256`
- current runner/certifier/matrix/KB mapping hashes when applicable
- paired `StopOnly`

If WebView2 `0x80000003`, `HRESULT(0x8000FFFF)`, `cdp_ready=false`, wrong
target, missing `tauri.localhost`, or missing `StopOnly` appears, stop business
gates immediately. Do not continue with provider, runner, targeted, `full16`,
`formal403`, or package work from that lifecycle.
<small>If WebView2 `0x80000003`, `HRESULT(0x8000FFFF)`, `cdp_ready=false`, wrong target, missing `tauri.localhost`, or missing `StopOnly` appears, stop business gates immediately.</small>
