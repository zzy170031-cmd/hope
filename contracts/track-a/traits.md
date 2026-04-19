# Trait Contract

## `PromptRenderer`

负责把结构化输入渲染成可送给模型的提示词包。

```rust
pub trait PromptRenderer {
    type 输入;
    type 输出;
    type 错误;

    fn render(&self, 输入: &Self::输入) -> Result<Self::输出, Self::错误>;
}
```

## `LLMProvider`

负责统一模型调用口径，主路为 Qwen-compatible，fallback 为 GPT-compatible。

```rust
pub trait LLMProvider {
    type 请求;
    type 回复;
    type 错误;

    fn complete(&self, 请求: &Self::请求) -> Result<Self::回复, Self::错误>;
}
```

## `Validator`

负责校验结构、边界、时长与导出前约束。

```rust
pub trait Validator {
    type 输入;
    type 输出;
    type 错误;

    fn validate(&self, 输入: &Self::输入) -> Result<Self::输出, Self::错误>;
}
```

## `Exporter`

负责把冻结后的结构导出为 Excel 17-sheet contract 对应的 workbook 形态。

```rust
pub trait Exporter {
    type 输入;
    type 输出;
    type 错误;

    fn export(&self, 输入: &Self::输入) -> Result<Self::输出, Self::错误>;
}
```
