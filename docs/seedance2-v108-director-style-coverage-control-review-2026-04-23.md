# Seedance2 V108 Director Style Coverage Control Review 2026-04-23

## Review Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- current control anchor before this review:
  `39119d6` (`docs: accept V108 validator evidence contract`)
- reviewed KB repo: `E:\codex\hope-kb`
- reviewed KB branch: `codex/contracts-freeze`
- reviewed KB commit:
  `75d8e52` (`docs: review Seedance2 V108 director style coverage`)
- reviewed KB memo:
  `docs/seedance2-v108-director-style-coverage-review-2026-04-23.md`
- review scope: main-control acceptance of KB docs-only coverage review

This review does not modify `hope-kb` seed files, manifests, snapshots, V3,
desktop, intake, Rust DTOs, validators, exporter, workbook, IPC, Qwen,
Seedance, product import behavior, positive few-shot, or
`reference_control_core`.

## Decision

`SEEDANCE2_V108_DIRECTOR_STYLE_COVERAGE_REVIEW_ACCEPTED_AS_PLANNING_INPUT`

Main control accepts the KB review as planning input for a future V108
shot-language / retrieval-selection contract.

Acceptance does not authorize adding seed records, changing the seven-director
library, importing V108 rows, creating runtime selectors, creating prompt text,
or exposing any new style taxonomy in product UI.

## Accepted Findings

The KB review establishes the following planning facts:

- V108 normalized staging has `115` rows.
- `style_cluster` distribution is:
  - `基础迁移 = 40`
  - `国漫 = 29`
  - `日漫 = 25`
  - `美漫 = 13`
  - `原创 = 8`
- `scene_category` distribution is:
  - `群像表演 = 27`
  - `热血打斗 = 25`
  - `场域追逐 = 24`
  - `情绪对话 = 19`
  - `其他 = 13`
  - `相遇表演 = 7`
- the current seven-director fusion model remains the default background model
  for broad action, crowd pressure, close combat, emotion landscape, daily
  micro-performance, subjective deformation, and suspense / transition behavior
- the current model is insufficient for V108-specific style-cluster routing in:
  - guofeng / wuxia / xianxia action grammar
  - war oath / army charge / epic group blocking
  - urban apocalypse / industrial pressure routing
  - guochao stage / original performance blocking

## Accepted Prototype Candidates

Main control accepts these four names as the immediate KB-only expansion
candidates:

- `国风武侠 / 仙侠动作调度`
- `国漫史诗群像 / 战争场面调度`
- `都市末世 / 工业压迫调度`
- `国潮舞台 / 原创表演调度`

They may be added by `hope-kb` as bounded director-style prototype records
before the main `V108ShotLanguageSelectionContract` is drafted.

They are not accepted as seed records, visible prompt text, runtime taxonomy,
UI copy, director imitation instructions, Qwen prompt instructions, Seedance
overlay content, exporter fields, workbook fields, validator enums, or
`reference_control_core` entries until the KB expansion gate completes and main
control reviews the resulting commit.

## Assertions Remain Frozen

```text
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
```

Additional boundaries remain unchanged:

```text
v108_rows_imported_into_seed = 0
v108_rows_imported_into_product_structure = 0
positive_fewshot_promotions = 0
qwen_calls = 0
seedance_calls = 0
v3_branch_edits = 0
hope_product_code_changes = 0
desktop_or_intake_changes = 0
rust_dto_validator_exporter_workbook_ipc_changes = 0
```

## Route Impact

This acceptance opens one immediate KB-only route:

`V108DirectorStyleExpansion`

That route may add enough director-style prototype coverage for the V108
evidence shape before the main shot-language selector is drafted.

After KB expansion is accepted, the main route may open:

`V108ShotLanguageSelectionContract`

That later route may define how a future planner chooses camera-language
evidence from multiple content candidates by using:

- `style_cluster`
- `scene_category`
- `camera_directing_core`
- `covered_points`
- `missed_points`
- `teaching_note`
- `ip_abstraction_note`
- `continuity_negative_core`
- current seven-director background lanes
- the four accepted prototype candidates above

Neither route may implement runtime generation, prompt assembly, model calls,
validator behavior, repair behavior, or product import.

## Validation

This was a docs-only control review. No code tests were required or run.
