# Hope V4 · Notion 建库清单（Seedance 优先版）

> 这份清单是基于 `Hope_V4_结构化分镜Schema_Seedance优先版.md` 进一步落地成的执行版文档。
> 用途：帮助你按正确顺序在 Notion 建库，并与 Excel 导出模板保持同一套字段真相。

## 1. 建库顺序

- [ ] 新建 `DB-01 global_project_bible`，先把项目默认模型、默认画幅、默认时长、合规策略定下来。
- [ ] 新建 `DB-02 story_segment_map`，把故事拆成 segment，并先确定每段的目标与结构偏好。
- [ ] 新建 `DB-04 character_lock` 和 `DB-05 scene_asset_lock`，把角色与场景的一致性锁提前固定。
- [ ] 新建 `DB-06 asset_registry`，把所有图 / 视频 / 音频 / storyboard / keyframe 先入库再引用。
- [ ] 新建 `DB-03 canonical_shot_board`，开始逐镜头填写六大 core 与生成控制轴。
- [ ] 新建 `DB-07 shot_beats`，只要镜头是 `multi_shot_bundle`，就把时间轴拆到 beat。
- [ ] 最后接入 Hope validator、Seedance 主适配器和 Excel exporter。

## 2. 库之间的关系图

```text
DB-01 global_project_bible
  └─→ DB-02 story_segment_map
        └─→ DB-03 canonical_shot_board
               └─→ DB-07 shot_beats

DB-04 character_lock ─┐
                      ├─→ DB-06 asset_registry ──→ DB-03 / DB-07
DB-05 scene_asset_lock┘
```

## 3. 全局约定

- `Prompt` 不是主真相，真正的主真相是 Shot 结构、资产绑定、连续性和 beat 时间结构。
- Notion 负责轻校验；Hope 负责强校验；Excel 负责结构化导出，不再只导一列 prompt。
- 所有参考资产必须先进入 `asset_registry`，再从 Shot 或 Beat 层引用。
- DB-03 建议把 A 层保留给人工编辑，把 B 层放到 Hope 内部 schema 或在 Notion 里折叠隐藏。
- 如果你要做 fallback，不要改 DB 结构，只改 adapter。

## 4. 每个数据库的建库清单

### DB-01_global_project_bible

项目总控：故事目标、模型策略、默认输出参数、合规与导出规则。

| key | 优先级 | Notion 类型 | 填充方式 | 枚举/备注 | 说明 |
| --- | --- | --- | --- | --- | --- |
| `bible_id` | P2 | Title | 系统 |  | 全局配置 ID |
| `work_title` | P2 | Text | 用户 |  | 作品名 |
| `version` | P2 | Text | 系统 |  | 版本号 |
| `primary_model` | P0 | Select | 用户 | primary_model | 默认主模型 |
| `fallback_models` | P1 | Multi-select | 用户/系统 |  | 备用导出顺序 |
| `story_core_intent` | P0 | Text | 用户 + Hope |  | 全片故事核心 |
| `story_emotional_baseline` | P1 | Text | Hope |  | 全片情绪底色 |
| `committee_directing_strategy` | P1 | Text | Hope-kb |  | 六类能力融合策略 |
| `story_visual_rules` | P1 | Text | Hope-kb + Hope |  | 由故事推导出的视觉规则 |
| `motion_performance_rules` | P1 | Text | Hope-kb |  | 动作与表演总规则 |
| `lighting_color_rules` | P1 | Text + 附件 | Hope-kb |  | 光影与色彩总规则 |
| `audio_generation_rules` | P1 | Text | Hope-kb + 用户 |  | 对白 / BGM / SFX / 环境音 / 静默策略 |
| `continuity_rules_global` | P1 | Text | Hope-kb |  | 连续性总规则 |
| `model_capability_profile` | P2 | Text/JSON | 系统 |  | 记录各模型真实可用的分辨率 / 时长 / 模式 / 输入上限 |
| `default_task_type` | P0 | Select | 用户 | task_type | 默认任务轴 |
| `default_input_profile` | P0 | Select | 用户 | input_profile | 默认输入轴 |
| `default_structure_mode` | P0 | Select | 用户 | structure_mode | 默认结构轴 |
| `default_duration_sec` | P0 | Number/Select | 用户 |  | 默认时长 |
| `default_aspect_ratio` | P0 | Select | 用户 | aspect_ratio | 默认画幅 |
| `default_resolution` | P0 | Select | 用户 | resolution | 默认分辨率；由 model_version 动态校验 |
| `default_generate_audio` | P0 | Checkbox | 用户 | bool_tf | 默认生成音频 |
| `adapter_policy` | P1 | Text | 系统/Hope |  | 例如：Seedance 主适配、Kling 备份 |
| `compliance_policy` | P1 | Text | Hope-kb |  | 肖像 / IP / 品牌 / 敏感内容合规规则 |
| `failure_modes_catalog` | P1 | Text | Hope-kb |  | 模型常见失败模式清单 |
| `export_naming_rule` | P2 | Text | 系统 |  | 文件命名规则 |
| `migration_notes` | P2 | Text | 系统 |  | V3 → V4 迁移记录 |

### DB-02_story_segment_map

叙事片段层：场景段落、情绪曲线、拼接关系、片段目标。

| key | 优先级 | Notion 类型 | 填充方式 | 枚举/备注 | 说明 |
| --- | --- | --- | --- | --- | --- |
| `segment_id` | P2 | Title | 系统 |  | 片段 ID |
| `bible_link` | P2 | Relation→DB-01 | 系统 |  | 关联项目圣经 |
| `episode_no` | P2 | Number | 系统 |  | 系列项目可用 |
| `scene_no` | P2 | Number | 系统 |  | 场景号 |
| `segment_no` | P2 | Number | 系统 |  | 片段号 |
| `segment_title` | P2 | Text | Hope |  | 片段标题 |
| `scene_attr` | P1 | Select | Hope | scene_attr | 内/外、日/夜、现实/回忆/梦境等 |
| `scene_summary` | P1 | Text | Hope |  | 片段摘要 |
| `segment_goal` | P0 | Text | Hope |  | 本段叙事目标 |
| `segment_emotion_curve` | P1 | Text | Hope-kb + Hope |  | 本段情绪曲线 |
| `preferred_structure_mode` | P0 | Select | Hope | structure_mode | 默认 single_shot / multi_shot_bundle |
| `preferred_model_override` | P1 | Select/Text | Hope | primary_model | 片段级模型覆盖（可空） |
| `continuity_priority` | P1 | Select | Hope-kb | continuity_priority | 强连续 / 中连续 / 可独立 |
| `stitch_in_note` | P1 | Text | Hope |  | 与上一片段衔接说明 |
| `stitch_out_note` | P1 | Text | Hope |  | 与下一片段衔接说明 |
| `target_duration_sec` | P2 | Number | 系统 |  | 片段目标时长 |
| `shot_count` | P2 | Rollup | 系统 |  | 子镜头数量 |
| `segment_export_ready` | P2 | Formula | 系统 |  | 片段导出是否就绪 |

### DB-03_canonical_shot_board

核心镜头真相表：一行一个 shot，A 层给导演与 Hope 主填，B 层给校验/编译/导出使用。

| key | 优先级 | Notion 类型 | 填充方式 | 枚举/备注 | 说明 |
| --- | --- | --- | --- | --- | --- |
| `shot_id` | P2 | Title | 系统 |  | 镜头唯一编号 |
| `segment_link` | P2 | Relation→DB-02 | 系统 |  | 所属片段 |
| `shot_no` | P2 | Number | 系统 |  | 片段内镜头号 |
| `global_order` | P2 | Number | 系统 |  | 全片镜头顺序 |
| `clip_group_id` | P2 | Text | 系统/Hope |  | 实际生成单元 ID |
| `task_type` | P0 | Select | Hope | task_type | generate / edit / extend |
| `input_profile` | P0 | Select | Hope | input_profile | t2v / i2v / v2v / multimodal |
| `structure_mode` | P0 | Select | Hope | structure_mode | single_shot / multi_shot_bundle |
| `primary_model` | P0 | Select | 默认继承 | primary_model | 主输出模型 |
| `model_version` | P0 | Text/Select | 默认继承 | primary_model | 例如 dreamina-seedance-2-0-260128 |
| `duration_sec` | P0 | Number | Hope |  | 本镜时长 |
| `aspect_ratio` | P0 | Select | 默认继承 | aspect_ratio | 画幅 |
| `resolution` | P0 | Select | 默认继承 | resolution | 分辨率，按模型版本校验 |
| `generate_audio` | P0 | Checkbox | 默认继承 | bool_tf | 是否生成音频 |
| `image_ref_assets` | 条件P0 | Relation→DB-06 | 用户/Hope |  | 图像参考资产 |
| `video_ref_assets` | 条件P0 | Relation→DB-06 | 用户/Hope |  | 视频参考资产 |
| `audio_ref_assets` | 条件P0 | Relation→DB-06 | 用户/Hope |  | 音频参考资产 |
| `frame_anchor_mode` | 条件P0 | Select | Hope | frame_anchor_mode | none / start_frame / start_end_frame |
| `start_frame_asset` | 条件P0 | Relation→DB-06 | 用户/Hope |  | 首帧锚点 |
| `end_frame_asset` | 条件P0 | Relation→DB-06 | 用户/Hope |  | 尾帧锚点 |
| `reference_control_core` | 条件P0 | Text | Hope |  | 每个参考锁什么、强度多大、冲突优先级 |
| `committee_role_weights` | P1 | Text/JSON | Hope-kb |  | 六类导演能力权重 |
| `shot_function` | P1 | Select | Hope-kb | shot_function | 叙事 / 情绪 / 动作 / 空镜 / 转场 / 蒙太奇 |
| `shot_purpose` | P0 | Text | Hope |  | 这镜为什么存在 |
| `visual_scene_core` | P0 | Text | Hope |  | 画面、环境、光影、材质、空气感 |
| `motion_performance_core` | P0 | Text | Hope |  | 动作、表演、微表情、呼吸、眼动 |
| `camera_directing_core` | P0 | Text | Hope-kb |  | 景别、角度、焦距感、运镜、稳定性 |
| `audio_directing_core` | P0 | Text | Hope/Hope-kb |  | 对白、环境音、BGM、SFX、同步点 |
| `continuity_lock_core` | P0 | Text | character/scene lock + Hope-kb |  | 外貌、服装、空间、轴线、首尾状态等 |
| `shot_beats_link` | 条件P0 | Relation→DB-07 | Hope |  | multi-shot 时关联 beats |
| `model_override_notes` | P1 | Text | Hope |  | 某模型专用补充说明 |
| `avoidance_constraints` | P1 | Text | Hope-kb |  | 模型常见失败避免项与约束说明 |
| `blocking_map` | P1 | Text/JSON |  |  | 人物 / 道具 / 动线 / 站位图式 |
| `focus_target` | P1 | Text |  |  | 当前镜头视觉关注点 |
| `lens_feel` | P1 | Select/Text |  |  | 24mm 广角感 / 50mm 标准 / 85mm 压缩感等 |
| `screen_direction` | P1 | Text |  |  | 主体在屏幕空间中的运动方向 |
| `eyeline_axis` | P1 | Text |  |  | 视线轴 / 反打轴线 |
| `entry_state` | P1 | Text |  |  | 本镜首状态 |
| `exit_state` | P1 | Text |  |  | 本镜末状态 |
| `acting_intent` | P1 | Text |  |  | 表演意图：试探 / 防御 / 掩饰 / 讨好等 |
| `cut_reason` | P1 | Text |  |  | 为什么在这里切镜 |
| `compiled_prompt_primary` | P0 | Text |  |  | 当前主模型最终 prompt |
| `compiled_payload_primary` | P2 | Text/JSON |  |  | 当前主模型最终 payload |
| `export_filename` | P2 | Text |  |  | 输出文件名 |
| `shot_status` | P2 | Select |  | shot_status | 草案 / Ready / 已生成 / 已采纳 / 锁定 / 需重生 |
| `shot_version` | P2 | Number |  |  | 版本号 |
| `regeneration_note` | P2 | Text |  |  | 重生原因与失败回填 |
| `validation_score` | P2 | Number/Text |  |  | 校验评分 |
| `export_ready` | P2 | Formula / 系统字段 |  |  | 是否可导出 |

### DB-04_character_lock

角色一致性锁：脸、服装、动作气质、音色与禁改项。

| key | 优先级 | Notion 类型 | 填充方式 | 枚举/备注 | 说明 |
| --- | --- | --- | --- | --- | --- |
| `character_id` | P2 | Title | 系统 |  | 角色 ID；也是语义引用基础 |
| `character_name` | P0 | Text | 用户/Hope |  | 角色名 |
| `story_function` | P1 | Text | Hope |  | 角色在故事中的功能 |
| `appearance_anchors` | P0 | Text | 用户/Hope |  | 可识别脸部锚点：痣 / 疤 / 眼型 / 发际线 / 体型等 |
| `costume_lock` | P0 | Text | 用户/Hope |  | 服装 / 材质 / 配饰锁 |
| `expression_range` | P1 | Text | Hope-kb |  | 表情范围 |
| `motion_personality` | P1 | Text | Hope-kb |  | 动作气质 |
| `habitual_gestures` | P1 | Text | Hope-kb |  | 标志性小动作 |
| `voice_identity` | P0/P1 | Text | 用户/Hope |  | 音色、年龄感、语速、语气习惯 |
| `character_assets` | P0 | Relation→DB-06 | 用户 |  | 角色参考图 / 视频 / 音频 |
| `forbidden_changes` | P0 | Text | Hope |  | 禁止变化项 |
| `continuity_lock` | P2 | Checkbox | 系统 | bool_tf | 是否锁定 |
| `character_prompt_fragment` | P1 | Text | Hope 自动生成 |  | 可复用角色片段 |
| `notes` | P2 | Text | 用户/Hope |  | 备注 |

### DB-05_scene_asset_lock

场景 / 道具 / 环境锁：空间布局、光色、材质、天气与禁改项。

| key | 优先级 | Notion 类型 | 填充方式 | 枚举/备注 | 说明 |
| --- | --- | --- | --- | --- | --- |
| `scene_asset_id` | P2 | Title | 系统 |  | 场景 / 道具主键 |
| `asset_type` | P0 | Select | 用户/Hope | asset_type_scene_lock | 场景 / 道具 / 环境 / 光效 / 天气 |
| `asset_name` | P0 | Text | 用户/Hope |  | 名称 |
| `story_function` | P1 | Text | Hope |  | 在故事中的作用 |
| `space_layout_lock` | P0 | Text | Hope |  | 空间布局与 blocking |
| `key_props_lock` | P0/P1 | Text | Hope |  | 关键道具与位置 |
| `weather_time_lock` | P0 | Text | Hope |  | 时间 / 天气 / 光线时间感 |
| `lighting_lock` | P0/P1 | Text | Hope-kb |  | 光源方向、软硬、色温、阴影边缘 |
| `color_lock` | P0/P1 | Text | Hope-kb |  | 主色 / 辅色 / 点缀色关系 |
| `material_spec` | P1 | Text | Hope |  | 地面、墙面、衣料、玻璃等材质 |
| `environment_motion` | P1 | Text | Hope |  | 雨、风、烟、尘、人流、微粒等 |
| `scene_assets` | P0 | Relation→DB-06 | 用户 |  | 场景参考图 / 视频 / 音频 |
| `forbidden_changes` | P0 | Text | Hope |  | 禁止变化项 |
| `continuity_lock` | P2 | Checkbox | 系统 | bool_tf | 是否锁定 |
| `scene_prompt_fragment` | P1 | Text | Hope 自动生成 |  | 可复用场景片段 |

### DB-06_asset_registry

统一多模态资产库：图 / 视频 / 音频 / storyboard / keyframe 的绑定、权限与风险控制。

| key | 优先级 | Notion 类型 | 填充方式 | 枚举/备注 | 说明 |
| --- | --- | --- | --- | --- | --- |
| `asset_id` | P2 | Title | 系统 |  | 资产唯一 ID |
| `asset_name` | P0 | Text | 用户 |  | 名称 |
| `modality` | P0 | Select | 用户 | modality | image / video / audio / text / storyboard / keyframe |
| `asset_role` | P0 | Select | 用户/Hope | asset_role | character / scene / prop / start_frame / end_frame / motion / dialogue / sfx / music / style / storyboard |
| `bind_scope` | P0 | Select | 用户/Hope | bind_scope | global / segment / shot / beat / character / scene |
| `linked_character` | P1 | Relation→DB-04 | 用户/系统 |  | 对应角色（可空） |
| `linked_scene_asset` | P1 | Relation→DB-05 | 用户/系统 |  | 对应场景 / 道具（可空） |
| `source_path` | P0 | Files / URL / Text | 用户 |  | 文件路径或存储位置 |
| `source_origin` | P1 | Select/Text | 用户 |  | 自制 / 用户上传 / 第三方采购 / 官方样例等 |
| `lock_dimensions` | P0 | Text | Hope |  | 锁脸 / 锁服装 / 锁色彩 / 锁空间 / 锁动态 / 锁音色等 |
| `priority` | P1 | Number / Select | Hope |  | 冲突优先级 |
| `rights_status` | P0 | Select | 用户 | rights_status | 已授权 / 待确认 / 禁商用 / 禁使用 |
| `real_person_flag` | P0 | Checkbox | 用户/系统 | bool_tf | 是否涉及真实人物 |
| `public_figure_flag` | P0 | Checkbox | 用户/系统 | bool_tf | 是否涉及公众人物 |
| `brand_logo_flag` | P0 | Checkbox | 用户/系统 | bool_tf | 是否含品牌 logo |
| `copyright_risk_flag` | P0 | Checkbox | 用户/系统 | bool_tf | 是否有 IP 风险 |
| `duration_or_length` | P2 | Number/Text | 系统 |  | 视频 / 音频时长，或图片张数信息 |
| `availability_status` | P2 | Select | 系统 | availability_status | 可用 / 限制可用 / 禁用 |
| `notes` | P2 | Text | 用户/Hope |  | 备注 |

### DB-07_shot_beats

多镜头 / 时间轴 beat 级结构：用于 multi-shot 编译与局部返修。

| key | 优先级 | Notion 类型 | 填充方式 | 枚举/备注 | 说明 |
| --- | --- | --- | --- | --- | --- |
| `beat_id` | P2 | Title | 系统 |  | beat 唯一 ID |
| `parent_shot` | P0 | Relation→DB-03 | 系统 |  | 所属镜头 |
| `beat_order` | P0 | Number | Hope |  | 顺序 |
| `t_in_sec` | P0 | Number | Hope |  | 起始时间 |
| `t_out_sec` | P0 | Number | Hope |  | 结束时间 |
| `beat_goal` | P0 | Text | Hope |  | 这一拍要完成什么 |
| `visual_beat` | P1 | Text | Hope |  | 本拍视觉要点 |
| `motion_beat` | P1 | Text | Hope |  | 本拍动作要点 |
| `camera_beat` | P1 | Text | Hope-kb |  | 本拍镜头要点 |
| `audio_beat` | P1 | Text | Hope |  | 本拍声音要点 |
| `transition_note` | P1 | Text | Hope |  | 本拍与下一拍如何接 |
| `ref_asset_ids` | P1 | Relation→DB-06 | Hope |  | 该 beat 使用的资产 |
| `continuity_note` | P1 | Text | Hope |  | 该 beat 的连续性约束 |
| `compiled_block_primary` | P2 | Text | 系统 |  | 当前主模型下编译后的 block |
| `beat_status` | P2 | Select | 系统 | beat_status | 草案 / Ready / 锁定 |

## 5. 推荐 Notion 视图

### DB-01_global_project_bible
- 按作品版本查看
- 按 primary_model 查看
- 按 export_naming_rule 查看

### DB-02_story_segment_map
- 按 episode_no / scene_no 查看
- 按 continuity_priority 查看
- 只看 segment_export_ready = false

### DB-03_canonical_shot_board
- 按 segment 分组
- Ready to Export
- Needs Regen
- Multi-shot Only
- 按 primary_model 分组

### DB-04_character_lock
- 按 continuity_lock 查看
- 按角色功能查看
- 只看缺少 character_assets 的条目

### DB-05_scene_asset_lock
- 按 asset_type 查看
- 按 continuity_lock 查看
- 只看缺少 scene_assets 的条目

### DB-06_asset_registry
- 按 modality 查看
- 按 rights_status 查看
- 按 bind_scope 查看
- 只看有风险标记的资产

### DB-07_shot_beats
- 按 parent_shot 分组
- 只看 beat_status != 锁定
- 按 beat_order 排序

## 6. Notion 侧最少轻校验

- 必填字段是否存在。
- Select / Multi-select 是否落在约定枚举内。
- `duration_sec` 是否在项目允许区间内。
- `resolution` 是否与 `model_version` 匹配。
- `frame_anchor_mode = start_frame` 时必须有 `start_frame_asset`。
- `frame_anchor_mode = start_end_frame` 时必须同时有 `start_frame_asset` 与 `end_frame_asset`。
- 图片 / 视频 / 音频参考数量是否超上限。
- `structure_mode = multi_shot_bundle` 时必须有关联 `shot_beats`。

## 7. Hope 侧必须做的强校验

- 六大 core 覆盖项检查。
- `single_shot` 是否只存在一个主动作和一个主调度重心。
- beat 级动作、镜头、声音是否冲突。
- 角色 / 场景 / 光源 / 轴线 / 进出状态是否连续。
- 资产锁定维度是否互相矛盾。
- 真实人物 / 公众人物 / 品牌 / 版权风险是否允许导出。
- 主适配器编译后的 prompt 与 payload 是否空洞、重复或超预算。

## 8. Excel 导出建议

建议至少保留以下 5 张导出 Sheet：

| Sheet | 用途 | 必备列 |
| --- | --- | --- |
| `EXP_shots_master` | 镜头真相导出 | `shot_id, clip_group_id, segment_id, task_type, input_profile, structure_mode, duration_sec, aspect_ratio, resolution, shot_function, shot_purpose, visual_scene_core, motion_performance_core, camera_directing_core, audio_directing_core, continuity_lock_core, export_ready` |
| `EXP_shot_beats` | beat / timeline 导出 | `beat_id, parent_shot, beat_order, t_in_sec, t_out_sec, beat_goal, visual_beat, motion_beat, camera_beat, audio_beat, ref_asset_ids` |
| `EXP_asset_registry` | 资产导出 | `asset_id, modality, asset_role, bind_scope, source_path, lock_dimensions, priority, rights_status, real_person_flag, public_figure_flag, brand_logo_flag, copyright_risk_flag` |
| `EXP_seedance_primary` | Seedance 主适配器导出 | `shot_id, model_version, image_ref_assets, video_ref_assets, audio_ref_assets, compiled_prompt_primary, compiled_payload_primary, export_filename` |
| `EXP_validator_report` | 校验报告 | `shot_id, validation_score, missing_items, asset_conflicts, continuity_warnings, compliance_warnings, export_ready` |

## 9. 交付物说明

- 这份 Markdown 用来搭 Notion。
- 同目录的 Excel 模板用来承接字段结构、下拉枚举与导出 Sheet。
- `Field_Spec` 工作表是字段总索引，后续你做 Hope schema、Excel exporter、Notion property map 时都可以直接对照它。

_生成日期：2026-04-22_