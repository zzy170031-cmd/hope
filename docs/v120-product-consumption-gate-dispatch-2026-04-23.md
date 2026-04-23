# V120 Product Consumption Gate Dispatch 2026-04-23

## Control Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- dispatch anchor before this gate: `7bcec64`
  (`docs: accept V120 full KB ingest route`)
- related KB completion anchor: `E:\codex\hope-kb / codex/contracts-freeze @ 91fb18c`
  (`data: ingest V120 full KB package`)
- related V3 completion anchor: `E:\codex\hope / codex/v3-field-overlay-proposal @ 23090ff`
  (`docs: align V3 overlay package with Seedance2 V120`)

Hope main is still under `RC_READY + scope freeze`.

This dispatch opens one bounded main-thread gate only:

`V120 product consumption bridge`

It does **not** authorize:

- live Qwen
- live Seedance
- direct raw-row runtime import of all 120 rows
- product-ready external reference handles
- asset binding / media paths / URLs
- `reference_control_core`
- silent normalization of V120 source semantics

## Thread To Dispatch

```text
Hope主线-V120ProductConsumptionGate【产品消费桥执行中】
```

## Target

```text
E:\codex\hope / codex/contracts-freeze
```

## Why This Gate Is Opened

KB and V3 are now complete inside their own lanes:

- KB has ingested the full V120 package
- V3 has aligned the V120 source meaning

But Hope main still lacks the product-consumption layer that turns that package
into structured product results.

Desktop must not read raw V120 workbook rows.
Desktop should call Hope main business operations.
Hope main should read the KB package, apply bridge rules, and return structured
results.

## Exact Goal

Build the first product-consumption bridge that lets Hope main consume the V120
KB package and expose three structured business surfaces:

1. `expand_script`
2. `generate_storyboard`
3. `export_bundle`

The bridge must be honest:

- product-consumable: yes
- direct model/live-adapter truth: still no

## Required Product Contract

### Business input / output surfaces

`expand_script`

- request:
  - `scene_type`
  - `synopsis_text`
- response:
  - `script_id`
  - `expanded_script_text`
  - `script_hash`
  - optional `warnings[]`

`generate_storyboard`

- request:
  - `task_name`
  - `script_id` or `expanded_script_text`
  - `selected_total_duration_seconds`
- response:
  - `result_id`
  - `rows[]`
  - `duration_plan`
  - `export_status`

Minimum `rows[]` contract:

- `shot_id`
- `order`
- `person`
- `shot_title`
- `scene_scale`
- `visual_description`
- `character_action`
- `dialogue`
- `prompt_text`
- `duration_seconds`

`export_bundle`

- request:
  - `result_id`
  - `export_format`
- response:
  - `export_manifest_id`
  - `artifacts[]`

## Required Internal Bridge Faces

The main thread must bridge the following V120/V3 gaps explicitly:

1. `reference_bundle -> external_reference_handle_candidates`
   - names only
   - no asset bindings
   - no URLs / paths / media IDs

2. `prompt_body -> prompt_body_candidate`
   - raw source prompt body must not become runtime truth directly
   - placeholder-bearing rows remain blocked

3. `continuity_negative_core -> validator blockers / warnings`
   - deterministic projection only
   - no hidden repair magic

4. `sample_type -> structure_mode / sequence_grouping`
   - preserve `single_shot / sequence_shot`
   - surface `sequence_id` + `shot_order`
   - blank single-shot sequence fields = `not_applicable`

5. `scene_performance_core -> canonical product projection`
   - do not silently split / merge
   - add an explicit projection layer

## Minimum File Scope

Main thread should start in this scope:

```text
E:\codex\hope\app\src\runtime.rs
E:\codex\hope\app\src\state.rs
E:\codex\hope\crates\core-domain\src\kb.rs
E:\codex\hope\crates\core-domain\src\contracts.rs
E:\codex\hope\crates\project-store\src\kb_runtime.rs
E:\codex\hope\crates\project-store\migrations\hope-kb\**
E:\codex\hope\crates\validators\src\contract.rs
```

If a helper module is needed for selector / projector structures, keep it
inside the minimum bounded surface and report it explicitly.

## Strong Guidance On First Cut

Do **not** start by pushing V120 into writer/export/runtime truth.

Do start by making Hope main able to:

1. load the V120 KB package
2. verify manifest / package shape
3. materialize stable DTOs
4. project source/evidence fields into product bridge structures
5. return structured product results for desktop use

The first working bridge may remain internal / metadata-backed, but it must be
real enough that desktop does not depend on fake placeholder payloads anymore.

## Suggested First Implementation Slice

The first slice should achieve all of the following:

1. `core-domain` defines v0.2/V120 DTOs for:
   - package manifest
   - golden sample library record
   - field coverage rule
   - failure mapping
   - repair mapping
   - source register
   - bridge result structures

2. `project-store` can load the V120 package instead of the old fixed v0.1-only
   shell

3. `app/state` can hold a loaded V120 runtime package

4. `runtime.rs` can surface:
   - `expand_script`
   - `generate_storyboard`
   - `export_bundle`
   as real structured operations, even if model/adapters remain closed

5. `validators` can carry evidence-aware blocker / warning outputs instead of
   only flat 5-column report rows

## Still Forbidden

The main thread must not:

- claim that V120 is now live few-shot runtime truth
- route desktop directly to raw KB package files
- open live Qwen or Seedance
- invent `reference_control_core`
- treat `reference_bundle` as real asset binding
- promote placeholder-bearing `prompt_body` into compiled runtime payload
- widen workbook / exporter contracts beyond the current bounded need
- edit `hope-kb`, `hope-desktop-shell`, `hope-intake-app`, or V3 branch work

## Required Report Back

The main thread must report:

- final branch / commit
- changed files
- added DTO list
- added loader / bootstrap list
- implemented bridge faces
- which desktop-facing business calls are now real
- which parts still remain gated
- validation / test result
- exact remaining blockers before desktop can call the bridge end-to-end

## Success Standard

This gate is successful only if main control can truthfully say:

> Hope main now knows how to consume the V120 KB package as product input and
> can return structured desktop-facing results without pretending that raw V120
> rows are already direct runtime/model truth.
