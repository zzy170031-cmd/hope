# Seedance2 V108 Shot-Language Selection Contract Dispatch 2026-04-23

## Dispatch Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- package type: docs-only main-thread planning dispatch
- accepted KB director-style expansion:
  `docs/seedance2-v108-director-style-expansion-control-review-2026-04-23.md`
- accepted validator evidence review:
  `docs/seedance2-v108-validator-evidence-contract-review-2026-04-23.md`

The KB director-style prerequisite is now accepted. The main thread may draft a
planning contract for choosing shot language from multiple content candidates.

## Thread Label

```text
Hope主线-Seedance2V108ShotLanguageSelection【Contract候选执行中】
```

## Task

Create a docs-only `V108ShotLanguageSelectionContract` candidate.

The candidate must describe how a future planner can choose suitable camera
language from multiple content candidates and convert that evidence into
bounded storyboard-word planning signals.

This is not implementation. The candidate must not generate final storyboard
text, prompts, Qwen messages, Seedance overlay text, exporter rows, workbook
fields, validator behavior, repair behavior, product import behavior, or UI
taxonomy.

## Required Inputs

Read these Hope files:

- `docs/seedance2-v108-shot-language-selection-contract-dispatch-2026-04-23.md`
- `docs/seedance2-v108-director-style-expansion-control-review-2026-04-23.md`
- `docs/seedance2-v108-director-style-coverage-control-review-2026-04-23.md`
- `docs/seedance2-v108-validator-evidence-contract-review-2026-04-23.md`
- `docs/seedance2-v108-validator-evidence-contract-candidate-2026-04-23.md`
- `docs/generation-field-rule-contract-review-2026-04-22.md`
- `docs/generation-field-rule-contract-candidate-2026-04-22.md`
- `docs/seedance2-v108-vnext-schema-decision-2026-04-23.md`

Read these KB files as read-only evidence:

- `E:\codex\hope-kb\docs\seedance2-v108-director-style-web-research-2026-04-23.md`
- `E:\codex\hope-kb\docs\seedance2-v108-director-style-coverage-review-2026-04-23.md`
- `E:\codex\hope-kb\docs\seedance2-v108-field-mapping-review-2026-04-23.md`
- `E:\codex\hope-kb\docs\seedance2-v108-v3-alignment-report-2026-04-23.md`
- `E:\codex\hope-kb\seed\v0.1\director_profiles.json`
- `E:\codex\hope-kb\seed\v0.1\director_rules.json`
- `E:\codex\hope-kb\seed\v0.1\director_reference_sets.json`
- `E:\codex\hope-kb\seed\v0.1\director_scene_affinity.json`

## Required Candidate Content

The candidate must define planning names for:

- `V108ShotLanguageSelectionContract`
- `V108ShotLanguageSourceBundle`
- `V108StyleLaneSelectionInput`
- `V108CameraLanguageSelectionRule`
- `V108StoryboardWordPlanningSignal`
- `V108ShotLanguageSelectionBlocker`
- `V108ShotLanguageSelectionReadiness`
- `V108ShotLanguageFutureGateOrder`

These names are planning names only. They must not become Rust types, database
tables, runtime enums, failure codes, severity levels, thresholds, IPC payloads,
workbook fields, exporter fields, Qwen templates, or Seedance instructions in
this gate.

The candidate must explain how selection would be bounded by:

- source content intent
- `style_cluster`
- `scene_category`
- `camera_directing_core`
- sequence context from `sample_type`, `sequence_id`, and `shot_order`
- evidence quality from `covered_points`, `missed_points`, `teaching_note`,
  `ip_abstraction_note`, and `continuity_negative_core`
- the original seven director-style background lanes
- the four newly accepted internal style lanes:
  - `director_08`: `内部风格通道：国风武侠/仙侠动作调度`
  - `director_09`: `内部风格通道：国漫史诗群像/战争场面调度`
  - `director_10`: `内部风格通道：都市末世/工业压迫调度`
  - `director_11`: `内部风格通道：国潮舞台/原创表演调度`

The candidate must include a planning matrix that shows:

- when the original seven director-style lanes are enough
- when one of the four new V108 internal style lanes should be considered
- how `国漫 / 热血打斗`, `国漫 / 场域追逐`, `国漫 / 群像表演`,
  `原创 / 群像表演`, and urban-apocalypse / industrial-pressure rows are routed
- when selection must fall back to raw evidence only
- when selection is blocked by placeholders, unresolved references, V3
  alignment gaps, missing schema fields, or unopened promotion gates

The candidate must explain the difference between:

- internal retrieval metadata
- camera-language planning signals
- final storyboard words
- runtime prompt text

Only the first two are allowed in this gate. Final storyboard words and runtime
prompt text remain closed.

## Required Assertions

Keep these assertions unchanged:

```text
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
```

Also keep these boundaries:

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

## Forbidden Scope

Do not change:

- Rust DTOs
- validator code
- repair code
- exporter code
- workbook shape
- IPC
- desktop
- intake
- `hope-kb`
- V3 branch files
- seed manifests
- seed bundles
- snapshot contracts
- Qwen or Seedance integrations

Do not:

- import V108 rows
- promote positive few-shot rows
- create `reference_control_core`
- invent external reference handles
- invent character appearances or scene designs
- write runtime prompt templates
- expose style-lane names as product-facing output
- use research links as product-ready external reference handles
- write instructions to imitate any real director

## Output

Create:

```text
docs/seedance2-v108-shot-language-selection-contract-candidate-2026-04-23.md
```

Commit and push to `origin/codex/contracts-freeze`.

## Report Back To Main Control

Return:

- final branch / commit
- changed files
- planning objects
- selection matrix summary
- eleven-style-lane routing summary
- unchanged assertions
- unresolved blockers
- validation result; if docs-only and no tests were run, say so
