# Video Model Target Adapter Contract 2026-04-25

## Scope

This is a docs-only contract for how Hope may later expose video-model target
adapters without turning the main Hope product into a text-to-video app.

It does not:

- modify runtime code
- modify desktop, `hope-kb`, or intake
- connect Seedance runtime
- connect HappyHorse
- generate video inside Hope
- add API key storage
- change the Qwen text-generation mainline
- change KB Router summary-only rules

## Main Product Boundary

Hope remains a product for structured creative planning and export.

The main product outputs stay:

1. script text
2. storyboard rows
3. shot language and structure
4. `prompt_text`
5. KB Router compressed context
6. export bundles

Hope is not redefined here as a direct video generation application.

## Target Adapter Principle

Future video-model integrations must sit behind a `target_adapter` boundary.

That means:

- Hope core produces structured business outputs
- target adapters consume those outputs
- target adapters do not rewrite Hope's main workflow
- target adapters do not backflow new schema requirements into the main product

The direction is one-way:

`Hope structured result -> export bundle -> target_adapter`

Not:

`video model runtime -> redesign Hope core`

## Seedance2.0 Position

Seedance2.0 is the current primary target adapter.

Its current position is intentionally narrow:

- prompt package target
- `prompt_text` adaptation target
- export contract target

It is not:

- the in-app runtime of Hope
- the current execution engine inside desktop
- a video generation endpoint connected in this gate

So the current product truth is still:

- Hope compiles and exports Seedance2.0-ready text
- Hope does not submit jobs to Seedance2.0 in this contract

## HappyHorse Position

HappyHorse enters only as:

- `reserved`
- `watchlist`
- `unstable_api`

It must not be treated as:

- default provider
- production adapter
- guaranteed available runtime
- accepted export target by default

Desktop and runtime must not assume HappyHorse is enabled just because the name
appears in docs.

## HappyHorse Promotion Gates

HappyHorse may only move out of reserved/watchlist status when all of the
following are clearly confirmed:

1. API documentation is stable
2. authentication behavior is stable
3. pricing is stable
4. region availability is explicit
5. rate limits are explicit
6. error codes are explicit
7. async task model is explicit
8. compliance boundary is explicit

If any of these are still unclear, HappyHorse stays reserved.

## Adapter Input Boundary

A target adapter may consume only Hope export-facing structured outputs.

Allowed adapter-facing inputs:

- storyboard rows
- `prompt_text`
- prompt package metadata
- export manifest metadata
- `selected_sample_ids`
- `selected_kb_rules`
- `kb_context_summary`

Not allowed as adapter input:

- full KB rows
- raw source workbook rows
- raw `prompt_body`
- raw `teaching_note`
- `source_register`
- provenance JSON
- overlay JSON
- desktop-only state
- local asset paths as implicit media binding

## KB Boundary Stays Frozen

The KB boundary does not widen for video target adapters.

The only KB-derived payload allowed across this contract remains:

- `kb_context_summary`
- `selected_sample_ids`
- `selected_kb_rules`

And the invariant remains:

- `full_kb_rows_included = 0`

So a target adapter must never receive the full 152-row KB package as its live
context payload from Hope.

## Security Boundary

Secrets remain outside code, logs, exports, and chat.

Rules:

- API keys must not be committed to the repo
- API keys must not be printed into logs
- API keys must not be written into export bundles
- API keys must not be pasted into chat as operating procedure
- credential references may exist later, but raw secrets stay external

This contract does not add any new secret storage mechanism.

## Failure and Fallback Rule

If a future target adapter is unavailable, unstable, gated, or policy-blocked,
Hope must fall back to local export behavior.

The fallback is:

- keep local `prompt_text`
- keep prompt package export
- keep storyboard/export bundle output
- do not break the main user workflow

Failure of a target adapter must not invalidate Hope's main text and export
deliverables.

## Non-Goals

This contract does not:

- promote HappyHorse to production
- open Seedance runtime execution
- change the Qwen text path
- change desktop UI
- change KB schema
- add video job orchestration
- add async render queue behavior
- add asset upload orchestration

## Current Contract Summary

The current route is:

- Hope mainline stays text-first and export-first
- Seedance2.0 stays the primary target adapter at the prompt/export boundary
- HappyHorse stays reserved/watchlist/unstable
- adapters may consume Hope exports later
- adapters may not redefine Hope core
