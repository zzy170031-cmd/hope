# Codex 使用核心规则

本文件是 Hope 五代理模式下的 Codex 使用核心规则。换机或开启新线程时，
优先把本文件作为跨线程沟通、总控指令、线程回报、接力和归档的格式准则。

## Copy-Ready 线程沟通格式

以后所有跨线程沟通、总控指令、线程回报、归档包、接力包、验收清单，
以及任何需要复制到另一个 Codex 线程的内容，默认使用 Markdown `text`
代码块输出。

格式要求：

1. 正文外部只保留一句极短说明。
2. 主要内容必须放进 copy-ready 代码块。
3. 给其他线程的指令必须可直接粘贴执行。

````text
```text
目标线程：

仓库路径：

branch / HEAD：

当前状态：

本轮目标：

明确边界：

允许：
- 

禁止：
- 

操作步骤：
1. 
2. 

验证命令：

回报格式：
```
````

## 指令字段

给其他线程的指令优先包含：

- 目标线程
- 仓库路径
- branch / HEAD
- 当前状态
- 本轮目标
- 明确边界
- 允许做什么
- 禁止做什么
- 操作步骤
- 验证命令
- 回报格式

命令必须逐行列出。契约字段必须逐行列出。

## 回报字段

线程回报优先包含：

- 线程名
- 仓库路径
- branch / HEAD
- git status
- 已完成
- 未完成
- 验证结果
- dirty 文件
- 风险 / 阻塞
- 需要总控决策的问题
- 下一步建议

## 安全边界

禁止在聊天、线程回报、导出或日志中写入：

- API key
- token
- 密钥
- full raw KB rows
- raw `prompt_body`
- `source_register`
- overlay JSON

如果指令与实际 Git / 文件状态冲突，以实际状态为准，先回报冲突，不要盲目执行。

不要回滚用户或其他线程改动，除非用户明确要求。

## Separate Controller Review And Thread Instructions

When a controller response contains both:

- controller review / recap / decision record
- instructions for a specific worker thread

the two parts must be split into separate Markdown `text` code blocks.

Do not put controller recap and worker-thread instructions in the same code
block.

Use this shape:

```text
Controller recap:

...
```

```text
Instructions for <target thread>:

...
```

The worker-thread instruction block must be copy-ready on its own. It should not
depend on surrounding controller recap text to be actionable.

## Proactive Controller Next-Step Dispatch

When a worker thread reports back to the controller, the controller must verify
the report against actual Git / file state when possible. After verification,
the controller should proactively provide the corresponding next step based on
the full current plan.

Do not wait for the user to ask "next step" when the next action is clear.

If other parallel threads can safely continue or synchronize at the same time,
the controller should also issue separate copy-ready instruction blocks for
those threads.

Default controller response shape after a worker-thread report:

```text
Controller recap:

- verified facts
- decision
- next overall gate
```

```text
Instructions for <reporting thread>:

- continue / archive / standby / fix / commit / verify
```

```text
Instructions for <parallel thread if applicable>:

- safe parallel action
- scope boundary
- report format
```

Only omit thread instruction blocks when there is genuinely no actionable next
step or when the next step is blocked by a user decision.

## Thread Naming Rule

Hope / Codex coordination threads should use the visible naming shape:

`<thread or module name>-【<Chinese target action>】`

Rules:

- Do not include English contract identifiers, implementation codenames, or
  camel-case feature names in the visible thread name.
- Keep the visible name focused on the target action in Chinese.
- If an English identifier is useful for engineering traceability, put it inside
  the instruction body, not in the thread title.
- Use `Hope契约层-【镜头强绑定与自适应场景契约】` instead of
  `Hope契约层-StoryboardShotGroundingContract【镜头强绑定与自适应场景契约】`.
- Use `Hope桌面端-【等待主线镜头适配契约】` instead of
  `Hope桌面端-ShotIntentAdaptiveUI【等待主线镜头适配契约】`.

### Workstream-Aware Thread Naming

When one project has multiple parallel work surfaces, the visible thread name
must include the workstream / thread role, not only the repo or product name.

Use this shape:

`（status）Project-WorkstreamThread-【Chinese target action】`

Rules:

- The project / repo part identifies ownership, such as `Hope桌面端`,
  `Hope主线`, `Hope契约层`, `Hope接入点`, or `Hope知识库`.
- The workstream part identifies the active lane or role, such as `图标线程`,
  `UI线程`, `功能线程`, `打包线程`, `发布线程`, `契约线程`, or `QA线程`.
- The target action describes the current concrete goal in Chinese.
- Do not collapse distinct workstreams into a generic name such as
  `（运行）Hope桌面端-【多尺寸帧修复】` when desktop UI, icon, packaging,
  and feature work may be running separately.
- If a thread's workstream changes, rename it explicitly in the next controller
  directive.

Examples:

- Use: `（运行）Hope桌面端-图标线程-【多尺寸帧修复】`
- Use: `（运行）Hope桌面端-UI线程-【主界面收口】`
- Use: `（运行）Hope桌面端-功能线程-【已定稿分镜集合接入】`
- Use: `（运行）Hope桌面端-打包线程-【内置WebView2离线安装器】`
- Use: `（待命）Hope桌面端-发布线程-【v1自动更新与版本发布体系规划】`
- Avoid: `（运行）Hope桌面端-【图标多尺寸帧修复】`

## Controller / Worker Directive Label Rule

All cross-thread copy-ready `text` blocks must declare their role and target.

When the controller sends instructions to a worker thread, the block must start
with:

```text
分线程指令：
目标线程：<target worker thread>
```

When a worker thread reports back to the controller, the block must start with:

```text
总控回报：
目标线程：总控线程
来源线程：<reporting worker thread>
```

Rules:

- Controller recaps remain separate and use `总控复盘：`.
- Worker instructions use `分线程指令：`, not generic labels such as
  `给某线程的指令`.
- Worker reports use `总控回报：`, not generic labels such as `回报` alone.
- The `目标线程：` line is mandatory in both instruction and report blocks.
- If a controller response contains both recap and worker instructions, keep
  them in separate `text` blocks.

## Thread Status Prefix Rule

Hope / Codex coordination thread names must use a status prefix when the
controller is assigning or summarizing thread state.

Allowed status prefixes:

- `（运行）`: the thread should actively execute the current task now.
- `（待命）`: the thread should wait, watch, or stay blocked by an upstream gate.
- `（结束）`: the thread's current task is complete, but the thread is not being archived yet.
- `（归档）`: the thread should prepare to archive or can be archived.

Rules:

- Put the status prefix at the very beginning of the visible thread name.
- Keep the rest of the thread name in the normal Chinese target-action shape.
- Change the prefix when the controller changes the next action.
- Examples:
  - `（运行）Hope主线-【镜头强绑定与场景自动适配】`
  - `（待命）Hope桌面端-【等待主线镜头适配契约】`
  - `（结束）Hope契约层-【镜头强绑定与自适应场景契约】`
  - `（归档）Hope接入点-ContractSmokeReadiness【待命】`

## Action-Only Thread Dispatch Rule

When the controller replies to the user with thread instructions, include only
threads that need an immediate action, such as `（运行）` or `（归档）` threads.

Rules:

- Omit `（待命）` threads unless their standby state changed or the user asks for
  a full status review.
- Omit `（结束）` threads unless they need reactivation, archival, or a final
  user-facing note.
- Do not repeat no-op standby instructions just to be exhaustive.
- A short controller recap may mention omitted standby/end threads in one line
  if it helps explain the overall plan.

## 中文沟通优先规则

Hope / Codex 协调线程默认使用中文沟通。

规则：

- 总控复盘、分线程指令、总控回报、过程更新、归档说明默认使用中文。
- 代码标识、字段名、命令、commit message、错误原文可以保留英文。
- 如果引用英文报错或英文文档，必须用中文解释结论和下一步。
- 除非用户明确要求英文，否则不要用英文写长段过程说明。
- 如果某个分线程开始用英文汇报，总控应提醒该线程后续改回中文。

## 总控跨线程联动检查规则

每次总控新开、继续、审查或接收核心线程 / 主线线程 / 关键功能线程回报时，必须先判断是否影响总控下辖的其他对应工作线程，再下发下一步。

判断原则：

- 不要把联动线程固定写死为某几个名字。
- 按当前项目真实工作面判断哪些线程受影响。
- 常见受影响线程可能包括但不限于：契约线程、功能线程、UI线程、验收线程、接入线程、知识库线程、QA线程、打包线程、发布线程、图标线程。
- 只有实际受影响的线程才需要下发指令。
- 不受影响的线程可以在总控复盘里简短说明原因，不要强行派发待命指令。

联动判断示例：

- 改 request / response 字段、runtime surface、持久化结构、导出结构、warning、validator、prompt 边界时，通常需要契约或下游消费线程同步。
- 改 shell 可见类型、命令面、export bundle 输入输出、preview / draft mirror、runtime metadata 时，通常需要接入或桌面消费线程同步。
- 改 taxonomy、routing category、shot_intent 标签、KB 选择规则、seed / snapshot 预期时，通常需要知识库或 QA 线程同步。
- 改 UI 展示、安装包、图标、发布、自动更新时，只联动对应 UI / 打包 / 图标 / 发布线程，不要错误扩大到契约或接入线程。

规则：

- 在总控复盘或核心线程指令里写明联动判断。
- 如果某个工作线程不需要动作，要简短说明原因。
- 如果某个工作面需要动作，必须写出对应工作线程内容：
  - 线程动作：新开 / 沿用 / 待命 / 归档。
  - 目标线程名。
  - 负责问题。
  - 启动条件或依赖。
  - 允许修改范围。
  - 禁止事项。
  - 回报格式。
- 只下发真正需要执行的线程；不要重复刷待命线程。
- 如果需要新增字段或契约边界，先开对应契约线程，再允许实现线程继续。
- 如果只是提升内部质量，且不改变既有契约 / 下游字段 / 展示边界，可以让其他线程继续待命。
- 不允许任何线程静默扩展成其他工作面的任务，除非总控明确放行。

## 全项目规则同步规则

当 Codex 全局规则、总控协作规则、线程命名规则、回报格式规则、主线联动规则发生变化时，总控必须考虑同步到所有 Hope 相关项目。

需要覆盖的项目：

- `E:\codex\hope`
- `E:\codex\hope-desktop-shell`
- `E:\codex\hope-kb`
- `E:\codex\hope-intake-app`

同步规则：

- 先做只读 Git 核验，再决定是否写入。
- 不要直接修改正在 dirty 的 `AGENTS.md`，除非用户明确要求并确认归属。
- 优先同步到项目内的规则文档，例如 `docs/codex-usage-core-rules.md`。
- 如果项目没有规则文档，可以新增 docs-only 规则文件，但必须单独提交，不能混入 runtime / UI / KB / fixture 改动。
- 如果项目正有运行线程或未知 dirty，先记录“待同步”，不要强行写入或提交。
- 同步完成后必须回报每个项目的状态：已同步 / 待同步 / 跳过原因。
- 如果某个项目只需要引用总规则而不需要复制全文，也要写明引用路径和更新策略。

禁止：

- 不要把全局规则同步和业务实现混提交。
- 不要回滚其他线程或用户改动来获得 clean 状态。
- 不要把未确认归属的 `AGENTS.md` / `WORKTREE_POLICY.md` 改动一起提交。
- 不要在同步规则时改 runtime、UI、KB seed、fixture、导出产物或安装包配置。
