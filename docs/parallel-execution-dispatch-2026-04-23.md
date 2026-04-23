# Parallel Execution Dispatch 2026-04-23

## Control Decision

KB expansion is closed for this cycle.

The project now runs parallel work across independent threads:

- Hope main: implement the accepted V108 internal retrieval selection slice.
- Desktop: run desktop MVP packaging / smoke readiness in parallel.
- Intake: run intake contract / snapshot smoke readiness in parallel.
- KB: standby only; no new KB expansion, no registry rows, no seed changes.
- V3: archived standby; no branch edits.

This dispatch does not merge KB into Hope, does not open Qwen / Seedance, and
does not open product-ready external references.

## Thread 1: Hope Main

```text
Hope主线-Seedance2V108RetrievalSelectionImplementation【小范围实现执行中】
```

Target:

```text
E:\codex\hope / codex/contracts-freeze
```

Starting control anchor:

```text
29bbf81 docs: scope V108 retrieval selection implementation
```

Task:

Implement only the accepted non-runtime planning/debug metadata slice.

Allowed files:

```text
crates/storyboard-pipeline/src/shot_language_selection.rs
crates/storyboard-pipeline/src/lib.rs
crates/storyboard-pipeline/tests/v108_shot_language_selection.rs
```

Required tests:

```text
cargo test -p storyboard-pipeline --test v108_shot_language_selection
cargo test -p storyboard-pipeline
```

Forbidden:

- app/runtime/product behavior
- exporter/workbook/IPC
- validator/repair
- Qwen/Seedance
- final storyboard words
- runtime prompt text
- V108 row import
- positive few-shot promotion
- registry rows
- external reference handles
- `reference_control_core`
- KB/V3/desktop/intake edits

Report:

- final branch / commit
- changed files
- implemented metadata output shape
- test results
- unchanged assertions

## Thread 2: Desktop

```text
Hope桌面端-MVPPackagingSmoke【并行验证中】
```

Target:

```text
E:\codex\hope-desktop-shell / codex/desktop-shell
```

Starting anchor:

```text
66d687d desktop-shell: add writer qwen input readiness boundary
```

Task:

Run desktop-shell readiness / packaging / smoke verification for the MVP path.
Do not wait for V108 registry or Qwen / Seedance. Treat the new selector as a
future internal metadata surface, not a desktop feature dependency.

Allowed:

- read existing desktop packaging/build docs
- run existing build/test/package/smoke commands
- document blockers
- fix packaging-only issues only if they are inside desktop-shell and do not
  alter product scope

Forbidden:

- changing `E:\codex\hope`
- changing KB, V3, or intake
- adding Qwen / Seedance integration
- exposing V108 style lane names as UI text
- adding reference handles or `reference_control_core`

Report:

- final branch / commit if changed
- build/package/smoke commands run
- pass/fail
- blockers for desktop MVP
- whether desktop remains ready for MVP candidate

## Thread 3: Intake

```text
Hope接入点-ContractSmokeReadiness【并行验证中】
```

Target:

```text
E:\codex\hope-intake-app / codex/controlled-intake-snapshot-bootstrap-app
```

Starting anchor:

```text
d74bd11 qwen-storyboard: tighten contract boundary
```

Task:

Run intake contract / snapshot / smoke readiness for the MVP path. Confirm the
current intake package remains compatible with the product boundary where Qwen,
Seedance, V108 row import, and reference handles are closed.

Allowed:

- read current intake contract docs
- run existing build/test/smoke commands
- document blockers
- fix intake-only contract smoke issues if they do not open Qwen / Seedance or
  product import scope

Forbidden:

- changing `E:\codex\hope`
- changing KB, V3, or desktop
- adding Qwen / Seedance calls
- importing V108 rows
- creating product-ready handles or `reference_control_core`

Report:

- final branch / commit if changed
- commands run
- pass/fail
- blockers for intake MVP
- whether intake remains ready for MVP candidate

## Thread 4: KB

```text
Hope-KB-Standby【知识库冻结待命】
```

Target:

```text
E:\codex\hope-kb / codex/contracts-freeze
```

Starting anchor:

```text
513c681 docs: propose V108 canonical object registry schema
```

Task:

Standby only. Do not expand KB, director lanes, golden samples, registry schema,
registry rows, seed, manifests, import maps, or snapshots unless main control
opens a new gate.

Allowed:

- pull / status check
- report clean standby

Forbidden:

- any content expansion
- registry rows
- V108 import
- positive few-shot promotion
- `reference_control_core`
- Qwen / Seedance
- Hope / desktop / intake edits

## Thread 5: V3

```text
Hope-V3FieldOverlay【归档待命】
```

Target:

```text
E:\codex\hope / codex/v3-field-overlay-proposal
```

Known archive anchor:

```text
bc745a4 docs: align V3 proposal with Seedance2 V108
```

Task:

Archived standby. Do not edit V3 unless main control opens a v0.2 overlay merge
or schema-alignment gate.

## Shared Git Rule

All threads should use:

```text
& 'C:\Program Files\Git\cmd\git.exe' -C <repo_path> pull --ff-only
& 'C:\Program Files\Git\cmd\git.exe' -C <repo_path> status --short --branch
```

If direct `git` is unavailable in an existing thread, this is a PATH refresh
issue, not a repo failure.
