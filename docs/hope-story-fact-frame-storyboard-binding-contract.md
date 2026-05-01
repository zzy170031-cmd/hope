# Hope Story Fact Frame & Storyboard Binding Contract

Status: draft executable contract for desktop QA gates.

Scope: desktop shell, qwen-max UI-driven release QA, pre-403 gates.

## Goal

Stop storyboard drift by binding every generated row to the current user input, accepted rewrite, scene type, duration, and KB rule pack.

This contract is a product/runtime/QA contract. It is not only a prompt instruction.

## Ownership

- Initiator: frontend UI starts a current-case run when the user inputs text, chooses scene type and duration, and confirms the rewrite.
- Executor: Rust runtime performs rewrite, duration planning, storyboard generation, local validation, bounded repair, and binding evidence generation.
- Constraint source: KB supplies bounded rule packs only. KB must not inject source-external facts.
- Controller: CurrentCaseBindingPreflight blocks stale source, stale accepted text, stale scene, stale duration, or stale task script binding before storyboard generation.
- Implementers: frontend owns accepted snapshot and trace; runtime owns StoryFactFrame, preflight and validators; runner owns UI-driven evidence; docs own product contract.
- Gate owner: QA thread proves four trace gate, scene-type rewrite gate, and cross rewrite drift smoke before 403-case.

## StoryFactFrame

StoryFactFrame is created before or at accepted rewrite confirmation.

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
