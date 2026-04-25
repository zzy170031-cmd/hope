# Shot Script Grounding And Adaptive Scene Contract 2026-04-25

## Scope

This is a docs-only contract for shot-level script grounding and adaptive scene
classification in `generate_storyboard`.

It does not:

- modify runtime code
- modify desktop UI or IPC
- modify intake
- modify `hope-kb`
- modify the KB 23-field main schema
- add `director_style_ref`
- connect live Doubao
- connect Seedance runtime
- generate video
- promote reserve rows
- promote rows with `usable_for_fewshot = No`

This contract sits downstream of script expansion and upstream of storyboard
row generation. It may be implemented later only through a separate runtime
gate.

Related contracts:

- `docs/text-model-prompt-contract-2026-04-24.md`
- `docs/kb-retrieval-router-contract-2026-04-24.md`
- `docs/desktop-packaged-router-runtime-boundary-2026-04-24.md`

## Contract Decision

`generate_storyboard` must be grounded first in shot-level story text.

The generation priority is frozen as:

1. `shot_script`
2. `expanded_script_text`
3. primary scene fields
4. KB Router summary

The KB Router summary may guide selection, constraints, and style-safe
compression. It must not override explicit shot story content.

## Contract Fields

The following fields are product-contract fields for this docs-only planning
surface. They are not KB schema changes.

```text
shot_script
expanded_script_text
primary_scene_type
primary_scene_label
primary_scene_category
shot_scene_type
shot_scene_label
shot_intent
adaptation_reason
split_script_to_shot_tasks
shot_duration_seconds
duration_source
```

## Field Definitions

`shot_script`

- shot-level source text for one storyboard shot
- derived from `expanded_script_text` or supplied by a later accepted caller
- the strongest grounding source for visual description, action, dialogue, and
  shot-specific scene adaptation
- must remain story text, not internal metadata

`expanded_script_text`

- expanded story/script text for the whole requested scene or segment
- fallback grounding source when no accepted `shot_script` exists
- may be split into shot tasks by `split_script_to_shot_tasks`
- must not contain API keys, provider debug data, raw KB rows, or internal
  overlay data

`primary_scene_type`

- normalized scene type for the whole request or segment
- derived from desktop request, project context, or accepted script analysis
- used as the broad default when shot-level evidence is absent

`primary_scene_label`

- human-readable label for the primary scene
- used for review, trace, and product-facing explanation
- must not be a real director, IP, or brand reference

`primary_scene_category`

- coarse scene family for routing and validation
- may support duration, structure, and KB Router selection
- must not be used as a hidden style imitation channel

`shot_scene_type`

- normalized scene type for a single shot task
- may differ from `primary_scene_type` only when `shot_script` or accepted
  shot evidence supports the adaptation
- must remain local to the shot task or generated storyboard row

`shot_scene_label`

- human-readable label for the shot-level scene
- explains the shot-local scene classification
- must not introduce real director, IP, or brand references

`shot_intent`

- compact shot-purpose signal, such as establishing, action beat, reaction,
  transition, reveal, dialogue, repair, or prompt compilation
- may help KB Router selection
- must not replace `shot_script`

`adaptation_reason`

- required whenever `shot_scene_type` differs from `primary_scene_type`
- short reason explaining the evidence for adaptation
- must cite story evidence by summary, not raw internal payloads
- examples of allowed reason families:
  - action beat requires local chase classification
  - dialogue beat requires intimate conversation classification
  - transition beat requires bridge or location-shift classification
  - repair task requires validator failure localization

`split_script_to_shot_tasks`

- deterministic or model-assisted planning step that divides
  `expanded_script_text` into shot-level tasks
- output is planning input for `generate_storyboard`
- does not call Seedance runtime
- does not generate video
- does not change the KB schema

`shot_duration_seconds`

- generated storyboard-row duration for one shot
- belongs to `GeneratedStoryboardRow`
- must mirror the allocated row duration from the accepted storyboard duration
  plan
- must not be inferred from KB samples, raw source rows, or prompt text

`duration_source`

- source marker explaining where `shot_duration_seconds` came from
- belongs to `GeneratedStoryboardRow`
- current allowed value:

```text
storyboard_duration_plan.allocated_row_duration_seconds
```

- any additional value requires a later contract gate

## split_script_to_shot_tasks Input

Allowed input fields:

```text
script_id
expanded_script_text
selected_total_duration_seconds
primary_scene_type
primary_scene_label
primary_scene_category
task_type
shot_count_hint
structure_type
kb_context_summary
selected_kb_rules
selected_sample_ids
```

Required input fields:

```text
expanded_script_text
selected_total_duration_seconds
primary_scene_type
```

Optional input fields:

```text
script_id
primary_scene_label
primary_scene_category
task_type
shot_count_hint
structure_type
kb_context_summary
selected_kb_rules
selected_sample_ids
```

Input must not include:

```text
raw prompt_body
source_register
overlay JSON
full raw KB rows
API key
token
plaintext secret
user local path
provider debug log
Seedance runtime payload
```

## split_script_to_shot_tasks Output

The output is a list of shot tasks.

Each shot task must include:

```text
shot_task_id
shot_order
shot_script
duration_seconds
shot_scene_type
shot_scene_label
shot_intent
adaptation_reason
grounding_source
```

Allowed `grounding_source` values:

```text
shot_script
expanded_script_text
primary_scene_fields
kb_router_summary
```

`adaptation_reason` may be empty only when:

```text
shot_scene_type == primary_scene_type
```

`duration_seconds` across all shot tasks must sum to:

```text
selected_total_duration_seconds
```

## GeneratedStoryboardRow Duration Fields

`GeneratedStoryboardRow` must carry explicit row-duration fields when the
runtime gate materializes this contract:

```text
shot_duration_seconds
duration_source
```

`shot_duration_seconds` is the product-facing row duration.

`duration_source` currently allows only:

```text
storyboard_duration_plan.allocated_row_duration_seconds
```

`shot_duration_seconds` must be duration-plan grounded. It must not be copied
from:

```text
KB summary
sample_id
selected_sample_ids
rule id
retrieval trace
grounding_source
primary_scene_type
shot_scene_type
raw prompt_body
source_register
overlay JSON
```

## generate_storyboard Input Priority

`generate_storyboard` must resolve grounding in this order:

```text
1. shot_script
2. expanded_script_text
3. primary scene fields
4. KB Router summary
```

Priority behavior:

- `shot_script` controls shot-local action, visual content, dialogue, and
  scene adaptation.
- `expanded_script_text` controls whole-segment continuity when shot text is
  incomplete.
- primary scene fields provide default classification and routing when shot
  evidence is ambiguous.
- KB Router summary provides compact rule and sample guidance only after story
  grounding is established.

The Router summary cannot override explicit `shot_script`.

## Adaptive Scene Rules

Shot-level scene adaptation is allowed only when evidence exists.

Allowed adaptation evidence:

```text
shot_script
expanded_script_text
validator failure localized to the shot
accepted primary scene fields
compressed KB Router rule summary
```

Forbidden adaptation evidence:

```text
raw prompt_body
source_register
overlay JSON
full raw KB rows
real director name
concrete IP name
brand name
API key
token
plaintext secret
user local path
```

If `shot_scene_type` differs from `primary_scene_type`, the row or shot task
must carry `adaptation_reason`.

If no valid evidence exists, `shot_scene_type` must default to
`primary_scene_type`.

## Product Field Boundary

The following may enter product contracts:

```text
shot_script
expanded_script_text
primary_scene_type
primary_scene_label
primary_scene_category
shot_scene_type
shot_scene_label
shot_intent
adaptation_reason
selected_total_duration_seconds
script_id
shot_task_id
shot_duration_seconds
duration_source
selected_sample_ids
selected_kb_rules
kb_context_summary
```

The following must not enter product fields:

```text
raw prompt_body
source_register
overlay JSON
full raw KB rows
API key
token
plaintext secret
provider authorization header
user local path
debug stack trace
Seedance runtime payload
```

`selected_sample_ids`, `selected_kb_rules`, and `kb_context_summary` are
allowed only in their compressed Router contract form.

## Prompt Text Boundary

Final `prompt_text` may only be compiled Seedance2.0 text-storyboard wording
for the generated storyboard row.

It may be informed by product-safe compiler inputs:

```text
shot_script
structured storyboard row
continuity constraints
```

The final `prompt_text` must remain clean Seedance2.0 Chinese
video-storyboard prompt text. It must not mix in internal routing, trace,
schema, KB, or grounding metadata.

It must not include:

```text
KB summary
sample_id
selected_sample_ids
rule id
retrieval trace
grounding_source
primary_scene_type
shot_scene_type
raw prompt_body
source_register
overlay JSON
retrieval_trace JSON
full raw KB rows
API key
token
plaintext secret
user local path
real director name
concrete IP name
brand name
debug information
```

`selected_kb_rules` and `kb_context_summary` may guide compilation outside the
final text, but their IDs, summaries, and trace labels must not appear inside
`prompt_text`.

## Role Action Quality Warning

The role/action grounding warning name is:

```text
role_action_grounding_incomplete
```

This warning is allowed when a generated row has insufficient shot-grounded
evidence for `role_action` or `character_action`.

It may be emitted only as validator or quality metadata. It must not be copied
into:

```text
prompt_text
Seedance prompt payload
raw product prose
KB fields
source rows
```

## Validator Requirements

A later implementation validator must check:

```text
shot_script presence or fallback reason
generate_storyboard grounding priority
shot_scene_type adaptation evidence
adaptation_reason required on scene-type changes
duration conservation
shot_duration_seconds source
duration_source allowed value
role_action_grounding_incomplete warning boundary
forbidden terms
no raw prompt_body leakage
no source_register leakage
no overlay JSON leakage
no selected_sample_ids leakage into prompt_text
no KB summary leakage into prompt_text
no retrieval trace leakage into prompt_text
no full raw KB rows leakage
no API key or token leakage
no user local path leakage
```

Duration conservation rule:

```text
sum(shot_task.duration_seconds) == selected_total_duration_seconds
```

Storyboard row duration source rule:

```text
GeneratedStoryboardRow.duration_source == storyboard_duration_plan.allocated_row_duration_seconds
```

The validator must fail if `shot_scene_type` changes without a non-empty
`adaptation_reason`.

The validator must fail if product fields are populated from forbidden sources.

## Still Gated

The following remain closed:

- runtime implementation
- desktop UI or IPC binding
- live Doubao
- live Seedance runtime
- video generation
- KB 23-field schema changes
- `director_style_ref`
- `hope-kb` seed edits
- reserve row promotion
- `usable_for_fewshot = No` promotion
- exporter behavior changes

## Completion Standard

This contract is complete when the docs-only file exists on
`codex/contracts-freeze` and no forbidden files are staged or committed.

No code tests are required for this docs-only gate. Validation is limited to
Git status, docs diff inspection, and Markdown contract review.
