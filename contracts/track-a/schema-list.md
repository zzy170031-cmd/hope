# Schema List

## `hope-kb`

| machine table | 中文用途 | 关键字段 |
| --- | --- | --- |
| `schema_meta` | schema 元信息 | `schema_name`, `schema_version`, `created_at` |
| `kb_snapshot` | 快照登记 | `snapshot_id`, `snapshot_hash`, `seed_format`, `created_at` |
| `director_profile` | 导演 profile | `director_profile_id`, `导演名`, `定位`, `主风格`, `镜头偏好`, `连续性偏好` |
| `director_cut_sample` | 导演代表 cut 样例 | `director_profile_id`, `样例标题`, `样例内容`, `代表性说明` |
| `committee_template` | committee 模板 | `template_id`, `模板名`, `适用场景`, `成员构成`, `职责描述` |
| `visual_term` | 视觉语言术语 | `term_id`, `中文术语`, `类别`, `定义`, `别名` |
| `cinematography_term` | 中文摄影术语 | `term_id`, `中文术语`, `类别`, `定义`, `别名` |
| `continuity_rule` | 连续性规则 | `rule_id`, `规则名`, `规则说明`, `适用层级` |
| `prompt_template` | 最小 prompt 模板 | `template_id`, `模板名`, `适用层级`, `模板正文`, `输入字段清单` |
| `seed_import_batch` | 种子导入批次 | `batch_id`, `来源`, `seed_format`, `content_hash`, `imported_at` |

## `hope`

| machine table | 中文用途 | 关键字段 |
| --- | --- | --- |
| `schema_meta` | schema 元信息 | `schema_name`, `schema_version`, `created_at` |
| `project` | 项目 | `project_id`, `标题`, `状态`, `目标时长分钟` |
| `episode` | Episode | `episode_id`, `project_id`, `序号`, `标题`, `目标时长分钟` |
| `narrative_scene` | NarrativeScene | `narrative_scene_id`, `episode_id`, `序号`, `标题`, `内容摘要` |
| `render_segment` | RenderSegment | `render_segment_id`, `narrative_scene_id`, `序号`, `起始镜头序号`, `结束镜头序号`, `目标时长秒` |
| `cut` | Cut | `cut_id`, `render_segment_id`, `序号`, `镜头描述`, `对白`, `时长秒` |
| `handoff_zone` | handoff zone | `handoff_zone_id`, `render_segment_id`, `起始边界`, `结束边界`, `边界类型` |
| `hard_lock` | project 级全局锁 | `hard_lock_id`, `project_id`, `锁名`, `锁值`, `作用范围` |
| `stale_propagation_event` | 陈旧传播记录 | `stale_event_id`, `来源层级`, `来源标识`, `目标层级`, `目标标识`, `追踪标识`, `时间戳` |
