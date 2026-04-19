# Tauri IPC Contract

## 入口命令

- `project_create_or_switch`
- `writer_entry_snapshot`
- `storyboard_rendersegment_cut_preview_snapshot`
- `validation_export_panel_snapshot`

## 最小请求形状

- 项目创建/切换：`项目标识`、`项目名`
- Writer 四层查看入口：`项目标识`
- Storyboard / RenderSegment / Cuts 预览入口：`项目标识`、`集数标识`、`叙事场景标识`、`渲染片段标识`
- Export / Validation 面板入口：`项目标识`

## 约束

- IPC 只传 contract payload，不直接触碰 SQLite
- payload 字段采用中文 token 字段时，不追加语言后缀
- IPC 失败返回必须可映射到 Validator / Exporter 口径
