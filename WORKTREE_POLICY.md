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
- at every key node, maintain one canonical thread label in the format `线程名【状态标签】`
- use the shared policy in `E:\codex\ENGINEERING_THREAD_LABEL_POLICY.md`
- labels must be updated from the actual local worktree state, not only from the remote branch state
- before handoff or commit decisions, record the anchor commit, clean / dirty state, and one-line scope boundary behind the current label
- every key-node handoff must end with `关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。`
- if a package is still being narrowed for a clean commit, prefer a package-specific label such as `P2包1收口` instead of a vague generic label
- if a bounded package has already reached a remote stable point, prefer a pushed-state label such as `包1已推送` or `包2已推送`
- if a thread has completed a bounded local package but must wait for another thread, mark it as `完成待命` instead of continuing to expand scope
