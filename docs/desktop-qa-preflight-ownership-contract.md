# Hope 桌面端 QA Preflight 与 Dirty Ownership 契约
<small>Hope desktop QA preflight and dirty-ownership contract</small>

状态：草案即执行。
<small>Status: draft but immediately executable.</small>

适用范围：Tauri build、release-like shell QA、环境线程、实现 QA 线程、handoff 线程。
<small>Scope: Tauri builds, release-like shell QA, environment threads, implementation-QA threads, and handoff threads.</small>

## 1. 契约目标
<small>1. Contract goal</small>

把 `HOPE-BUILD-010` 与 `HOPE-OWN-011` 从“知道有风险”升级成强制 preflight 与 ownership 边界。
<small>Upgrade `HOPE-BUILD-010` and `HOPE-OWN-011` from known risks into mandatory preflight and ownership boundaries.</small>

## 2. 每次 QA / Build 前必须记录
<small>2. Mandatory recording before every QA or build step</small>

进入任何会触发 Tauri CLI、release shell、CDP 或 runner 的动作前，必须记录：
<small>Before any step that can trigger Tauri CLI, the release shell, CDP, or a runner, record:</small>

- `git status --short --branch`
- `git diff --cached --name-only`
- `git diff --name-only`
- `git diff -- app/Cargo.toml`
- 本线程 allowed files / forbidden files
- 当前 dirty ownership 说明

如果 dispatch 没明确打开某个 tracked file，默认该文件禁止修改。
<small>If the dispatch does not explicitly open a tracked file, that file is forbidden by default.</small>

## 3. `app/Cargo.toml` Preflight / Postflight
<small>3. `app/Cargo.toml` preflight and postflight</small>

每次 Tauri build 前后都必须单独记录 `app/Cargo.toml` diff。
<small>Record the `app/Cargo.toml` diff separately before and after every Tauri build.</small>

允许的白名单只有一类：
<small>There is only one whitelist class:</small>

- Tauri CLI 自动 rewrite `features=[]` 的已知模式。

白名单处理规则：
<small>Whitelist handling rules:</small>

- 即使是白名单 rewrite，也必须明确记录为 `whitelist-observed`。
- 白名单 rewrite 不等于问题关闭，也不等于 QA 线程获得修改 source 的权限。
- 如果 dispatch 没显式开放 `app/Cargo.toml` ownership，白名单 rewrite 也要立刻报告并停止当前 gate 结论。
- 只有当 controller 明确开放该边界时，线程才可把白名单 rewrite 记录为已观测比较项，而不是立即当成未知污染。

超出白名单的任何 diff 都是 `boundary-blocker`。
<small>Any diff outside the whitelist is a `boundary-blocker`.</small>

## 4. QA 线程不得越界修改 Source / Tracked Files
<small>4. QA threads must not cross into source/tracked files</small>

QA 线程的默认职责是读取、运行、取证、回报，不是顺手修代码。
<small>The default role of a QA thread is to read, run, capture evidence, and report, not to “fix something quickly.”</small>

除非 dispatch 明确授权，QA 线程不得修改：
<small>Unless the dispatch explicitly authorizes them, QA threads must not modify:</small>

- `app/src/**`
- `scripts/**`
- `tests/**`
- `app/Cargo.toml`
- 任何其他 tracked source / runtime / test 文件

## 5. Dirty Ownership 记录要求
<small>5. Dirty-ownership recording requirements</small>

每次 QA 前后都必须记录：
<small>Record the following before and after each QA step:</small>

- 哪些 tracked files 已经是脏的。
- 这些脏文件属于哪个线程或哪个已知 ownership bucket。
- 本线程本次允许新增的 dirty files 是什么。
- 本线程本次禁止触碰的 dirty files 是什么。

如果无法解释某个 tracked diff 的归属，当前 QA 结论不得进入 gate。
<small>If the ownership of a tracked diff cannot be explained, the current QA result must not enter a gate.</small>

## 6. Allowed / Forbidden Files 必须与 Dispatch 对齐
<small>6. Allowed and forbidden files must match the dispatch</small>

dispatch 里的文件边界是执行契约，不是提示语。
<small>The file boundary in the dispatch is an execution contract, not a suggestion.</small>

回报里至少要写清：
<small>The report must state at least:</small>

- dispatch 允许的文件
- dispatch 禁止的文件
- 实际被读取的文件
- 实际被修改的文件
- 是否有 boundary mismatch

## 7. 触发停止的 Ownership Blocker
<small>7. Ownership blockers that force a stop</small>

以下任一情况出现时必须停下：
<small>Stop immediately when any of the following occurs:</small>

- 未授权的 `app/Cargo.toml` diff
- 超白名单 diff
- 未授权的 source / tracked file 修改
- 无法解释的 dirty ownership
- 需要通过修改 source / scripts / tests 才能继续 QA

## 8. Fresh Closure Evidence 要求
<small>8. Fresh closure evidence requirements</small>

`HOPE-BUILD-010` 或 `HOPE-OWN-011` 不能只靠“这次没再脏”关闭。
<small>`HOPE-BUILD-010` and `HOPE-OWN-011` cannot close just because “nothing got dirtier this time.”</small>

closure 至少需要：
<small>Closure requires at least:</small>

- 一次 fresh QA / environment thread 全流程遵守本契约。
- preflight 与 postflight 记录齐全。
- `app/Cargo.toml` diff 已被白名单解释或完全未触发。
- 所有 tracked diff 的 ownership 都清晰可追。
