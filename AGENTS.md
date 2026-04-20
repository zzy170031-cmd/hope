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
- Update the label before starting the next package whenever a key node is reached.
- The label must reflect the real local state, not only the remote Git HEAD.
- Every key-node handoff must end with:
  `关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。`

### Current canonical thread name and label

- `Hope接入线程-SnapshotBootstrap【包2已推送】`

### Label intent

- `包2已推送`: package 2 has reached a remote stable point; do not widen scope from the pushed anchor without a new bounded package

## Main references

- `docs/hope-final-plan.md`
- `docs/parallel-execution-appendix.md`
- `docs/day-1-3-contract-freeze-checklist.md`
- `WORKTREE_POLICY.md`
- `DONE_CRITERIA.md`
- `E:\codex\ENGINEERING_THREAD_LABEL_POLICY.md`
