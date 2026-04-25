# Finalized Storyboard Bank Export Contract 2026-04-25

## Scope

This is a docs-only V1 contract for the finalized storyboard shot bank and
multi-shot export surface.

It does not:

- modify runtime code
- modify desktop UI or IPC
- modify intake
- modify `hope-kb`
- connect live Doubao
- connect Seedance runtime
- generate video
- expose raw `prompt_body`
- expose `source_register`
- expose overlay JSON
- expose API keys or provider secrets

Related contracts:

- `docs/text-model-prompt-contract-2026-04-24.md`
- `docs/kb-retrieval-router-contract-2026-04-24.md`
- `docs/desktop-packaged-router-runtime-boundary-2026-04-24.md`
- `docs/shot-script-grounding-adaptive-scene-contract-2026-04-25.md`

## Contract Decision

Hope V1 needs a local finalized storyboard collection before multi-shot export
can be treated as a whole-project result.

The collection name is:

```text
finalized_storyboard_bank
```

Chinese product label:

```text
已定稿分镜集合
```

The bank stores confirmed shot results. It is not a raw generation cache, not a
KB cache, not a provider transcript, and not a Seedance runtime queue.

## finalized_storyboard_bank Definition

`finalized_storyboard_bank` is a project-local collection of confirmed
storyboard shot results.

It may be used to:

- list already finalized shots
- update a finalized shot after user confirmation
- remove a finalized shot from export scope
- export a multi-shot storyboard bundle
- display per-shot confirmation and prompt status to desktop

It must not be used to:

- store API keys
- store full raw KB rows
- store raw `prompt_body`
- store `source_register`
- store overlay JSON
- store provider debug logs
- trigger live Doubao
- trigger Seedance runtime
- generate video

## Finalized Shot Structure

A single finalized shot result must carry this V1 shape:

```text
project_id
script_id
shot_task_id
result_id
shot_order
shot_task_name
rows
prompt_text
shot_duration_seconds
duration_source
confirmed
updated_at_ms
rows_hash
```

Field meanings:

`project_id`

- project-local identifier
- required for bank partitioning

`script_id`

- script or expanded-script identifier that produced the shot task
- required for traceability

`shot_task_id`

- source shot-task identifier from the shot grounding contract
- required for linking a finalized shot back to shot planning

`result_id`

- generated result identifier for the saved shot result
- required for update and remove operations

`shot_order`

- numeric ordering key for multi-shot export
- export must sort by this field ascending

`shot_task_name`

- human-readable name for desktop display and export review
- must not contain API keys, provider secrets, raw source rows, or internal
  overlay data

`rows`

- generated storyboard rows for the single shot task
- may contain only product-safe storyboard fields
- must not be presented as the whole-film result by itself

`prompt_text`

- clean Seedance2.0 Chinese video-storyboard prompt text for the saved shot
- must not include internal routing, trace, KB, sample, or source metadata

`shot_duration_seconds`

- saved shot-level duration
- should equal the duration represented by the saved rows
- participates in export total-duration summary

`duration_source`

- marker explaining the source of `shot_duration_seconds`
- current allowed value:

```text
storyboard_duration_plan.allocated_row_duration_seconds
```

`confirmed`

- boolean user/product confirmation flag
- only `confirmed = true` shots are eligible for V1 multi-shot export

`updated_at_ms`

- last update timestamp in milliseconds
- used for desktop display and conflict diagnostics

`rows_hash`

- stable hash over the saved `rows` payload
- export must re-check it before packaging
- protects against exporting stale or mismatched rows

## V1 Runtime Surface Candidates

The candidate V1 runtime surface is:

```text
save_storyboard_shot_result
list_storyboard_shot_results
update_storyboard_shot_result
remove_storyboard_shot_result
export_storyboard_bank
```

These names are contract candidates only. This docs-only gate does not create
runtime functions, IPC commands, database tables, migrations, or desktop UI.

## save_storyboard_shot_result

Purpose:

- save one generated shot result into `finalized_storyboard_bank`
- compute or accept a validated `rows_hash`
- set `confirmed` according to explicit caller intent

Required input fields:

```text
project_id
script_id
shot_task_id
result_id
shot_order
shot_task_name
rows
prompt_text
shot_duration_seconds
duration_source
confirmed
updated_at_ms
rows_hash
```

Must reject:

```text
raw prompt_body
source_register
overlay JSON
API key
token
plaintext secret
provider authorization header
Seedance runtime payload
```

## list_storyboard_shot_results

Purpose:

- return finalized bank entries for one project and optional script
- support desktop review of confirmed and unconfirmed shots

Allowed filter fields:

```text
project_id
script_id
confirmed
```

Default order:

```text
shot_order ascending
updated_at_ms ascending within same shot_order
```

## update_storyboard_shot_result

Purpose:

- update one saved shot result after user review
- recalculate or validate `rows_hash`
- preserve `project_id`, `script_id`, `shot_task_id`, and `result_id`
  identity unless a later migration gate opens

Allowed update fields:

```text
shot_order
shot_task_name
rows
prompt_text
shot_duration_seconds
duration_source
confirmed
updated_at_ms
rows_hash
```

The update path must not silently convert one shot's `rows` into the full
project result.

## remove_storyboard_shot_result

Purpose:

- remove one saved shot result from `finalized_storyboard_bank`
- remove it from V1 export eligibility

Required selector fields:

```text
project_id
result_id
```

Removal must not mutate KB data, route cache data, provider credentials, or raw
source records.

## export_storyboard_bank

Purpose:

- export a multi-shot storyboard bundle from confirmed finalized shots

Required input fields:

```text
project_id
export_format
```

Optional input fields:

```text
script_id
include_unconfirmed
```

V1 default:

```text
include_unconfirmed = false
```

Only `confirmed = true` shots are exported by default. Exporting unconfirmed
shots requires a later explicit product gate.

## Export Rules

V1 export must:

```text
sort by shot_order ascending
sum shot_duration_seconds
validate rows_hash
export only confirmed = true shots
preserve per-shot result_id and shot_task_id traceability
```

V1 export must not:

```text
export unconfirmed shots by default
pretend current single-shot rows are the whole-film result
include raw prompt_body
include source_register
include overlay JSON
include API key
include token
include plaintext secret
include provider authorization header
include Seedance runtime payload
trigger Doubao
trigger Seedance runtime
generate video
```

If no confirmed shots exist, `export_storyboard_bank` must return a no-export
result rather than packaging the latest single-shot generation as a full
storyboard.

If `rows_hash` validation fails for any confirmed shot, export must stop and
report the failing `result_id`.

## Export Output Contract

The V1 export response may include:

```text
export_manifest_id
project_id
script_id
confirmed_shot_count
exported_result_ids
total_shot_duration_seconds
artifact_refs
warnings
```

The V1 export response must not include:

```text
raw prompt_body
source_register
overlay JSON
full raw KB rows
API key
token
plaintext secret
provider authorization header
Seedance runtime payload
```

`artifact_refs` must be product artifact references only. They must not expose
user-local absolute paths unless a later desktop/package boundary explicitly
allows that path surface.

## Prompt Text Boundary

Saved and exported `prompt_text` must remain clean Seedance2.0 Chinese
video-storyboard prompt text.

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
API key
token
plaintext secret
provider debug log
```

Prompt text may be shown and exported as product text. It is not a debug
container.

## V1 Continuity Boundary

Desktop V1 may display finalized per-shot continuity status:

```text
confirmed characters
key actions
prompt_text status
shot_order
shot_duration_seconds
confirmed
rows_hash status
```

Desktop V1 may help users review whether the set of confirmed shots is coherent
before export.

V1 does not require later shot generation to read:

```text
confirmed_shots_context
```

Cross-shot continuity generation remains V2.

V2-only areas:

```text
automatic confirmed_shots_context injection
cross-shot character continuity generation
cross-shot action continuity generation
global wardrobe/prop/location continuity planner
automatic regeneration based on finalized bank context
```

## Validation Requirements

A later implementation validator must check:

```text
confirmed = true for exported shots
shot_order sort order
total_shot_duration_seconds sum
rows_hash validity
single-shot rows are not labeled as whole-film result
prompt_text cleanliness
no raw prompt_body leakage
no source_register leakage
no overlay JSON leakage
no API key or token leakage
no Seedance runtime payload
```

The export duration summary must be:

```text
sum(finalized_storyboard_bank.confirmed_shots.shot_duration_seconds)
```

The export set must be:

```text
finalized_storyboard_bank where confirmed == true sorted by shot_order
```

## Still Gated

The following remain closed:

- runtime implementation
- database or project-store schema implementation
- desktop UI or IPC binding
- live Doubao
- live Seedance runtime
- video generation
- V2 cross-shot continuity generation
- automatic `confirmed_shots_context` injection
- KB schema changes
- exporter behavior changes

## Mainline Runtime Next Integration Surface

When total control opens a runtime gate, mainline may evaluate these candidate
surfaces:

```text
save_storyboard_shot_result
list_storyboard_shot_results
update_storyboard_shot_result
remove_storyboard_shot_result
export_storyboard_bank
```

Runtime must keep V1 storage product-safe and must reject forbidden internal
payloads before save or export.

## Desktop Next Integration Surface

When total control opens a desktop gate, desktop may evaluate these V1 surfaces:

```text
finalized shot list
confirmed toggle
per-shot prompt_text status
per-shot key action display
per-shot character display
rows_hash status display
export confirmed shots command
```

Desktop must not display raw `prompt_body`, `source_register`, overlay JSON,
API keys, provider debug logs, or Seedance runtime payloads.

## Completion Standard

This contract is complete when committed as docs-only on
`codex/contracts-freeze`.

No code tests are required for this docs-only gate. Validation is limited to
Git status, docs diff inspection, and Markdown contract review.
