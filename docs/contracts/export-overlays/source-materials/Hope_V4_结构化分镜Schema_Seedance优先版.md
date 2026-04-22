
# Hope · V4 结构化分镜 Schema（Seedance 2.0 优先 / 可编译到多模型）

> **目标**：给 Hope 一套真正可工程化的分镜字段规范：以专业动画导演分镜逻辑为核心，以 Seedance 2.0 为主输出引擎，同时保留向 Kling / Sora / Runway / Veo 编译的能力。  
> **V4 核心升级**：保留 V3 的六大 core 与导演口吻，但把底层骨架改成 **Canonical Shot Spec + Asset Registry + Shot Beats + Model Adapter**。  
> **适用链路**：Hope → Hope-kb → Notion / Excel → Adapter → Seedance 2.0 API / 其他视频模型。

---

## 0. 版本说明

| 版本 | 核心变化 | 日期 |
| --- | --- | --- |
| V1.1 | 多模型适配、P0-P5 分级、6 个数据库 | 2026-04-22 |
| V1.2 | 收敛到 Seedance 单一目标、融合字段、5 个数据库 | 2026-04-22 |
| V3 | 强表头契约、六大融合字段、黄金示例驱动 | 2026-04-22 |
| **V4（本版）** | **从“Seedance 专用字段表”升级为“结构化分镜真相 + Seedance 主适配器”**；新增 `asset_registry` 与 `shot_beats`；拆解 `generation_mode`；分离“官方规格”与“Hope 内部编译约定”；补齐连续性与资产系统；导出层升级为多 Sheet Excel | 2026-04-22 |

### V4 相对 V3 的关键升级

1. **保留六大 core，不保留“Prompt = 真相”**  
   V4 将“Prompt”降级为派生结果，真正的主数据变成镜头结构、资产绑定、连续性与 beat 时间轴。

2. **从单模型强绑定改为“Seedance 主适配器”**  
   Seedance 仍是默认主出口，但 Canonical Schema 不再把自身写死为 Seedance 专用真相。

3. **新增统一资产库 `asset_registry`**  
   所有图 / 视频 / 音频 / 首尾帧 / 角色资产 / 场景资产都进入统一资产层，不再靠长 prompt 里的“参考图 1 / 参考图 2”。

4. **新增 `shot_beats`**  
   multi-shot / timeline 不再只是一格长文本，而是可被 Hope 编译的 beat 级时间结构。

5. **把 `generation_mode` 拆成四条正交轴**  
   `task_type`、`input_profile`、`structure_mode`、`frame_anchor_mode` 四轴替代 V3 的一列混合枚举。

6. **加入隐藏原子字段**  
   在六大 core 之外，补入 `blocking_map`、`lens_feel`、`screen_direction`、`entry_state / exit_state`、`acting_intent` 等原子字段，支持高质量编译和校验。

7. **Notion 只做轻校验，Hope 做语义校验**  
   V3 把太多语义校验压给 Formula；V4 改为：Notion 做 presence / enum / range / relation count，Hope validator 做语义覆盖、冲突检查、合规检查。

---

## 1. V4 十二条铁律

1. **分镜真相存结构，不存长 Prompt。**
2. **Seedance 是主适配器，不是唯一真相。**
3. **一行默认代表一个 Shot，实际生成单元由 `clip_group_id` 组织。**
4. **资产是一级对象，必须进入统一资产库。**
5. **multi-shot 必须 beat 化，不能只靠长文本。**
6. **保留六大 core，但必须补一层机器可读原子字段。**
7. **官方规格与 Hope 内部约定必须分层书写。**
8. **连续性不是备注，而是正式字段。**
9. **IP / 肖像 / 品牌风险必须前置，不放到最后人工判断。**
10. **Notion 做轻约束，Hope 做强校验，Excel 做导出。**
11. **Excel 不只导一列 Prompt，而要导结构化镜头表。**
12. **允许主模型切换与 fallback，不允许 schema 随模型一起重写。**

---

## 2. 官方能力锚定 vs Hope 内部编译约定

> 这一章是 V4 与 V3 最重要的分层修正。  
> **官方能力锚定** = 来自官方文档、影响能力边界的事实。  
> **Hope 内部编译约定** = 为了工程落地设计的写法，不应冒充“官方固定语法”。

### 2.1 Seedance 2.0 / 2.0 fast：官方已确认能力锚点

| 项 | 官方已确认 | V4 设计结论 |
| --- | --- | --- |
| 多模态参考输入 | Seedance 2.0 官方 launch 文案与 BytePlus 教程公开了：可同时输入 **0–9 张图片、0–3 段视频、0–3 段音频**，并结合自然语言指令。[^seed-official][^seed-byteplus-count] | `image_ref_assets` / `video_ref_assets` / `audio_ref_assets` 分列，不再只用一个 `reference_assets` 总 Relation。校验逻辑按模态分桶，而不是只写死“总数 ≤12”。 |
| 标准模型能力 | BytePlus ModelArk 模型列表显示 `dreamina-seedance-2-0-260128` 支持 **Multimodal Reference to Video / Video Modification / Video Extension / I2V First Frame / I2V First & Last Frames**。[^seed-model-list] | `task_type`、`input_profile`、`frame_anchor_mode` 分轴；V4 支持 `generate / edit / extend`。 |
| fast 模型能力 | `dreamina-seedance-2-0-fast-260128` 也支持 Multimodal Reference、Modification、Extension、First Frame、First & Last Frames。[^seed-model-list] | 标准版与 fast 在 schema 层共用，只在 adapter profile 中限制能力与分辨率。 |
| 时长 / 帧率 / 格式 | 官方模型列表给出 **4–15 秒 / 24 fps / mp4**。[^seed-model-list] | `duration_sec` 统一按 4–15 校验；fps 不作为人工字段，写入 adapter profile。 |
| 分辨率 | 官方模型列表显示：**标准版支持 480p / 720p / 1080p，fast 支持 480p / 720p**。[^seed-model-list] | **分辨率不能再被写死为“Seedance 2.0 仅支持 480p/720p”**。V4 改为：`resolution` 必须按 `model_version` 动态校验。 |
| 音画同步 | Seedance 2.0 标准版 / fast 均支持音画同步。[^seed-model-list][^seed-audio-flag] | `generate_audio` 保留；`audio_directing_core` 继续要求对白时间段、同步点、静默设计。 |
| 文本分镜 / storyboard 参考 | 官方 launch 文案展示了模型可参考 **shooting script / storyboard、镜头尺度、运镜、画面与 copy**。[^seed-official] | 允许将 storyboard / shot list / 文字分镜当作参考资产或编译输入，而不是只当自然语言 prompt。 |

### 2.2 V3 中两处需要纠偏的规格写法

#### 纠偏 1：分辨率
V3 把 Seedance 2.0 写成“仅支持 480p / 720p”，这在当前公开的 BytePlus ModelArk 模型列表中并不准确：  
**标准版 `dreamina-seedance-2-0-260128` 公开支持 1080p；fast 版才是 480p / 720p。**[^seed-model-list]

**V4 处理原则**：  
- `resolution` 不再是固定死枚举“480p / 720p”；  
- 改为“**模型版本约束下的合法分辨率**”；  
- 如果你在实际产品流里只打算走低成本或某个固定 UI 档位，可以在 adapter profile 里进一步收紧，但不要把它写成 Canonical Schema 的唯一真相。

#### 纠偏 2：资产数量
V3 直接写了“单次最多 12 个（图+视频+音频合计）”。  
从当前官方公开资料里，我能明确确认的是：**图片 ≤ 9、视频 ≤ 3、音频 ≤ 3**。[^seed-official][^seed-byteplus-count]  
如果你的具体调用路径或 UI 层还有“总文件数上限”，那应写进 **adapter profile**，而不是写死进镜头真相层。

**V4 处理原则**：  
- Canonical Schema 只认：`images <= 9`、`videos <= 3`、`audios <= 3`  
- 若平台另有总数 / 大小 / 时长限制，放在 `model_capability_profile` 里做 adapter 校验

### 2.3 Hope 内部编译约定（不是官方固定语法）

以下内容都可以继续保留，但在文档里必须明确标注为 **Hope internal convention**：

1. `@char_lin / @scene_alley / @sfx_footstep` 这种语义化标签命名  
2. `timeline` 里采用 `[0.0–4.5s]` 这种分段写法  
3. `seedance_payload_json` 的内部字段组织方式  
4. `committee_role_weights` 的表达形式  
5. `shot_beats` 的数据库与 Excel Sheet 结构  
6. `fallback_models` 与 `adapter_policy` 的路由逻辑  

这些都非常合理，也建议继续做，但不应在文档里写成“Seedance 官方固定格式”。

### 2.4 为什么仍然要保留多模型适配层

虽然本项目主打 Seedance 2.0，但当前主流视频模型都在强化“结构控制”：

- Seedance 2.0：多模态参考、edit、extend、首尾帧、音画同步[^seed-official][^seed-model-list]
- Kling VIDEO 3.0 / Omni：3–15 秒、multi-shot、原生音频、元素引用与 AI Director[^kling-guide][^kling-omni]
- Sora 2：storyboard、stitch、extend、editor、characters / image-to-video 路径[^sora-create][^sora-notes]
- Runway Gen-4：5 或 10 秒、强运动导向提示、I2V 场景下文本重点写 motion[^runway-guide][^runway-create]
- Veo：官方 prompt guide 已经明确把 framing、motion、style、lighting、character、location、action、dialogue 分块[^veo-guide]

**V4 结论**：  
不是回到“多模型通吃”的空泛状态，而是：  
**Canonical Schema 只写镜头真相；模型差异交给 Adapter。**

---

## 3. 总体架构：从“Prompt 工具”升级为“镜头编译系统”

```text
故事 / 剧本
  ↓
Sequence / Scene / Segment 规划
  ↓
Canonical Shot Spec（镜头真相）
  ↓
Asset Binding（角色 / 场景 / 首尾帧 / 动作 / 音频）
  ↓
Shot Beats（multi-shot 时间结构）
  ↓
Hope Validator（语义覆盖 / 连续性 / 合规 / 模型约束）
  ↓
Model Adapter（Seedance 主适配器，可选 Kling / Sora / Runway / Veo）
  ↓
Excel / JSON 导出
  ↓
生成 / 编辑 / 延展
```

### 3.1 五个层级对象

| 层级 | 作用 | 说明 |
| --- | --- | --- |
| Project | 作品层 | 一部短片 / 一集 / 一个 campaign |
| Segment | 叙事片段层 | 对应场景段落、情绪段落或拼接片段 |
| Shot | 镜头层 | 默认一行一个镜头，是核心真相对象 |
| Beat | 节拍层 | multi-shot 内的子段，用于时间轴编译 |
| Clip Group | 生成单元层 | 一次实际提交给模型的执行单元；一个 clip group 可包含 1 个 shot，也可包含多个 shot/beats |

### 3.2 新增三个 ID

| key | 含义 |
| --- | --- |
| `shot_id` | 镜头唯一 ID |
| `beat_id` | beat 唯一 ID |
| `clip_group_id` | 实际生成单元 ID；用于把多个 shot / beat 组织进一次调用 |

---

## 4. 数据库架构

### 4.1 标准版：7 个数据库

| DB | 数据库名 | 用途 |
| --- | --- | --- |
| DB-01 | `global_project_bible`（兼容旧名 `global_seedance_bible`） | 全局故事、导演组策略、模型策略、合规策略 |
| DB-02 | `story_segment_map` | 叙事片段、拼接关系、片段目标 |
| DB-03 | `canonical_shot_board`（兼容旧名 `seedance_cut_board`） | 核心主表，一行一个 shot |
| DB-04 | `character_lock` | 角色一致性锁 |
| DB-05 | `scene_asset_lock` | 场景 / 道具 / 光影 / 材质锁 |
| DB-06 | `asset_registry` | **统一多模态资产库** |
| DB-07 | `shot_beats` | **multi-shot / timeline 的 beat 级结构库** |

### 4.2 轻量版：6 个数据库（过渡期可用）

如果你暂时不想建 DB-07，可以先把 beat 存在 DB-03 的 `timeline_segments_json` 里；但正式生产建议仍拆成 DB-07，否则返修和导出会越来越痛苦。

---

## 5. 枚举与字段轴定义（V4 新增）

### 5.1 取代 V3 的 `generation_mode`

V3 的 `generation_mode` 把“文生 / 图生 / 首尾帧 / 多参考 / 多镜头 / 音频驱动”混在一列里，后续会导致校验与导出爆炸。  
V4 改成四条正交轴：

| 轴 | key | 建议枚举 |
| --- | --- | --- |
| 任务轴 | `task_type` | `generate` / `edit` / `extend` |
| 输入轴 | `input_profile` | `t2v` / `i2v` / `v2v` / `multimodal` |
| 结构轴 | `structure_mode` | `single_shot` / `multi_shot_bundle` |
| 锚点轴 | `frame_anchor_mode` | `none` / `start_frame` / `start_end_frame` |

### 5.2 这四轴如何组合

| 典型场景 | `task_type` | `input_profile` | `structure_mode` | `frame_anchor_mode` |
| --- | --- | --- | --- | --- |
| 纯文生单镜头 | generate | t2v | single_shot | none |
| 图生单镜头 | generate | i2v | single_shot | start_frame |
| 首尾帧转场镜头 | generate | i2v | single_shot | start_end_frame |
| 多参考镜头 | generate | multimodal | single_shot | none / start_frame |
| 多段 15 秒镜头 | generate | multimodal | multi_shot_bundle | none / start_frame |
| 对现有视频做修改 | edit | v2v | single_shot / multi_shot_bundle | none |
| 对现有视频做延展 | extend | v2v | single_shot / multi_shot_bundle | none |

---

## 6. DB-01：`global_project_bible` 固定表头

> 兼容旧名 `global_seedance_bible`。  
> 不建议再把 DB 名称永久绑死在 Seedance 上，但字段里仍保留 Seedance 作为默认主模型。

| key | 等级 | Notion 类型 | 填充方式 | 说明 |
| --- | --- | --- | --- | --- |
| `bible_id` | P2 | Title | 系统 | 全局配置 ID |
| `work_title` | P2 | Text | 用户 | 作品名 |
| `version` | P2 | Text | 系统 | 版本号 |
| `primary_model` | P0 | Select | 用户 | 默认主模型：建议 `dreamina-seedance-2-0-260128` |
| `fallback_models` | P1 | Multi-select | 用户/系统 | 备用导出顺序 |
| `story_core_intent` | P0 | Text | 用户 + Hope | 全片故事核心 |
| `story_emotional_baseline` | P1 | Text | Hope | 全片情绪底色 |
| `committee_directing_strategy` | P1 | Text | Hope-kb | 六类能力融合策略 |
| `story_visual_rules` | P1 | Text | Hope-kb + Hope | 由故事推导出的视觉规则 |
| `motion_performance_rules` | P1 | Text | Hope-kb | 动作与表演总规则 |
| `lighting_color_rules` | P1 | Text + 附件 | Hope-kb | 光影与色彩总规则 |
| `audio_generation_rules` | P1 | Text | Hope-kb + 用户 | 对白 / BGM / SFX / 环境音 / 静默策略 |
| `continuity_rules_global` | P1 | Text | Hope-kb | 连续性总规则 |
| `model_capability_profile` | P2 | Text/JSON | 系统 | **记录各模型真实可用的分辨率 / 时长 / 模式 / 输入上限** |
| `default_task_type` | P0 | Select | 用户 | 默认任务轴 |
| `default_input_profile` | P0 | Select | 用户 | 默认输入轴 |
| `default_structure_mode` | P0 | Select | 用户 | 默认结构轴 |
| `default_duration_sec` | P0 | Number/Select | 用户 | 默认时长 |
| `default_aspect_ratio` | P0 | Select | 用户 | 默认画幅 |
| `default_resolution` | P0 | Select | 用户 | 默认分辨率；**由 model_version 动态校验** |
| `default_generate_audio` | P0 | Checkbox | 用户 | 默认生成音频 |
| `adapter_policy` | P1 | Text | 系统/Hope | 例如：Seedance 主适配、Kling 备份 |
| `compliance_policy` | P1 | Text | Hope-kb | 肖像 / IP / 品牌 / 敏感内容合规规则 |
| `failure_modes_catalog` | P1 | Text | Hope-kb | 模型常见失败模式清单 |
| `export_naming_rule` | P2 | Text | 系统 | 文件命名规则 |
| `migration_notes` | P2 | Text | 系统 | V3 → V4 迁移记录 |

---

## 7. DB-02：`story_segment_map` 固定表头

| key | 等级 | Notion 类型 | 填充方式 | 说明 |
| --- | --- | --- | --- | --- |
| `segment_id` | P2 | Title | 系统 | 片段 ID |
| `bible_link` | P2 | Relation→DB-01 | 系统 | 关联项目圣经 |
| `episode_no` | P2 | Number | 系统 | 系列项目可用 |
| `scene_no` | P2 | Number | 系统 | 场景号 |
| `segment_no` | P2 | Number | 系统 | 片段号 |
| `segment_title` | P2 | Text | Hope | 片段标题 |
| `scene_attr` | P1 | Select | Hope | 内/外、日/夜、现实/回忆/梦境等 |
| `scene_summary` | P1 | Text | Hope | 片段摘要 |
| `segment_goal` | P0 | Text | Hope | 本段叙事目标 |
| `segment_emotion_curve` | P1 | Text | Hope-kb + Hope | 本段情绪曲线 |
| `preferred_structure_mode` | P0 | Select | Hope | 默认 single_shot / multi_shot_bundle |
| `preferred_model_override` | P1 | Select/Text | Hope | 片段级模型覆盖（可空） |
| `continuity_priority` | P1 | Select | Hope-kb | 强连续 / 中连续 / 可独立 |
| `stitch_in_note` | P1 | Text | Hope | 与上一片段衔接说明 |
| `stitch_out_note` | P1 | Text | Hope | 与下一片段衔接说明 |
| `target_duration_sec` | P2 | Number | 系统 | 片段目标时长 |
| `shot_count` | P2 | Rollup | 系统 | 子镜头数量 |
| `segment_export_ready` | P2 | Formula | 系统 | 片段导出是否就绪 |

---

## 8. DB-03：`canonical_shot_board` 固定表头（核心主表）

> 兼容旧名 `seedance_cut_board`。  
> V4 的 DB-03 分为两层：  
> **A. 人类主填字段**：给导演、策划、Hope 使用。  
> **B. 隐藏/系统字段**：给 Validator、Adapter、Exporter 使用。  
> 如果你希望 Notion 侧更简洁，B 层可以只保留在 Hope 内部 schema，不必全部暴露为编辑列。

### 8.1 A 层：人类主填字段

| key | 等级 | 类型 | 填充方式 | 说明 |
| --- | --- | --- | --- | --- |
| `shot_id` | P2 | Title | 系统 | 镜头唯一编号 |
| `segment_link` | P2 | Relation→DB-02 | 系统 | 所属片段 |
| `shot_no` | P2 | Number | 系统 | 片段内镜头号 |
| `global_order` | P2 | Number | 系统 | 全片镜头顺序 |
| `clip_group_id` | P2 | Text | 系统/Hope | 实际生成单元 ID |
| `task_type` | P0 | Select | Hope | `generate / edit / extend` |
| `input_profile` | P0 | Select | Hope | `t2v / i2v / v2v / multimodal` |
| `structure_mode` | P0 | Select | Hope | `single_shot / multi_shot_bundle` |
| `primary_model` | P0 | Select | 默认继承 | 主输出模型 |
| `model_version` | P0 | Text/Select | 默认继承 | 例如 `dreamina-seedance-2-0-260128` |
| `duration_sec` | P0 | Number | Hope | 本镜时长 |
| `aspect_ratio` | P0 | Select | 默认继承 | 画幅 |
| `resolution` | P0 | Select | 默认继承 | 分辨率，按模型版本校验 |
| `generate_audio` | P0 | Checkbox | 默认继承 | 是否生成音频 |
| `image_ref_assets` | 条件P0 | Relation→DB-06 | 用户/Hope | 图像参考资产 |
| `video_ref_assets` | 条件P0 | Relation→DB-06 | 用户/Hope | 视频参考资产 |
| `audio_ref_assets` | 条件P0 | Relation→DB-06 | 用户/Hope | 音频参考资产 |
| `frame_anchor_mode` | 条件P0 | Select | Hope | `none / start_frame / start_end_frame` |
| `start_frame_asset` | 条件P0 | Relation→DB-06 | 用户/Hope | 首帧锚点 |
| `end_frame_asset` | 条件P0 | Relation→DB-06 | 用户/Hope | 尾帧锚点 |
| `reference_control_core` | 条件P0 | Text | Hope | 每个参考锁什么、强度多大、冲突优先级 |
| `committee_role_weights` | P1 | Text/JSON | Hope-kb | 六类导演能力权重 |
| `shot_function` | P1 | Select | Hope-kb | 叙事 / 情绪 / 动作 / 空镜 / 转场 / 蒙太奇 |
| `shot_purpose` | P0 | Text | Hope | **这镜为什么存在** |
| `visual_scene_core` | P0 | Text | Hope | 画面、环境、光影、材质、空气感 |
| `motion_performance_core` | P0 | Text | Hope | 动作、表演、微表情、呼吸、眼动 |
| `camera_directing_core` | P0 | Text | Hope-kb | 景别、角度、焦距感、运镜、稳定性 |
| `audio_directing_core` | P0 | Text | Hope/Hope-kb | 对白、环境音、BGM、SFX、同步点 |
| `continuity_lock_core` | P0 | Text | character/scene lock + Hope-kb | 外貌、服装、空间、轴线、首尾状态等 |
| `shot_beats_link` | 条件P0 | Relation→DB-07 | Hope | multi-shot 时关联 beats |
| `model_override_notes` | P1 | Text | Hope | 某模型专用补充说明 |
| `avoidance_constraints` | P1 | Text | Hope-kb | 模型常见失败避免项与约束说明 |

### 8.2 B 层：隐藏 / 系统字段

| key | 等级 | 类型 | 说明 |
| --- | --- | --- | --- |
| `blocking_map` | P1 | Text/JSON | 人物 / 道具 / 动线 / 站位图式 |
| `focus_target` | P1 | Text | 当前镜头视觉关注点 |
| `lens_feel` | P1 | Select/Text | 24mm 广角感 / 50mm 标准 / 85mm 压缩感等 |
| `screen_direction` | P1 | Text | 主体在屏幕空间中的运动方向 |
| `eyeline_axis` | P1 | Text | 视线轴 / 反打轴线 |
| `entry_state` | P1 | Text | 本镜首状态 |
| `exit_state` | P1 | Text | 本镜末状态 |
| `acting_intent` | P1 | Text | 表演意图：试探 / 防御 / 掩饰 / 讨好等 |
| `cut_reason` | P1 | Text | 为什么在这里切镜 |
| `compiled_prompt_primary` | P0 | Text | 当前主模型最终 prompt |
| `compiled_payload_primary` | P2 | Text/JSON | 当前主模型最终 payload |
| `export_filename` | P2 | Text | 输出文件名 |
| `shot_status` | P2 | Select | 草案 / Ready / 已生成 / 已采纳 / 锁定 / 需重生 |
| `shot_version` | P2 | Number | 版本号 |
| `regeneration_note` | P2 | Text | 重生原因与失败回填 |
| `validation_score` | P2 | Number/Text | 校验评分 |
| `export_ready` | P2 | Formula / 系统字段 | 是否可导出 |

### 8.3 为什么 V4 要加入这些隐藏原子字段

V3 最大的优点是“导演口吻很强”，最大的问题是“机器可读性不够”。  
顶级动画导演分镜并不只是散文，它同时包含：

- 人物怎么站、怎么进、怎么出
- 镜头从哪里看、为什么这样看
- 动作背后的表演意图
- 这刀为什么切、下一刀接什么

V4 通过 `blocking_map` / `lens_feel` / `screen_direction` / `entry_state` / `exit_state` / `acting_intent` / `cut_reason` 这层原子字段，把这些信息从散文里抽出来，供 Hope 做：

1. 语义完整性检查  
2. 跨镜连续性校验  
3. 模型差异化编译  
4. Excel / JSON 稳定导出

---

## 9. DB-04：`character_lock` 固定表头

| key | 等级 | 类型 | 填充方式 | 说明 |
| --- | --- | --- | --- | --- |
| `character_id` | P2 | Title | 系统 | 角色 ID；也是语义引用基础 |
| `character_name` | P0 | Text | 用户/Hope | 角色名 |
| `story_function` | P1 | Text | Hope | 角色在故事中的功能 |
| `appearance_anchors` | P0 | Text | 用户/Hope | 可识别脸部锚点：痣 / 疤 / 眼型 / 发际线 / 体型等 |
| `costume_lock` | P0 | Text | 用户/Hope | 服装 / 材质 / 配饰锁 |
| `expression_range` | P1 | Text | Hope-kb | 表情范围 |
| `motion_personality` | P1 | Text | Hope-kb | 动作气质 |
| `habitual_gestures` | P1 | Text | Hope-kb | 标志性小动作 |
| `voice_identity` | P0/P1 | Text | 用户/Hope | 音色、年龄感、语速、语气习惯 |
| `character_assets` | P0 | Relation→DB-06 | 用户 | 角色参考图 / 视频 / 音频 |
| `forbidden_changes` | P0 | Text | Hope | 禁止变化项 |
| `continuity_lock` | P2 | Checkbox | 系统 | 是否锁定 |
| `character_prompt_fragment` | P1 | Text | Hope 自动生成 | 可复用角色片段 |
| `notes` | P2 | Text | 用户/Hope | 备注 |

---

## 10. DB-05：`scene_asset_lock` 固定表头

| key | 等级 | 类型 | 填充方式 | 说明 |
| --- | --- | --- | --- | --- |
| `scene_asset_id` | P2 | Title | 系统 | 场景 / 道具主键 |
| `asset_type` | P0 | Select | 用户/Hope | 场景 / 道具 / 环境 / 光效 / 天气 |
| `asset_name` | P0 | Text | 用户/Hope | 名称 |
| `story_function` | P1 | Text | Hope | 在故事中的作用 |
| `space_layout_lock` | P0 | Text | Hope | 空间布局与 blocking |
| `key_props_lock` | P0/P1 | Text | Hope | 关键道具与位置 |
| `weather_time_lock` | P0 | Text | Hope | 时间 / 天气 / 光线时间感 |
| `lighting_lock` | P0/P1 | Text | Hope-kb | 光源方向、软硬、色温、阴影边缘 |
| `color_lock` | P0/P1 | Text | Hope-kb | 主色 / 辅色 / 点缀色关系 |
| `material_spec` | P1 | Text | Hope | 地面、墙面、衣料、玻璃等材质 |
| `environment_motion` | P1 | Text | Hope | 雨、风、烟、尘、人流、微粒等 |
| `scene_assets` | P0 | Relation→DB-06 | 用户 | 场景参考图 / 视频 / 音频 |
| `forbidden_changes` | P0 | Text | Hope | 禁止变化项 |
| `continuity_lock` | P2 | Checkbox | 系统 | 是否锁定 |
| `scene_prompt_fragment` | P1 | Text | Hope 自动生成 | 可复用场景片段 |

---

## 11. DB-06：`asset_registry` 固定表头（V4 新增）

> 这是 V4 必须新增的一库。  
> 没有这个库，`reference_assets` 关系最终一定会退化为“在 prompt 里描述参考图”，无法真正做资产绑定、冲突优先级、复用、权限与合规控制。

| key | 等级 | 类型 | 填充方式 | 说明 |
| --- | --- | --- | --- | --- |
| `asset_id` | P2 | Title | 系统 | 资产唯一 ID |
| `asset_name` | P0 | Text | 用户 | 名称 |
| `modality` | P0 | Select | 用户 | `image / video / audio / text / storyboard / keyframe` |
| `asset_role` | P0 | Select | 用户/Hope | `character / scene / prop / start_frame / end_frame / motion / dialogue / sfx / music / style / storyboard` |
| `bind_scope` | P0 | Select | 用户/Hope | `global / segment / shot / beat / character / scene` |
| `linked_character` | P1 | Relation→DB-04 | 用户/系统 | 对应角色（可空） |
| `linked_scene_asset` | P1 | Relation→DB-05 | 用户/系统 | 对应场景 / 道具（可空） |
| `source_path` | P0 | Files / URL / Text | 用户 | 文件路径或存储位置 |
| `source_origin` | P1 | Select/Text | 用户 | 自制 / 用户上传 / 第三方采购 / 官方样例等 |
| `lock_dimensions` | P0 | Text | Hope | 锁脸 / 锁服装 / 锁色彩 / 锁空间 / 锁动态 / 锁音色等 |
| `priority` | P1 | Number / Select | Hope | 冲突优先级 |
| `rights_status` | P0 | Select | 用户 | 已授权 / 待确认 / 禁商用 / 禁使用 |
| `real_person_flag` | P0 | Checkbox | 用户/系统 | 是否涉及真实人物 |
| `public_figure_flag` | P0 | Checkbox | 用户/系统 | 是否涉及公众人物 |
| `brand_logo_flag` | P0 | Checkbox | 用户/系统 | 是否含品牌 logo |
| `copyright_risk_flag` | P0 | Checkbox | 用户/系统 | 是否有 IP 风险 |
| `duration_or_length` | P2 | Number/Text | 系统 | 视频 / 音频时长，或图片张数信息 |
| `availability_status` | P2 | Select | 系统 | 可用 / 限制可用 / 禁用 |
| `notes` | P2 | Text | 用户/Hope | 备注 |

### 11.1 `asset_registry` 的设计要求

1. **所有参考输入必须先入库再引用**  
   不允许在镜头表里直接写“参考图 1、参考图 2”。

2. **语义标签从资产库生成，不手工临时命名**  
   例如：  
   `@char_lin_face_v02`  
   `@scene_alley_dusk_v01`  
   `@startframe_shot_021`  
   `@audio_cafe_ambience_01`

3. **权限与风险字段必须真实存在**  
   `rights_status / real_person_flag / public_figure_flag / brand_logo_flag / copyright_risk_flag` 不是装饰字段，而是决定是否允许导出的硬约束。

---

## 12. DB-07：`shot_beats` 固定表头（V4 新增）

> V3 的 `timeline_segments` 只有一格文本，能跑 Demo，但不利于迭代、局部返修和 Excel 导出。  
> V4 把多镜头 / 多节拍拆成 beat 级结构。

| key | 等级 | 类型 | 填充方式 | 说明 |
| --- | --- | --- | --- | --- |
| `beat_id` | P2 | Title | 系统 | beat 唯一 ID |
| `parent_shot` | P0 | Relation→DB-03 | 系统 | 所属镜头 |
| `beat_order` | P0 | Number | Hope | 顺序 |
| `t_in_sec` | P0 | Number | Hope | 起始时间 |
| `t_out_sec` | P0 | Number | Hope | 结束时间 |
| `beat_goal` | P0 | Text | Hope | 这一拍要完成什么 |
| `visual_beat` | P1 | Text | Hope | 本拍视觉要点 |
| `motion_beat` | P1 | Text | Hope | 本拍动作要点 |
| `camera_beat` | P1 | Text | Hope-kb | 本拍镜头要点 |
| `audio_beat` | P1 | Text | Hope | 本拍声音要点 |
| `transition_note` | P1 | Text | Hope | 本拍与下一拍如何接 |
| `ref_asset_ids` | P1 | Relation→DB-06 | Hope | 该 beat 使用的资产 |
| `continuity_note` | P1 | Text | Hope | 该 beat 的连续性约束 |
| `compiled_block_primary` | P2 | Text | 系统 | 当前主模型下编译后的 block |
| `beat_status` | P2 | Select | 系统 | 草案 / Ready / 锁定 |

### 12.1 什么时候必须建 beat

以下任一条件满足时，`shot_beats` 必须存在：

1. `structure_mode = multi_shot_bundle`
2. 一个镜头内出现多个明显节拍转折
3. 有明确时间点的对白 / 动作 / 镜头运动切换
4. 需要对某一小段进行局部重生或替换

---

## 13. 六大 core 保留，但改为“核心散文 + 原子字段支撑”

> V4 不推翻六大 core。  
> 相反，V4 认为六大 core 是你的方案里最有价值的导演语言层。  
> 但它们不该再单独承担全部真相，而是要由原子字段支撑。

### 13.1 六大 core 的保留清单

1. `visual_scene_core`
2. `motion_performance_core`
3. `camera_directing_core`
4. `audio_directing_core`
5. `continuity_lock_core`
6. `reference_control_core`

### 13.2 新增的原子支撑字段

| 原子字段 | 主要服务于哪个 core | 作用 |
| --- | --- | --- |
| `blocking_map` | visual / motion / continuity | 明确人物、道具、空间关系与走位 |
| `focus_target` | camera / visual | 明确镜头注意力中心 |
| `lens_feel` | camera | 把焦距感单独提出来 |
| `screen_direction` | continuity / motion | 控制左右运动与跨镜方向 |
| `eyeline_axis` | continuity / camera | 控制视线与反打 |
| `entry_state` | continuity / motion | 首状态 |
| `exit_state` | continuity / motion | 尾状态 |
| `acting_intent` | motion | 指明表演意图，不只是动作外壳 |
| `cut_reason` | camera / continuity | 解释为什么此处切镜 |

### 13.3 为什么必须补这层原子字段

世界级动画导演的分镜从来不是“只写得文艺”。  
它同时也是：

- 可拍摄的
- 可切镜的
- 可被团队复现的
- 可被下一镜接住的

V4 的做法不是削弱导演口吻，而是把导演口吻背后的“可执行语法”抽出来。

---

## 14. 六大 core 字段契约（V4 版）

> V3 的强项是“必须信息点很厚”；V4 继续保留这个思路。  
> 但 V4 把“字数硬下限”改为“**基础预算 + 动态预算**”。

### 14.1 基础预算规则

| field | 建议基础预算 | 说明 |
| --- | --- | --- |
| `visual_scene_core` | 120+ 字 | T2V / 弱参考场景使用完整预算 |
| `motion_performance_core` | 140+ 字 | 动作、表演、眼动、节拍是最吃描述密度的 |
| `camera_directing_core` | 80+ 字 | 机位、焦距感、运镜速度、稳定性不能省 |
| `audio_directing_core` | 90+ 字 | 对白时间段、环境音层次、同步点必须写 |
| `continuity_lock_core` | 100+ 字 | 跨镜连续性必须具体到可核查项 |
| `reference_control_core` | 40+ 字 / 资产 | 每个参考至少说明四件事 |

### 14.2 动态预算规则（V4 新增）

#### 情况 A：`t2v + 弱参考`
使用完整基础预算。

#### 情况 B：`i2v + 强首帧 / 强角色参考`
`visual_scene_core` 与 `camera_directing_core` 可降到 **0.7x 基础预算**，但：
- `motion_performance_core`
- `audio_directing_core`
- `continuity_lock_core`
- `reference_control_core`
仍需保持完整密度。

#### 情况 C：`multi_shot_bundle`
shot 级 core 写“整体包络”，beat 级库写“局部节拍”。  
不再强迫把每个 2 秒细节都塞进 shot 级长文本。

#### 情况 D：空镜 / establishing
`motion_performance_core` 可降到 **0.6x–0.7x 基础预算**，但 `visual_scene_core` 与 `continuity_lock_core` 要加厚。

### 14.3 `visual_scene_core` 必须覆盖项（保留 V3 精神）

建议覆盖以下内容：

1. 主体位置与姿态  
2. 时间感 / 空间感  
3. 环境动态  
4. 主光方向与入射角  
5. 光的软硬与阴影边缘  
6. 色温感受  
7. 主 / 辅 / 点缀色关系  
8. 前 / 中 / 后景组织  
9. 主体与背景对比关系  
10. 关键材质  
11. 空气感微动态  
12. 颜色反射 / 次级色回弹  
13. 画内文字说明（无则显式写无）

### 14.4 `motion_performance_core` 必须覆盖项

建议覆盖以下内容：

1. 起始姿态  
2. 主动作  
3. 结束姿态  
4. 节拍三段（预备 / 主动作 / 回稳）  
5. 重量与动量感  
6. 接触反作用  
7. 呼吸  
8. 眼动  
9. 微表情  
10. 身体语言  
11. 二级动画  
12. 停顿设计  
13. 与对白 / 声音同步点  
14. 情绪转折秒点  

**新增硬规则**：  
- `single_shot`：一个 shot 只允许一个主动作  
- `multi_shot_bundle`：每个 beat 只允许一个主动作

### 14.5 `camera_directing_core` 必须覆盖项

1. 景别  
2. 机位角度  
3. 焦距感受  
4. 焦点位置  
5. 景深范围  
6. 构图  
7. 运镜类型  
8. 运镜速度与速度曲线  
9. 稳定性层级  
10. 运动模糊程度  

**V4 新要求**：  
`lens_feel` 不再只埋在散文里，建议同步写入原子字段。

### 14.6 `audio_directing_core` 必须覆盖项

1. 对白 / 无对白  
2. 说话人  
3. 语气与声线  
4. 对白时间段  
5. 环境音层次  
6. 空间混响感  
7. BGM 情绪与前后景关系  
8. BGM / 动作同步点  
9. 关键 SFX 与画面对齐  
10. 距离感  
11. 静默设计

### 14.7 `continuity_lock_core` 必须覆盖项

1. 外貌锚点  
2. 服装 / 配饰细节  
3. 场景布局与道具位置  
4. 时间天气锁  
5. 光源方向 / 色温 / 角度  
6. 色彩方案锁  
7. 轴线  
8. 空气流向  
9. 惯用动作延续  
10. 本镜首状态如何接上镜  
11. 本镜末状态如何交下一镜  

### 14.8 `reference_control_core` 必须覆盖项

每个参考资产至少说明：

1. 语义标签  
2. 锁定维度  
3. 约束强度（强锁 / 参考 / 灵感）  
4. 与其他参考冲突时的优先级  

---

## 15. `committee_role_mix` 改为 `committee_role_weights`

V3 用 `committee_role_mix` Multi-select 表达“导演组能力融合”，但它只能表达“有 / 没有”，无法表达“这镜 60% 表演、25% 调度、15% 连续性”。

### 15.1 V4 建议写法

```json
{
  "scene_staging": 0.20,
  "motion_direction": 0.25,
  "emotion_performance": 0.25,
  "transition_design": 0.10,
  "audio_direction": 0.10,
  "continuity_supervision": 0.10
}
```

### 15.2 如果你不想用 JSON

可退化为：

- `primary_role`
- `secondary_role`
- `support_roles`

但长期来看，`committee_role_weights` 更利于 Hope-kb 与 few-shot 路由。

---

## 16. 编译规则：Canonical → Seedance 主适配器

### 16.1 single_shot 编译顺序（Seedance 主适配器）

```text
[Asset binding / reference_control_core]

[shot_purpose]

[visual_scene_core]

[motion_performance_core]

[camera_directing_core]

[audio_directing_core]

[continuity_lock_core]

[avoidance_constraints]
```

### 16.2 multi_shot_bundle 编译顺序（V4）

```text
[Asset binding / reference_control_core]

[跨段 continuity 说明]

[0.0–X.Xs] Beat 01 block
[X.X–Y.Ys] Beat 02 block
[Y.Y–Z.Zs] Beat 03 block

[avoidance_constraints]
```

### 16.3 为什么 V4 必须走 beat 编译

因为真正需要返修时，你不是想重写整段 15 秒长 prompt，而是想改：

- 第 2.2 秒那一下视线
- 第 4.0 秒到第 5.5 秒的转身
- 第 6.0 秒的自行车铃声
- 第 8.5 秒切近景的速度

beat 化以后，Hope 才能做局部重编译，而不是整段报废。

### 16.4 Seedance 主适配器的关键校验

1. `duration_sec` 必须 4–15 秒  
2. `resolution` 按 `model_version` 校验  
3. `images <= 9`、`videos <= 3`、`audios <= 3`  
4. `start_frame_asset / end_frame_asset` 必须与 `frame_anchor_mode` 一致  
5. `multi_shot_bundle` 必须有 `shot_beats`  
6. `generate_audio = true` 时，`audio_directing_core` 不能空  
7. `edit / extend` 时，`input_profile` 不能为纯 `t2v`

---

## 17. 其他模型的 adapter 只负责翻译，不改真相

### 17.1 Kling adapter

| 方向 | 编译思路 |
| --- | --- |
| 标准 Kling 3.0 | 适合 prompt-driven 多人物、多镜头、multi-shot |
| Kling 3.0 Omni | 适合 reference-driven、一致性更强、元素绑定更重 |
| 编译重点 | 将 `shot_beats` 编译为 multi-shot block；资产绑定映射为 Elements / References；音频维持在 beat 级 |

### 17.2 Sora adapter

| 方向 | 编译思路 |
| --- | --- |
| storyboard | 将 `shot_beats` 编译成 second-by-second storyboard |
| stitch | 将 `clip_group_id` 输出为 stitch 友好的段落 |
| extend | 将 `exit_state` / `continuity_lock_core` 编译为 extend 指令基础 |

### 17.3 Runway adapter

| 方向 | 编译思路 |
| --- | --- |
| I2V | 既然图像已给出主体与风格，文本应重点写 motion、timing、camera |
| T2V | 可同时写视觉与动作，但仍建议简洁直接 |
| 编译重点 | 对强参考场景压缩 visual 重复信息，突出运动与镜头变化 |

### 17.4 Veo adapter

| 方向 | 编译思路 |
| --- | --- |
| 单镜头 | 将 core 字段重排为 framing / motion / style / lighting / character / location / action / dialogue |
| 编译重点 | 保持块状清晰，避免过度冗长 |

---

## 18. Excel 导出：不要只导一列 Prompt

> 你的初衷是通过 Excel 导出对应分镜提示词。  
> V4 建议把 Excel 变成“镜头控制台”，而不是“Prompt 清单”。

### 18.1 最少导出 5 张 Sheet

| Sheet | 用途 | 必备列 |
| --- | --- | --- |
| `shots_master` | 镜头真相表 | `shot_id, clip_group_id, segment_id, task_type, input_profile, structure_mode, duration_sec, aspect_ratio, resolution, shot_function, shot_purpose, visual_scene_core, motion_performance_core, camera_directing_core, audio_directing_core, continuity_lock_core, export_ready` |
| `shot_beats` | 时间轴 / multi-shot 表 | `beat_id, parent_shot, beat_order, t_in_sec, t_out_sec, beat_goal, visual_beat, motion_beat, camera_beat, audio_beat, ref_asset_ids` |
| `asset_registry` | 多模态资产表 | `asset_id, modality, asset_role, bind_scope, source_path, lock_dimensions, priority, rights_status, real_person_flag, public_figure_flag, brand_logo_flag, copyright_risk_flag` |
| `export_seedance_primary` | Seedance 主导出表 | `shot_id, model_version, image_ref_assets, video_ref_assets, audio_ref_assets, compiled_prompt_primary, compiled_payload_primary, export_filename` |
| `validator_report` | 校验报告表 | `shot_id, validation_score, missing_items, asset_conflicts, continuity_warnings, compliance_warnings, export_ready` |

### 18.2 可选导出 Sheet

如果后续要做多模型 fallback，可再加：

- `export_kling_fallback`
- `export_sora_fallback`
- `export_runway_fallback`
- `export_veo_fallback`

但这些不是 V4 最小闭环的必需项。

---

## 19. 自动化校验：Notion 负责轻校验，Hope 负责强校验

### 19.1 Notion / 表层校验（轻）

适合放在 Formula / Relation / Rollup 层：

1. 必填字段是否存在  
2. 枚举是否合法  
3. `duration_sec` 是否在范围内  
4. `resolution` 是否属于当前模型合法档  
5. `frame_anchor_mode` 与首尾帧资产是否匹配  
6. 图 / 视频 / 音频参考数量是否超上限  
7. `multi_shot_bundle` 是否有关联 `shot_beats`

### 19.2 Hope Validator（强）

必须放在 Hope / 代码里做：

1. 六大 core 覆盖项检查  
2. `single_shot` 是否只存在一个主动作  
3. beat 级动作是否冲突  
4. 资产锁定维度是否自相矛盾  
5. 角色 / 场景 / 光源 / 轴线 / 进出状态是否连续  
6. 肖像 / 公众人物 / 品牌 / 版权风险检查  
7. 各模型 adapter 的模式互斥与参数合法性  
8. prompt 编译后的空洞词、黑名单、重复信息压缩

### 19.3 `export_ready` 的定义

只有以下条件全部通过，才允许 `export_ready = true`：

- 字段存在性通过  
- 模式组合合法  
- beat 结构合法  
- 资产与权限合法  
- 连续性无硬冲突  
- 编译成功  
- 主模型 adapter 校验通过

---

## 20. V4 的黑名单与重写策略

### 20.1 黑名单继续保留，但从“绝对阻断”改成“阻断 + 自动重写建议”

仍建议对以下词进行严格处理：

- 电影级、大师级、完美、顶级、专业、高级、精致、精良
- 优雅、自然、流畅、天衣无缝、恰到好处
- 温暖的阳光、柔和的光线、美丽的光影
- 悠扬、动人、优美、震撼、感人
- 令人动容、引人入胜、扣人心弦、淋漓尽致

### 20.2 重写策略

- 抽象情绪 → 可见动作 / 空间状态 / 光影变化
- 抽象美学 → 具体镜头语法
- 抽象音乐评价 → 声音层次、距离、同步点、静默设计
- “保持一致” → 具体可核查项

---

## 21. 轻量迁移说明：V3 → V4 对照表

| V3 项 | V4 替换 / 处理 | 原因 |
| --- | --- | --- |
| `global_seedance_bible` | `global_project_bible`（兼容旧名） | 降低模型绑定深度 |
| `seedance_cut_board` | `canonical_shot_board`（兼容旧名） | Prompt 不是主真相 |
| `generation_mode` | `task_type + input_profile + structure_mode + frame_anchor_mode` | 解耦四个正交轴 |
| `reference_assets` | `image_ref_assets + video_ref_assets + audio_ref_assets + asset_registry` | 真正多模态化 |
| `frame_anchors_spec` | `frame_anchor_mode + start_frame_asset + end_frame_asset` | 更可校验 |
| `timeline_segments` | `shot_beats`（或过渡期 `timeline_segments_json`） | 支持局部返修 |
| `committee_role_mix` | `committee_role_weights` | 能表达占比 |
| `seedance_negative_policy` | `compliance_policy`（DB-01）+ `avoidance_constraints`（DB-03） | 全局合规与镜头约束分层 |
| `seedance_prompt_final` | `compiled_prompt_primary` | Prompt 成为派生字段 |
| `seedance_payload_json` | `compiled_payload_primary` | 保留，但改为 adapter 产物 |
| DB-03 字段数混乱 | V4 明确分 A/B 两层 | 避免 Notion / Excel / 代码表头漂移 |

---

## 22. 实施路线（V4）

### 阶段 1：Schema 迁移（先做）

1. 保留 V3 五库  
2. 新增 `asset_registry`  
3. 新增 `shot_beats`  
4. 给 DB-03 增加四条轴字段与 `clip_group_id`  
5. 给 DB-03 增加隐藏原子字段

### 阶段 2：Hope / Hope-kb 升级

1. few-shot 仍然围绕六大 core 做  
2. 新增原子字段提取与回填  
3. 新增 asset binding 与 conflict priority  
4. 新增 beat 编译器  
5. 新增 validator

### 阶段 3：Excel / 导出链路升级

1. 导出 5 张基础 Sheet  
2. 主适配器先只做 Seedance  
3. 生成后回写 `shot_version / regeneration_note / validation_score`

### 阶段 4：可选的 fallback 扩展

1. 加 Kling adapter  
2. 加 Sora storyboard / stitch adapter  
3. 加 Runway motion-lean adapter  
4. 加 Veo block-style adapter

---

## 23. 这版文档的总判断

V3 的优点是：

- 导演语言强
- 六大 core 合理
- Hope-kb 黄金示例思路正确

V3 的问题是：

- 工程骨架仍偏“Prompt 工具”
- 缺统一资产层
- 缺 beat 层
- 模式轴混合
- 过多语义校验压给表层
- Seedance 专用绑定过深

V4 的目标不是推翻 V3，而是让 V3 真的能成为：

> **结构化分镜真相层 + Seedance 主适配器 + Excel 导出控制台**

如果你的目标是“最终通过 Excel 稳定导出对应的分镜提示词，并能长期迭代”，那 V4 会比 V3 稳得多。

---

## 24. 附录 A：建议固定枚举

### 24.1 `primary_model`

- `dreamina-seedance-2-0-260128`
- `dreamina-seedance-2-0-fast-260128`
- `kling-video-3-0`
- `kling-video-3-0-omni`
- `sora-2`
- `runway-gen-4`
- `veo-3`

### 24.2 `shot_function`

- `narrative`
- `emotional`
- `action`
- `establishing`
- `transition`
- `montage`

### 24.3 `asset_role`

- `character`
- `scene`
- `prop`
- `start_frame`
- `end_frame`
- `motion`
- `dialogue`
- `sfx`
- `music`
- `style`
- `storyboard`

---

## 25. 附录 B：一个推荐的 `committee_role_weights` 模板

```json
{
  "scene_staging": 0.20,
  "motion_direction": 0.20,
  "emotion_performance": 0.25,
  "transition_design": 0.10,
  "audio_direction": 0.15,
  "continuity_supervision": 0.10
}
```

可按镜头类型调整，例如：

- 对话镜：`emotion_performance`、`audio_direction` 提高
- 动作镜：`motion_direction`、`camera_directing` 相关权重提高
- 转场镜：`transition_design`、`continuity_supervision` 提高
- 空镜：`scene_staging`、`continuity_supervision` 提高

---

## 26. 附录 C：一个推荐的 `asset_registry` 命名规则

```text
@{scope}_{entity}_{purpose}_{version}
```

例如：

- `@char_lin_face_v02`
- `@char_yang_voice_v01`
- `@scene_alley_dusk_v03`
- `@prop_cart_close_v01`
- `@startframe_shot_021_v01`
- `@endframe_shot_021_v01`
- `@audio_cafe_ambience_v02`

---

## 27. 附录 D：Seedance 主适配器 payload 示例（示意）

> 以下为 Hope 内部编译示意，不代表官方固定 API 字段名。

```json
{
  "model": "dreamina-seedance-2-0-260128",
  "task_type": "generate",
  "input_profile": "multimodal",
  "structure_mode": "multi_shot_bundle",
  "duration_sec": 10,
  "aspect_ratio": "16:9",
  "resolution": "720p",
  "generate_audio": true,
  "image_refs": [
    "@char_lin_face_v02",
    "@scene_alley_dusk_v03",
    "@startframe_shot_021_v01"
  ],
  "video_refs": [],
  "audio_refs": [
    "@audio_cafe_ambience_v02"
  ],
  "prompt": "[0.0–3.0s] ... [3.0–6.5s] ... [6.5–10.0s] ...",
  "avoidance": "...",
  "metadata": {
    "shot_id": "SH021",
    "clip_group_id": "CG008",
    "version": 3
  }
}
```

---

## 28. 附录 E：世界级动画导演分镜对 V4 的影响点

V4 的设计不是在“模仿某位导演风格”，而是在吸收专业动画导演分镜的工作方式：

1. **分镜是“电影的设计图”**  
   不是只写情绪，而是写画面、切法、台词、演技、状况指示。[^ghibli-storyboard][^ghibli-storyboard-2]

2. **从 sequence → shot → frame 去工作**  
   不是先写一段大 prompt，再希望模型自己切镜。[^disney-process]

3. **shot 本身就是 camera setup**  
   所以 `single_shot` 必须保持“一个主动作、一套镜头策略”。[^disney-layout]

4. **layout 关心 lens、f-stop、camera move、angle、staging、composition**  
   这正是为什么 V4 要把 `lens_feel`、`blocking_map`、`cut_reason` 抽出来。[^disney-layout]

---

## 29. 附录 F：参考来源（官方 / 高可信）

### Seedance / BytePlus
- ByteDance Seed Team – Official Launch of Seedance 2.0  
  <https://seed.bytedance.com/en/blog/official-launch-of-seedance-2-0>
- BytePlus ModelArk – Model list  
  <https://docs.byteplus.com/en/docs/ModelArk/1330310>
- BytePlus ModelArk – Seedance 2.0 series tutorial  
  <https://docs.byteplus.com/en/docs/ModelArk/2291680>
- BytePlus ModelArk – List video generation tasks / audio flag related docs  
  <https://docs.byteplus.com/api/docs/ModelArk/1521675>

### Kling
- Kling AI – VIDEO 3.0 Model User Guide  
  <https://app.klingai.com/global/quickstart/klingai-video-3-model-user-guide>
- Kling AI – VIDEO 3.0 Omni Guide  
  <https://app.klingai.com/global/blog/kling-video-3-omni-multi-shot-native-audio-guide>

### Sora
- OpenAI Help – Creating videos with Sora  
  <https://help.openai.com/en/articles/12460853-creating-videos-with-sora>
- OpenAI Help – Sora Release Notes  
  <https://help.openai.com/en/articles/12593142-sora-release-notes>

### Runway
- Runway Help – Gen-4 Video Prompting Guide  
  <https://help.runwayml.com/hc/en-us/articles/39789879462419-Gen-4-Video-Prompting-Guide>
- Runway Help – Creating with Gen-4 Video  
  <https://help.runwayml.com/hc/en-us/articles/37327109429011-Creating-with-Gen-4-Video>

### Veo
- Google DeepMind – How to create effective prompts with Veo 3  
  <https://deepmind.google/models/veo/prompt-guide/>

### Disney / Ghibli
- Walt Disney Animation Studios – Filmmaking Process  
  <https://disneyanimation.com/process/>
- Walt Disney Animation Studios – Layout & Final Layout  
  <https://disneyanimation.com/process/layout/>
- Studio Ghibli – 絵コンテは「映画の設計図」  
  <https://www.ghibli.jp/ged_01/20making/000508.html>
- Studio Ghibli Museum Shop – 絵コンテに書かれる内容  
  <https://www.ghibli-museum-shop.jp/contents/storyboard/>

---

## 30. 注释

[^seed-official]: ByteDance Seed Team 官方发布页指出，Seedance 2.0 支持混合模态输入，可同时输入最多 9 张图片、3 段视频、3 段音频，并结合自然语言指令。
[^seed-byteplus-count]: BytePlus ModelArk 的 Seedance 2.0 教程检索摘要公开列出：Images 0–9、Videos 0–3、Audio 0–3。
[^seed-model-list]: BytePlus ModelArk 模型列表公开列出了 `dreamina-seedance-2-0-260128` 与 `dreamina-seedance-2-0-fast-260128` 的能力、分辨率、时长、帧率与支持模式。
[^seed-audio-flag]: BytePlus 相关 API 文档说明 `generate_audio`/音频同步参数可用于 Seedance 2.0 / 2.0 fast。
[^kling-guide]: Kling VIDEO 3.0 官方 User Guide 公开写明支持 3–15 秒视频，并强调更适合长镜与更复杂叙事。
[^kling-omni]: Kling VIDEO 3.0 Omni 官方说明强调 native audio、multi-shot、elements / references 与 AI Director。
[^sora-create]: OpenAI 的 Sora 帮助文档公开说明了 prompt 中可写 subject、setting、camera、motion、audio intent，并支持 stitch、extend、editor 等流程。
[^sora-notes]: OpenAI 的 Sora release notes 公开了 storyboard、stitch、extend、characters / image-to-video 等更新。
[^runway-guide]: Runway Gen-4 Prompting Guide 公开说明：Gen-4 生成 5 或 10 秒视频，并在 I2V 场景下强调 prompt 重点写 motion。
[^runway-create]: Runway 的 Creating with Gen-4 文档说明了其 5/10 秒工作方式与图像输入导向。
[^veo-guide]: Google DeepMind 的 Veo prompt guide 将 framing、motion、style、lighting、character、location、action、dialogue 作为关键提示维度。
[^ghibli-storyboard]: 吉卜力官方介绍指出，絵コンテ是把电影画面、卡ット割り、台词、演技与状况指示一镜一镜写出来的“电影的设计图”。
[^ghibli-storyboard-2]: 吉卜力美术馆线上说明指出，絵コンテ里会写故事、角色心情、动作、台词、相机运动、镜头长度等。
[^disney-process]: Disney Animation 官方流程页强调电影是从 sequence 到 shot 再到 frame 逐层协作完成的。
[^disney-layout]: Disney Animation 的 Layout & Final Layout 页面明确提出 lens、f-stop、camera move、angle、staging、composition 等问题。
