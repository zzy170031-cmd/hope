# Hope Anime Script Confirmation Protocol

Status: docs-only protocol freeze candidate
Scope: rewrite / expansion confirmation editor, accepted snapshot,
StoryFactFrame mapping, PromptPackage handoff, pre-403 QA gate

## Goal

Freeze the upstream contract for the anime script confirmation text used after
rewrite or expansion and before storyboard prompt packaging.

The confirmation text is a fact-locking and handoff surface. It must help Hope
produce AI image prompts, AI video prompts, and human editing instructions, but
Hope itself does not directly generate images or videos in this protocol.

This protocol binds the confirmation editor to the internal accepted snapshot,
StoryFactFrame, duration plan, and PromptPackage surfaces so downstream
storyboard rows are generated from the current accepted source, scene type, and
duration instead of stale UI state.

## Non-Goals

This protocol does not:

- change existing product fields.
- add a complex UI form.
- redesign the storyboard table, export format, or prompt fields.
- move character portrait design into the confirmation text box.
- expose rows, `prompt_text`, hashes, validators, traces, internal schemas, raw
  KB rows, source registers, overlay JSON, secrets, tokens, or raw env values in
  the confirmation text box.
- open 403-case work, packaging, release shell runs, or matrix execution.

## Product Boundary

Hope remains a bounded content packaging system:

```text
user source
-> rewrite / expansion
-> user confirms the rewritten script text
-> accepted snapshot
-> StoryFactFrame
-> duration plan
-> storyboard prompt package
-> Excel-ready / production handoff structure
```

The confirmation text box is the user-visible checkpoint for accepted narrative
and performance anchors. It is not a database editor and not a raw prompt
debugging surface.

## Confirmation Text Contract

The confirmation text may contain plain script prose plus short, human-readable
anchors that are derived from the user source, selected scene type, and selected
duration.

Allowed confirmation anchors:

- 人物名称/代号
- 角色功能
- 人物关系
- 角色表演锚
- 基础场景描述
- 复杂场景描述
- 视频运动锚
- 目标时长/节奏落点
- 禁止新增

These anchors may be rendered as simple text sections inside the existing
confirmation editor or as a lightweight summary near it. They must not become a
new complex form or replace the existing storyboard table/export/prompt fields.

The confirmed body remains the source of `prompt_text` packaging, but the
visible confirmation text must not display `prompt_text` itself as an internal
field.

## Character Boundary

The character portion of the confirmation text may only describe:

- character name or code
- role function
- relationship
- action
- expression
- micro-action
- tone

It must not become a character portrait or appearance setting surface. It must
not invent costume, hair, eye color, age, body type, wound, weapon, rank,
species, faction, or other source-external identity details. If a visual
character fact is explicitly present in the source and needed for continuity, it
belongs to the accepted story facts and downstream visual packaging, not to a
free character portrait design field.

## Scene Boundary

Scene description may serve AI image prompting, AI video prompting, storyboard
planning, and human editing, but only as an expression of accepted source facts.

Allowed scene description:

- location or spatial facts present in the source.
- visible action and pressure relationships present in the source.
- atmosphere, rhythm, or scene-type expression that does not change facts.
- shot-friendly visual wording for source-bound action.
- scene complexity that comes from source complexity, not invention.

Forbidden scene drift:

- source-external setting.
- source-external props.
- source-external characters.
- source-external headcount.
- source-external wounds.
- source-external weapons.
- source-external worldbuilding, rank, army scale, or object state.

If scene-type expression conflicts with accepted facts, accepted facts win.

## Accepted Snapshot Binding

The "confirm use" action freezes an `accepted_rewrite_snapshot`. This snapshot
is internal and may be exposed only through sanitized QA evidence, not in the
confirmation text box.

Minimum binding intent:

- current case identity
- source text hash
- accepted rewrite hash
- selected scene type and scene label
- selected duration and target duration mode
- accepted confirmation body
- accepted anchor map
- StoryFactFrame hash
- bounded KB rule pack identity when used

Storyboard generation must use the accepted snapshot. It must not read old
textarea content, old task text, old trace state, old scene state, or old
duration state after confirmation.

## StoryFactFrame Mapping

StoryFactFrame extracts facts from the current user source and accepted
confirmation text. It is a constraint frame, not a creative expansion source.

Required StoryFactFrame responsibilities:

- preserve explicit characters, relationships, locations, actions, conflict
  pressure, event order, and must-keep facts.
- record forbidden additions that would be tempting for the scene type.
- distinguish short input from complex input by fullness of fields, not by
  inventing richer facts.
- preserve selected scene type and duration as binding context.
- keep KB guidance summary-only and unable to override user facts.

Short input fills the same conceptual frame with fewer facts. Complex input may
fill relationship tables, event chains, scene segments, pressure lines, and
forbidden additions, but must use the same no-invention rule.

## PromptPackage Mapping

Existing product field names remain frozen. This protocol maps confirmation
anchors into those fields without renaming or replacing them.

| Confirmation / internal source | Existing field | Rule |
| --- | --- | --- |
| 人物名称/代号 | `person` | Use only source-bound names, codes, or role labels. Do not invent portrait details. |
| 角色表演锚 | `character_action` | Convert action, expression, micro-action, and tone into visible performance instructions. |
| 基础/复杂场景描述 | `visual_description` | Package source-bound location, spatial pressure, visible action, and scene expression. |
| 视频运动锚 | `camera_movement` | Describe camera movement only when it clarifies accepted action, emotion, or scale. |
| 确认稿正文 + 锚点 | `prompt_text` | Produce clean downstream packaging from accepted facts and anchors; do not let packaging rewrite facts. |
| 目标时长/节奏落点 | duration plan | Convert selected duration and rhythm beats into validator-checkable timing and segment intent. |

`prompt_text` is packaging, not the source of truth. If `prompt_text` would
change accepted facts, regenerate it from the accepted snapshot and
StoryFactFrame.

## Duration And Rhythm Rules

The selected target duration is part of the accepted binding. Duration planning
may use rhythm anchors such as hook, information peak, action beat, reaction
beat, or suspense landing, but these anchors must remain tied to accepted facts.

The manga structure seed rules may guide timing as bounded craft advice:

- `manga_rule_01`: early hook.
- `manga_rule_02`: emotional or information peak density.
- `manga_rule_03`: suspense ending.
- `manga_rule_04`: character function legibility.
- `manga_rule_05`: short readable dialogue.

These rules are advisory. They must not create new characters, props, wounds,
weapons, facts, or plot events.

## KB Consumption Boundary

KB may contribute only bounded summaries, selected rule IDs, and compact
handoff notes. Runtime consumption must remain snapshot-bound and summary-only.

Allowed KB-facing payload concepts:

- snapshot version or hash
- selected sample IDs
- selected KB rule IDs
- fallback reason code
- KB context summary
- payload byte accounting

Forbidden KB exposure:

- raw KB rows
- raw prompt body
- source registry values
- overlay JSON
- secrets, tokens, authorization headers, or raw env
- full source passages copied into the confirmation text box

KB guidance is below user facts, continuity state, finalized storyboard facts,
scene expression, director scheduling, shot language, and final prompt
packaging priority rules. If KB guidance conflicts with accepted facts, drop
the KB guidance.

## Validation And QA Evidence

The confirmation protocol is accepted only when runtime and QA can prove that
the current accepted snapshot drives downstream generation.

Sanitized evidence may include:

- current case identity
- source text hash
- accepted rewrite hash
- task script hash
- StoryFactFrame hash
- source profile
- scene type
- duration seconds
- duration plan hash
- storyboard rows hash
- must-keep facts
- missing source facts
- forbidden facts
- forbidden fact hits
- stale binding detected
- KB rule pack IDs
- KB snapshot hash

Sanitized evidence must not expose raw prompt bodies, raw KB rows, source
registers, overlay JSON, secrets, tokens, raw env, or internal schemas in the
confirmation text box.

Passing `rows_match=true` is not enough. A pass also requires
`stale_binding_detected=false`, no missing required source facts, no forbidden
fact hits, visible provider/model gate integrity when live QA is in scope, and
explicit repair/fallback reporting when repair or fallback occurs.

## Pre-403 Cross-Drift Smoke Gate

Before entering 403-case, add a cross-drift smoke gate after the `qwen-max`
storyboard grounding fix, four-group trace gate, scene-type rewrite gate,
StoryFactFrame binding gate, and anime script confirmation protocol gate.

The smoke gate must prove that different source, scene, and duration bindings
produce distinct accepted snapshots and downstream package evidence when they
should, while preserving stable evidence when the accepted input is unchanged.

Required coverage:

- same script, different duration, same scene.
- same script, same duration, different scene.
- same script, same scene, different duration.
- same duration, different script, same scene.
- same duration, same scene, different script.

Each case must record:

- source profile
- scene type
- duration
- accepted text hash
- StoryFactFrame hash
- duration plan hash
- storyboard rows hash
- rows match result
- sanitized binding evidence
- repair or fallback state
- layout bounds
- StopOnly cleanup result

Acceptance requirements:

- changed source changes source text hash, accepted rewrite hash,
  StoryFactFrame hash, and storyboard package evidence.
- changed scene changes scene binding and scene-expression anchors without
  source-external additions.
- changed duration changes duration plan evidence and timing segmentation.
- unchanged accepted source/scene/duration does not drift because of stale UI
  state.
- every run reports `stale_binding_detected=false`.
- every run reports no missing required source facts and no forbidden fact hits.
- fallback or repair, if present, is explicit and cannot be counted as a live
  qwen pass unless the validator accepts the repaired result.
- `StopOnly` cleanup completes after each group.

403-case must remain closed until this cross-drift smoke gate is green or the
controller explicitly accepts a degraded gate with recorded risk.

## Red Lines

- Do not use the confirmation text box as a hidden schema editor.
- Do not surface hashes, validators, traces, raw KB, rows, `prompt_text`, or raw
  prompt bodies in the confirmation text box.
- Do not invent source-external facts to make image/video prompts richer.
- Do not let character anchors become portrait design.
- Do not let scene-type expression override user facts.
- Do not silently switch provider/model, run hidden repair, or count fallback as
  a live pass.
- Do not enter 403-case before the `qwen-max` storyboard grounding fix,
  four-group trace, scene-type rewrite, StoryFactFrame binding, anime script
  confirmation protocol, and cross-drift smoke gates are complete.

## Implementation Readiness Boundary

This document freezes the target protocol candidate only. It does not claim the
current implementation already enforces the accepted snapshot, StoryFactFrame,
PromptPackage mapping, or cross-drift smoke gate. Implementation, runner, test,
release-shell, 403-case, packaging, and commit work must be opened by separate
bounded instructions after this docs-only protocol freeze is accepted.

## Freeze Candidate Summary

This protocol freezes the docs-only target behavior for anime script
confirmation:

- confirmation text is a fact-locking narrative and anchor surface.
- accepted snapshot is the internal binding source.
- StoryFactFrame preserves facts and forbids drift.
- existing product fields remain unchanged.
- PromptPackage maps from accepted body and anchors into `person`,
  `character_action`, `visual_description`, `camera_movement`, `prompt_text`,
  and duration plan.
- Hope outputs production handoff material for AI image, AI video, and human
  editing workflows without directly generating images or videos.
- cross-drift smoke becomes a required pre-403 acceptance gate.
