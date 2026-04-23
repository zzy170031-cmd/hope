# Seedance2 V108 Shot-Language Selection Contract Review 2026-04-23

## Review Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- reviewed candidate commit:
  `b3c684e` (`docs: add V108 shot-language selection contract`)
- reviewed candidate memo:
  `docs/seedance2-v108-shot-language-selection-contract-candidate-2026-04-23.md`
- prior control dispatch:
  `60ce117` (`docs: dispatch V108 shot-language selection`)
- review scope: main-control docs-only acceptance / rejection

This review does not implement runtime selection, final storyboard writing,
prompt assembly, Qwen / Seedance integration, validator behavior, repair
behavior, exporter behavior, workbook shape, IPC, desktop, intake, V3,
`hope-kb` edits, product import, positive few-shot promotion, external
reference handles, or `reference_control_core`.

## Decision

`SEEDANCE2_V108_SHOT_LANGUAGE_SELECTION_CONTRACT_CANDIDATE_ACCEPTED`

Main control accepts the candidate as the current planning contract for V108
shot-language selection and storyboard-word planning signals.

Acceptance is planning-only. It does not authorize implementation, product
prompt text, final storyboard words, runtime prompt templates, Qwen messages,
Seedance overlay wording, exporter rows, workbook fields, UI taxonomy, or
product-facing reference handles.

## Acceptance Findings

The candidate satisfies the dispatch requirements:

- preserves all accepted V108 assertions
- references the accepted KB director-style expansion from `7` to `11` lanes
- defines the required planning objects:
  - `V108ShotLanguageSelectionContract`
  - `V108ShotLanguageSourceBundle`
  - `V108StyleLaneSelectionInput`
  - `V108CameraLanguageSelectionRule`
  - `V108StoryboardWordPlanningSignal`
  - `V108ShotLanguageSelectionBlocker`
  - `V108ShotLanguageSelectionReadiness`
  - `V108ShotLanguageFutureGateOrder`
- gives a conservative selection matrix for original seven lanes and
  `director_08` through `director_11`
- distinguishes internal retrieval metadata, camera-language planning signals,
  final storyboard words, and runtime prompt text
- keeps final storyboard words and runtime prompt text closed
- keeps placeholders, unresolved references, V3 alignment gaps, schema gaps,
  unopened promotion gates, undefined validator evidence, and reserve rows as
  blockers
- keeps Qwen, Seedance, exporter, workbook, desktop, intake, validator, repair,
  and product import out of scope

## Accepted Eleven-Lane Routing Summary

The original seven lanes remain sufficient for broad action, close combat,
crowd pressure, emotion landscape, micro-performance, subjective deformation,
and suspense / transition routing.

The four new KB lanes are accepted as precision lanes only:

- `director_08`: `内部风格通道：国风武侠/仙侠动作调度`
  - use when V108 evidence needs guofeng / wuxia / xianxia action, weapon
    continuity, qinggong movement, bamboo / roof / rain spatial anchors, or
    burst-and-landing rhythm
- `director_09`: `内部风格通道：国漫史诗群像/战争场面调度`
  - use when V108 evidence needs war oath, army formation, command hierarchy,
    battlefield depth, flag / drum bridges, or crowd-motion rhythm
- `director_10`: `内部风格通道：都市末世/工业压迫调度`
  - use when V108 evidence needs subway / overpass / ruins / evacuation,
    infrastructure pressure, alarm layering, or industrial spatial compression
- `director_11`: `内部风格通道：国潮舞台/原创表演调度`
  - use when V108 evidence needs stage axis, performer entrance, synchronized
    group movement, music beat, spotlight timing, formation continuity, or
    final-freeze staging

The new lanes are not defaults. They require concrete source evidence and
remain internal metadata / planning signals only.

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

## Still Blocked

The following blockers remain unresolved:

- `placeholder_present`
- `prompt_body_blocked_by_placeholder`
- `reference_handle_unresolved`
- `v3_alignment_gap`
- `schema_field_missing`
- `promotion_gate_not_accepted`
- `validator_evidence_not_defined`
- `reserve_row`

These blockers prevent product import, positive few-shot, final storyboard
words, runtime prompt text, product-ready references, validator implementation,
repair behavior, exporter changes, and workbook changes.

## Next Gate

Because shot-language selection is accepted only as internal planning metadata,
the next gate is:

```text
Hope-KB-V108CanonicalObjectRegistryReadiness【Reference边界评估中】
```

That gate is KB-only and docs-only. It must analyze V108 reference evidence and
the future canonical object registry boundary before any product-facing
external reference handle selection is considered.

It must not create product-ready handles or `reference_control_core`.

## Validation

This was a docs-only review. No code tests were required or run.
