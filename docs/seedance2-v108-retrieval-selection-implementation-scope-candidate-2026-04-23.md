# Seedance2 V108 Retrieval Selection Implementation Scope Candidate 2026-04-23

## Route

This is a docs-only implementation scope candidate for the Hope main control
thread.

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor before this memo:
  `780bdb8` (`docs: dispatch V108 retrieval selection scope`)
- accepted shot-language selection review:
  `docs/seedance2-v108-shot-language-selection-contract-review-2026-04-23.md`
- accepted registry schema proposal review:
  `docs/seedance2-v108-canonical-object-registry-schema-proposal-control-review-2026-04-23.md`
- accepted validator evidence review:
  `docs/seedance2-v108-validator-evidence-contract-review-2026-04-23.md`
- package type: docs-only implementation scope candidate

This memo does not implement retrieval selection. It only names the safest
future files, tests, forbidden files, validation commands, rollback plan, and
remaining blockers for a later implementation gate.

## Reviewed Inputs

Hope-side inputs:

- `docs/seedance2-v108-retrieval-selection-implementation-scope-dispatch-2026-04-23.md`
- `docs/seedance2-v108-shot-language-selection-contract-review-2026-04-23.md`
- `docs/seedance2-v108-shot-language-selection-contract-candidate-2026-04-23.md`
- `docs/seedance2-v108-canonical-object-registry-schema-proposal-control-review-2026-04-23.md`
- `docs/seedance2-v108-canonical-object-registry-readiness-control-review-2026-04-23.md`
- `docs/seedance2-v108-validator-evidence-contract-review-2026-04-23.md`
- `docs/seedance2-v108-validator-evidence-contract-candidate-2026-04-23.md`
- `docs/generation-field-rule-contract-review-2026-04-22.md`
- `docs/generation-field-rule-contract-candidate-2026-04-22.md`
- `docs/seedance2-v108-vnext-schema-decision-2026-04-23.md`

KB-side inputs, read only from `E:\codex\hope-kb`:

- `docs/seedance2-v108-director-style-web-research-2026-04-23.md`
- `docs/seedance2-v108-canonical-object-registry-schema-proposal-2026-04-23.md`
- `seed/v0.1/director_profiles.json`
- `seed/v0.1/director_rules.json`
- `seed/v0.1/director_reference_sets.json`
- `seed/v0.1/director_scene_affinity.json`

## Scope Decision

`SEEDANCE2_V108_RETRIEVAL_SELECTION_IMPLEMENTATION_SCOPE_CANDIDATE_DOCS_ONLY`

Recommended first implementation slice:

```text
pure planning/debug metadata module + fixture-only tests
```

The later implementation should not start with a runtime selector. It should
first create a non-runtime module that can classify synthetic V108-like evidence
into internal planning metadata:

- selected or blocked internal style lane IDs
- abstract camera-language planning signals
- blocker families
- no-runtime-import assertions

This is safer than a runtime selector because registry rows, product-ready
external reference handles, positive few-shot promotion, import schema, runtime
prompt text, Qwen calls, and Seedance calls all remain closed.

## Exact Implementation Goal

The later code gate should implement an internal planning/debug metadata
surface only.

Allowed future output shape:

```text
internal_style_lane_candidates
camera_language_planning_signals
selection_blockers
selection_readiness
no_runtime_import_assertions
```

Disallowed future output shape:

```text
final_storyboard_words
runtime_prompt_text
Qwen message content
Seedance prompt or overlay wording
exporter rows
workbook columns
product-ready external_reference_handles
reference_control_core
```

The module may answer "which internal lane would be considered and why" for
debug purposes. It must not answer "what storyboard sentence should be written"
or "what prompt should be sent".

## Candidate Implementation File List

The later implementation gate should be limited to these files:

```text
E:\codex\hope\crates\storyboard-pipeline\src\shot_language_selection.rs
E:\codex\hope\crates\storyboard-pipeline\src\lib.rs
E:\codex\hope\crates\storyboard-pipeline\tests\v108_shot_language_selection.rs
```

Expected file responsibilities:

| file | allowed later responsibility | hard boundary |
| --- | --- | --- |
| `crates/storyboard-pipeline/src/shot_language_selection.rs` | New pure planning/debug metadata module for internal lane candidates, planning signals, blockers, and readiness. | No final words, no prompt text, no import, no registry rows, no model calls. |
| `crates/storyboard-pipeline/src/lib.rs` | Minimal module export only, for example exposing `shot_language_selection`. | Do not change existing runtime-authoritative storyboard planning behavior. |
| `crates/storyboard-pipeline/tests/v108_shot_language_selection.rs` | New fixture-only integration tests with synthetic V108-like inputs and no raw V108 row import. | Do not use real V108 rows as product data; do not assert final generated text. |

Implementation should not require Cargo dependency changes. If a later code
gate believes dependencies or workspace configuration are required, that is a
new scope gate.

## Candidate Test List

The later implementation gate should add and run:

```text
cargo test -p storyboard-pipeline --test v108_shot_language_selection
cargo test -p storyboard-pipeline
```

Minimum test cases for
`crates/storyboard-pipeline/tests/v108_shot_language_selection.rs`:

- `routes_guofeng_action_to_director_08_metadata_only`
- `routes_epic_group_or_war_to_director_09_metadata_only`
- `routes_urban_apocalypse_pressure_to_director_10_metadata_only`
- `routes_stage_group_performance_to_director_11_metadata_only`
- `keeps_original_seven_lanes_for_broad_action_emotion_suspense`
- `placeholder_blocks_selection_to_raw_evidence_only`
- `reference_unresolved_preserves_canonical_name_missing`
- `schema_and_promotion_gates_keep_runtime_closed`
- `never_emits_final_storyboard_words_or_runtime_prompt_text`

If the later gate touches only `storyboard-pipeline`, no exporter, validator,
project-store, app, desktop, intake, or KB tests are required.

## Forbidden File List

The later implementation gate must not touch:

```text
E:\codex\hope\app\src\*
E:\codex\hope\ui\src\*
E:\codex\hope\crates\core-domain\src\*
E:\codex\hope\crates\project-store\src\*
E:\codex\hope\crates\validators\src\*
E:\codex\hope\crates\export-engine\src\*
E:\codex\hope\crates\export-engine\tests\*
E:\codex\hope\crates\writer-pipeline\src\*
E:\codex\hope\contracts\fixtures\week3-shared-fixture.json
E:\codex\hope\Cargo.toml
E:\codex\hope\Cargo.lock
E:\codex\hope-kb\*
E:\codex\hope-desktop-shell\*
E:\codex\hope-intake-app\*
```

Inside `crates/storyboard-pipeline/src/lib.rs`, the later gate must not alter:

```text
build_storyboard_plan(...)
aggregate_director_assignment(...)
build_prompt_layers(...)
derive_handoff_zone(...)
validate_render_segment_window(...)
```

Those functions preserve the existing runtime-authoritative behavior and the
precedence / collision / negative-boundary protections already landed in
focused tests.

## Data Boundary For Eleven Style Lanes

The eleven style lanes may enter only as internal static planning metadata in
the future module.

Allowed lane evidence:

- lane ID, such as `director_08`
- internal style family, such as `guofeng_wuxia_action`
- abstract affinity families, such as `battle`, `chase`, `crowd`,
  `industrial_pressure`, or `stage_performance`
- allowed camera-language signal names, such as `weapon_continuity`,
  `formation_depth`, `evacuation_axis`, `stage_axis`, or `music_sync_cue`

Forbidden lane evidence:

- visible product UI labels
- real-person imitation instructions
- Qwen prompt wording
- Seedance prompt wording
- final storyboard prose
- exporter fields
- workbook fields
- validator enums
- repair rules
- reference handles
- `reference_control_core`

The original seven lanes remain the default broad background model. The
additional four lanes are precision routes only:

- `director_08`: guofeng / wuxia / xianxia action, weapon continuity, qinggong
  movement axis, roof / bamboo / rain spatial anchors
- `director_09`: epic Chinese-animation crowd, war oath, army formation,
  command hierarchy, flag / drum bridge, crowd rhythm
- `director_10`: urban apocalypse, subway / overpass / ruins / evacuation,
  infrastructure pressure, alarm bridge, industrial compression
- `director_11`: guochao stage, performer entrance, synchronized group
  movement, music beat, spotlight timing, final freeze

## Registry Schema Boundary

The accepted registry schema proposal may be referenced as blocked metadata
only.

Allowed future metadata:

- `canonical_name_missing`
- `alias_unreviewed`
- `schema_not_accepted`
- `product_handle_gate_not_accepted`
- `reference_control_core_closed`

Forbidden future data:

- registry rows
- canonical object records
- accepted object names
- product-ready `external_reference_handles`
- source object seed files
- manifest or import_map entries
- asset URLs, asset IDs, image paths, character appearances, scene designs
- `reference_control_core`

The selector must treat unresolved reference material as blocker evidence, not
as something to repair locally or infer from style lanes.

## Blocker Behavior

The later implementation must preserve these blocker rules:

| blocker | required behavior |
| --- | --- |
| `placeholder_present` | Return raw evidence / blocked readiness; do not select lane as product behavior. |
| `prompt_body_blocked_by_placeholder` | Do not use prompt text as selection output or few-shot context. |
| `reference_handle_unresolved` | Keep reference evidence raw; do not create handles. |
| `canonical_name_missing` | Preserve blocked registry metadata; do not invent object names. |
| `v3_alignment_gap` | Do not reuse old V3 meanings or split fused fields without a later gate. |
| `schema_field_missing` | Do not import V108 rows or write product structures. |
| `promotion_gate_not_accepted` | Do not promote positive few-shot or runtime use. |
| `reference_control_core_closed` | Do not create or infer `reference_control_core`. |

Blockers must be additive and inspectable. They must not be silently converted
into defaults or fallback selections.

## Proposed Minimal Implementation Slices

Slice order, from safest to riskiest:

1. `fixture-only tests`
   - Add synthetic test inputs and expected blocker/signal assertions.
   - No production code yet if main control wants an even softer first gate.
2. `pure planning/debug metadata module`
   - Add `shot_language_selection.rs` with structs/enums local to
     `storyboard-pipeline`.
   - Return internal lane candidates, planning signal names, blockers, and
     readiness.
3. `non-runtime internal selector`
   - Add deterministic selection ranking inside the same module.
   - Still no runtime integration, no `build_storyboard_plan` changes, no
     prompt text, no final words.
4. `debug metadata bridge`
   - Only after a later gate, decide whether planning output can be surfaced as
     debug metadata. This is not part of the first implementation.
5. `runtime integration`
   - Not eligible now. Requires separate gates for import/schema, validator,
     registry rows/handles, writer/Qwen, and exporter boundaries.

Recommended first implementation package:

```text
Slice 1 + Slice 2 only:
fixture-only tests + pure planning/debug metadata module
```

Do not start with Slice 3 unless main control explicitly accepts the metadata
module and confirms that deterministic ranking is still non-runtime.

## Rollback Plan

If the later implementation gate drifts, rollback is simple because the first
slice should touch only `storyboard-pipeline`:

1. Revert the new integration test file:
   `crates/storyboard-pipeline/tests/v108_shot_language_selection.rs`.
2. Revert the new module:
   `crates/storyboard-pipeline/src/shot_language_selection.rs`.
3. Revert the minimal module export line in:
   `crates/storyboard-pipeline/src/lib.rs`.
4. Re-run `cargo test -p storyboard-pipeline`.
5. Confirm `git status --short --branch` is clean or only contains the intended
   reverted changes.

No data migration, seed rebuild, exporter fixture refresh, workbook update,
desktop build, intake build, Qwen key, or Seedance key should be involved.

## Later Validation Plan

Required validation for the later implementation gate:

```text
cargo fmt -p storyboard-pipeline
cargo test -p storyboard-pipeline --test v108_shot_language_selection
cargo test -p storyboard-pipeline
```

Additional validation only if drift occurs:

- If `core-domain` is touched, stop and return to scope gate.
- If `validators` is touched, stop and return to validator implementation
  scope gate.
- If `export-engine` is touched, stop and return to exporter scope gate.
- If `app`, `ui`, desktop, or intake are touched, stop and return to product
  integration scope gate.
- If `hope-kb` is touched, stop and return to KB-only gate.

## Unchanged Assertions

The later implementation must preserve:

```text
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
qwen_calls = 0
seedance_calls = 0
desktop_or_intake_changes = 0
v3_branch_edits = 0
```

## Unresolved Blockers

The following blockers remain unresolved after this scope candidate:

- `placeholder_present`
- `prompt_body_blocked_by_placeholder`
- `reference_handle_unresolved`
- `canonical_name_missing`
- `v3_alignment_gap`
- `schema_field_missing`
- `promotion_gate_not_accepted`
- `reference_control_core_closed`
- no registry rows
- no product-ready external reference handles
- no V108 row import
- no positive few-shot promotion
- no final storyboard words
- no runtime prompt text

## Explicitly Closed In This Gate

This scope candidate does not authorize:

- Rust implementation in this docs-only gate
- app/runtime/exporter/workbook/IPC changes
- `hope-kb` changes
- V3 changes
- desktop changes
- intake changes
- V108 row import
- positive few-shot promotion
- registry row creation
- product-ready `external_reference_handles`
- `reference_control_core`
- final storyboard words
- runtime prompt text
- Qwen calls
- Seedance calls

## Candidate Completion

This candidate is complete when committed and pushed to
`origin/codex/contracts-freeze`.

Recommended next label:

```text
Hope主线-Seedance2V108RetrievalSelectionScope【候选已完成·待接收】
```

Validation for this package is docs-only. No code tests are required or run in
this gate.
