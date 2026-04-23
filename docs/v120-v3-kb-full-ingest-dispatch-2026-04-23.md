# V120 V3 KB Full Ingest Dispatch 2026-04-23

## Control Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- current control anchor before this dispatch: `055dc0f`
  (`docs: dispatch desktop single-page workbench`)
- desktop active gate: single-page workbench rebuild
- intake state: green standby
- Hope main route remains `RC_READY + scope freeze`

This dispatch opens a **bounded synchronized V3 + KB full-ingest gate**.

It does **not** reopen Hope product runtime implementation, desktop runtime
behavior, intake runtime behavior, Qwen, Seedance, live Hope import of all 120
rows, product-ready external references, or `reference_control_core`.

## Source Priority

Primary source:

- V120 local workbook (`golden-sample-v120-xlsx`)
- V120 local control memo (`golden-sample-v120-doc`)

Comparison baseline:

- accepted V108 workbook and memo in
  `E:\codex\hope-kb\golden-samples\seedance2-v108`

Working rule:

- V120 is the new primary source
- V108 is the comparison baseline
- KB may ingest toward full V120 package state
- Hope main runtime truth does not automatically become 120 because KB ingests
  120

## Why This Gate Is Opened

The user has explicitly decided that this sample batch should be ingested into
the KB as the full version.

Main control accepts that with these conditions:

1. the work stays in V3 documentation / source-material alignment and KB-only
   package ingest
2. the work does not widen Hope main runtime scope
3. the desktop single-page workbench rebuild remains the active product-facing
   thread and is not blocked by this ingest gate

## Thread 1: V3 Source Alignment

Suggested thread label:

```text
Hope-V3FieldOverlay【V120样本对齐中】
```

Repo / branch:

```text
E:\codex\hope / codex/v3-field-overlay-proposal
```

Required first step:

- fast-forward local V3 branch to the accepted remote anchor
  `origin/codex/v3-field-overlay-proposal @ bc745a4`

Task:

- use V120 as the latest primary source input
- compare V120 against accepted V108 baseline and current V3 docs
- refresh the V3 docs-only overlay package so it reflects the latest source
  reality
- preserve explicit field mapping, deprecated assumptions, unresolved gaps,
  placeholder / `future_model_fill_surface`, and the planning-vs-implementation
  split

Allowed:

- `docs/contracts/export-overlays/**`
- V3 docs-only source notes
- updated sample counts, field deltas, renamed fields, preserved gaps
- docs-only handoff to KB

Forbidden:

- no edits on `codex/contracts-freeze`
- no desktop files
- no intake files
- no KB repo edits
- no Rust / validator / exporter / workbook / IPC / Qwen / Seedance work
- no language that implies the fields are already live in Hope runtime

Required report back:

- final branch / commit
- changed files
- V120 vs V108 field / count delta summary
- unresolved gaps preserved
- explicit KB handoff note

## Thread 2: KB Full Ingest

Suggested thread label:

```text
Hope-KB-V120FullIngest【完整版知识库入库中】
```

Repo / branch:

```text
E:\codex\hope-kb / codex/contracts-freeze
```

Task:

- ingest the KB-side sample package toward the full V120 set
- use V120 as the primary workbook / memo source and V108 as the comparison
  baseline
- update the KB package surfaces, source register, manifest, validation, and
  snapshot artifacts as required by the package change
- preserve provenance, negative-sample meaning, few-shot eligibility
  boundaries, and future Hope-consumer boundaries

Expected package surface:

- `seed/v0.2/golden_sample_library.json`
- `seed/v0.2/golden_sample_field_coverage_rules.json`
- `seed/v0.2/golden_sample_failure_mapping.json`
- `seed/v0.2/golden_sample_repair_mapping.json`
- `seed/v0.2/source_register.json`
- `seed/v0.2/manifest.json`
- `docs/live-progress.md`

Required validation:

- seed bundle validation
- snapshot rebuild if manifest or package contents change
- hash / count reporting before and after

Forbidden:

- no `E:\codex\hope` product-code edits
- no desktop edits
- no intake edits
- no Qwen / Seedance integration
- no direct runtime import into Hope main
- no automatic positive few-shot promotion into live Hope runtime use
- no product-ready external references
- no `reference_control_core`

Required report back:

- final branch / commit
- changed files
- V120 vs V108 delta summary
- before / after sample counts
- source register / manifest delta
- validation result
- snapshot result
- remaining closed gates

## Synchronization Rule

The two threads are synchronized like this:

1. V3 first confirms the latest source meaning and field mapping against V120
2. KB ingests against that aligned meaning
3. if KB needs clarification on a renamed field or unresolved gap, it records
   the gap instead of guessing

They may run in parallel, but KB must not silently normalize an unresolved V3
field conflict.

## Shared Boundaries

Until a later written gate opens more scope, both threads must preserve:

- no Hope main runtime changes
- no desktop runtime changes
- no intake runtime changes
- no Qwen
- no Seedance
- no V108 runtime import
- no V120 runtime import into live Hope product use
- no product-ready external reference handles
- no `reference_control_core`
- no fake claim that KB full ingest already equals product runtime readiness

## Exact Next Step

1. sync V3 local branch to `bc745a4`
2. refresh V3 docs against V120, with V108 kept as comparison baseline
3. run KB full ingest against the refreshed V3 meaning and V120 primary source
4. return both results to main control before any Hope product-side runtime gate
