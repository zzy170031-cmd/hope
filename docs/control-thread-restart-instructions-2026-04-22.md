# Control Thread Restart Instructions 2026-04-22

## Purpose

This memo records the restart package for moving Hope coordination to another
computer. It is docs-only and does not open product implementation.

The source of truth is Git. Do not rely on old chat scrollback after restart.

## Push Verification

All active lines were pushed or verified up to date before this memo:

- `E:\codex\hope`
  - branch: `codex/contracts-freeze`
  - pushed anchor before this memo: `5fc15fe`
  - latest decision: validator planning accepted; KB snapshot gate dispatched
- `E:\codex\hope`
  - branch: `codex/v3-field-overlay-proposal`
  - pushed anchor: `69c645c`
  - latest state: V3 golden-sample freeze candidate archived / standby
- `E:\codex\hope-kb`
  - branch: `codex/contracts-freeze`
  - pushed anchor: `818c098`
  - latest state: v0.2 golden-sample snapshot-import readiness complete;
    awaiting main-control review
- `E:\codex\hope-desktop-shell`
  - branch: `codex/desktop-shell`
  - pushed anchor: `66d687d`
  - latest state: WriterReadiness / InputBoundary complete and waiting
- `E:\codex\hope-intake-app`
  - branch: `codex/controlled-intake-snapshot-bootstrap-app`
  - pushed anchor: `d74bd11`
  - latest state: Qwen/storyboard contract boundary complete and waiting

## Current Route

- Hope remains `RC_READY` plus controlled v0.2 readiness planning.
- `hope-kb` remains separate from `hope`.
- V3/V4 fields remain planning / freeze-candidate input until a future explicit
  gate.
- Desktop and intake remain waiting.
- Qwen and Seedance remain closed.
- The user is preparing additional golden sample cases; treat new cases as a
  future KB-only intake/update gate, not as Hope product implementation.

## Immediate Next Control Action After Restart

1. Pull / fetch all repos.
2. Review `hope-kb` commit `818c098` and its readiness note:
   `E:\codex\hope-kb\docs\golden-sample-v0.2-snapshot-import-readiness-2026-04-22.md`.
3. Write a Hope main-control review accepting or rejecting the KB v0.2
   snapshot-import readiness package.
4. Do not open Hope validator implementation until the KB snapshot package has
   been accepted by main control.
5. If the user supplies additional golden sample cases first, open a new
   bounded KB-only update gate and keep Hope product work closed.

## Restart Prompt: Main Control

```text
本线程接替旧 Hope 总控线程。不要继承旧聊天上下文，以 Git 当前状态为唯一事实源。

先读取：
- E:\codex\hope 当前分支、git status、最新 control-thread-restart-instructions、最新 control-thread-handoff / checkpoint、nightly handoff、project-thread-startup、parallel-execution-appendix、control-thread-context-supervision
- E:\codex\hope-kb 当前分支、git status、最新 live-progress、nightly handoff、project-thread-startup、parallel-execution-appendix
- E:\codex\hope-desktop-shell 当前分支、git status、最新 commit
- E:\codex\hope-intake-app 当前分支、git status、最新 commit
- E:\codex\hope 的 codex/v3-field-overlay-proposal 分支最新 commit

当前预期路线：
- hope：RC_READY + controlled v0.2 readiness planning；主控只做复核、调度、边界判断、Git 可见 handoff
- hope-kb：独立 KB hardening / snapshot readiness；暂不合流到 hope
- V3/V4 字段：v0.2 freeze candidate 已完成，归档待命；未来进入主线必须等显式 gate
- desktop 和 intake：保持待命，除非主控明确打开 scope gate
- Qwen / Seedance：保持关闭

当前关键锚点：
- hope：codex/contracts-freeze @ 最新远端，至少包含 5fc15fe
- hope-kb：codex/contracts-freeze @ 818c098
- V3 proposal：codex/v3-field-overlay-proposal @ 69c645c
- desktop：codex/desktop-shell @ 66d687d
- intake：codex/controlled-intake-snapshot-bootstrap-app @ d74bd11

重启后的第一件事：
复核 hope-kb 的 818c098 v0.2 snapshot-import readiness，写 Hope 主控 acceptance / rejection。若用户先给新的黄金样本案例，则先开 KB-only 样本更新 gate，不要直接进入 Hope 产品实现。
```

## Restart Prompt: Hope Main Thread

```text
线程名：Hope主线-GoldenSampleValidatorPlanning【v0.2候选已推送·待主控复核】
仓库 / 分支：E:\codex\hope / codex/contracts-freeze
当前锚点：拉取远端最新；已知 validator candidate 为 b304c4f，主控 acceptance / KB dispatch 为 5fc15fe

重启后先执行：
- git status / git log 核对当前分支
- 读取 E:\codex\hope\docs\golden-sample-validator-contract-candidate-2026-04-22.md
- 读取 E:\codex\hope\docs\golden-sample-validator-contract-review-2026-04-22.md
- 读取 E:\codex\hope\docs\v0.2-snapshot-import-readiness-dispatch-2026-04-22.md

当前状态：
- validator planning 已完成并被主控接收
- 不继续写 validator implementation
- 等待主控复核 KB 818c098 snapshot-import readiness 后再决定下一 gate

禁止范围：
- 不改 Rust DTO / validator / exporter / workbook / IPC
- 不改 desktop / intake / hope-kb seed
- 不接 Qwen / Seedance
```

## Restart Prompt: KB Thread

```text
线程名：Hope-KB-V0.2SnapshotImportReadiness【SnapshotGate已推送·待主控复核】
线程ID：019da89f-49a2-7e11-bd2f-c0138165fdd9
仓库 / 分支：E:\codex\hope-kb / codex/contracts-freeze
当前锚点：818c098 Add golden sample v0.2 snapshot import readiness

重启后先执行：
- git status / git log 核对当前分支
- 读取 E:\codex\hope-kb\docs\live-progress.md
- 读取 E:\codex\hope-kb\docs\golden-sample-v0.2-snapshot-import-readiness-2026-04-22.md

当前状态：
- v0.2 import_map / manifest / migration / versioned snapshot builder 已推送
- v0.1 no-regression validation passed
- v0.1 default snapshot build passed via fallback output
- v0.2 explicit snapshot build passed，SQLite quick_check = ok
- 等待主控 acceptance

如果用户新增黄金样本案例：
- 等待主控打开新的 KB-only update gate
- 不自行合流到 Hope
- 不改 Hope / desktop / intake
- 不补造 reference_control_core
```

## Restart Prompt: V3 Field Thread

```text
线程名：Hope-V3字段线程-GoldenSampleContract【v0.2 Freeze候选已推送·归档待命】
线程ID：019db40b-4299-75c3-8ea7-d6b1ff3a8175
仓库 / 分支：E:\codex\hope / codex/v3-field-overlay-proposal
当前锚点：69c645c docs: promote golden sample overlay to freeze candidate

重启后先执行：
- git status / git log 核对当前分支
- 读取 E:\codex\hope\docs\v3-golden-sample-contract-freeze-review-2026-04-22.md

当前状态：
- V3 freeze candidate 已被主控接收并归档
- 不继续扩写 proposal
- 不修改 Hope / hope-kb / desktop / intake
- 等待未来主控 gate；新的黄金样本案例需先由 KB 接收和主控复核后，才可能触发 V3 更新
```

## Restart Prompt: Desktop Thread

```text
线程名：Hope桌面端-WriterReadiness【InputBoundary待命】
线程ID：019daa78-87fb-7380-a8be-c19a7bf168b7
仓库 / 分支：E:\codex\hope-desktop-shell / codex/desktop-shell
当前锚点：66d687d desktop-shell: add writer qwen input readiness boundary

重启后先执行：
- git status / git log 核对当前分支

当前状态：
- WriterReadiness / InputBoundary 包已完成
- 保持待命
- 不接 live Qwen
- 不新增 UI 包
- 不读取或展示 KB v0.2 golden sample 内部数据，除非主控后续打开 desktop read-only provenance gate
```

## Restart Prompt: Intake Thread

```text
线程名：Hope接入线程-Qwen分镜生成Contract【Contract边界待命】
线程ID：019daa9c-1955-7852-92ea-e9a2139acaef
仓库 / 分支：E:\codex\hope-intake-app / codex/controlled-intake-snapshot-bootstrap-app
当前锚点：d74bd11 qwen-storyboard: tighten contract boundary

重启后先执行：
- git status / git log 核对当前分支

当前状态：
- Qwen/storyboard contract boundary 已完成
- 保持待命
- 不接 live Qwen
- 不生成 prompt package
- 不做 segment/cut handoff
- 不做 runtime integration
- 等待未来 intake / Qwen retrieval boundary gate
```

