# Chinese Token Fields Contract

## 冻结规则

- 所有 token 字段默认中文
- 不带语言后缀
- 不做中文/英文双字段 shadow
- machine identifier 与 token 字段分层处理，不能混用

## 示例

- `导演名`
- `项目标识`
- `目标时长分钟`
- `镜头描述`
- `起始边界`
- `trace_id` 仅作为 machine trace 标识，不进入 token 字段主命名

## 禁止项

- `name_cn`
- `title_zh`
- `description_en`
- 任何为了方便而追加的语言后缀字段
