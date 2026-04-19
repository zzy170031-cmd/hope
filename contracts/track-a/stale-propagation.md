# Stale Propagation Contract

`stale propagation` 冻结为按层级依赖传播，最小粒度覆盖 `Project/Episode/NarrativeScene/RenderSegment/Cut` 五级。

## 事件记录

每条 stale 记录必须包含：

- `来源层级`
- `来源标识`
- `目标层级`
- `目标标识`
- `追踪标识`
- `时间戳`

## 规则

- 任何下层变更都可以标记上层依赖失效
- 传播链必须可追踪，不能只有“已过期”布尔值
- 事件只记录 contract 事实，不在这里实现重算逻辑
