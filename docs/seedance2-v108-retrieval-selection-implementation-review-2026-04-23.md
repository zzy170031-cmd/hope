# Seedance2 V108 Retrieval Selection Implementation Review 2026-04-23

## Review Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- reviewed implementation commit:
  `d10bd49` (`storyboard-pipeline: add V108 retrieval metadata selector`)
- reviewed scope anchor:
  `29bbf81` (`docs: scope V108 retrieval selection implementation`)
- review scope: main-control code review and acceptance / return-for-patch

The implementation stays inside the accepted file boundary:

```text
crates/storyboard-pipeline/src/shot_language_selection.rs
crates/storyboard-pipeline/src/lib.rs
crates/storyboard-pipeline/tests/v108_shot_language_selection.rs
```

However, main control does not accept this patch as fully green yet.

## Decision

`SEEDANCE2_V108_RETRIEVAL_SELECTION_IMPLEMENTATION_RETURN_FOR_PATCH`

The implementation is in scope and on the right slice, but it needs one small
corrective patch before it should be treated as the accepted retrieval metadata
baseline.

## Findings

### 1. Actual V108 evidence is still Chinese/source-text, but the selector only routes on English cue words

In
[shot_language_selection.rs](E:\codex\hope\crates\storyboard-pipeline\src\shot_language_selection.rs:257)
through
[shot_language_selection.rs](E:\codex\hope\crates\storyboard-pipeline\src\shot_language_selection.rs:439),
the routing logic only checks English cue strings such as `guofeng`, `war`,
`urban`, `stage`, `action`, `emotion`, and `suspense`.

The accepted V108 planning route is built from source surfaces like
`style_cluster`, `scene_category`, and `camera_directing_core`, whose real
evidence today is Chinese / source text (`国漫`, `热血打斗`, `场域追逐`, `群像表演`,
`都市末世`, `工业压迫`, `舞台`, `群舞`, etc.). Without Chinese aliases or an
explicit normalization layer inside this non-runtime module, the new lanes
`director_08` through `director_11` will not trigger for the actual evidence
families this patch is meant to model.

The tests in
[v108_shot_language_selection.rs](E:\codex\hope\crates\storyboard-pipeline\tests\v108_shot_language_selection.rs:108)
through
[v108_shot_language_selection.rs](E:\codex\hope\crates\storyboard-pipeline\tests\v108_shot_language_selection.rs:205)
also use only English synthetic strings, so this gap is not covered.

### 2. Unknown evidence currently fabricates a `director_02` route instead of staying conservative

In
[shot_language_selection.rs](E:\codex\hope\crates\storyboard-pipeline\src\shot_language_selection.rs:441),
the selector falls back to `director_02` with `CrowdPressure` whenever no
evidence cue matches.

That is too aggressive for the accepted planning contract. For unknown, neutral,
or weak evidence, the selector should keep no lane candidates or a clearly
bounded "no route chosen" metadata state. Defaulting to crowd pressure invents a
route that the source evidence did not justify.

## Environment Note

The implementation thread also reported a local Rust toolchain metadata failure
(`libcore-*.rmeta` / `can't find crate for std`). That is an environment blocker,
not evidence that the code slice is invalid, but it means the required tests are
still not verified green.

## Accepted Boundary Still Holds

The patch does keep these boundaries:

- no final storyboard words
- no runtime prompt text
- no Qwen / Seedance calls
- no exporter / workbook / IPC edits
- no KB / desktop / intake / V3 edits
- no registry rows
- no product-ready `external_reference_handles`
- no `reference_control_core`

## Required Corrective Patch

Main control requires one focused follow-up patch in the same three files only:

1. Add Chinese/source-text aliases or an explicit local normalization layer for
   the accepted V108 evidence families.
2. Remove the fabricated default route to `director_02` when no cue matches.
3. Add tests for Chinese/source-text routing and for the unknown-evidence /
   no-route case.
4. Re-run the accepted `storyboard-pipeline` tests once the local Rust toolchain
   metadata issue is fixed.

## Validation

This review is based on the pushed implementation diff plus the implementation
thread's reported environment-blocked test state. No clean green test rerun was
available at review time.
