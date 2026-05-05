# Hope 桌面端 Validator Source Grounding 契约
<small>Hope desktop validator source-grounding contract</small>

状态：草案即执行。
<small>Status: draft but immediately executable.</small>

适用范围：token/entity classification、source role grounding、StoryFactFrame 绑定、row-level validator 归因。
<small>Scope: token/entity classification, source-role grounding, StoryFactFrame binding, and row-level validator attribution.</small>

## 1. 契约目标
<small>1. Contract goal</small>

把 `HOPE-CONTRACT-007` 从“局部实现已修、fresh closure 未复证”升级成稳定的 validator 判断边界。
<small>Turn `HOPE-CONTRACT-007` from a partially fixed implementation issue into a stable validator decision boundary.</small>

## 2. 两层判断边界
<small>2. Two-layer decision boundary</small>

token/entity classification 与 source role grounding 不是同一层。
<small>Token/entity classification and source-role grounding are not the same layer.</small>

- token/entity classification 只回答“这段文本像不像候选实体、角色片段、描述片段或抽象主体”。
- source role grounding 只回答“这个候选是否能绑定回 source 中的角色槽位或 source-bound fragment”。
- 不能用一层的成功掩盖另一层的失败。

## 3. Source-Bound Fragment 规则
<small>3. Source-bound fragment rules</small>

`主角初`、`初现`、`初现于`、`主角` 这类片段必须先按 source-bound fragment 处理，再决定是否升级为 grounded role。
<small>Fragments such as `主角初`, `初现`, `初现于`, and `主角` must first be handled as source-bound fragments before deciding whether they upgrade to grounded roles.</small>

最低要求：
<small>Minimum requirements:</small>

- 片段必须来自 source、accepted rewrite、StoryFactFrame 或其允许的规范化映射。
- 片段必须能解释为 source 中已经存在的角色、位置关系或事件片段。
- 不能因为模型“像是在说某个人”就自动推断出真实人物候选。

## 4. Bare Source Role `主角` 的 Grounded 条件
<small>4. Grounded conditions for the bare source role `主角`</small>

bare source role `主角` 只有在以下条件同时成立时才算 grounded：
<small>The bare source role `主角` is grounded only when all of the following hold:</small>

- source 或 accepted rewrite 已明确存在该角色槽位。
- 当前 row / field 与该槽位的 source fragment 有可追踪命中。
- `bind_source_role_hit=true`，且命中理由不是纯模型风格解释。
- 没有被更具体、且同样 grounded 的 source-bound 候选覆盖。

若只存在抽象叙述、镜头主语或气氛主语，而没有可绑定角色槽位，则 `主角` 只能保持 `ungrounded_source_role`。
<small>If only an abstract narrator, camera subject, or atmospheric subject exists without a bindable role slot, `主角` remains `ungrounded_source_role`.</small>

## 5. Visual / Abstract Subject 与真实人物候选的区别
<small>5. Distinguish visual/abstract subjects from real-person candidates</small>

- visual subject：画面主语、镜头焦点、动作承受体。
- abstract subject：情绪、压迫关系、空间压力、场景主体。
- real-person candidate：可绑定到 source / StoryFactFrame 角色表的实际人物候选。

只有 real-person candidate 才能进入人物槽位判定。
<small>Only real-person candidates may enter person-slot validation.</small>

visual subject 或 abstract subject 不得被偷渡成真实人物。
<small>Visual or abstract subjects must not be smuggled into real-person slots.</small>

## 6. Invented Name Hard-Fail
<small>6. Invented-name hard fail</small>

invented names 仍然必须 hard-fail。
<small>Invented names must still hard-fail.</small>

以下情况一律不允许以“风格合理”过关：
<small>The following may never pass by claiming they are stylistically reasonable:</small>

- source、accepted rewrite、StoryFactFrame 都不存在的人名。
- 只在模型输出里第一次出现的人名。
- 为了让 `主角` 更“自然”而补出的拟人名或别称。

## 7. 必要 Sanitized Evidence
<small>7. Required sanitized evidence</small>

每次 source grounding 判定都必须保留以下脱敏字段：
<small>Each source-grounding decision must retain the following sanitized fields:</small>

- `candidate`
- `normalized_candidate`
- `bind_source_role_hit`
- `field`
- `row`
- `reason_code`

推荐 reason code：
<small>Recommended reason codes:</small>

- `source_fragment_exact`
- `source_fragment_normalized`
- `bare_source_role_grounded`
- `bare_source_role_ungrounded`
- `visual_subject_only`
- `abstract_subject_only`
- `invented_name_hard_fail`
- `source_role_conflict`

## 8. 禁止用模型风格解释替代契约判断
<small>8. Do not replace contract judgment with model-style explanations</small>

以下说法不能作为放行依据：
<small>The following are not valid pass rationales:</small>

- “模型只是风格化表达。”
- “中文里这样说也通。”
- “它大概指向主角。”
- “用户应该能理解。”

只允许用 source-bound evidence、accepted snapshot、StoryFactFrame 与 row/field 定位结果做判断。
<small>Only source-bound evidence, accepted snapshot, StoryFactFrame, and row/field-localized results may drive the judgment.</small>

## 9. Fresh Closure Evidence 入口
<small>9. Entry to fresh closure evidence</small>

当 `app/src/runtime.rs` 的抽取、分类或 grounding 逻辑发生变化后：
<small>After extraction, classification, or grounding logic changes in `app/src/runtime.rs`:</small>

1. 本地 targeted tests 只能说明实现局部收敛。
2. 旧 provider artifact 自动降级为 `reference-only`。
3. 必须先拿 fresh release-like shell。
4. 再拿 post-fix fresh provider artifact。
5. 只有 fresh provider artifact 满足本契约，`HOPE-CONTRACT-007` 才能考虑 closure。
