# Track G Exporter Contract

## Source of Truth

- `core-domain/src/traits.rs` is the only source for `Exporter`, `PromptRenderer`, `LLMProvider`, and `Validator` trait shape.
- `contracts/track-a/excel-17-sheet.md` is the only source for workbook sheet order and Chinese column headers.
- `app/src/ipc.rs` is the only source for IPC command names.

## Scope

Track G only freezes workbook contract, manifest/承载, and the field mapping needed for Excel / JSON / Markdown export.

Fixed inputs:

- `E:\codex\hope\contracts\fixtures\week3-shared-fixture.json`
- `E:\codex\hope\contracts\fixtures\week3-validation-report.json`

Fixed output directory:

- `E:\codex\hope\contracts\fixtures\exports\`

Out of scope:

- new business fields
- schema reversal
- UI bypass of exporter
- any image/video generation

## Workbook Order

The workbook uses the exact 17-sheet canonical order from Track A:

1. `project_meta`
2. `episode_meta`
3. `narrative_scene`
4. `render_segment`
5. `cut`
6. `prompt_package`
7. `handoff_zone`
8. `hard_lock`
9. `stale_event`
10. `validation_report`
11. `export_manifest`
12. `director_profile`
13. `director_cut_sample`
14. `committee_template`
15. `visual_term`
16. `cinematography_term`
17. `continuity_rule`

## Sheet Contract

| # | sheet machine name | 中文展示名 | 中文列头 |
| --- | --- | --- | --- |
| 1 | `project_meta` | 项目元数据 | `项目标识` `标题` `状态` `目标时长分钟` |
| 2 | `episode_meta` | 集元数据 | `集数标识` `项目标识` `序号` `标题` `目标时长分钟` |
| 3 | `narrative_scene` | NarrativeScene | `叙事场景标识` `集数标识` `序号` `标题` `内容摘要` |
| 4 | `render_segment` | RenderSegment | `渲染片段标识` `叙事场景标识` `序号` `起始镜头序号` `结束镜头序号` `目标时长秒` |
| 5 | `cut` | Cut | `镜头标识` `渲染片段标识` `序号` `镜头描述` `对白` `时长秒` |
| 6 | `prompt_package` | PromptPackage | `提示词包标识` `来源层级` `正文` `版本` |
| 7 | `handoff_zone` | handoff zone | `交接带标识` `渲染片段标识` `起始边界` `结束边界` `边界类型` |
| 8 | `hard_lock` | 全局硬锁 | `硬锁标识` `项目标识` `锁名` `锁值` `作用范围` |
| 9 | `stale_event` | 陈旧传播 | `事件标识` `来源层级` `来源标识` `目标层级` `目标标识` `追踪标识` `时间戳` |
| 10 | `validation_report` | 校验报告 | `校验报告标识` `项目标识` `通过` `问题数` `更新时间戳` |
| 11 | `export_manifest` | 导出清单 | `导出清单标识` `项目标识` `工作簿版本` `状态` |
| 12 | `director_profile` | 导演档案 | `导演档案标识` `导演名称` `定位` `主风格` `镜头偏好` |
| 13 | `director_cut_sample` | 导演切样 | `切样标识` `导演档案标识` `样例标题` `样例内容` |
| 14 | `committee_template` | committee 模板 | `模板标识` `模板名称` `适用场景` `成员构成` `职责描述` |
| 15 | `visual_term` | 视觉术语 | `术语标识` `中文术语` `类别` `定义` `别名` |
| 16 | `cinematography_term` | 摄影术语 | `术语标识` `中文术语` `类别` `定义` `别名` |
| 17 | `continuity_rule` | 连续性规则 | `规则标识` `规则名称` `规则说明` `适用层级` |

## Field Mapping

- Excel uses the frozen sheet order above.
- Excel column headers use the frozen Chinese headers and must not add extra business columns.
- JSON mirrors each sheet as an array of row objects, and each row object key must exactly match the Excel Chinese column header.
- Markdown mirrors each sheet as a table, and the column order must exactly match Excel.
- Workbook-level metadata is allowed only as envelope data, not as business schema.

## Shared Source Rule

- `prompt_package`, `cut`, and `validation_report` must all originate from the fixed shared fixture and validation report inputs.
- The exporter must not maintain private sample bundles or exporter-only sample data.
- The three exports must be derived from the same shared workbook manifest.

## PromptPackage Traceability

`prompt_package` is the trace anchor for prompt material.

- `来源层级` identifies where the prompt package came from.
- `正文` is the canonical rendered text.
- `版本` is the freeze boundary for prompt revisions.

## Validation Sheet Mapping

`validation_report` is the workbook carrier for validation status.

- `通过` is the pass signal.
- `问题数` is the failure pressure indicator.
- `更新时间戳` is the last validation update point.

## Minimal Sample Workbook

The minimal sample workbook exported by Track G must include:

- one workbook manifest
- one sample row per sheet
- one JSON preview that preserves the same column keys
- one Markdown preview that preserves the same table headers

This sample is contract-only and does not introduce new business fields.
