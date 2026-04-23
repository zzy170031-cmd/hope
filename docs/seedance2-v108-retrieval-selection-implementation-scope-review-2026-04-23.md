# Seedance2 V108 Retrieval Selection Implementation Scope Review 2026-04-23

## Review Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- reviewed candidate commit:
  `29bbf81` (`docs: scope V108 retrieval selection implementation`)
- reviewed candidate memo:
  `docs/seedance2-v108-retrieval-selection-implementation-scope-candidate-2026-04-23.md`
- prior dispatch:
  `780bdb8` (`docs: dispatch V108 retrieval selection scope`)
- review scope: main-control acceptance of implementation scope candidate

This review accepts a bounded code implementation gate. It does not itself
change Rust code, runtime behavior, Qwen, Seedance, desktop, intake, KB, V3,
exporter, workbook, product import, positive few-shot, registry rows, external
reference handles, or `reference_control_core`.

## Decision

`SEEDANCE2_V108_RETRIEVAL_SELECTION_IMPLEMENTATION_SCOPE_ACCEPTED`

Main control accepts the recommended first implementation slice:

```text
pure planning/debug metadata module + fixture-only tests
```

The next main-thread code gate may implement a non-runtime internal selector
that classifies synthetic V108-like evidence into planning/debug metadata only.

## Accepted Implementation Files

The code gate is limited to:

```text
E:\codex\hope\crates\storyboard-pipeline\src\shot_language_selection.rs
E:\codex\hope\crates\storyboard-pipeline\src\lib.rs
E:\codex\hope\crates\storyboard-pipeline\tests\v108_shot_language_selection.rs
```

No Cargo dependency changes are accepted.

## Accepted Test Commands

The code gate must run:

```text
cargo test -p storyboard-pipeline --test v108_shot_language_selection
cargo test -p storyboard-pipeline
```

If the implementation touches only `storyboard-pipeline`, exporter, validator,
project-store, app, desktop, intake, and KB tests are not required.

## Hard Boundaries

The implementation must not emit:

- final storyboard words
- runtime prompt text
- Qwen message content
- Seedance prompt or overlay wording
- exporter rows
- workbook columns
- product-ready `external_reference_handles`
- `reference_control_core`

The implementation must not import V108 rows, promote positive few-shot, create
registry rows, call Qwen or Seedance, change desktop or intake, or alter
runtime-authoritative storyboard planning behavior.

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

## Next Gate

Main control opens:

```text
Hope主线-Seedance2V108RetrievalSelectionImplementation【小范围实现执行中】
```

Desktop and intake may run parallel readiness / smoke tasks without waiting for
this implementation, because their packages are already on standby and their
tasks can remain isolated from `hope` product code.

## Validation

This was a docs-only control review. No code tests were required or run.
