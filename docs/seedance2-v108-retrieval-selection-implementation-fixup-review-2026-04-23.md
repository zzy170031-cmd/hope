# Seedance2 V108 Retrieval Selection Implementation Fixup Review 2026-04-23

## Review Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- reviewed implementation commit:
  `9023ce1` (`storyboard-pipeline: fix V108 retrieval cue routing`)
- prior fixup dispatch:
  `eb7af8c` (`docs: request V108 retrieval selection fixup`)
- review scope: main-control acceptance of the focused retrieval-selection fixup

This review covers only the accepted implementation slice inside
`storyboard-pipeline`.

## Decision

`SEEDANCE2_V108_RETRIEVAL_SELECTION_IMPLEMENTATION_FIXUP_ACCEPTED`

Main control accepts the retrieval-selection implementation slice as the current
internal metadata baseline.

The accepted implementation remains:

- non-runtime
- metadata-only
- no final storyboard words
- no runtime prompt text
- no Qwen / Seedance calls
- no exporter / workbook / IPC change
- no registry rows
- no product-ready `external_reference_handles`
- no `reference_control_core`

## Accepted Fixes

The focused patch satisfies the required corrective items:

1. Chinese / source-text cue routing is now covered for the accepted V108
   evidence families.
2. Unknown evidence no longer fabricates a default `director_02` route.
3. New fixture-only tests were added for Chinese/source-text routing and the
   unknown-evidence conservative path.

Accepted route examples:

- `国漫 / 热血打斗` and `国漫 / 场域追逐` can route to `director_08`
- `国漫 / 群像表演` with `战争 / 誓师 / 阵列 / 旗鼓` can route to `director_09`
- `都市末世 / 工业压迫` with `地铁 / 高架 / 废墟 / 撤离 / 警报` can route to
  `director_10`
- `原创 / 群像表演` with `舞台 / 群舞 / 音乐 / 聚光灯 / 入场 / 定格` can route to
  `director_11`
- unknown evidence now returns no lane candidates and remains `RawEvidenceOnly`

## Validation

Main control re-ran the required tests with the working toolchain:

```text
rustup run 1.95.0-x86_64-pc-windows-msvc cargo test -p storyboard-pipeline --test v108_shot_language_selection
rustup run 1.95.0-x86_64-pc-windows-msvc cargo test -p storyboard-pipeline
```

Results:

- `v108_shot_language_selection`: `14 passed`
- `storyboard-pipeline` package tests: `16 passed`
- `Doc-tests storyboard_pipeline`: passed

The default `stable` toolchain on this machine still has rustlib / metadata
damage. That remains an environment note, not a blocker for accepting this code
slice, because the accepted tests are green under the isolated
`1.95.0-x86_64-pc-windows-msvc` toolchain.

## Assertions Remain Frozen

```text
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
qwen_calls = 0
seedance_calls = 0
desktop_or_intake_changes = 0
v3_branch_edits = 0
```

## Route Impact

Hope main now has the accepted internal retrieval metadata selector slice.

The primary remaining MVP blocker is desktop packaging / release toolchain
recovery, not the retrieval selector logic.
