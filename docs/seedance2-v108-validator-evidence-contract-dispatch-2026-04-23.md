# Seedance2 V108 Validator Evidence Contract Dispatch 2026-04-23

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor before this dispatch:
  `cd56773e5f31cd989634e66b310468a4f25dff24`
  (`docs: accept Seedance2 V108 import dry-run`)
- KB anchor:
  `b9660be7f44904ec56ba78462632db4251e88de3`
  (`docs: add Seedance2 V108 import dry-run report`)
- V3 archived anchor:
  `bc745a4d8b48ddc259c8b1c94ab907f8946797c0`
  (`docs: align V3 proposal with Seedance2 V108`)
- dispatch type: docs-only Hope main-thread validator evidence contract
  candidate

This dispatch does not open product implementation. It only opens the next
planning artifact after the accepted V108 import dry-run report.

## Current Control Decision

`OPEN_SEEDANCE2_V108_VALIDATOR_EVIDENCE_CONTRACT_CANDIDATE_DOCS_ONLY`

The accepted state before this gate is:

- `SEEDANCE2_V108_VNEXT_SCHEMA_CONTRACT_CANDIDATE_ACCEPTED`
- `SEEDANCE2_V108_IMPORT_CONTRACT_CANDIDATE_ACCEPTED`
- `SEEDANCE2_V108_IMPORT_DRY_RUN_REPORT_ACCEPTED`

The next active thread should be the Hope main thread:

- `Hope主线-Seedance2V108ValidatorEvidence【Contract候选执行中】`

Keep KB, V3, desktop, and intake waiting unless main control opens their own
separate gate.

## Assertions That Must Not Change

The candidate must preserve these accepted dry-run assertions:

```text
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
```

It must also preserve:

```text
v108_rows_imported_into_seed = 0
v108_rows_imported_into_product_structure = 0
positive_fewshot_promotions = 0
qwen_calls = 0
seedance_calls = 0
v3_branch_edits = 0
hope_product_code_changes = 0
desktop_or_intake_changes = 0
rust_dto_validator_exporter_workbook_ipc_changes = 0
```

## Required Inputs

The Hope main thread must read:

- `E:\codex\hope\docs\seedance2-v108-import-dry-run-review-2026-04-23.md`
- `E:\codex\hope-kb\docs\seedance2-v108-import-dry-run-report-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-import-contract-review-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-import-contract-candidate-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-vnext-schema-contract-review-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-vnext-schema-contract-candidate-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-vnext-schema-decision-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-v3-docs-alignment-review-2026-04-23.md`
- `E:\codex\hope-kb\docs\seedance2-v108-v3-alignment-report-2026-04-23.md`

## Task

Create a docs-only validator evidence contract candidate, for example:

- `docs/seedance2-v108-validator-evidence-contract-candidate-2026-04-23.md`

The candidate should define how V108 source evidence may be reasoned about by a
future validator layer without implementing validators.

Required planning topics:

- canonical evidence source fields:
  - `covered_points`
  - `missed_points`
  - `teaching_note`
  - `ip_abstraction_note`
  - `continuity_negative_core`
  - supporting context from `scene_performance_core`,
    `camera_directing_core`, `audio_directing_core`, `technical_profile`,
    `prompt_body`, and `reference_bundle`
- evidence families or planning signal groups for future validation
- blocker interaction with the accepted dry-run blockers:
  - `validator_evidence_not_defined`
  - `placeholder_present`
  - `prompt_body_blocked_by_placeholder`
  - `reference_handle_unresolved`
  - `v3_alignment_gap`
  - `schema_field_missing`
  - `promotion_gate_not_accepted`
  - `reserve_row`
- how validator evidence stays raw source evidence while
  `rows_ready_for_product_import = 0`
- how positive few-shot remains blocked while
  `rows_ready_for_positive_fewshot = 0`
- how unresolved references keep
  `product_ready_external_reference_handles = 0`
- why `reference_control_core_coverage = 0` stays closed
- future gate order after this candidate

Suggested planning object names:

- `V108ValidatorEvidenceContract`
- `V108ValidatorEvidenceSource`
- `V108ValidatorEvidenceFamily`
- `V108ValidatorEvidenceBlocker`
- `V108ValidatorEvidenceReadiness`
- `V108ValidatorEvidenceNoRuntimeImportAssertion`
- `V108ValidatorEvidenceFutureGateOrder`

These are planning names only. They are not Rust DTO names, database tables,
IPC contracts, workbook sheets, exporter fields, or runtime enum names.

## Forbidden Scope

The Hope main thread must not:

- modify Rust DTOs
- implement validators
- add failure-code enums, severity enums, thresholds, blocking logic, or repair
  behavior
- modify exporter behavior
- modify workbook shape
- modify IPC
- modify desktop
- modify intake
- modify `hope-kb`
- edit V3 branch files
- call Qwen
- call Seedance
- assemble runtime prompts
- import V108 rows into seed or product structures
- promote positive few-shot rows
- create `reference_control_core`
- invent reference images, URLs, asset IDs, character appearance, scene design,
  or canonical object registry data

## Acceptance Criteria

The thread can report completion when:

- one docs-only validator evidence contract candidate is committed and pushed
- no product code or side-repo file changed
- the memo cites Hope `cd56773`, KB `b9660be`, and V3 `bc745a4`
- all five required dry-run assertions remain unchanged
- the memo explicitly keeps implementation closed
- the memo states the next eligible gate after validator evidence planning

## Message To Send

Send this to the Hope main thread:

```text
线程名：Hope主线-Seedance2V108ValidatorEvidence【Contract候选执行中】
仓库 / 分支：E:\codex\hope / codex/contracts-freeze
当前锚点：cd56773 docs: accept Seedance2 V108 import dry-run

本轮任务：
只做 docs-only V108 validator evidence contract candidate。
不要实现 validator，不要改 Rust/DTO/exporter/workbook/IPC，不要动 KB/V3/desktop/intake。

必读输入：
- E:\codex\hope\docs\seedance2-v108-validator-evidence-contract-dispatch-2026-04-23.md
- E:\codex\hope\docs\seedance2-v108-import-dry-run-review-2026-04-23.md
- E:\codex\hope-kb\docs\seedance2-v108-import-dry-run-report-2026-04-23.md
- E:\codex\hope\docs\seedance2-v108-import-contract-review-2026-04-23.md
- E:\codex\hope\docs\seedance2-v108-import-contract-candidate-2026-04-23.md
- E:\codex\hope\docs\seedance2-v108-vnext-schema-contract-review-2026-04-23.md
- E:\codex\hope\docs\seedance2-v108-vnext-schema-contract-candidate-2026-04-23.md
- E:\codex\hope\docs\seedance2-v108-vnext-schema-decision-2026-04-23.md

必须保持断言：
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0

输出要求：
- 新建 docs/seedance2-v108-validator-evidence-contract-candidate-2026-04-23.md
- 定义 V108 validator evidence 的 planning contract
- 说明 covered_points / missed_points / teaching_note / ip_abstraction_note /
  continuity_negative_core 的未来证据用途
- 说明 placeholder / reference unresolved / v3 alignment gap / schema missing /
  promotion gate 等 blocker 如何影响 validator readiness
- 明确所有 115 行仍是 raw source evidence，不进入 product import、不进入 positive few-shot
- 明确不创建 reference_control_core
- 给出下一步 gate 顺序

禁止范围：
- 不改 Rust DTO / validator / exporter / workbook / IPC
- 不改 hope-kb / V3 / desktop / intake
- 不接 Qwen / Seedance
- 不导入 V108 rows
- 不推广 positive few-shot
- 不发明外部引用、角色外观、场景设计或 registry 数据

完成后提交并 push。

回报主控：
- final branch / commit
- changed files
- evidence families / planning objects
- unchanged assertions
- unresolved blockers
- validation result；如 docs-only 未运行测试也要说明
```

