# Golden Sample v0.2 Schema Overlay Proposal 2026-04-22

## Route Metadata

| Item | Value |
| --- | --- |
| repo | `E:\codex\hope` |
| proposal branch | `codex/v3-field-overlay-proposal` |
| proposal base | `origin/codex/v3-field-overlay-proposal @ 13f8d7d` |
| source review memo | `docs/contracts/export-overlays/source-materials/kb-golden-sample-intake-review-2026-04-22.md` |
| source schema note | `docs/contracts/export-overlays/source-materials/golden-sample-v0.2-schema-note-2026-04-22.md` |
| normalized source artifact | `docs/contracts/export-overlays/source-materials/golden-sample-library-642bd7876e0d4649814f35fb133b67f3-2026-04-22.normalized.json` |
| handoff type | docs-only v0.2 schema overlay proposal |
| implementation scope | no product code, no exporter, no IPC, no Rust DTO, no validator, no desktop UI, no workbook, no Qwen/Seedance integration, no hope-kb seed mutation |

## Current Conclusion

The golden sample intake result should become an independent v0.2 `golden_sample_library` schema proposal. It must not be flattened into the frozen v0.1 `classic_case_examples` seed table, because the source rows carry few-shot eligibility, negative-sample signals, validator coverage, V3 core coverage, teaching notes, source timestamps, and per-row provenance that v0.1 cannot preserve without loss.

## 2026-04-23 Seedance2 V120 Source Format Addendum

This 2026-04-22 proposal describes the earlier 40-row / 17-source-field golden
sample intake shape. After main-control acceptance of V120 as the next full KB
ingest source, the V120 workbook / control memo becomes the latest canonical
source format for new sample alignment work and V108 becomes the comparison
baseline.

The V120/V108 23-field source shape is not silently merged into this old
proposal. It is recorded as a docs-only alignment addendum at:

```text
docs/contracts/export-overlays/seedance2-v120-v3-docs-alignment-refresh-2026-04-23.md
```

V120 source-format rules that supersede or constrain this proposal:

- field count remains `23`, so V120 changes source priority and row/count
  emphasis rather than adding or removing source columns
- `shot_id` remains the source row identifier; old `sample_id` mappings are
  historical until a future import gate defines the bridge
- `quality_grade` remains the source grade; old `tier` remains historical for
  the 40-row v0.2 package
- `sample_type` still means `single_shot` or `sequence_shot`; it must not reuse
  old sample role meanings such as positive, negative, or repair
- `scene_performance_core` still merges the old `visual_scene_core` and
  `motion_performance_core` axes
- `continuity_negative_core` still combines continuity and negative constraints
- `reference_bundle` still maps only toward `external_reference_handles`
- `prompt_body` is still source prompt body / candidate text, not compiled
  Seedance output
- V120 expands reviewed source rows from `115` to `120`, grows `reserve` rows
  from `7` to `12`, and adds `CNSEQ04`, `CNSEQ05`, and `CNSEQ06`

Rows or fields with blank, placeholder, or pending-fill content are marked
`future_model_fill_surface`. They must not be guessed, imported as completed
content, or used as positive few-shot. This addendum keeps the original v0.1
prohibition intact: do not flatten either the 40-row package or V120 into
`classic_case_examples`. Exact V120 field-level placeholder counts remain a
future KB ingest recomputation task, not a V3 guess-fill task.

## Source Snapshot Summary

The normalized staging artifact reviewed by control contains:

| Item | Value |
| --- | --- |
| artifact_id | `golden_sample_library_normalized_staging_2026_04_22` |
| source repo | `hope-kb` |
| source branch | `codex/contracts-freeze` |
| source CSV | `docs/source-exports/golden-sample-library-642bd7876e0d4649814f35fb133b67f3-2026-04-22.csv` |
| source CSV hash | `sha256:2d0ce22f8c85ca59de6abf35a48239aadef9a2bd01bb3f7b4685bf80e06bf843` |
| normalized_at | `2026-04-22` |
| row_count | `40` |
| source field count | `17` |
| imported_into_v0_1_seed | `false` |
| preserves_all_source_rows | `true` |
| preserves_all_source_fields | `true` |
| preserves_empty_source_cut_values | `true` |
| does_not_invent_missing_content | `true` |

Coverage from the normalized artifact:

| Dimension | Counts |
| --- | --- |
| core | `visual_scene_core=8`, `motion_performance_core=8`, `camera_directing_core=8`, `audio_directing_core=8`, `continuity_lock_core=8` |
| tier | `优=10`, `好=10`, `中=10`, `差=10` |
| usable_for_fewshot | `Yes=30`, `No=10` |
| director_voice | `宫崎骏系`, `新海诚系`, `通用` |
| shot_type | `CU`, `LS`, `MCU`, `MLS`, `MS`, `OTS` |

## Existing Source Fields To Preserve

The v0.2 schema must preserve all 17 source fields exactly as source data:

```text
sample_title
core
covered_points
created_at
director_voice
empty_words_detected
genre
missed_points
sample_id
scene_tag
shot_type
source_cut
teaching_note
tier
updated_at
usable_for_fewshot
word_count
```

The schema must also preserve source CSV hash and import provenance:

```text
source_csv
source_sha256
normalized_artifact_id
normalized_at
row_index
stable_sample_id
kb_staging_status
v0_1_seed_mapping
preservation_contract
```

## Proposed v0.2 Schema

This schema is proposal-only. It is not a current runtime table and must not be added to the v0.1 seed bundle in this branch.

| field_name | 中文名/含义 | type | required? | source / derived | future_consumer | notes |
| --- | --- | --- | --- | --- | --- | --- |
| `sample_id` | 源样本 ID | string | yes | source | KB / Qwen / validator | Preserve source value such as `GS-7`. |
| `stable_sample_id` | 稳定样本 ID | string | yes | normalized | KB / validator | Preserve normalized stable ID; may equal source `sample_id`. |
| `row_index` | 源行序号 | integer | yes | normalized | provenance | Required for source audit. |
| `sample_title` | 样本标题 | string | yes | source | Qwen / desktop debug | Human-readable retrieval title. |
| `sample_text` | 样本文本正文 | text | v0.2 required when available | future source | Qwen / validator / repair | Not present in current 17-field artifact; future gate should add without discarding current fields. |
| `core` | V3 core 轴 | enum | yes | source | Qwen / validator | Values observed: `visual_scene_core`, `motion_performance_core`, `camera_directing_core`, `audio_directing_core`, `continuity_lock_core`. |
| `covered_points` | 已覆盖要点 | text | yes | source | validator | First-class validator evidence. |
| `missed_points` | 缺失要点 | text | yes | source | validator / repair | Negative and repair evidence. |
| `tier` | 档位 | enum | yes | source | Qwen / validator | Preserve `优/好/中/差`; do not normalize away source label. |
| `usable_for_fewshot` | 是否可用于 few-shot | boolean/string | yes | source | Qwen retrieval | Preserve source `Yes/No`; future normalized boolean may be derived. |
| `empty_words_detected` | 检测到的空洞词 | text | yes | source | validator / negative samples | Negative-sample signal; must not be dropped. |
| `director_voice` | 内部口吻/导演风格标签 | string | yes | source | KB retrieval only | Internal retrieval hint; must not be exported as "imitate director" prompt text. |
| `genre` | 类型/风格标签 | string/list | yes | source | Qwen retrieval | Source comma labels preserved. |
| `scene_tag` | 场景标签 | string/list | yes | source | Qwen retrieval / validator coverage | Source comma labels preserved. |
| `shot_type` | 景别/镜头类型 | string | yes | source | Qwen retrieval / V3 coverage | Observed values include `CU/LS/MCU/MLS/MS/OTS`. |
| `source_cut` | 来源镜头 | string/null | yes | source | provenance | Empty source values are intentional and must stay empty. |
| `teaching_note` | 教学说明 | text | yes | source | Qwen / validator / repair | Explains why a row is good/bad/useful. |
| `word_count` | 字数 | integer/string | yes | source | validator | Preserve source value; future normalized numeric field may be derived. |
| `created_at` | 源创建时间 | string/datetime | yes | source | provenance | Preserve source timestamp format. |
| `updated_at` | 源更新时间 | string/datetime | yes | source | provenance | Preserve source timestamp format. |
| `source_csv` | 源 CSV 路径 | string | yes | artifact | provenance | From normalized artifact. |
| `source_sha256` | 源 CSV hash | string | yes | artifact | provenance | `sha256:2d0ce22f8c85ca59de6abf35a48239aadef9a2bd01bb3f7b4685bf80e06bf843`. |
| `normalized_artifact_id` | normalized artifact ID | string | yes | artifact | provenance | From staging JSON. |
| `normalized_at` | normalized 日期 | string/date | yes | artifact | provenance | From staging JSON. |
| `kb_staging_status` | KB staging 状态 | enum | yes | normalized | governance | Preserve `normalized_only_not_seed_imported`. |
| `v0_1_seed_mapping` | v0.1 映射决策 | enum | yes | normalized | governance | Preserve `blocked_lossy_classic_case_schema`. |
| `imported_into_v0_1_seed` | 是否导入 v0.1 seed | boolean | yes | artifact | governance | Must remain false for this staging artifact. |
| `preserves_empty_source_cut_values` | 是否保留空 source_cut | boolean | yes | artifact | provenance | Guards against invented source links. |
| `source_fields_json` | 原始 17 字段快照 | object | recommended | derived copy | provenance | Lossless storage for future import; source 17 fields remain first-class too. |
| `usable_for_validator` | 可用于 validator | boolean | v0.2 derived | derived | validator | Derived from tier/covered/missed/empty words; not present in source. |
| `usable_for_repair` | 可用于 repair | boolean | v0.2 derived | derived | repair | Derived from missed_points/teaching_note/tier. |
| `sample_type` | 样本类型 | enum | v0.2 derived | derived | Qwen / validator / repair | Proposed values: `positive`, `negative`, `repair_before`, `repair_after`, `coverage_reference`. |
| `sample_usage` | 样本用途 | enum/list | v0.2 derived | derived | Qwen / validator / repair | Proposed values: `fewshot`, `validator`, `repair`, `regression`, `negative_sample`. |
| `negative_sample_signal` | 负样本信号 | string/list | v0.2 derived | derived | validator / repair | Derived from `tier=差`, `usable_for_fewshot=No`, and `empty_words_detected`. |
| `v3_core_coverage_profile` | V3 core coverage profile | object/string | v0.2 derived | derived | validator | Parses `covered_points` and `missed_points` without losing source text. |
| `generation_stage` | 生成阶段 | enum | v0.2 derived | derived | Qwen orchestration | Proposed values: `story`, `screenplay`, `cut`, `export`, `validation`. |
| `v4_profile` | V4 profile | enum | v0.2 derived | derived | Qwen orchestration | Proposed values: `story_guard`, `screenplay_guard`, `cut_guard`, `export_contract`, `validation_profile`. |
| `retrieval_tags` | 检索标签 | string/list | v0.2 derived | derived | Qwen retrieval | Derived from core/tier/genre/scene_tag/shot_type/director_voice. |
| `source_ids` | 来源 ID 列表 | string/list | future | future | provenance | Future source register bridge; not required in current artifact. |
| `validator_expected_result` | 预期校验结果 | enum/object | future | future | validator | Future validator fixture field. |
| `expected_failure_codes` | 预期失败码 | string/list | future | future | validator / repair | Future negative/regression mapping. |
| `repair_template_id` | 修复模板 ID | string | future | future | repair | Future repair mapping. |
| `repair_hint` | 修复提示 | text | future | future | repair | Can initially derive from `teaching_note`. |
| `negative_boundary_marker` | 负向边界标记 | string/list | future | future | repair | Prevents repair from rewriting unrelated layers. |

## Why Not v0.1 classic_case_examples

The current v0.1 `classic_case_examples` table is broad case metadata. The golden sample library is row-level training, coverage, negative, and teaching data.

Do not import this artifact into v0.1 because it would flatten or discard:

- per-core V3 coverage
- `covered_points`
- `missed_points`
- tier distribution
- few-shot eligibility
- empty-word negative-sample signals
- director voice retrieval signal
- genre / scene / shot type retrieval tags
- intentional empty `source_cut`
- teaching notes
- source timestamps
- source CSV hash and import provenance

## Relationship To Existing V3/V4 Overlay

This document extends `v3-field-overlay-proposal-2026-04-22.md` with a dedicated golden-sample schema overlay.

| Existing V3/V4 concept | Golden sample role |
| --- | --- |
| `visual_scene_core` | 8 staged rows: positive, mid, and negative evidence for visual coverage. |
| `motion_performance_core` | 8 staged rows for motion/performance density and failure contrast. |
| `camera_directing_core` | 8 staged rows for camera coverage and missing-point evidence. |
| `audio_directing_core` | 8 staged rows for audio sync, sound layer, silence, and negative signals. |
| `continuity_lock_core` | 8 staged rows for continuity lock coverage and negative examples. |
| V4 atomic fields | Future derived coverage fields may map teaching notes and covered/missed points to atomic validators. |
| `content_generation_model=Qwen` | Future retrieval can choose positive rows for generation context; no implementation now. |
| `target_video_model=Seedance 2.0` | Samples guide Seedance-ready content shape, but they do not call Seedance. |
| `adapter_profile` | Future validator may check whether sample-guided output compiles; no implementation now. |

Important: current normalized artifact does not include `reference_control_core` rows. If future V3/V4 proposal requires reference-control sample coverage, that must be a new intake batch or a future v0.2 schema extension, not invented here.

## Future Service Boundaries

### Future Qwen Content Generation

Allowed only after a future implementation gate:

- retrieve positive few-shot rows by `core + tier + genre + scene_tag + shot_type`
- exclude `usable_for_fewshot=No` from positive prompt context
- use `teaching_note` to explain why a sample is strong or weak
- keep `director_voice` internal and never emit "imitate specific director" in final output

Current proposal action: planning only.

### Future Validator

Allowed only after a future implementation gate:

- use `covered_points` and `missed_points` as validator evidence
- use `empty_words_detected` as negative-sample and empty-language signal
- compare generated core content against tier-specific coverage expectations
- use source hash/provenance for fixture traceability

Current proposal action: planning only.

### Future Repair

Allowed only after a future implementation gate:

- convert `missed_points` and `teaching_note` into repair hints
- use negative rows to detect and rewrite empty words
- map future `expected_failure_codes` to repair templates
- respect future `negative_boundary_marker` so repair does not rewrite unrelated layers

Current proposal action: planning only.

## Future hope-kb Work, Not For This Branch

Future v0.2 KB gate may add:

```text
seed/v0.2/golden_sample_library.json
seed/v0.2/golden_sample_field_coverage_rules.json
seed/v0.2/golden_sample_failure_mapping.json
seed/v0.2/golden_sample_repair_mapping.json
source_register entries for the source CSV and normalized staging artifact
schema validation for the 17 preserved source fields
```

Not allowed in this branch:

- editing `hope-kb`
- editing `hope-kb` seed bundle
- editing `hope-kb` manifest/import map
- building a runtime snapshot from these rows

## Future Hope Product Work, Not For This Branch

Future v0.2 Hope gate may add:

```text
golden_sample_refs on generated V3/V4 core outputs
validator checks backed by golden sample coverage
repair recommendations backed by negative rows and missed_points
desktop read-only sample provenance display
export/debug metadata for source sample IDs
```

Not allowed in this branch:

- Rust DTO changes
- validator implementation
- exporter or workbook changes
- desktop UI changes
- IPC changes
- Qwen connection
- Seedance connection

## Explicit Prohibitions

This docs-only proposal does not authorize:

- importing golden samples into v0.1 `classic_case_examples`
- changing the frozen v0.1 17-sheet workbook
- changing exporter code or workbook columns
- changing IPC
- changing Rust DTOs or validators
- changing desktop UI
- connecting real Qwen
- connecting real Seedance
- changing `hope-kb` seed bundle
- merging `hope-kb` into `hope`
- treating this proposal as a frozen runtime contract

## Recommended Control-Thread Summary

```text
Golden sample intake should remain KB-only staging and become a v0.2 golden_sample_library overlay proposal. It preserves all 40 rows, all 17 source fields, source CSV hash/provenance, V3 core coverage, few-shot eligibility, negative-sample signals, teaching notes, and timestamps. It must not be imported into v0.1 classic_case_examples. Future use is Qwen few-shot, validator evidence, repair mapping, and V3/V4 core coverage, but no implementation is authorized now.
```

## Verification Commands

```powershell
git -C E:\codex\hope status --short --branch
git -C E:\codex\hope log -1 --oneline --decorate
```
