# 项目进度查错文档
<small>Project Progress Error Review Document</small>

最后更新时间：2026-05-04
<small>Last Updated: 2026-05-04</small>

写作规则来源：
<small>Authoring Rule Source</small>

- `C:\Users\Administrator\Desktop\hope\【错题本书写规则】.docx` 是本文件的写作 source-of-truth。
<small>`C:\Users\Administrator\Desktop\hope\【错题本书写规则】.docx` is the source-of-truth for this document.</small>
- 本文件先写 Markdown，再同步生成 DOCX；两者内容必须一致。
<small>This document is authored in Markdown first and then synced into DOCX; both copies must stay identical.</small>
- 本文件只使用脱敏命令、脱敏路径、脱敏状态与文档级证据；不输出 raw env、API key、raw prompt、raw provider response、source_register 或 overlay JSON。
<small>This document uses sanitized commands, sanitized paths, sanitized states, and document-level evidence only; no raw env, API key, raw prompt, raw provider response, source_register, or overlay JSON is exposed.</small>

## 1. 文档目的与阅读对象
<small>1. Purpose and Audience</small>

本文件不是项目日志摘要，而是按「项目进度查错」视角整理 Hope 桌面端主线推进中已经暴露、已经关闭、仍然阻断、容易误判与下一 gate 前必须复核的问题。
<small>This document is not a project log recap; it reorganizes Hope Desktop mainline issues from the perspective of progress-oriented error review.</small>

阅读对象：
<small>Audience</small>

- 当前 Hope 主线推进线程与接力线程。
<small>Current Hope mainline threads and handoff threads.</small>
- 负责 gate 判断、release-QA、validator 契约与文档治理的评审者。
<small>Reviewers responsible for gate judgment, release QA, validator contracts, and document governance.</small>
- 未来换设备、换线程、换上下文后需要快速恢复判断边界的人。
<small>Future operators who need to recover the decision boundary after a device, thread, or context switch.</small>

写作伦理：
<small>Authoring Ethics</small>

- 不甩锅：本文件把个人失误改写为系统未强制提醒、流程未设置检查点、dispatch 未明确授权。
<small>No blame: personal lapses are rewritten as missing system reminders, missing checkpoints, or missing dispatch authorization.</small>
- 不掩盖：失败尝试写进「解决过程」，不只保留最后一步。
<small>No cover-up: failed attempts are recorded in Resolution, not hidden behind the final step.</small>
- 不虚构：凡是没有 fresh 复证通过的项，只能写成 open 或 pending，不能写成已修复。
<small>No fabrication: any item without fresh re-verification stays open or pending, never “fixed.”</small>
- 不主观：不用「我觉得」「应该是」「估计」，只写命令、状态、文档条款与明确推断边界。
<small>No subjectivity: no “I think,” “probably,” or “should be”; only commands, states, cited rules, and bounded inference.</small>

## 2. 当前项目进度摘要
<small>2. Current Project Progress Summary</small>

当前 live 状态摘要：
<small>Current Live-State Summary</small>

- Hope 参考仓库当前分支是 `codex/desktop-shell`，当前 `HEAD` 与 `origin/codex/desktop-shell` 都是 `02a5ee0`。
<small>The Hope reference repo is on `codex/desktop-shell`, and both `HEAD` and `origin/codex/desktop-shell` are `02a5ee0`.</small>
- 当前无 staged 变更，但工作树仍含 tracked dirty files：`app/src/main.rs`、`app/src/runtime.rs`、`docs/desktop-release-qa-handoff.md`、`scripts/hope-model-contract-certifier.mjs`、`scripts/hope-ui-driven-trace-runner.mjs`、`scripts/start-hope-release-cdp.ps1`、`tests/qa/desktop-model-certification-matrix.json`。
<small>There are no staged changes, but the working tree still contains tracked dirty files in source, docs, scripts, and tests.</small>
- 已经明确关闭的主线问题集中在：provider 失败分类、artifact 自描述、shell/runtime 证据边界、release provenance、proxy 清理、模型分配与 secret hygiene。
<small>The already-closed cluster covers provider failure taxonomy, artifact self-description, shell/runtime evidence boundaries, release provenance, proxy cleanup, model assignment, and secret hygiene.</small>
- 仍未关闭的主线问题集中在：`主角初 -> 初现 -> 主角` 的 token/entity classification 与 source role grounding、`app/Cargo.toml` preflight 风险、QA dirty ownership 边界，以及当前 gate 不能被单案结果过读。
<small>The still-open cluster covers token/entity classification plus source-role grounding, the `app/Cargo.toml` preflight risk, QA dirty-ownership boundaries, and the rule that one case cannot be over-read as a higher gate pass.</small>

当前接受的最稳结论：
<small>Latest Accepted Conclusions</small>

- 本地 targeted tests 通过，不等于 fresh provider QA 通过。
<small>Passing local targeted tests does not equal passing fresh provider QA.</small>
- `certifier_ok=true` 不等于 case passed。
<small>`certifier_ok=true` does not equal case passed.</small>
- 旧 artifact、旧 binary、旧 profile、旧 env 只能是 `reference-only` 或 `stale`，不能驱动当前 gate。
<small>Old artifacts, binaries, profiles, and env state can only be `reference-only` or `stale`, never current gate drivers.</small>

## 3. gate 层级与当前阻断状态
<small>3. Gate Hierarchy and Current Blockers</small>

当前 gate 层级：
<small>Current Gate Ladder</small>

- 单案本地验证。
<small>Single-case local validation.</small>
- `qwen3.6-plus` live3 3-case。
<small>`qwen3.6-plus` live3 3-case.</small>
- required text model pool live3。
<small>Required text model pool live3.</small>
- `full16`。
<small>`full16`.</small>
- `403-case`。
<small>`403-case`.</small>
- `package`。
<small>`package`.</small>

当前阻断说明：
<small>Current Blocking Notes</small>

- 当前没有 fresh 回报证明 `qwen3.6-plus` live3 3-case 已通过，因此该层级只能写为 pending。
<small>There is no fresh report proving `qwen3.6-plus` live3 3-case passed, so this layer remains pending.</small>
- required text model pool live3 仍未闭合，因此 `full16`、`403-case` 与 `package` 不能被提前解锁。
<small>The required text model pool live3 is still open, so `full16`, `403-case`, and `package` remain locked.</small>
- `主角初 -> 初现 -> 主角` 相关 validator/source grounding 项仍缺 post-fix fresh provider artifact，因此不能被描述为 closed。
<small>The `主角初 -> 初现 -> 主角` validator and source-grounding issue still lacks a post-fix fresh provider artifact and cannot be marked closed.</small>
- `app/Cargo.toml` 被 Tauri CLI 自动改写 `features=[]` 的风险仍缺 preflight guard，因此 source-tracked file 安全边界仍未完全收口。
<small>The `app/Cargo.toml` auto-rewrite risk still lacks a preflight guard, so the source-tracked-file safety boundary is not fully closed.</small>

进 gate 前必须守住的解释边界：
<small>Interpretation Boundaries Before Any Gate Advance</small>

- 不得把单案成功写成 live3 通过。
<small>Do not rewrite a single-case success as a live3 pass.</small>
- 不得把 targeted tests passed 写成 fresh QA passed。
<small>Do not rewrite targeted tests passed as fresh QA passed.</small>
- 不得把旧 artifact 写成 fresh evidence。
<small>Do not rewrite old artifacts as fresh evidence.</small>
- 不得把 plain Cargo build 写成 Tauri release-like provenance。
<small>Do not rewrite plain Cargo build as Tauri release-like provenance.</small>

## 4. 已关闭错题索引
<small>4. Closed Issue Index</small>

- `HOPE-PROV-001`：provider timeout 双源漂移；关闭依据是 release-QA handoff 已把 QA timeout override 与产品默认行为分开定义。
<small>`HOPE-PROV-001`: provider timeout dual-source drift; closed because the release-QA handoff now separates QA timeout override from product-default behavior.</small>
- `HOPE-PROV-002`：provider retry telemetry 与 validator hard-fail 混淆；关闭依据是 matrix stage 名称与 hard-fail 分类已固定。
<small>`HOPE-PROV-002`: provider retry telemetry vs validator hard-fail confusion; closed because matrix stage naming and hard-fail taxonomy are now fixed.</small>
- `HOPE-EVID-003`：raw compact evidence 字段不自含；关闭依据是 raw `provider_retry_evidence` 回填与 fresh raw telemetry 复证。
<small>`HOPE-EVID-003`: raw compact evidence was not self-contained; closed by raw `provider_retry_evidence` backfill and fresh raw telemetry re-verification.</small>
- `HOPE-SHELL-004`：WebView2 弹窗、child 未生成、zero target、devUrl target mismatch；Shell Gate 已在 clean workspace `E:\codex\hope-desktop-shell-clean-9a8f4e0-r2`、anchor `9a8f4e0349fa56a7cc57fb2e3817fabaa75afb6e`、release exe hash `E3BB9738851AF64FBE642517CA8DE2BEE82B23F67B4B19E9C8C817E8C6AD6D26`、valid runs `hope-cdp-20260505-193016-26016` 与 `hope-cdp-20260505-193110-27216` 上窄口径 stabilized；provider/live3/full16/403-case/package 仍未解锁。
<small>`HOPE-SHELL-004`: WebView2 popup, missing child, zero target, and devUrl target mismatch; the Shell Gate is narrowly stabilized in clean workspace `E:\codex\hope-desktop-shell-clean-9a8f4e0-r2` at anchor `9a8f4e0349fa56a7cc57fb2e3817fabaa75afb6e`, release exe hash `E3BB9738851AF64FBE642517CA8DE2BEE82B23F67B4B19E9C8C817E8C6AD6D26`, and valid runs `hope-cdp-20260505-193016-26016` plus `hope-cdp-20260505-193110-27216`; provider/live3/full16/403-case/package remain locked.</small>
- `HOPE-EVID-005`：stale artifact / stale binary / stale profile / stale env 污染 gate；关闭依据是 stale evidence 规则已经写入全局与项目规则。
<small>`HOPE-EVID-005`: stale artifact, stale binary, stale profile, and stale env gate pollution; closed by global and project stale-evidence rules.</small>
- `HOPE-BUILD-006`：direct cargo build 与 Tauri release-like provenance 混淆；关闭依据是 plain Cargo output 已被明确排除出 accepted release provenance。
<small>`HOPE-BUILD-006`: direct Cargo build vs Tauri release-like provenance confusion; closed because plain Cargo output is explicitly excluded from accepted release provenance.</small>
- `HOPE-CONTRACT-008`：prompt_text boundary / accepted snapshot / StoryFactFrame 下游证据缺口；关闭依据是 matrix 已要求 hash、boundary、rows_match 与 row_diffs。
<small>`HOPE-CONTRACT-008`: prompt_text boundary, accepted snapshot, and StoryFactFrame evidence gap; closed because the matrix now requires hash, boundary, rows_match, and row_diffs.</small>
- `HOPE-PROV-009`：proxy env contamination；关闭依据是 sanitized proxy before/after-clear 记录与 clean rerun 边界。
<small>`HOPE-PROV-009`: proxy env contamination; closed by sanitized proxy before/after-clear records and clean-rerun boundaries.</small>
- `HOPE-GOV-012`：bounded `/goal` 未及时使用导致反复沟通；关闭依据是全局与 repo 规则已写入 repeated-issue closed-loop and bounded-goal rule。
<small>`HOPE-GOV-012`: delayed bounded `/goal` use causing repeated communication; closed because global and repo rules now codify repeated-issue closed-loop and bounded-goal escalation.</small>
- `HOPE-GOV-013`：模型选择强制规则；关闭依据是 dispatch 必须写死模型，线程回报必须写实际使用模型。
<small>`HOPE-GOV-013`: model assignment enforcement; closed because dispatch must fix the model and the report must state the actual model used.</small>
- `HOPE-SEC-015`：secret / env / API key 本地化与 Git 泄露防线；关闭依据是 sanitized-only disclosure 规则已经固定。
<small>`HOPE-SEC-015`: local secret/env/API key discipline and Git leak barrier; closed because sanitized-only disclosure is already codified.</small>

## 5. 未关闭错题索引
<small>5. Open Issue Index</small>

- `HOPE-CONTRACT-007`：`主角初 -> 初现 -> 主角` 暴露出的 token/entity classification 与 source role grounding 问题；缺 post-fix fresh provider artifact；下一步是 release-like rebuild 后 fresh QA rerun。
<small>`HOPE-CONTRACT-007`: token/entity classification and source-role grounding issue exposed by `主角初 -> 初现 -> 主角`; a source-grounding contract now exists, but the post-fix fresh provider artifact is still missing; next step is fresh QA rerun after a release-like rebuild.</small>
- `HOPE-BUILD-010`：`app/Cargo.toml` 被 Tauri CLI 自动改写 `features=[]` 的 preflight 风险；已新增 QA preflight / ownership contract，但缺 fresh preflight guard 执行证据，因此状态维持 `contract-pending`。
<small>`HOPE-BUILD-010`: preflight risk that Tauri CLI may rewrite `app/Cargo.toml` to `features=[]`; a QA preflight/ownership contract now exists, but fresh guard-execution evidence is still missing, so the status remains `contract-pending`.</small>
- `HOPE-OWN-011`：QA 线程越界修改 source/tracked 文件的 dirty ownership 风险；已新增 QA ownership contract，但缺 fresh “从启动到回报都守边界”的闭环样例，因此状态维持 `contract-pending`。
<small>`HOPE-OWN-011`: dirty-ownership risk when QA threads cross into source/tracked files; a QA ownership contract now exists, but a fresh end-to-end bounded example is still missing, so the status remains `contract-pending`.</small>
- `HOPE-GATE-016`：当前 gate 层级容易被单案结果过读；已新增 gate evidence ladder contract，但缺 fresh `qwen3.6-plus` live3 3-case 与 required text model pool live3 通过回报；下一步仍是按层级继续取 fresh evidence。
<small>`HOPE-GATE-016`: the current gate ladder is still vulnerable to over-reading from a single case; a gate-evidence ladder contract now exists, but fresh `qwen3.6-plus` live3 3-case and required-pool live3 evidence are still missing, so the next step remains collecting fresh evidence layer by layer.</small>

## 6. reference-only 资料索引
<small>6. Reference-only Evidence Index</small>

- `HOPE-SAMPLE-014`：21/27 样本集当前只能作为 QA/reference 材料，不能作为 runtime truth 或 formal import gate evidence。
<small>`HOPE-SAMPLE-014`: the 21/27 sample sets can currently serve only as QA/reference material, not runtime truth or formal-import gate evidence.</small>
- `HOPE-SHELL-004` 的 2026-05-04 `launch-diagnostic-minimal-noproxy-reproof.json` 与配套 `stoponly-final.json` 只可作为第 5 次复发的 comparison/reference-only；Shell Gate 当前依据是 clean workspace `E:\codex\hope-desktop-shell-clean-9a8f4e0-r2`、anchor `9a8f4e0349fa56a7cc57fb2e3817fabaa75afb6e`、release exe hash `E3BB9738851AF64FBE642517CA8DE2BEE82B23F67B4B19E9C8C817E8C6AD6D26` 与 valid runs `hope-cdp-20260505-193016-26016` / `hope-cdp-20260505-193110-27216`，不是旧 artifact。
<small>The 2026-05-04 `launch-diagnostic-minimal-noproxy-reproof.json` and paired `stoponly-final.json` for `HOPE-SHELL-004` are comparison/reference-only for the fifth recurrence; the current Shell Gate basis is clean workspace `E:\codex\hope-desktop-shell-clean-9a8f4e0-r2`, anchor `9a8f4e0349fa56a7cc57fb2e3817fabaa75afb6e`, release exe hash `E3BB9738851AF64FBE642517CA8DE2BEE82B23F67B4B19E9C8C817E8C6AD6D26`, and valid runs `hope-cdp-20260505-193016-26016` / `hope-cdp-20260505-193110-27216`, not the old artifact.</small>
- `HOPE-CONTRACT-007` 的 pre-fix validator review 与 pre-fix provider artifact 只能作为参考背景，不能关闭当前 token/source grounding 项。
<small>The pre-fix validator review and pre-fix provider artifacts for `HOPE-CONTRACT-007` are background only and cannot close the current token/source-grounding issue.</small>
- proxy-blocked artifact、wrong-target shell artifact、plain Cargo build artifact、旧 binary artifact、Hope-2-v0、`E:\codex\hope-local-stale-archive`、旧 `%TEMP%` artifact、旧 CDP profile 与旧 no-`RepoRoot` run 都只能作为 `reference-only` 或 `stale` 提醒。
<small>Proxy-blocked artifacts, wrong-target shell artifacts, plain-Cargo build artifacts, old binary artifacts, Hope-2-v0, `E:\codex\hope-local-stale-archive`, old `%TEMP%` artifacts, old CDP profiles, and old no-`RepoRoot` runs are reference-only or stale reminders only.</small>

## 7. 项目进度错题详表
<small>7. Detailed Project Progress Error Entries</small>

### `HOPE-PROV-001`：provider timeout 双源漂移
<small>`HOPE-PROV-001`: Provider timeout dual-source drift</small>

- 编号：`HOPE-PROV-001`
<small>ID: `HOPE-PROV-001`</small>
- 问题陈述：同一次 provider timeout 会在产品默认逻辑与 QA-only override 之间被解释成不同失败类型，导致主线进度判断漂移。
<small>Problem statement: The same provider timeout could be interpreted differently between product-default logic and QA-only override, drifting mainline progress judgment.</small>
- 首次症状：同类 timeout 在不同汇报里分别被写成 transport 问题、provider 不稳或 validator 失败前置条件。
<small>First symptom: Similar timeout cases were reported as transport failure, provider instability, or validator-precondition failure in different reports.</small>
- 影响范围：会误导修复顺序，浪费单案复跑，并让 live3 前的 blocker 判断失真。
<small>Impact: It misorders remediation, wastes reruns, and distorts pre-live3 blocker judgment.</small>
- 根因类型：设计缺陷
<small>Root-cause type: design-defect</small>
- 为什么当时没挡住：QA timeout override 存在，但没有被写成与产品默认行为分离的强制解释边界。
<small>Why it escaped: The QA timeout override existed, but it was not codified as a mandatory interpretation boundary distinct from product-default behavior.</small>
- 解决过程：先把 timeout 直接归为 provider 不稳，这条路没有形成可执行判断；随后检查 release-QA handoff，才把 QA-only timeout override、hard-fail 与 transport timeout 的关系拆开，并据此重写汇报口径。
<small>Resolution: The first attempt flattened timeout into provider instability and failed to produce a usable decision rule; the later step re-read the release-QA handoff and separated QA-only timeout override, hard-fail, and transport timeout into distinct reporting paths.</small>
- 修复验证证据：`E:\codex\hope-desktop-shell\docs\desktop-release-qa-handoff.md` 已明确写出 QA hard-fail 模式下的 sanitized timeout override 与“timeout exhaustion 仍算失败”的规则。
<small>Verification evidence: `E:\codex\hope-desktop-shell\docs\desktop-release-qa-handoff.md` explicitly defines the sanitized timeout override in QA hard-fail mode and states that timeout exhaustion still fails the gate.</small>
- 如何防范：以后所有 provider 汇报都必须同时写明产品默认值、QA override、proxy 状态与最终 hard-fail stage，否则不能进入 gate 结论。
<small>Prevention: Every future provider report must state product default, QA override, proxy status, and final hard-fail stage together, or it cannot drive a gate conclusion.</small>
- Playbook · 第一检查项：先看当前运行是否使用 `-QaProviderHardFail`，以及 timeout 说明是否明确属于 QA-only override。
<small>Playbook · First Check: Check whether the current run used `-QaProviderHardFail` and whether the timeout note is explicitly scoped to QA-only override.</small>
- Playbook · 必要证据：launcher 启动方式、expected model、provider 状态摘要、timeout stage 标签。
<small>Playbook · Evidence Required: launcher mode, expected model, provider status summary, and timeout stage label.</small>
- Playbook · 停止条件：一旦确认该失败纯属 transport timeout，就停止把它继续扩展成 validator 或 source 问题。
<small>Playbook · Stop Condition: Stop once the failure is confirmed as pure transport timeout; do not expand it into validator or source issues.</small>
- Playbook · 升级条件：相同 fresh 环境与相同 provider/model 路径下，timeout 仍重复出现且无法被 stage 归类。
<small>Playbook · Escalation: Escalate if the timeout repeats under the same fresh environment and provider/model path but still resists stage classification.</small>
- Playbook · 禁止捷径：不要把所有 timeout 直接写成“provider unstable”。
<small>Playbook · Forbidden Shortcut: Do not flatten all timeout cases into “provider unstable.”</small>
- 禁止重复排查：在没有先区分 QA override 与产品默认行为前，不要重开这条错题。
<small>Do not reopen: Do not reopen this entry before separating QA override from product-default behavior.</small>
- 当前状态：closed
<small>Status: closed</small>

### `HOPE-PROV-002`：provider retry telemetry 与 validator hard-fail 混淆
<small>`HOPE-PROV-002`: Provider retry telemetry confused with validator hard-fail</small>

- 编号：`HOPE-PROV-002`
<small>ID: `HOPE-PROV-002`</small>
- 问题陈述：retry exhaustion、QA evidence missing、validator hard-gate fail 与 prompt boundary fail 曾被混成同一个“failed”叙事。
<small>Problem statement: Retry exhaustion, QA evidence missing, validator hard-gate fail, and prompt boundary fail were previously collapsed into one generic “failed” story.</small>
- 首次症状：不同 stage 的失败会使用相同的总结词，导致下一步修复动作随意切换。
<small>First symptom: Different failure stages were summarized with the same wording, causing arbitrary next-step repair choices.</small>
- 影响范围：会把 provider、validator 与 runner 的 ownership 混在一起，拖慢主线推进。
<small>Impact: It mixes provider, validator, and runner ownership and slows mainline progress.</small>
- 根因类型：沟通失真
<small>Root-cause type: communication-loss</small>
- 为什么当时没挡住：matrix 虽然已有 stage 名称，但汇报模板没有强制保留这些名称。
<small>Why it escaped: The matrix already had stage names, but report templates did not force those names to be preserved.</small>
- 解决过程：先尝试沿用“failed”总括写法，这条尝试无法提供后续行动指引；随后改为直接对照 matrix 的 hard-fail taxonomy，把 retry exhaustion、QA evidence missing、validator hard-gate fail 与 prompt boundary fail 分开写。
<small>Resolution: The first attempt kept the generic “failed” wording and produced no actionable follow-up; the later step mapped reports directly to the matrix hard-fail taxonomy and split the failure classes explicitly.</small>
- 修复验证证据：`E:\codex\hope-desktop-shell\docs\desktop-release-qa-handoff.md` 与 `tests/qa/desktop-model-certification-matrix.json` 都已把 runner hard-fail stages 分开列出。
<small>Verification evidence: `E:\codex\hope-desktop-shell\docs\desktop-release-qa-handoff.md` and `tests/qa/desktop-model-certification-matrix.json` both list runner hard-fail stages separately.</small>
- 如何防范：以后凡是 runner、certifier 或 handoff 汇报，只能用 stage 原名，不得再用统称 `failed` 覆盖。
<small>Prevention: Future runner, certifier, and handoff reports must use the original stage names and may not flatten them into a generic `failed` label.</small>
- Playbook · 第一检查项：先从 matrix 或 certifier 结果里读出 stage 原名，再讨论根因。
<small>Playbook · First Check: Read the original stage name from the matrix or certifier output before discussing root cause.</small>
- Playbook · 必要证据：hard-fail stage、expected model、fallback 状态、validator 是否运行。
<small>Playbook · Evidence Required: hard-fail stage, expected model, fallback status, and whether validator ran.</small>
- Playbook · 停止条件：一旦 stage 唯一确定，就停止跨层猜测。
<small>Playbook · Stop Condition: Stop cross-layer guessing once the stage is uniquely identified.</small>
- Playbook · 升级条件：transport 与 validator 证据仍然同时扭在一起，artifact 又不自描述。
<small>Playbook · Escalation: Escalate if transport and validator evidence still appear entangled and the artifact remains non-self-describing.</small>
- Playbook · 禁止捷径：不要为了汇报省事把 validator 失败改写成 provider 不稳。
<small>Playbook · Forbidden Shortcut: Do not rewrite validator failure as provider instability for reporting convenience.</small>
- 禁止重复排查：在没有 stage 原名的情况下，不要重开根因讨论。
<small>Do not reopen: Do not reopen root-cause discussion without the original stage name.</small>
- 当前状态：closed
<small>Status: closed</small>

### `HOPE-EVID-003`：raw compact evidence 字段不自含
<small>`HOPE-EVID-003`: Raw compact evidence was not self-contained</small>

- 编号：`HOPE-EVID-003`
<small>ID: `HOPE-EVID-003`</small>
- 问题陈述：compact evidence 过去无法独立解释 case、model、stage、freshness 与失败含义，需要额外聊天上下文补完。
<small>Problem statement: Compact evidence previously could not explain case, model, stage, freshness, and failure meaning on its own and required extra chat context.</small>
- 首次症状：评审者单看 compact artifact 时，无法判断它到底证明了什么，也无法区分 raw 事实与 certifier 派生解释。
<small>First symptom: Reviewers reading only the compact artifact could not tell what it proved or whether they were seeing raw facts versus certifier-derived interpretation.</small>
- 影响范围：会让关闭结论不稳，并造成 evidence semantics 只能靠口头补充。
<small>Impact: It makes closure claims unstable and forces evidence semantics to depend on tribal explanation.</small>
- 根因类型：错误证据
<small>Root-cause type: wrong-evidence</small>
- 为什么当时没挡住：compact 字段设计优先追求轻量输出，没有把“raw artifact 必须自含”写成先决条件。
<small>Why it escaped: Compact-field design prioritized lightweight output and did not codify “raw artifact must be self-contained” as a prerequisite.</small>
- 解决过程：先尝试用 certifier 派生字段解释 compact artifact，这条路径无法让 outsider 独立核对；随后补齐 raw `provider_retry_evidence` 字段，并用 fresh raw telemetry 重新验证 compact 含义，才完成收口。
<small>Resolution: The first attempt relied on certifier-derived fields to explain compact evidence, which outsiders could not independently verify; the later step backfilled raw `provider_retry_evidence` fields and re-verified the meaning with fresh raw telemetry.</small>
- 修复验证证据：当前关闭依据是 raw `provider_retry_evidence` 回填与 fresh raw telemetry 复证；不再把 certifier 派生解释单独当成 closure evidence。
<small>Verification evidence: Closure is now based on raw `provider_retry_evidence` backfill plus fresh raw telemetry re-verification, not certifier-derived interpretation alone.</small>
- 如何防范：以后新增 compact 字段前，必须先证明 raw artifact 已经自含，否则该字段不得进入 gate summary。
<small>Prevention: Any future compact field must first prove that the raw artifact is already self-contained; otherwise it cannot enter a gate summary.</small>
- Playbook · 第一检查项：先看 raw artifact 是否能独立回答 case、model、stage 与 freshness，再看 compact summary。
<small>Playbook · First Check: Confirm the raw artifact can independently answer case, model, stage, and freshness before reading the compact summary.</small>
- Playbook · 必要证据：raw artifact、fresh raw telemetry、compact artifact、字段映射说明。
<small>Playbook · Evidence Required: raw artifact, fresh raw telemetry, compact artifact, and field traceability notes.</small>
- Playbook · 停止条件：只要 raw artifact 不能独立成立，就停止用 compact 字段关单。
<small>Playbook · Stop Condition: Stop using compact fields for closure as soon as the raw artifact fails to stand on its own.</small>
- Playbook · 升级条件：有人提出新的 compact schema，但 raw artifact 仍未补齐语义。
<small>Playbook · Escalation: Escalate if a new compact schema is proposed while the raw artifact is still semantically incomplete.</small>
- Playbook · 禁止捷径：不要再用 certifier 派生说明替代 raw 证据。
<small>Playbook · Forbidden Shortcut: Do not substitute certifier-derived explanation for raw evidence.</small>
- 禁止重复排查：在 raw artifact 仍不自含时，不要重开 compact 关单讨论。
<small>Do not reopen: Do not reopen compact-closure discussion while the raw artifact remains non-self-contained.</small>
- 当前状态：closed
<small>Status: closed</small>

### `HOPE-SHELL-004`：WebView2 弹窗、child 未生成、CDP target 为零、devUrl target mismatch
<small>`HOPE-SHELL-004`: WebView2 popup, missing child process, zero CDP target, and devUrl target mismatch</small>

- 编号：`HOPE-SHELL-004`
<small>ID: `HOPE-SHELL-004`</small>
- 问题陈述：shell 启动异常、WebView2 环境问题、child process 缺失与 wrong target URL 曾被混成同一类 release-QA 失败。
<small>Problem statement: Shell startup failures, WebView2 environment issues, missing child process, and wrong target URL were previously treated as one generic release-QA failure class.</small>
- 首次症状：`hope-app` 启动后出现 `cdp_ready=false`、`target_count=0`，或者 target URL 落到 `127.0.0.1:5173` 而不是 `tauri.localhost`。
<small>First symptom: `hope-app` launched with `cdp_ready=false`, `target_count=0`, or a target URL at `127.0.0.1:5173` instead of `tauri.localhost`.</small>
- 影响范围：会阻断 UI-driven QA，并把 shell/runtime 问题误包装成 provider 或 validator blocker。
<small>Impact: It blocks UI-driven QA and can mispackage shell/runtime issues as provider or validator blockers.</small>
- 根因类型：流程边界失败
<small>Root-cause type: process-boundary failure</small>
- 为什么当时没挡住：shell 诊断、provider 运行与 release provenance 没有被拆成独立检查点。
<small>Why it escaped: Shell diagnostics, provider runs, and release provenance were not separated into independent checkpoints.</small>
- 解决过程：先试图把 zero-target 与 wrong-target 一起归类成“壳层不稳定”，这条路无法指导下一步；随后把 zero-target 转到 `LaunchDiagnosticOnly`，把 wrong-target 归入 release provenance blocker，并要求每个 shell 生命周期以 `StopOnly` 结束。第 5 次复发证明“仅有规则文字”不足以关单，因此本次新增 shell startup hard gate contract，把 `AppDefault + LaunchDiagnosticOnly`、`before_main_window_build`、`0x80000003` popup、`tauri.localhost` target 与 cleanup 全部升级成强制 gate。
<small>Resolution: The first attempt grouped zero-target and wrong-target into a vague shell-instability bucket and did not guide the next action; the later step routed zero-target to `LaunchDiagnosticOnly`, wrong-target to release-provenance blocking, and enforced `StopOnly` at the end of each shell lifecycle. The fifth recurrence proved that text-only rules were insufficient for closure, so this cycle adds a shell-startup hard-gate contract covering `AppDefault + LaunchDiagnosticOnly`, `before_main_window_build`, `0x80000003` popup handling, the `tauri.localhost` target, and cleanup.</small>
- 修复验证证据：`E:\codex\hope-desktop-shell\docs\desktop-release-qa-handoff.md` 与新增 `E:\codex\hope-desktop-shell\docs\desktop-shell-webview2-cdp-startup-contract.md` 已写明正式 shell gate、wrong-target blocker、host-window blocker、explicit `-RepoRoot` clean workspace 边界与 `StopOnly` cleanup；Shell Gate 已在 clean workspace `E:\codex\hope-desktop-shell-clean-9a8f4e0-r2`、anchor `9a8f4e0349fa56a7cc57fb2e3817fabaa75afb6e`、release exe hash `E3BB9738851AF64FBE642517CA8DE2BEE82B23F67B4B19E9C8C817E8C6AD6D26`、valid runs `hope-cdp-20260505-193016-26016` 与 `hope-cdp-20260505-193110-27216` 上窄口径 stabilized。该证据只关闭 shell startup / CDP target 启动层，不解锁 provider/live3/full16/403-case/package。
<small>Verification evidence: `E:\codex\hope-desktop-shell\docs\desktop-release-qa-handoff.md` and the new `E:\codex\hope-desktop-shell\docs\desktop-shell-webview2-cdp-startup-contract.md` now define the formal shell gate, wrong-target blocker, host-window blocker, explicit `-RepoRoot` clean-workspace boundary, and `StopOnly` cleanup; the Shell Gate is narrowly stabilized in clean workspace `E:\codex\hope-desktop-shell-clean-9a8f4e0-r2` at anchor `9a8f4e0349fa56a7cc57fb2e3817fabaa75afb6e`, release exe hash `E3BB9738851AF64FBE642517CA8DE2BEE82B23F67B4B19E9C8C817E8C6AD6D26`, and valid runs `hope-cdp-20260505-193016-26016` plus `hope-cdp-20260505-193110-27216`. This evidence closes only the shell startup / CDP target startup layer and does not unlock provider/live3/full16/403-case/package.</small>
- 如何防范：以后每次 CDP 汇报都必须同时带 workspace path、HEAD、exe hash、`/json/list` artifact、post-setup probe、target URL/title、PID 与 cleanup 结果；shell gate 不绿时一律禁止 provider / runner / live3 / pool / `full16` / `403-case` / package。
<small>Prevention: Every future CDP report must include workspace path, HEAD, exe hash, `/json/list` artifact, post-setup probe, target URL/title, PID, and cleanup result together; when the shell gate is not green, provider, runner, live3, pool, `full16`, `403-case`, and package work are all forbidden.</small>
- Playbook · 第一检查项：先判断是 zero-target、wrong-target，还是 child-process absence，再决定是否进入 provider 讨论。
<small>Playbook · First Check: Decide whether the issue is zero-target, wrong-target, or missing child process before touching provider discussion.</small>
- Playbook · 必要证据：explicit `-RepoRoot`、workspace path、HEAD、exe hash、`/json/list` artifact、post-setup probe、PID、diagnostic log 摘要、StopOnly 结果。
<small>Playbook · Evidence Required: explicit `-RepoRoot`, workspace path, HEAD, exe hash, `/json/list` artifact, post-setup probe, PID, diagnostic log summary, and StopOnly result.</small>
- Playbook · 停止条件：一旦确认问题只在 shell/runtime 层出现，就停止往业务 QA 扩散。
<small>Playbook · Stop Condition: Stop expanding into business QA as soon as the issue is shown to be shell/runtime-only.</small>
- Playbook · 升级条件：fresh release-like build 仍无法落到 `http://tauri.localhost/#/workbench`。
<small>Playbook · Escalation: Escalate if a fresh release-like build still fails to resolve to `http://tauri.localhost/#/workbench`.</small>
- Playbook · 禁止捷径：不要把 `127.0.0.1:5173`、no-`RepoRoot` run、Hope-2-v0 artifact、旧 CDP profile、旧 `%TEMP%` artifact 或弹窗观察当成 release-shell evidence。
<small>Playbook · Forbidden Shortcut: Do not treat `127.0.0.1:5173`, a no-`RepoRoot` run, a Hope-2-v0 artifact, an old CDP profile, an old `%TEMP%` artifact, or popup-only observation as release-shell evidence.</small>
- 禁止重复排查：在没有先分清 zero-target 与 wrong-target 前，不要重开 provider 根因。
<small>Do not reopen: Do not reopen provider root-cause discussion before separating zero-target from wrong-target.</small>
- 当前状态：shell-gate-stabilized
<small>Status: shell-gate-stabilized</small>

### `HOPE-EVID-005`：stale artifact / stale binary / stale profile / stale env 污染 gate
<small>`HOPE-EVID-005`: Stale artifact, stale binary, stale profile, and stale env polluted gate judgment</small>

- 编号：`HOPE-EVID-005`
<small>ID: `HOPE-EVID-005`</small>
- 问题陈述：相关代码、运行态或打包条件变化后，旧 artifact、旧 binary、旧 profile 与旧 env 仍被拿来解释当前 gate。
<small>Problem statement: Historical artifacts, binaries, profiles, and env state kept being used to explain the current gate after relevant code, runtime, or packaging changes.</small>
- 首次症状：报告里保留了旧产物，但没有明确标注 `stale` 或 `reference-only`。
<small>First symptom: Old outputs remained in reports without explicit `stale` or `reference-only` labeling.</small>
- 影响范围：会制造假通过、假失败与错误 blocker，并拖慢排查。
<small>Impact: It creates false passes, false failures, and wrong blockers while slowing diagnosis.</small>
- 根因类型：错误证据
<small>Root-cause type: wrong-evidence</small>
- 为什么当时没挡住：历史产物在诊断阶段确实有参考价值，但没有被重新降级标注。
<small>Why it escaped: Historical outputs were useful for diagnosis, but they were not reclassified downward after relevant changes.</small>
- 解决过程：先沿用旧 artifact 做对照，结果把新旧状态混在一起；随后把所有历史产物统一降级成 `stale` 或 `reference-only`，只允许 fresh rerun 结果驱动当前 gate。
<small>Resolution: The first attempt kept historical artifacts in active comparison and mixed old and new states; the later step downgraded all historical outputs to `stale` or `reference-only` and allowed only fresh rerun evidence to drive the gate.</small>
- 修复验证证据：`E:\codex\AGENTS.md` 与 `E:\codex\hope-desktop-shell\docs\codex-usage-core-rules.md` 都已明确 stale evidence 不能驱动当前 gate。
<small>Verification evidence: `E:\codex\AGENTS.md` and `E:\codex\hope-desktop-shell\docs\codex-usage-core-rules.md` both explicitly forbid stale evidence from driving the current gate.</small>
- 如何防范：所有汇报都必须给 artifact 标 fresh、stale/reference-only 或 blocked 标签，否则不允许关单。
<small>Prevention: Every report must label each artifact as fresh, stale/reference-only, or blocked, or it cannot close an issue.</small>
- Playbook · 第一检查项：先比对 artifact 时间、生成命令与当前代码/运行态/打包条件是否仍匹配。
<small>Playbook · First Check: Compare artifact timestamp and generation conditions against current code, runtime, and packaging state.</small>
- Playbook · 必要证据：生成命令、时间戳、相关变更说明、freshness 标签。
<small>Playbook · Evidence Required: generation command, timestamp, related-change note, and freshness label.</small>
- Playbook · 停止条件：只要 artifact 早于相关变更，就停止拿它做当前 gate 判断。
<small>Playbook · Stop Condition: Stop using an artifact for current gate judgment as soon as it is shown to predate a relevant change.</small>
- Playbook · 升级条件：没有 fresh rerun 条件，却仍被要求对当前 gate 下结论。
<small>Playbook · Escalation: Escalate if a current gate conclusion is demanded without any possibility of a fresh rerun.</small>
- Playbook · 禁止捷径：不要因为旧 artifact “看起来差不多”就继续沿用。
<small>Playbook · Forbidden Shortcut: Do not reuse an old artifact just because it “looks close enough.”</small>
- 禁止重复排查：在 freshness 没重新分类前，不要重开当前 gate 结论。
<small>Do not reopen: Do not reopen a current gate conclusion before freshness has been reclassified.</small>
- 当前状态：closed
<small>Status: closed</small>

### `HOPE-BUILD-006`：direct cargo build 与 Tauri release-like build provenance 混淆
<small>`HOPE-BUILD-006`: Direct Cargo build was confused with Tauri release-like build provenance</small>

- 编号：`HOPE-BUILD-006`
<small>ID: `HOPE-BUILD-006`</small>
- 问题陈述：普通 `cargo build -p hope-app --release` 生成的二进制曾被误写成接近 Tauri release-like build 的 gate evidence。
<small>Problem statement: A plain `cargo build -p hope-app --release` output was once over-reported as near-equivalent gate evidence for a Tauri release-like build.</small>
- 首次症状：只因为二进制存在且能启动，就被过读成“接近 release-ready”。
<small>First symptom: Binary existence and launchability were over-read as “near release-ready.”</small>
- 影响范围：会制造错误 provenance 结论，并掩盖打包 UI 资产或 target URL 问题。
<small>Impact: It creates false provenance conclusions and hides packaged-UI or target-URL problems.</small>
- 根因类型：工具误用
<small>Root-cause type: tool-misuse</small>
- 为什么当时没挡住：plain Cargo build 与 formal Tauri build path 的差异没有被当作强制 gate。
<small>Why it escaped: The difference between plain Cargo build and formal Tauri build path was not treated as a mandatory gate.</small>
- 解决过程：先把“能启动的 exe”当成足够证据，这条路径被 release-QA 规则否决；随后改为只接受带 `app/tauri.conf.json`、`beforeBuildCommand`、`ui/dist` 与 `tauri.localhost` 证据的 release-like path。
<small>Resolution: The first attempt treated a launchable exe as sufficient evidence, which was rejected by the release-QA rules; the later step accepted only the release-like path that proves `app/tauri.conf.json`, `beforeBuildCommand`, `ui/dist`, and `tauri.localhost`.</small>
- 修复验证证据：`E:\codex\hope-desktop-shell\docs\desktop-release-qa-handoff.md` 已明确排除 plain Cargo build 作为 accepted release provenance。
<small>Verification evidence: `E:\codex\hope-desktop-shell\docs\desktop-release-qa-handoff.md` explicitly excludes plain Cargo build from accepted release provenance.</small>
- 如何防范：以后任何 release 结论都必须同时说明 build path 与运行时 target URL。
<small>Prevention: Any future release conclusion must state both the build path and the runtime target URL.</small>
- Playbook · 第一检查项：先问当前 exe 来自哪条 build path，再看 target URL 是否落到 `tauri.localhost`。
<small>Playbook · First Check: Ask which build path produced the current exe, then check whether the target URL resolves to `tauri.localhost`.</small>
- Playbook · 必要证据：build command family、`ui/dist` 存在性、`custom_protocol` 相关标记、target URL。
<small>Playbook · Evidence Required: build-command family, `ui/dist` presence, `custom_protocol`-related markers, and target URL.</small>
- Playbook · 停止条件：一旦确认它只是 plain Cargo 输出，就停止把它用于 release-like 讨论。
<small>Playbook · Stop Condition: Stop using the output in release-like discussion as soon as it is identified as plain Cargo build output.</small>
- Playbook · 升级条件：自称 release-like build，却仍落到 `127.0.0.1:5173` 或缺失打包 UI 资产。
<small>Playbook · Escalation: Escalate if a claimed release-like build still lands on `127.0.0.1:5173` or lacks packaged UI assets.</small>
- Playbook · 禁止捷径：不要只凭“它能跑起来”就关闭 provenance 问题。
<small>Playbook · Forbidden Shortcut: Do not close a provenance issue just because the binary launches.</small>
- 禁止重复排查：在 build path 未明确前，不要重开 package readiness 讨论。
<small>Do not reopen: Do not reopen package-readiness discussion before the build path is explicit.</small>
- 当前状态：closed
<small>Status: closed</small>

### `HOPE-CONTRACT-007`：`主角初 -> 初现 -> 主角` 暴露出的 token/entity classification 与 source role grounding 问题
<small>`HOPE-CONTRACT-007`: Token/entity classification and source-role grounding issue exposed by `主角初 -> 初现 -> 主角`</small>

- 编号：`HOPE-CONTRACT-007`
<small>ID: `HOPE-CONTRACT-007`</small>
- 问题陈述：`主角初 -> 初现 -> 主角` 这类局部名称变化会在 token/entity classification、StoryFactFrame binding 与 source role grounding 之间漂移，导致 validator 无法稳定判责。
<small>Problem statement: Partial-name transitions such as `主角初 -> 初现 -> 主角` can drift across token/entity classification, StoryFactFrame binding, and source-role grounding, preventing stable validator attribution.</small>
- 首次症状：row-level validation 之前，就已经无法稳定判断分歧起点属于 extraction、binding 还是 source role grounding。
<small>First symptom: Before row-level validation, the disagreement could no longer be stably attributed to extraction, binding, or source-role grounding.</small>
- 影响范围：会阻断当前主线继续推进到 fresh live3，并让 source-fact binding 与角色归属同时失真。
<small>Impact: It blocks current mainline progress toward fresh live3 and distorts both source-fact binding and role attribution.</small>
- 根因类型：契约缺失
<small>Root-cause type: missing-contract</small>
- 为什么当时没挡住：结构化字段契约比 token/entity 与 source role grounding 契约更完整，导致前层语义漏洞被后层结构暂时掩盖。
<small>Why it escaped: Structural field contracts were stronger than token/entity and source-role grounding contracts, so front-layer semantic gaps were temporarily masked by downstream structure.</small>
- 解决过程：第一步曾把问题收缩成纯 token classification，这只能解释 `主角初 -> 初现` 的局部名称问题，不能关闭 source role grounding；第二步在 `app/src/runtime.rs` 对 `初现 / 初现于` 做了本地实现修复，并通过 targeted tests；第三步需要 release-like rebuild 与 post-fix fresh provider artifact，当前这一步仍未完成。
<small>Resolution: The first attempt reduced the issue to pure token classification, which could explain the `主角初 -> 初现` substring case but not close source-role grounding; the second step added a local implementation fix for `初现 / 初现于` in `app/src/runtime.rs` and passed targeted tests; the third required step is a release-like rebuild plus post-fix fresh provider artifact, and that step is still pending.</small>
- 修复验证证据：当前只有本地实现修复与 targeted tests 通过；新增 `E:\codex\hope-desktop-shell\docs\desktop-validator-source-grounding-contract.md` 已把 token/entity classification、source-bound fragment、bare source role `主角`、invented-name hard-fail 与必要 sanitized evidence 固定下来；但缺少 post-fix fresh provider artifact，因此不能写成已修复。
<small>Verification evidence: Only the local implementation fix and targeted tests are currently available; the new `E:\codex\hope-desktop-shell\docs\desktop-validator-source-grounding-contract.md` now fixes token/entity classification, source-bound fragments, the bare source role `主角`, invented-name hard fail, and required sanitized evidence; but the post-fix fresh provider artifact is still missing, so the item cannot be described as fixed.</small>
- 如何防范：凡是修改 `app/src/runtime.rs` 中的抽取、分类或 source role grounding 逻辑，fresh QA 前都必须先重建 release-like shell，并以 fresh provider artifact 复证；不得再用“模型风格解释”替代 source grounding 契约判断。
<small>Prevention: Any change to extraction, classification, or source-role grounding logic in `app/src/runtime.rs` must force a release-like rebuild before fresh QA and must be re-verified by a fresh provider artifact; a “model style” explanation may no longer replace the source-grounding contract judgment.</small>
- Playbook · 第一检查项：先判断分歧起点是在 extraction、StoryFactFrame binding、source role grounding 还是 row validation，再确认手里的 artifact 是否早于本地修复。
<small>Playbook · First Check: Identify whether the disagreement starts in extraction, StoryFactFrame binding, source-role grounding, or row validation, then confirm whether the available artifact predates the local fix.</small>
- Playbook · 必要证据：targeted test 结果、snapshot hash、StoryFactFrame hash、validator stage、post-fix fresh provider artifact。
<small>Playbook · Evidence Required: targeted-test result, snapshot hash, StoryFactFrame hash, validator stage, and post-fix fresh provider artifact.</small>
- Playbook · 停止条件：一旦确认 artifact 早于修复版本，或确认问题仍停留在 source role grounding 契约缺失，就停止拿旧 artifact 继续下当前 gate 结论。
<small>Playbook · Stop Condition: Stop driving the current gate with old evidence once the artifact is shown to predate the fix or the issue is confirmed as a source-role-grounding contract gap.</small>
- Playbook · 升级条件：release-like rebuild 后的多个 fresh artifacts 仍复现同类归因错误。
<small>Playbook · Escalation: Escalate if multiple fresh artifacts still reproduce the same attribution error after a release-like rebuild.</small>
- Playbook · 禁止捷径：修改 `app/src/runtime.rs` 后，不得复用旧 `target\release\hope-app.exe`、旧 provider artifact 或 pre-fix validator review。
<small>Playbook · Forbidden Shortcut: After modifying `app/src/runtime.rs`, do not reuse old `target\release\hope-app.exe`, old provider artifacts, or pre-fix validator reviews.</small>
- 禁止重复排查：在没有 post-fix fresh provider artifact 前，不要把该项重新包装成“只是模型风格波动”。
<small>Do not reopen: Do not reframe this issue as mere model-style variance before a post-fix fresh provider artifact exists.</small>
- 当前状态：open
<small>Status: open</small>

### `HOPE-CONTRACT-008`：prompt_text boundary / accepted snapshot / StoryFactFrame 下游证据缺口
<small>`HOPE-CONTRACT-008`: Prompt-text boundary, accepted snapshot, and StoryFactFrame downstream evidence gap</small>

- 编号：`HOPE-CONTRACT-008`
<small>ID: `HOPE-CONTRACT-008`</small>
- 问题陈述：结构上看似正常的结果，仍可能缺失 accepted snapshot、StoryFactFrame 与 prompt_text boundary 的下游绑定证明。
<small>Problem statement: A structurally normalized result can still lack downstream proof for accepted snapshot, StoryFactFrame, and prompt-text boundary binding.</small>
- 首次症状：汇报引用 `schema ok` 或 rows normalization，但没有同时给出 binding truth。
<small>First symptom: Reports cited `schema ok` or row normalization without presenting binding truth alongside them.</small>
- 影响范围：会制造假通过，并让 stale snapshot 与 row drift 更难发现。
<small>Impact: It creates false passes and makes stale snapshot reuse and row drift harder to detect.</small>
- 根因类型：错误证据
<small>Root-cause type: wrong-evidence</small>
- 为什么当时没挡住：结构类检查更容易被引用，绑定类证据没有被提升到同一 gate 权重。
<small>Why it escaped: Structural checks were easier to cite, while binding evidence was not elevated to the same gate weight.</small>
- 解决过程：先依赖 `schema ok` 与 normalization，总结成本较低但无法证明 binding truth；随后把 accepted snapshot hash、StoryFactFrame hash、source text hash、prompt boundary、rows_match 与 row_diffs 升级成必带字段。
<small>Resolution: The first attempt relied on `schema ok` and normalization because they were easy to summarize but could not prove binding truth; the later step promoted accepted snapshot hash, StoryFactFrame hash, source text hash, prompt boundary, rows_match, and row_diffs into mandatory fields.</small>
- 修复验证证据：`E:\codex\hope-desktop-shell\docs\desktop-release-qa-handoff.md` 与 `tests/qa/desktop-model-certification-matrix.json` 都把这些字段列为 live gate contract。
<small>Verification evidence: `E:\codex\hope-desktop-shell\docs\desktop-release-qa-handoff.md` and `tests/qa/desktop-model-certification-matrix.json` both treat these fields as live gate contract requirements.</small>
- 如何防范：任何 future gate 汇报都必须并列写结构状态与绑定状态。
<small>Prevention: Any future gate report must state structural status and binding status together.</small>
- Playbook · 第一检查项：先看 artifact 是否同时携带 snapshot hash、StoryFactFrame hash、prompt_text boundary 与 rows_match。
<small>Playbook · First Check: Confirm whether the artifact carries snapshot hash, StoryFactFrame hash, prompt-text boundary, and rows_match together.</small>
- Playbook · 必要证据：hash、boundary、rows_match、row_diffs。
<small>Playbook · Evidence Required: hash, boundary, rows_match, and row_diffs.</small>
- Playbook · 停止条件：任一绑定字段缺失、过期或不可追溯时立即停止。
<small>Playbook · Stop Condition: Stop immediately when any binding field is missing, stale, or untraceable.</small>
- Playbook · 升级条件：artifact 结构有效，却仍无法证明 binding truth。
<small>Playbook · Escalation: Escalate if the artifact looks structurally valid but still cannot prove binding truth.</small>
- Playbook · 禁止捷径：不要把 `schema ok` 当成 case pass。
<small>Playbook · Forbidden Shortcut: Do not treat `schema ok` as case pass.</small>
- 禁止重复排查：在绑定证据未齐前，不要重开质量层讨论。
<small>Do not reopen: Do not reopen quality-level discussion while binding evidence is incomplete.</small>
- 当前状态：closed
<small>Status: closed</small>

### `HOPE-PROV-009`：proxy env contamination / 代理环境污染
<small>`HOPE-PROV-009`: Proxy env contamination</small>

- 编号：`HOPE-PROV-009`
<small>ID: `HOPE-PROV-009`</small>
- 问题陈述：provider QA 首轮会因继承 `HTTP_PROXY`、`HTTPS_PROXY`、`ALL_PROXY` 而先触发 `runner_proxy_env`，污染对真实 provider 行为的判断。
<small>Problem statement: The first provider-QA attempt can inherit `HTTP_PROXY`, `HTTPS_PROXY`, and `ALL_PROXY`, trigger `runner_proxy_env`, and contaminate judgment about real provider behavior.</small>
- 首次症状：第一份 QA artifact 看起来像 provider blocked，但实质是代理环境污染。
<small>First symptom: The first QA artifact looked provider-blocked, but the real issue was inherited proxy state.</small>
- 影响范围：会浪费单案重跑次数，并把本该 reference-only 的 artifact 误写成 gate evidence。
<small>Impact: It wastes single-case reruns and can mislabel a reference-only artifact as gate evidence.</small>
- 根因类型：环境漂移
<small>Root-cause type: environment-drift</small>
- 为什么当时没挡住：proxy env 没被纳入 provider QA preflight，而是被默认当成“系统背景”。
<small>Why it escaped: Proxy env was not treated as provider-QA preflight evidence and was instead assumed to be harmless background state.</small>
- 解决过程：先直接读首轮 artifact 试图推断 provider 行为，这条路没有排除环境污染；随后清理代理变量、设置 `NO_PROXY=localhost,127.0.0.1,::1`，并把 proxy before/after-clear 与 clean rerun 一起记录。
<small>Resolution: The first attempt inferred provider behavior directly from the first-run artifact and did not exclude environment contamination; the later step cleared proxy variables, set `NO_PROXY=localhost,127.0.0.1,::1`, and paired proxy before/after-clear with the clean rerun.</small>
- 修复验证证据：release-QA handoff 已要求用 sanitized proxy before/after 与 clean rerun 作为 closure evidence；proxy-blocked artifact 只能 reference-only。
<small>Verification evidence: The release-QA handoff requires sanitized proxy before/after state plus a clean rerun as closure evidence; proxy-blocked artifacts are reference-only.</small>
- 如何防范：以后 provider QA 前固定记录 proxy env before/after clear。
<small>Prevention: Always record proxy env before and after clearing it before provider QA.</small>
- Playbook · 第一检查项：先看 launcher 或 artifact 摘要是否已出现 `runner_proxy_env` 或 proxy 继承痕迹。
<small>Playbook · First Check: Inspect the launcher or artifact summary for `runner_proxy_env` or inherited proxy signals.</small>
- Playbook · 必要证据：sanitized proxy before/after-clear 记录、`NO_PROXY` 设置、clean rerun artifact。
<small>Playbook · Evidence Required: sanitized proxy before/after-clear record, `NO_PROXY` setting, and the clean-rerun artifact.</small>
- Playbook · 停止条件：手里只有 proxy-blocked artifact 时立即停在 `reference-only`。
<small>Playbook · Stop Condition: Stop at `reference-only` as soon as the only available artifact is proxy-blocked.</small>
- Playbook · 升级条件：显式清理代理变量后，proxy 继承痕迹仍重复出现。
<small>Playbook · Escalation: Escalate if proxy signals keep reappearing after explicit clearing.</small>
- Playbook · 禁止捷径：不要用 proxy-blocked artifact 关闭 provider gate。
<small>Playbook · Forbidden Shortcut: Do not close a provider gate with a proxy-blocked artifact.</small>
- 禁止重复排查：在 clean rerun 前，不要把首轮 proxy 污染结果当成最终 provider 结论。
<small>Do not reopen: Do not treat a proxy-polluted first run as the final provider conclusion before a clean rerun.</small>
- 当前状态：closed
<small>Status: closed</small>

### `HOPE-BUILD-010`：`app/Cargo.toml` 被 Tauri CLI 自动改写 `features=[]` 的 preflight 风险
<small>`HOPE-BUILD-010`: Preflight risk that Tauri CLI may auto-rewrite `features=[]` in `app/Cargo.toml`</small>

- 编号：`HOPE-BUILD-010`
<small>ID: `HOPE-BUILD-010`</small>
- 问题陈述：Tauri CLI 在构建路径中可能自动改写 `app/Cargo.toml` 的 `features=[]`，从而让 source-tracked file 在未明确授权时发生 preflight 变化。
<small>Problem statement: Tauri CLI can auto-rewrite `features=[]` in `app/Cargo.toml` during the build path, causing source-tracked-file changes without explicit authorization.</small>
- 首次症状：相关线程需要做 QA 或 build 判断时，会先遇到 tracked-file state 是否仍可解释的问题，而不是直接进入业务结论。
<small>First symptom: Threads trying to make QA or build judgments first encounter uncertainty about tracked-file state instead of proceeding directly to business conclusions.</small>
- 影响范围：会污染 dirty ownership、增加 preflight 风险，并让 QA/build 线程更容易越界。
<small>Impact: It pollutes dirty ownership, increases preflight risk, and makes QA or build threads more likely to cross scope boundaries.</small>
- 根因类型：第三方变更
<small>Root-cause type: third-party-change</small>
- 为什么当时没挡住：当前允许读取的规则与文档已经能指出风险，但还没有把 `app/Cargo.toml` preflight guard 固定成每次 QA/build 前的强制检查点。
<small>Why it escaped: The available rules and docs can describe the risk, but they have not yet codified an `app/Cargo.toml` preflight guard as a mandatory checkpoint before each QA or build step.</small>
- 解决过程：先默认依赖事后 `git diff` 发现异常，这种做法只能在变更发生后补救，不能阻止 preflight 污染；当前更稳的做法是把该风险单列为 open blocker，等待明确的 preflight guard 与 fresh ownership evidence。
<small>Resolution: The first approach relied on after-the-fact `git diff`, which can only detect pollution after it occurs and cannot prevent it; the safer current stance is to keep the issue open until an explicit preflight guard and fresh ownership evidence exist.</small>
- 修复验证证据：当前 live `git status` 已显示 `app/Cargo.toml` 处于 dirty state；新增 `E:\codex\hope-desktop-shell\docs\desktop-qa-preflight-ownership-contract.md` 已把 preflight/postflight diff、`features=[]` whitelist handling 与超白名单 blocker 固定下来；但本契约尚缺一次 fresh 执行闭环，因此本项不能关闭。
<small>Verification evidence: Current live `git status` already shows `app/Cargo.toml` as dirty; the new `E:\codex\hope-desktop-shell\docs\desktop-qa-preflight-ownership-contract.md` now codifies preflight/postflight diff recording, `features=[]` whitelist handling, and over-whitelist blocking; but one fresh execution loop is still missing, so this item cannot close.</small>
- 如何防范：任何会触发 Tauri CLI 的 QA/build 线程，在执行前后都必须固定记录 `app/Cargo.toml` tracked state，并把异常视为 boundary blocker；即使是 `features=[]` 白名单 rewrite，也只能记录为 `whitelist-observed`，不能自动当成已关闭。
<small>Prevention: Any QA or build thread that can invoke Tauri CLI must record `app/Cargo.toml` tracked state before and after execution and treat anomalies as boundary blockers; even the `features=[]` whitelist rewrite may only be recorded as `whitelist-observed`, never automatically as closure.</small>
- Playbook · 第一检查项：在准备进入 build 或 release-QA 前，先检查 `app/Cargo.toml` 是否已经处于可解释状态。
<small>Playbook · First Check: Before entering build or release QA, check whether `app/Cargo.toml` is already in an explainable tracked state.</small>
- Playbook · 必要证据：启动前 tracked-file 状态、结束后 tracked-file 状态、授权边界说明。
<small>Playbook · Evidence Required: tracked-file state before start, tracked-file state after finish, and the authorization-boundary note.</small>
- Playbook · 停止条件：一旦出现未授权的 `app/Cargo.toml` 变化，就停止当前 QA/build 结论。
<small>Playbook · Stop Condition: Stop the current QA or build conclusion as soon as an unauthorized `app/Cargo.toml` change appears.</small>
- Playbook · 升级条件：需要继续推进 QA/build，但仍没有可复用的 preflight guard。
<small>Playbook · Escalation: Escalate if QA or build must proceed while no reusable preflight guard exists.</small>
- Playbook · 禁止捷径：不要把“当前这次没脏”写成“这个风险已经关闭”。
<small>Playbook · Forbidden Shortcut: Do not rewrite “not dirty this time” as “the risk is closed.”</small>
- 禁止重复排查：在 preflight guard 没落地前，不要把本项当作已解决。
<small>Do not reopen: Do not treat this item as solved before the preflight guard is codified.</small>
- 当前状态：contract-pending
<small>Status: contract-pending</small>

### `HOPE-OWN-011`：QA 线程越界修改 source/tracked 文件的 dirty ownership 风险
<small>`HOPE-OWN-011`: Dirty-ownership risk when QA threads cross into source/tracked files</small>

- 编号：`HOPE-OWN-011`
<small>ID: `HOPE-OWN-011`</small>
- 问题陈述：当 QA 线程在没有明确授权边界时触碰 source/tracked 文件，后续很难证明哪些结论来自 QA，哪些来自实现变更。
<small>Problem statement: When a QA thread touches source/tracked files without an explicit authorization boundary, it becomes difficult to prove which conclusions belong to QA and which belong to implementation changes.</small>
- 首次症状：当前 live `git status` 持续存在 source、docs、scripts 与 tests 的 tracked dirty state，而如果线程不先说明 ownership，就会让 QA 结论失去可解释性。
<small>First symptom: Current live `git status` keeps showing tracked dirty state in source, docs, scripts, and tests; without ownership notes, QA conclusions become hard to explain.</small>
- 影响范围：会污染验收范围、模糊归属，并放大“测试线程顺手改代码”的风险。
<small>Impact: It pollutes acceptance scope, blurs ownership, and amplifies the risk of test threads casually changing code.</small>
- 根因类型：流程边界失败
<small>Root-cause type: process-boundary failure</small>
- 为什么当时没挡住：dispatch 常写任务目标，但没有总是把 source/tracked file 允许边界和 dirty ownership 作为同等级约束写死。
<small>Why it escaped: Dispatches often named the task target but did not always codify source/tracked-file boundaries and dirty ownership as equal-priority constraints.</small>
- 解决过程：先尝试只靠结果汇报解释 dirty state，这条路无法把 QA 与实现边界分开；当前收敛做法是每次 QA 前后都先记录 tracked-file diff 与授权边界，但这套纪律还没有形成一次 fresh 全量闭环，所以仍保持 open。
<small>Resolution: The first attempt tried to explain dirty state only in the final report, which could not separate QA from implementation boundaries; the current safer practice is to record tracked-file diff and authorization boundaries before and after each QA step, but that discipline has not yet closed with one fresh end-to-end proof, so the issue remains open.</small>
- 修复验证证据：当前 live `git status` 仍显示 tracked dirty files；`E:\codex\AGENTS.md` 与 `E:\codex\hope-desktop-shell\docs\codex-usage-core-rules.md` 已要求线程固定记录 dirty ownership，本次新增 `E:\codex\hope-desktop-shell\docs\desktop-qa-preflight-ownership-contract.md` 又把 preflight/postflight 记录、dispatch 文件边界与 ownership blocker 固定下来；但尚缺 fresh “从启动到回报均守边界”的闭环样例。
<small>Verification evidence: Current live `git status` still shows tracked dirty files; `E:\codex\AGENTS.md` and `E:\codex\hope-desktop-shell\docs\codex-usage-core-rules.md` already require dirty-ownership recording, and this cycle adds `E:\codex\hope-desktop-shell\docs\desktop-qa-preflight-ownership-contract.md` to codify preflight/postflight recording, dispatch file boundaries, and ownership blockers; however, a fresh end-to-end bounded example is still missing.</small>
- 如何防范：所有 QA 线程都必须把“允许修改哪些 tracked files”写进 dispatch；如果没有这行，就默认 source/tracked file 全禁；`app/Cargo.toml` 需要单列 ownership gate。
<small>Prevention: Every QA thread must state which tracked files may be modified in the dispatch; if that line is missing, source/tracked files are forbidden by default; `app/Cargo.toml` needs its own ownership gate.</small>
- Playbook · 第一检查项：QA 前先记录 tracked-file status，并核对 dispatch 是否授权 touching source/tracked files。
<small>Playbook · First Check: Record tracked-file status before QA and confirm whether the dispatch authorizes touching source/tracked files.</small>
- Playbook · 必要证据：启动前 status、结束后 status、diff 摘要、dispatch 授权边界。
<small>Playbook · Evidence Required: pre-run status, post-run status, diff summary, and dispatch authorization boundary.</small>
- Playbook · 停止条件：只要出现未授权 tracked-file 变化，就停止把当前 QA 结果推进到 gate。
<small>Playbook · Stop Condition: Stop promoting the current QA result into a gate as soon as any unauthorized tracked-file change appears.</small>
- Playbook · 升级条件：QA 任务需要继续推进，但 tracked-file ownership 仍无法解释。
<small>Playbook · Escalation: Escalate if QA must continue while tracked-file ownership remains unexplained.</small>
- Playbook · 禁止捷径：不要把“只是顺手改一下”当作 QA 合法动作。
<small>Playbook · Forbidden Shortcut: Do not treat “just a quick fix” as a legitimate QA action.</small>
- 禁止重复排查：在 tracked-file ownership 未解释清楚前，不要重开业务结论争论。
<small>Do not reopen: Do not reopen business-level conclusion debates before tracked-file ownership is explained.</small>
- 当前状态：contract-pending
<small>Status: contract-pending</small>

### `HOPE-GOV-012`：bounded `/goal` 未及时使用导致反复沟通
<small>`HOPE-GOV-012`: Delayed bounded `/goal` use caused repeated communication loops</small>

- 编号：`HOPE-GOV-012`
<small>ID: `HOPE-GOV-012`</small>
- 问题陈述：同一类问题重复出现后，线程仍继续用普通往返沟通，而不是及时升级到 bounded `/goal` 或 closed-loop repair。
<small>Problem statement: After the same issue class recurred, threads continued ordinary back-and-forth communication instead of escalating promptly to bounded `/goal` or closed-loop repair.</small>
- 首次症状：第二次、第三次出现相似问题时，汇报内容越来越像重复转述，而不是收敛执行边界。
<small>First symptom: On the second and third occurrences of similar issues, reports started repeating previous descriptions instead of tightening the execution boundary.</small>
- 影响范围：会消耗上下文、拖慢决策，并让同类问题反复占用总控沟通带宽。
<small>Impact: It consumes context, slows decisions, and repeatedly occupies controller communication bandwidth.</small>
- 根因类型：沟通失真
<small>Root-cause type: communication-loss</small>
- 为什么当时没挡住：重复问题升级路径没有被明文写成“第几次出现就应该进入 bounded `/goal`”。
<small>Why it escaped: The escalation path for recurring issues was not explicitly codified as “at which repetition count bounded `/goal` should start.”</small>
- 解决过程：先沿用普通线程往返，这种做法在重复问题面前不能快速闭环；随后把 repeated-issue closed-loop 与 bounded `/goal` 规则写进 `E:\codex\AGENTS.md` 和 repo-local `codex-usage-core-rules.md`。
<small>Resolution: The first approach kept ordinary thread back-and-forth, which could not close recurring issues quickly; the later step codified repeated-issue closed-loop escalation and bounded `/goal` rules in `E:\codex\AGENTS.md` and repo-local `codex-usage-core-rules.md`.</small>
- 修复验证证据：当前全局规则与项目规则都已包含 repeated issue、closed-loop governance 与 bounded `/goal` 的升级条件。
<small>Verification evidence: The current global and project rules both contain repeated-issue, closed-loop governance, and bounded `/goal` escalation conditions.</small>
- 如何防范：同一失败类第二次出现就评估 closed-loop，第三次出现默认准备 bounded `/goal`。
<small>Prevention: Evaluate closed-loop on the second recurrence of a failure class and default to bounded `/goal` preparation on the third.</small>
- Playbook · 第一检查项：先看这是不是同一失败类的第二次或第三次出现。
<small>Playbook · First Check: Check whether this is the second or third occurrence of the same failure class.</small>
- Playbook · 必要证据：前次失败摘要、当前失败摘要、授权文件边界、验证命令与 stop conditions。
<small>Playbook · Evidence Required: previous failure summary, current failure summary, authorized file boundary, validation commands, and stop conditions.</small>
- Playbook · 停止条件：一旦当前问题明显不再是同一失败类，就停止套用 bounded `/goal` 升级逻辑。
<small>Playbook · Stop Condition: Stop applying bounded `/goal` escalation as soon as the issue is clearly no longer the same failure class.</small>
- Playbook · 升级条件：同一失败类已重复出现，且修复边界已经足够窄。
<small>Playbook · Escalation: Escalate when the same failure class recurs and the repair boundary is already narrow enough.</small>
- Playbook · 禁止捷径：不要把 repeated issue 继续当成“再解释一次就能解决”的普通沟通问题。
<small>Playbook · Forbidden Shortcut: Do not keep treating a repeated issue as an ordinary “one more explanation” communication problem.</small>
- 禁止重复排查：在 bounded `/goal` 触发条件满足后，不要继续原地转述。
<small>Do not reopen: Do not continue plain restatement after the bounded `/goal` trigger condition is met.</small>
- 当前状态：closed
<small>Status: closed</small>

### `HOPE-GOV-013`：模型选择强制规则
<small>`HOPE-GOV-013`: Model assignment enforcement</small>

- 编号：`HOPE-GOV-013`
<small>ID: `HOPE-GOV-013`</small>
- 问题陈述：线程若被要求自证内部模型元数据，会在创建或切换阶段被无谓阻断，影响主线推进。
<small>Problem statement: Threads stall unnecessarily when asked to self-prove hidden internal model metadata during creation or switching, slowing mainline progress.</small>
- 首次症状：总控已经固定模型，但线程仍卡在“如何证明自己当前就是这个模型”。
<small>First symptom: The controller had already fixed the model, yet the thread still stalled on proving that it was using that exact model.</small>
- 影响范围：会制造无意义 blocker，并拖慢文档、治理与 QA 线程。
<small>Impact: It creates unnecessary blockers and slows documentation, governance, and QA threads.</small>
- 根因类型：沟通失真
<small>Root-cause type: communication-loss</small>
- 为什么当时没挡住：模型建议一度只被理解为“建议”，而不是总控固定约束。
<small>Why it escaped: Model guidance was once treated as advisory rather than controller-fixed.</small>
- 解决过程：先尝试让线程自证内部元数据，这条路与实际可见边界不匹配；随后改成总控 dispatch 直接写死模型，线程只负责按该模型执行并在回报中写“实际使用模型”。
<small>Resolution: The first attempt asked the thread to prove hidden internal metadata, which did not match the visible boundary; the later step changed dispatches so the controller fixes the model and the thread only executes under that model and reports the actual model used.</small>
- 修复验证证据：当前线程指令已经固定 `GPT-5.4 medium`，本回报也将显式写出该值。
<small>Verification evidence: The current thread instruction already fixes `GPT-5.4 medium`, and this report explicitly states that value.</small>
- 如何防范：以后只要模型层级重要，dispatch 必须写死模型与推理档位。
<small>Prevention: Whenever model tier matters, the dispatch must explicitly fix the model and reasoning level.</small>
- Playbook · 第一检查项：先看最新 dispatch 是否已写明模型与推理档位。
<small>Playbook · First Check: Inspect the latest dispatch for an explicit model and reasoning-level field.</small>
- Playbook · 必要证据：dispatch 模型字段、线程 create/switch 动作、回报中的“实际使用模型”。
<small>Playbook · Evidence Required: dispatch model field, thread create/switch action, and the “actual model used” line in the report.</small>
- Playbook · 停止条件：如果当前线程模型与 dispatch 固定模型不一致，就停止执行并要求重开或切换。
<small>Playbook · Stop Condition: Stop execution and request reopen/switch if the active thread model does not match the dispatch-fixed model.</small>
- Playbook · 升级条件：线程无法按要求重开或切换到指定模型。
<small>Playbook · Escalation: Escalate if the thread cannot be reopened or switched to the required model.</small>
- Playbook · 禁止捷径：不要要求线程读取内部模型元数据来证明“足够正确”。
<small>Playbook · Forbidden Shortcut: Do not require hidden model-metadata introspection as proof of correctness.</small>
- 禁止重复排查：在 dispatch 已经固定模型时，不要再把内部元数据不可见当成 blocker。
<small>Do not reopen: Do not treat invisible internal metadata as a blocker once the dispatch has already fixed the model.</small>
- 当前状态：closed
<small>Status: closed</small>

### `HOPE-SAMPLE-014`：21/27 样本 reference-only / formal import blocked 边界
<small>`HOPE-SAMPLE-014`: 21/27 sample boundary stays reference-only while formal import remains blocked</small>

- 编号：`HOPE-SAMPLE-014`
<small>ID: `HOPE-SAMPLE-014`</small>
- 问题陈述：21/27 样本集虽然适合 QA/reference，但在 formal import 未开启前，不能被误当作 runtime truth 或当前 gate evidence。
<small>Problem statement: The 21/27 sample sets may be strong QA/reference material, but before formal import opens they must not be mistaken for runtime truth or current gate evidence.</small>
- 首次症状：样本文本质量较高时，容易被过读成“已经能直接支撑主线判断”。
<small>First symptom: High-quality sample text is easily over-read as if it can already support mainline judgment directly.</small>
- 影响范围：会污染 runtime 边界、样本使用边界与 formal import 叙述。
<small>Impact: It pollutes runtime boundaries, sample-usage boundaries, and formal-import narratives.</small>
- 根因类型：数据污染
<small>Root-cause type: data-pollution</small>
- 为什么当时没挡住：QA/reference 价值很高，容易让样本看上去“几乎就等于正式源”。
<small>Why it escaped: Strong QA/reference value makes the sample sets look almost like formal source material.</small>
- 解决过程：先把样本草案当作强参考使用，这种做法会模糊 runtime 边界；当前收敛做法是把样本集固定标成 `reference-only`，直到 formal import gate 真正开启。
<small>Resolution: The first approach treated the draft sample sets as strong reference and blurred runtime boundaries; the current bounded approach is to keep the sets explicitly marked `reference-only` until the formal-import gate actually opens.</small>
- 修复验证证据：现有允许读取的样本边界材料只支持 QA/reference 使用，不提供 fresh runtime 或 formal import closure evidence。
<small>Verification evidence: The currently allowed sample-boundary materials support QA/reference usage only and do not provide fresh runtime or formal-import closure evidence.</small>
- 如何防范：今后所有样本资料都必须显式区分 QA/reference、runtime-ready 与 formal-import-ready。
<small>Prevention: All future sample materials must explicitly separate QA/reference, runtime-ready, and formal-import-ready states.</small>
- Playbook · 第一检查项：先看样本当前标签是不是 `reference-only`。
<small>Playbook · First Check: Check whether the sample is currently tagged `reference-only`.</small>
- Playbook · 必要证据：样本状态标签、review 状态、formal import readiness 说明。
<small>Playbook · Evidence Required: sample-state tag, review status, and formal-import readiness note.</small>
- Playbook · 停止条件：只要 formal import readiness 不成立，就停止把样本推进到 runtime truth。
<small>Playbook · Stop Condition: Stop pushing the sample into runtime truth as soon as formal-import readiness is not established.</small>
- Playbook · 升级条件：有人要求拿样本直接支撑当前 live gate 或 runtime 行为。
<small>Playbook · Escalation: Escalate if anyone tries to use the sample set directly for the current live gate or runtime behavior.</small>
- Playbook · 禁止捷径：不要因为样本“写得好”就把它升级成 fresh gate evidence。
<small>Playbook · Forbidden Shortcut: Do not upgrade a sample into fresh gate evidence just because it is well written.</small>
- 禁止重复排查：在 formal import gate 未开启前，不要重开“样本是否可直接入 runtime”的争论。
<small>Do not reopen: Do not reopen the “can the sample go directly into runtime?” debate before the formal-import gate opens.</small>
- 当前状态：reference-only
<small>Status: reference-only</small>

### `HOPE-SEC-015`：secret / env / API key 本地化与 Git 泄露防线
<small>`HOPE-SEC-015`: Local secret/env/API key discipline and Git leak barrier</small>

- 编号：`HOPE-SEC-015`
<small>ID: `HOPE-SEC-015`</small>
- 问题陈述：主线排查中若把 raw env、raw prompt、raw response、API key 或 raw source_register 当作“方便证据”，就会直接破坏文档与 Git 安全边界。
<small>Problem statement: If raw env, raw prompt, raw response, API key, or raw source_register is treated as “convenient evidence” during mainline diagnosis, the document and Git safety boundary is broken immediately.</small>
- 首次症状：操作者需要 provider readiness 细节时，容易多暴露一层原始字段。
<small>First symptom: When provider-readiness details are needed, it becomes easy to over-expose one more raw layer.</small>
- 影响范围：会造成 secret 暴露、文档不可共享与 Git 历史污染。
<small>Impact: It causes secret exposure, non-shareable documents, and Git-history contamination.</small>
- 根因类型：流程边界失败
<small>Root-cause type: process-boundary failure</small>
- 为什么当时没挡住：raw 信息看起来最直接，且“只在本地看一下”的心理门槛过低。
<small>Why it escaped: Raw information looks like the fastest path, and the local-only mental threshold is too low.</small>
- 解决过程：先依赖原始字段做本地判断，这种路径无法进入共享文档；随后改为只用 sanitized presence/status、类别码与脱敏摘要，并把 raw 内容全面排除出文档与回报。
<small>Resolution: The first approach relied on raw fields for local judgment, which cannot enter shared documentation; the later approach switched to sanitized presence/status, category codes, and redacted summaries while excluding raw content from documents and reports.</small>
- 修复验证证据：`E:\codex\hope-desktop-shell\docs\desktop-release-qa-handoff.md` 与 `E:\codex\hope-desktop-shell\docs\codex-usage-core-rules.md` 都明确禁止 raw secret-bearing 内容进入文档、日志或导出产物。
<small>Verification evidence: `E:\codex\hope-desktop-shell\docs\desktop-release-qa-handoff.md` and `E:\codex\hope-desktop-shell\docs\codex-usage-core-rules.md` both explicitly forbid raw secret-bearing content in docs, logs, or exported artifacts.</small>
- 如何防范：任何共享输出只允许布尔值、枚举值、类别码与脱敏摘要，不允许秘密原文。
<small>Prevention: Any shared output may contain only booleans, enums, category codes, and redacted summaries, never raw secret-bearing text.</small>
- Playbook · 第一检查项：先问当前诊断是否能完全用 sanitized 字段表达。
<small>Playbook · First Check: Ask whether the current diagnosis can be expressed fully with sanitized fields.</small>
- Playbook · 必要证据：presence/status flags、category codes、脱敏摘要。
<small>Playbook · Evidence Required: presence/status flags, category codes, and redacted summaries.</small>
- Playbook · 停止条件：一旦发现需要粘贴 raw secret-bearing 内容，就立即停止。
<small>Playbook · Stop Condition: Stop immediately when raw secret-bearing content would need to be pasted.</small>
- Playbook · 升级条件：如果唯一的调试路径似乎必须依赖 raw 内容，就升级到更高权限的本地非共享排查。
<small>Playbook · Escalation: Escalate into a higher-privilege local non-shared diagnosis path if raw content appears to be the only available debugging route.</small>
- Playbook · 禁止捷径：不要为了“这次方便”就贴 raw env、raw prompt 或 raw provider response。
<small>Playbook · Forbidden Shortcut: Do not paste raw env, raw prompt, or raw provider response for convenience.</small>
- 禁止重复排查：在 sanitized-only 方案仍可表达时，不要重开 raw 内容暴露路径。
<small>Do not reopen: Do not reopen a raw-content path while a sanitized-only path still exists.</small>
- 当前状态：closed
<small>Status: closed</small>

### `HOPE-GATE-016`：当前 gate 层级容易被单案结果过读
<small>`HOPE-GATE-016`: The current gate ladder is still vulnerable to over-reading from a single case</small>

- 编号：`HOPE-GATE-016`
<small>ID: `HOPE-GATE-016`</small>
- 问题陈述：单案、本地 targeted tests 或单模型结果容易被过读成 `qwen3.6-plus` live3、required text model pool live3、`full16` 甚至更高 gate 的通过信号。
<small>Problem statement: Single-case, local targeted-test, or single-model results are too easily over-read as passing `qwen3.6-plus` live3, required-pool live3, `full16`, or even higher gates.</small>
- 首次症状：在还没有 fresh live3 三案回报时，口径已经倾向于把“某一步通过”写成“主线快恢复了”。
<small>First symptom: Before any fresh live3 three-case report exists, wording already starts drifting toward “the mainline is nearly restored.”</small>
- 影响范围：会放宽 gate 边界，制造假关闭，并把后续 `403-case` 与 `package` 准备建立在不够新鲜的证据上。
<small>Impact: It weakens gate boundaries, creates false closure, and risks building `403-case` and `package` readiness on evidence that is not fresh enough.</small>
- 根因类型：沟通失真
<small>Root-cause type: communication-loss</small>
- 为什么当时没挡住：gate 层级虽已文档化，但“什么不能过读”为何时必须停下的规则还不够高频地出现在每次汇报里。
<small>Why it escaped: The gate ladder is documented, but the rule for what may not be over-read is not repeated prominently enough in each report.</small>
- 解决过程：先用局部通过信号推动整体乐观判断，这种做法没有对应 fresh gate evidence；随后把 gate ladder 明确写成“单案 -> `qwen3.6-plus` live3 3-case -> required pool live3 -> `full16` -> `403-case` -> `package`”，并明确写出当前没有 fresh 三案通过回报，因此只能保持 pending。
<small>Resolution: The first approach let local success signals fuel broad optimism without matching fresh gate evidence; the later approach rewrote the ladder explicitly as “single case -> `qwen3.6-plus` live3 3-case -> required pool live3 -> `full16` -> `403-case` -> `package`” and stated clearly that without a fresh three-case report the status remains pending.</small>
- 修复验证证据：`E:\codex\hope-desktop-shell\docs\desktop-release-qa-handoff.md` 已明确 required text gate pool、live3、`full16` 与 higher gates 的层级；本次新增 `E:\codex\hope-desktop-shell\docs\desktop-gate-evidence-ladder-contract.md` 进一步把 single case、`qwen3.6-plus` live3、required pool、`full16`、`403-case` 与 package 的 acceptance evidence 固定下来；当前线程仍没有收到 fresh live3 三案通过回报。
<small>Verification evidence: `E:\codex\hope-desktop-shell\docs\desktop-release-qa-handoff.md` already defines the hierarchy among the required text gate pool, live3, `full16`, and higher gates; this cycle adds `E:\codex\hope-desktop-shell\docs\desktop-gate-evidence-ladder-contract.md` to formalize acceptance evidence for single case, `qwen3.6-plus` live3, required pool, `full16`, `403-case`, and package; the current thread still has no fresh live3 three-case pass report.</small>
- 如何防范：以后所有 gate 汇报都必须同时写“当前层级”和“尚未解锁的上一层与下一层”；不得再把 `certifier_ok=true`、targeted tests passed、单案结果或旧 artifact 上读成更高 gate。
<small>Prevention: Every future gate report must name both the current layer and the still-locked lower and higher adjacent layers; `certifier_ok=true`, targeted tests passed, single-case results, and old artifacts must never be read upward into a higher gate.</small>
- Playbook · 第一检查项：先问手里的证据究竟对应哪一层 gate，而不是“感觉像快好了”。
<small>Playbook · First Check: Ask which exact gate layer the current evidence belongs to, not whether it “feels close to done.”</small>
- Playbook · 必要证据：fresh 回报时间、对应 gate 名称、expected/executed/passed/failed 计数或等价状态字段。
<small>Playbook · Evidence Required: fresh report timestamp, target gate name, and expected/executed/passed/failed counts or equivalent status fields.</small>
- Playbook · 停止条件：一旦发现证据只能支撑更低层级，就停止往更高 gate 过读。
<small>Playbook · Stop Condition: Stop over-reading upward as soon as the evidence supports only a lower gate layer.</small>
- Playbook · 升级条件：有人要求在缺少 fresh live3 或 required-pool evidence 时直接讨论 `full16`、`403-case` 或 `package`。
<small>Playbook · Escalation: Escalate if anyone asks to discuss `full16`, `403-case`, or `package` without fresh live3 or required-pool evidence.</small>
- Playbook · 禁止捷径：不要把 `certifier_ok=true`、targeted tests passed 或单案结果写成更高 gate passed。
<small>Playbook · Forbidden Shortcut: Do not rewrite `certifier_ok=true`, targeted tests passed, or a single-case result as a higher gate pass.</small>
- 禁止重复排查：在 fresh live3 三案通过回报出来前，不要重开“主线已经恢复到更高 gate”的叙述。
<small>Do not reopen: Do not reopen the narrative that the mainline has recovered to a higher gate before a fresh live3 three-case pass report exists.</small>
- 当前状态：open
<small>Status: open</small>

## 8. 下一 gate 前自检清单
<small>8. Pre-next-gate Self-check Checklist</small>

☐ 每条错题都已经填写 16 个字段。
<small>☐ Every ledger entry has all 16 required fields.</small>
☐ 每条错题都包含 Problem statement、Resolution、Prevention 与 Playbook 五子字段。
<small>☐ Every ledger entry contains Problem statement, Resolution, Prevention, and all 5 Playbook sub-fields.</small>
☐ 失败尝试已经写进解决过程，而不是只保留最后一步。
<small>☐ Failed attempts are recorded in Resolution instead of keeping only the last step.</small>
☐ 第一检查项都可以在 5 分钟内执行。
<small>☐ Every First Check is executable within 5 minutes.</small>
☐ 首次症状写的是可复现的可观测信号，而不是根因。
<small>☐ Every First symptom is a reproducible observable signal, not a root cause.</small>
☐ 根因类型只使用受限词表：环境漂移、流程边界失败、契约缺失、错误证据、工具误用、沟通失真、人为疏忽、第三方变更、数据污染、设计缺陷。
<small>☐ Root-cause types only use the controlled vocabulary.</small>
☐ 没有写“我觉得”“应该是”“估计”这类主观措辞。
<small>☐ No subjective wording such as “I think,” “should be,” or “probably” remains.</small>
☐ 没有甩锅表达；个人失误都已改写成系统、流程或 dispatch 缺口。
<small>☐ No blame language remains; personal-lapse wording has been converted into system, process, or dispatch gaps.</small>
☐ 没有把 open 写成 closed，也没有把 reference-only 写成 fresh evidence。
<small>☐ No open issue is written as closed, and no reference-only item is written as fresh evidence.</small>
☐ 没有输出 secret、raw env、API key、raw provider response、raw prompt、source_register 或 overlay JSON。
<small>☐ No secret, raw env, API key, raw provider response, raw prompt, source_register, or overlay JSON is exposed.</small>
☐ 中文在上、英文在下，没有出现同一行混排的 `English / 中文` 或 `中文 / English`。
<small>☐ Chinese stays on the first line and English on the second, with no same-line bilingual mixing.</small>
☐ DOCX 英文辅助行比中文小 2 号，并完成至少 3 页抽查。
<small>☐ DOCX helper lines are 2 pt smaller than Chinese lines, and at least 3 pages were spot-checked.</small>
☐ Markdown 与 DOCX 的结构、章节顺序与字段内容一致。
<small>☐ Markdown and DOCX match in structure, section order, and field content.</small>
☐ 桌面文件夹内 `项目进度查错文档.md` 与 `项目进度查错文档.docx` 已同步刷新。
<small>☐ The Desktop folder has refreshed copies of both `项目进度查错文档.md` and `项目进度查错文档.docx`.</small>
☐ 当前 gate 的 pending 项已明确写出缺什么 fresh evidence、下一步是什么。
<small>☐ Every pending gate item explicitly states what fresh evidence is missing and what the next step is.</small>

## 9. 后续维护规则
<small>9. Maintenance Rules</small>

- 本文件的 source-of-truth 是 Markdown；每次修改先改 `项目进度查错文档.md`，再同步重生 DOCX。
<small>The source of truth for this document is Markdown; every update edits `项目进度查错文档.md` first and then regenerates the DOCX.</small>
- 只要 open / closed / reference-only 分类变化，就必须同时更新第 4、5、6 章索引与第 7 章详细条目。
<small>Any change in open, closed, or reference-only classification must update Chapters 4, 5, 6, and the detailed entry in Chapter 7 together.</small>
- 每次 close-out 都必须重新运行自检清单，并记录文件路径、大小与 LastWriteTime。
<small>Every close-out must rerun the self-check list and record file path, size, and LastWriteTime.</small>
- 双语排版必须持续遵守：中文第一行、英文第二行、Markdown 用 `<small>...</small>`、DOCX 英文小 2 号。
<small>Bilingual formatting must keep the same discipline: Chinese first line, English second line, Markdown with `<small>...</small>`, and DOCX helper lines 2 pt smaller.</small>
- 如果新的问题没有 fresh closure evidence，就只允许写成 open 或 reference-only，不得提前写成 closed。
<small>If a new issue lacks fresh closure evidence, it may only be written as open or reference-only and may not be upgraded to closed early.</small>

## 10. 2026-05-07 WebView2 Proxy-Isolation Update
<small>10. 2026-05-07 WebView2 proxy-isolation update</small>

This update does not reopen or replace `HOPE-SHELL-004`. It records a bounded
correction inside the existing shell/CDP startup playbook: all release shell /
WebView2 / CDP launch paths must isolate proxy environment before starting
`hope-app.exe`.

Updated prevention rule:

- `AppDefault + LaunchDiagnosticOnly` remains the only formal shell gate.
- `StopOnly` clean is required cleanup evidence, but it is not a shell pass by
  itself.
- `--noerrdialogs`, `--disable-breakpad`, and `--disable-crash-reporter` remain
  forbidden as popup-suppression shortcuts.
- The launcher must clear `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `NO_PROXY`,
  `http_proxy`, `https_proxy`, `all_proxy`, and `no_proxy` before starting
  `hope-app.exe`.
- `-NoProxy` is QA/provider semantics and evidence only. It must not be the
  only trigger for WebView2 environment isolation.
- The launcher must fail closed before starting `hope-app.exe` when
  `webview2_process_query_capability.query_ok=false` or command-line inspection
  is denied; otherwise `StopOnly` cannot prove that WebView2 popup / crash
  residue has been cleaned.
- `StopOnly` cleanup scope is Hope-owned WebView2 only: command lines naming
  `hope-app.exe` or the `hope-webview2-cdp` temp profile. System WebView2
  processes owned by Windows surfaces such as `SearchHost.exe` or `Widgets.exe`
  must be recorded as non-Hope context and must not be killed or reported as
  Hope release-shell residue.
- Artifacts must record sanitized evidence only:
  `launcher_process_env_proxy_present_before`,
  `webview2_launch_env_proxy_cleared`, `app_process_env_proxy_present`, and
  `no_proxy_loopback_only`, plus
  `webview2_process_query_capability.query_ok`.
- If `0x80000003` or `HRESULT(0x8000FFFF)` appears, stop business gates and
  route to shell/CDP blocker handling; do not continue targeted, `full16`,
  `403-case`, provider, runner, or package work.

Fresh 2026-05-07 validation evidence:

- `AppDefault + LaunchDiagnosticOnly`
- `ok=true`
- `cdp_ready=true`
- `target_url=http://tauri.localhost/#/workbench`
- `webview2_popup_suppression_detected=false`
- `webview2_app_popup_suppression_arg_present=false`
- `webview2_launch_env_proxy_cleared=true`
- `app_process_env_proxy_present=false`
- paired `StopOnly` clean

## 11. 2026-05-08 Contract-Boundary Update
<small>11. 2026-05-08 contract-boundary update</small>

This update records a docs-only boundary expansion. It does not mark any
business QA gate as passed and does not close targeted, `full16`, `formal403`,
or package readiness.
<small>This update records a docs-only boundary expansion. It does not mark any business QA gate as passed and does not close targeted, `full16`, `formal403`, or package readiness.</small>

Added companion contracts:
<small>Added companion contracts:</small>

- `docs/hope-field-aware-story-contract.md`
- `docs/hope-provider-failover-contract.md`
- `docs/hope-qa-evidence-freshness-contract.md`

Boundary now fixed:
<small>Boundary now fixed:</small>

- story stage order from `SeedSourceFrame` to `StoryboardInputFrame`
- narrative-first `扩写故事` / `改写剧本` body
- field-aware entity resolution so situation/title words such as `危局` do not
  become `person`
- all 21 scene types through unified `SceneProfile`
- target duration as narrative capacity
- KB oracle positive and negative evidence
- provider failover only for quota, unavailable model, or entitlement /
  permission HTTP 403
- evidence freshness fields and stale-trigger rules
- UTF-8 and Chinese sentinel preflight for `runtime.rs`

Next closure evidence required:
<small>Next closure evidence required:</small>

- implementation thread must report exact file allowlist before edits
- runtime/UI/runner/certifier/matrix changes must produce fresh artifact
  identity
- targeted A/B/C/D must rerun fresh before `full16`
- `full16` must rerun fresh before `formal403`
- no old release exe, old runner output, old certifier output, or old CDP
  artifact may be counted as current pass evidence

## 12. 2026-05-08 Bounded Goal Error Cards
<small>12. 2026-05-08 bounded goal error cards</small>

This section is updated while the bounded `/goal` repair is running. It is an
error ledger update only; it does not mark targeted, `full16`, `formal403`, or
package gates as passed.

### `HOPE-SHELL-017`: WebView2 popup / CDP launch failure

- 错题编号: `HOPE-SHELL-017`
- 问题现象: release shell may fail to materialize the CDP target, or earlier
  runs may show WebView2 popup/crash symptoms such as `0x80000003` or
  `HRESULT(0x8000FFFF)`.
- 错误做法: masking the symptom with `--noerrdialogs`,
  `--disable-breakpad`, or `--disable-crash-reporter`, or treating `StopOnly`
  clean as a shell pass.
- 根因: shell launch, WebView2 profile, proxy inheritance, and CDP target
  readiness were previously conflated.
- 正确处理: run `AppDefault + LaunchDiagnosticOnly`, isolate proxy env before
  launch, require `target_url=http://tauri.localhost/#/workbench`, then run
  `StopOnly`.
- 必带证据: `ok=true`, `cdp_ready=true`, expected `target_url`,
  `webview2_launch_env_proxy_cleared=true`,
  `app_process_env_proxy_present=false`, popup suppression fields false, paired
  `StopOnly` with no Hope-owned WebView2 residue.
- 证据判读补充: do not inject `--noerrdialogs`,
  `--disable-breakpad`, or `--disable-crash-reporter` through launcher or app
  WebView2 arguments. If a WebView2 Runtime child process reports an internal
  `has_noerrdialogs=true`, it is not sufficient by itself to fail the gate; the
  hard evidence is `webview2_popup_suppression_detected=false`,
  `webview2_app_popup_suppression_arg_present=false`,
  `webview2_disable_breakpad_seen=false`, and
  `webview2_disable_crash_reporter_seen=false`.
- 防复发规则: any popup/crash or wrong-target signal stops business gates and
  routes to shell/CDP repair first.
- 当前验证状态: active; 2026-05-08 AppDefault gate on release exe
  `C09056140DC7F777369D52C1E834EA4619D0896E190769B69E3AD8F532B814BF` passed,
  followed by `StopOnly` clean. This is shell evidence only and does not mark
  targeted or `full16` passed.
- 相关文件 / artifact 类型: `scripts/start-hope-release-cdp.ps1`,
  launch JSON, stop-only JSON.
- 是否仍有风险: yes; WebView2 process query can require elevated permission on
  this machine and must be handled before business gates.

### `HOPE-SHELL-018`: proxy env contamination

- 错题编号: `HOPE-SHELL-018`
- 问题现象: `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, or `NO_PROXY` inherited
  from the parent process can pollute WebView2 launch or provider QA evidence.
- 错误做法: assuming `-NoProxy` alone is the WebView2 isolation switch.
- 根因: provider QA semantics and WebView2 process environment isolation were
  previously coupled.
- 正确处理: release shell / CDP startup clears proxy env by default before
  launching `hope-app.exe`; `-NoProxy` remains QA/provider evidence only.
- 必带证据: sanitized before/after proxy presence,
  `webview2_launch_env_proxy_cleared=true`,
  `app_process_env_proxy_present=false`, no raw env values.
- 防复发规则: every release shell launch artifact must include sanitized proxy
  evidence.
- 当前验证状态: active and enforced by launcher evidence.
- 相关文件 / artifact 类型: `scripts/start-hope-release-cdp.ps1`, launch JSON.
- 是否仍有风险: low if launch JSON is fresh and sanitized.

### `HOPE-SRC-019`: `runtime.rs` non-ASCII corruption

- 错题编号: `HOPE-SRC-019`
- 问题现象: Chinese contract sentinels in `app/src/runtime.rs` become corrupted,
  causing compile or semantic failures.
- 错误做法: whole-file PowerShell/Node mechanical replacement on files
  containing non-ASCII text.
- 根因: unsafe text rewrite path and missing source integrity preflight.
- 正确处理: save forensic copy, SHA256, and diff; export clean base from HEAD;
  rebuild by small `apply_patch` edits or other UTF-8 safe scoped edits.
- 必带证据: forensic artifact path, `runtime_rs_sha256`, UTF-8 readable check,
  replacement-char absent, sentinel check for key Chinese terms.
- 防复发规则: source integrity preflight must run before targeted/full16/formal
  gates.
- 当前验证状态: active; runtime tests passed after recovery in this chain.
- 相关文件 / artifact 类型: `app/src/runtime.rs`, forensic target directory.
- 是否仍有风险: medium; avoid full-file mechanical rewrites.

### `HOPE-ENTITY-020`: field-unaware entity drift

- 错题编号: `HOPE-ENTITY-020`
- 问题现象: situation/title/action fragments such as `危局`, `林峰压`, or
  `苏瑶并` can be misread as source-external character names.
- 错误做法: adding one more broad noise word each time a false positive appears.
- 根因: global string scanning without field role semantics.
- 正确处理: use `FieldAwareEntityResolver` with `FieldRole` and `EntityKind`.
  `person` stays strict; `shot_title` may classify situation terms;
  `character_action` may classify action fragments; real external characters
  still hard fail.
- 必带证据: tests for `shot_title=危局` pass, `person=危局` fail,
  `character_action` fragments do not become names, and real external names
  remain blocked.
- 防复发规则: every new false-positive category needs a field-aware test, not a
  wider whitelist.
- 当前验证状态: active; covered by runtime field-aware tests.
- 相关文件 / artifact 类型: `app/src/runtime.rs`, Rust tests.
- 是否仍有风险: medium; new field types must be classified before gate advance.

### `HOPE-STORY-021`: narrative body replaced by strategy text

- 错题编号: `HOPE-STORY-021`
- 问题现象: expand/rewrite main body becomes action choreography, scene
  strategy, duration plan, KB note, storyboard breakdown, or prompt-like text.
- 错误做法: putting support explanations into the main textarea or
  `acceptedConfirmationBody`.
- 根因: `NarrativeStoryBody` and `SupportNotes` boundary was not hard enough.
- 正确处理: main story body is first and complete; support notes may appear only
  below the body or in trace/artifacts and must not become accepted source.
- 必带证据: narrative body has character, goal, causality, emotional turn, and
  resolution beat; no strategy/trace/prompt/KB explanation in user body.
- 防复发规则: confirmation freezes only the narrative body, and storyboard input
  derives from that confirmed body.
- 当前验证状态: active; targeted A/B/C evidence passed earlier in this chain, but
  latest-code group rerun is still required before `full16`.
- 相关文件 / artifact 类型: `app/src/runtime.rs`, `ui/src/App.tsx`, runner JSON.
- 是否仍有风险: yes until all targeted cases are rerun on the latest release exe.

### `HOPE-SCENE-022`: scene and duration changed only as labels

- 错题编号: `HOPE-SCENE-022`
- 问题现象: changing `scene_type` or `target_duration` changes metadata but not
  the story structure or storyboard rows.
- 错误做法: checking only selected label text or duration string.
- 根因: missing or incomplete `SceneProfile` and `DurationCapacity` application.
- 正确处理: every scene uses a unified profile with expression, space,
  conflict, rhythm, camera, positive tags, and drift bans; duration becomes
  beat count, paragraph capacity, action density, and shot duration allocation.
- 必带证据: story changes when scene/duration changes; rows bind current scene
  and current total duration; no old scene style residue.
- 防复发规则: do not advance from targeted to `full16` until scene and duration
  evidence is fresh for all A/B/C/D cases.
- 当前验证状态: active; targeted D exposed a visible duration evidence gap and is
  being repaired before proceeding.
- 相关文件 / artifact 类型: `app/src/runtime.rs`, UI table, runner JSON.
- 是否仍有风险: yes until targeted D and latest-code targeted group pass.

### `HOPE-KB-023`: KB trace-only pseudo evidence

- 错题编号: `HOPE-KB-023`
- 问题现象: artifacts contain KB hash or rule ids but do not prove that KB
  affected story or storyboard structure.
- 错误做法: treating KB trace presence as content correctness.
- 根因: positive structural influence and raw leakage guards were separate.
- 正确处理: require `kb_oracle_present`, `affects_structure`, rule pack evidence,
  and raw-leak absence together.
- 必带证据: KB oracle hashes/rule packs, structure-change evidence, raw KB rows
  absent, raw sample text absent.
- 防复发规则: KB evidence must be positive and negative; trace-only is not a
  pass.
- 当前验证状态: active; runner/certifier carry KB oracle gates.
- 相关文件 / artifact 类型: runner JSON, certifier JSON, KB mapping JSON.
- 是否仍有风险: medium; formal matrix must inherit this gate.

### `HOPE-PROV-024`: provider failover confused with fallback

- 错题编号: `HOPE-PROV-024`
- 问题现象: model switching can accidentally hide validator, grounding, rows,
  KB, or CDP failures.
- 错误做法: using the secondary model as a general retry for business gate
  failures.
- 根因: provider failover was not typed narrowly enough.
- 正确处理: only quota, unavailable model, or entitlement/permission HTTP 403
  may switch from `qwen3.6-plus` to `qwen3.6-plus-2026-04-02`.
- 必带证据: `attempted_models`, `final_model`, `provider_failover_used`,
  `provider_failover_reason`, HTTP status fields, `fallback_used=false`,
  `local_candidate=false`.
- 防复发规则: validator/grounding/rows/KB/CDP failures remain hard gates and do
  not become provider success.
- 当前验证状态: active; latest targeted D evidence used primary model only and no
  local fallback.
- 相关文件 / artifact 类型: `app/src/runtime.rs`, runner JSON, certifier JSON.
- 是否仍有风险: low if typed failover evidence is preserved.

### `HOPE-ROWS-025`: `rows_match` pseudo success

- 错题编号: `HOPE-ROWS-025`
- 问题现象: `rows_match=true` can be over-read when runtime was blocked or rows
  are empty/partial.
- 错误做法: treating `rows_match` as full content correctness.
- 根因: equality and non-empty/runtime-success checks were not bound together.
- 正确处理: require `rows_non_empty`, `runtime_blocked=false`,
  `validator_hard_gate_fail=false`, prompt boundary pass, and row diffs empty.
- 必带证据: response rows count, UI rows count, row hashes, row diffs,
  validator gate pass.
- 防复发规则: `rows_match=true` is necessary but never sufficient.
- 当前验证状态: active; runner hard gate includes pseudo-success guard.
- 相关文件 / artifact 类型: `scripts/hope-ui-driven-trace-runner.mjs`,
  runner JSON.
- 是否仍有风险: medium; partial visible table evidence created the targeted D
  duration blocker.
- 2026-05-09 addendum: fresh full16 case
  `rewrite_scene_same_duration_different_text_same` repeated the same boundary
  with `response_rows_count=0`, `ui_rows_count=0`, `rows_match=true`, and
  `validator_gate_passed=true`, while top-level accepted snapshot /
  StoryFactFrame hashes were empty strings.
- 2026-05-09 wrong fix path: do not route this class into provider / `403` /
  CDP lanes, do not add word allowlists, and do not read `rows_match=true` or
  `validator_gate_passed=true` alone as a pass.
- 2026-05-09 prevention extension: blocked `generate_storyboard` responses must
  preserve sanitized binding hashes, UI QA trace must fail the validator gate
  on `status=Blocked` or zero rows, and runner/certifier must require non-empty
  rows plus non-empty binding hashes together.

### `HOPE-FRAME-026`: prompt and visible-frame contamination

- 错题编号: `HOPE-FRAME-026`
- 问题现象: user-facing `prompt_text` contains internal identifiers, or
  `visual_description` reads like strategy/anchor notes instead of a visible
  frame.
- 错误做法: renaming internal fields or deleting the checker to pass.
- 根因: renderer mixed internal evidence strings with user-facing fields.
- 正确处理: render `visual_description` from `VisibleFrame`; render
  `prompt_text` in layers: shot goal, image, action, camera, dialogue, and
  constraints.
- 必带证据: prompt boundary pass, forbidden internal-source terms absent,
  visible frame has scene, people, positions, action, depth, camera, motion
  trace, and material/lighting.
- 防复发规则: internal evidence may stay in trace/artifact only, never in
  user-visible row fields.
- 当前验证状态: active; targeted B/C repairs passed visual/prompt gates, and
  targeted D v5 fresh pass verified composite-person visible-frame rendering.
- 相关文件 / artifact 类型: `app/src/runtime.rs`, UI rows, runner JSON.
- 是否仍有风险: medium; latest-code targeted group rerun is required.

### `HOPE-EVID-027`: stale release exe or stale artifact over-read

- 错题编号: `HOPE-EVID-027`
- 问题现象: runtime/UI/runner changes are followed by claims based on an older
  release exe, old CDP profile, or old targeted/full16 artifact.
- 错误做法: reading a prior pass as a current pass after source changes.
- 根因: freshness identity was not enforced at every gate.
- 正确处理: record runtime, runner, certifier, matrix, KB mapping, dirty patch,
  and release exe SHA256; rebuild when source is newer than exe.
- 必带证据: `release_exe_sha256`, `runtime_rs_sha256`, runner/certifier/matrix
  hashes, artifact start/finish time, parent gate hash.
- 防复发规则: every source or runner change invalidates downstream release gate
  evidence until rebuild and fresh rerun.
- 当前验证状态: active; current release exe SHA256 is recorded for targeted D v5:
  `C59F5DAEAEE71A2C08ADE5D2B0477044275FEF3A509A38E0FA6660FCD6402156`.
- 相关文件 / artifact 类型: release exe, launch JSON, runner JSON.
- 是否仍有风险: yes until all targeted cases rerun on the latest exe.

### `HOPE-BUILD-028`: Tauri schema / `app/Cargo.toml` build side effects

- 错题编号: `HOPE-BUILD-028`
- 问题现象: `cargo tauri build` can dirty generated schema files or
  `app/Cargo.toml`, creating out-of-scope changes.
- 错误做法: bulk reset, stash, clean, or mixing build side effects into business
  changes.
- 根因: build tooling writes tracked files outside the active implementation
  surface.
- 正确处理: save forensic copies, diffs, SHA256, and status; restore only
  controller-approved build side-effect paths to HEAD; never touch unrelated
  dirty.
- 必带证据: forensic directory, exact restore path list, post-restore status,
  release exe hash preserved.
- 防复发规则: after every Tauri build, diff `app/Cargo.toml` and schema paths
  before continuing gates.
- 当前验证状态: active; three schema files were restored only after explicit
  controller approval; the latest rebuild redirtied the same three files and
  they were backed up under
  `target/forensic-schema-restore/20260508-181643` before restoring only those
  three paths.
- 相关文件 / artifact 类型: `app/gen/schemas/*.json`, `app/Cargo.toml`,
  forensic target directory.
- 是否仍有风险: medium; must recheck after each rebuild.

### `HOPE-UIROW-029`: paginated visible rows under-read

- 错题编号: `HOPE-UIROW-029`
- 问题现象: targeted D had six generated UI rows of 10 seconds each, but the
  runner read only the current visible table page, saw two rows, and reported
  `visible_rows_duration_missing` with a visible sum of 20 instead of 60.
- 错误做法: passing from backend `ui_rows` alone, or treating the first visible
  page as all user-visible rows.
- 根因: the UI table is paginated; the runner's visible evidence collector did
  not expand page size before calculating scene/duration evidence.
- 正确处理: before visible row evidence is calculated, set the storyboard page
  size control to the maximum available page size, wait for rows to render, then
  compute visible duration and scene evidence from the expanded table.
- 必带证据: `table_pagination_evidence`, `tableRowCount`, visible row duration
  sum, expected duration, `rows_match`, non-empty rows, and StopOnly cleanup.
- 防复发规则: visible-table gates must verify the evidence surface covers all
  rows needed for the case; backend rows cannot replace user-visible proof.
- 当前验证状态: fresh-verified; targeted D v5 passed on release exe
  `C59F5DAEAEE71A2C08ADE5D2B0477044275FEF3A509A38E0FA6660FCD6402156` with
  `table_pagination_evidence.after_rows=6`, visible duration sum `60`, and
  paired StopOnly clean.
- 相关文件 / artifact 类型: `scripts/hope-ui-driven-trace-runner.mjs`, UI runner
  JSON.
- 是否仍有风险: low for targeted D; still rerun A/B/C/D as a latest-code group
  before claiming targeted group pass.

### `HOPE-FULL16-030`: provider fallback evidence hidden below nested schema

- 错题编号: `HOPE-FULL16-030`
- 问题现象: fresh full16/targeted case 1 produced a valid runner result,
  non-empty rows, primary model success, no hard gate failures, and clean
  StopOnly, but the higher-gate orchestrator marked the case failed because
  `fallback_used`, `local_candidate`, `rows_match`, and `row_diffs` were present
  only inside nested evidence objects instead of the compact runner top level.
- 错误做法: treating nested provider evidence as sufficient for higher gates, or
  weakening the full16 orchestrator check to ignore `fallback_used=false` and
  `local_candidate=false`.
- 根因: runner/certifier evidence schema was not fully flattened for the fields
  explicitly required by targeted/full16/formal403 handoff reports.
- 正确处理: promote sanitized `fallback_used` and `local_candidate` from
  `fallback_gate_evidence`, and `rows_match` / `row_diffs` / row counts from
  `validator_gate_evidence`, into the compact runner result while keeping the
  nested objects for detailed audit.
- 必带证据: full16 case artifact with `ok=true`, non-empty rows,
  `attempted_models`, `final_model`, `provider_failover_used=false`,
  `fallback_used=false`, `local_candidate=false`, `rows_match=true`,
  `row_diffs=[]`, non-empty row counts, `no_http_403=true`, and paired
  StopOnly clean.
- 防复发规则: higher-gate orchestrators must fail closed when required evidence
  fields are missing, then fix the producer schema rather than deleting the
  assertion.
- 当前验证状态: producer-schema fix applied to
  `scripts/hope-ui-driven-trace-runner.mjs`; fresh targeted A/B/C/D and full16
  reruns are required before this card can be marked closed.
- 相关文件 / artifact 类型: `scripts/hope-ui-driven-trace-runner.mjs`,
  full16 runner JSON, full16 summary JSON.
- 是否仍有风险: medium until full16 is rerun fresh with the promoted fields.

### `HOPE-BIND-031`: narrative body sentence facts over-required as storyboard atoms

- 错题编号: `HOPE-BIND-031`
- 问题现象: fresh full16 case 2 generated non-empty rows and preserved the core B
  alley pursuit facts, but `generate_storyboard` was marked `Blocked` because
  `must_keep_facts` contained whole narrative-body sentences and a tokenization
  fragment such as `阿青终`; the validator then reported `missing_source_facts`,
  followed by visible row scene/duration failures because blocked rows were not
  exposed in the table.
- 错误做法: either forcing every narrative sentence to appear verbatim in rows,
  or deleting `missing_source_facts` / visible row gates to pass.
- 根因: StoryFactFrame binding mixed atomic source facts with narrative-body
  prose beats; the equivalence layer only knew the canonical B source sentence,
  not B-source narrative-body sentence coverage.
- 正确处理: keep the hard gate, but map B-source narrative prose and short
  tokenization fragments to atomic storyboard coverage: characters, protection,
  reminder, pursuer, alley mouth, and pressure/approach must all be present in
  real row fields.
- 必带证据: Rust regression for B narrative-body facts, runner evidence with
  `validator_gate_passed=true`, `rows_match=true`, non-empty row counts,
  visible scene/duration evidence, and StopOnly clean.
- 防复发规则: StoryFactFrame must distinguish atomic facts from narrative body
  prose; rows prove story facts through structured equivalence, not by copying
  whole paragraphs.
- 当前验证状态: runtime fix and minimal Rust regression added; cargo and fresh
  full16 rerun are still required before closing.
- 相关文件 / artifact 类型: `app/src/runtime.rs`, full16 runner JSON,
  StoryFactFrame binding evidence.
- 是否仍有风险: medium until fresh full16 case 2 and the remaining 14 cases pass.

### `HOPE-BIND-032`: agency-loss narrative fact over-required as visible row text

- 错题编号: `HOPE-BIND-032`
- 问题现象: fresh full16 case 3 produced non-empty backend rows for the hot-blood
  battle / 30s path, but `generate_storyboard` was blocked before UI table
  materialization because one narrative beat about losing the remaining
  initiative was treated as a verbatim row requirement.
- 错误做法: either copying the whole narrative beat into row text just to appease
  `missing_source_facts`, or weakening `missing_source_facts`,
  `visible_rows_scene_missing`, and `visible_rows_duration_missing`.
- 根因: StoryFactFrame equivalence covered the original A ruin source sentence
  and several pressure beats, but did not classify the initiative-loss sentence
  as a narrative-level pressure/agency beat that can be proven by the current
  source profile's visible row atoms.
- 正确处理: keep `generate_storyboard` blocked on real binding failure, but map
  this narrative beat to structured row atoms for the current source profile:
  A ruin rows require protagonist, enemy, ruin/duel space, approach or
  compressed distance, and a visible hold/brace action; B alley rows require
  Lin Feng, Su Yao, A Qing, pursuer, alley mouth, and approach pressure.
- 必带证据: Rust regression for the agency-loss beat, runner evidence with
  `validator_gate_passed=true`, visible rows non-empty, scene/duration binding
  present, `rows_match=true`, and paired `StopOnly` clean.
- 防复发规则: narrative body beats are not storyboard copy text. Each new
  narrative equivalence must name the atomic visible row facts it requires and
  must not allow prompt-only coverage. This is now promoted into
  `docs/hope-story-fact-frame-storyboard-binding-contract.md` and
  `docs/hope-qa-evidence-freshness-contract.md` as the repeated blocker
  promotion boundary.
- 当前验证状态: runtime fix and minimal Rust regressions added for A ruin and B
  alley variants; cargo, rebuild, AppDefault, targeted refresh, and full16 fresh
  rerun are still required before closing.
- 相关文件 / artifact 类型: `app/src/runtime.rs`, full16 runner JSON,
  StoryFactFrame binding evidence.
- 是否仍有风险: medium until fresh full16 reaches 16/16 on the latest release
  exe.

### `HOPE-CONTRACT-033`: recurring blocker fixed locally without prevention boundary

- Error card: `HOPE-CONTRACT-033`
- Problem symptom: the same failure class reappears under a new case, scene
  type, duration, source profile, evidence layer, or shell lifecycle after a
  previous local patch made one case pass.
- Wrong fix path: continue the next gate, promise to document later, add a loose
  word allowlist, weaken a runner/certifier assertion, broaden provider
  failover, or reuse stale artifacts to preserve momentum.
- Root cause: the repair loop treated each failure as an isolated local defect
  instead of promoting the repeated class into a contract boundary, regression
  test, evidence schema, and ledger entry.
- Correct fix path: classify the live failure, name the wrong path, write the
  relevant prevention boundary, add the smallest Rust test or runner/certifier
  assertion, update this ledger immediately, then repair implementation without
  weakening the hard gate.
- Required evidence: contract section, regression/assertion name, sanitized
  blocker artifact, fresh release/AppDefault/targeted/full16 rerun scope when
  code or gate producers changed.
- Regression or assertion: recurring blocker closure must fail closed when the
  required contract boundary, regression/assertion, or fresh artifact identity
  is missing.
- Prevention rule: a recurring blocker is not closed by a single case pass. It
  is closed only when the same mistake cannot pass through a different case or
  layer without being caught.
- Current validation status: active; `docs/hope-qa-evidence-freshness-contract.md`
  now contains `Recurring Blocker Closure Boundary`.
- Related files / artifact types:
  `docs/hope-qa-evidence-freshness-contract.md`,
  `docs/error-ledger/project-progress-error-ledger.md`, runner JSON, certifier
  JSON, release launch JSON.
- Remaining risk: medium until all active blocker classes have matching
  executable assertions and the latest-code targeted/full16 reruns prove the
  boundary in practice.

### `HOPE-CONTRACT-034`: three field-aware incidents repaired as isolated words

- Error card: `HOPE-CONTRACT-034`
- Three incidents:
  1. `危局` in `shot_title` / `visual_description` was at risk of becoming an
     external person while `person=危局` must still hard fail.
  2. `林峰压` / source-name action fragments were treated as possible names
     instead of field-aware action fragments.
  3. Fresh full16 case `expand_scene_same_duration_same_text_different`
     blocked on `shot_title` candidate `苏瑶并`, producing empty rows with
     `rows_match=true` and `validator_pseudo_success_detected`.
  4. Fresh full16 case `expand_scene_same_duration_same_text_same` blocked on
     `shot_title` candidate `临界`, another title/situation term promoted into
     a person candidate.
- Why same boundary: all three are non-`person` fields promoting situation
  words, source-prefixed action fragments, or conjunction tails into person
  candidates because field semantics were not decisive enough.
- Wrong fix path: add each word to a broad global allowlist, weaken
  `missing_source_facts`, delete `validator_pseudo_success_detected`, or treat
  `rows_match=true` with empty rows as pass.
- Root cause: `FieldAwareEntityResolver` did not fully classify
  source-prefixed action/conjunction fragments before the generic
  unknown-name candidate path.
- Correct boundary: `person` remains strict; `shot_title`, `scene_title`,
  `visual_description`, and `character_action` use field role plus source
  prefix plus bounded tail classification. The prefix must be a current source
  character or accepted role; the tail must be a bounded action, state, grammar,
  or conjunction fragment.
- Implementation integration point: `app/src/runtime.rs`
  `resolve_field_aware_entity_candidate`,
  `field_aware_source_action_fragment`, and validator tests.
- Runner/certifier acceptance fields: non-empty response/UI rows,
  `rows_match=true`, `validator_gate_passed=true`,
  `validator_pseudo_success_detected=false`, prompt boundary pass, visible-frame
  pass, and fresh artifact identity.
- Counterexample that must still fail: `person=危局`, `person=断戟立`, and a
  true source-external person such as `李明` when absent from the current source
  or accepted narrative frame.
- Current validation status: active; contract boundary added to
  `docs/hope-qa-evidence-freshness-contract.md` and
  `docs/hope-field-aware-story-contract.md`; runtime regression and fresh rerun
  are required before closure.
- Related files / artifact types: `app/src/runtime.rs`,
  `docs/hope-field-aware-story-contract.md`,
  `docs/hope-qa-evidence-freshness-contract.md`, full16 runner JSON.
- Remaining risk: medium until fresh full16 no longer reproduces empty-row
  pseudo-success from this field-aware class.

### `HOPE-RUNNER-035`: launch JSON truncated by Node child-process buffer

- Error card: `HOPE-RUNNER-035`
- Problem symptom: fresh targeted A stopped at `launch_failed_not_run`; the
  case launch and StopOnly artifacts were `{}`, while the same release shell
  command run directly returned valid AppDefault/CDP JSON for the current exe.
- Wrong fix path: rerun the same gate, treat it as a WebView2/CDP failure,
  weaken launch evidence, or continue to B/C/D with an empty launch artifact.
- Root cause: the targeted/full16 orchestration path used Node `spawnSync`
  without an explicit `maxBuffer`, so a large sanitized launcher JSON could be
  truncated before parsing and collapse into `{}`. In the sandboxed Codex
  process, the same path can also fail with `spawnSync powershell.exe EPERM`
  unless the gate is rerun with controlled escalation.
- Correct fix path: keep launch evidence required, increase the child-process
  buffer for release-shell and runner subprocesses, and preserve process
  status/error fields when JSON parsing fails or when the child process emits no
  stdout. If the recorded process error is sandbox `EPERM`, rerun the same
  orchestrator under controlled escalation instead of changing the gate.
- Required evidence: direct same-argument release shell pass, StopOnly clean,
  orchestrator rerun with non-empty launch/StopOnly JSON, and unchanged hard
  checks for `cdp_ready`, target URL, proxy clearing, no fallback, and non-empty
  rows.
- Prevention rule: an empty launch artifact is not a content failure and is not
  allowed to advance gates; the evidence producer must carry enough buffered
  output or fail with explicit process status.
- Current validation status: active; targeted/full16 temporary orchestrators and
  the formal403 runner now set an explicit child-process `maxBuffer`. Fresh
  targeted and full16 reruns are required before closing.
- Related files / artifact types: `target/qa-targeted-fresh-*` orchestrator,
  `target/qa-full16-fresh-*` orchestrator,
  `tests/qa/desktop-formal403-runner.mjs`, release launch JSON, StopOnly JSON.
- Remaining risk: low after fresh targeted proves non-empty launch/StopOnly
  artifacts; keep watching formal403 because it emits the largest evidence set.

### `HOPE-BIND-036`: A-ruin decision pressure prose required as row text

- Error card: `HOPE-BIND-036`
- Problem symptom: fresh full16 case `expand_scene_different_duration_same_text_same`
  produced real non-empty rows for the A ruin source under the national-war
  scene, but `missing_source_facts` still blocked because a narrative sentence
  about ruin pressure, front-line standoff, and judging the next move was
  treated as text that rows must reproduce.
- Wrong fix path: copy the whole narrative sentence into `visual_description`,
  count `prompt_text` as source coverage, remove `missing_source_facts`, or
  accept `rows_match=true` while the validator remains blocked.
- Root cause: the StoryFactFrame equivalence layer covered A-ruin pressure and
  agency-loss beats, but not the decision-pressure beat where visible row atoms
  should prove the same fact through protagonist, enemy, ruin/duel space,
  approach pressure, and a visible brace/look/decision response.
- Correct fix path: add a source-profile equivalence rule for A-ruin decision
  pressure, require visible row atoms, and add a Rust regression proving the
  full prose sentence is not required verbatim.
- Required evidence: Rust regression
  `binding_gate_maps_a_ruin_decision_pressure_narrative_to_visible_atoms`,
  fresh full16 case rerun with `validator_gate_passed=true`, non-empty UI and
  response rows, visible scene/duration evidence, prompt boundary pass, and
  StopOnly clean.
- Prevention rule: narrative beats about motive, pressure, decision, agency, or
  resolution must be mapped by source-profile equivalence; rows prove visible
  atoms, not prose sentences or prompt-only fields.
- Current validation status: active; runtime and contract boundary updated,
  cargo/rebuild/AppDefault/targeted/full16 rerun are still required.
- Related files / artifact types: `app/src/runtime.rs`,
  `docs/hope-story-fact-frame-storyboard-binding-contract.md`,
  full16 runner JSON, StoryFactFrame binding evidence.
- Remaining risk: medium until the latest release exe reaches fresh full16
  16/16 without new StoryFactFrame narrative-beat misses.

### `HOPE-PROV-037`: prompt-like expand output fell to local candidate

- Error card: `HOPE-PROV-037`
- Problem symptom: fresh full16 case `rewrite_scene_same_duration_same_text_same`
  passed CDP, rows, prompt boundary, and provider HTTP 200, but failed
  `live_fallback_detected` because the pre-rewrite expand step emitted
  `text_model_live_expand_fallback` after a prompt/storyboard-like provider
  response failed story-body validation.
- Wrong fix path: treat `rows_match=true` as sufficient, hide the warning,
  switch models on validator failure, or allow a deterministic local candidate
  to count as live provider success.
- Root cause: `repair_live_expanded_script_text` returned `None` before
  source-bound canonical repair whenever live text looked like prompt or
  storyboard packaging, so the caller fell back to local candidate text.
- Correct fix path: keep validator failure distinct from provider failover;
  discard prompt-like provider text only into source-bound narrative repair with
  `_repaired` evidence and `fallback_used=false`, `local_candidate=false`,
  `no_live_fallback=true`. If no source-bound repair exists, block.
- Required evidence: Rust regression
  `live_expand_repair_converts_prompt_like_output_without_local_fallback`,
  fresh targeted pass, fresh full16 rerun with no `text_model_live_expand_fallback`
  signal, provider attempted model evidence, and StopOnly clean.
- Prevention rule: prompt-like live output is not accepted as story body and is
  not local fallback. It may only become a sanitized source-bound narrative
  repair; model failover is still restricted to provider availability failures.
- Current validation status: active; runtime and provider contract updated,
  cargo/rebuild/AppDefault/targeted/full16 rerun are required before closure.
- Related files / artifact types: `app/src/runtime.rs`,
  `docs/hope-provider-failover-contract.md`, full16 runner JSON,
  provider/fallback evidence.
- Remaining risk: medium until full16 reaches 16/16 with no local fallback
  signals on expand or storyboard stages.
