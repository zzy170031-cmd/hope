# 项目错题在线记录
<small>Project error-ledger online record</small>

本目录保存 Hope 桌面端项目推进中的错题记录，作为可在线查看、可随提交更新的工程复盘入口。
<small>This directory stores the Hope desktop project error ledger as a Git-tracked engineering review record.</small>

## 当前主记录
<small>Current main record</small>

- [项目进度查错文档 Markdown](project-progress-error-ledger.md)
  <small>Project progress error ledger in Markdown</small>
- [项目进度查错文档 DOCX](project-progress-error-ledger.docx)
  <small>Project progress error ledger in DOCX</small>

## 配套契约入口
<small>Companion contract entry points</small>

- [Shell / WebView2 / CDP 启动契约](../desktop-shell-webview2-cdp-startup-contract.md)
  <small>Shell / WebView2 / CDP startup hard gate contract</small>
## 更新规则
<small>Update rules</small>

- 先更新 Markdown，再同步重生 DOCX。
  <small>Update Markdown first, then regenerate the DOCX copy.</small>
- 只记录 sanitized evidence，不记录 raw env、API key、raw prompt、raw provider response、source_register 或 overlay JSON。
  <small>Record sanitized evidence only; never include raw env, API keys, raw prompts, raw provider responses, source_register, or overlay JSON.</small>
- open、closed、reference-only 状态必须按 fresh evidence 更新，不得把 reference-only 当成 gate evidence。
  <small>Open, closed, and reference-only statuses must follow fresh evidence; reference-only material must not be treated as gate evidence.</small>
- 主线问题重复出现时，应及时补充错题条目，并写清楚 Problem、Resolution、Prevention、Playbook。
  <small>When a mainline issue repeats, add or update the entry with Problem, Resolution, Prevention, and Playbook.</small>
