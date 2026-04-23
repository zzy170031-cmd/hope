# Desktop Single-Page Workbench Dispatch 2026-04-23

## Control Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- current control anchor: `dff9163`
  (`docs: dispatch desktop internal trial and UI alignment`)
- accepted desktop packaging anchor: `58c0d40`
  (`desktop-shell: fix MSI packaging acceptance`)
- current local desktop checkpoint under review: `codex/desktop-shell @ f359f99`
  (`desktop-shell: align internal trial UI`)
- intake anchor: `d74bd11`
- KB anchor: `513c681`
- V3 anchor: `origin/codex/v3-field-overlay-proposal @ bc745a4`

The current Hope route remains `RC_READY + scope freeze`.

This dispatch **supersedes the generic desktop UI alignment wording** from
`desktop-internal-trial-ui-alignment-dispatch-2026-04-23.md` with a more
precise product target.

The local desktop feedback is accepted as a valid intermediate conclusion:

- packaging / install / launch are already working
- the current desktop shell still reads like a staged workbench
- the current multi-route structure still exposes implementation layers
- the next step must be a product-shaped desktop workbench, not another round
  of generic skeleton cleanup

## Product Definition For This Step

Build **Hope Desktop Internal Trial Workbench v1** as a **single-page Chinese
workbench**, not as a multi-route engineering shell.

This step is about:

1. reshaping the desktop UI into the approved product workflow
2. making the workbench interaction real and coherent inside the frozen
   desktop boundary
3. keeping KB / Qwen / Seedance / V108 runtime growth closed

This step is **not** proof that the full Hope generation engine is already
connected.

## Reference Handling Rule

The user-provided reference files and UI mock must be used like this:

- `UI界面参考.png`:
  use as the primary layout and interaction rhythm reference
- `操作参考.md` and `操作执行.md`:
  use only as workflow inspiration

They must **not** become:

- a new backend architecture
- a Node / Express / OpenAI direct-call implementation
- a generic anime-director prompt chain
- a replacement for Hope KB, Hope contracts, or Hope runtime boundaries

## Exact Desktop Target

The desktop shell must converge to this single-page structure:

1. top product bar
2. `脚本区`
3. task strip
4. `分镜产出区`
5. bottom-right export action

### Top Product Bar

Must contain:

- Hope product name
- model selector
- `API 文档`
- `API 接口`

The model selector is a UI/session selector only in this step. It must not
claim real model switching.

### Script Area

The first work area must be named:

```text
脚本区
```

It must support:

- fused scene type selection
- story synopsis input
- `扩写脚本`
- showing the current expanded-script working result

Current truth constraint:

- the UI may present the script expansion as a Hope workbench result shaped by
  KB-facing constraints
- but it must not pretend that KB-backed live generation is already connected

### Storyboard Output Area

The second work area must be named:

```text
分镜产出区
```

It must support:

- task name input
- `新建任务`
- `导入扩写剧本`
- duration selection
- `开始生成`
- table results
- 5 rows per page
- row edit / duplicate / delete
- pagination

The table should use Hope-facing business fields:

- 序号
- 人物
- 镜头
- 景别
- 画面描述
- 角色动作
- 对白 / 旁白
- 分镜提示词
- 时长(秒)
- 操作

### Export

The workbench must keep a clear bottom-right export action.

Allowed now:

- real desktop export from the current front-end working state
- CSV first, or `.xlsx` if done entirely in the front-end layer

Not allowed now:

- claiming full product-ready exporter integration

## Required Interaction Truth

This step must distinguish three layers clearly:

1. KB / readonly readiness signals
2. writer snapshot / preview snapshot as current packaged sources
3. front-end working state as the editable task / table / export source

The workbench may be fully interactive, but it must stay honest:

- real interaction: yes
- real runtime generation chain: no

## Mandatory UI Changes

The following must happen in this step:

- remove obvious `Desktop MVP`, `Internal Trial`, `UI Skeleton`, `Track B`,
  and similar stage-language from the primary product surface
- stop exposing implementation-layer route labels as the main user workflow
- stop centering bridge status / readonly technical wording in the top area
- stop presenting the storyboard table as “structure only” if the new table is
  the user-facing working result area
- remove the explanatory green-box placeholder wording from the reference
  layout and replace it with real desktop UI copy

## Allowed File Scope

```text
E:\codex\hope-desktop-shell\ui\src\App.tsx
E:\codex\hope-desktop-shell\ui\src\routes.ts
E:\codex\hope-desktop-shell\ui\src\styles.css
E:\codex\hope-desktop-shell\ui\src\components\**
E:\codex\hope-desktop-shell\ui\src\types.ts
E:\codex\hope-desktop-shell\ui\README.md
E:\codex\hope-desktop-shell\docs\**
```

If front-end-only export requires a small UI dependency, keep it bounded to the
desktop UI package and document it clearly.

## Forbidden

- no `app/src/runtime.rs` expansion
- no new IPC commands
- no bridge contract widening
- no `crates/**` changes
- no KB / intake / V3 / `E:\codex\hope` code edits
- no live Qwen
- no Seedance
- no V108 runtime import
- no product-ready external references
- no `reference_control_core`
- no pretending that KB live retrieval or Seedance timing logic is already
  connected

## Thread Instruction

```text
线程名：Hope桌面端-SinglePageWorkbenchRebuild【单页工作台重构中】
```

Target:

```text
E:\codex\hope-desktop-shell / codex/desktop-shell
```

Task:

- use `f359f99` only as an intermediate cleanup checkpoint
- rebuild the desktop UI into the approved single-page Chinese workbench
- keep all work desktop-only
- keep interaction real, but keep capability claims honest

Expected report back:

- internal trial issues carried forward
- pages/modules replaced by the new single-page workbench
- changed files
- build result
- export result
- final branch / commit
- whether the new UI now matches the approved Hope desktop workbench direction

## Exact Next Step

1. do not continue polishing the old multi-route shell as the main product
2. rebuild it into the approved single-page Chinese workbench
3. keep V3 paused for user-owned content optimization
4. keep KB and intake on standby
