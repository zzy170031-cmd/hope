# Schema Relationships

```mermaid
graph TD
  project --> episode
  episode --> narrative_scene
  narrative_scene --> render_segment
  render_segment --> cut
  render_segment --> handoff_zone
  project --> hard_lock

  project --> stale_propagation_event
  episode --> stale_propagation_event
  narrative_scene --> stale_propagation_event
  render_segment --> stale_propagation_event
  cut --> stale_propagation_event
```

## 约束说明

- `handoff_zone` 只引用 `render_segment` 边界，不内嵌到 `render_segment`
- `stale_propagation_event` 保存 source/target/trace/timestamp
- `render_segment` 不允许跨 `narrative_scene`
- `hard_lock` 是 `project` 级全局约束
