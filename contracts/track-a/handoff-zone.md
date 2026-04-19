# Handoff Zone Contract

`handoff zone` 是独立实体，引用 `render_segment` 边界，不作为 segment 内自由字段。

## 最小字段

- `handoff_zone_id`
- `render_segment_id`
- `起始边界`
- `结束边界`
- `边界类型`
- `备注`

## 约束

- 一个 `handoff_zone` 必须能定位到唯一 `render_segment`
- `handoff_zone` 不替代 `render_segment` 边界本身
