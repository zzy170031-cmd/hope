# Golden Sample v0.2 Contract-Freeze Candidate 2026-04-22

## Route Metadata

| Item | Value |
| --- | --- |
| repo | `E:\codex\hope` |
| proposal branch | `codex/v3-field-overlay-proposal` |
| candidate base | `7a08cd6` (`docs: add golden sample v0.2 overlay proposal`) |
| Hope control anchor reviewed | `c076a4c` (`docs: accept KB golden sample v0.2 package`) |
| KB package anchor reviewed | `E:\codex\hope-kb @ 445c452` (`Add golden sample v0.2 schema package`) |
| worktree posture at dispatch | `hope` clean on `codex/contracts-freeze`; `hope-kb` clean on `codex/contracts-freeze` |
| handoff type | docs-only v0.2 contract-freeze candidate |
| implementation scope | no product code, no exporter, no IPC, no Rust DTO, no validator, no desktop, no intake, no workbook, no Qwen/Seedance integration, no hope-kb mutation |

## Current Conclusion

The KB golden sample package is ready to be treated as a v0.2 `golden_sample_library` contract-freeze candidate for planning, validation design, repair design, and future Qwen retrieval, but it is still not authorized for Hope runtime implementation or v0.1 contract changes.

## 2026-04-23 Seedance2 V108 Alignment Refresh

Main control later accepted `E:\codex\hope-kb @ 6f210f0` as the KB-only
Seedance2 V108 sample update readiness package, with Hope control anchor
`142225e3e6d029c2ed83e382c292d3a43cc9ddb0`.

This freeze candidate remains the accepted docs-only record for the earlier
40-row / 17-source-field KB v0.2 package. It is now historical reference for
field-format purposes where V108 differs. The current V108 alignment addendum is:

```text
docs/contracts/export-overlays/seedance2-v108-v3-docs-alignment-refresh-2026-04-23.md
```

Current V108 alignment rules:

- V108 XLSX / DOCX is the canonical source format for this gate.
- The old 40-row package remains unchanged and is not rewritten from V108.
- V108 rows are not imported into current v0.2 sample records.
- V108 rows are not promoted to runtime positive few-shot.
- Conflicts are marked `v3_alignment_gap`, not hard-merged.
- Blank or placeholder surfaces are marked `future_model_fill_surface`.
- `reference_bundle` means only `external_reference_handles`; do not invent
  `reference_control_core`, image paths, URLs, asset IDs, character appearance,
  or scene detail.

No Hope runtime implementation, validator, repair engine, exporter, workbook,
desktop, intake, Qwen, Seedance, or `hope-kb` mutation is authorized by this
refresh.

## Source Snapshots Included

This candidate carries the actual KB v0.2 package evidence in the proposal branch, not only a summary.

| Source | Proposal snapshot path | Reviewed source |
| --- | --- | --- |
| Hope package review | `docs/contracts/export-overlays/source-materials/kb-golden-sample-v0.2-package-review-2026-04-22.md` | `E:\codex\hope\docs\kb-golden-sample-v0.2-package-review-2026-04-22.md` |
| KB validation readiness | `docs/contracts/export-overlays/source-materials/golden-sample-v0.2-validation-readiness-2026-04-22.md` | `E:\codex\hope-kb\docs\golden-sample-v0.2-validation-readiness-2026-04-22.md` |
| Library seed | `docs/contracts/export-overlays/source-materials/golden_sample_library.json` | `E:\codex\hope-kb\seed\v0.2\golden_sample_library.json` |
| Field coverage rules | `docs/contracts/export-overlays/source-materials/golden_sample_field_coverage_rules.json` | `E:\codex\hope-kb\seed\v0.2\golden_sample_field_coverage_rules.json` |
| Failure mapping | `docs/contracts/export-overlays/source-materials/golden_sample_failure_mapping.json` | `E:\codex\hope-kb\seed\v0.2\golden_sample_failure_mapping.json` |
| Repair mapping | `docs/contracts/export-overlays/source-materials/golden_sample_repair_mapping.json` | `E:\codex\hope-kb\seed\v0.2\golden_sample_repair_mapping.json` |
| Source register | `docs/contracts/export-overlays/source-materials/source_register.json` | `E:\codex\hope-kb\seed\v0.2\source_register.json` |

## Package Counts

| Asset | Candidate count |
| --- | ---: |
| `golden_sample_library.records` | 40 |
| `golden_sample_field_coverage_rules.records` | 5 |
| `golden_sample_failure_mapping.records` | 40 |
| `golden_sample_repair_mapping.records` | 40 |
| `source_register.sources` | 3 |
| `source_register.provenance_entries` | 1 |

Coverage distribution:

| Dimension | Candidate distribution |
| --- | --- |
| V3 core coverage | `visual_scene_core=8`, `motion_performance_core=8`, `camera_directing_core=8`, `audio_directing_core=8`, `continuity_lock_core=8` |
| tier | `优=10`, `好=10`, `中=10`, `差=10` |
| few-shot eligibility | `eligible=true=30`, `eligible=false=10` |
| source preservation | all 40 source rows and all 17 source fields preserved |
| source CSV hash | `2d0ce22f8c85ca59de6abf35a48239aadef9a2bd01bb3f7b4685bf80e06bf843` |

## Three-Layer Model Split

| Layer | Candidate field | Meaning |
| --- | --- | --- |
| content generation | `content_generation_model` | Qwen/千问 remains Hope's text generation engine. Golden samples may later support Qwen few-shot retrieval and output constraint, but no Qwen connection is authorized now. |
| target video model | `target_video_model` | Seedance 2.0 remains the downstream video execution target. Golden samples shape Seedance-ready content expectations, but do not call Seedance. |
| adapter profile | `adapter_profile` | Future Seedance adapter compiles structured Hope output into prompt/payload. Golden samples can later help validate adapter readiness, but are not adapter payload fields. |

## Final Candidate Field List: `golden_sample_library`

The v0.2 candidate keeps the actual KB package shape. The 17 original source fields remain nested under `source_fields` and must not be flattened into v0.1 `classic_case_examples`.

| field_name | 中文名/含义 | required? | source / derived | future consumer | notes |
| --- | --- | --- | --- | --- | --- |
| `machine_id` | 机器稳定 ID | yes | derived | KB / validator | Example: `golden_sample_gs_7`. Stable row key for fixtures. |
| `sample_id` | 源样本 ID | yes | source mirror | KB / Qwen / validator / repair | Example: `GS-7`. Mirrors `source_fields.sample_id`. |
| `schema_version` | schema 版本 | yes | package | governance | Candidate value: `golden_sample_library.v0.2`. |
| `source_fields` | 原始 17 字段容器 | yes | source | provenance / all downstream planning | Lossless source field envelope. |
| `source_fields.sample_title` | 样本标题 | yes | source | Qwen retrieval / reviewer trace | Human title. |
| `source_fields.core` | V3 core 轴 | yes | source | Qwen / validator | One of the covered V3 core values. |
| `source_fields.covered_points` | 已覆盖要点 | yes | source | validator / repair | Positive coverage evidence. |
| `source_fields.created_at` | 源创建时间 | yes | source | provenance | Preserve source timestamp. |
| `source_fields.director_voice` | 内部导演口吻标签 | yes | source | Qwen retrieval | Internal guidance only. Do not export imitation language. |
| `source_fields.empty_words_detected` | 空泛词检测 | yes | source | validator / negative sample / repair | Empty-language signal. |
| `source_fields.genre` | 类型标签 | yes | source | Qwen retrieval | Preserve source labels. |
| `source_fields.missed_points` | 缺失要点 | yes | source | validator / repair | Negative and repair evidence. |
| `source_fields.sample_id` | 源样本 ID | yes | source | provenance | Source row ID. |
| `source_fields.scene_tag` | 场景标签 | yes | source | Qwen retrieval / validator | Preserve source labels. |
| `source_fields.shot_type` | 景别/镜头类型 | yes | source | Qwen retrieval / V3 coverage | Observed values include `CU`, `LS`, `MCU`, `MLS`, `MS`, `OTS`. |
| `source_fields.source_cut` | 来源镜头 | yes | source | provenance | Empty values are intentional and must remain empty. |
| `source_fields.teaching_note` | 教学说明 | yes | source | Qwen / validator / repair | Explains why the sample is useful, weak, or negative. |
| `source_fields.tier` | 样本档位 | yes | source | Qwen / validator | Candidate distribution: `优/好/中/差`, 10 each. |
| `source_fields.updated_at` | 源更新时间 | yes | source | provenance | Preserve source timestamp. |
| `source_fields.usable_for_fewshot` | 是否可用于 few-shot | yes | source | Qwen retrieval | Preserve source value while also deriving boolean eligibility. |
| `source_fields.word_count` | 字数 | yes | source | validator | Preserve source count. |
| `provenance` | 溯源容器 | yes | package | audit / fixtures | Required for source proof. |
| `provenance.source_csv` | 源 CSV 路径 | yes | source register | provenance | Canonical KB export path. |
| `provenance.source_sha256` | 源 CSV hash | yes | source register | provenance | `2d0ce22f8c85ca59de6abf35a48239aadef9a2bd01bb3f7b4685bf80e06bf843`. |
| `provenance.source_row_index` | 源行号 | yes | package | provenance | Source-row audit. |
| `provenance.normalized_staging_artifact` | normalized staging artifact | yes | package | provenance | Links back to normalized staging JSON. |
| `provenance.source_created_at` | 源创建时间镜像 | yes | source | provenance | Mirrors source timestamp. |
| `provenance.source_updated_at` | 源更新时间镜像 | yes | source | provenance | Mirrors source timestamp. |
| `classification` | 分类归一容器 | yes | derived | retrieval / filtering | Safe derived view of source fields. |
| `classification.core` | 归一 V3 core | yes | derived from source | Qwen retrieval / validator | Matches `source_fields.core`. |
| `classification.tier` | 归一档位 | yes | derived from source | Qwen retrieval / validator | Supports positive/negative selection. |
| `classification.director_voice` | 归一内部口吻 | yes | derived from source | Qwen retrieval | Internal only. |
| `classification.genre_tags` | 类型标签数组 | yes | derived from source | Qwen retrieval | Parsed retrieval tags. |
| `classification.scene_tags` | 场景标签数组 | yes | derived from source | Qwen retrieval | Parsed retrieval tags. |
| `classification.shot_type` | 归一镜头类型 | yes | derived from source | Qwen retrieval | Supports shot-type filtering. |
| `classification.usable_for_fewshot` | few-shot 布尔值 | yes | derived from source | Qwen retrieval | Candidate distribution: true 30, false 10. |
| `fewshot` | few-shot 选择容器 | yes | derived | Qwen retrieval | Future retrieval gate only. |
| `fewshot.eligible` | 是否可入正向 few-shot | yes | derived | Qwen retrieval | True rows can be candidates; false rows must not be positive prompt context. |
| `fewshot.source_value` | 源 few-shot 值 | yes | source mirror | provenance | Preserves original source label. |
| `fewshot.retrieval_status` | 检索状态 | yes | derived | Qwen retrieval | Example status: `candidate_positive_fewshot`. |
| `validator_evidence` | validator 证据容器 | yes | derived from source | validator / repair | Future validator input, not implementation. |
| `validator_evidence.covered_points` | 已覆盖证据 | yes | source mirror | validator | Must remain text-preserving. |
| `validator_evidence.missed_points` | 缺失证据 | yes | source mirror | validator / repair | Drives future repair hints. |
| `validator_evidence.empty_words_detected` | 空泛词证据 | yes | source mirror | validator / repair | Drives empty-word failure signal. |
| `validator_evidence.word_count` | 字数证据 | yes | source mirror | validator | Future quantitative check. |
| `validator_evidence.validator_target` | validator 目标 | yes | derived | validator | Example: `Visual Scene Core Validator`. |
| `validator_evidence.has_coverage_gap` | 是否有覆盖缺口 | yes | derived | validator | Boolean fixture signal. |
| `validator_evidence.has_empty_word_signal` | 是否有空泛词信号 | yes | derived | validator | Boolean fixture signal. |
| `negative_sample` | 负样本容器 | yes | derived | validator / repair / Qwen retrieval exclusion | Separates weak rows from positive examples. |
| `negative_sample.is_negative_sample` | 是否负样本 | yes | derived | validator / repair | 10 records carry negative-sample signal. |
| `negative_sample.signal_codes` | 负样本信号码 | yes | derived | validator / repair | Derived from tier/few-shot/empty/missed evidence. |
| `v3_core_coverage` | V3 core 覆盖容器 | yes | derived | V3/V4 overlay / validator | Connects samples to V3 core axes. |
| `v3_core_coverage.core` | 覆盖 core | yes | derived | V3/V4 overlay | Mirrors covered core. |
| `v3_core_coverage.coverage_rule_id` | coverage rule ID | yes | derived | validator planning | Links to `golden_sample_field_coverage_rules`. |
| `v3_core_coverage.source_coverage_statement` | 源覆盖说明 | yes | derived/source | validator planning | Human-readable coverage statement. |
| `repair_mapping_planning` | repair 映射规划容器 | yes | derived | repair planning | Planning only. |
| `repair_mapping_planning.failure_mapping_id` | failure mapping ID | yes | derived | validator / repair | Links to failure mapping record. |
| `repair_mapping_planning.repair_mapping_id` | repair mapping ID | yes | derived | repair | Links to repair mapping record. |
| `repair_mapping_planning.planning_only` | 仅规划标记 | yes | package | governance | Must remain true until implementation gate. |

## Companion Candidate Assets

### `golden_sample_field_coverage_rules`

| field_name | Meaning | Candidate status |
| --- | --- | --- |
| `rule_id` | Stable coverage rule ID | required |
| `core` | Covered V3 core | required |
| `row_count` | Number of samples for the core | required, observed as 8 per covered core |
| `source_sample_ids` | Source sample IDs under this rule | required |
| `required_source_fields` | Required 17 source fields | required, all 17 preserved |
| `validator_evidence_fields` | Fields usable as future validator evidence | required: `covered_points`, `missed_points`, `empty_words_detected`, `word_count`, `teaching_note` |
| `fewshot_gate` | Positive/negative few-shot selection rule | required |
| `negative_sample_gate` | Negative-sample rule | required |
| `coverage_summary` | Per-core tier/few-shot summary | required |
| `future_gate_owner` | Future downstream owners | required, planning only |
| `planning_only` | Implementation lock | required, true |

### `golden_sample_failure_mapping`

| field_name | Meaning | Candidate status |
| --- | --- | --- |
| `mapping_id` | Stable failure mapping ID | required |
| `sample_id` | Source sample ID | required |
| `core` | V3 core | required |
| `tier` | Sample tier | required |
| `usable_for_fewshot` | Source few-shot value | required |
| `negative_sample_signal` | Negative-sample signal | required |
| `planned_failure_codes` | Future failure codes | required |
| `validator_evidence` | Source evidence snapshot | required |
| `source_field_refs` | Referenced source fields | required |

Failure code definitions in the candidate package:

```text
golden_coverage_gap
golden_empty_word_noise
golden_negative_sample
```

### `golden_sample_repair_mapping`

| field_name | Meaning | Candidate status |
| --- | --- | --- |
| `mapping_id` | Stable repair mapping ID | required |
| `sample_id` | Source sample ID | required |
| `core` | V3 core | required |
| `repair_planning_mode` | Future repair mode | required |
| `linked_failure_mapping_id` | Linked failure mapping | required |
| `planned_repair_inputs` | Repair input fields | required |
| `future_repair_gate` | Future repair gate owner | required |
| `planning_only` | Implementation lock | required, true |

Common planned repair inputs:

```text
missed_points
empty_words_detected
teaching_note
director_voice
shot_type
```

### `source_register`

| field_name | Meaning | Candidate status |
| --- | --- | --- |
| `source_id` | Stable source ID | required |
| `source_type` | Source type | required |
| `label` | Human label | required |
| `path` | Source path | required |
| `sha256` | Source hash | required |
| `applies_to` | Assets covered by the source | required |
| `notes` | Source notes | required |
| `provenance_id` | Provenance entry ID | required |
| `source_ids` | Sources included in provenance entry | required |
| `generated_files` | Generated package files | required |
| `preservation_contract` | Source preservation contract | required |

Required preservation contract:

```text
preserves_all_source_rows: true
preserves_all_17_source_fields: true
imports_into_v0_1_snapshot: false
planning_only_until_future_hope_gate: true
```

## Relationship To V3 Core

This candidate supports the V3 field contract by giving Qwen/validator planning a row-level evidence library for five V3 core axes:

```text
visual_scene_core
motion_performance_core
camera_directing_core
audio_directing_core
continuity_lock_core
```

Each covered core has 8 records with a balanced tier distribution. These records should guide future generation quality, validator evidence, negative sample checks, and repair planning.

`reference_control_core` remains uncovered. This is still an open V3 field gap and must not be backfilled by invention. If full six-core V3 freeze requires golden-sample coverage for reference control, main control should dispatch a future intake/schema extension before treating six-core sample coverage as complete.

## Relationship To V4 Overlay

V4 remains the structured output and export overlay shape. The golden sample candidate does not become a V4 payload schema and does not become a Seedance adapter contract.

Future relationship:

| V4 / overlay layer | Golden sample role |
| --- | --- |
| Canonical Shot Spec | Provides row-level examples and validator evidence for Qwen-generated core content. |
| V4 atomic fields | `covered_points`, `missed_points`, and `teaching_note` may later map to atomic validator expectations. |
| Seedance-ready export overlay | Samples help constrain content quality before adapter compile; samples are not exported as user-facing Seedance fields. |
| Adapter profile | May later consume validation results, not raw golden rows. |

## Relationship To Future Validator

The candidate is validator-ready as planning input only.

Future validator uses may include:

- `covered_points` and `missed_points` as core coverage evidence
- `empty_words_detected` as empty-language signal
- `tier` and `usable_for_fewshot` as positive/negative selection evidence
- `word_count` as quantitative evidence
- `source_sha256` and `source_row_index` as fixture traceability
- `golden_coverage_gap`, `golden_empty_word_noise`, and `golden_negative_sample` as expected failure-code families

No Hope validator implementation is authorized in this candidate.

## Relationship To Future Repair

The candidate is repair-ready as planning input only.

Future repair uses may include:

- using `missed_points` to propose missing content additions
- using `empty_words_detected` to remove empty or vague language
- using `teaching_note` as a human-readable repair rationale
- using `director_voice` only as internal retrieval context, not as final imitation language
- linking `failure_mapping_id` to `repair_mapping_id`

No repair engine, repair DTO, desktop repair UI, or exporter behavior is authorized in this candidate.

## Relationship To Future Qwen Retrieval

The candidate supports future Qwen retrieval design but does not connect Qwen.

Future retrieval rules:

- positive few-shot context can only use rows where `fewshot.eligible=true`
- rows where `fewshot.eligible=false` or `negative_sample.is_negative_sample=true` must be excluded from positive prompt context
- retrieval can filter by `core`, `tier`, `genre_tags`, `scene_tags`, `shot_type`, and internal `director_voice`
- `director_voice` remains an internal KB retrieval label and must not be emitted as "imitate a named director" output text
- retrieval must preserve source provenance when rows are used as fixtures or examples

## Why This Does Not Enter v0.1

Do not import golden samples into v0.1 `classic_case_examples`.

Reasons:

- v0.1 does not preserve the 17 source fields without loss
- v0.1 cannot represent row-level few-shot eligibility and negative-sample signals cleanly
- v0.1 cannot preserve coverage/failure/repair mappings as first-class candidate assets
- v0.1 cannot carry the new source register and provenance contract without opening a schema gate
- v0.1 17-sheet workbook is frozen and must remain unchanged

## Future Gate Order

This candidate can only become product-consumable after explicit future gates, in this order:

1. KB v0.2 source/register freeze review.
2. KB v0.2 import map, migration/table definition, manifest, and content hash.
3. KB v0.2 snapshot builder or explicit versioned builder mode.
4. Hope validator contract gate for golden sample evidence and expected failure codes.
5. Hope repair contract gate for failure-to-repair mapping behavior.
6. Hope exporter/Excel overlay gate for additive v0.2 export/debug metadata only.
7. Hope desktop readonly provenance/readiness gate.
8. Hope intake/Qwen retrieval contract gate.
9. Seedance adapter gate, only after structured Hope output and validator readiness are frozen.

All gates above are future work. None are authorized by this candidate commit.

## Explicit Prohibitions

This candidate still prohibits:

- changing the frozen v0.1 contract
- changing the frozen v0.1 17-sheet workbook
- changing exporter code
- changing IPC
- changing Rust DTOs
- changing validators
- changing desktop business UI
- changing intake
- connecting real Qwen
- connecting real Seedance
- modifying `E:\codex\hope-kb`
- modifying `hope-kb` seed bundles from this branch
- merging `hope-kb` into `hope`
- importing golden samples into v0.1 `classic_case_examples`
- treating this document as implementation authorization

## Difference From `7a08cd6`

`7a08cd6` established the first golden-sample v0.2 overlay proposal from normalized staging.

This candidate adds:

- the accepted KB v0.2 schema package review snapshot
- the KB validation readiness snapshot
- actual v0.2 seed package snapshots for library, coverage rules, failure mapping, repair mapping, and source register
- the final candidate field list based on the real package shape, not only the earlier proposed schema
- explicit relationship to V3 core, V4 overlay, validator, repair, and Qwen retrieval
- explicit confirmation that `reference_control_core` is still uncovered
- explicit future Hope validator/export/desktop/intake gate order
- explicit freeze-candidate archive posture

## Open Items

| Item | Status | Required future action |
| --- | --- | --- |
| `reference_control_core` golden-sample coverage | still open | Future intake batch or schema extension if six-core sample coverage is required. |
| v0.2 snapshot import readiness | still open | Add v0.2 import map, migration, manifest/hash, and builder/versioned builder gate. |
| Hope validator contract | still closed | Future contract gate only. |
| Hope export/desktop/intake implementation | still closed | Future scope gate only. |
| Qwen/Seedance integration | still closed | Future integration gates only. |

## Archive Readiness

This thread can be archived as:

```text
Hope-V3-field-thread-GoldenSampleContract-v0.2-Freeze-candidate-pushed-archive-standby
```

Archive condition: main control accepts that this is a docs-only v0.2 freeze candidate with the known `reference_control_core` coverage gap preserved as an unresolved future intake/gate item.

## Verification Commands

```powershell
git -C E:\codex\hope status --short --branch
git -C E:\codex\hope log -1 --oneline --decorate
```
