# Hope Agent Rules

This repo is built through Codex-only parallel execution.

## Core rules

- Use Codex / Codex subagents only.
- Do not use Claude for implementation tracks.
- Treat `contracts/` as the source of truth after Day 3 freeze.
- Do not change a contract without going through `CONTRACTS.md`.
- Keep product committee roles separate from engineering tracks.

## Track ownership

- Track A: schema, migrations, domain, store, IPC contracts
- Track B: UI and Tauri frontend calls
- Track C: `hope-kb` content and seed assets
- Track D: writer pipeline
- Track E: storyboard / committee runtime / handoff generation
- Track F: validators
- Track G: exporters
- Track H: integration, E2E, release gating

## Delivery discipline

Every track handoff must include:

1. change summary
2. contract dependency
3. smallest runnable example
4. risks / blockers
5. current canonical thread label
6. key-node reminder line when applicable

## Milestone status labels

At every key milestone, every active thread must publish and maintain a short canonical status label for observation and coordination.

- Use the display format `线程名【状态标签】`.
- Use the shared policy in `E:\codex\ENGINEERING_THREAD_LABEL_POLICY.md`.
- Update the label before starting the next package whenever a thread reaches a key node such as: freeze accepted, package completed, tests green, commit-ready, handoff-ready, or environment-blocked.
- The label must reflect the real local state, not only the remote Git HEAD.
- Keep one canonical label per active thread; do not allow multiple conflicting labels for the same thread at the same time.
- Every key-node handoff must end with:
  `关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。`
- Every labeled thread state should stay traceable to:
  1. repo path
  2. branch
  3. anchor commit
  4. clean / dirty worktree state
  5. one-line scope boundary

### Current canonical thread names and labels

- `Hope主线程-冻结监督【冻结待命】`
- `Hope-KB-检查点支持【检查点待命】`
- `Hope桌面端-Phase2状态源加固【包1已推送】`
- `Hope接入线程-SnapshotBootstrap【包2已推送】`

### Label intent

- `冻结待命`: docs-only supervision, no implementation
- `检查点待命`: keep the checkpoint stable and wait for bounded feedback
- `包1收口`: package is in scope-tightening / commit-ready hardening
- `包1完成待命`: package 1 is complete locally and waiting for the next explicit dependency or authorization
- `包1已推送`: package 1 has reached a remote stable point; keep the route bounded until a new package is explicitly authorized
- `包2已推送`: package 2 has reached a remote stable point; do not widen scope from the pushed anchor without a new bounded package

## Main references

- `docs/hope-final-plan.md`
- `docs/parallel-execution-appendix.md`
- `docs/day-1-3-contract-freeze-checklist.md`
- `WORKTREE_POLICY.md`
- `DONE_CRITERIA.md`
- `E:\codex\ENGINEERING_THREAD_LABEL_POLICY.md`
