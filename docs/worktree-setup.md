# Worktree Setup

Recommended pattern:

1. keep `E:\codex\hope` on `main`
2. create one worktree per active track branch
3. keep integration work in the main checkout or a dedicated integration worktree

## Example

```powershell
git -C E:\codex\hope worktree add E:\codex\wt\hope-track-a-core -b codex/track-a-core
git -C E:\codex\hope worktree add E:\codex\wt\hope-track-b-ui -b codex/track-b-ui
```

## Device sync rule

- one device should own one active worktree branch at a time
- push frequently
- do not leave contract edits unpushed
