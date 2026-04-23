# Seedance2 V108 Retrieval Selection Fixup Dispatch 2026-04-23

## Dispatch Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- reviewed implementation:
  `d10bd49` (`storyboard-pipeline: add V108 retrieval metadata selector`)
- review decision:
  `docs/seedance2-v108-retrieval-selection-implementation-review-2026-04-23.md`
- package type: focused code fixup dispatch

Main control keeps the same implementation slice and same file boundary. This
dispatch only corrects routing conservatism and Chinese/source-text coverage.

## Thread Label

```text
Hope主线-Seedance2V108RetrievalSelectionFixup【小补丁执行中】
```

## Task

Apply one focused corrective patch on top of `d10bd49`.

Keep the implementation non-runtime and metadata-only. Do not widen scope.

## Allowed Files

Only these files may change:

```text
crates/storyboard-pipeline/src/shot_language_selection.rs
crates/storyboard-pipeline/src/lib.rs
crates/storyboard-pipeline/tests/v108_shot_language_selection.rs
```

## Required Fixes

### A. Add Chinese / source-text cue coverage

The selector must recognize the accepted V108 evidence families when they
appear as Chinese/source-text cues, not only English synthetic keywords.

At minimum, cover route families for:

- `国漫 / 热血打斗` -> `director_08`
- `国漫 / 场域追逐` -> `director_08`
- `国漫 / 群像表演` + 战争 / 誓师 / 阵列 / 旗鼓 -> `director_09`
- `都市末世 / 工业压迫` + 地铁 / 高架 / 废墟 / 撤离 / 警报 -> `director_10`
- `原创 / 群像表演` + 舞台 / 群舞 / 音乐 / 聚光灯 / 入场 / 定格 -> `director_11`

You may solve this with local alias lists or a tiny local normalization helper
inside `shot_language_selection.rs`. Do not add dependencies.

### B. Remove fabricated default routing

If no cue matches, the selector must not fabricate a `director_02` crowd-pressure
route.

The result should stay conservative:

- either no `lane_candidates`
- or an explicit non-routed metadata state with no invented planning signal

But it must not assign a lane that source evidence did not justify.

### C. Add tests

Add fixture-only tests for:

- `routes_chinese_guoman_wuxia_to_director_08_metadata_only`
- `routes_chinese_epic_group_or_war_to_director_09_metadata_only`
- `routes_chinese_urban_apocalypse_to_director_10_metadata_only`
- `routes_chinese_stage_group_performance_to_director_11_metadata_only`
- `unknown_evidence_does_not_fabricate_default_lane`

Keep existing English synthetic tests if they still represent the same planning
surface.

## Test Commands

Re-run:

```text
cargo test -p storyboard-pipeline --test v108_shot_language_selection
cargo test -p storyboard-pipeline
```

If the local Rust toolchain metadata issue is still blocking, keep the patch
small, commit it, and report the same environment blocker explicitly. Do not
widen scope to chase toolchain repair from the main implementation thread.

## Forbidden Scope

Do not:

- change app/runtime/product behavior
- change exporter/workbook/IPC
- change validator/repair
- change KB / V3 / desktop / intake
- import V108 rows
- promote positive few-shot
- create registry rows
- create product-ready handles
- create `reference_control_core`
- write final storyboard words
- write runtime prompt text
- call Qwen or Seedance
- change Cargo dependencies

## Completion

Commit and push to `origin/codex/contracts-freeze`.

Report back to main control:

- final branch / commit
- changed files
- fix summary
- new Chinese/source-text test cases
- unknown-evidence behavior
- test results, or the remaining environment blocker if tests still cannot run
