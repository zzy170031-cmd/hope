# Golden Sample Validator Planning Dispatch 2026-04-22

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor before this memo: `0b96716` (`docs: archive V3 golden sample freeze candidate`)
- route: `RC_READY` plus controlled v0.2 readiness planning
- dispatch type: docs-only downstream planning gate

This memo opens only the next planning task. It does not open product
implementation and does not authorize Rust DTO, validator, exporter, workbook,
desktop, intake, Qwen, or Seedance changes.

## Decision

`OPEN_DOCS_ONLY_GOLDEN_SAMPLE_VALIDATOR_PLANNING`

The V3 golden-sample contract-freeze candidate has been accepted and archived
for now. The next active thread is the Hope main validator planning thread:

- `Hope主线-GoldenSampleValidatorPlanning【v0.2待开门】`

All other child threads remain on standby unless the control thread opens their
own named scope gate.

## Current Anchors

Hope main:

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor: `0b96716` (`docs: archive V3 golden sample freeze candidate`)
- state: clean and aligned with `origin/codex/contracts-freeze`

Hope KB:

- repo: `E:\codex\hope-kb`
- branch: `codex/contracts-freeze`
- anchor: `445c452` (`Add golden sample v0.2 schema package`)
- accepted package: `KB_GOLDEN_SAMPLE_V0_2_SCHEMA_PACKAGE_ACCEPTED`

V3 field overlay:

- branch: `origin/codex/v3-field-overlay-proposal`
- anchor: `69c645c` (`docs: promote golden sample overlay to freeze candidate`)
- accepted package: `V3_GOLDEN_SAMPLE_CONTRACT_FREEZE_CANDIDATE_ACCEPTED`
- state: archived / standby

Desktop:

- repo: `E:\codex\hope-desktop-shell`
- branch: `codex/desktop-shell`
- anchor: `66d687d` (`desktop-shell: add writer qwen input readiness boundary`)
- state: waiting; no new desktop gate opened

Controlled intake:

- repo: `E:\codex\hope-intake-app`
- branch: `codex/controlled-intake-snapshot-bootstrap-app`
- anchor: `d74bd11` (`qwen-storyboard: tighten contract boundary`)
- state: waiting; no new intake or live Qwen gate opened

## Required Inputs For The Main Thread

The validator planning thread must read:

- `E:\codex\hope\docs\v0.2-scope-gate-readiness-2026-04-22.md`
- `E:\codex\hope\docs\kb-golden-sample-v0.2-package-review-2026-04-22.md`
- `E:\codex\hope\docs\v3-golden-sample-contract-freeze-review-2026-04-22.md`
- `E:\codex\hope-kb\docs\golden-sample-v0.2-validation-readiness-2026-04-22.md`
- `E:\codex\hope-kb\seed\v0.2\golden_sample_library.json`
- `E:\codex\hope-kb\seed\v0.2\golden_sample_field_coverage_rules.json`
- `E:\codex\hope-kb\seed\v0.2\golden_sample_failure_mapping.json`
- `E:\codex\hope-kb\seed\v0.2\golden_sample_repair_mapping.json`
- `E:\codex\hope-kb\seed\v0.2\source_register.json`

## Planning Output

The main thread should create one docs-only candidate memo, for example:

- `docs/golden-sample-validator-contract-candidate-2026-04-22.md`

The memo should define the future validator contract around these planning
questions:

- how `golden_sample_library` records become validator evidence
- which fields are required for coverage checks
- which fields are only debug / provenance / retrieval metadata
- how `covered_points`, `missed_points`, `empty_words_detected`, `word_count`,
  `tier`, and `usable_for_fewshot` map to future validator behavior
- how `negative_sample` records are excluded from few-shot retrieval and used
  as validator negative examples
- how failure mapping and repair mapping should be consumed by a later repair
  planning thread
- what fixture or benchmark impact would be expected when implementation
  finally opens

The candidate should use these draft validator signal families unless the
thread finds a better docs-only naming scheme:

- `golden_coverage_gap`
- `golden_empty_word_noise`
- `golden_negative_sample`
- `golden_unusable_fewshot`
- `golden_source_provenance_gap`

## Explicit Non-Implementation Boundary

The main thread must not change:

- Rust DTOs
- Rust validators
- exporter behavior
- workbook sheet shape
- IPC contracts
- desktop UI
- intake UI
- Qwen prompt generation
- Seedance adapters
- `hope-kb` seed files
- v0.1 seed, manifest, import map, migration, or snapshot contract

This task is a planning gate only.

## Acceptance Criteria

The main thread can report completion when all of these are true:

- one docs-only validator contract candidate is committed and pushed
- the memo cites Hope `0b96716`, KB `445c452`, and V3 `69c645c`
- no product code files changed
- unresolved gaps remain explicitly named, especially `reference_control_core`
  and v0.2 snapshot-import readiness
- future gate order is stated for validator, repair, exporter/debug metadata,
  desktop read-only provenance, and intake / Qwen retrieval boundary
- the report includes final branch / commit, changed files, and whether any
  validation was run

## Side Thread State

Keep these threads waiting:

- `Hope-KB-GoldenSampleLibraryV0.2【SchemaGate已推送】`
- `Hope-V3字段线程-GoldenSampleContract【v0.2 Freeze候选已推送·归档待命】`
- `Hope桌面端-WriterReadiness【InputBoundary待命】`
- `Hope接入线程-Qwen分镜生成Contract【Contract边界待命】`

## Message To Send

Send this to the existing Hope main / implementation thread:

```text
线程名：Hope主线-GoldenSampleValidatorPlanning【v0.2待开门】
仓库 / 分支：E:\codex\hope / codex/contracts-freeze
当前锚点：0b96716 docs: archive V3 golden sample freeze candidate

本轮任务：
只做 docs-only validator contract planning，规划 v0.2 golden_sample_library
未来如何进入 Hope validator / repair / exporter / debug metadata 链路。

必读输入：
- E:\codex\hope\docs\v0.2-scope-gate-readiness-2026-04-22.md
- E:\codex\hope\docs\kb-golden-sample-v0.2-package-review-2026-04-22.md
- E:\codex\hope\docs\v3-golden-sample-contract-freeze-review-2026-04-22.md
- E:\codex\hope-kb\docs\golden-sample-v0.2-validation-readiness-2026-04-22.md
- E:\codex\hope-kb\seed\v0.2\golden_sample_library.json
- E:\codex\hope-kb\seed\v0.2\golden_sample_field_coverage_rules.json
- E:\codex\hope-kb\seed\v0.2\golden_sample_failure_mapping.json
- E:\codex\hope-kb\seed\v0.2\golden_sample_repair_mapping.json
- E:\codex\hope-kb\seed\v0.2\source_register.json

输出要求：
- 新建一个 docs-only validator contract candidate memo
- 说明 golden_sample_library 如何作为 validator evidence
- 说明 covered_points / missed_points / empty_words_detected / word_count /
  tier / usable_for_fewshot / negative_sample 的未来 validator 用法
- 给出 failure code / repair mapping / fixture impact / future gate order
- 明确 reference_control_core 和 v0.2 snapshot-import 仍是未决缺口

禁止范围：
- 不改 Rust DTO / validator / exporter / workbook / IPC
- 不改 desktop / intake
- 不接 Qwen / Seedance
- 不改 hope-kb seed
- 不碰 v0.1 seed / manifest / import_map / snapshot contract

完成后提交并 push。

回报主控：
- final branch / commit
- changed files
- validator signal families
- unresolved gaps
- validation result，如 docs-only 未运行也要说明
```
