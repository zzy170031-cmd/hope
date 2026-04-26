# V0 Author Screenwriter Director Continuity Contract 2026-04-26

## Scope

This is a docs-only V0 contract for the full creative chain:

```text
user topic or synopsis
-> KB authoring and narrative craft summaries
-> novel chapter or story setting
-> screenwriting adaptation script
-> director staging and scheduling intent
-> shot task split
-> shot storyboard rows
-> prompt_text
-> finalized storyboard area
-> continuity summary
-> next chapter or next section continuation
```

This contract defines product fields and stage boundaries only. It does not
modify runtime code, tests, desktop UI, the KB 23-field main schema, seed data,
exports, validators, provider calls, or packaged runtime behavior.

Hope V0 is not a generic text generator. V0 must preserve a structured
author-to-director chain so a user can continue a story across chapters or
sections without losing accepted storyboard decisions.

It does not:

- modify runtime code
- modify tests
- modify desktop UI or IPC
- modify `hope-kb`
- modify the KB 23-field main schema
- add `director_style_ref`
- introduce real author, IP, director, or brand style imitation
- expose raw `prompt_body`
- expose `source_register`
- expose overlay JSON
- expose full raw KB rows
- write or display API keys
- connect live Doubao
- connect Seedance runtime
- generate video

Related contracts:

- `docs/text-model-prompt-contract-2026-04-24.md`
- `docs/kb-retrieval-router-contract-2026-04-24.md`
- `docs/desktop-packaged-router-runtime-boundary-2026-04-24.md`
- `docs/shot-script-grounding-adaptive-scene-contract-2026-04-25.md`
- `docs/finalized-storyboard-bank-export-contract-2026-04-25.md`

## Contract Decision

The following stages are V0 product contract stages, not V1 or V2 deferrals:

```text
generate_novel_chapter
adapt_chapter_to_script
split_script_to_shot_tasks
generate_storyboard
save_storyboard_shot_result
update_continuity_state
```

The chain may be deterministic, model-assisted, or mixed in a later runtime
gate, but the contract shape is V0. Later implementation may choose local
fallbacks when live providers are disabled or unavailable.

V0 continuity is summary-level continuity state plus finalized storyboard
references. It is not a video runtime, a hidden raw KB replay mechanism, or a
real-author style clone.

## V0 Content Priority Ladder

The V0 priority order is:

```text
content
> screenwriting structure
> director staging and scheduling
> shot storyboard
> prompt_text
```

`content` is the first priority. Content decides:

- story theme
- character goal
- conflict
- emotional arc
- chapter progression
- why the audience keeps watching

Screenwriting structure serves content. It turns novel chapters and story
settings into script that can be performed, split, and advanced.

Director staging serves content and screenwriting structure. Director KB may
enhance rhythm, performance continuity, emotional progression, audience focus,
and blocking, but it must not rewrite character facts, goals, locations,
conflicts, timeline, or continuity facts for technique.

Shot storyboard serves director staging. A shot must not take over the story or
generate images unrelated to current plot facts.

`prompt_text` is the final video-storyboard prompt product. It expresses only
the current shot image. It must not carry internal KB, rules, samples, trace, or
debug information.

## V0 Contract Invariants

These invariants are part of the V0 contract:

```text
content_facts_are_source_of_truth = true
director_layer_must_not_override_content = true
camera_layer_must_not_override_story_facts = true
authoring_layer_must_not_clone_real_author_style = true
authoring_layer_must_preserve_story_continuity = true
authoring_layer_must_not_exaggerate_for_style = true
authoring_layer_must_not_break_character_motivation = true
authoring_layer_must_not_break_timeline = true
authoring_layer_must_not_break_prop_state = true
authoring_layer_must_keep_next_scene_bridge = true
kb_director_hints_are_advisory = true
```

`content_facts_are_source_of_truth`

- user story facts, accepted chapter facts, continuity state, and finalized
  storyboard references outrank KB hints

`director_layer_must_not_override_content`

- director suggestions may improve staging, rhythm, and performance focus
- director suggestions may not replace who wants what, where the scene is, what
  conflict is active, or what continuity already established

`camera_layer_must_not_override_story_facts`

- camera and shot decisions may clarify current action
- camera and shot decisions may not invent unrelated spectacle, switch the
  story subject, or contradict the scene facts

`authoring_layer_must_not_clone_real_author_style`

- authoring craft is generic creative capability
- it is not a real-author style library, IP style clone, or brand-style clone

`authoring_layer_must_preserve_story_continuity`

- authoring craft must treat story continuity as its first constraint
- writing technique must serve character goals, relationships, conflict,
  emotional progression, location, timeline, props, foreshadowing, finalized
  continuity, and the next-section handoff

`authoring_layer_must_not_exaggerate_for_style`

- prose may be vivid, but it must not exaggerate for style in a way that
  rewrites user facts, inserts unrelated spectacle, or sacrifices clear plot

`authoring_layer_must_not_break_character_motivation`

- a character's goal, pressure, emotion, and action motivation must remain
  continuous across adjacent passages unless the user explicitly revises them
  or the continuity delta explains the turn

`authoring_layer_must_not_break_timeline`

- time, order, and location transitions must stay explainable and compatible
  with continuity state

`authoring_layer_must_not_break_prop_state`

- key prop ownership, placement, damage state, and foreshadowing function must
  not change without an explicit continuity delta

`authoring_layer_must_keep_next_scene_bridge`

- every accepted chapter or section must leave a usable bridge for the next
  script, shot split, or story continuation

`kb_director_hints_are_advisory`

- KB director hints are advisory summaries
- accepted story and continuity facts are binding

## Relationship To Earlier V1/V2 Notes

Earlier docs may describe broader finalized storyboard bank export or automatic
cross-shot regeneration as later work. This V0 contract is narrower and earlier:

- V0 must be able to save an accepted storyboard shot result as a product-safe
  finalized reference.
- V0 must be able to update compact continuity state from accepted chapter,
  script, shot, storyboard, and finalized-reference decisions.
- V0 must be able to feed that continuity state into the next chapter or next
  section.

This does not open full multi-shot export behavior, automatic regeneration from
the finalized bank, desktop UI, database migrations, or video runtime. Those
remain separate implementation gates.

## V0 Capability Layers

V0 must carry four compressed capability layers across the chain:

```text
authoring_craft_summary
screenwriting_adaptation_summary
directing_kb_context_summary
continuity_context_summary
```

These layers are product-safe summaries. They may inform stage output, review,
validation, and continuation, but they must not become raw prompt dumps,
full-KB payloads, or hidden style imitation channels.

`authoring_craft_summary`

- compressed guidance for premise, theme, narrative tension, chapter shape,
  character desire, stakes, and prose pacing
- derived from generic craft rules, accepted user intent, and allowed KB
  summaries
- must not name or imitate a real author, IP, brand, or protected style
- represents the `authoring_craft` content-creation capability layer, not a
  real-author style library

`authoring_craft` may include these generic capability signals:

```text
premise_hook
character_desire
character_pressure
conflict_engine
emotional_turn
suspense_setup
payoff_setup
scene_purpose
visualizable_action
chapter_cliffhanger
screenplay_compression
director_bridge
```

These signals are content-craft labels only. They must not contain real author
names, real IP names, brand names, or protected-style imitation labels.

`screenwriting_adaptation_summary`

- compressed guidance for turning chapter prose into script beats, dialogue
  intent, scene transitions, and action/reaction rhythm
- may preserve plot meaning while changing medium-specific structure
- must not override user intent or accepted continuity state

`directing_kb_context_summary`

- compressed directing and storyboard guidance for camera, blocking,
  shot purpose, shot duration, scene adaptation, and prompt_text constraints
- may be populated from the KB Router summary-only boundary
- must not contain `director_style_ref`, real director names, raw source rows,
  raw `prompt_body`, `source_register`, or overlay JSON

`continuity_context_summary`

- compressed state from previous chapters, finalized storyboard decisions, and
  accepted continuity deltas
- used to continue the next chapter or next section without replaying raw
  project history
- must not expose provider transcripts, debug traces, full raw KB rows, or API
  keys

## Layer Responsibilities And Outputs

### Authoring Layer

The authoring layer is responsible for:

- generating plot with a content hook
- making clear what the character wants
- making clear what pressures the character
- building an ongoing conflict engine
- designing emotional turns
- planting suspense and payoff setup
- turning abstract psychology into visible, filmable action
- giving downstream screenwriting and director layers a clear content intent
- preserving story continuity before style, rhetorical intensity, or prose
  flourish
- ensuring the next section can inherit the current character, conflict,
  location, timeline, prop, and emotional state

Allowed authoring outputs:

```text
authoring_craft_summary
premise_hook
character_desire
character_pressure
conflict_engine
emotional_turn
suspense_setup
payoff_setup
scene_purpose
visualizable_action
chapter_cliffhanger
screenplay_compression
director_bridge
chapter_summary
character_motivation_summary
conflict_progression_summary
emotional_progression_summary
timeline_continuity_summary
prop_state_summary
next_scene_bridge
continuity_warnings
continuity_delta
```

The authoring layer must not:

- imitate a real author
- clone a real IP or brand style
- rewrite explicit user facts
- exaggerate user facts for prose style
- make character motivation jump only for drama
- insert unrelated imagery, side plots, or large-scale spectacle for technique
- sacrifice clear plot for a sense of refinement
- change finalized character relationships to force conflict
- break shot filmability for a polished sentence
- damage timeline, location, or prop state for emotional intensity
- produce a passage that cannot be continued by script or shot generation
- bypass the summary-only KB boundary
- become KB-free generic text generation when KB context is available

### Screenwriting Layer

The screenwriting layer is responsible for:

- turning novel chapters or story settings into script
- preserving user facts, characters, locations, props, conflicts, and timeline
- compressing literary description into performable action, dialogue, and
  scenes
- preserving content intent while changing medium-specific structure

Required screenwriting outputs:

```text
scene_beats
dialogue_intent
action_blocks
turning_points
scene_purpose
script_summary
screenwriting_adaptation_summary
continuity_delta
```

### Director Layer

The director layer reads authoring and screenwriting outputs. It is responsible
only for improving:

- rhythm
- scene staging
- performance continuity
- emotional progression
- audience attention
- blocking

Required director outputs:

```text
director_intent_summary
performance_focus
blocking_hint
rhythm_hint
visual_focus
continuity_note
directing_kb_context_summary
```

Director outputs must not override content facts. Director KB hints remain
advisory even when they are relevant.

### Shot Layer

The shot layer turns director intent into concrete storyboard rows.

Required shot outputs:

```text
shot_intent
shot_scene_type
shot_title
character_action
prompt_text
```

Every row must serve the current plot fragment. It must not generate a
beautiful but unrelated image, switch the story subject, or contradict
continuity.

### prompt_text Layer

`prompt_text` is the final video-storyboard prompt text for the current shot.

It may express:

- current shot image
- subject and action
- visual continuity that belongs to the shot
- clean Seedance2.0 storyboard wording

It must not express:

- raw KB rows
- raw `prompt_body`
- selected sample payloads
- rule IDs
- retrieval trace
- source register data
- overlay JSON
- debug notes
- provider secrets
- real author, IP, director, or brand imitation labels

## Story Length Profiles

V0 must define these story length profiles:

```text
short_story_2000_2500
two_minute_story_2500_3500
```

`short_story_2000_2500`

- target prose length: 2000-2500 Chinese characters or equivalent story volume
- intended for one compact chapter, short story segment, or single strong beat
- expected shape: premise, escalation, turn, resolution hook
- may produce fewer shot tasks when adapted

`two_minute_story_2500_3500`

- target prose length: 2500-3500 Chinese characters or equivalent story volume
- intended for a richer two-minute story segment before shot splitting
- expected shape: stronger setup, clearer midpoint, more action/reaction
  beats, and enough continuity anchors for storyboard generation
- may produce more shot tasks and a denser finalized storyboard area

The length profile controls story volume and pacing only. It is not a video
duration request, not a Seedance runtime request, and not a new KB schema
field.

## KB Boundary

Every stage that uses KB context must stay inside the compressed Router
boundary:

```text
kb_context_summary
selected_sample_ids
selected_kb_rules
retrieval_trace_user_summary
full_kb_rows_included = 0
```

`kb_context_summary`

- product-safe compressed guidance for the current stage
- should summarize relevant craft, screenwriting, directing, validation, and
  continuity rules
- must not include raw source rows, raw workbook payloads, raw `prompt_body`,
  `source_register`, overlay JSON, real author/director/IP/brand references,
  API keys, local paths, or provider debug data

`selected_sample_ids`

- stable identifiers for selected KB samples
- may support audit and reproducibility
- must not be expanded into full sample rows in product payloads,
  `prompt_text`, exports, logs, or user-visible summaries

`selected_kb_rules`

- compressed rule summaries selected for the current task
- may guide authoring, adaptation, directing, validation, and continuity
- must not carry raw row bodies or real style imitation labels

`retrieval_trace_user_summary`

- short user-safe trace explaining why a small set of summaries/rules was used
- may mention broad reasons such as scene match, continuity need, duration
  profile, or shot intent
- must not expose raw `retrieval_trace` JSON, internal hashes, local cache
  paths, source workbook details, or excluded raw candidate payloads

`full_kb_rows_included`

- must always equal `0`
- applies to authoring, script adaptation, storyboard generation, finalized
  storyboard save, continuity update, provider requests, exports, logs, and
  desktop-visible summaries

If a stage needs more KB help, it must request a narrower compressed selection.
It must not attach the full KB package.

`generate_novel_chapter` is included in this boundary. It must not be treated
as KB-free generic text generation when a KB summary is available. Its allowed
KB fields are:

```text
kb_context_summary
selected_sample_ids
selected_kb_rules
retrieval_trace_user_summary
full_kb_rows_included = 0
```

The chapter generation stage must not receive raw KB rows, raw `prompt_body`,
`source_register`, overlay JSON, API keys, tokens, provider secrets, real-author
style imitation labels, real IP labels, brand-style labels, or
`director_style_ref`.

## Conflict Handling

If an authoring or director KB suggestion conflicts with user story content,
accepted continuity state, or finalized storyboard references, the system must
resolve the conflict in this order:

```text
1. user story content
2. continuity state
3. finalized storyboard bank
4. KB suggestion downgraded to advisory or dropped
```

Allowed warning codes:

```text
kb_director_hint_dropped_due_to_story_conflict
authoring_hint_dropped_due_to_user_fact_conflict
continuity_fact_overrode_kb_hint
authoring_continuity_break_detected
authoring_exaggeration_dropped
authoring_style_overrode_story_blocked
character_motivation_drift_detected
timeline_drift_detected
prop_state_conflict_detected
next_scene_bridge_missing
```

`kb_director_hint_dropped_due_to_story_conflict`

- emitted when a director KB hint would change a story fact, character goal,
  location, conflict, timeline, or continuity fact

`authoring_hint_dropped_due_to_user_fact_conflict`

- emitted when an authoring craft hint would override the user's premise,
  selected facts, characters, or intended conflict

`continuity_fact_overrode_kb_hint`

- emitted when accepted continuity state or finalized storyboard references
  outrank a KB hint

`authoring_continuity_break_detected`

- emitted when chapter prose would break established character, location,
  timeline, prop, relationship, or finalized-storyboard continuity

`authoring_exaggeration_dropped`

- emitted when an exaggerated authoring suggestion is dropped because it does
  not serve current story facts

`authoring_style_overrode_story_blocked`

- emitted when prose style would reduce clarity, alter facts, or obscure the
  plot and is therefore blocked

`character_motivation_drift_detected`

- emitted when character desire, pressure, emotion, or action motivation drifts
  without accepted story evidence

`timeline_drift_detected`

- emitted when time order, location transition, or scene progression would jump
  without explanation

`prop_state_conflict_detected`

- emitted when a key prop's ownership, placement, damage state, or
  foreshadowing role conflicts with continuity

`next_scene_bridge_missing`

- emitted when an accepted section does not leave a usable bridge for the next
  chapter, script, shot split, or storyboard generation

If an `authoring_craft` suggestion conflicts with story continuity, the system
must resolve the conflict in this order:

```text
1. continuity state
2. finalized storyboard bank
3. user input facts
4. authoring craft downgraded to advisory or dropped
5. product-safe warning returned
```

The system must not silently rewrite continuity facts to preserve style,
intensity, or prose flourish.

Warnings may be returned as product-safe metadata. They must not copy raw KB
rows, raw prompt bodies, source register data, overlay JSON, provider payloads,
or secrets into user-visible text.

## Stage Chain Contract

The required V0 stage order is:

```text
generate_novel_chapter
adapt_chapter_to_script
split_script_to_shot_tasks
generate_storyboard
save_storyboard_shot_result
update_continuity_state
```

Stages may be retried or repaired, but the accepted output of an earlier stage
must remain the strongest source for the next stage unless the user explicitly
revises it.

### generate_novel_chapter

Purpose:

- convert user topic, synopsis, or continuation request into a chapter or story
  segment
- apply `authoring_craft_summary`
- respect selected story length profile
- read `continuity_context_summary` when continuing an existing story

Required input fields:

```text
story_id
chapter_id
chapter_order
user_topic_or_synopsis
story_length_profile
authoring_craft_summary
continuity_context_summary
kb_context_summary
selected_sample_ids
selected_kb_rules
retrieval_trace_user_summary
full_kb_rows_included
```

Required output fields:

```text
story_id
chapter_id
chapter_order
chapter_title
chapter_text
chapter_summary
authoring_craft_summary
premise_hook
character_desire
character_pressure
conflict_engine
emotional_turn
suspense_setup
payoff_setup
scene_purpose
visualizable_action
chapter_cliffhanger
director_bridge
character_motivation_summary
conflict_progression_summary
emotional_progression_summary
timeline_continuity_summary
prop_state_summary
next_scene_bridge
continuity_warnings
continuity_delta
warnings
```

Rules:

- `chapter_text` is product story text, not prompt/debug metadata
- `chapter_summary` must be short enough to feed continuation
- `generate_novel_chapter` must use summary-only KB context when available
- `full_kb_rows_included` must equal `0`
- content facts from user input and continuity are stronger than KB hints
- authoring craft must preserve character motivation, timeline, prop state, and
  next-scene bridge
- prose style must be dropped when it conflicts with clear story continuity
- `continuity_delta` records newly established facts that later stages must
  preserve
- output must not imitate a real author, IP, brand, or protected style

### adapt_chapter_to_script

Purpose:

- adapt accepted `chapter_text` into script form
- preserve plot, character intent, location, props, and timeline constraints
- apply `screenwriting_adaptation_summary`

Required input fields:

```text
story_id
chapter_id
chapter_order
chapter_text
chapter_summary
screenwriting_adaptation_summary
continuity_context_summary
kb_context_summary
selected_sample_ids
selected_kb_rules
retrieval_trace_user_summary
full_kb_rows_included
```

Required output fields:

```text
script_id
story_id
chapter_id
chapter_order
script_text
script_summary
screenwriting_adaptation_summary
scene_beats
dialogue_intent
action_blocks
turning_points
scene_purpose
continuity_delta
warnings
```

Rules:

- `script_text` may change medium and structure but must not rewrite accepted
  plot meaning without user intent
- scene beats, dialogue intent, action blocks, turning points, and scene
  purpose must serve the accepted content facts
- dialogue and scene headings must stay product text
- raw KB rows, raw `prompt_body`, provider logs, and overlay data are forbidden

### split_script_to_shot_tasks

Purpose:

- split accepted `script_text` into ordered shot tasks
- preserve shot-level story grounding before `generate_storyboard`
- allocate shot duration and shot intent

Required input fields:

```text
script_id
story_id
chapter_id
chapter_order
script_text
script_summary
selected_total_duration_seconds
primary_scene_type
primary_scene_label
primary_scene_category
directing_kb_context_summary
continuity_context_summary
kb_context_summary
selected_sample_ids
selected_kb_rules
retrieval_trace_user_summary
full_kb_rows_included
```

Required output fields:

```text
script_id
story_id
chapter_id
chapter_order
shot_tasks
shot_task_count
duration_plan_summary
continuity_delta
warnings
```

Each `shot_tasks` item must carry:

```text
shot_task_id
shot_order
shot_task_name
shot_script
shot_intent
shot_scene_type
shot_scene_label
adaptation_reason
shot_duration_seconds
duration_source
```

Rules:

- `shot_script` is the strongest grounding source for storyboard generation
- if `shot_scene_type` differs from `primary_scene_type`, non-empty
  `adaptation_reason` is required
- shot duration remains a storyboard planning value, not a Seedance runtime
  call

### generate_storyboard

Purpose:

- generate storyboard rows and row-level `prompt_text` from accepted shot tasks
- apply `directing_kb_context_summary`
- keep the existing shot grounding priority:

```text
shot_script
script_text
primary scene fields
KB Router summary
```

Required input fields:

```text
story_id
chapter_id
chapter_order
script_id
shot_task_id
shot_order
shot_script
shot_intent
shot_scene_type
shot_scene_label
adaptation_reason
shot_duration_seconds
duration_source
director_intent_summary
performance_focus
blocking_hint
rhythm_hint
visual_focus
continuity_note
directing_kb_context_summary
continuity_context_summary
kb_context_summary
selected_sample_ids
selected_kb_rules
retrieval_trace_user_summary
full_kb_rows_included
```

Required output fields:

```text
storyboard_result_id
story_id
chapter_id
chapter_order
script_id
shot_task_id
shot_order
rows
prompt_text
shot_duration_seconds
duration_source
director_intent_summary
performance_focus
blocking_hint
rhythm_hint
visual_focus
continuity_note
directing_kb_context_summary
continuity_delta
warnings
```

Rules:

- `rows` must be product-safe storyboard rows
- director intent and camera choices must serve content facts and the accepted
  script
- director hints are advisory if they conflict with content or continuity
- final `prompt_text` must be clean Seedance2.0 Chinese video-storyboard
  wording for the row or shot
- final `prompt_text` must not include KB summary, sample IDs, rule IDs,
  retrieval trace, grounding metadata, raw `prompt_body`, `source_register`,
  overlay JSON, full raw KB rows, internal hashes, debug traces, API keys,
  local paths, real director names, concrete IP names, or brand names
- generation may use summaries, but it must not expose them inside
  `prompt_text`

### save_storyboard_shot_result

Purpose:

- save a confirmed or reviewable storyboard shot result into the finalized
  storyboard area
- connect finalized shots to story, chapter, script, and shot-task identity
- preserve the product-safe row and `prompt_text` boundary

Required input fields:

```text
story_id
chapter_id
chapter_order
script_id
shot_task_id
storyboard_result_id
shot_order
shot_task_name
rows
prompt_text
shot_duration_seconds
duration_source
confirmed
updated_at_ms
rows_hash
continuity_delta
```

Required output fields:

```text
finalized_storyboard_ref
story_id
chapter_id
chapter_order
script_id
shot_task_id
storyboard_result_id
confirmed
updated_at_ms
rows_hash
warnings
```

`finalized_storyboard_ref` must be a product-safe reference, not a raw row dump,
provider transcript, source workbook pointer, local absolute path, or Seedance
runtime payload.

### update_continuity_state

Purpose:

- update summary-level continuity after chapter text, script, shot tasks,
  storyboard rows, or finalized storyboard refs are accepted
- provide compact context for the next chapter or next section
- prevent the next generation from contradicting accepted story facts

Required input fields:

```text
story_id
chapter_id
chapter_order
chapter_summary
character_state_summary
location_state_summary
prop_state_summary
timeline_state_summary
unresolved_threads
style_bible_summary
finalized_storyboard_refs
continuity_delta
```

Required output fields:

```text
story_id
chapter_id
chapter_order
continuity_context_summary
character_state_summary
location_state_summary
prop_state_summary
timeline_state_summary
unresolved_threads
style_bible_summary
finalized_storyboard_refs
continuity_delta
updated_at_ms
warnings
```

Rules:

- continuity state must be compact summary state, not full raw chapter text,
  full raw storyboard rows, provider transcripts, or full KB rows
- `finalized_storyboard_refs` may point to accepted storyboard results but must
  not inline raw internal payloads
- `continuity_delta` must describe what changed since the previous accepted
  state
- the next `generate_novel_chapter` call may use
  `continuity_context_summary` as accepted context

## Continuity Field Definitions

The required V0 continuity fields are:

```text
story_id
chapter_id
chapter_order
chapter_summary
character_state_summary
location_state_summary
prop_state_summary
timeline_state_summary
unresolved_threads
style_bible_summary
finalized_storyboard_refs
continuity_delta
```

`story_id`

- stable project story identifier
- partitions continuity state across different user stories

`chapter_id`

- stable chapter or section identifier
- links chapter text, script, shot tasks, storyboard results, and continuity
  updates

`chapter_order`

- numeric order for continuation
- next chapter generation must treat lower accepted orders as prior context

`chapter_summary`

- compressed summary of the accepted chapter or section
- used for continuation and review
- must not include raw provider debug data or full prompts

`character_state_summary`

- compact summary of character identity, relationships, emotional state,
  goals, wounds, wardrobe-relevant facts, and changed status
- must not be a raw transcript or hidden prompt dump

`location_state_summary`

- compact summary of established locations, spatial relations, atmosphere, and
  continuity-sensitive environmental facts

`prop_state_summary`

- compact summary of important objects, ownership, placement, damage state, or
  story function

`timeline_state_summary`

- compact summary of time of day, sequence order, elapsed time, deadlines,
  flashbacks, or time jumps

`unresolved_threads`

- open plot questions, promised payoffs, pending conflicts, or emotional hooks
- used to continue rather than restart the story

`style_bible_summary`

- generic style and tone rules accepted for this story
- may include pacing, mood, language level, genre contract, and visual
  consistency notes
- must not name or imitate a real author, real director, concrete IP, or brand

`finalized_storyboard_refs`

- references to accepted storyboard shots or finalized storyboard bank entries
- may include IDs and short product-safe labels
- must not inline raw rows, raw `prompt_body`, `source_register`, overlay JSON,
  full KB rows, provider logs, API keys, or local paths

`continuity_delta`

- compact summary of changes introduced by the latest accepted stage
- may be appended to or merged into continuity state
- should identify changed character, location, prop, timeline, style, and
  unresolved-thread facts

## Next Chapter Continuation Contract

The next chapter or next section may use:

```text
story_id
previous_chapter_id
next_chapter_id
next_chapter_order
user_continuation_request
story_length_profile
authoring_craft_summary
continuity_context_summary
character_state_summary
location_state_summary
prop_state_summary
timeline_state_summary
unresolved_threads
style_bible_summary
finalized_storyboard_refs
kb_context_summary
selected_sample_ids
selected_kb_rules
retrieval_trace_user_summary
full_kb_rows_included
```

The continuation output must not silently reset accepted continuity. If the user
asks for a change that conflicts with existing continuity, the stage should
produce a warning or explicit continuity delta rather than hiding the conflict.

## Product Field Boundary

The following may enter V0 product contracts:

```text
generate_novel_chapter
adapt_chapter_to_script
split_script_to_shot_tasks
generate_storyboard
save_storyboard_shot_result
update_continuity_state
authoring_craft_summary
screenwriting_adaptation_summary
directing_kb_context_summary
continuity_context_summary
authoring_craft
premise_hook
character_desire
character_pressure
conflict_engine
emotional_turn
suspense_setup
payoff_setup
scene_purpose
visualizable_action
chapter_cliffhanger
screenplay_compression
director_bridge
scene_beats
dialogue_intent
action_blocks
turning_points
director_intent_summary
performance_focus
blocking_hint
rhythm_hint
visual_focus
continuity_note
shot_intent
shot_scene_type
shot_title
character_action
prompt_text
content_facts_are_source_of_truth
director_layer_must_not_override_content
camera_layer_must_not_override_story_facts
authoring_layer_must_not_clone_real_author_style
authoring_layer_must_preserve_story_continuity
authoring_layer_must_not_exaggerate_for_style
authoring_layer_must_not_break_character_motivation
authoring_layer_must_not_break_timeline
authoring_layer_must_not_break_prop_state
authoring_layer_must_keep_next_scene_bridge
kb_director_hints_are_advisory
kb_director_hint_dropped_due_to_story_conflict
authoring_hint_dropped_due_to_user_fact_conflict
continuity_fact_overrode_kb_hint
authoring_continuity_break_detected
authoring_exaggeration_dropped
authoring_style_overrode_story_blocked
character_motivation_drift_detected
timeline_drift_detected
prop_state_conflict_detected
next_scene_bridge_missing
short_story_2000_2500
two_minute_story_2500_3500
kb_context_summary
selected_sample_ids
selected_kb_rules
retrieval_trace_user_summary
full_kb_rows_included
story_id
chapter_id
chapter_order
chapter_summary
character_motivation_summary
conflict_progression_summary
emotional_progression_summary
timeline_continuity_summary
next_scene_bridge
continuity_warnings
character_state_summary
location_state_summary
prop_state_summary
timeline_state_summary
unresolved_threads
style_bible_summary
finalized_storyboard_refs
continuity_delta
```

The following must not enter product fields, visible summaries, exports, logs,
provider payloads, or `prompt_text`:

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
provider transcript
Seedance runtime payload
real author imitation label
real director imitation label
concrete IP style label
brand style label
director_style_ref
```

## Validator Requirements

A later implementation validator must check:

```text
stage order is preserved or explicitly retried
story_length_profile is one of the V0 allowed profiles
authoring_craft_summary is present for generate_novel_chapter
generate_novel_chapter uses summary-only KB context when KB context is available
generate_novel_chapter is not KB-free generic text generation
screenwriting_adaptation_summary is present for adapt_chapter_to_script
scene_beats, dialogue_intent, action_blocks, turning_points, and scene_purpose serve accepted content
directing_kb_context_summary is present for generate_storyboard
director_intent_summary, performance_focus, blocking_hint, rhythm_hint, visual_focus, and continuity_note serve content facts
continuity_context_summary is present when continuing an existing story
selected_sample_ids remain IDs only
selected_kb_rules remain compressed summaries
retrieval_trace_user_summary is user-safe
full_kb_rows_included == 0
content_facts_are_source_of_truth == true
director_layer_must_not_override_content == true
camera_layer_must_not_override_story_facts == true
authoring_layer_must_not_clone_real_author_style == true
authoring_layer_must_preserve_story_continuity == true
authoring_layer_must_not_exaggerate_for_style == true
authoring_layer_must_not_break_character_motivation == true
authoring_layer_must_not_break_timeline == true
authoring_layer_must_not_break_prop_state == true
authoring_layer_must_keep_next_scene_bridge == true
kb_director_hints_are_advisory == true
authoring craft suggestions downgrade or drop when they conflict with continuity
chapter output includes character_motivation_summary
chapter output includes conflict_progression_summary
chapter output includes emotional_progression_summary
chapter output includes timeline_continuity_summary
chapter output includes prop_state_summary
chapter output includes next_scene_bridge
chapter output includes continuity_warnings
same character goal, emotion, and action motivation stay continuous across passages
key prop state does not change without continuity delta
location and timeline do not jump without explanation
chapter ending leaves a bridge for the next section
authoring output contains no unrelated technique-driven imagery, side plot, or spectacle
novel-to-script adaptation preserves story facts
storyboard rows can trace back to original story continuity
KB hints downgrade when they conflict with user story content
continuity state outranks conflicting KB hints
finalized storyboard refs outrank conflicting KB hints
warning code is returned when KB or authoring hints are dropped
chapter_id and chapter_order propagate through script, shot tasks, storyboard, finalized refs, and continuity
finalized_storyboard_refs do not inline raw rows or internal payloads
continuity_delta is non-empty when accepted facts change
each storyboard row serves the current plot fragment
prompt_text remains clean Seedance2.0 storyboard prompt text
prompt_text carries current shot image only
prompt_text carries no internal KB, rules, sample payloads, trace, or debug information
no raw prompt_body leakage
no source_register leakage
no overlay JSON leakage
no full raw KB rows leakage
no API key or token leakage
no provider authorization leakage
no user local path leakage
no Seedance runtime payload leakage
no director_style_ref
no real author, real director, concrete IP, or brand style imitation label
```

## Still Gated

The following remain closed:

- runtime implementation
- tests
- desktop UI or IPC binding
- `hope-kb` seed edits
- KB 23-field schema changes
- `director_style_ref`
- live Doubao
- Seedance runtime
- video generation
- provider credential persistence changes
- export behavior changes

## Completion Standard

This contract is complete when this docs-only file exists on
`codex/contracts-freeze` and no forbidden runtime, test, desktop, KB schema,
seed, provider, or export files are staged or committed.

No code tests are required for this docs-only gate. Validation is limited to
Git status, docs diff inspection, and Markdown contract review.
