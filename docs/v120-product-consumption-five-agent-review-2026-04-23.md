# V120 Product Consumption Five-Agent Review 2026-04-23

## Review Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- review anchor before this memo: `7bcec64`
  (`docs: accept V120 full KB ingest route`)
- related KB anchor: `E:\codex\hope-kb / codex/contracts-freeze @ 91fb18c`
  (`data: ingest V120 full KB package`)
- related V3 anchor: `E:\codex\hope / codex/v3-field-overlay-proposal @ 23090ff`
  (`docs: align V3 overlay package with Seedance2 V120`)
- review mode: five-agent control audit before opening Hope main implementation

This review answers one question only:

> After KB and V3 are complete, what exact bridge must Hope main build so the
> desktop product can start calling V120-backed product logic without pretending
> that V120 raw rows are already live runtime truth?

## Final Decision

`V120_PRODUCT_CONSUMPTION_BRIDGE_REQUIRED`

Main control confirms:

- KB V120 full ingest is complete
- V3 V120 alignment is complete
- but Hope main still does **not** have a product-consumption layer for the V120
  package

So the next gate is **not** “import 120 rows straight into runtime truth”.
The next gate is:

`V120 product consumption bridge`

That bridge must sit between:

- `hope-kb` v0.2 package reality
- V3/V120 source semantics
- desktop workbench business calls

## Five-Agent Combined Findings

### Lane 1: Hope Main Reality

Current Hope main can already consume KB only in limited places:

- `scene_taxonomy`
- `failure_pattern`
- `prompt_template`

It cannot yet directly consume V120 package assets such as:

- `golden_sample_library`
- `golden_sample_field_coverage_rules`
- `golden_sample_failure_mapping`
- `golden_sample_repair_mapping`
- `source_register`

Current code still lacks:

- v0.2/V120 Rust DTOs
- real snapshot/manifest consumer for the V120 package
- runtime bootstrap path that loads those DTOs into app state
- validator evidence-aware product structures
- retrieval/selection layer for writer/storyboard/export use

### Lane 2: KB Package Reality

The current KB v0.2 package is already stable as a **read-only package**:

- `120` records
- `classification`, `fewshot`, `negative_sample`, `provenance`,
  `repair_mapping_planning`, `source_fields`, `v3_core_coverage`,
  `validator_evidence`
- `97` few-shot-eligible rows
- `23` closed rows
- `11` negative rows
- `12` reserve rows

This is enough for:

- retrieval
- gating
- routing
- provenance
- validator/repair joins

It is **not yet** enough for direct model calls, because the package still does
not expose:

- runtime prompt payload
- normalized reference handles
- asset bindings
- live `reference_control_core`

### Lane 3: V3 Semantic Gaps

V3 says the remaining product-call gaps are concentrated in five bridge faces:

1. `reference_bundle -> external_reference_handles` candidate layer
2. `prompt_body -> prompt_body_candidate -> compiled_*` layered prompt bridge
3. `continuity_negative_core -> validator blockers / warnings`
4. `sample_type -> structure_mode / sequence_grouping`
5. `scene_performance_core -> canonical core mapping`

These must be solved with additive product structures, not by silently merging
or flattening source fields.

### Lane 4: Desktop Product Needs

The desktop workbench does **not** need the raw V120 23-column rows.
It needs three product calls:

1. `expand_script`
2. `generate_storyboard`
3. `export_bundle`

And it needs those calls to return structured product results, not workbook
source rows.

## Product Consumption Target

The target after this gate is:

### Desktop-facing business calls

`expand_script`

- input:
  - `scene_type`
  - `synopsis_text`
- output:
  - `script_id`
  - `expanded_script_text`
  - `script_hash`
  - optional `warnings[]`

`generate_storyboard`

- input:
  - `task_name`
  - `script_id` or `expanded_script_text`
  - `selected_total_duration_seconds`
- output:
  - `result_id`
  - `rows[]`
  - `duration_plan`
  - `export_status`

Minimum `rows[]` shape:

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

- input:
  - `result_id`
  - `export_format`
- output:
  - `export_manifest_id`
  - `artifacts[]`

### Internal bridge layers that Hope main must add

1. `KbGoldenSampleRuntimePackage`
   - manifest + source register + 120-row library + rules/mappings

2. `GoldenSampleSelectorInput`
   - scene type
   - structure mode
   - desired duration
   - quality / library gates

3. `GoldenSampleSelectionResult`
   - selected rows
   - excluded rows
   - reasoned gate decisions

4. `ExternalReferenceHandleCandidate`
   - names only
   - no paths / URLs / asset IDs
   - no `reference_control_core`

5. `PromptBodyCandidate`
   - source prompt body candidate
   - blocked if placeholder-bearing or gate-closed

6. `ContinuityBlockerSet`
   - continuity warnings
   - negative constraint blockers
   - failure codes / validation blockers

7. `SequenceGrouping`
   - `sequence_id`
   - `shot_order`
   - `single_shot` treated as `not_applicable`, not missing

8. `ScenePerformanceProjection`
   - stable product projection from fused
     `scene_performance_core`

## Exact Product Rule

Hope main must follow this rule:

> Desktop never reads raw V120 workbook rows directly.
> Hope main reads the KB package, applies gates and bridge rules, and returns
> structured product results.

In other words:

- KB package is the product's read-only source
- Hope main is the canonical product-consumption layer
- desktop consumes Hope results, not raw KB tables

## What Opens In This Gate

Allowed in the next main-thread gate:

- v0.2/V120 DTOs in `core-domain`
- runtime package consumer in `project-store`
- app bootstrap into state
- validator evidence-aware bridge structures
- selector / projector structures for:
  - `sample_type`
  - `sequence grouping`
  - `scene_performance_core`
  - `continuity_negative_core`
  - `prompt_body`
  - `reference_bundle`
- structured desktop-facing result contracts

## What Remains Closed

Still closed after this review:

- direct V120 raw-row runtime import
- direct desktop access to V120 23-column source rows
- live Qwen
- live Seedance
- positive few-shot runtime promotion
- product-ready external reference handles
- asset binding / media paths / URLs
- `reference_control_core`
- silent normalization of fused source fields

## Main-Control Call

Main control opens the next Hope main gate as:

`Hope主线-V120ProductConsumptionGate`

This gate should aim for:

- product-consumable V120 bridge
- structured desktop-callable results
- no fake “runtime-ready” claim beyond the bridge actually implemented
