# Seedance2 V120 Full KB Ingest Review 2026-04-23

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- review type: main-control acceptance of V120 as the new full KB ingest source
- route rule: Hope main remains `RC_READY + scope freeze`

This review is docs-only on the Hope main repo. It does not authorize Hope
runtime implementation, desktop runtime changes, intake runtime changes, Qwen
calls, Seedance calls, product-ready external references, or
`reference_control_core`.

## Reviewed Sources

New primary local source provided by the user:

- V120 local workbook (`golden-sample-v120-xlsx`)
- V120 local control memo (`golden-sample-v120-doc`)

Previous comparison baseline:

- accepted V108 workbook in `E:\codex\hope-kb\golden-samples\seedance2-v108`
- accepted V108 memo in `E:\codex\hope-kb\golden-samples\seedance2-v108`

## Verification Summary

Local verification confirms:

- V120 working sheet row count: `120`
- V120 field count: `23`
- V120 remains schema-compatible with the accepted V108 23-column source shape
- V120 status counts:
  - `official=108`
  - `reserve=12`
- V120 sample type counts:
  - `single_shot=72`
  - `sequence_shot=48`
- V120 sequence groups expand from 9 to 12 by adding:
  - `CNSEQ04`
  - `CNSEQ05`
  - `CNSEQ06`
- V120 materially increases the sequence / chase / ensemble emphasis compared
  with the accepted V108 baseline

## Main-Control Decision

`SEEDANCE2_V120_FULL_KB_INGEST_ACCEPTED`

Main control accepts V120 as the new primary source for the next KB ingest
package.

This explicitly changes the previous assumption that V120 should stay only as a
local review workbook. The new control decision is:

1. `hope-kb` may ingest the V120 sample set as the new full KB package target.
2. `V120` is now primary and `V108` becomes the comparison baseline.
3. V108 history is preserved; it is not deleted or rewritten.
4. This acceptance applies to the KB repo package route, not to Hope runtime
   promotion.

## What This Does Authorize

This review authorizes the next KB-side work package to:

- expand the KB sample package from the previous accepted baseline toward the
  full V120 set
- update source register / manifest / validation / snapshot outputs as needed
- align V3 docs to the V120 meaning and counts before or alongside KB ingest

## What This Does Not Authorize

This review still does not authorize:

- importing all 120 rows into live Hope product runtime use
- promoting all 120 rows as runtime positive few-shot truth inside Hope
- Qwen or Seedance integration
- validator / exporter / workbook / IPC implementation
- desktop or intake runtime changes
- product-ready external references
- `reference_control_core`

## Control Interpretation

From this point forward:

- `V120 = full KB ingest target`
- `V108 = previous accepted baseline for comparison`
- `40` current seed rows remain the current live-product reality until a later
  Hope-side runtime gate says otherwise

