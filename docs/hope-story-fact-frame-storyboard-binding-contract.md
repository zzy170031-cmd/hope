# Hope Story Fact Frame & Storyboard Binding Contract

Status: draft executable contract for desktop QA gates.

Scope: desktop shell, qwen-max UI-driven release QA, pre-403 gates.

## Goal

Stop storyboard drift by binding every generated row to the current user input, accepted rewrite, scene type, duration, and KB rule pack.

This contract is a product/runtime/QA contract. It is not only a prompt instruction.

This contract inherits `docs/hope-scene-type-rewrite-contract.md`. StoryFactFrame
must bind to the confirmed narrative story body and stable atomic facts, not to
old scene style, old duration strategy, storyboard text, prompt text, or QA
trace.

## Ownership

- Initiator: frontend UI starts a current-case run when the user inputs text, chooses scene type and duration, and confirms the rewrite.
- Executor: Rust runtime performs rewrite, duration planning, storyboard generation, local validation, bounded repair, and binding evidence generation.
- Constraint source: KB supplies bounded rule packs only. KB must not inject source-external facts.
- Controller: CurrentCaseBindingPreflight blocks stale source, stale accepted text, stale scene, stale duration, or stale task script binding before storyboard generation.
- Implementers: frontend owns accepted snapshot and trace; runtime owns StoryFactFrame, preflight and validators; runner owns UI-driven evidence; docs own product contract.
- Gate owner: QA thread proves four trace gate, scene-type rewrite gate, and cross rewrite drift smoke before 403-case.

## StoryFactFrame

StoryFactFrame is created before or at accepted rewrite confirmation.

For rewrite, the source side of StoryFactFrame is the current accepted story
fact source after stable-fact extraction. The selected `scene_type` and duration
are control inputs, not facts. They may shape expression and capacity, but they
must not rewrite the fact boundary.
Duration binding applies to every product target duration. Current fixed values
and future positive fixed values must normalize into capacity bands; a changed
duration must change narrative capacity and downstream storyboard timing, not
only the stored `duration_seconds` field.

Minimum MVP fields:

- source_profile
- characters
- relationships
- locations
- events
- pressure_relations
- must_keep_facts
- forbidden_facts

Short text rule:

- Extract only explicit characters, roles, locations, actions, conflict pressure and event order.
- Do not infer weapons, injuries, ranks, armies, numbers, props, or identities.

Complex text rule:

- Extract character table, relationship table, location table, event chain, pressure line, scene segments, and forbidden additions.
- Use the same schema as short text; complex text only fills more fields.

## KBRulePack

KB is a bounded rule pack, not a free creative material source.

Required evidence:

- kb_rule_pack_ids
- kb_snapshot_hash
- selected rule summaries
- allowed expression scope
- forbidden external facts

Forbidden:

- Raw KB rows in prompt.
- KB-generated new characters, props, weapons, ranks, injuries, army scale, or worldbuilding facts not present in the source.
- Silent model or KB rule switching inside a single run.

## Accepted Rewrite Snapshot

The "confirm use" action freezes an accepted_rewrite_snapshot.

Minimum fields:

- current_case_id
- source_text_hash
- accepted_rewrite_hash
- scene_type
- scene_label
- duration_seconds
- target_duration_mode
- story_fact_frame_hash
- StoryFactFrame

Storyboard generation must use the accepted snapshot. It must not read old textarea, old trace, old task, old scene, or old duration state.

## CurrentCaseBindingPreflight

Before generate_storyboard, the runtime checks:

- current task script hash matches the submitted task script.
- scene_type is present.
- selected duration is supported.
- accepted snapshot binding fields are present when provided.

Future expansion:

- source_text_hash must match accepted snapshot source.
- accepted_rewrite_hash must match accepted full rewrite.
- task queue must reference the current accepted snapshot.
- stale_binding_detected blocks generation.

## StoryboardBindingEvidence

Every generate_storyboard response exposes binding evidence:

- current_case_id
- source_text_hash
- accepted_rewrite_hash
- task_script_hash
- story_fact_frame_hash
- source_profile
- scene_type
- duration_seconds
- duration_plan_hash
- storyboard_rows_hash
- must_keep_facts
- missing_source_facts
- forbidden_facts
- forbidden_fact_hits
- stale_binding_detected
- kb_rule_pack_ids
- kb_snapshot_hash

This evidence must be sanitized. It must not include secret, raw env, raw prompt body, raw KB rows, source register, or overlay JSON.

## Narrative Beat Binding Boundary

`StoryFactFrame.must_keep_facts` must separate three classes before validation:

- atomic source facts: explicit people, roles, places, objects explicitly in the
  source, visible actions, conflict pressure, and event order.
- narrative beats: complete-story prose about motive, pressure, emotion,
  relation change, decision, agency, or resolution.
- field noise: title words, situation words, action fragments, support notes,
  prompt packaging, trace labels, and UI evidence strings.

Storyboard rows must prove atomic source facts directly in row fields. Narrative
beats must never be required as verbatim row text. They must be mapped through a
source-profile equivalence rule into visible row atoms.

Required equivalence rule shape:

```text
source_profile
narrative_beat_kind
required_visible_row_atoms
forbidden_shortcut_fields
regression_test_name
error_ledger_card
```

Examples of required visible row atoms:

- A ruin / duel pressure: protagonist, enemy, ruin or duel space, approach or
  compressed distance, and a visible hold/brace action.
- A ruin / decision pressure: protagonist, enemy, ruin or duel space, approach
  or pressure, plus a visible decision/response atom such as bracing, looking
  toward the enemy, or holding the duel distance. Do not require the full prose
  sentence about "judging the next move" to appear in rows.
- B alley pursuit pressure: Lin Feng, Su Yao, A Qing, pursuer, alley mouth, and
  approach pressure.

Forbidden shortcuts:

- Do not copy the entire `NarrativeStoryBody` sentence into a storyboard row to
  satisfy `missing_source_facts`.
- Do not count `prompt_text`, support notes, trace, source register, overlay
  JSON, hidden backend-only fields, or QA diagnostic text as story-fact coverage.
- Do not resolve repeated `missing_source_facts` by adding a global loose word
  allowlist.
- Do not remove or downgrade `missing_source_facts`, visible row scene/duration,
  prompt boundary, KB leakage, or pseudo-success gates to make a case pass.

When a fresh gate exposes a new narrative-beat binding miss:

1. Classify it as atomic source fact, narrative beat, or field noise.
2. If it is a narrative beat, add or update a source-profile equivalence rule.
3. Add the smallest Rust regression proving the beat maps to required visible
   row atoms and does not use prompt-only coverage.
4. Add or update runner/certifier assertion only if the evidence path is the
   real gap.
5. Add or update an error-ledger card with phenomenon, root cause, wrong path,
   correct fix, evidence, and remaining risk.
6. Rebuild release and rerun targeted before rerunning `full16`.

This boundary exists to prevent recurring local fixes where each fresh full16
case discovers the same narrative-body sentence problem under a different
source profile.

## UI Contract

The rewrite confirmation editor should become a fact-locking point.

Visible product sections:

-人物关系
-场景事实
-当前场景类型
-当前时长
-禁止新增
-确认改写正文

MVP may expose the full structured evidence only in QA trace first. Product UI can later show a lighter summary.

## QA Gate

Pre-403 order:

1. B_hot blocker fixed.
2. Four-group trace gate passes.
3. Scene-type-driven rewrite gate passes.
4. Cross rewrite drift smoke passes.
5. Core Challenger reviews evidence.
6. Only then prepare 403-case.

Cross rewrite drift smoke must cover:

- same script, different duration, same scene.
- same script, same duration, different scene.
- same script, same scene, different duration.
- same duration, different script, same scene.
- same duration, same scene, different script.

Each case records source_profile, scene_type, duration, accepted_text_hash, duration_plan_hash, storyboard_rows_hash, rows_match, binding evidence, repair/fallback, layout bounds and StopOnly result.

## Red Lines

- Do not claim rows_match=true means semantic pass.
- Do not hide fallback or repair.
- Do not use fallback rows as qwen live pass.
- Do not enter 403-case before binding and cross drift gates.
- Do not package before 403-case or an explicitly accepted degraded gate.

## 2026-05-08 Field-Aware Binding Addendum

This contract inherits:

```text
docs/hope-field-aware-story-contract.md
docs/hope-provider-failover-contract.md
docs/hope-qa-evidence-freshness-contract.md
```

`StoryFactFrame` must bind to `AcceptedNarrativeFrame` and the confirmed
`NarrativeStoryBody`, not to `SupportNotes`, storyboard rows, `prompt_text`, or
old scene/duration expression.

Before storyboard generation, `StoryboardInputFrame` must prove:

- current accepted body hash
- current scene type and profile ID
- current duration capacity profile
- KB rule-pack IDs and snapshot hash
- positive KB structural evidence
- negative raw-leakage evidence
- field-aware entity classification for `person`, title fields,
  `visual_description`, `character_action`, and `prompt_text`

Storyboard rows cannot flow backward into the story body. `prompt_text` remains
packaging and lineage evidence, not a source of person or story facts.
