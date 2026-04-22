# Generation Field Rule Contract Candidate 2026-04-22

## Route

This is a docs-only contract-freeze candidate for the Hope main control thread.

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor before this memo: `94e3933` (`docs: plan qwen kb v3 field rules`)
- route: `RC_READY` plus controlled v0.2 readiness planning
- package type: docs-only generation field rule contract candidate
- thread label: `Hope主线-GenerationFieldRuleContract【候选冻结中】`

This candidate does not open product implementation. It does not modify Rust
DTOs, validators, exporter behavior, workbook shape, IPC, desktop, intake,
Qwen, Seedance, V3 proposal branches, or `hope-kb`.

## Reviewed Inputs

Hope-side inputs:

- `docs/qwen-kb-v3-field-rule-planning-2026-04-22.md`
- `docs/kb-golden-sample-v0.2-snapshot-import-readiness-review-2026-04-22.md`
- `docs/golden-sample-validator-contract-candidate-2026-04-22.md`
- `docs/v3-golden-sample-contract-freeze-review-2026-04-22.md`
- `docs/control-thread-context-supervision-2026-04-22.md`

KB-side inputs, read only from `E:\codex\hope-kb`:

- branch: `codex/contracts-freeze`
- anchor: `818c098` (`Add golden sample v0.2 snapshot import readiness`)
- `docs/golden-sample-v0.2-snapshot-import-readiness-2026-04-22.md`
- `seed/v0.1/prompt_templates.json`
- `seed/v0.1/runtime_consume_contracts.json`
- `seed/v0.2/golden_sample_field_coverage_rules.json`

## Candidate Decision

`GENERATION_FIELD_RULE_CONTRACT_CANDIDATE_FROZEN_DOCS_ONLY`

Hope may use this candidate as the canonical planning contract for how future
generation phases are allowed to write fields, read KB assets, retrieve golden
samples, invoke V3 cores, route validator gates, and scope repair.

This is not an implementation authorization. It is the candidate that must be
accepted before any writer/orchestrator, validator, intake/Qwen retrieval, or
export/debug implementation package can start.

## Contract Objects

The future implementation-facing contract should preserve these object names:

- `GenerationFieldRuleContract`
- `GenerationPhase`
- `FieldOwnershipRule`
- `KbSourceSelectionRule`
- `GoldenSampleRetrievalRule`
- `V3CoreCoverageRule`
- `ValidationGateRule`
- `RepairScopeRule`
- `ImplementationGateOrder`

The candidate is intentionally written as a document first. DTOs, schemas, and
runtime structs remain closed until a later explicit implementation gate.

## Generation Phase List

The candidate generation phases are:

1. `input_preflight`
2. `synopsis_to_story`
3. `story_to_outline`
4. `story_to_screenplay`
5. `dialogue_refine`
6. `screenplay_to_segments`
7. `segment_to_cuts`
8. `director_variant_action`
9. `director_variant_emotion`
10. `director_variant_suspense`
11. `director_variant_transition`
12. `v3_core_generation`
13. `reference_control_planning`
14. `cut_to_layout`
15. `layout_to_render`
16. `validation`
17. `repair_pass`
18. `repair_hardlocks`
19. `repair_continuity`
20. `repair_export_contract`
21. `repair_handoff_boundaries`
22. `export_summary`
23. `export_adapter`

The phase list follows the accepted planning package and expands it with the
explicit prompt stages present in `prompt_templates.json`.

## Phase Field Rule Matrix

| generation_phase | allowed_write_fields | forbidden_fields | allowed_kb_sources | golden_sample_retrieval | V3 core write timing | validator_gate_owner |
| --- | --- | --- | --- | --- | --- | --- |
| `input_preflight` | input envelope, project intent, target length, target style, duration policy, hard-lock inventory, optional asset inventory | Story, Screenplay, RenderSegment, Cut, PromptPackage, V3 core fields, export rows, Seedance payload | source policy, runtime consume contracts as gate references | none | none | future input completeness validator |
| `synopsis_to_story` | world summary, protagonist goal, conflict source, character arc summary, story outline, beat seed list, story core intent, emotional baseline | NarrativeScene, DialogueTurn, RenderSegment, Cut, layout prompt, render prompt, PromptPackage, V3 core fields, exporter rows | `prompt_01`, story structure templates, manga structure rules, dialogue style rules as background | none unless a later story-level sample gate exists | none | future story structure validator |
| `story_to_outline` | beat list, beat count, beat goal, obstacle, turn, result, emotional progression, target-duration allocation | NarrativeScene body, DialogueTurn body, RenderSegment boundary, Cut, layout prompt, render prompt, V3 core fields | `prompt_02`, story structure templates, character arc patterns | none unless a later outline-level sample gate exists | none | future beat coverage validator |
| `story_to_screenplay` | NarrativeScene drafts, DialogueTurn drafts, scene goal, conflict, result, characters, emotional state, dialogue task | RenderSegment crossing rules, cut list, prompt package, layout prompt, render prompt, Seedance adapter fields | `prompt_03`, dialogue style rules, scene taxonomy as classification support | optional future retrieval by genre or scene only; no raw golden row output | none | future screenplay structure validator |
| `dialogue_refine` | refined dialogue text, dialogue tone notes, subtext notes tied to an existing DialogueTurn | new plot beats, scene boundaries, RenderSegment, Cut, layout prompt, render prompt, V3 core fields | `prompt_08`, dialogue style rules, director profile as style constraint | no golden sample prompt context under current package | none | future dialogue preservation validator |
| `screenplay_to_segments` | RenderSegment plan, target duration, segment boundary, `narrative_scene_id`, segment purpose, segment ordering | crossing NarrativeScene boundaries, Cut body, V3 core prose, layout prompt, render prompt, exporter rows | `prompt_04`, scene taxonomy, runtime consume contracts `runtime_01` and `runtime_02` | no positive prompt context | none | segment duration and boundary validator |
| `segment_to_cuts` | Cut plan, cut number, shot description, action description, dialogue placement, `transition_out`, `continuity_refs`, cut reason | layout prompt, render prompt, compiled Seedance payload, exporter workbook columns | `prompt_05`, committee handoff rules, committee style merge rules, continuity rules, scene taxonomy, runtime contract `runtime_02` | future retrieval may select examples by `scene_tag` or `shot_type` only after retrieval gate | none | continuity, handoff, cut traceability validators |
| `director_variant_action` | action-segment cut variant, action rhythm, action axis, impact points, continuity-preserving cut adjustments | hard-lock rewrites, unrelated scene rewrites, layout prompt, render prompt, V3 core fields | `prompt_11`, action director profile, committee style merge rules, continuity rules | no positive prompt context unless a future retrieval gate opens director-variant examples | none | action continuity and hard-lock validators |
| `director_variant_emotion` | emotion-segment cut variant, shot emphasis, environment-emotion support, dialogue pause placement, micro-performance notes | plot outcome rewrites, relationship logic rewrites, layout prompt, render prompt, V3 core fields | `prompt_12`, emotion director profile, dialogue style rules, character arc patterns | no positive prompt context unless a future retrieval gate opens director-variant examples | none | emotion continuity and scene-purpose validators |
| `director_variant_suspense` | suspense-segment cut variant, reveal boundary, information delay, clue framing, continuity-preserving cut adjustments | direct answer leaks, scene boundary rewrites, layout prompt, render prompt, V3 core fields | `prompt_13`, suspense director profile, scene taxonomy, continuity rules | no positive prompt context unless a future retrieval gate opens director-variant examples | none | suspense reveal-boundary validator |
| `director_variant_transition` | handoff-zone cuts, transition logic, buffer cut references, source/target segment bridge notes | NarrativeScene boundary crossing, unrelated cut rewrites, layout prompt, render prompt, V3 core fields | `prompt_14`, committee handoff rules, runtime contract `runtime_03`, transition director profile | no positive prompt context unless a future retrieval gate opens handoff examples | none | Handoff Coverage and Continuity Validator |
| `v3_core_generation` | `visual_scene_core`, `motion_performance_core`, `camera_directing_core`, `audio_directing_core`, `continuity_lock_core`, source/cut linkage metadata | `reference_control_core`, layout prompt, render prompt, Seedance payload, exporter workbook columns | golden sample library, golden sample field coverage rules, V3 vocabulary, continuity rules, director/committee rules | positive few-shot only when `usable_for_fewshot = Yes` and negative gates pass | writes covered V3 cores after Cut exists and before layout/render prompt writing | golden coverage, empty-word, negative-sample, unusable-fewshot, provenance validators |
| `reference_control_planning` | reference-control planning notes, unresolved asset requirements, asset registry dependency notes | authoritative `reference_control_core`, reference-image binding, final asset payload, render prompt default fill | future asset registry and source register only; current golden samples cannot fill this | none from current v0.2 package | no `reference_control_core` write until future coverage or asset-registry gate | future asset/reference validator |
| `cut_to_layout` | layout prompt, composition, viewpoint, spatial direction, light direction, camera/layout constraints | character appearance detail, final render style, render prompt, negative prompt, Seedance payload | `prompt_06`, camera terms, visual vocabulary, scene tokens, selected V3 visual/camera context | examples may guide internal constraints only after retrieval gate; no negative rows in positive context | reads V3 visual/camera fields; does not write V3 cores | prompt quality and layout/render separation validator |
| `layout_to_render` | render prompt, visible action detail, role tokens, hard-lock preservation, negative prompt where contract allows, target model family metadata | changing cut intent, changing layout, inventing assets, local default refill, validation message leakage | `prompt_07`, director profiles, failure patterns, prompt templates, runtime contract `runtime_04`, selected V3 cores | positive few-shot may guide prose density after retrieval gate; negative rows excluded | reads covered V3 cores; does not write V3 cores | prompt quality, hard-lock, style unity, export sanity validators |
| `validation` | validation report, validator name, status, severity, message, related cut/segment, blocking flag, coverage findings | rewriting source payload, repairing fields in place, render prompt mutation | failure patterns, runtime contracts `runtime_01` through `runtime_05`, golden failure mapping, classic cases | negative samples may be validator fixtures; positive rows may be comparison evidence | reads V3 cores as validation subjects only | Hope validator owner |
| `repair_pass` | minimal repair draft scoped to validator failures, repaired field proposals, repair rationale, target schema alignment | rewriting unrelated story/segments/cuts/prompts, exporter row mutation, Qwen live calls | `prompt_09`, failure mappings, repair mappings, degraded-input examples | gap and negative rows may provide repair evidence after gate | repair may target V3 core fields only when validator names those fields | repair contract owner |
| `repair_hardlocks` | hard-lock repair proposal for failed cuts/render prompts, literal hard-lock restoration notes | layout changes, narrative changes, unrelated prompt rewrites | `prompt_15`, failure patterns, hard-lock rules, runtime contract `runtime_04` | negative or gap rows may be repair evidence, not positive prompt context | no V3 writes unless a named V3 hard-lock field failure exists | hard-lock validator and repair owner |
| `repair_continuity` | adjacent-cut continuity repair proposal, eye-line/screen-direction/action-bridge corrections, named rule fix | whole-segment rewrite, unrelated cut reorder, story rewrite | `prompt_16`, continuity rules, committee handoff rules, runtime contracts `runtime_02` and `runtime_03` | continuity rows may provide repair evidence after gate | may target `continuity_lock_core` only for named continuity failure | Continuity Validator and repair owner |
| `repair_export_contract` | export mapping repair proposal, shared contract source alignment, column-source correction | workbook behavior change, new sheet creation, product exporter local special casing | `prompt_17`, export templates, runtime contracts, shared contract rows | no golden sample positive context; provenance metadata only after exporter gate | no V3 writes | Export contract check and repair owner |
| `repair_handoff_boundaries` | handoff zone repair proposal, buffer cut repair proposal, bridge logic, source/target segment boundary notes | crossing NarrativeScene boundary, unrelated cut rewrite, layout/render mutation | `prompt_18`, committee handoff rules, runtime contract `runtime_03` | handoff examples may be repair evidence only after gate | no V3 writes unless named continuity field is failed | Handoff Coverage and repair owner |
| `export_summary` | human-readable export summary, review notes, validation summary | Excel workbook rows, JSON export contract, Markdown export contract, product exporter behavior | `prompt_10`, export templates, runtime contracts | provenance/debug metadata only after exporter gate | no V3 writes | exporter/debug metadata owner |
| `export_adapter` | export/debug metadata planning notes after future gate, adapter compile notes | changing v0.1 workbook, exporter behavior, IPC payloads, Seedance payload | export templates, runtime consume contracts | provenance/debug metadata only after exporter gate | no V3 writes | exporter/debug metadata owner |

## KB Source Selection Rules

The source router must preserve stage ownership:

- Story phases may read story structure templates, manga structure rules,
  dialogue style rules, and `prompt_01` through `prompt_03`.
- Dialogue refinement may read `prompt_08`, dialogue style rules, and director
  profiles, but it must preserve the existing dialogue task.
- Segment and cut phases may read `prompt_04`, `prompt_05`, scene taxonomy,
  continuity rules, committee handoff rules, committee style merge rules, and
  runtime contracts `runtime_01` through `runtime_03`.
- Director variant phases may read `prompt_11` through `prompt_14` and the
  relevant director profile, but they must preserve hard locks, boundaries, and
  traceability.
- V3 core generation may read golden sample library records, field coverage
  rules, vocabulary, director/committee rules, and continuity rules only after a
  future retrieval boundary gate.
- Layout and render phases may read `prompt_06`, `prompt_07`, prompt templates,
  selected V3 cores, director profiles, failure patterns, and runtime contract
  `runtime_04`.
- Validation may read failure patterns, golden failure mapping, runtime
  contracts, classic cases, and selected generated payloads.
- Repair phases may read `prompt_09`, `prompt_15`, `prompt_16`, `prompt_17`,
  `prompt_18`, failure mappings, repair mappings, degraded-input examples, and
  only the current payload fields named by validator findings.
- Export summary or adapter planning may read `prompt_10`, export templates, and
  runtime contracts, but cannot change workbook/exporter behavior under this
  candidate.

## Golden Sample Retrieval Rules

Golden sample rows are future evidence, not current product output.

Positive retrieval must satisfy all of these conditions:

- the intake / Qwen retrieval boundary gate has been accepted
- the requested `core` is one of the five covered cores
- `usable_for_fewshot = Yes`
- the row is not marked by the negative-sample gate
- the row is not in tier `差`
- the retrieval context preserves `sample_id`, `core`, source fields, and
  provenance metadata
- the row is used as guidance, not copied as final wording

Rows must be excluded from positive prompt context when any of these is true:

- `usable_for_fewshot = No`
- `tier = 差`
- the row is marked as a negative sample
- the row is selected only as validator evidence or repair evidence
- the row belongs to a missing or uncovered core
- source provenance is unavailable or inconsistent

Allowed retrieval filters after the retrieval gate:

- `core`
- `tier`
- `genre`
- `scene_tag`
- `shot_type`
- `director_voice` as internal retrieval metadata only
- `covered_points` and `missed_points` for validation or repair routing only

Negative or unusable rows may still be used as:

- validator negative fixtures
- coverage-gap repair evidence
- empty-word repair evidence
- debug/provenance references after exporter/debug metadata gates

They must never become positive few-shot examples.

## V3 Core Coverage Rule

The accepted v0.2 KB package covers exactly five V3 cores, eight rows per core:

- `visual_scene_core`
- `motion_performance_core`
- `camera_directing_core`
- `audio_directing_core`
- `continuity_lock_core`

The `v3_core_generation` phase may write only those five cores, and only after:

1. the Cut exists and is traceable to a RenderSegment
2. hard locks and continuity constraints are available
3. the retrieval boundary gate has defined how golden rows enter context
4. positive and negative sample exclusion has been enforced
5. validator evidence rules have accepted how coverage and empty-word evidence
   will be judged

The V3 core writer must not compile Seedance payloads, change exporter columns,
or rewrite the source Cut's narrative purpose.

## `reference_control_core` Boundary

`reference_control_core` remains uncovered in the current KB v0.2 golden sample
package.

Current candidate rule:

- no phase may generate authoritative `reference_control_core`
- no phase may infer `reference_control_core` from the five covered cores
- no phase may backfill asset/reference control from director voice, source cut,
  or prompt wording
- `reference_control_planning` may record planning notes and unresolved asset
  requirements only
- future coverage requires either a KB sample update gate, an asset registry
  gate, or another main-control accepted source package

This prevents V3 from skipping KB and main-control review to enter
implementation through a reference-control side door.

## Validator Gate Mapping

| validation_gate | owns / checks | relevant phases |
| --- | --- | --- |
| future input completeness validator | input envelope, target length, style intent, hard-lock inventory | `input_preflight` |
| future story structure validator | story premise, protagonist goal, conflict, character arc, outline | `synopsis_to_story` |
| future beat coverage validator | beat count, beat function, turn, result, emotional progression | `story_to_outline` |
| future screenplay structure validator | NarrativeScene and DialogueTurn separation, scene goal, conflict/result | `story_to_screenplay` |
| future dialogue preservation validator | dialogue role preservation, no plot rewrite during polish | `dialogue_refine` |
| segment duration and boundary validator | RenderSegment boundary, duration policy, no cross-scene segment | `screenplay_to_segments` |
| Continuity Validator | continuity refs, adjacent-cut action and screen direction, continuity core checks | `segment_to_cuts`, `director_variant_*`, `repair_continuity` |
| Handoff Coverage | handoff zone integrity, buffer cuts, source/target segment bridge | `director_variant_transition`, `repair_handoff_boundaries` |
| golden coverage validator | covered/missed V3 checklist evidence | `v3_core_generation`, `validation` |
| empty-word validator | vague wording and `empty_words_detected` evidence | `v3_core_generation`, `validation`, repair phases |
| negative-sample validator | negative fixture exclusion and unusable positive context detection | `v3_core_generation`, `validation` |
| unusable-fewshot validator | `usable_for_fewshot = No` exclusion from positive retrieval | `v3_core_generation`, future retrieval gate |
| provenance validator | `sample_id`, source fields, source hash, row/provenance linkage | `v3_core_generation`, `validation`, future exporter/debug metadata |
| prompt quality review / export sanity check | prompt alias leakage, validation-message leakage, Chinese prompt noise | `cut_to_layout`, `layout_to_render`, `validation`, repair phases |
| hard-lock validator | hard-lock preservation in cuts and render prompts | `segment_to_cuts`, `layout_to_render`, `repair_hardlocks` |
| Export contract check | export field origin, sheet/column mapping, runtime consumer projection | `repair_export_contract`, `export_summary`, `export_adapter` |

The validator gate mapping is planning-only. It does not define Rust types,
thresholds, fixture files, or runtime execution behavior yet.

## Repair Scope Rule

Repair is allowed only after validation names a failed field or link.

Global repair constraints:

- repair only fields named by validator findings
- preserve passed fields
- preserve source provenance and traceability
- preserve hard locks unless the validator finding is specifically about
  malformed hard locks
- avoid whole-stage rewrites when a local repair is possible
- do not use negative samples as positive prompt context
- keep negative-boundary evidence inspectable
- never repair by changing exporter/workbook behavior under this candidate
- never call Qwen or Seedance under this candidate

Phase-specific repair constraints:

- `repair_pass` may propose minimal field-level repairs for current payloads
  against a target schema.
- `repair_hardlocks` may touch only hard-lock loss, order instability, or
  local director override in failed render prompts.
- `repair_continuity` may touch only failed links, adjacent cuts, continuity
  fields, and named continuity rules.
- `repair_export_contract` may propose mapping corrections but cannot implement
  exporter behavior or workbook changes.
- `repair_handoff_boundaries` may touch only failed handoff zones, buffer cuts,
  adjacent cuts, and bridge notes.

## Runtime Consume Boundary

The current v0.1 runtime consume contracts remain read-only evidence for future
consumer-facing rules.

This candidate preserves these boundaries:

- `runtime_01` governs snapshot/bootstrap gates and required validation/export
  surfaces.
- `runtime_02` governs RenderSegment and Cut projection.
- `runtime_03` governs handoff zone projection.
- `runtime_04` governs PromptPackage projection.
- `runtime_05` governs validation feedback projection.

Future implementation must consume these as constraints, not as permission to
rewrite workbook shape or infer missing fields locally.

## Future Implementation Gate Order

Do not jump directly to Hope implementation. Use this order:

1. `Generation Field Rule Contract Acceptance`
   Main control accepts or revises this candidate as the canonical docs-only
   field rule contract.
2. `KB support update if needed`
   Add missing KB-side metadata only through a bounded KB-only gate, especially
   for story-stage samples or `reference_control_core` coverage.
3. `V3 proposal refresh if needed`
   Update V3 freeze/proposal materials only after KB changes are accepted by
   main control.
4. `Hope validator evidence contract`
   Define DTO and validator behavior for golden coverage, empty-word,
   negative-sample, unusable-fewshot, provenance-gap, and stage ownership
   signals.
5. `Hope repair mapping contract`
   Define how failure mapping and repair mapping become bounded repair
   suggestions.
6. `Intake / Qwen retrieval boundary`
   Define retrieval filters, positive few-shot eligibility, negative exclusion,
   prompt context assembly, and provenance preservation before live model calls.
7. `Writer / orchestrator implementation`
   Implement stage sequencing only after validator, repair, and retrieval
   contracts are accepted.
8. `Exporter/debug metadata planning`
   Decide whether read-only provenance metadata can be exposed without changing
   the v0.1 workbook contract.
9. `Desktop read-only provenance planning`
   Define UI-only provenance display if needed. No KB internal data should be
   exposed by default.
10. `Seedance adapter boundary`
    Open only after prompt package, validation, and exporter/debug boundaries
    are stable.

## Explicit Prohibitions

This candidate does not authorize:

- Rust DTO changes
- validator implementation
- repair implementation
- writer or orchestrator implementation
- exporter behavior changes
- workbook changes
- IPC changes
- desktop changes
- intake changes
- Qwen calls
- Seedance calls
- `hope-kb` edits
- merging `hope-kb` into `hope`
- V3 branch edits
- V3 implementation
- importing golden samples into v0.1 assets
- using negative samples as positive prompt examples
- inventing `reference_control_core` coverage

## Candidate Conclusion

`GENERATION_FIELD_RULE_CONTRACT_CANDIDATE_DOCS_ONLY_COMPLETE`

This candidate freezes the stage-field-rule matrix, KB source routing, golden
sample retrieval and exclusion rules, V3 covered-core write timing,
`reference_control_core` boundary, validator gate mapping, repair scope rules,
and future implementation gate order.

The next action is main-control acceptance or revision of this candidate. All
implementation gates remain closed.

关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。
