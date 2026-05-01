# Desktop Anime Long Sample Library Plan

Status: full 21-scene long-sample draft library, needs human review.
Thread: Hope桌面端-文本资源线程-.
Repo: `E:\codex\hope-desktop-shell`.
Branch: `codex/desktop-shell`.
Anchor: `40b779d`.

## Goal

建立 KB 联动型动漫长文本样本库，用于支撑 Hope KB 写作组、导演组、动漫结构规则、Prompt Knowledge、确认稿协议和后续 cross-drift / pre-403 gate 的质量参考。

样例是 craft pattern、结构参考、导演调度参考和测试 source，不是用户 source 的替代物，也不是唯一正确写法。

## Runtime Boundary

样例用途：

- KB 写作组 / 导演组参考。
- 确认稿协议质量标尺。
- cross-drift / pre-403 gate 测试 source。
- 场景类型表达参考。
- AI 生图 / AI 视频 / 人工剪辑服务锚示范。

样例禁止用途：

- 不得覆盖用户原文。
- 不得覆盖 accepted snapshot。
- 不得覆盖 StoryFactFrame。
- 不得覆盖当前 scene type / duration。
- 不得把样例人物、道具、地点、世界观带入用户任务。
- 不得直接作为 `prompt_text` 原文拼接。
- 不得把样例当成唯一正确写法。
- 不得因为样例里有武器/道具，就给所有同类任务新增武器/道具。

优先级：

```text
用户事实 > accepted snapshot > StoryFactFrame > 当前 scene/duration > KB rule pack > 样例参考
```

运行时只能使用：

- sample_id。
- scene_type。
- compact sample summary。
- craft pattern。
- writing/director rule tags。
- negative drift tags。
- modern reference pattern。

运行时不能使用：

- 大段 raw sample_text。
- 样例专有人名/道具/世界观。
- 样例剧情原文。
- 样例作为用户 source 的替代物。

## KB Person Alias Policy

样本库对外使用代号承接 KB 人物与作品参考，避免把真实作者、真实导演或真实作品名变成运行时模仿目标。

- 写作组候选来源使用 `writer_A/B/C...` 与 `work_A/B/C...`。
- 导演组候选来源使用 `director_A/B/C...`。
- 机器锚仍可保留 KB 内部 ID，例如 `director_01`、`director_rule_01`、`case_01`，用于追溯本地 KB 记录。
- 不在运行时把真实作者、真实导演、真实作品名作为 prompt 目标。
- 只使用 craft pattern、director rule tags、scene taxonomy、negative drift tags 和 compact summary。

## KB Linkage

本样本库读取并联动以下本地 KB：

- `E:\codex\hope-kb\docs\prompt-knowledge-core-v0.2.md`：Hope runtime 只能消费 verified snapshot outputs，且必须 summary-only；不得泄漏 raw KB rows、raw prompt body 或完整来源内容。
- `E:\codex\hope-kb\docs\kb-full-chain-content-production-routing-draft-2026-04-26.md`：用户事实和连续性优先，写作连续性、场景表达、导演调度、镜头语言依次提供压缩建议。
- `E:\codex\hope-kb\docs\kb-seed-v0.1.md`：提供导演 profile、视觉语言、摄影术语、连续性规则、prompt template、代表 cut 样例等种子分类。
- `E:\codex\hope-kb\docs\kb-source-index-v0.1.md`：外部来源优先规则，先用 storyboard / animation 基础，再用下游目标环境来源，最后使用团队归纳。
- `E:\codex\hope-kb\seed\v0.1\manga_structure_rules.json`：5 条动漫结构规则，分别为 3 秒钩子、情绪点密度、悬念结尾、角色标签化、短台词与气泡约束。
- `docs/hope-anime-script-confirmation-protocol.md`：确认稿正文必须绑定人物名称/代号、角色功能、人物关系、角色表演锚、基础/复杂场景、视频运动锚、目标时长/节奏落点、禁止新增。
- `docs/hope-story-fact-frame-storyboard-binding-contract.md`：StoryFactFrame 是约束框，不是扩写源；样例不得越过用户输入、accepted rewrite、scene type、duration。

临时稳定规则 ID：

- `kb:manga_rule_01_hook`
- `kb:manga_rule_02_emotion_density`
- `kb:manga_rule_03_suspense_end`
- `kb:manga_rule_04_character_legibility`
- `kb:manga_rule_05_short_dialogue`
- `kb:prompt_knowledge_core_summary_only`
- `kb:full_chain_routing_user_facts_first`
- `kb:director_group_spatial_scheduling`
- `kb:writing_group_visible_action`
- `kb:confirmation_protocol_anchor_contract`
- `kb:story_fact_frame_no_invention`

## Source And Copyright Strategy

本轮调整为现代案例优先、公版古典补充：

- 优先参考近五年动画、漫剧、国漫、成人动画、游戏改编动画、网文改编、短剧、AI 视频创作流程和 AI 视频工具公开能力。
- 近十年高质量动画影视案例可作为补充参考。
- 公版古典只作为二级结构库，用于补充军阵、奇观、行旅、群像、异闻、权谋等基础桥段，不作为现代样本主味道。
- 现代版权作品只能提取结构、节奏、人物关系、镜头调度、动作组织、场景组织和制作流程，不把长段原文写入仓库。
- 不照搬现代作品角色名、专有世界观、专有设定、专有道具或标志性桥段。
- 最终进入 tests/docs 的 sample_text 优先使用 Hope 原创样本，且声明 sample is reference, not source of truth。

## External Source Coverage

现代动画/影视制作与案例优先：

- Netflix Tudum Arcane season two behind the scenes: https://www.netflix.com/tudum/features/arcane-season-two-behind-the-scenes
- Riot / Arcane Bridging the Rift docuseries playlist: https://www.youtube.com/playlist?list=PLbAFXJC0J5GYEkfxnGTWnvgcEypgBeAb5
- Sony Pictures Imageworks Spider-Verse production page: https://www.imageworks.com/our-craft/feature-animation/movies/spider-man-spider-verse
- Sony Pictures Imageworks Across the Spider-Verse page: https://www.imageworks.com/index.php/our-craft/feature-animation/movies/spider-man-across-spider-verse
- Disney Animation Encanto process: https://disneyanimation.com/process-encanto/
- 中国青年报：科幻动画剧集《灵笼》第二季收官: https://m.cyol.com/gb/articles/2025-08/05/content_wd5eQ5FRVy.html
- 中新网湖北：《灵笼》第二季开播: https://www.hb.chinanews.com.cn/news/2025/0525/417311.html

动画/影视制作流程：

- Toei Animation production process: https://corp.toei-anim.co.jp/en/company/animation_production.html
- Disney Animation Layout: https://www.disneyanimation.com/process/layout/
- Toon Boom layout posing: https://learn.toonboom.com/modules/layout-cleanup/topic/what-is-layout-posing
- Toon Boom pre-production storyboard: https://learn.toonboom.com/modules/animation-workflow/topic/pre-production
- Adobe Animation Storyboarding: https://www.adobe.com/creativecloud/animation/discover/animation-storyboarding.html

剧本/分镜格式：

- Screenplay.com Basic Screenplay Format: https://screenplay.com/pages/basic-screenplay-format
- Final Draft screenplay format: https://www.finaldraft.com/learn/how-to-format-a-screenplay/
- Wikipedia Storyboard: https://en.wikipedia.org/wiki/Storyboard
- Wikipedia Screenplay: https://en.wikipedia.org/wiki/Screenplay

AI 视频生成能力：

- ByteDance Seedance 1.0: https://seed.bytedance.com/en/seedance
- ByteDance Seedance 2.0 launch: https://seed.bytedance.com/blog/seedance-2-0-official-launch
- Kling VIDEO 3.0 Omni guide: https://kling.ai/quickstart/klingai-video-3-omni-model-user-guide
- Kling VIDEO 3.0 Motion Control guide: https://kling.ai/quickstart/motion-control-user-guide

国内动画教育：

- 中国传媒大学动画与数字艺术学院: https://animation.cuc.edu.cn/
- 中国传媒大学动画与数字艺术学院专业介绍: https://animation.cuc.edu.cn/_upload/tpl/02/7a/634/template634/programmes-zh.html
- 教育部：北京电影学院动画学院特色办学纪实: https://www.moe.gov.cn/jyb_xwfb/moe_2082/s6236/s6415/201207/t20120711_139111.html
- 中国网中国动漫：北京电影学院动画学院公开资料: https://animation.china.com.cn/2024-03/26/content_42735981.html

公版文学作为补充：

- Project Gutenberg 西遊記: https://www.gutenberg.org/ebooks/23962
- Project Gutenberg 三國志演義: https://www.gutenberg.org/ebooks/23950
- Project Gutenberg 三國志: https://www.gutenberg.org/ebooks/25606
- Project Gutenberg 水滸傳: https://www.gutenberg.org/ebooks/23863
- Project Gutenberg 聊齋志異: https://www.gutenberg.org/ebooks/51828
- Project Gutenberg 山海經: https://www.gutenberg.org/ebooks/25288

## Sample Library Structure

机器可读索引与全量 21 篇草稿样本文本存放在：

```text
tests/qa/desktop-anime-scene-long-samples.index.json
```

结构：

- `library_id`：样本库 ID。
- `runtime_usage_boundary`：运行时可用/禁用字段边界。
- `reference_sources`：联网资料 URL 与用途。
- `scene_index`：21 个场景类型的全量规划。
- `long_samples`：21 个场景类型的原创长样本草稿，统一标记为 `draft_needs_review`。
- `next_step`：全量草稿完成后的人工审核与边界约束。

## Scene Planning Summary

完整字段见 JSON `scene_index`。每个场景均包含 `sample_theme`、`character_pattern`、`relationship_pattern`、`writing_group_focus`、`director_group_focus`、`kb_rule_ids`、`public_reference_pattern`、`expected_motion_anchor`、`expected_editing_rhythm`、`forbidden_drift`、`future_sample_status`。

1. `hot_blood_battle` / 热血战斗：双人守线、逆势爆发、近身动作与情绪峰值。
2. `ensemble_performance` / 群像表演：多人目标错位、轮流接力、视线与前后景分层。
3. `emotional_dialogue` / 情绪对话：旧关系再确认、沉默动作替代心理直说、短台词推进。
4. `encounter_performance` / 相遇表演：陌生双方试探、误会解除、空间距离变化。
5. `field_chase` / 场域追逐：固定场域内追与躲、障碍利用、方向一致性。
6. `spectacle_showcase` / 奇观展示：大景观从人物反应进入，不用景观覆盖人物事实。
7. `daily_healing` / 日常治愈：低冲突修复、手上小动作、柔性节奏。
8. `guoman_hot_blood_combat` / 国漫热血打斗：力量释放、招式起承转合、动作锚清晰。
9. `guoman_ensemble_performance` / 国漫群像表演：团队分工、战术接力、人物功能标签。
10. `ink_wuxia_combat` / 水墨武打：留白、节奏停顿、动作轨迹如笔势但不改事实。
11. `eastern_spectacle` / 东方奇观：山河、云海、城郭、仪式感调度。
12. `xianxia_action` / 仙侠动作：高低位移、法术可视化、禁止新神器/新门派。
13. `urban_fantasy` / 都市奇幻：日常空间异常化、现实物件边界清楚。
14. `chinese_war_formation` / 国战军阵建立：军阵层级、旗鼓节奏、沙尘与视线调度。
15. `weapon_highlight` / 武将兵器高光：兵器作为动作节点，不把兵器升级成身份设定。
16. `council_strategy` / 朝堂军帐权谋：站位、沉默、递物、短句压力。
17. `siege_defense` / 多军团攻城：城墙、梯、门、火线、守攻关系清楚。
18. `slg_sandbox_view` / 沙盘战略视口：俯瞰、棋子/路线/区域反馈，不替代剧情。
19. `slg_march_encirclement` / 行军轨迹合围：多路推进、路线闭合、时间节奏可视化。
20. `slg_city_growth` / 城建演进反馈：建筑状态变化、资源流向、反馈循环。
21. `slg_battle_report` / 战报 UI：战后摘要、损益关系、信息层级与镜头式 UI 节奏。

## Full Draft Long Samples

当前已补齐 21 个 3000-5000 字 Hope 原创长样本草稿：

- 每个 `scene_type` 均有 1 篇完整 `long_sample`。
- 首批 3 篇保留为 `draft_needs_review`，不标记为人工审核已通过。
- 新补 18 篇同样标记为 `draft_needs_review`。
- 每篇样本均保留 `not_source_of_truth=true`、代号化 `writer_A / director_A` 体系、KB rule pack、prompt/image-video/human-editing notes 与 forbidden drift。

全量样本仍只作为 reference / craft pattern / 测试 source，不得作为用户 source、accepted snapshot、StoryFactFrame、当前 scene type 或 duration 的替代物。后续只按人工审核意见修订，不因完成全量草稿而进入 403-case、打包或发布。

## Prompt And Service Notes

Prompt service 可使用：

- `visual_description`：源事实绑定的地点、空间压力、可见动作、氛围和场景类型表达。
- `character_action`：角色表演锚、动作、表情、微动作、语气和关系压力。
- `camera_movement`：服务动作、情绪或规模的推拉摇移、跟拍、俯仰、切换。
- `prompt_text`：从 accepted facts 和 anchors 生成的干净下游包装。

Prompt service 禁止使用：

- 大段样例原文。
- 样例专有人名、道具、地点、世界观。
- raw KB rows。
- raw prompt body。
- 与用户事实冲突的样例桥段。

Image/video service 使用方式：

- 生图：提取主体、空间、动作锚和气氛，不复制样例角色设定。
- 视频：提取镜头段落、动作路径、运动锚、剪辑节奏。
- 主体一致性：使用用户已确认人物/代号，不从样例继承外观。
- 减少运动漂移：每段给出明确起点、运动方向、接触点、落点、停顿。

人工剪辑使用方式：

- 起势：3 秒内给出冲突、情绪或视觉锚。
- 推进：每 15-20 秒出现信息或情绪波峰。
- 转折：让关系、位置或目标发生可见变化。
- 情绪峰值：用动作和镜头落点承载，不只用旁白。
- 结尾落点：保留下一段动力或明确收束。
- 可剪辑段落：按 hook、setup、pressure、turn、peak、landing 切分。

## Expansion Rule

合理补全允许出现武器、道具、装备、临时持物、场面物件，前提是符合场景类型、动作逻辑、目标时长和当前场景，并且不改变核心剧情。

禁止新增高风险事实：

- 神器。
- 专属装备。
- 新身份。
- 新阵营。
- 新人物。
- 新伤势。
- 新人数规模。
- 改变地点。
- 改变剧情因果。

不写人物画像化外观：

- 发型。
- 发色。
- 脸型。
- 五官。
- 身材。
- 人物参考图。

## Next Step

全量 21 篇长样本草稿已补齐。下一步是人工审核 21 篇样本，按明确修改意见修订 `docs/desktop-anime-long-sample-library-plan.md` 与 `tests/qa/desktop-anime-scene-long-samples.index.json`。在审核完成前，不标记 approved，不进入 403-case，不打包，不 push，不改 runtime/UI/runner/scripts。运行时仍只能使用 compact sample summary、craft pattern、writing/director rule tags、negative drift tags 与 modern reference pattern，不得读取或拼接大段 raw sample_text。
