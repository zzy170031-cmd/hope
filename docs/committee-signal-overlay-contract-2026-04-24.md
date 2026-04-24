# Committee Signal Overlay Contract 2026-04-24

## Scope

This is a docs-only contract for optional planning overlays associated with KB
retrieval and committee-style storyboard planning signals.

It does not:

- modify the 23-field KB schema
- add `director_style_ref` or any overlay field to the main storyboard table
- make overlays required for generation or repair
- promote reserve rows
- promote rows with `usable_for_fewshot = No`
- connect live Qwen, Doubao, or Seedance
- modify desktop, intake, `hope-kb`, seed files, runtime code, validators, or
  exporter behavior

This contract defines two optional planning-only overlays:

- `committee_signal_overlay`
- `retrieval_signal_overlay`

Both overlays are sidecar planning metadata. They are not product output.

## Overlay Decision

`committee_signal_overlay` and `retrieval_signal_overlay` are allowed only as
optional planning or debug metadata.

They must not become:

- required request fields
- required response fields for desktop
- main-table columns
- workbook columns
- exporter payload fields
- Seedance prompt fields
- Qwen message content
- validator pass conditions
- replacement for `selected_sample_ids`
- replacement for `selected_kb_rules`
- replacement for `kb_context_summary`
- replacement for `retrieval_trace`

The product must still be able to run when overlays are absent.

## Relationship To Router Contract

The Router top-level response remains fixed by
`docs/kb-retrieval-router-contract-2026-04-24.md`:

```text
selected_sample_ids
selected_kb_rules
kb_context_summary
retrieval_trace
```

Any overlay data must live inside local planning/debug storage or inside a
trace-only extension that is explicitly omitted from model messages and final
exports unless a later debug-export gate opens.

The Router must not return overlay JSON as top-level product response.

## Committee Signal Overlay

`committee_signal_overlay` may describe internal planning votes or signal
families.

Allowed planning shape:

```json
{
  "overlay_type": "committee_signal_overlay",
  "overlay_version": "2026-04-24",
  "scope": "planning_only",
  "applies_to": "generate_storyboard | repair_storyboard",
  "signals": [
    {
      "lane_id": "internal_lane_id",
      "signal_family": "action_axis | emotion_axis | suspense_axis | stage_axis | continuity_axis | duration_axis",
      "reason_code": "scene_match | shot_intent_match | structure_match | repair_match",
      "confidence": 0.72
    }
  ],
  "blocked": [
    {
      "reason_code": "reserve_gate | reference_unresolved | placeholder_present | promotion_gate_not_accepted"
    }
  ]
}
```

Allowed content:

- internal lane IDs
- abstract signal families
- reason codes
- confidence values
- blocker names
- source snapshot/version IDs

Forbidden content:

- real director names
- concrete IP names
- brand names
- final storyboard wording
- final `prompt_text`
- raw `prompt_body`
- full KB rows
- source workbook rows
- API keys
- user local paths
- asset URLs
- image paths
- reference-control payloads

The name `director_style_ref` remains forbidden as a main-table field and must
not be introduced by this overlay.

## Retrieval Signal Overlay

`retrieval_signal_overlay` may explain the local signal derivation that led to
a Router selection.

Allowed planning shape:

```json
{
  "overlay_type": "retrieval_signal_overlay",
  "overlay_version": "2026-04-24",
  "scope": "planning_only",
  "input_signal_summary": {
    "scene_type": "normalized scene type",
    "shot_intent": "normalized optional shot intent",
    "structure_type": "normalized optional structure type",
    "duration_bucket": "short | medium | long",
    "task_type": "generate_storyboard"
  },
  "selected_sample_ids": ["sample_id"],
  "selected_rule_ids": ["rule_id"],
  "excluded_reason_codes": [
    "reserve_gate",
    "usable_for_fewshot_no",
    "placeholder_present"
  ]
}
```

Allowed content:

- normalized signal names
- selected IDs
- selected rule IDs
- exclusion reason codes
- token-budget estimates
- cache-key metadata

Forbidden content:

- full selected sample payloads
- all 152 KB rows
- raw source rows
- raw `prompt_body`
- model messages
- final generated text
- API keys
- user paths
- desktop UI state
- runtime provider responses

## Planning-Only Boundary

Overlay metadata can support:

- local debugging
- route review
- validator planning
- later implementation scoping
- trace inspection during development

Overlay metadata cannot directly drive:

- final storyboard prose
- final `prompt_text`
- Qwen request messages
- Seedance prompt payloads
- workbook export rows
- table schema
- acceptance of reserve rows
- acceptance of rows with `usable_for_fewshot = No`
- duration repair
- forbidden-term suppression

If a later implementation wants overlays to affect product behavior, it needs a
separate scope gate naming exact files, tests, and forbidden files.

## Model Request Boundary

Future Qwen or custom text-provider requests may receive:

- `kb_context_summary`
- compressed selected rule summaries
- selected sample IDs
- user business input
- validator failure context for repair

They must not receive:

- `committee_signal_overlay`
- `retrieval_signal_overlay`
- overlay JSON
- full KB rows
- raw source rows
- raw `prompt_body`
- real director or IP names
- API keys
- user local paths
- debug logs

Any prompt assembly layer that sees overlay JSON must drop it before building a
provider request.

## Export Boundary

Product exports may include:

- final storyboard rows
- final `prompt_text`
- total duration plan
- selected sample IDs for traceability
- selected rule IDs or short selected rule summaries
- KB snapshot version/checksum

Product exports must not include:

- full overlay JSON
- internal committee vote arrays
- raw source rows
- all KB rows
- raw `prompt_body`
- API keys
- user local paths
- debug stack traces

A separate debug-export gate may later decide whether overlay metadata can be
exported into a non-product audit artifact. This contract does not open that
gate.

## Validator Boundary

Validators must not require overlays to pass.

Validators may check that overlays did not leak into:

- final `prompt_text`
- Qwen request messages
- Seedance prompt payloads
- product workbook rows
- normal user exports

Validators still rely on contract data:

- `selected_sample_ids`
- `selected_kb_rules`
- `kb_context_summary`
- `retrieval_trace`
- generated storyboard rows
- compiled `prompt_text`
- duration plan

Overlay absence is valid.

## Still Gated

The following remain closed:

- overlay-driven runtime generation
- overlay-driven validator acceptance
- overlay fields in the main table
- `director_style_ref`
- live Qwen
- live Doubao
- live Seedance
- video generation
- desktop UI or IPC changes
- exporter/workbook changes
- reserve row promotion
- `hope-kb` seed edits

## Completion Standard

This contract is complete when it is committed as docs-only on
`codex/contracts-freeze`.

No tests are required for this docs-only gate unless a later implementation
gate opens code changes.
