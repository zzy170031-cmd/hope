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

## Standby Thread Naming Rule

When a Hope / Codex coordination thread is in standby, waiting, watch-only, or
blocked-by-upstream state, prefix the visible thread name with `（待命）`.

Rules:

- Use the prefix only for threads that should not actively implement right now.
- Remove the prefix when the controller explicitly reactivates the thread.
- Keep the rest of the thread name in the normal Chinese target-action shape.
- Examples:
  - Active: `Hope主线-【镜头强绑定与场景自动适配】`
  - Standby: `（待命）Hope主线-【镜头强绑定与场景自动适配】`
  - Active: `Hope桌面端-【等待主线镜头适配契约】`
  - Standby: `（待命）Hope桌面端-【等待主线镜头适配契约】`
