# Hope

Hope is a desktop-first AI manga / animation prompt-package workbench.

Current v0.1 focus:

- all-Chinese pipeline
- synopsis -> story -> screenplay -> storyboard
- continuous panel prompt packages
- Excel as the main delivery artifact
- no in-product image generation
- no in-product video generation

## Workspace shape

- `app/`: Tauri shell and desktop integration
- `ui/`: React + TypeScript UI
- `crates/core-domain/`: shared domain contracts
- `crates/project-store/`: SQLite project store
- `crates/writer-pipeline/`: Writer Layer 0-3 generation
- `crates/storyboard-pipeline/`: committee runtime, cuts, segments, prompt rendering
- `crates/validators/`: export and consistency gates
- `crates/export-engine/`: Excel / JSON / Markdown export
- `contracts/`: contract freeze artifacts for Day 1-3
- `docs/`: product and execution docs

## Primary docs

- `docs/hope-final-plan.md`
- `docs/parallel-execution-appendix.md`
- `docs/day-1-3-contract-freeze-checklist.md`
- `docs/project-thread-startup.md`
- `WORKTREE_POLICY.md`
- `CONTRACTS.md`
- `AGENTS.md`

## Git and multi-device notes

- This repo is intended to be the main product repo.
- A separate companion repo `hope-kb` holds system knowledge assets.
- Use worktrees for parallel implementation branches instead of cloning multiple copies on one machine.
- Keep the main thread focused on integration and milestone governance.
