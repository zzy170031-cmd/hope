# Hope V3/V4 Field Overlay Proposal Handoff 2026-04-22

## Route Metadata

| Item | Value |
| --- | --- |
| repo | `E:\codex\hope` |
| base branch | `codex/contracts-freeze` |
| proposal branch | `codex/v3-field-overlay-proposal` |
| anchor commit | `c3d6142 (HEAD -> codex/contracts-freeze, origin/codex/contracts-freeze) docs: accept seedance v1 as v0.2 export overlay proposal` |
| start worktree state | clean: `git status --short --branch` returned `## codex/contracts-freeze...origin/codex/contracts-freeze` |
| handoff type | docs-only proposal |
| implementation scope | no code, no Rust DTO, no exporter, no IPC, no desktop implementation, no hope-kb mutation |

## Current Conclusion

Hope uses Qwen/千问 to generate structured storyboard truth; V3/V4 define the Seedance-ready field overlay that constrains Qwen output and later compiles to Seedance 2.0 prompt/payload, but this proposal must not replace the frozen Hope v0.1 17-sheet workbook contract.

## 2026-04-23 Seedance2 V108 Alignment Refresh

Main control accepted `E:\codex\hope-kb @ 6f210f0` as the KB-only Seedance2
V108 sample update readiness package at Hope anchor
`142225e3e6d029c2ed83e382c292d3a43cc9ddb0`. From this point, the local V108
XLSX / DOCX table format is the latest canonical source format for sample and
field alignment.

This 2026-04-22 field overlay remains historical proposal reference. Where it
conflicts with V108, use
`docs/contracts/export-overlays/seedance2-v108-v3-docs-alignment-refresh-2026-04-23.md`
as the current docs-only alignment addendum.

V108-specific alignment changes:

- `sample_type` now means `single_shot` or `sequence_shot`; it must not be
  confused with the old golden-sample sample-category meaning.
- `scene_performance_core` fuses the old `visual_scene_core` and
  `motion_performance_core` source axes.
- `continuity_negative_core` fuses continuity locks with negative constraints
  and needs a future split/retention decision.
- `reference_bundle` is only source input for `external_reference_handles`, not
  image paths, URLs, asset IDs, or completed `reference_control_core`.
- `prompt_body` is source prompt text / candidate text, not compiled Seedance
  adapter output.
- Blank and placeholder surfaces are preserved as `future_model_fill_surface`;
  they must not be guessed, backfilled, or promoted into positive few-shot.

No Hope product implementation, exporter, Rust DTO, validator, desktop, intake,
Qwen, Seedance, workbook, or `hope-kb` mutation is authorized by this refresh.

## Source Materials Included

This proposal intentionally carries the source material alongside the field handoff, not only the conclusion.

| Source | Local / canonical location | Notes |
| --- | --- | --- |
| V3 Notion canonical page | `https://www.notion.so/34a004b868bb80a78f00c4fcef57ab71` | Canonical V3 source; connector snapshot reviewed on 2026-04-22. Embedded DB links are listed below. |
| V3 DB-01 项目/规格库 | `https://www.notion.so/f2a96cc3773244c4b038a33528ab21be` | Existing Notion DB referenced by V3. |
| V3 DB-02 角色/资产库 | `https://www.notion.so/89450dfe46db41599ec5db9c5998a900` | Existing Notion DB referenced by V3. |
| V3 DB-03 分镜主表 | `https://www.notion.so/ba3ddcdf13b34987a3dd486a92bded97` | Existing Notion DB referenced by V3. |
| V3 DB-04 连续性锁库 | `https://www.notion.so/4122b6d19f594534a9b2fe743e652f55` | Existing Notion DB referenced by V3. |
| V3 DB-05 Prompt 产出库 | `https://www.notion.so/01b73cbcc60549c8aef121e1af5615ae` | Existing Notion DB referenced by V3. |
| V3 黄金样本库 | `https://www.notion.so/642bd7876e0d4649814f35fb133b67f3` | User will add 48 golden samples later; fields are reserved here. |
| V4 structured schema source | `docs/contracts/export-overlays/source-materials/Hope_V4_结构化分镜Schema_Seedance优先版.md` | Verbatim local source copied from `D:\Downloads`. |
| V4 Notion build-list source | `docs/contracts/export-overlays/source-materials/Hope_V4_Notion建库清单.md` | Verbatim local source copied from `D:\Downloads`. |
| Seedance2 V108 alignment refresh | `docs/contracts/export-overlays/seedance2-v108-v3-docs-alignment-refresh-2026-04-23.md` | 2026-04-23 addendum. V108 XLSX / DOCX is the latest canonical source format for this gate. |

V3 source note: the canonical source remains the Notion page above because it contains embedded databases and live Notion schema. This handoff preserves the V3 field content that matters for engineering review in the field tables and appendices below, and commits the complete local V4 markdown sources as companion material.

## Model Split

V3/V4 must stop using a single ambiguous `primary_model` field. The project-level model boundary is:

| Layer | Field | Required value / meaning |
| --- | --- | --- |
| content generation | `content_generation_model` | Qwen/千问; generates story, screenplay, cut text, six core fields, repair drafts. |
| target video execution | `target_video_model` | Seedance 2.0; downstream video execution target, not the text generator. |
| target model version | `target_video_model_version` | Example: `dreamina-seedance-2-0-260128`; model capability validation is version-specific. |
| adapter | `adapter_profile` | Example: `seedance_2_0_primary_adapter`; compiles Canonical Shot Spec to target prompt/payload. |
| fallback targets | `fallback_video_models` | Optional future target list such as Kling/Sora/Runway/Veo; adapter-only, does not rewrite Canonical Shot Spec. |

New governing sentence:

```text
Hope 用千问生成结构化分镜真相；V3/V4 定义千问输出字段与 Seedance-ready 编译要求；Seedance 2.0 是下游视频执行模型与 adapter profile，不是 Hope 的 primary text model。
```

## Runtime Boundary

The intended chain is:

```text
Desktop Input
-> Hope Orchestrator
-> KB Snapshot / Rules / Golden Samples
-> Qwen Text Generation
-> V3/V4 Structured Shot Spec
-> Hope Validator
-> Desktop Review
-> Excel Export
-> optional Seedance Adapter / Payload
```

Notion is a planning/build surface and light-validation surface. It is not the runtime source of truth for the frozen v0.1 contract.

## Field Ownership Legend

| belongs_to | Meaning |
| --- | --- |
| `hope_v0.1` | Existing frozen Hope contract / domain concept. Do not mutate in this proposal. |
| `qwen_generation_overlay` | Field that constrains or captures Qwen-generated Seedance-ready content. |
| `kb_support` | Knowledge/source/sample/repair/failure support that belongs in KB or Notion KB management. |
| `seedance_adapter_output` | Derived prompt/payload/export artifact compiled from structured shot truth. |
| `desktop_review` | Human-review/readiness surface, not a payload editor. |
| `validator_overlay` | Future v0.2 validation/blocking/repair output. |
| `excel_overlay` | Future v0.2 additive sheet or export profile. |

## Field Table

| field_name | 中文名/含义 | belongs_to | v0.1_existing? | v0.2_proposal_only? | rename_needed? | implementation_allowed_now? | notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `Project` | 项目 | `hope_v0.1` | yes | no | no | no | Frozen main entity; do not mutate. |
| `Episode` | 集 | `hope_v0.1` | yes | no | no | no | Frozen main entity; do not mutate. |
| `NarrativeScene` | 叙事场景 | `hope_v0.1` | yes | no | no | no | V4 `story_segment_map` must map to this, not replace it. |
| `RenderSegment` | 渲染片段，30-90s | `hope_v0.1` | yes | no | no | no | Seedance 4-15s is target clip duration, not this field. |
| `Cut` | 镜头 / 分镜记录 | `hope_v0.1` | yes | no | no | no | V4 CanonicalShotSpec overlays Cut; it does not replace Cut. |
| `PromptPackage` | 提示词包 | `hope_v0.1` | yes | no | no | no | Current v0.1 columns remain unchanged. |
| `Validation` | 校验报告 | `hope_v0.1` | yes | no | no | no | Future v0.2 validation overlay must be additive. |
| `content_generation_model` | 内容生成模型，千问/Qwen | `qwen_generation_overlay` | no | yes | new | no | P0 replacement for ambiguous `primary_model`. |
| `target_video_model` | 下游视频执行模型 | `seedance_adapter_output` | no | yes | new | no | Example: `seedance_2_0`. |
| `target_video_model_version` | 下游模型版本 | `seedance_adapter_output` | no | yes | new | no | Example: `dreamina-seedance-2-0-260128`. |
| `adapter_profile` | 适配器配置 | `seedance_adapter_output` | no | yes | new | no | Example: `seedance_2_0_primary_adapter`. |
| `fallback_video_models` | 备用视频执行模型 | `seedance_adapter_output` | no | yes | new | no | Adapter-only future extension. |
| `primary_model` | 原 V4 主模型 | mixed / deprecated | no | yes | yes | no | Do not use as new truth; split into generation and target model fields. |
| `model_version` | 原 V4 模型版本 | mixed / deprecated | no | yes | yes | no | Rename to `target_video_model_version` where it means Seedance. |
| `model_capability_profile` | 模型能力档案 | `seedance_adapter_output` | no | yes | no | no | Holds resolution/duration/input limits by target model version. |
| `generation_phase` | 阶段化生成状态 | `qwen_generation_overlay` | no | yes | new | no | Values: story, screenplay, cut, export, validation/repair. |
| `story_core_intent` | 全片故事核心 | `qwen_generation_overlay` | partial | yes | no | no | Derived from user synopsis + Hope story layer. |
| `story_emotional_baseline` | 全片情绪底色 | `qwen_generation_overlay` | no | yes | no | no | Qwen output, KB-informed. |
| `committee_directing_strategy` | 导演组融合策略 | `kb_support` | no | yes | no | no | Must reference KB role codes, not a parallel role system. |
| `committee_role_mix` | V3 角色混合 | deprecated | no | yes | yes | no | Replace with `committee_role_weights`. |
| `committee_role_weights` | 导演组权重 | `qwen_generation_overlay` | no | yes | no | no | Must map to KB role codes: `chief/scene/action/emotion/transition/suspense/comedy`. |
| `shot_function` | 镜头功能显示值 | mixed | no | yes | yes | no | Split into code/display/taxonomy fields. |
| `shot_function_code` | 镜头功能机器码 | `qwen_generation_overlay` | no | yes | new | no | Avoids taxonomy drift. |
| `shot_function_display` | 镜头功能显示名 | `desktop_review` | no | yes | new | no | Human label only. |
| `kb_scene_type` | KB 场景分类 | `kb_support` | no | yes | new | no | Maps to KB scene taxonomy. |
| `scene_taxonomy_alias_id` | 分类别名 ID | `kb_support` | no | yes | new | no | Keeps V4 shot function aligned with KB aliases. |
| `task_type` | 任务类型 generate/edit/extend | `qwen_generation_overlay` | no | yes | no | no | Replaces part of V3 `generation_mode`. |
| `input_profile` | 输入类型 t2v/i2v/v2v/multimodal | `qwen_generation_overlay` | no | yes | no | no | Replaces part of V3 `generation_mode`. |
| `structure_mode` | single_shot / multi_shot_bundle | `qwen_generation_overlay` | no | yes | no | no | Replaces part of V3 `generation_mode`. |
| `frame_anchor_mode` | 首帧/首尾帧锚定模式 | `qwen_generation_overlay` | no | yes | no | no | Replaces part of V3 `generation_mode`. |
| `generation_mode` | V3 混合生成模式 | deprecated | no | yes | yes | no | Replace with four orthogonal fields above. |
| `target_clip_duration_sec` | Seedance 执行单元时长 | `seedance_adapter_output` | no | yes | yes | no | Rename from V3/V4 `duration_sec` when applying 4-15s target constraint. |
| `duration_sec` | 通用时长 | mixed | partial | yes | yes | no | Ambiguous with RenderSegment; avoid for target clip. |
| `aspect_ratio` | 画幅 | `seedance_adapter_output` | no | yes | no | no | Target adapter field; validate by target capability profile. |
| `resolution` | 分辨率 | `seedance_adapter_output` | no | yes | no | no | Target-version-specific validation. |
| `generate_audio` | 是否生成音频 | `seedance_adapter_output` | no | yes | no | no | Adapter field; audio core still Qwen-generated. |
| `visual_scene_core` | 画面/环境/光影/材质/空气感 | `qwen_generation_overlay` | no | yes | no | no | V3 core preserved; Qwen output, not Seedance native field. |
| `motion_performance_core` | 动作/表演/节拍/眼动/呼吸 | `qwen_generation_overlay` | no | yes | no | no | V3 core preserved. |
| `camera_directing_core` | 景别/角度/焦距感/运镜 | `qwen_generation_overlay` | no | yes | no | no | V3 core preserved; KB vocabulary-informed. |
| `audio_directing_core` | 对白/环境音/BGM/SFX/同步点 | `qwen_generation_overlay` | no | yes | no | no | V3 core preserved. |
| `continuity_lock_core` | 角色/服装/空间/轴线/首尾状态 | `qwen_generation_overlay` | no | yes | no | no | V3 core preserved. |
| `reference_control_core` | 参考素材锁定说明 | `qwen_generation_overlay` | no | yes | no | no | V3 core preserved; should reference AssetRegistry. |
| `blocking_map` | 走位/调度图谱 | `qwen_generation_overlay` | no | yes | new | no | V4 atomic support field. |
| `lens_feel` | 焦距/镜头感 | `qwen_generation_overlay` | no | yes | new | no | V4 atomic support field. |
| `screen_direction` | 画面方向/轴线方向 | `qwen_generation_overlay` | no | yes | new | no | V4 atomic support field. |
| `entry_state` | 镜头进入状态 | `qwen_generation_overlay` | no | yes | new | no | V4 atomic support field. |
| `exit_state` | 镜头退出状态 | `qwen_generation_overlay` | no | yes | new | no | V4 atomic support field. |
| `acting_intent` | 表演意图 | `qwen_generation_overlay` | no | yes | new | no | V4 atomic support field; must be converted to visible action for target prompt. |
| `cut_reason` | 这一镜为什么存在 | `qwen_generation_overlay` | no | yes | new | no | Similar to V4 `shot_purpose`; useful for validator. |
| `shot_purpose` | 镜头目的 | `qwen_generation_overlay` | no | yes | no | no | Keep as human-readable purpose. |
| `asset_registry` | 统一资产库 | `excel_overlay` | no | yes | no | no | V4 overlay; not KB schema; not current domain. |
| `asset_id` | 资产 ID | `excel_overlay` | no | yes | no | no | Belongs to AssetRegistry overlay. |
| `modality` | 资产模态 image/video/audio/storyboard | `excel_overlay` | no | yes | no | no | Needed for Seedance input buckets. |
| `asset_role` | 资产角色 | `excel_overlay` | no | yes | no | no | Character/scene/prop/start_frame/end_frame/etc. |
| `bind_scope` | 绑定范围 | `excel_overlay` | no | yes | no | no | global/segment/shot/beat/etc. |
| `rights_status` | 权利状态 | `validator_overlay` | no | yes | no | no | Export blocker when invalid. |
| `real_person_flag` | 真人风险 | `validator_overlay` | no | yes | no | no | Compliance validation only. |
| `public_figure_flag` | 公众人物风险 | `validator_overlay` | no | yes | no | no | Compliance validation only. |
| `brand_logo_flag` | 品牌标识风险 | `validator_overlay` | no | yes | no | no | Compliance validation only. |
| `copyright_risk_flag` | 版权风险 | `validator_overlay` | no | yes | no | no | Compliance validation only. |
| `image_ref_assets` | 图像参考资产 | `seedance_adapter_output` | no | yes | no | no | Derived relation/bucket from AssetRegistry. |
| `video_ref_assets` | 视频参考资产 | `seedance_adapter_output` | no | yes | no | no | Derived relation/bucket from AssetRegistry. |
| `audio_ref_assets` | 音频参考资产 | `seedance_adapter_output` | no | yes | no | no | Derived relation/bucket from AssetRegistry. |
| `shot_beats` | beat/timeline 结构 | `excel_overlay` | no | yes | no | no | V4 overlay; formalizes V3 `timeline_segments`. |
| `shot_beats_link` | 关联 beat | `qwen_generation_overlay` | no | yes | no | no | Used only when `structure_mode=multi_shot_bundle`. |
| `timeline_segments` | V3 timeline 文本 | deprecated | no | yes | yes | no | Replace with `shot_beats`; transitional `timeline_segments_json` allowed only in proposal. |
| `timeline_segments_json` | timeline JSON 过渡字段 | `seedance_adapter_output` | no | yes | new | no | Export/adapter-only if DB-07 is not available. |
| `beat_id` | beat ID | `excel_overlay` | no | yes | no | no | V4 overlay sheet field. |
| `beat_order` | beat 顺序 | `excel_overlay` | no | yes | no | no | V4 overlay sheet field. |
| `t_in_sec` | beat 起点 | `excel_overlay` | no | yes | no | no | Must fit target clip duration. |
| `t_out_sec` | beat 终点 | `excel_overlay` | no | yes | no | no | Must fit target clip duration. |
| `beat_goal` | beat 目标 | `qwen_generation_overlay` | no | yes | no | no | Human-readable timeline purpose. |
| `visual_beat` | beat 画面要点 | `qwen_generation_overlay` | no | yes | no | no | Qwen output. |
| `motion_beat` | beat 动作要点 | `qwen_generation_overlay` | no | yes | no | no | Qwen output. |
| `camera_beat` | beat 镜头要点 | `qwen_generation_overlay` | no | yes | no | no | Qwen output. |
| `audio_beat` | beat 声音要点 | `qwen_generation_overlay` | no | yes | no | no | Qwen output. |
| `transition_note` | beat 衔接说明 | `qwen_generation_overlay` | no | yes | no | no | Qwen output; KB transition vocabulary-informed. |
| `compiled_prompt_primary` | V4 当前主 prompt | deprecated | no | yes | yes | no | Rename; `primary` is ambiguous. |
| `compiled_payload_primary` | V4 当前主 payload | deprecated | no | yes | yes | no | Rename; `primary` is ambiguous. |
| `compiled_prompt_seedance` | Seedance prompt 派生产物 | `seedance_adapter_output` | no | yes | new | no | Derived from Canonical Shot Spec; not editable source truth. |
| `compiled_payload_seedance` | Seedance payload 派生产物 | `seedance_adapter_output` | no | yes | new | no | Derived by adapter; not human-authored truth. |
| `seedance_prompt_final` | V3 最终 prompt | deprecated | no | yes | yes | no | Rename to `compiled_prompt_seedance`. |
| `seedance_payload_json` | V3 Seedance API 快照 | deprecated | no | yes | yes | no | Rename to `compiled_payload_seedance`. |
| `payload_hash` | payload hash | `seedance_adapter_output` | no | yes | new | no | Traceability; export/debug only. |
| `payload_schema_version` | payload schema version | `seedance_adapter_output` | no | yes | new | no | Adapter output schema marker. |
| `export_filename` | 导出文件名 | `seedance_adapter_output` | no | yes | no | no | Derived naming result. |
| `source_core_hashes` | 源 core hash | `validator_overlay` | no | yes | new | no | Traceability from generated fields to payload. |
| `export_ready` | 是否可导出 | deprecated | no | yes | yes | no | Do not rely on Notion Formula as truth. |
| `export_ready_status` | 导出就绪状态 | `validator_overlay` | no | yes | new | no | Hope Validator output. |
| `validation_score` | 校验评分 | `validator_overlay` | no | yes | no | no | Future v0.2 validator output. |
| `validator_run_id` | 校验运行 ID | `validator_overlay` | no | yes | new | no | Traceability. |
| `validator_version` | 校验版本 | `validator_overlay` | no | yes | new | no | Traceability. |
| `blocking_failure_codes` | 阻断失败码 | `validator_overlay` | no | yes | new | no | Drives repair and export blockers. |
| `export_blockers_json` | 导出阻断 JSON | `validator_overlay` | no | yes | new | no | Structured blocker payload. |
| `missing_items` | 缺失项 | `validator_overlay` | no | yes | no | no | Desktop review/export panel summary. |
| `asset_conflicts` | 资产冲突 | `validator_overlay` | no | yes | no | no | Derived from AssetRegistry and reference controls. |
| `continuity_warnings` | 连续性警告 | `validator_overlay` | no | yes | no | no | Derived from continuity locks and beat/cut context. |
| `compliance_warnings` | 合规警告 | `validator_overlay` | no | yes | no | no | IP/person/brand risk. |
| `adapter_compile_warnings` | adapter 编译警告 | `validator_overlay` | no | yes | new | no | Indicates prompt/payload compile issues. |
| `repair_recommendations` | 修复建议 | `validator_overlay` | partial | yes | no | no | Existing desktop has a bounded repair shape; v0.2 can add Seedance-specific details later. |
| `golden_sample_library` | 黄金样本库 | `kb_support` | no | yes | new | no | Keep separate from shot board. |
| `sample_text` | 样本文本 | `kb_support` | no | yes | new | no | Required for few-shot/validator/repair. |
| `sample_type` | positive/negative/repair_before/repair_after | `kb_support` | no | yes | new | no | Avoids "good copy only" sample library. |
| `sample_usage` | fewshot/validator/repair/regression | `kb_support` | no | yes | new | no | One sample can serve multiple uses. |
| `v4_profile` | V4 阶段 profile | `kb_support` | no | yes | new | no | story_guard/screenplay_guard/cut_guard/export_contract/validation_profile. |
| `covered_core_fields` | 覆盖 core 字段 | `kb_support` | no | yes | new | no | Used for sample retrieval and validation coverage. |
| `covered_atomic_fields` | 覆盖原子字段 | `kb_support` | no | yes | new | no | Supports V4 atomic-field validator. |
| `validator_expected_result` | 预期校验结果 | `kb_support` | no | yes | new | no | Lets golden samples serve validator tests. |
| `expected_failure_codes` | 预期失败码 | `kb_support` | no | yes | new | no | For negative/regression samples. |
| `repair_template_id` | 修复模板 ID | `kb_support` | no | yes | new | no | Maps failures to repair templates. |
| `repair_hint` | 修复提示 | `kb_support` | no | yes | new | no | Lets samples serve repair. |
| `negative_boundary_marker` | 负向边界标记 | `kb_support` | no | yes | new | no | Prevents repair from rewriting unrelated layers. |
| `director_profile_id` | 内部导演档案 ID | `kb_support` | no | yes | no | no | Internal only; do not export "imitate director". |
| `style_profile_id` | 内部风格档案 ID | `kb_support` | no | yes | no | no | Internal only; no final prompt imitation language. |
| `voice_profile_id` | 内部口吻档案 ID | `kb_support` | no | yes | new | no | Internal guidance only. |

## V3 Source Snapshot Carried Into This Handoff

V3 source page title:

```text
Hope · V3 专业动画导演分镜 × Seedance 2.0 字段规范最终版
```

V3 core principle:

```text
少字段、厚内容、强融合；表头是契约，内容是导演口吻连续散文。
```

V3 key value that remains valid:

- Six core fields are still valuable as Qwen-generated director-language output fields.
- Golden samples should drive few-shot generation and repair, not rigid sentence templates.
- Field content should avoid list-like template filling and empty adjectives.
- Failures should be prevented before export through constraints and validator/repair.

V3 parts superseded by this handoff:

- "目标模型: Seedance 2.0" must be restated as "target_video_model: Seedance 2.0"; the content generation model is Qwen.
- `generation_mode` must split into `task_type + input_profile + structure_mode + frame_anchor_mode`.
- `committee_role_mix` must become `committee_role_weights` and map to KB role codes.
- `seedance_prompt_final` and `seedance_payload_json` become adapter-derived fields, not canonical truth.
- `export_ready` must be Hope Validator output, not a Notion Formula source of truth.
- V3's "asset total <= 12" must move into model capability profile / adapter validation.

V3 important fields preserved:

```text
visual_scene_core
motion_performance_core
camera_directing_core
audio_directing_core
continuity_lock_core
reference_control_core
```

V3 field/database source links preserved:

```text
V3 page: https://www.notion.so/34a004b868bb80a78f00c4fcef57ab71
DB-01: https://www.notion.so/f2a96cc3773244c4b038a33528ab21be
DB-02: https://www.notion.so/89450dfe46db41599ec5db9c5998a900
DB-03: https://www.notion.so/ba3ddcdf13b34987a3dd486a92bded97
DB-04: https://www.notion.so/4122b6d19f594534a9b2fe743e652f55
DB-05: https://www.notion.so/01b73cbcc60549c8aef121e1af5615ae
Golden samples DB: https://www.notion.so/642bd7876e0d4649814f35fb133b67f3
```

## V4 Source Material Carried Into This Handoff

The full V4 local source documents are committed as companion docs:

```text
docs/contracts/export-overlays/source-materials/Hope_V4_结构化分镜Schema_Seedance优先版.md
docs/contracts/export-overlays/source-materials/Hope_V4_Notion建库清单.md
```

V4 ideas accepted for v0.2 proposal:

- Prompt is not canonical truth.
- Canonical shot structure should exist before prompt/payload compilation.
- `asset_registry` is useful for multimodal references, rights, conflicts, and reuse.
- `shot_beats` is useful for multi-shot/timeline control and local repair.
- Model capability profile should validate target model constraints by version.
- Notion should do light validation only; Hope Validator should own semantic readiness.

V4 ideas that must be restated:

- Seedance is not the "primary model" for Hope. It is the target video model.
- `primary_model` must be split as described above.
- `compiled_*_primary` must be renamed because "primary" means Qwen in Hope's text route.
- V4 export sheets are future v0.2 overlays, not replacements for frozen v0.1 workbook sheets.

## Proposed v0.2 Overlay Shapes

These names are proposal-only and not implemented:

```text
SeedanceExportOverlay
SeedanceShotOverlay
SeedanceAssetRegistry
SeedanceShotBeat
SeedanceCompiledOutput
SeedanceValidationReport
ModelCapabilityProfile
```

Mapping direction:

| Existing v0.1 source | Future v0.2 overlay |
| --- | --- |
| `CutRecord` | `SeedanceShotOverlay` / Canonical Shot Spec |
| `RenderSegment` | clip-group planning source; not replaced by 4-15s target clip |
| `PromptPackageRecord` | `SeedanceCompiledOutput` |
| `ValidationReport` | `SeedanceValidationReport` |
| KB snapshot | golden samples, role codes, taxonomy, failure/repair mappings |

## Proposed Excel Overlay Sheets

The frozen v0.1 17-sheet workbook must remain unchanged. Future v0.2 may add separate overlay sheets or a separate export profile:

```text
EXP_v4_shots_master
EXP_v4_shot_beats
EXP_v4_asset_registry
EXP_seedance_primary
EXP_v4_validator_report
```

`EXP_seedance_primary` should use target-model wording, for example:

```text
shot_id
target_video_model
target_video_model_version
adapter_profile
image_ref_assets
video_ref_assets
audio_ref_assets
compiled_prompt_seedance
compiled_payload_seedance
export_filename
payload_hash
```

## Desktop Boundary

Desktop may eventually show Seedance-ready readiness in existing surfaces only:

- Keep current `projects / writer / preview / export` surfaces.
- Do not add a fifth desktop surface for this proposal.
- Show human-review fields and blockers.
- Do not turn desktop into a Seedance payload editor.

Suggested visible/read-only review fields:

```text
shot_id / cut_id
render_segment_id
sequence_no
shot_purpose
shot_function_display
visual_scene_core
motion_performance_core
camera_directing_core
audio_directing_core
continuity_lock_core
reference_control_core
export_ready_status
blocking_failure_codes
missing_items
repair_recommendations
asset reference summaries
shot beat summaries
```

Suggested export/debug-only fields:

```text
compiled_prompt_seedance
compiled_payload_seedance
payload_hash
payload_schema_version
adapter_profile
model_capability_profile
source_core_hashes
reference_assets_json
shot_beats_json
validator_run_id
validator_version
```

## Golden Sample Field Requirements

The user plans to add 48 golden samples. The sample library must not be only "good copy"; it should support few-shot, validator, and repair.

Required proposed fields:

```text
sample_id
sample_title
sample_text
sample_type
sample_usage
generation_stage
v4_profile
content_generation_model
target_video_model
adapter_profile
covered_v4_fields
covered_core_fields
covered_atomic_fields
shot_function_code
kb_scene_type
committee_role_codes
committee_role_weights
retrieval_tags
source_ids
source_type
source_notes
confidence_level
last_reviewed_at
validator_expected_result
expected_failure_codes
blocking_scope
repair_template_id
repair_hint
negative_boundary_marker
word_count
empty_words_detected
language_check
rights_or_ip_risk_flag
usable_for_fewshot
usable_for_validator
usable_for_repair
```

## Explicitly Forbidden

This branch must not:

- modify the frozen v0.1 17-sheet contract
- modify exporter behavior
- modify IPC
- modify Rust domain/contracts/validators
- modify desktop business UI
- connect to or call real Seedance
- merge or mutate `hope-kb`
- widen KB runtime integration
- treat this proposal as a frozen implementation contract
- replace `CutRecord`, `PromptPackageRecord`, or current workbook columns

## Implementation Permissions For This Branch

| Area | Allowed now? | Notes |
| --- | --- | --- |
| docs-only proposal | yes | This file and source-material copies only. |
| Notion changes | no | Not performed in this branch. |
| hope-kb changes | no | Explicitly out of scope. |
| Rust DTO/schema | no | Requires future v0.2 contract gate. |
| exporter | no | Frozen v0.1 contract remains. |
| IPC | no | Out of scope. |
| desktop UI | no | Out of scope. |
| real Seedance integration | no | Out of scope. |

## Recommended Next Control-Thread Message

```text
V3/V4 field overlay is ready for governance review as a docs-only proposal on branch codex/v3-field-overlay-proposal. It keeps Hope v0.1 frozen and treats V3/V4 as a v0.2 Seedance-ready export overlay. The key correction is to split model semantics: content_generation_model=Qwen, target_video_model=Seedance 2.0, adapter_profile=Seedance adapter. No exporter/Rust/IPC/desktop/hope-kb implementation is authorized by this proposal.
```

## Verification Commands

Run from any checkout of this branch:

```powershell
git -C E:\codex\hope status --short --branch
git -C E:\codex\hope log -1 --oneline --decorate
```

Pre-change verification from this handoff:

```text
git -C E:\codex\hope status --short --branch
## codex/contracts-freeze...origin/codex/contracts-freeze

git -C E:\codex\hope log -1 --oneline --decorate
c3d6142 (HEAD -> codex/contracts-freeze, origin/codex/contracts-freeze) docs: accept seedance v1 as v0.2 export overlay proposal
```

Final post-commit verification is expected to show this proposal branch clean and its latest commit as the docs-only handoff commit.
