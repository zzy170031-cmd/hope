# Seedance2 V108 Director Style Expansion Dispatch 2026-04-23

## Dispatch Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- KB target repo: `E:\codex\hope-kb`
- KB target branch: `codex/contracts-freeze`
- KB starting anchor:
  `75d8e52` (`docs: review Seedance2 V108 director style coverage`)
- source control review:
  `docs/seedance2-v108-director-style-coverage-control-review-2026-04-23.md`
- package type: KB-only director-style expansion gate

This dispatch opens KB work because V108 has enough `国漫` and `原创` evidence
to make later shot-language selector design expensive to retrofit.

## Thread Label

```text
Hope-KB-V108DirectorStyleExpansion【导演风格扩展执行中】
```

## Task

Add enough bounded director-style prototype coverage for the V108 evidence
shape before the main `V108ShotLanguageSelectionContract` is drafted.

This is a KB-only seed expansion gate. It may update KB director-style seed
assets and KB progress docs. It must not modify `E:\codex\hope`, V3, desktop,
intake, Qwen, Seedance, product import behavior, positive few-shot promotion,
or `reference_control_core`.

## Required Inputs

Read:

- `docs/seedance2-v108-director-style-coverage-review-2026-04-23.md`
- `docs/seedance2-v108-field-mapping-review-2026-04-23.md`
- `docs/seedance2-v108-v3-alignment-report-2026-04-23.md`
- `docs/normalized-staging/seedance2-v108-fused-golden-sample.normalized.json`
- `seed/v0.1/director_profiles.json`
- `seed/v0.1/director_rules.json`
- `seed/v0.1/director_reference_sets.json`
- `seed/v0.1/director_scene_affinity.json`
- `seed/v0.1/manifest.json`
- `scripts/validate-seed-bundle.ps1`

Read the Hope control review as read-only evidence:

- `E:\codex\hope\docs\seedance2-v108-director-style-coverage-control-review-2026-04-23.md`

## Expansion Requirements

Add four director-style prototype entries, unless file-level validation shows a
schema reason to split one candidate further:

- `国风武侠 / 仙侠动作调度`
- `国漫史诗群像 / 战争场面调度`
- `都市末世 / 工业压迫调度`
- `国潮舞台 / 原创表演调度`

These are style prototypes, not instructions to imitate specific real
directors. If the existing schema requires a director-name-like display field,
use prototype labels that clearly read as internal style lanes, not real-person
claims.

Update the relevant KB assets consistently:

- `seed/v0.1/director_profiles.json`
- `seed/v0.1/director_rules.json`
- `seed/v0.1/director_reference_sets.json`
- `seed/v0.1/director_scene_affinity.json`
- `seed/v0.1/manifest.json`
- `docs/live-progress.md`

If the repo has a local checklist or README that must reflect manifest counts,
update it only if validation or existing documentation requires it.

## Boundary Rules

Do not:

- import V108 rows into seed examples
- change `seed/v0.2/golden_sample_library.json`
- change v0.2 import maps, snapshot contracts, or normalized staging artifacts
- promote positive few-shot rows
- create `reference_control_core`
- invent external reference handles, URLs, asset IDs, character appearances, or
  scene designs
- add Qwen or Seedance prompts
- change Hope product code, validator code, Rust DTOs, exporter, workbook, IPC,
  desktop, intake, or V3 branch files

The new entries must remain internal KB style lanes for later retrieval and
camera-language selection. They must not be product-facing output.

## Validation

Run the KB seed validation used by the repo, at minimum:

```text
powershell -ExecutionPolicy Bypass -File scripts/validate-seed-bundle.ps1
```

If snapshot build tooling is required by the repo after manifest count changes,
run the established snapshot builder and report the generated path/hash. If the
builder is not required or not available for this gate, state that clearly.

## Output

Commit and push to `origin/codex/contracts-freeze`.

## Report Back To Main Control

Return:

- final branch / commit
- changed files
- director profile count before / after
- director rule count before / after
- director reference set count before / after
- director scene affinity count before / after
- prototype labels added
- validation result
- snapshot result, if run
- unchanged assertions:
  - `rows_ready_for_product_import = 0`
  - `rows_ready_for_positive_fewshot = 0`
  - `product_ready_external_reference_handles = 0`
  - `reference_control_core_coverage = 0`
- confirmation that no Hope / V3 / desktop / intake files were changed
