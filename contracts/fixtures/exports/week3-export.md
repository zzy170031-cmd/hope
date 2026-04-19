# Hope 导出工作簿

- workbook machine name: `hope_export_workbook`
- workbook is sourced from the shared fixture and validation report

## 项目元数据 / `project_meta`

| 项目标识 | 标题 | 状态 | 目标时长分钟 | 更新时间戳 |
|---|---|---|---|---|
| project-week3-001 | Hope 10min single episode | 冻结中 | 10 | 1713513600 |

## 集元数据 / `episode_meta`

| 集标识 | 项目标识 | 序号 | 标题 | 目标时长分钟 |
|---|---|---|---|---|
| episode-week3-001 | project-week3-001 | 1 | 第 1 集 | 10 |

## 叙事场景 / `narrative_scene`

| 叙事场景标识 | 集标识 | 序号 | 标题 | 内容摘要 |
|---|---|---|---|---|
| narrative-scene-week3-001 | episode-week3-001 | 1 | 第 1 集-场景1 | 第 1 集的第1个叙事场景，保持中景与冷色调推进。 |
| narrative-scene-week3-002 | episode-week3-001 | 2 | 第 1 集-场景2 | 第 1 集的第2个叙事场景，保持中景与冷色调推进。 |
| narrative-scene-week3-003 | episode-week3-001 | 3 | 第 1 集-场景3 | 第 1 集的第3个叙事场景，保持中景与冷色调推进。 |

## RenderSegment / `render_segment`

| RenderSegment标识 | 叙事场景标识 | 序号 | 起始镜头序号 | 结束镜头序号 | 目标时长秒 | 实际时长秒 |
|---|---|---|---|---|---|---|
| render-segment-week3-001 | narrative-scene-week3-001 | 1 | 10 | 12 | 67 | 67 |
| render-segment-week3-002 | narrative-scene-week3-001 | 2 | 13 | 15 | 67 | 67 |
| render-segment-week3-003 | narrative-scene-week3-001 | 3 | 16 | 18 | 67 | 67 |
| render-segment-week3-004 | narrative-scene-week3-002 | 4 | 19 | 21 | 67 | 67 |
| render-segment-week3-005 | narrative-scene-week3-002 | 5 | 22 | 24 | 67 | 67 |
| render-segment-week3-006 | narrative-scene-week3-002 | 6 | 25 | 27 | 67 | 67 |
| render-segment-week3-007 | narrative-scene-week3-003 | 7 | 28 | 30 | 66 | 66 |
| render-segment-week3-008 | narrative-scene-week3-003 | 8 | 31 | 33 | 66 | 66 |
| render-segment-week3-009 | narrative-scene-week3-003 | 9 | 34 | 36 | 66 | 66 |

## Cut / `cut`

| Cut标识 | RenderSegment标识 | 序号 | 镜头描述 | 对白 | 时长秒 |
|---|---|---|---|---|---|
| cut-week3-001 | render-segment-week3-001 | 1 | render-segment-week3-001 的第1个 cut，保持中景推进。 | 第 1 集 第1拍继续执行。 | 23 |
| cut-week3-002 | render-segment-week3-001 | 2 | render-segment-week3-001 的第2个 cut，保持中景推进。 | 第 1 集 第2拍继续执行。 | 22 |
| cut-week3-003 | render-segment-week3-001 | 3 | render-segment-week3-001 的第3个 cut，保持中景推进。 | 第 1 集 第3拍继续执行。 | 22 |
| cut-week3-004 | render-segment-week3-002 | 4 | render-segment-week3-002 的第1个 cut，保持中景推进。 | 第 1 集 第4拍继续执行。 | 23 |
| cut-week3-005 | render-segment-week3-002 | 5 | render-segment-week3-002 的第2个 cut，保持中景推进。 | 第 1 集 第5拍继续执行。 | 22 |
| cut-week3-006 | render-segment-week3-002 | 6 | render-segment-week3-002 的第3个 cut，保持中景推进。 | 第 1 集 第6拍继续执行。 | 22 |
| cut-week3-007 | render-segment-week3-003 | 7 | render-segment-week3-003 的第1个 cut，保持中景推进。 | 第 1 集 第7拍继续执行。 | 23 |
| cut-week3-008 | render-segment-week3-003 | 8 | render-segment-week3-003 的第2个 cut，保持中景推进。 | 第 1 集 第8拍继续执行。 | 22 |
| cut-week3-009 | render-segment-week3-003 | 9 | render-segment-week3-003 的第3个 cut，保持中景推进。 | 第 1 集 第9拍继续执行。 | 22 |
| cut-week3-010 | render-segment-week3-004 | 10 | render-segment-week3-004 的第1个 cut，保持中景推进。 | 第 1 集 第10拍继续执行。 | 23 |
| cut-week3-011 | render-segment-week3-004 | 11 | render-segment-week3-004 的第2个 cut，保持中景推进。 | 第 1 集 第11拍继续执行。 | 22 |
| cut-week3-012 | render-segment-week3-004 | 12 | render-segment-week3-004 的第3个 cut，保持中景推进。 | 第 1 集 第12拍继续执行。 | 22 |
| cut-week3-013 | render-segment-week3-005 | 13 | render-segment-week3-005 的第1个 cut，保持中景推进。 | 第 1 集 第13拍继续执行。 | 23 |
| cut-week3-014 | render-segment-week3-005 | 14 | render-segment-week3-005 的第2个 cut，保持中景推进。 | 第 1 集 第14拍继续执行。 | 22 |
| cut-week3-015 | render-segment-week3-005 | 15 | render-segment-week3-005 的第3个 cut，保持中景推进。 | 第 1 集 第15拍继续执行。 | 22 |
| cut-week3-016 | render-segment-week3-006 | 16 | render-segment-week3-006 的第1个 cut，保持中景推进。 | 第 1 集 第16拍继续执行。 | 23 |
| cut-week3-017 | render-segment-week3-006 | 17 | render-segment-week3-006 的第2个 cut，保持中景推进。 | 第 1 集 第17拍继续执行。 | 22 |
| cut-week3-018 | render-segment-week3-006 | 18 | render-segment-week3-006 的第3个 cut，保持中景推进。 | 第 1 集 第18拍继续执行。 | 22 |
| cut-week3-019 | render-segment-week3-007 | 19 | render-segment-week3-007 的第1个 cut，保持中景推进。 | 第 1 集 第19拍继续执行。 | 22 |
| cut-week3-020 | render-segment-week3-007 | 20 | render-segment-week3-007 的第2个 cut，保持中景推进。 | 第 1 集 第20拍继续执行。 | 22 |
| cut-week3-021 | render-segment-week3-007 | 21 | render-segment-week3-007 的第3个 cut，保持中景推进。 | 第 1 集 第21拍继续执行。 | 22 |
| cut-week3-022 | render-segment-week3-008 | 22 | render-segment-week3-008 的第1个 cut，保持中景推进。 | 第 1 集 第22拍继续执行。 | 22 |
| cut-week3-023 | render-segment-week3-008 | 23 | render-segment-week3-008 的第2个 cut，保持中景推进。 | 第 1 集 第23拍继续执行。 | 22 |
| cut-week3-024 | render-segment-week3-008 | 24 | render-segment-week3-008 的第3个 cut，保持中景推进。 | 第 1 集 第24拍继续执行。 | 22 |
| cut-week3-025 | render-segment-week3-009 | 25 | render-segment-week3-009 的第1个 cut，保持中景推进。 | 第 1 集 第25拍继续执行。 | 22 |
| cut-week3-026 | render-segment-week3-009 | 26 | render-segment-week3-009 的第2个 cut，保持中景推进。 | 第 1 集 第26拍继续执行。 | 22 |
| cut-week3-027 | render-segment-week3-009 | 27 | render-segment-week3-009 的第3个 cut，保持中景推进。 | 第 1 集 第27拍继续执行。 | 22 |

## PromptPackage / `prompt_package`

| PromptPackage标识 | 来源层级 | 正文 | 版本 |
|---|---|---|---|
| prompt-package-layout-episode-week3-001-render-segment-week3-001 | layout_prompt | 请生成可直接用于外部模型的连续画面布局描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持冷静克制的神情，不改变发型、服装和年龄感。风格锁主约束：冷色调，中景，第 1 集的当前连续画面布局稳定推进。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体与构图：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第1个叙事场景，保持中景与冷色调推进。。动作与情绪：当前连续画面 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第1个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：当前连续画面 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-render-episode-week3-001-render-segment-week3-001 | render_prompt | 请生成可直接用于外部模型的连续画面最终渲染描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持写实皮肤质感与自然五官比例。风格锁主约束：冷色调、克制写实光线、中景连续调度，围绕当前连续画面稳定输出。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体、构图与渲染：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第1个叙事场景，保持中景与冷色调推进。。动作与情绪：当前连续画面 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第1个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：当前连续画面 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-layout-episode-week3-001-render-segment-week3-002 | layout_prompt | 请生成可直接用于外部模型的连续画面布局描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持冷静克制的神情，不改变发型、服装和年龄感。风格锁主约束：冷色调，中景，第 1 集的当前连续画面布局稳定推进。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体与构图：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第1个叙事场景，保持中景与冷色调推进。。动作与情绪：当前连续画面 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第1个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：当前连续画面 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-render-episode-week3-001-render-segment-week3-002 | render_prompt | 请生成可直接用于外部模型的连续画面最终渲染描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持写实皮肤质感与自然五官比例。风格锁主约束：冷色调、克制写实光线、中景连续调度，围绕当前连续画面稳定输出。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体、构图与渲染：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第1个叙事场景，保持中景与冷色调推进。。动作与情绪：当前连续画面 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第1个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：当前连续画面 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-layout-episode-week3-001-render-segment-week3-003 | layout_prompt | 请生成可直接用于外部模型的连续画面布局描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持冷静克制的神情，不改变发型、服装和年龄感。风格锁主约束：冷色调，中景，第 1 集的当前连续画面布局稳定推进。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体与构图：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第1个叙事场景，保持中景与冷色调推进。。动作与情绪：当前连续画面 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第1个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：当前连续画面 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-render-episode-week3-001-render-segment-week3-003 | render_prompt | 请生成可直接用于外部模型的连续画面最终渲染描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持写实皮肤质感与自然五官比例。风格锁主约束：冷色调、克制写实光线、中景连续调度，围绕当前连续画面稳定输出。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体、构图与渲染：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第1个叙事场景，保持中景与冷色调推进。。动作与情绪：当前连续画面 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第1个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：当前连续画面 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-layout-episode-week3-001-render-segment-week3-004 | layout_prompt | 请生成可直接用于外部模型的连续画面布局描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持冷静克制的神情，不改变发型、服装和年龄感。风格锁主约束：冷色调，中景，第 1 集的004布局稳定推进。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体与构图：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第2个叙事场景，保持中景与冷色调推进。。动作与情绪：004 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第2个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：004 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-render-episode-week3-001-render-segment-week3-004 | render_prompt | 请生成可直接用于外部模型的连续画面最终渲染描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持写实皮肤质感与自然五官比例。风格锁主约束：冷色调、克制写实光线、中景连续调度，围绕004稳定输出。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体、构图与渲染：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第2个叙事场景，保持中景与冷色调推进。。动作与情绪：004 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第2个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：004 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-layout-episode-week3-001-render-segment-week3-005 | layout_prompt | 请生成可直接用于外部模型的连续画面布局描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持冷静克制的神情，不改变发型、服装和年龄感。风格锁主约束：冷色调，中景，第 1 集的005布局稳定推进。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体与构图：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第2个叙事场景，保持中景与冷色调推进。。动作与情绪：005 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第2个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：005 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-render-episode-week3-001-render-segment-week3-005 | render_prompt | 请生成可直接用于外部模型的连续画面最终渲染描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持写实皮肤质感与自然五官比例。风格锁主约束：冷色调、克制写实光线、中景连续调度，围绕005稳定输出。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体、构图与渲染：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第2个叙事场景，保持中景与冷色调推进。。动作与情绪：005 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第2个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：005 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-layout-episode-week3-001-render-segment-week3-006 | layout_prompt | 请生成可直接用于外部模型的连续画面布局描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持冷静克制的神情，不改变发型、服装和年龄感。风格锁主约束：冷色调，中景，第 1 集的006布局稳定推进。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体与构图：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第2个叙事场景，保持中景与冷色调推进。。动作与情绪：006 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第2个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：006 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-render-episode-week3-001-render-segment-week3-006 | render_prompt | 请生成可直接用于外部模型的连续画面最终渲染描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持写实皮肤质感与自然五官比例。风格锁主约束：冷色调、克制写实光线、中景连续调度，围绕006稳定输出。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体、构图与渲染：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第2个叙事场景，保持中景与冷色调推进。。动作与情绪：006 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第2个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：006 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-layout-episode-week3-001-render-segment-week3-007 | layout_prompt | 请生成可直接用于外部模型的连续画面布局描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持冷静克制的神情，不改变发型、服装和年龄感。风格锁主约束：冷色调，中景，第 1 集的007布局稳定推进。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体与构图：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第3个叙事场景，保持中景与冷色调推进。。动作与情绪：007 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第3个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：007 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-render-episode-week3-001-render-segment-week3-007 | render_prompt | 请生成可直接用于外部模型的连续画面最终渲染描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持写实皮肤质感与自然五官比例。风格锁主约束：冷色调、克制写实光线、中景连续调度，围绕007稳定输出。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体、构图与渲染：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第3个叙事场景，保持中景与冷色调推进。。动作与情绪：007 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第3个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：007 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-layout-episode-week3-001-render-segment-week3-008 | layout_prompt | 请生成可直接用于外部模型的连续画面布局描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持冷静克制的神情，不改变发型、服装和年龄感。风格锁主约束：冷色调，中景，第 1 集的008布局稳定推进。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体与构图：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第3个叙事场景，保持中景与冷色调推进。。动作与情绪：008 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第3个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：008 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-render-episode-week3-001-render-segment-week3-008 | render_prompt | 请生成可直接用于外部模型的连续画面最终渲染描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持写实皮肤质感与自然五官比例。风格锁主约束：冷色调、克制写实光线、中景连续调度，围绕008稳定输出。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体、构图与渲染：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第3个叙事场景，保持中景与冷色调推进。。动作与情绪：008 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第3个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：008 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-layout-episode-week3-001-render-segment-week3-009 | layout_prompt | 请生成可直接用于外部模型的连续画面布局描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持冷静克制的神情，不改变发型、服装和年龄感。风格锁主约束：冷色调，中景，第 1 集的009布局稳定推进。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体与构图：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第3个叙事场景，保持中景与冷色调推进。。动作与情绪：009 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第3个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：009 的第1个 cut，保持中景推进。。 | 1 |
| prompt-package-render-episode-week3-001-render-segment-week3-009 | render_prompt | 请生成可直接用于外部模型的连续画面最终渲染描述：角色外观锚点：同一位三十多岁的东亚男性，短黑发，清瘦体态，深色长外套配深色内搭，面部线条克制，眼神冷静但带轻微疲惫感，保持写实皮肤质感与自然五官比例。风格锁主约束：冷色调、克制写实光线、中景连续调度，围绕009稳定输出。；不允许切换到不同色温、不同人物设计、不同服装体系或不同场景气质。镜头主体、构图与渲染：同一主角停留在同一场景主体内，采用中景构图，人物位于画面主焦点，环境信息围绕主体展开；场景与氛围：第 1 集的第3个叙事场景，保持中景与冷色调推进。。动作与情绪：009 的第1个 cut，保持中景推进。。连续性主约束：当前镜头承接上一镜头，保持同一主角视角延续、同一场景连续构图和同一动作链条；不切换人物，不切换场景主体，不改变角色服装与发型；上一镜头到当前镜头的延续内容为：第 1 集的第3个叙事场景，保持中景与冷色调推进。；当前镜头重点动作继续保持：009 的第1个 cut，保持中景推进。。 | 1 |

## 交接带 / `handoff_zone`

| HandoffZone标识 | RenderSegment标识 | 起始边界 | 结束边界 | 边界类型 |
|---|---|---|---|---|
| handoff-zone-week3-001 | render-segment-week3-001 | 10 | 12 | render_segment_boundary |
| handoff-zone-week3-002 | render-segment-week3-002 | 13 | 15 | render_segment_boundary |
| handoff-zone-week3-003 | render-segment-week3-003 | 16 | 18 | render_segment_boundary |
| handoff-zone-week3-004 | render-segment-week3-004 | 19 | 21 | render_segment_boundary |
| handoff-zone-week3-005 | render-segment-week3-005 | 22 | 24 | render_segment_boundary |
| handoff-zone-week3-006 | render-segment-week3-006 | 25 | 27 | render_segment_boundary |
| handoff-zone-week3-007 | render-segment-week3-007 | 28 | 30 | render_segment_boundary |
| handoff-zone-week3-008 | render-segment-week3-008 | 31 | 33 | render_segment_boundary |
| handoff-zone-week3-009 | render-segment-week3-009 | 34 | 36 | render_segment_boundary |

## 全局硬锁 / `hard_lock`

| HardLock标识 | 项目标识 | 锁名 | 锁值 | 作用范围 |
|---|---|---|---|---|
| hard-lock-week3-001 | project-week3-001 | negative_prompt_guard | 禁止出现血腥暴力词汇 | project |

## Stale传播 / `stale_event`

| StaleEvent标识 | 来源层级 | 来源标识 | 目标层级 | 目标标识 | 追踪标识 | 时间戳 | 原因 |
|---|---|---|---|---|---|---|---|
| stale-event-week3-001 | render_segment | render-segment-week3-001 | cut | cut-week3-001 | trace-render-segment-week3-001-cut-week3-001 | 1713513601 | render_segment updated |
| stale-event-week3-002 | render_segment | render-segment-week3-002 | cut | cut-week3-004 | trace-render-segment-week3-002-cut-week3-004 | 1713513602 | render_segment updated |
| stale-event-week3-003 | render_segment | render-segment-week3-003 | cut | cut-week3-007 | trace-render-segment-week3-003-cut-week3-007 | 1713513603 | render_segment updated |
| stale-event-week3-004 | render_segment | render-segment-week3-004 | cut | cut-week3-010 | trace-render-segment-week3-004-cut-week3-010 | 1713513604 | render_segment updated |
| stale-event-week3-005 | render_segment | render-segment-week3-005 | cut | cut-week3-013 | trace-render-segment-week3-005-cut-week3-013 | 1713513605 | render_segment updated |
| stale-event-week3-006 | render_segment | render-segment-week3-006 | cut | cut-week3-016 | trace-render-segment-week3-006-cut-week3-016 | 1713513606 | render_segment updated |
| stale-event-week3-007 | render_segment | render-segment-week3-007 | cut | cut-week3-019 | trace-render-segment-week3-007-cut-week3-019 | 1713513607 | render_segment updated |
| stale-event-week3-008 | render_segment | render-segment-week3-008 | cut | cut-week3-022 | trace-render-segment-week3-008-cut-week3-022 | 1713513608 | render_segment updated |
| stale-event-week3-009 | render_segment | render-segment-week3-009 | cut | cut-week3-025 | trace-render-segment-week3-009-cut-week3-025 | 1713513609 | render_segment updated |

## 校验报告 / `validation_report`

| 校验报告标识 | 项目标识 | 通过 | 问题数 | 更新时间戳 |
|---|---|---|---|---|
| validation_report_hard-lock-week3-001_hard_lock | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-001_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-001_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-001_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-001_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-002_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-002_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-002_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-002_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-003_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-003_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-003_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-003_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-004_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-004_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-004_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-004_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-005_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-005_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-005_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-005_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-006_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-006_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-006_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-006_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-007_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-007_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-007_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-007_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-008_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-008_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-008_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-008_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-009_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-layout-episode-week3-001-render-segment-week3-009_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-009_style | project-week3-001 | true | 0 | 1713513600 |
| validation_report_prompt-package-render-episode-week3-001-render-segment-week3-009_negative | project-week3-001 | true | 0 | 1713513600 |
| validation_report_stale-event-week3-001_continuity | project-week3-001 | true | 0 | 1713513601 |
| validation_report_cut-week3-001_trace | project-week3-001 | true | 0 | 1713513601 |
| validation_report_stale-event-week3-002_continuity | project-week3-001 | true | 0 | 1713513602 |
| validation_report_cut-week3-004_trace | project-week3-001 | true | 0 | 1713513602 |
| validation_report_stale-event-week3-003_continuity | project-week3-001 | true | 0 | 1713513603 |
| validation_report_cut-week3-007_trace | project-week3-001 | true | 0 | 1713513603 |
| validation_report_stale-event-week3-004_continuity | project-week3-001 | true | 0 | 1713513604 |
| validation_report_cut-week3-010_trace | project-week3-001 | true | 0 | 1713513604 |
| validation_report_stale-event-week3-005_continuity | project-week3-001 | true | 0 | 1713513605 |
| validation_report_cut-week3-013_trace | project-week3-001 | true | 0 | 1713513605 |
| validation_report_stale-event-week3-006_continuity | project-week3-001 | true | 0 | 1713513606 |
| validation_report_cut-week3-016_trace | project-week3-001 | true | 0 | 1713513606 |
| validation_report_stale-event-week3-007_continuity | project-week3-001 | true | 0 | 1713513607 |
| validation_report_cut-week3-019_trace | project-week3-001 | true | 0 | 1713513607 |
| validation_report_stale-event-week3-008_continuity | project-week3-001 | true | 0 | 1713513608 |
| validation_report_cut-week3-022_trace | project-week3-001 | true | 0 | 1713513608 |
| validation_report_stale-event-week3-009_continuity | project-week3-001 | true | 0 | 1713513609 |
| validation_report_cut-week3-025_trace | project-week3-001 | true | 0 | 1713513609 |
| validation_report_render-segment-week3-001_duration | project-week3-001 | true | 0 | 1713513600 |
| validation_report_render-segment-week3-002_duration | project-week3-001 | true | 0 | 1713513600 |
| validation_report_render-segment-week3-003_duration | project-week3-001 | true | 0 | 1713513600 |
| validation_report_render-segment-week3-004_duration | project-week3-001 | true | 0 | 1713513600 |
| validation_report_render-segment-week3-005_duration | project-week3-001 | true | 0 | 1713513600 |
| validation_report_render-segment-week3-006_duration | project-week3-001 | true | 0 | 1713513600 |
| validation_report_render-segment-week3-007_duration | project-week3-001 | true | 0 | 1713513600 |
| validation_report_render-segment-week3-008_duration | project-week3-001 | true | 0 | 1713513600 |
| validation_report_render-segment-week3-009_duration | project-week3-001 | true | 0 | 1713513600 |
| validation_report_handoff-zone-week3-001_handoff | project-week3-001 | true | 0 | 1713513600 |
| validation_report_handoff-zone-week3-002_handoff | project-week3-001 | true | 0 | 1713513600 |
| validation_report_handoff-zone-week3-003_handoff | project-week3-001 | true | 0 | 1713513600 |
| validation_report_handoff-zone-week3-004_handoff | project-week3-001 | true | 0 | 1713513600 |
| validation_report_handoff-zone-week3-005_handoff | project-week3-001 | true | 0 | 1713513600 |
| validation_report_handoff-zone-week3-006_handoff | project-week3-001 | true | 0 | 1713513600 |
| validation_report_handoff-zone-week3-007_handoff | project-week3-001 | true | 0 | 1713513600 |
| validation_report_handoff-zone-week3-008_handoff | project-week3-001 | true | 0 | 1713513600 |
| validation_report_handoff-zone-week3-009_handoff | project-week3-001 | true | 0 | 1713513600 |

## 导出清单 / `export_manifest`

| 导出清单标识 | 项目标识 | 工作簿版本 | 状态 |
|---|---|---|---|
| export-manifest-week3-001 | project-week3-001 | 10min-single-episode-fixture | 准备导出 |

## 导演档案 / `director_profile`

| 导演档案标识 | 导演名称 | 定位 | 主风格 | 镜头偏好 |
|---|---|---|---|---|
| director-profile-week3-001 | 导演甲 | 场景主导 | 克制写实 | 中景与近景结合 |

## 导演切样 / `director_cut_sample`

| 导演切样标识 | 导演档案标识 | 样例标题 | 样例内容 | 代表性说明 |
|---|---|---|---|---|
| director-cut-sample-week3-001 | director-profile-week3-001 | 10min single episode切样 | 10min single episode维持冷色调、中景、稳定切分。 | 用于 E2E benchmark 的统一视觉与动作节奏参考。 |

## 委员会模板 / `committee_template`

| 模板标识 | 模板名称 | 适用场景 | 成员构成 | 职责描述 |
|---|---|---|---|---|
| committee-template-week3-001 | 基础评审委员会 | 常规叙事段落 | 场景导演,动作导演 | 统一镜头节奏、动作推进与提示词风格。 |

## 视觉术语 / `visual_term`

| 术语标识 | 中文术语 | 类别 | 定义 | 别名 |
|---|---|---|---|---|
| visual-term-week3-001 | 冷色调 | 色彩 | 偏冷且低饱和的整体色彩基调。 | 冷调 |

## 摄影术语 / `cinematography_term`

| 术语标识 | 中文术语 | 类别 | 定义 | 别名 |
|---|---|---|---|---|
| cinematography-term-week3-001 | 中景 | 景别 | 突出人物与环境关系的中距离景别。 | 中镜 |

## 连续性规则 / `continuity_rule`

| 规则标识 | 规则名称 | 规则说明 | 适用层级 |
|---|---|---|---|
| continuity-rule-week3-001 | 场景动作连续 | 同一叙事场景内镜头衔接保持动作方向一致。 | NarrativeScene |

