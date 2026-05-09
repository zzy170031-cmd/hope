# Hope Scene Type Rewrite Contract

Status: executable contract for desktop targeted/full16/formal403 gates.

Scope: `扩写故事`, `改写剧本`, confirmation editor, accepted snapshot,
StoryFactFrame, storyboard generation, KB/golden structure oracle, and
UI-driven QA evidence.

## Product Pipeline

All 21 desktop scene types use the same rewrite pipeline:

```text
current accepted story fact source
+ target scene_type
+ target duration
+ KB scene rule pack
= narrative story body
```

`扩写故事` starts from the seed source facts. `改写剧本` starts from the current
accepted story fact source. Both stages must use the selected scene type,
selected target duration, and KB/golden structure oracle before producing user
visible story text.

## Scene Type Boundary

`scene_type` is an expression control condition, not a fact source.

It may change:

- narrative angle
- action density
- emotional temperature
- spatial organization
- rhythm capacity
- director/camera tendency
- scene-specific expression mechanism

It must not change:

- characters
- location or spatial facts
- core event
- conflict relationship
- event order
- current accepted story fact source

Switching scenes rewrites expression. It does not authorize new people, props,
places, worldbuilding, injuries, weapons, rank, army scale, or other source
external facts.

## Target Duration Boundary

`target_duration` is a narrative capacity control condition, not a sentence to
print into the story body.

Every target duration exposed by the product must change story capacity. The
current fixed desktop/formal set is `5`, `10`, `15`, `30`, `45`, and `60`
seconds, but this contract is not limited to those literal values. Any current
or future positive fixed duration must be normalized into narrative-capacity
bands before text generation. Long-text auto mode uses an estimated duration
and must still follow the same capacity rule. Future fixed durations must not
require scene-pair special cases.

- 5s: one strong action or one turn.
- 10s: one compact pressure beat plus an immediate decision.
- 15s: setup, pressure, response, and one closing beat.
- 30s: clearer space progression, obstacle, reaction, and result.
- 45s: multiple linked turns with one stronger relationship or pressure shift.
- 60s: fuller multi-beat progression and a clearer emotional or relational
  change.

The body must not merely mention the duration value. It must adapt sentence
count, event density, obstacle count, reaction beats, and closing beat to the
selected duration. Tests and gates must not limit this rule to 15/30/60 only,
and must fail if only a `duration_seconds` field changes while the visible
story prose remains the same capacity.

## Narrative Story Body Contract

`expanded_script_text`, `acceptedConfirmationBody`, and the rewrite main textarea
must contain only a narrative story body.

A valid narrative story body includes:

- characters
- situation
- motivation
- causality
- conflict progression
- emotional or relationship change
- closing beat

The story body must read like continuous story prose. It is not allowed to be:

- action choreography
- scene blocking notes
- storyboard breakdown
- strategy explanation
- internal analysis
- prompt-like text
- KB trace, hash, `source_register`, or overlay JSON

Hard-fail body terms include internal fields and meta wording such as
`prompt_text`, `duration_seconds`, `scene_type`, `target_duration_seconds`,
`kb_context_summary`, `selected_sample_ids`, `selected_kb_rules`,
`retrieval_trace`, `source_register`, `overlay_json`, `hash`, `oracle`, `镜头`,
`画面描述`, `角色动作`, `景别`, `运镜`, `动作设计`, `调度`, `节奏策略`,
`场景策略`, `结构参考`, `表达焦点`, `动作密度`, `镜头容量`, `主体为`,
`重点落在`, `用于`, `服务于`, and any raw prompt or validator trace.

## KB / Golden Structure Oracle Boundary

KB participates from the first `扩写故事` call and continues through `改写剧本`
and storyboard generation.

KB may provide:

- scene expression patterns
- writing-group structure reference
- director-group staging reference
- rhythm and conflict progression
- shot capacity
- negative drift guards
- golden sample structure oracle

KB must not provide or leak:

- raw `sample_text`
- raw KB rows
- sample characters, locations, props, or worldview
- `source_register`
- overlay JSON
- raw `prompt_body`
- API keys, tokens, secrets, or raw env values

KB is a core auxiliary craft reference. It is not an additional fact source and
not raw prose inserted into prompts or user-visible text.

For story-body generation, KB must affect structure, not wording disclosure.
The visible body should show stronger situation setup, conflict progression,
emotional turn, and closing beat because KB rule packs were applied. It must
not contain phrases like "根据 KB", "知识库规则", "结构参考", or other explanatory
language that exposes the auxiliary layer to the user as prose.

## Confirmation Boundary

The user confirms a narrative story body.

After confirmation, that body becomes the current accepted story fact source for
later rewrite. The next rewrite must extract stable atomic facts from it:
characters, places, events, relationships, conflict, pressure, and event order.
Old scene expression and old duration strategy must not become permanent facts.

## Storyboard Boundary

`visual_description`, `character_action`, `camera_movement`, and user-visible
`prompt_text` are derived from the confirmed narrative body, StoryFactFrame,
current scene type, current duration, and KB director rules.

Storyboard fields must not flow backward into the story body. `prompt_text` is
not a story source.

## No Special-Case Implementation

The implementation must not hard-code only:

- `热血战斗`
- `场域追逐`
- A/B targeted samples

Those are QA samples, not product boundaries. All 21 scene types must be driven
by a scene rule pack and the same story-body gate.

## QA Gate Inheritance

Targeted UI-driven checks may sample strong-difference paths, but they must
prove the shared contract:

- body is narrative story prose
- scene type changes expression
- duration changes capacity
- KB oracle participates without raw leakage
- desktop release UI shows the same narrative body in the main editor,
  confirmation dialog, task candidate, and generated rows
- accepted snapshot, StoryFactFrame, task, generation request, and UI rows share
  the same binding

Fresh full16 inherits this story-body gate for all 16 cases. Formal 403 inherits
it for 4 entry baseline, 378 fixed matrix, and 21 long-text matrix cases.

No targeted pass may be reported as full16 pass. No full16 pass may be reported
as formal403 pass.

## 2026-05-08 System Boundary Inheritance

This rewrite contract now inherits the executable boundaries in:

```text
docs/hope-field-aware-story-contract.md
docs/hope-provider-failover-contract.md
docs/hope-qa-evidence-freshness-contract.md
```

`扩写故事` and `改写剧本` must preserve the stage order:

```text
SeedSourceFrame
-> AcceptedNarrativeFrame
-> SceneRewritePlan
-> NarrativeStoryBody
-> SupportNotes
-> StoryboardInputFrame
```

The main visible story body must be complete narrative prose and must be the
first block in the main editor and confirmation first screen. Support notes,
KB summaries, trace summaries, and boundary explanations belong only after the
body or in folded/support surfaces.

Scene switching and duration switching must create a new `SceneRewritePlan`
from the current accepted source, current `scene_type`, current
`target_duration`, and KB oracle. Old scene expression and old duration pacing
are not accepted facts and must not survive into the new body as stale residue.

Field validation must be field-aware. Situation/title words such as `危局` are
allowed in title or visible-scene fields when context supports them, but are a
hard failure in `person`. The implementation direction is
`FieldAwareEntityResolver`, not larger global person-token wordlists.
