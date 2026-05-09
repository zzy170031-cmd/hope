# Hope 桌面端 Gate Evidence Ladder 契约
<small>Hope desktop gate-evidence ladder contract</small>

状态：草案即执行。
<small>Status: draft but immediately executable.</small>

适用范围：single case、live3、required pool、`full16`、`403-case`、package / release preflight。
<small>Scope: single case, live3, required pool, `full16`, `403-case`, and package/release preflight.</small>

## 1. 契约目标
<small>1. Contract goal</small>

把 `HOPE-GATE-016` 从“口头提醒不要过读”升级成分层 acceptance contract。
<small>Upgrade `HOPE-GATE-016` from a verbal warning into a layered acceptance contract.</small>

## 2. Gate Ladder
<small>2. Gate ladder</small>

正式层级只能按以下顺序推进：
<small>The formal ladder may advance only in the following order:</small>

1. single case
2. `qwen3.6-plus` live3 3-case
3. required text model pool live3
4. `full16`
5. `403-case`
6. package / release preflight

任何一层都不得被上读成更高层。
<small>No layer may be over-read as a higher one.</small>

## 3. 每层 Gate 的 Acceptance Evidence
<small>3. Acceptance evidence for each gate</small>

single case：
<small>single case:</small>

- 只证明单案当前路径可执行。
- 必须带 fresh shell / provider / validator / cleanup 摘要。
- 不得上读成 live3 或 pool。

`qwen3.6-plus` live3 3-case：
<small>`qwen3.6-plus` live3 3-case:</small>

- `expected=3`
- `executed=3`
- `passed=3`
- `failed=0`
- `not_run=0`
- `no_http_403=true`
- `no_live_fallback=true`
- `fallback_used=false`
- `local_candidate=false`
- `rows_match=true`
- `row_diffs=[]`
- source grounding / binding / prompt boundary 全部为 fresh pass

required text model pool live3：
<small>required text model pool live3:</small>

- 每个 callable required text model 都有 fresh 3/3 live3 evidence。
- 至少五个 callable text models 通过。
- 不可用模型要被显式记录为 unavailable reason，而不是静默移除。

`full16`：
<small>`full16`:</small>

- required pool live3 已 fresh 通过。
- 16 个 live WebView2/CDP cases 全部完成。
- `script_goal`、`scene_type`、`duration`、source text、accepted snapshot、StoryFactFrame、prompt boundary、rows_match 全部通过。
- Core Challenger 与 Audit Specialist 接受证据。

`403-case`：
<small>`403-case`:</small>

- `full16` 已 fresh 通过。
- 403 matrix 的进入已被明确打开。
- shell / provider / validator / ownership / cleanup 没有未关闭 blocker。

package / release preflight：
<small>package / release preflight:</small>

- shell gate 绿色。
- `403-case` 已通过，或 controller 明确接受降级 gate。
- dirty ownership 已收口。
- release-like rebuild 可重复。
- 无 secret 泄露风险。

## 4. 明确禁止的上读
<small>4. Explicitly forbidden upward reads</small>

以下内容都不能自动升级成更高 gate 通过：
<small>None of the following may automatically upgrade into a higher gate pass:</small>

- single case
- `certifier_ok=true`
- targeted tests passed
- executable readiness
- one model pass
- old artifact
- comparison artifact
- stale binary

## 5. Stale / Reference-Only 降级规则
<small>5. Stale/reference-only downgrade rules</small>

source、runtime、shell、validator、build provenance 任一相关变化后，旧证据自动降级。
<small>Any relevant change in source, runtime, shell, validator, or build provenance automatically downgrades prior evidence.</small>

自动降级触发器：
<small>Automatic downgrade triggers:</small>

- `app/src/main.rs` 变化
- `app/src/runtime.rs` 变化
- shell / runner / validator 脚本变化
- Tauri build provenance 变化
- `ui/dist` 变化
- target URL 从 `tauri.localhost` 漂移到 devUrl

降级后只能写成 `stale` 或 `reference-only`，不得继续驱动当前 gate。
<small>After downgrade, evidence may only be labeled `stale` or `reference-only` and must not drive the current gate.</small>

## 6. `ready_for_full16` 前置条件
<small>6. Preconditions for `ready_for_full16`</small>

只有以下条件同时满足，才允许写 `ready_for_full16`：
<small>`ready_for_full16` may be written only when all of the following hold:</small>

- shell startup gate 绿色
- provider hard gate 满足
- `HOPE-CONTRACT-007` 没有 fresh source-grounding blocker
- `qwen3.6-plus` live3 3-case 已 fresh 通过
- required text model pool live3 已 fresh 通过
- dirty ownership 可解释

## 7. `ready_for_package` 前置条件
<small>7. Preconditions for `ready_for_package`</small>

只有以下条件同时满足，才允许写 `ready_for_package`：
<small>`ready_for_package` may be written only when all of the following hold:</small>

- shell startup gate 绿色
- release-like provenance 绿色
- `full16` 已 fresh 通过
- `403-case` 已通过，或 controller 明确接受降级 gate
- dirty ownership 已收口
- StopOnly cleanup 没有残留 blocker

## 8. 回报书写要求
<small>8. Report-writing requirements</small>

每次 gate 汇报都必须显式写：
<small>Each gate report must explicitly state:</small>

- 当前层级
- 已通过的下层
- 尚未解锁的上层
- 对应 evidence 路径 / 时间 / freshness
- 是否存在 `reference-only` 或 `stale` 比较项

如果当前证据只够支撑较低层，就必须停在较低层。
<small>If current evidence supports only a lower layer, the report must stop at that lower layer.</small>

## 9. 2026-05-08 Evidence Freshness Addendum
<small>9. 2026-05-08 evidence freshness addendum</small>

Targeted, `full16`, and `formal403` evidence must inherit:
<small>Targeted, `full16`, and `formal403` evidence must inherit:</small>

```text
docs/hope-field-aware-story-contract.md
docs/hope-provider-failover-contract.md
docs/hope-qa-evidence-freshness-contract.md
```

Every artifact must record:
<small>Every artifact must record:</small>

- `workspace`
- `branch`
- `HEAD`
- `origin_HEAD`
- `dirty_patch_hash`
- `release_exe_sha256`
- `runtime_rs_sha256`
- `runner_sha256`
- `certifier_sha256`
- `matrix_sha256`
- `kb_mapping_sha256`
- `gate_level`
- `case_id`
- `parent_gate_summary_hash`
- `started_at`
- `completed_at`

If runtime, UI, runner, certifier, matrix, KB mapping, or release build
provenance changes, older evidence automatically becomes `stale` /
`reference-only`.
<small>If runtime, UI, runner, certifier, matrix, KB mapping, or release build provenance changes, older evidence automatically becomes `stale` / `reference-only`.</small>

Before any business gate, the source integrity preflight must prove `runtime.rs`
is UTF-8 readable and the sentinels `热血战斗`, `场域追逐`, `危局`, `林峰`, and
`苏瑶` are intact.
<small>Before any business gate, the source integrity preflight must prove `runtime.rs` is UTF-8 readable and the sentinels `热血战斗`, `场域追逐`, `危局`, `林峰`, and `苏瑶` are intact.</small>
