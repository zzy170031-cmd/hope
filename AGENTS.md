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

## Main references

- `docs/hope-final-plan.md`
- `docs/parallel-execution-appendix.md`
- `docs/day-1-3-contract-freeze-checklist.md`
- `WORKTREE_POLICY.md`
- `DONE_CRITERIA.md`
