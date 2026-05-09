# Desktop 21 Scene Type Reference Samples Draft

Status: draft_reviewed_needs_validator.
Purpose: QA/reference-only candidate. This is not source of truth, not runtime prompt input, not a replacement for the 27-sample set, and not 16-case / 403 pass evidence.

## Input And Artifacts

- Input DOCX: `C:\Users\Administrator\Desktop\hope\Hope桌面端-21scene_type-QA样本集.docx`
- DOCX SHA256: `1677ece79c7277dcb5a44073c43f949299caafed132384d463f998e2228a9c47`
- Draft JSON: `tests/qa/desktop-21-scene-type-reference-samples.draft.json`
- Sample count: 21
- runtime_usage: `forbidden`

## 21 Sample List

| # | scene_type | DOCX label | canonical label | alias | reference_id | source_sample_id |
|---|---|---|---|---|---|---|
| 01 | `hot_blood_battle` | 热血战斗 | 热血战斗 | none | `qa_reference_21_hot_blood_battle_001` | `sample_hot_blood_battle_qa_reference_001` |
| 02 | `ensemble_performance` | 群像表演 | 群像表演 | none | `qa_reference_21_ensemble_performance_001` | `sample_ensemble_performance_qa_reference_001` |
| 03 | `emotional_dialogue` | 情绪对话 | 情绪对话 | none | `qa_reference_21_emotional_dialogue_001` | `sample_emotional_dialogue_qa_reference_001` |
| 04 | `encounter_performance` | 相遇表演 | 相遇表演 | none | `qa_reference_21_encounter_performance_001` | `sample_encounter_performance_qa_reference_001` |
| 05 | `field_chase` | 场域追逐 | 场域追逐 | none | `qa_reference_21_field_chase_001` | `sample_field_chase_qa_reference_001` |
| 06 | `spectacle_showcase` | 奇观展示 | 奇观展示 | none | `qa_reference_21_spectacle_showcase_001` | `sample_spectacle_showcase_qa_reference_001` |
| 07 | `daily_healing` | 日常治愈 | 日常治愈 | none | `qa_reference_21_daily_healing_001` | `sample_daily_healing_qa_reference_001` |
| 08 | `guoman_hot_blood_combat` | 国漫热血打斗 | 国漫热血打斗 | none | `qa_reference_21_guoman_hot_blood_combat_001` | `sample_guoman_hot_blood_combat_qa_reference_001` |
| 09 | `guoman_ensemble_performance` | 国漫群像表演 | 国漫群像表演 | none | `qa_reference_21_guoman_ensemble_performance_001` | `sample_guoman_ensemble_performance_qa_reference_001` |
| 10 | `ink_wuxia_combat` | 水墨武打 | 水墨武打 | none | `qa_reference_21_ink_wuxia_combat_001` | `sample_ink_wuxia_combat_qa_reference_001` |
| 11 | `eastern_spectacle` | 东方奇观 | 东方奇观 | none | `qa_reference_21_eastern_spectacle_001` | `sample_eastern_spectacle_qa_reference_001` |
| 12 | `xianxia_action` | 仙侠动作 | 仙侠动作 | none | `qa_reference_21_xianxia_action_001` | `sample_xianxia_action_qa_reference_001` |
| 13 | `urban_fantasy` | 都市奇幻 | 都市奇幻 | none | `qa_reference_21_urban_fantasy_001` | `sample_urban_fantasy_qa_reference_001` |
| 14 | `chinese_war_formation` | 国战军阵建立 | 国战军阵建立 | none | `qa_reference_21_chinese_war_formation_001` | `sample_chinese_war_formation_qa_reference_001` |
| 15 | `weapon_highlight` | 武将兵器高光 | 武将兵器高光 | none | `qa_reference_21_weapon_highlight_001` | `sample_weapon_highlight_qa_reference_001` |
| 16 | `council_strategy` | 朝堂军帐权谋 | 朝堂军帐权谋 | none | `qa_reference_21_council_strategy_001` | `sample_council_strategy_qa_reference_002` |
| 17 | `siege_defense` | 多军团攻城防守 | 多军团攻城 | 多军团攻城防守 | `qa_reference_21_siege_defense_001` | `sample_siege_defense_qa_reference_001` |
| 18 | `slg_sandbox_view` | 沙盘战略视口 | 沙盘战略视口 | none | `qa_reference_21_slg_sandbox_view_001` | `sample_slg_sandbox_view_qa_reference_001` |
| 19 | `slg_march_encirclement` | 行军轨迹合围 | 行军轨迹合围 | none | `qa_reference_21_slg_march_encirclement_001` | `sample_slg_march_encirclement_qa_reference_001` |
| 20 | `slg_city_growth` | 城建演进反馈 | 城建演进反馈 | none | `qa_reference_21_slg_city_growth_001` | `sample_slg_city_growth_qa_reference_001` |
| 21 | `slg_battle_report` | 战报 UI | 战报 UI | none | `qa_reference_21_slg_battle_report_001` | `sample_slg_battle_report_qa_reference_001` |

## sample_id Strategy

- Keep the DOCX `sample_<scene>_qa_reference_001` / `sample_council_strategy_qa_reference_002` as `source_sample_id`.
- Use a separate draft namespace: `qa_reference_21_<scene_type>_001`.
- Do not reuse existing 27-sample `sample_<scene>_001` or supplemental IDs.
- The `council_strategy` DOCX `_002` is a source version marker, not the draft ordering ID.

## scene_type / scene_label Crosswalk

- All 21 `scene_type` names match repo scene_index names; no duplicates or missing entries.
- `siege_defense` uses the canonical label shown in row 17 of the table above from repo scene_index.
- `siege_defense` preserves the DOCX source label shown in row 17 of the table above as alias.

## KB Crosswalk Candidate

`E:\codex\hope-kb\seed\v0.1\scene_taxonomy.json` is a legacy 12-category taxonomy, not an exact 21 desktop scene_type join. This table is only a coarse structure reference, not a KB rule ID.

| scene_type | legacy taxonomy candidate | status |
|---|---|---|
| `hot_blood_battle` | 战斗开场, 战斗高潮 | candidate / structure-only / not KB rule id |
| `ensemble_performance` | 群像集结 | candidate / structure-only / not KB rule id |
| `emotional_dialogue` | 关系停顿, 微表演对白 | candidate / structure-only / not KB rule id |
| `encounter_performance` | 关系停顿, 微表演对白 | candidate / structure-only / not KB rule id |
| `field_chase` | 追逐推进 | candidate / structure-only / not KB rule id |
| `spectacle_showcase` | 爆点揭示, 情绪景观 | candidate / structure-only / not KB rule id |
| `daily_healing` | 关系停顿, 微表演对白, 片尾收束 | candidate / structure-only / not KB rule id |
| `guoman_hot_blood_combat` | 战斗开场, 战斗高潮 | candidate / structure-only / not KB rule id |
| `guoman_ensemble_performance` | 群像集结 | candidate / structure-only / not KB rule id |
| `ink_wuxia_combat` | 战斗开场, 战斗高潮, 关系停顿 | candidate / structure-only / not KB rule id |
| `eastern_spectacle` | 爆点揭示, 情绪景观 | candidate / structure-only / not KB rule id |
| `xianxia_action` | 战斗开场, 战斗高潮, 梦境切层 | candidate / structure-only / not KB rule id |
| `urban_fantasy` | 爆点揭示, 悬疑线索, 梦境切层 | candidate / structure-only / not KB rule id |
| `chinese_war_formation` | 群像集结, 战斗开场 | candidate / structure-only / not KB rule id |
| `weapon_highlight` | 战斗高潮, 爆点揭示 | candidate / structure-only / not KB rule id |
| `council_strategy` | 关系停顿, 微表演对白, 悬疑线索 | candidate / structure-only / not KB rule id |
| `siege_defense` | 群像集结, 战斗开场, 战斗高潮 | candidate / structure-only / not KB rule id |
| `slg_sandbox_view` | no exact match; needs desktop/SLG taxonomy | candidate / structure-only / not KB rule id |
| `slg_march_encirclement` | no exact match; needs desktop/SLG taxonomy | candidate / structure-only / not KB rule id |
| `slg_city_growth` | no exact match; needs desktop/SLG taxonomy | candidate / structure-only / not KB rule id |
| `slg_battle_report` | no exact match; needs desktop/SLG taxonomy | candidate / structure-only / not KB rule id |

## Duration Mapping

- DOCX 8s: reference-only; for 5s / 10s, keep the minimum action skeleton or UI state.
- 16-case 15s / matrix_15s_candidate: candidate after review; requires human-defined setup + key action + closure.
- 30s / matrix_30s_candidate: closest existing candidate for 16-case / 403; best first assertion candidate after review, but not 16-case/403 evidence and not gate entry authorization.
- 45s: derive by trimming 60s only after `drop_allowed` is reviewed.
- 60s: reference-only; must not add plot, characters, props, places, or worldview.

## Runtime / Prompt Boundary

Never allowed into runtime prompt_text: `sample_text`, storyboard row raw prose, smoke_extracts text, sample character names, sample props, sample places, sample worldview, prompt_service_notes raw prose.

Allowed as QA oracle only: expected_prompt_oracle, expected_row_oracle, video_failure_criteria, human_editing_notes, forbidden_drift, smoke_extract paraphrase anchors.

May be abstracted into craft/rule tags: low dialogue, object path, reverse action, positional relation, spatial compression, ensemble focus handoff, neutral UI, player agency, visible feedback, action semantics not reversed, camera axis continuity.

## Human Review Rubric

1. Originality / IP boundary: no real IP, work, character, product UI, author, or director imitation target.
2. sample_text visual quality: clear people, space, action chain, objects, start/end states.
3. storyboard row executability: person / visual_description / character_action / camera_movement complete.
4. expected_prompt_oracle usability: abstract visual anchors, action anchors, craft anchors only.
5. expected_row_oracle judgability: row count, four columns, key beats, and rewrite conditions are checkable.
6. video_failure_criteria judgability: failures can be judged from image/action/UI state.
7. forbidden_drift testability: covers sample_text, names, props, places, worldview, smoke_extracts, override boundaries.
8. scene_type representativeness: demonstrates positive mechanism and differs from adjacent scene types.
9. 16-case / KB reference applicability: can map to 15s/30s, rewrite/expand, and structure-only KB crosswalk.

## Pre-Import Blockers

- sample_id strategy needs controller approval.
- `siege_defense` canonical label / alias needs controller approval.
- KB 12-category legacy taxonomy to 21 desktop scene_type crosswalk needs approval.
- 8/30/60 to 5/10/15/30/45/60 duration mapping needs human review.
- smoke_extracts, duration_profiles, runtime_boundary, expected_guard need import validator checks.
- review_scores are filled; external human signoff and source/IP evidence remain pending.
- Runtime source-of-truth enforcement must be added to the schema/import contract before any runtime use.
- This draft must not replace the 27-sample set and must not be used as 16-case/403 pass evidence.

## Human Review Rubric Results

- Review status: `draft_reviewed_needs_validator`
- Samples reviewed: 21
- Blocked samples: 0
- High-risk samples: 6
- Average score across all dimensions: 2.56
- Formal import: not recommended; validator and controller review are still required.
- Runtime use: forbidden.

| scene_type | review_status | average score | follow-up |
|---|---|---:|---|
| `hot_blood_battle` | draft_reviewed_needs_validator | 2.64 | normal follow-up |
| `ensemble_performance` | draft_reviewed_needs_validator | 2.64 | normal follow-up |
| `emotional_dialogue` | draft_reviewed_needs_validator | 2.64 | normal follow-up |
| `encounter_performance` | draft_reviewed_needs_validator | 2.64 | normal follow-up |
| `field_chase` | draft_reviewed_needs_validator | 2.64 | normal follow-up |
| `spectacle_showcase` | draft_reviewed_needs_validator | 2.36 | needs stronger positive large-scale spectacle distinction after human review |
| `daily_healing` | draft_reviewed_needs_validator | 2.64 | normal follow-up |
| `guoman_hot_blood_combat` | draft_reviewed_needs_validator | 2.36 | adjacent to hot_blood_battle / wuxia / xianxia; needs clearer positive differentiation |
| `guoman_ensemble_performance` | draft_reviewed_needs_validator | 2.36 | adjacent to ensemble_performance; needs stronger guoman-specific ensemble differentiation |
| `ink_wuxia_combat` | draft_reviewed_needs_validator | 2.64 | normal follow-up |
| `eastern_spectacle` | draft_reviewed_needs_validator | 2.36 | large-scale spectacle value is good but needs positive scale review |
| `xianxia_action` | draft_reviewed_needs_validator | 2.36 | adjacent to other restrained action samples; needs duration and power-scale review |
| `urban_fantasy` | draft_reviewed_needs_validator | 2.64 | normal follow-up |
| `chinese_war_formation` | draft_reviewed_needs_validator | 2.64 | normal follow-up |
| `weapon_highlight` | draft_reviewed_needs_validator | 2.64 | normal follow-up |
| `council_strategy` | draft_reviewed_needs_validator | 2.64 | normal follow-up |
| `siege_defense` | draft_reviewed_needs_validator | 2.36 | label alias is handled but controller must confirm canonical label before import validator |
| `slg_sandbox_view` | draft_reviewed_needs_validator | 2.64 | normal follow-up |
| `slg_march_encirclement` | draft_reviewed_needs_validator | 2.64 | normal follow-up |
| `slg_city_growth` | draft_reviewed_needs_validator | 2.64 | normal follow-up |
| `slg_battle_report` | draft_reviewed_needs_validator | 2.64 | normal follow-up |

### High-Risk Samples

- `spectacle_showcase`: needs stronger positive large-scale spectacle distinction after human review
- `guoman_hot_blood_combat`: adjacent to hot_blood_battle / wuxia / xianxia; needs clearer positive differentiation
- `guoman_ensemble_performance`: adjacent to ensemble_performance; needs stronger guoman-specific ensemble differentiation
- `xianxia_action`: adjacent to other restrained action samples; needs duration and power-scale review
- `eastern_spectacle`: large-scale spectacle value is good but needs positive scale review
- `siege_defense`: label alias is handled but controller must confirm canonical label before import validator

### Import Validator Preconditions

- Controller confirms sample_id strategy and council_strategy source marker semantics.
- Controller confirms `siege_defense` canonical label and alias handling.
- Schema transformer maps this independent draft schema to any formal import schema.
- Import validator checks review_scores, runtime boundaries, smoke_extract structure, duration_profiles, and no prompt contamination.
- Human IP/source review evidence is recorded before any formal import.

### Why Formal Import Is Still Not Recommended

- review_scores are filled for draft review, but external human signoff, source/IP evidence, and import validator evidence are not yet present.
- KB crosswalk is candidate-only and not an exact KB join.
- Runtime source-of-truth enforcement is pending schema/import contract.
- This draft is not 16-case/403 pass evidence and does not authorize gate entry.

## Import Validator Preparation

- import_validation.status: `preflight_ready_not_formal_import`
- validator_scope: `draft_schema_boundary_only`
- formal_import_allowed: `false`
- runtime_usage_allowed: `false`
- gate_usage_allowed: `false`
- requires_external_human_signoff: `true`
- requires_source_ip_review: `true`
- requires_import_validator_script: `true`

### High-Risk Validator Attention

| scene_type | risk reason | adjacent scene_type confusion risk | required follow-up | import_validator_attention |
|---|---|---|---|---|
| `spectacle_showcase` | Large-scale spectacle candidate is useful, but validator must confirm the spectacle mechanism is not reduced to generic environment display. | May blur with `eastern_spectacle` or `urban_fantasy` if the validator only checks scale and misses the scene_type-specific reveal pattern. | Require validator notes for positive spectacle mechanism, visible scale transition, and distinction from culturally specific or fantasy spectacle samples. | true |
| `guoman_hot_blood_combat` | Action energy is strong but adjacent to `hot_blood_battle`, `ink_wuxia_combat`, and `xianxia_action`. | May be mistaken for generic hot-blood combat, wuxia motion craft, or xianxia power-scale action if style and combat semantics are not checked separately. | Require validator notes separating guoman combat rhythm from generic battle, wuxia ink movement, and xianxia power-scale cues. | true |
| `guoman_ensemble_performance` | Ensemble staging is useful but adjacent to the general `ensemble_performance` category. | May collapse into generic group performance if validator ignores guoman-specific ensemble rhythm, role handoff, and visual emphasis. | Require validator notes proving the sample adds a distinct guoman ensemble pattern rather than duplicating general ensemble coverage. | true |
| `xianxia_action` | Power-scale and duration handling need review before it can guide action assertions. | May blur with `ink_wuxia_combat`, `eastern_spectacle`, or `guoman_hot_blood_combat` if motion craft, power scale, and mythic action cues are not separated. | Require validator notes for power-scale anchors, duration compression, and separation from wuxia craft or generic combat. | true |
| `eastern_spectacle` | Spectacle value is strong but requires positive scale and cultural-structure review. | May blur with `spectacle_showcase` or `xianxia_action` if validator checks only large visual scale and misses eastern spectacle structure. | Require validator notes for structure-only cultural spectacle anchors, scale progression, and no real work/author/director imitation. | true |
| `siege_defense` | Canonical label and source alias are handled, but controller confirmation is still required before formal import. | May blur with `chinese_war_formation`, `slg_march_encirclement`, or generic battle if validator does not preserve multi-force siege/defense semantics. | Require validator notes confirming canonical label 多军团攻城, alias 多军团攻城防守, and separation from formation, march encirclement, and battle-report samples. | true |

### Validator Preflight Checklist

- conversion_notes status is `draft_reviewed_needs_validator`; `human_review_rubric_completed=true` means rubric fields are filled only, while `external_human_signoff_complete=false` remains a formal import blocker.
- Treat `abstract_craft_tags` as candidate-only draft metadata; an import validator must ignore or recompute them until a controlled vocabulary is authorized.
- Confirm top-level and sample-level QA/reference flags stay true and runtime usage stays forbidden.
- Confirm all 21 samples have import_validation, review_scores, human_review_notes, smoke_extracts, duration_profiles, runtime_boundary, and expected_guard fields.
- Confirm smoke_extracts remain paraphrase/fact anchors with original_text_substring=false and runtime_prompt_allowed=false.
- Confirm duration_profiles cover DOCX 8s, 15s/30s candidates, matrix 5s/10s reference-only, 45s after review, and 60s reference.
- Confirm source_ip_review_ready=false, formal_import_ready=false, and runtime_ready=false for every sample.
- Confirm only the six listed high-risk samples carry high_risk_review.import_validator_attention=true.
- Confirm no runtime, 16-case, 403, packaging, or release gate consumes this draft.

### Formal Import Blocker Status

Formal import remains blocked pending external human signoff, source/IP review evidence, an import validator script, schema/import mapping, and controller approval. This draft remains QA/reference-only and must not enter runtime source of truth.

### Runtime / Gate Usage Status

Runtime usage, 16-case usage, 403 usage, packaging usage, and release-gate usage are all forbidden for this draft. The preflight state is not gate evidence and does not authorize runtime prompt consumption.

## Formal 403 Boundary

This draft may support formal 403 only as a sanitized structure oracle after a
controller gate explicitly accepts that use. It remains `qa_only=true`,
`reference_only=true`, `not_source_of_truth=true`,
`raw_sample_text_for_runtime=false`, and `formal_import_allowed=false`.

Allowed oracle use is limited to scene taxonomy reference, duration-profile
reference, expected prompt oracle, expected row oracle, and forbidden drift
tags. The KB crosswalk remains `candidate_not_exact_join`; it is not a KB rule
ID and must not be reported as an exact KB join.

Forbidden in runtime prompts, generated rows, logs, chat, and QA artifacts:
`sample_text`, storyboard row raw prose, raw `smoke_extracts`,
`source_sample_id`, sample character names, sample props, sample places, sample
worldview, raw KB rows, `source_register`, and overlay JSON.

## Scene Rewrite Contract Inheritance

Runtime and QA use of the 21 scene types must follow
`docs/hope-scene-type-rewrite-contract.md`.

The 21 reference samples remain structure-only. They may inform scene taxonomy,
duration profiles, craft tags, director-rule tags, negative drift guards, and
compact sample summaries, but they must not provide raw prose or sample facts to
the user story body.

Duration profiles are guidance for every product target duration, not a closed
set of sample rewrites. Current fixed values (`5`, `10`, `15`, `30`, `45`,
`60`) and future positive fixed values must be normalized into narrative
capacity bands. QA must fail if a scene/duration switch only changes stored
fields while the visible story body does not change expression or capacity.

Every scene type must use the same rule-pack mechanism. QA samples such as
`热血战斗 -> 场域追逐` are targeted probes only; they are not implementation
branches and do not authorize hard-coded scene-pair handling.
