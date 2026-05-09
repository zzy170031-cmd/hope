# Hope Field-Aware Story Contract

Status: executable contract boundary for desktop story expansion, screenplay
rewrite, confirmation, storyboard input, KB oracle use, and QA assertions.

Scope: `扩写故事`, `改写剧本`, scene type switching, target duration switching,
KB/golden structure oracle, field-level validation, visible story body,
storyboard input, `visual_description`, and `prompt_text`.

This document extends:

- `docs/hope-scene-type-rewrite-contract.md`
- `docs/hope-story-fact-frame-storyboard-binding-contract.md`
- `docs/hope-anime-script-confirmation-protocol.md`
- `docs/desktop-release-qa-handoff.md`

It is a contract and integration checklist only. It does not authorize changes
to runtime, UI, runner, certifier, QA matrix, package, commit, or push work.

## 1. Stage Boundary

The product pipeline is fixed as six named frames. Implementations and QA
artifacts must not collapse these names into one generic text blob.

```text
SeedSourceFrame
-> AcceptedNarrativeFrame
-> SceneRewritePlan
-> NarrativeStoryBody
-> SupportNotes
-> StoryboardInputFrame
```

`SeedSourceFrame`

- User original input.
- Stores source facts only: explicit people, role labels, locations, visible
  events, motivations, relationship pressure, event order, and source wording
  that is necessary for fact recovery.
- Does not store scene style, old scene expression, target-duration strategy,
  KB prose, prompt text, storyboard rows, raw trace, or old rewrite output as
  durable facts.

`AcceptedNarrativeFrame`

- The currently confirmed story fact source after the user accepts expansion or
  rewrite.
- Contains the current `NarrativeStoryBody` plus stable atomic facts extracted
  from it.
- Drops old scene expression and old duration pacing when a later scene or
  duration switch happens.

`SceneRewritePlan`

- The executable plan for `扩写故事` or `改写剧本`.
- Inputs are exactly: current accepted source, target `scene_type`, target
  `target_duration`, and KB/golden structure oracle.
- It is not user-visible prose and must not be pasted into the main editor.

`NarrativeStoryBody`

- The user-visible complete story prose.
- Must be the first main block in the UI main editor and the first-screen body
  of the confirmation draft.
- Must not be preceded by support notes, trace, field summaries, KB rationale,
  prompt text, or storyboard decomposition.

`SupportNotes`

- Compact support material only: character notes, scene notes, pacing notes,
  KB summaries, boundary reminders, and validator-readable evidence summaries.
- Must appear only after `NarrativeStoryBody` or in a collapsed/support area.
- Does not participate as a story fact source.

`StoryboardInputFrame`

- Derived only from the confirmed `NarrativeStoryBody`, accepted atomic facts,
  current scene type, current duration, and bounded KB director rules.
- Must not flow backward to overwrite `NarrativeStoryBody`.
- `prompt_text`, storyboard rows, visual descriptions, and support notes are not
  allowed to become the next accepted narrative source.

## 2. Narrative Story Body Contract

For both `扩写故事` and `改写剧本`, the main body must be complete story prose.

Required story elements:

- characters or source role labels
- situation
- motivation
- causality
- conflict progression
- emotional or relationship change
- closing beat
- current `scene_type` expression
- target-duration narrative capacity
- KB oracle influence on structure

Forbidden body shapes:

- action list
- strategy explanation
- storyboard breakdown
- shot list
- prompt
- trace
- validator explanation
- KB explanation
- raw KB rows
- raw sample text
- raw `prompt_body`
- `source_register`
- overlay JSON

Hard body gate:

- If the body can be read as production notes rather than story narration, it
  fails.
- If it only mentions `scene_type` or `target_duration` without changing prose
  expression and capacity, it fails.
- If it exposes KB as an explanation instead of showing KB-shaped structure, it
  fails.
- If it keeps old scene language after a scene switch, it fails.
- If it keeps old duration capacity after a duration switch, it fails.

UI placement gate:

- The UI main editor first visible block must be `NarrativeStoryBody`.
- The confirmation draft first screen must show `NarrativeStoryBody`.
- Support notes, warning summaries, and KB summaries may appear only after that
  body or in folded/support surfaces.

## 3. Field-Aware Validator Semantics

The validator must be field-aware. It must not keep expanding global token
wordlists to guess whether every Chinese term is a person.

Required implementation concept:

```text
FieldAwareEntityResolver(field_name, field_value, source_facts, accepted_frame)
```

Resolver outputs:

- `person_entity`
- `role_label`
- `situation_word`
- `title_word`
- `visible_object`
- `space_word`
- `action_fragment`
- `camera_word`
- `support_only`
- `unknown_person_candidate`
- `forbidden_external_fact`

Field semantics:

`person`

- Strict person or source role field.
- May contain only explicit source names, accepted role labels, or confirmed
  source-bound name groups.
- Unknown people are hard fail.
- Situation words such as `危局`, spaces, camera words, props, body parts,
  action tails, and field labels are never valid `person`.

`shot_title` / `scene_title`

- Title and situation fields.
- May contain situation words, title words, and scene-state words such as
  `危局`.
- Must not introduce a real new person, new source-external faction, new
  place, or new object as fact.

`visual_description`

- Visible-frame field.
- Validates only what is currently visible.
- Must not classify every Chinese token as a person.
- Situation words, camera words, space words, and visible objects stay in their
  own classes unless the accepted source explicitly marks them as people.

`character_action`

- Visible action/performance field.
- Action fragments, conjunction tails, state words, and partial clauses first
  resolve to `action_fragment`, not person.
- A value such as a verb phrase cannot be promoted into `person` merely because
  it begins with a known name or role.

`prompt_text`

- Packaging field.
- Checks structure, leakage, source lineage, and forbidden markers.
- Is not a person fact source and not a story fact source.

`support_notes`

- Support surface only.
- Does not participate in story fact recovery.
- Cannot introduce people, places, props, worldbuilding, or validator-approved
  facts.

Mandatory hard-fail examples:

- `person=危局`
- `person=镜头`
- `person=画面`
- `person=空间`
- `person=当前视觉事件`
- `person=/`
- `person=` empty when the row needs a visible actor
- unknown proper name that is not in source or accepted facts

Mandatory non-person handling:

- `危局` in `shot_title` or `scene_title` is a situation/title word.
- `危局` in `visual_description` is a visible situation state if the frame
  supports it.
- `危局` in `person` is a hard fail.

Three-strike field semantics boundary:

- When three incidents share the same pattern of a non-`person` field promoting
  situation words, action fragments, conjunction tails, or source-prefixed
  clause fragments into people, the repair must happen in
  `FieldAwareEntityResolver`, not in a per-word allowlist.
- `危局`, `临界`, `林峰压`, and `苏瑶并` are canonical counterexamples: they
  must not be treated as new people in `shot_title`, `scene_title`,
  `visual_description`, or `character_action` when their field role makes them
  situation/action fragments.
- The same strings must not become globally allowed people. `person=危局` and
  a true source-external person such as `李明` still hard fail when absent from
  the current source or accepted narrative frame.
- A source-prefixed fragment is valid only when the prefix resolves to a
  current source character or accepted role and the remaining tail is a bounded
  action, state, grammar, or conjunction fragment.
- Runner/certifier acceptance must still require non-empty rows, prompt
  boundary pass, visible-frame pass, `rows_match=true`, and no stale artifact.

## 4. SceneProfile Contract For All 21 Scene Types

Every scene type must be driven by one unified `SceneProfile`. Special-casing
only `热血战斗`, `场域追逐`, A/B samples, or any other hot path is forbidden.

Each profile must define:

- `scene_type_id`
- Chinese display name
- narrative expression mode
- space organization mode
- conflict/pressure mode
- rhythm progression mode
- camera/viewpoint tendency
- allowed strengthening
- forbidden drift
- KB oracle tags

Canonical profile set:

| scene_type_id | display | narrative expression | space organization | pressure mode | rhythm | camera/view | allowed strengthening | forbidden drift | KB oracle tags |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `hot_blood_battle` | 热血战斗 | confrontation, resolve, kinetic pressure | action-centered space | direct opponent pressure | fast beat escalation | impact cuts, low/high emphasis | stronger action causality | new weapons, injuries, ranks, armies | `combat_pressure`, `resolve_turn`, `action_density` |
| `ensemble_performance` | 群像表演 | multiple roles readable in one beat | role-layered staging | role conflict or coordination | alternating focus | group blocking, role handoff | clearer role functions | unnamed crowd inflation | `ensemble_roles`, `handoff`, `group_readability` |
| `emotional_dialogue` | 情绪对话 | emotion and relation turn | intimate spatial contrast | withheld information or relationship pressure | slower reaction beats | close framing, eyeline | clearer subtext | new backstory not in source | `emotion_turn`, `dialogue_pressure`, `subtext` |
| `encounter_performance` | 相遇表演 | first-contact recognition | crossing paths or threshold | curiosity, mistake, or tension | reveal then response | approach, reveal, reverse angle | clearer encounter cue | new identity reveal | `encounter`, `recognition`, `threshold` |
| `field_chase` | 场域追逐 | pursuit through space | path, obstacle, exit | pursuit, block, escape pressure | motion chain | tracking, obstacle reveal | clearer path and misread | new pursuer count, new route lore | `field_path`, `chase_pressure`, `spatial_cue` |
| `spectacle_showcase` | 奇观展示 | large visual wonder serving source facts | scale layers | awe or scale pressure | reveal and expansion | wide reveal, vertical scale | scale and atmosphere | new mythology/worldview | `spectacle`, `scale_layer`, `awe` |
| `daily_healing` | 日常治愈 | low-conflict repair or warmth | familiar everyday space | mild misunderstanding or care | soft beat closure | gentle follow, stable framing | emotional softness | erase source conflict or enemy into neighbor | `daily_repair`, `warmth`, `low_conflict` |
| `guoman_hot_blood_combat` | 国漫热血打斗 | national-animation combat emphasis | iconic action space | heroic pressure | high-energy turn | stylized impact | posture, resolve, rhythm | new factions, artifacts, ranks | `guoman_combat`, `iconic_pose`, `combat_pressure` |
| `guoman_ensemble_performance` | 国漫群像表演 | stylized group role interplay | layered group tableau | group alignment or split | role-by-role build | ensemble tableau | role readability | extra named roles | `guoman_ensemble`, `role_tableau`, `handoff` |
| `ink_wuxia_combat` | 水墨武打 | restrained martial rhythm | ink-like spatial negative space | duel pressure | pause then strike | brushlike motion, silhouette | martial rhythm | weapons/school lore not in source | `ink_motion`, `wuxia_duel`, `negative_space` |
| `eastern_spectacle` | 东方奇观 | eastern-scale visual ritual or wonder | vertical/depth layers | fate, place, or scale pressure | gradual reveal | wide/depth reveal | atmosphere and scale | invented mythology | `eastern_spectacle`, `ritual_scale`, `depth` |
| `xianxia_action` | 仙侠动作 | elevated action and momentum | height, distance, crossing | pursuit or duel pressure | leap/turn/landing | vertical motion | motion clarity | sect, spell, artifact invention | `xianxia_motion`, `vertical_action`, `duel` |
| `urban_fantasy` | 都市奇幻 | everyday city with uncanny pressure | city layers, signs, transit | hidden anomaly or chase | normal-to-strange turn | neon/reflection/reveal | urban atmosphere | new supernatural lore | `urban_anomaly`, `city_layer`, `uncanny` |
| `chinese_war_formation` | 国战军阵建立 | ordered military pressure without new facts | formation-like composition | organized pressure | setup then compression | wide formation view | order and pressure | army scale, ranks, armor, banners not sourced | `formation_pressure`, `order`, `war_scale_guard` |
| `weapon_highlight` | 武将兵器高光 | weapon focus only if source has weapon | object-person relation | readiness or threat | highlight then action | insert, glint, reveal | source weapon visibility | invented weapon/body detail | `weapon_highlight`, `object_focus`, `source_weapon_only` |
| `council_strategy` | 朝堂军帐权谋 | strategy tension from accepted facts | table/seat/power layout | political or tactical pressure | controlled exchange | table, map, eyeline | power relation | official titles, map lore, camps not sourced | `strategy_pressure`, `power_layout`, `no_new_rank` |
| `siege_defense` | 多军团攻城 | defense pressure when source supports it | wall/front/depth | siege compression | waves and response | wide to detail | siege pressure | multiple armies if source has one enemy | `siege_pressure`, `defense`, `scale_guard` |
| `slg_sandbox_view` | 沙盘战略视口 | strategic overview from source facts | map/sandbox abstraction | route, resource, or threat pressure | observe, decide, feedback | overhead, UI-like layers | strategic readability | treating UI/map labels as people | `sandbox_view`, `strategic_readability`, `ui_guard` |
| `slg_march_encirclement` | 行军轨迹合围 | movement-route pressure | route arcs and closure | encirclement or pursuit | path convergence | overhead tracking | route clarity | new troops, routes, numbers | `march_route`, `encirclement`, `path_guard` |
| `slg_city_growth` | 城建演进反馈 | construction/state feedback | city/resource layout | build pressure or consequence | state change feedback | isometric or UI feedback | state readability | new systems not sourced | `city_growth`, `feedback`, `state_change` |
| `slg_battle_report` | 战报 UI | report-style summary of accepted events | UI/report layers | result pressure | status then consequence | panel/readout view | report clarity | UI text as people, new battle facts | `battle_report`, `ui_report`, `result_guard` |

## 5. Target Duration Capacity Contract

`target_duration` controls narrative capacity. It is not a body explanation
field and must not be printed as a meta instruction.

Each duration profile defines:

- `beat_count`
- beginning/development/turn/closing capacity
- paragraph length
- action density
- suggested shot count
- single-shot duration limit

Fixed duration profiles:

| seconds | beat_count | narrative capacity | paragraph length | action density | suggested shot count | single-shot limit |
| --- | --- | --- | --- | --- | --- | --- |
| 5 | 1 | one clear action or one emotional turn | 1 compact paragraph | very low | 1 | max 5s |
| 10 | 2 | pressure cue plus immediate response | 1 short paragraph | low | 1-2 | max 10s |
| 15 | 3-4 | setup, pressure, response, closing beat | 1-2 short paragraphs | medium | 2 | max 10s, split allowed as `10 + 5` |
| 30 | 5-6 | fuller setup, obstacle, reaction, result | 2-3 paragraphs | medium-high | 3-4 | max 10-15s |
| 45 | 7-9 | multiple linked turns plus relationship or pressure shift | 3-4 paragraphs | high | 4-6 | max 10-15s |
| 60 | 9-12 | full multi-beat progression with clear emotional/relationship change | 4-6 paragraphs | high but readable | 5-8 | max 10-15s |

Future positive fixed durations:

- Normalize to the closest capacity band.
- Interpolate beat count and paragraph length.
- Preserve source facts before increasing density.
- Never require a new scene-pair special case.
- Must fail if only `duration_seconds` changes while body capacity and
  storyboard duration plan remain unchanged.

## 6. KB Oracle Contract

KB participates from the first `扩写故事` call. It continues through `改写剧本`
and storyboard generation.

KB may provide:

- structure reference
- writing-group/director-group rules
- rhythm and conflict progression
- negative drift constraints
- golden sample structure oracle
- scene profile tags
- expected prompt/row oracle summaries

KB must not provide or leak:

- raw `sample_text`
- raw KB rows
- sample people
- sample places
- sample props
- sample worldview
- `source_register`
- overlay JSON
- raw `prompt_body`
- API keys, tokens, secrets, raw env values

Positive evidence:

- `kb_rule_pack_ids_non_empty=true`
- `kb_snapshot_hash_valid=true`
- `kb_oracle_present=true`
- `kb_oracle_affects_structure=true`

Negative evidence:

- `raw_kb_rows_absent=true`
- `raw_sample_text_absent=true`
- `sample_people_absent=true`
- `sample_places_absent=true`
- `sample_props_absent=true`
- `sample_worldview_absent=true`
- `source_register_absent=true`
- `overlay_json_absent=true`
- `prompt_body_absent=true`

Structural-effect gate:

- KB influence must be visible as better setup, conflict progression, rhythm,
  negative drift control, and closing beat.
- KB influence must not appear as explanatory prose such as "according to KB",
  "knowledge base rule", "oracle", or raw trace wording.

## 7. VisibleFrame And Prompt Text Contract

`visual_description` must be derived from `VisibleFrame`.

Required `VisibleFrame` components:

- scene space
- character position
- current visible action
- shot scale
- camera movement
- foreground/midground/background or spatial layer
- visible motion trace

Forbidden `visual_description` substitutes:

- "scene anchor"
- "performance anchor"
- "motion anchor"
- "current visual event" as a literal placeholder
- strategy
- rhythm progression
- internal evidence chain
- KB rationale
- validator rationale

`prompt_text` must be layered and source-bound.

Required layers:

- shot objective
- picture
- action
- camera
- dialogue/voiceover
- constraints

Prompt boundary checks:

- Uses accepted facts, StoryFactFrame, scene profile, duration profile, and
  sanitized rule tags only.
- Does not become a person/source fact surface.
- Does not contain raw prompt body, raw KB row, raw sample text,
  `source_register`, overlay JSON, sample entity marker, source sample ID, or
  internal trace.

## 8. QA Acceptance Checklist

Targeted A/B/C/D must prove:

- `NarrativeStoryBody` is complete story prose.
- `NarrativeStoryBody` is first in the UI main editor.
- Confirmation first screen starts with `NarrativeStoryBody`.
- Same source plus different scene changes scene expression.
- Same source plus different duration changes narrative capacity.
- `危局` is not treated as a person.
- Unknown person in `person` hard fails.
- `shot_title`/`scene_title` may contain situation/title words.
- KB evidence is present and structural.
- Raw KB/sample/source-register/overlay/prompt data is absent.
- Storyboard rows derive from confirmed body and do not overwrite it.

`full16` inherits every targeted assertion and additionally requires all 16
source/scene/duration/script-goal combinations to carry fresh binding evidence.

`formal403` inherits targeted and `full16`, then applies the same field-aware
contract to 4 entry baselines, 378 fixed cases, and 21 long-text cases.

No targeted pass may be read as `full16`. No `full16` pass may be read as
`formal403`.

## 9. Deep Problem Spectrum

This contract is not a patch note for the current screenshots. It defines the
system boundary that prevents recurrence across source facts, narrative body,
scene expression, duration capacity, KB oracle, field semantics, provider
failover, UI state, artifact freshness, and source integrity.

### P0-1 Input Fact Layer

Problem family:

- User seed input, expanded story body, rewritten story body, and storyboard
  rows can be accidentally treated as one mutable source.
- Scene expression can be written back into facts.
- SupportNotes, KB summaries, strategy notes, or validator notes can become
  factual source material.
- Original characters, places, relationships, conflicts, and event order can be
  dropped during rewrite.

Contract boundary:

- `SeedSourceFrame`, `AcceptedNarrativeFrame`, rewrite output, and storyboard
  rows are separate source layers.
- `scene_type` expression is never a fact source.
- SupportNotes and KB summaries are support-only.
- Rewrite must preserve source people, locations, relationships, conflict, and
  event order unless the user explicitly changes the source.

### P0-2 Confirmation Body Layer

Problem family:

- "Confirm use" can mistakenly confirm raw input, support notes, rows,
  `prompt_text`, or KB summary instead of story prose.
- The body can degrade into action notes, strategy notes, or storyboard notes.
- Main body and support notes can become visually indistinguishable.
- Storyboard fields can later pollute the accepted narrative source.

Contract boundary:

- Confirmation upgrades only `NarrativeStoryBody` into
  `AcceptedNarrativeFrame`.
- Body must be complete story narration.
- Main body and support notes must be visually and structurally separable.
- Storyboard rows and row edits never write back to the narrative source.

### P0-3 Scene Expression Layer

Problem family:

- `scene_type` can become a factual source.
- Old scene expression can remain after switching scene type.
- Hot-path fixes can cover only `热血战斗`, `场域追逐`, or A/B samples.
- Scene words, situation words, and camera words can be globally misread as
  person names.

Contract boundary:

- `scene_type` is expression control only.
- Scene switch invalidates old expression.
- All 21 scene types use one `SceneProfile` shape.
- Field-aware entity resolution decides by field role, not by global token
  matching.

### P0-4 Duration Capacity Layer

Problem family:

- `target_duration` changes only a numeric field.
- Story prose and rows keep the same capacity.
- Total row duration is correct while story capacity or shot capacity is wrong.

Contract boundary:

- Duration maps to beat count, paragraph capacity, action density, and shot
  capacity.
- Fixed durations `5/10/15/30/45/60` and future positive fixed durations use
  the same resolver.
- `duration_sum_matches_target=true` is necessary but not sufficient.

### P0-5 KB Oracle Layer

Problem family:

- `kb_rule_pack_ids` and snapshot hash exist but KB does not shape structure.
- KB leaks raw sample text, raw KB rows, sample people/places/props/worldview,
  `source_register`, overlay JSON, or raw `prompt_body`.
- Golden sample material is mistaken for user fact source.

Contract boundary:

- `kb_oracle_affects_structure=true` is required.
- KB influence must be visible in structure, not disclosure.
- Golden samples are structure oracle only and never fact source.
- Leakage is hard fail.

### P0-6 Field Semantics Layer

Problem family:

- `person`, `shot_title`, `visual_description`, `character_action`,
  `prompt_text`, and `support_notes` are validated with one global semantic
  rule.
- `危局` is treated as person even when it is a title/situation word.
- Fragments such as `林峰压` or `苏瑶并` are promoted to new names.
- Abstract `visual_description` text is misclassified as person pollution
  instead of visible-frame failure.

Contract boundary:

- Each field has a `FieldRole`.
- Each token resolves to an `EntityKind` inside that field role.
- Action tails and conjunction tails are `action_fragment`.
- Abstract visual prose fails the visible-frame gate, not the person gate.

### P0-7 Provider Failover Layer

Problem family:

- Model switching can hide validator, grounding, row, prompt, KB, visual, or
  CDP failures.
- Local fallback, content fallback, and live provider failover can be merged.

Contract boundary:

- Provider failover handles only quota, model unavailable, or entitlement /
  permission HTTP 403.
- Validator and business-contract failures never trigger successful model
  failover.
- `provider_failover_used`, `fallback_used`, `local_candidate`, and
  `no_live_fallback` stay separate.

### P0-8 UI Display Layer

Problem family:

- Button labels do not match output semantics.
- Main text, confirmation, task candidate, rows, and edit dialogs read from
  different snapshots.
- Scene/duration/body changes leave old tasks and rows marked current.
- Row edits mutate story body.

Contract boundary:

- `扩写故事` and `改写剧本` have separate inputs and both produce
  `NarrativeStoryBody`.
- Every visible surface binds the same narrative body hash, scene type, target
  duration, and KB snapshot hash.
- Any source/scene/duration/body change stales candidates and rows.
- Row edits only edit rows.

### P0-9 Artifact Freshness Layer

Problem family:

- Runtime/UI/runner/certifier/matrix/KB mapping changes are tested with old
  release exe or old artifacts.
- Empty rows can still produce `rows_match=true`.
- `StopOnly` clean can be over-read as shell pass.
- Targeted evidence can be over-read as `full16` or `formal403`.

Contract boundary:

- Any relevant code/artifact input change stales old release exe, old CDP gate,
  and old targeted/full16/formal403 artifacts.
- Empty response/UI rows with `rows_match=true` are pseudo-success hard fail.
- `StopOnly` is cleanup evidence only.
- Gate layers cannot read upward.

### P0-10 Source Integrity Layer

Problem family:

- `runtime.rs` contains Chinese and contract strings that can be damaged by
  full-file mechanical replacement.
- Business gates can continue after encoding damage.

Contract boundary:

- UTF-8 readability and sentinel checks are preflight hard gates.
- Sentinels: `热血战斗`, `场域追逐`, `危局`, `林峰`, `苏瑶`.
- Full-file mechanical replacement across non-ASCII content is forbidden
  unless a separate source-integrity gate explicitly approves and verifies it.

## 10. Type Boundaries

The implementation contract must expose or internally preserve these type
boundaries. Names may map to Rust/TypeScript enums or equivalent structured
types, but stringly ad hoc status is not sufficient for gate evidence.

`FieldRole`

```text
person
title
visual
action
prompt
support
```

`EntityKind`

```text
source_character
source_location
situation_term
scene_term
camera_term
action_fragment
external_character
```

`SourceFrameKind`

```text
seed
accepted
rewrite_plan
storyboard_input
```

`SceneProfileId`

```text
hot_blood_battle
ensemble_performance
emotional_dialogue
encounter_performance
field_chase
spectacle_showcase
daily_healing
guoman_hot_blood_combat
guoman_ensemble_performance
ink_wuxia_combat
eastern_spectacle
xianxia_action
urban_fantasy
chinese_war_formation
weapon_highlight
council_strategy
siege_defense
slg_sandbox_view
slg_march_encirclement
slg_city_growth
slg_battle_report
```

`DurationCapacity`

```text
short
medium
long
custom
```

`KBOracleKind`

```text
structure
director_rule
writing_rule
negative_drift
sample_shape
```

`ProviderErrorKind`

```text
quota
model_unavailable
entitlement_403
timeout
network
unknown
```

`ValidationErrorKind`

```text
field_semantics_violation
source_drift
narrative_body_invalid
visible_frame_invalid
prompt_leakage
kb_leakage
pseudo_success
```

## 11. State Boundaries

The state machine must distinguish these booleans/enums. A later gate may
rename them only if the evidence remains one-to-one and documented.

```text
seed_source_locked
accepted_story_fact_source_present
rewrite_plan_created
narrative_body_created
narrative_body_confirmed
storyboard_input_created
provider_live_success
provider_failover_used
local_fallback_used
validator_passed
repair_applied
repair_within_boundary
evidence_fresh
artifact_stale
```

State transition rules:

- `seed_source_locked=true` before expansion or rewrite starts.
- `accepted_story_fact_source_present=true` is required before `改写剧本`.
- `rewrite_plan_created=true` before `NarrativeStoryBody` generation.
- `narrative_body_confirmed=true` before creating current storyboard tasks.
- `storyboard_input_created=true` before generating rows.
- `provider_live_success=true` cannot coexist with `local_fallback_used=true`.
- `repair_applied=true` requires `repair_within_boundary=true` to pass.
- `artifact_stale=true` forces `evidence_fresh=false`.

## 12. Error Boundaries

Allowed provider-level escalation:

- provider quota can trigger model failover.
- entitlement/permission HTTP 403 can trigger model failover.
- model unavailable can trigger model failover.

Forbidden provider-level escalation:

- validator failed cannot trigger successful model failover.
- source grounding failed cannot trigger successful model failover.
- rows mismatch cannot trigger successful model failover.
- prompt boundary failed cannot trigger successful model failover.
- KB leakage cannot trigger successful model failover.
- CDP failure cannot trigger successful model failover.

Hard failures:

- KB leakage is `kb_leakage` hard fail.
- `rows_match=true` with `response_rows=[]` or `ui_rows=[]` is
  `pseudo_success` hard fail.
- abstract `visual_description` is `visible_frame_invalid` hard fail.
- `prompt_text` containing `source_register`, overlay JSON, raw KB, raw sample,
  or raw prompt body is `prompt_leakage` hard fail.
- damaged `runtime.rs` encoding is source-integrity hard fail.
- stale release exe is evidence-freshness hard fail.

## 13. UI Visible State Contract

`扩写故事`

```text
SeedSourceFrame
+ current scene_type
+ target_duration
+ KB oracle
-> NarrativeStoryBody
```

`改写剧本`

```text
AcceptedNarrativeFrame
+ current scene_type
+ target_duration
+ KB oracle
-> NarrativeStoryBody
```

UI rules:

- Main text area first block must be `NarrativeStoryBody`.
- SupportNotes must be after the body, in a collapsed area, or in a secondary
  region.
- If no `AcceptedNarrativeFrame` exists, `改写剧本` is disabled or must show a
  prompt to confirm the story first.
- Confirmation first screen must match the main text body.
- Clicking confirm use upgrades only `NarrativeStoryBody` into
  `AcceptedNarrativeFrame`.
- After `scene_type` changes, candidate tasks are stale, storyboard rows are
  stale, generation is disabled, and the user must rewrite/confirm/create task.
- After `target_duration` changes, candidate tasks are stale, storyboard rows
  are stale, generation is disabled, and the user must rewrite/confirm/create
  task.
- After body text changes, candidate tasks are stale, storyboard rows are stale,
  generation is disabled, and the user must confirm/create task again.
- Candidate tasks must bind: `source_type`, `narrative_body_hash`,
  `scene_type`, `target_duration`, `kb_snapshot_hash`, and `sync_status`.
- Story-material edit dialog saves only the body area. SupportNotes, KB
  summary, and boundary notes are read-only or folded.
- Storyboard row editing changes only the row. It does not change
  `NarrativeStoryBody` or `AcceptedNarrativeFrame`.

## 14. Counterexamples And Failure Reasons

1. Body is strategy explanation, not story.
   Failure: `narrative_body_invalid`.

2. Confirmation draft and main text body differ.
   Failure: `accepted_frame_mismatch`.

3. Scene changes, but old candidate task can still be queued as current.
   Failure: `stale_candidate_task`.

4. Duration changes, but old rows still display as current.
   Failure: `stale_storyboard_rows`.

5. Story-material edit dialog contains prompt text or storyboard instructions.
   Failure: `support_surface_polluted_body`.

6. Raw sample text or sample people/places/props leak from KB/golden sample.
   Failure: `kb_leakage`.

7. `response_rows=[]`, `ui_rows=[]`, and `rows_match=true`.
   Failure: `pseudo_success`.

8. Model switch hides validator failure.
   Failure: `provider_failover_not_allowed`.

9. `visual_description` is abstract strategy, not visible frame.
   Failure: `visible_frame_invalid`.

10. Editing a storyboard row changes the story body.
    Failure: `row_edit_backflow`.

11. `runtime.rs` encoding is damaged but business gate continues.
    Failure: `source_integrity_hard_fail`.

12. Release exe is stale but targeted/full16/formal403 continues.
    Failure: `evidence_freshness_hard_fail`.
