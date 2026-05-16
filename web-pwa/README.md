# Hope Web PWA

Hope Web PWA 是 Hope 分镜工作台的浏览器版本，目标是按桌面端产品链路完成：

原始文本 -> KB 写作组扩写/改写 -> 用户确认正文 -> KB 导演组生成 storyboard task -> 生成 rows / visual_description / prompt_text -> 用户确认当前镜头 -> 导出分镜词 / 导出完整剧本。

## 当前核实结论

- 当前目录 `E:\codex\hope-web-pwa` 不是 Git 仓库。
- 21 个 `scene_type` 已改成 canonical `value / label` 同源配置，统一来自 `config/scene-type-canonical.json`。
- QA runner 已改成 fresh artifact root 运行，不读取旧 `.codex-run` partial 作为通过证据。
- `formal403:plan-only` 只做结构核对，不能当 formal403 完成。

## 安装

```bash
npm install
```

## 本地运行

开发模式：

```bash
npm run dev
```

预览构建产物：

```bash
npm run build
npm run preview
```

默认地址：

- 开发：`http://127.0.0.1:4173/`
- 预览：`http://127.0.0.1:4174/`

## 正常使用链路

1. 打开页面，点击顶部 `API接口`。
2. 输入 `provider / base_url / endpoint / model / API Key`。
3. 点击 `测试连接`，确认浏览器直连可用。
4. 在 `正文输入` 中粘贴原始文本，选择 `scene_type` 和 `时长`。
5. 点击 `扩写故事` 或 `改写剧本`。
6. 检查 `正文草稿`，必要时用 `放大编辑` 修改。
7. 点击 `确定使用`，让 accepted body 成为唯一事实源。
8. 点击 `新建镜头任务`。
9. 点击 `开始生成`，得到当前镜头结果表格。
10. 如需人工修改，改完后点击 `确认当前镜头`。
11. 点击底部 `导出分镜词` 或 `导出完整剧本`。

## API 配置说明

页面支持以下 provider 入口：

- `qwen`
- `openai-compatible custom`
- `deepseek`（预留，不开放浏览器直连）
- `doubao`（预留，不开放浏览器直连）

页面允许配置：

- `base_url`
- `endpoint`
- `model`
- `writing_model`
- `director_model`
- `validator_model`
- `API Key`

安全策略：

- 默认只在当前会话中保存 API Key。
- 只有勾选“允许写入本机浏览器”后，API Key 才会写入 `localStorage`。
- 页面、README、导出、dist、QA artifact 都不应包含 secret、raw KB rows、source_register、overlay JSON 或 prompt_body。

## KB 与 evidence 口径

Web 版内嵌的是 sanitized KB policy snapshot，不会导出 raw KB rows。

页面与 runner 中至少保留以下 evidence：

- `kb_snapshot_hash`
- `selected_sample_ids`
- `selected_kb_rules`
- `writing_group_rule_pack_ids`
- `director_group_rule_pack_ids`
- `kb_context_summary`
- `applied_to`
- `influence_axes`
- `raw_kb_rows_included = 0`
- `raw_sample_text_absent = true`
- `source_register_absent = true`
- `overlay_json_absent = true`
- `prompt_body_absent = true`

## 导出说明

页面当前直接导出 Excel：

- `hope-storyboard-prompts.xls`
- `hope-full-script.xls`

导出最低字段：

- `镜头序号`
- `人物`
- `景别`
- `运镜`
- `画面描述`
- `角色动作`
- `对白/旁白`
- `分镜提示词`
- `时长`
- `备注/状态`

完整剧本导出还会带：

- `confirmed_narrative_body`
- `scene_type_id`
- `scene_type_label`
- `target_duration_seconds`
- `sanitized_kb_summary`
- `sanitized_provenance_summary`

禁止导出：

- raw KB rows
- raw sample_text
- source_register
- overlay JSON
- prompt_body
- API key / env
- 本机绝对路径

## QA 命令

```bash
npm run targeted
npm run full16
npm run formal403:plan-only
npm run formal403:live
```

说明：

- `targeted`：重点覆盖扩写、改写、scene 差异、手动编辑、导出校验。
- `full16`：按 16 个 fresh case 跑完整工作链路。
- `formal403:plan-only`：只核对 `4 + 378 + 21 = 403` 的结构，不跑 live。
- `formal403:live`：从 fresh artifact root 执行 real run；如果未达 `403/403`，必须停在首个 blocker。

## fresh artifact 规则

- runner 每次都会创建新的 artifact root。
- 不读取旧 `.codex-run` partial 当通过证据。
- `53/399` 这类 partial 只能算失败现场，不能算 formal403 完成。

## 交付 dist 给别人怎么用

先本地构建：

```bash
npm run build
```

然后把整个 `dist/` 目录连同本 README 一起交付给使用者。

推荐部署方式：

```bash
npx serve dist
```

也可以放到任意静态服务器的根路径下。由于当前 `vite.config.ts` 没有改 base，本构建默认按根路径部署；如果部署到子路径，请先自行验证资源路径是否正确。

不建议直接双击 `dist/index.html` 使用 `file://` 打开，因为浏览器模块加载、下载与网络请求行为会不稳定。
