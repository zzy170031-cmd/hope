# Seedance2 V108 Director Style Expansion Control Review 2026-04-23

## Review Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- control dispatch commit:
  `e866712` (`docs: dispatch V108 director style expansion`)
- reviewed KB repo: `E:\codex\hope-kb`
- reviewed KB branch: `codex/contracts-freeze`
- reviewed KB commit:
  `434c74a` (`docs: add V108 director style expansion research`)
- reviewed KB research doc:
  `docs/seedance2-v108-director-style-web-research-2026-04-23.md`
- review scope: main-control acceptance of KB-only director-style expansion

This review does not modify `hope-kb`, V3, desktop, intake, Rust DTOs,
validators, exporter, workbook, IPC, Qwen, Seedance, product import behavior,
positive few-shot promotion, or `reference_control_core`.

## Decision

`SEEDANCE2_V108_DIRECTOR_STYLE_EXPANSION_ACCEPTED`

Main control accepts the KB director-style expansion as the current internal
style-lane baseline for future V108 shot-language selection planning.

Acceptance does not authorize runtime generation, product prompt text, Qwen
messages, Seedance overlay text, product UI taxonomy, V108 row import, positive
few-shot promotion, external reference handles, or `reference_control_core`.

## Accepted Changes

The KB expansion changed only KB files:

- `docs/live-progress.md`
- `docs/seedance2-v108-director-style-web-research-2026-04-23.md`
- `seed/v0.1/director_profiles.json`
- `seed/v0.1/director_rules.json`
- `seed/v0.1/director_reference_sets.json`
- `seed/v0.1/director_scene_affinity.json`
- `seed/v0.1/manifest.json`

Accepted count changes:

```text
director_profiles: 7 -> 11
director_rules: 7 -> 11
director_reference_sets: 7 -> 11
director_scene_affinity: 7 -> 11
```

The v0.1 manifest now records count `11` for all four director-style assets.

## Accepted New Internal Style Lanes

The following lanes are accepted as internal KB style lanes:

- `director_08`: `内部风格通道：国风武侠/仙侠动作调度`
- `director_09`: `内部风格通道：国漫史诗群像/战争场面调度`
- `director_10`: `内部风格通道：都市末世/工业压迫调度`
- `director_11`: `内部风格通道：国潮舞台/原创表演调度`

They are accepted because the KB research doc ties each lane to V108 gaps,
public source links, representative works / scenes, and camera-language
capabilities. The accepted gaps are:

- `国漫 / 热血打斗 = 11`
- `国漫 / 场域追逐 = 10`
- `国漫 / 群像表演 = 7`
- `原创 / 群像表演 = 8`
- urban-apocalypse / industrial-pressure evidence across subway, overpass,
  ruins, evacuation, alarm, and future-battlefield rows
- stage / group-performance evidence across ritual performance, music staging,
  synchronized movement, entrance timing, and final-freeze rows

## Accepted Boundary

The new lanes are internal retrieval and camera-language metadata only.

They must not be used as:

- real-person imitation instructions
- visible product prompt text
- UI labels
- desktop taxonomy output
- Qwen prompt wording
- Seedance overlay wording
- exporter fields
- workbook fields
- validator enums
- repair rules
- external reference handles
- `reference_control_core` entries

The KB research links are evidence for why a lane can guide shot-language
selection. They are not product-ready external reference handles.

## Assertions Remain Frozen

```text
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
```

Additional unchanged boundaries:

```text
v108_rows_imported_into_product_structure = 0
positive_fewshot_promotions = 0
qwen_calls = 0
seedance_calls = 0
v3_branch_edits = 0
hope_product_code_changes = 0
desktop_or_intake_changes = 0
rust_dto_validator_exporter_workbook_ipc_changes = 0
```

The KB seed expansion updates v0.1 director-style assets only. It does not
import V108 rows into v0.2 golden samples and does not promote V108 source rows.

## Validation

Main control re-ran the KB seed validation locally:

```text
powershell -ExecutionPolicy Bypass -File scripts\validate-seed-bundle.ps1
```

Result: passed.

The validation output confirmed:

```text
director_profile = 11
director_rule = 11
director_reference_set = 11
director_scene_affinity = 11
Seed bundle validation passed.
```

KB also reported v0.1 snapshot rebuild success with `quick_check = ok` at:

```text
snapshots/hope-kb-v0.1.rebuilt-15.sqlite3
```

## Route Impact

The director-style prerequisite for the main V108 shot-language selection
contract is now satisfied.

Main control may now dispatch:

```text
Hope主线-Seedance2V108ShotLanguageSelection【Contract候选执行中】
```

That next gate remains docs-only. It may reference the eleven internal
director-style lanes as planning metadata, but it must not implement runtime
generation or promote any V108 row.
