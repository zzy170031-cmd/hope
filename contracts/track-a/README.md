# Track A Contract Freeze

本目录是 Day 1-3 的 Track A 冻结包，只收敛 schema、domain、store、IPC 与 trait 骨架，不扩业务实现。

## 已冻结范围

- `hope-kb` schema DDL
- `hope` 项目库 schema DDL
- `PromptRenderer` trait
- `LLMProvider` trait
- `Validator` trait
- `Exporter` trait
- Tauri IPC contract
- Excel 17-sheet contract
- stale propagation contract
- `RenderSegment` boundary contract
- handoff zone contract
- 中文 token 字段 contract

## 关键硬规则

- `stale propagation` 按层级依赖传播，最小粒度覆盖 `Project/Episode/NarrativeScene/RenderSegment/Cut`
- `stale propagation` 记录 `source/target/trace/timestamp`
- `RenderSegment` 与 `handoff zone` 为独立实体
- `handoff_zone` 引用 `render_segment` 边界，不是 segment 内自由字段
- Excel 17-sheet 使用机器 sheet 名，列头使用中文展示名
- 所有 token 字段默认中文，不带语言后缀

## 文件索引

- [schema-list.md](schema-list.md)
- [schema-relationships.md](schema-relationships.md)
- [traits.md](traits.md)
- [ipc.md](ipc.md)
- [excel-17-sheet.md](excel-17-sheet.md)
- [stale-propagation.md](stale-propagation.md)
- [rendersegment-boundary.md](rendersegment-boundary.md)
- [handoff-zone.md](handoff-zone.md)
- [chinese-token-fields.md](chinese-token-fields.md)
- [ddl/hope-kb-0001-init.sql](ddl/hope-kb-0001-init.sql)
- [ddl/hope-project-0001-init.sql](ddl/hope-project-0001-init.sql)
