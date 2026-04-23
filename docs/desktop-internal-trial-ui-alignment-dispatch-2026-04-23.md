# Desktop Internal Trial + UI Alignment Dispatch 2026-04-23

## Control Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- current control acceptance anchor: `aef49ec`
  (`docs: accept desktop MVP internal release`)
- current desktop anchor: `58c0d40`
  (`desktop-shell: fix MSI packaging acceptance`)
- current intake anchor: `d74bd11`
  (`qwen-storyboard: tighten contract boundary`)
- current KB anchor: `513c681`
  (`docs: propose V108 canonical object registry schema`)
- current V3 anchor: `origin/codex/v3-field-overlay-proposal @ bc745a4`

The current Hope route remains `RC_READY + scope freeze`.

This dispatch does three things only:

1. starts desktop internal trial use with the accepted MVP package
2. opens a bounded desktop UI alignment gate
3. keeps V3 paused for user-owned content optimization until a later written
   v0.2 gate

It does **not** reopen product scope, Qwen / Seedance integration, V108 row
import, external reference handles, or `reference_control_core`.

## Thread 1: Desktop Internal Trial

```text
Hope桌面端-DesktopMVPInternalTrial【内部试用推进中】
```

Target:

```text
E:\codex\hope-desktop-shell / codex/desktop-shell
```

Current accepted package basis:

```text
E:\codex\hope-desktop-shell\target\release\bundle\msi\Hope Desktop Shell_0.1.0_x64_en-US.msi
```

Task:

- use the accepted MSI for internal trial / internal distribution
- confirm install, launch, shortcut, and basic shell availability in normal use
- collect trial feedback as release notes / bug notes, not as scope expansion

Allowed:

- package reproduction
- install / uninstall / launch verification
- internal trial notes
- desktop-only bug fixes if they are regressions inside the current frozen
  package path

Forbidden:

- no new feature growth
- no Qwen live call
- no Seedance integration
- no V108 row import
- no product-ready external references
- no `reference_control_core`
- no KB / intake / V3 / Hope main edits

## Thread 2: Desktop UI Alignment

```text
Hope桌面端-DesktopUIAlignment【界面对齐调整中】
```

Target:

```text
E:\codex\hope-desktop-shell / codex/desktop-shell
```

Reason:

The current accepted desktop package is valid for internal MVP release, but the
UI is still a Track B / skeleton shell instead of the previously discussed final
detailed desktop workflow.

Task:

Start bounded desktop UI alignment work, but keep it strictly desktop-only.

Focus on:

- information architecture alignment
- product naming / labeling alignment
- layout and navigation alignment
- replacing obvious skeleton-stage wording in the UI
- making the workbench read like a real product shell instead of a phase label

Allowed file scope:

```text
ui/src/App.tsx
ui/src/routes.ts
ui/src/styles.css
ui/src/components/**
ui/src/types.ts
ui/README.md
docs/** (desktop-only UI notes if needed)
```

Forbidden:

- no `app/src/runtime.rs` behavior expansion
- no new IPC commands
- no bridge contract widening
- no `crates/**` changes
- no KB / intake / V3 edits
- no Qwen / Seedance integration
- no V108 import or runtime product capability growth

Current required constraint:

UI alignment may improve product feel and workflow clarity, but it must not
smuggle in backend scope growth.

## Thread 3: V3

```text
Hope-V3FieldOverlay【内容优化中·暂缓推进】
```

Status:

- user is currently optimizing V3 content
- do not push V3 implementation work forward from control
- do not reopen v0.2 engineering scope yet

Next V3 action can happen only after:

- desktop internal trial is underway
- UI alignment gate is materially advanced
- user confirms V3 content optimization is ready for the next review
- control writes a separate v0.2 scope gate

## Thread 4: Intake

```text
Hope接入点-ContractSmokeReadiness【green·待命】
```

Keep standby only.

No new work unless the desktop internal trial reveals a real intake dependency
regression.

## Thread 5: KB

```text
Hope-KB-Standby【知识库冻结待命】
```

Keep standby only.

Do not reopen KB promotion / registry / few-shot work during the current
desktop internal trial step.

## Shared Boundary

Until the next written gate, all threads must preserve:

- no live Qwen
- no Seedance
- no V108 rows imported into runtime
- no product-ready external reference handles
- no `reference_control_core`
- no exporter / workbook / validator / IPC widening

## Exact Next Step

1. continue desktop internal trial with the accepted MVP package
2. begin desktop UI alignment as a bounded desktop-only package
3. leave V3 paused while the user optimizes content
