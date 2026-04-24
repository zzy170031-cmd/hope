# KB Retrieval Router Contract 2026-04-24

## Scope

This is a docs-only contract for the low-token KB routing layer that sits
between desktop business input and future text-model prompt assembly.

It does not:

- modify the 23-field KB schema
- add `director_style_ref` or any other field to the main storyboard table
- promote reserve rows or rows with `usable_for_fewshot = No`
- connect live Qwen
- connect live Doubao
- connect Seedance runtime
- generate video
- modify desktop, intake, `hope-kb`, seed files, runtime code, validators, or
  exporter behavior

The contract extends the boundary already stated in
`docs/text-model-prompt-contract-2026-04-24.md`: Hope must not send the full
152-row KB package to a text model.

## Problem Statement

The KB snapshot may be large enough for product reasoning, validation, and
traceability, but a real product call must be low-token.

The product route is:

1. desktop sends a small business request
2. local Router derives retrieval signals
3. local Router reads the read-only KB snapshot
4. local Router selects a small top-k set of samples and rule summaries
5. prompt assembly receives only compressed context
6. the final model request never carries the full KB snapshot

## Desktop Request Contract

Desktop may provide only this input envelope to the Router:

```json
{
  "scene_type": "string",
  "synopsis_text": "string",
  "duration_seconds": 60,
  "task_type": "generate_storyboard",
  "shot_intent": "optional string",
  "structure_type": "optional string"
}
```

Required fields:

- `scene_type`
- `synopsis_text`
- `duration_seconds`
- `task_type`

Optional fields:

- `shot_intent`
- `structure_type`

Allowed `task_type` values for this contract:

- `expand_script`
- `generate_storyboard`
- `repair_storyboard`
- `compile_seedance_prompt_text`

The Router must not require desktop to send raw KB rows, source workbooks,
overlay JSON, director references, API keys, user paths, media asset bindings,
or Seedance payloads.

## Retrieval Signal Derivation

The Router may derive local retrieval signals from the desktop request.

Allowed internal signals:

- normalized `scene_type`
- normalized `shot_intent`
- normalized `structure_type`
- `duration_bucket`
- `task_type`
- short synopsis keyword set
- blocker hints from forbidden terms or obvious placeholder text
- `synopsis_hash`

The signals are local planning inputs only. They are not main-table fields,
workbook columns, exporter columns, prompt text, or user-visible style labels.

The Router must preserve the original user `synopsis_text` as product input.
It may not rewrite the synopsis inside the cache key, trace, or validator
result in a way that changes user intent.

## KB Snapshot Read Contract

The Router reads from a read-only `kb_runtime_snapshot`.

The snapshot is treated as:

- versioned
- checksum-verifiable
- immutable during a product call
- local to the installed product package or a verified product data update

The Router may read:

- sample IDs
- sample eligibility fields
- scene and structure tags
- rule and coverage summaries
- validator and repair planning mappings
- source provenance needed for traceability

The Router must not mutate:

- KB rows
- the 23-field schema
- `library_status`
- `usable_for_fewshot`
- source prompt bodies
- reserve status

## Reserve And Eligibility Gates

Rows with `library_status = reserve` remain reserve.

Rows with `usable_for_fewshot = No` remain ineligible for positive few-shot
selection.

The Router must not place either category in `selected_sample_ids` as positive
sample context.

Reserve or ineligible rows may appear only as:

- excluded candidates inside `retrieval_trace`
- blocker evidence summarized by a stable rule ID
- validation evidence that explains why a row was not used

They must not be promoted into product import, positive few-shot context, final
storyboard wording, `prompt_text`, or model request payloads.

## Router Response Contract

The Router returns only this top-level response shape:

```json
{
  "selected_sample_ids": ["sample_id"],
  "selected_kb_rules": [
    {
      "rule_id": "string",
      "family": "routing | continuity | duration | prompt_text | forbidden_terms | reserve_gate | repair",
      "summary": "short compressed rule summary",
      "applies_to": ["generate_storyboard"]
    }
  ],
  "kb_context_summary": "compressed local summary for prompt assembly",
  "retrieval_trace": {
    "kb_version": "string",
    "snapshot_id": "string",
    "snapshot_checksum": "string",
    "content_cache_key": "string",
    "task_type": "generate_storyboard",
    "top_k_samples": 3,
    "top_k_rules": 8,
    "duration_seconds": 60,
    "selection_reasons": [
      {
        "sample_id": "sample_id",
        "reason_code": "scene_match | shot_intent_match | structure_match | duration_match | repair_match"
      }
    ],
    "excluded_candidates": [
      {
        "sample_id": "sample_id",
        "reason_code": "reserve_gate | usable_for_fewshot_no | placeholder_present | reference_unresolved"
      }
    ],
    "token_budget": {
      "kb_context_summary_target": "800-1500 Chinese characters",
      "full_kb_rows_included": 0
    }
  }
}
```

No other top-level Router output is allowed in this contract.

`retrieval_trace` is allowed because traceability needs to explain which local
snapshot and gates produced the compressed context. Trace fields must remain
debug and audit metadata. They must not be copied into final `prompt_text`.

## Top-K Rules

Default sample selection:

| task | default top_k_samples | maximum without a later gate |
| --- | ---: | ---: |
| `expand_script` | 0-2 | 3 |
| `generate_storyboard` | 3-5 | 5 |
| `repair_storyboard` | 1-2 | 2 |
| `compile_seedance_prompt_text` | 0-1 | 1 |

Default rule selection:

| task | default top_k_rules | maximum without a later gate |
| --- | ---: | ---: |
| `expand_script` | 4-8 | 10 |
| `generate_storyboard` | 8-15 | 18 |
| `repair_storyboard` | 6-12 | 15 |
| `compile_seedance_prompt_text` | 4-10 | 12 |

The Router should prefer fewer samples when rules already cover the task.
Repair tasks should prefer local validator failure context plus one or two
closest samples, not broad few-shot context.

## Token Budget

`kb_context_summary` should target 800-1500 Chinese characters.

The summary should include:

- task-relevant scene and structure guidance
- selected sample IDs
- short sample lessons, not raw rows
- selected rule summaries
- known blockers and reserve-gate notes when relevant
- duration and prompt-text constraints

The summary must not include:

- the full 152-row KB
- full source row JSON
- raw workbook rows
- raw `prompt_body`
- overlay JSON
- real director, IP, or brand names
- API keys
- user local paths
- debug stack traces
- cache file paths

If prompt assembly needs more context, it must ask the Router for a narrower
selection, not attach the full KB.

## Cache Key Contract

The required content cache key uses:

```text
kb_version + scene_type + shot_intent + structure_type + duration + synopsis_hash
```

`duration` means `duration_seconds`.

`synopsis_hash` must be derived from normalized synopsis text using a stable
non-reversible hash. The route cache may store per-task results under the same
content key, because `task_type` changes top-k behavior while the required
content key remains stable.

The cache value may contain:

- `selected_sample_ids`
- `selected_kb_rules`
- `kb_context_summary`
- `retrieval_trace`
- task-specific top-k metadata

The cache value must not contain:

- API keys
- live provider responses unless a later provider-result cache gate opens
- full KB rows
- raw source workbook data
- user local file paths

## Qwen Request Boundary

A future Qwen text request may carry:

- desktop business input
- `selected_sample_ids`
- compressed `selected_kb_rules`
- `kb_context_summary`
- local validator failures for repair tasks
- target duration

A future Qwen text request must not carry:

- the full 152-row KB package
- raw KB workbook rows
- the full 23-field row payload for selected samples
- reserve rows as positive few-shot context
- rows with `usable_for_fewshot = No` as positive few-shot context
- overlay JSON
- raw `prompt_body` as final prompt
- API keys inside messages
- user local paths
- debug logs

Hard rule:

```text
qwen_request.full_kb_rows_included must always equal 0.
```

Any request that attempts to attach all KB rows, all sample payloads, or raw
source tables must fail validation before network I/O.

## Prompt Text Source Boundary

Final `prompt_text` may only be the compiled Seedance2.0 text-storyboard prompt
for a specific storyboard row.

Allowed source:

- structured storyboard row
- selected KB rule summaries
- selected sample IDs as trace metadata
- deterministic prompt-text compiler output

Forbidden source:

- raw `prompt_body` copied as final text
- internal Router fields
- overlay JSON
- retrieval trace JSON
- real director names
- concrete IP names
- brand names
- API keys
- user paths
- cache paths
- debug information

`prompt_text` is product output. It is not a debug dump.

## Validator Contract

A validator for this route must check:

1. Forbidden terms
   - scan model request text and final `prompt_text`
   - block real director names, concrete IP names, brand names, API keys, user
     paths, cache paths, and internal debug markers
2. Duration conservation
   - generated storyboard row durations must sum to requested
     `duration_seconds`
   - repair must preserve total duration unless the user explicitly changes it
3. Reserve gate
   - `selected_sample_ids` must not include reserve rows as positive context
   - `selected_sample_ids` must not include rows with
     `usable_for_fewshot = No`
   - excluded reserve or ineligible rows must appear only in trace or rule
     evidence
4. Prompt-text source
   - final `prompt_text` must declare compiler provenance as
     `compiled_seedance2_prompt_text`
   - final `prompt_text` must not equal raw `prompt_body`
   - final `prompt_text` must not include overlay JSON or retrieval JSON
5. Full-KB request guard
   - future provider requests must report `full_kb_rows_included = 0`
   - requests with all 152 rows, raw source rows, or full row payload arrays
     must be rejected
6. Trace consistency
   - `retrieval_trace.kb_version` must match the loaded snapshot
   - `retrieval_trace.content_cache_key` must match the required key inputs
   - selected IDs must exist in the loaded snapshot

These checks are contract requirements. This docs-only gate does not implement
them.

## Still Gated

The following remain closed:

- live Qwen
- live Doubao
- Seedance runtime
- video generation
- desktop provider settings UI
- desktop Router IPC binding
- KB schema changes
- `hope-kb` seed edits
- reserve row promotion
- positive few-shot promotion beyond the accepted eligibility gates
- main-table overlay fields
- exporter behavior changes

## Completion Standard

This contract is complete when it is committed as docs-only on
`codex/contracts-freeze`.

No tests are required for this docs-only gate unless a later implementation
gate opens code changes.
