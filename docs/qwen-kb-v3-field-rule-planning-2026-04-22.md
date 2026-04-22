# Qwen KB V3 Field Rule Planning 2026-04-22

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor before this memo: `5ea18bd` (`docs: accept kb v0.2 snapshot import readiness`)
- route: `RC_READY` plus controlled v0.2 readiness planning
- package type: docs-only orchestration contract planning

This planning package does not open product implementation. It does not modify
Rust DTOs, validators, exporter behavior, workbook shape, IPC, desktop, intake,
Qwen, Seedance, or `hope-kb`.

## Problem Statement

Hope now has three separate planning assets:

- KB prompt/stage templates and runtime-consume contracts.
- V3/V4 field overlay constraints for Qwen-generated structured shot content.
- KB v0.2 golden sample snapshot assets for future few-shot, validator, and
  repair evidence.

The missing layer is a canonical rule contract that says, for each story
generation stage:

- which fields may be written
- which fields must not be written yet
- which KB sources can guide the stage
- which golden sample rows may be retrieved
- which V3 fields are in scope
- which validator or repair gate later owns the result

Without this layer, Qwen, KB, V3, validator, and exporter can each be correct
in isolation while the overall story-to-output flow remains underspecified.

## Decision

`QWEN_KB_V3_FIELD_RULE_PLANNING_ACCEPTED_DOCS_ONLY`

Main control accepts that Hope needs a future `Generation Field Rule Contract`
before any Qwen retrieval or V3 product implementation begins.

This package defines the planning matrix only. It is not an implementation
authorization.

## Current Evidence

Existing KB prompt stages include:

- `synopsis_to_story`
- `story_to_outline`
- `story_to_screenplay`
- `screenplay_to_segments`
- `segment_to_cuts`
- `cut_to_layout`
- `layout_to_render`
- `dialogue_refine`
- `repair_pass`
- director variant stages
- repair stages

Existing V3 core fields:

- `visual_scene_core`
- `motion_performance_core`
- `camera_directing_core`
- `audio_directing_core`
- `continuity_lock_core`
- `reference_control_core`

Existing KB v0.2 golden-sample coverage:

- 40 rows total
- 5 covered V3 cores, 8 rows per core
- covered cores: visual, motion/performance, camera, audio, continuity
- `reference_control_core` remains uncovered
- positive few-shot candidates must exclude negative or unusable rows

Existing Hope runtime posture:

- current storyboard runtime keeps KB taxonomy as metadata when explicit runtime
  inputs exist
- current prompt layers are runtime-authoritative and are not rewritten from KB
  metadata
- current v0.1 workbook, exporter, IPC, validators, and DTOs remain frozen

## Proposed Contract Name

Use this name for the future contract package:

```text
Generation Field Rule Contract
```

Short label:

```text
Qwen-KB-V3 Field Rule Contract
```

## Planning Matrix

| phase | upstream input | fields allowed to write | fields not allowed yet | allowed KB sources | golden sample use | Qwen template family | validator / repair owner |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `input_preflight` | user synopsis, target length, style intent, hard locks, optional assets | input envelope, project intent, duration policy, hard-lock inventory | V3 core fields, render prompts, exporter fields | source policy, runtime consume contracts | none | none / preflight only | future input completeness validator |
| `synopsis_to_story` | input envelope | story premise, world summary, protagonist goal, conflict source, character arc, story outline, story core intent, story emotional baseline | RenderSegment, Cut, PromptPackage, V3 shot cores, Seedance payload | `prompt_01`, story structure templates, manga structure rules, dialogue style rules as background | no direct few-shot unless future gate defines story-level positive rows | `synopsis_to_story` | future story structure validator |
| `story_to_outline` | Story output | beat list, beat goal, conflict/turn/result per beat, emotional progression | Cut, layout prompt, render prompt, V3 shot cores | `prompt_02`, story structure templates, character arc patterns | no direct few-shot unless future gate defines outline-level positive rows | `story_to_outline` | future beat coverage validator |
| `story_to_screenplay` | Story and beats | NarrativeScene drafts, DialogueTurn drafts, scene goal, conflict, result, characters, emotional state | RenderSegment crossing rules, prompt package, Seedance adapter fields | `prompt_03`, dialogue style rules, scene taxonomy as classification support | optional future retrieval by genre/scene only; no raw golden row output | `story_to_screenplay` | future screenplay structure validator |
| `screenplay_to_segments` | NarrativeScene and beats | RenderSegment plan, target duration, segment boundaries, narrative_scene_id, segment purpose | crossing NarrativeScene boundary, V3 core prose, render prompt, exporter rows | `prompt_04`, scene taxonomy, runtime consume contracts | no golden sample positive prompt context | `screenplay_to_segments` | segment duration and boundary validator |
| `segment_to_cuts` | RenderSegment, committee selection, hard locks, continuity constraints | Cut plan, shot description, dialogue, transition_out, continuity refs, cut reason | layout prompt, render prompt, compiled payload | `prompt_05`, committee handoff rules, committee style merge rules, continuity rules, scene taxonomy | future retrieval can select examples by scene_tag/shot_type after gate | `segment_to_cuts` | continuity, handoff, and cut traceability validators |
| `v3_core_generation` | Cut plus segment context, hard locks, selected KB rules | `visual_scene_core`, `motion_performance_core`, `camera_directing_core`, `audio_directing_core`, `continuity_lock_core`; optional story/cut linkage metadata | `reference_control_core` unless future coverage gate exists, Seedance payload, exporter workbook columns | golden sample library, field coverage rules, vocabulary, continuity rules, director/committee rules | positive few-shot only when `fewshot.eligible=true`; exclude negative/unusable rows | future V3 core writer templates, not yet implemented | golden coverage, empty-word, negative-sample validators |
| `reference_control_planning` | asset registry or reference inputs, when present | reference-control planning notes only | authoritative `reference_control_core` output until KB coverage or asset registry gate exists | source register and future asset registry only | current v0.2 golden samples cannot fill this gap | future reference-control template | future asset/reference validator |
| `cut_to_layout` | Cut and V3 visual/camera context | layout prompt, composition, viewpoint, spatial/light direction | character appearance detail, render style overreach, Seedance payload | `prompt_06`, camera terms, visual vocabulary, scene tokens | use examples only as internal constraints after retrieval gate | `cut_to_layout` | prompt quality and layout/render separation validator |
| `layout_to_render` | layout prompt, characters, hard locks, selected V3 cores | render prompt, visible action detail, hard-lock preservation, negative prompt where contract allows | changing cut intent, changing layout, inventing assets, local default refill | `prompt_07`, director profiles, failure patterns, prompt templates | positive few-shot may guide prose density; negative rows excluded from positive context | `layout_to_render` | prompt quality, hard-lock, style unity validators |
| `validation` | generated structured output and prompt package | validation report, blocking failures, missing items, coverage findings | rewriting source payload directly | failure patterns, runtime consume contracts, golden failure mapping | negative samples can be validator fixtures, not prompt context | validator templates only | Hope validator owner |
| `repair` | validation findings, current payload, target schema | minimal field repair suggestions and repair drafts scoped to failed fields | rewriting unrelated story/segments/cuts/prompts | `prompt_09`, `prompt_15`, `prompt_16`, `prompt_17`, `prompt_18`, repair mappings | gap and negative rows can provide repair evidence after gate | repair template family | repair contract owner |
| `export_adapter` | validated structured output and prompt package | export/debug metadata, adapter compile notes after future gate | changing v0.1 workbook or exporter behavior in this package | export templates, runtime consume contracts | only provenance/debug metadata after exporter gate | export summary only | exporter/debug metadata owner |

## Core Rule Families

### 1. Stage Ownership

Each generation phase may write only its owned fields. A later phase may read
earlier fields, but it must not silently rewrite them.

Examples:

- `story_to_screenplay` can read story beats but must not define RenderSegment
  boundaries.
- `screenplay_to_segments` can create RenderSegment boundaries but must not
  write render prompts.
- `v3_core_generation` can write V3 core prose but must not compile Seedance
  payload.
- `layout_to_render` can write render prompt text but must not change the cut's
  narrative purpose.

### 2. KB Source Selection

KB assets should be routed by stage:

- story stages use story structure, manga structure, dialogue style, and
  high-level prompt templates
- segment and cut stages use scene taxonomy, committee rules, continuity rules,
  and runtime-consume contracts
- V3 core stages use golden samples, field coverage rules, vocabulary, and
  continuity rules
- validator/repair stages use failure mappings, repair mappings, degraded-input
  examples, and repair prompt templates

### 3. Golden Sample Retrieval

Golden samples may become future Qwen retrieval evidence only after the intake /
Qwen retrieval boundary gate.

Planning rules:

- positive few-shot context may use only rows with positive eligibility
- rows marked negative or unusable must be excluded from positive prompt context
- negative rows may be used for validator fixtures and repair planning
- retrieval filters may include core, tier, genre tags, scene tags, shot type,
  and internal director voice
- director voice remains internal retrieval metadata and must not become final
  imitation wording
- source provenance must remain traceable when a row influences output

### 4. V3 Core Coverage

The current KB v0.2 golden samples cover five V3 cores:

- visual scene
- motion/performance
- camera directing
- audio directing
- continuity lock

`reference_control_core` remains uncovered. It must not be generated from the
current golden sample package as if coverage existed.

### 5. Validator And Repair Boundaries

Validators should judge generated fields against rule families and evidence,
not treat golden samples as literal pass/fail answers.

Repair should:

- repair only fields named by the validator finding
- preserve passed fields
- preserve source provenance
- avoid rewriting unrelated stages
- keep negative-boundary markers visible

## Required Future Artifacts

The next docs-only package should produce a frozen candidate with these
artifacts:

- `GenerationFieldRuleContract`
- `GenerationPhase`
- `FieldOwnershipRule`
- `KbSourceSelectionRule`
- `GoldenSampleRetrievalRule`
- `V3CoreCoverageRule`
- `ValidationGateRule`
- `RepairScopeRule`

Implementation must wait for a later explicit gate.

## Future Gate Order

Do not jump directly to Hope implementation. Use this order:

1. `Generation Field Rule Contract Candidate`
   Freeze the matrix, field ownership, KB source routing, golden-sample
   retrieval rules, and validator/repair ownership.
2. `KB support update if needed`
   Add only missing KB-side rule metadata, especially if story-stage or
   `reference_control_core` samples are required.
3. `V3 proposal refresh if needed`
   Update the V3 freeze candidate only after KB changes are accepted by main
   control.
4. `Hope validator evidence contract`
   Define DTO and validator behavior for coverage, empty-word, negative-sample,
   unusable-fewshot, and provenance-gap signals.
5. `Intake / Qwen retrieval boundary`
   Define retrieval, prompt context assembly, and negative exclusion before live
   model calls.
6. `Writer / orchestrator implementation`
   Implement stage sequencing only after the above contracts are accepted.
7. `Exporter/debug metadata and desktop readonly provenance`
   Open only after validator and retrieval boundaries are stable.

## Explicit Prohibitions

This planning package does not authorize:

- Rust DTO changes
- validator implementation
- writer or orchestrator implementation
- exporter behavior changes
- workbook changes
- IPC changes
- desktop changes
- intake changes
- Qwen calls
- Seedance calls
- `hope-kb` edits
- V3 branch edits
- importing golden samples into v0.1 assets
- using negative samples as positive prompt examples
- inventing `reference_control_core` coverage

## Completion State

`QWEN_KB_V3_FIELD_RULE_PLANNING_PACKAGE_COMPLETE`

This package completes the first docs-only planning pass for the missing
stage-field-rule orchestration layer. The next action, if the user continues,
is to freeze a dedicated `Generation Field Rule Contract Candidate` before any
implementation package begins.

关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。
