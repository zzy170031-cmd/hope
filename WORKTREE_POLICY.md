# Worktree Policy

Use one main checkout and multiple worktrees.

## Main checkout

- Keep `E:\codex\hope` as the integration checkout.
- Do not use the main checkout for large feature work.

## Recommended branch layout

- `main`: stable integration branch
- `codex/contracts-freeze`
- `codex/track-a-core`
- `codex/track-b-ui`
- `codex/track-c-kb`
- `codex/track-d-writer`
- `codex/track-e-storyboard`
- `codex/track-f-validators`
- `codex/track-g-export`
- `codex/track-h-integration`

## Recommended local worktree layout

- `E:\codex\wt\hope-track-a-core`
- `E:\codex\wt\hope-track-b-ui`
- `E:\codex\wt\hope-track-c-kb`
- `E:\codex\wt\hope-track-d-writer`
- `E:\codex\wt\hope-track-e-storyboard`
- `E:\codex\wt\hope-track-f-validators`
- `E:\codex\wt\hope-track-g-export`
- `E:\codex\wt\hope-track-h-integration`

## Rules

- one branch, one main owner
- do not share the same branch across devices at the same time
- merge back through the integration branch only
- keep uncommitted changes out of the integration checkout
