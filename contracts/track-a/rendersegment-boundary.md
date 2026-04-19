# RenderSegment Boundary Contract

## 冻结规则

- `RenderSegment` 目标时长为 `30s-90s`
- `RenderSegment` 不允许跨 `NarrativeScene`
- `RenderSegment` 的边界必须可回溯到 `Cut`
- `RenderSegment` 与 `handoff zone` 是独立实体关系

## 最小字段

- `render_segment_id`
- `narrative_scene_id`
- `序号`
- `起始镜头序号`
- `结束镜头序号`
- `目标时长秒`
- `实际时长秒`
