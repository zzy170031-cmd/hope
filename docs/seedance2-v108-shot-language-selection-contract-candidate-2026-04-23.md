# Seedance2 V108 Shot-Language Selection Contract Candidate 2026-04-23

## Route

This is a docs-only shot-language selection contract candidate for the Hope
main control thread.

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor before this memo:
  `60ce117` (`docs: dispatch V108 shot-language selection`)
- accepted validator evidence review:
  `39119d6` (`docs: accept V108 validator evidence contract`)
- accepted KB director-style expansion:
  `434c74a` (`docs: add V108 director style expansion research`)
- package type: docs-only V108 shot-language / retrieval-selection planning
  contract candidate

This candidate does not implement runtime selection. It does not generate final
storyboard words, runtime prompt text, Qwen messages, Seedance overlay wording,
exporter rows, workbook fields, validator behavior, repair behavior, product
import behavior, UI taxonomy, or `reference_control_core`.

## Reviewed Inputs

Hope-side inputs:

- `docs/seedance2-v108-shot-language-selection-contract-dispatch-2026-04-23.md`
- `docs/seedance2-v108-director-style-expansion-control-review-2026-04-23.md`
- `docs/seedance2-v108-director-style-coverage-control-review-2026-04-23.md`
- `docs/seedance2-v108-validator-evidence-contract-review-2026-04-23.md`
- `docs/seedance2-v108-validator-evidence-contract-candidate-2026-04-23.md`
- `docs/generation-field-rule-contract-review-2026-04-22.md`
- `docs/generation-field-rule-contract-candidate-2026-04-22.md`
- `docs/seedance2-v108-vnext-schema-decision-2026-04-23.md`

KB-side inputs, read only from `E:\codex\hope-kb`:

- `docs/seedance2-v108-director-style-web-research-2026-04-23.md`
- `seed/v0.1/director_profiles.json`
- `seed/v0.1/director_rules.json`
- `seed/v0.1/director_reference_sets.json`
- `seed/v0.1/director_scene_affinity.json`

## Candidate Decision

`SEEDANCE2_V108_SHOT_LANGUAGE_SELECTION_CONTRACT_CANDIDATE_FROZEN_DOCS_ONLY`

Hope may use this candidate as the planning contract for how a future selector
could choose camera-language evidence from multiple V108 content candidates and
convert that evidence into bounded storyboard-word planning signals.

This candidate is not a runtime selector, not a product writer, not a model
adapter, not a prompt template, not a validator, and not an exporter contract.

## Required Assertions

The accepted V108 assertions remain unchanged:

```text
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
```

Additional closed-scope assertions remain unchanged:

```text
v108_rows_imported_into_product_structure = 0
positive_fewshot_promotions = 0
qwen_calls = 0
seedance_calls = 0
v3_branch_edits = 0
hope_product_code_changes = 0
desktop_or_intake_changes = 0
rust_dto_validator_exporter_workbook_ipc_changes = 0
```

The accepted KB director-style expansion changes the internal style-lane asset
count from `7` to `11`, but it does not import V108 rows, promote positive
few-shot, create product-ready references, or open product implementation.

## Planning Objects

Future planning should preserve these object names:

- `V108ShotLanguageSelectionContract`
- `V108ShotLanguageSourceBundle`
- `V108StyleLaneSelectionInput`
- `V108CameraLanguageSelectionRule`
- `V108StoryboardWordPlanningSignal`
- `V108ShotLanguageSelectionBlocker`
- `V108ShotLanguageSelectionReadiness`
- `V108ShotLanguageFutureGateOrder`

These are planning names only. They are not Rust types, database tables,
runtime enums, failure codes, severity levels, thresholds, IPC payloads,
workbook fields, exporter fields, Qwen templates, Seedance instructions, or UI
labels in this gate.

## Source Bundle Boundary

`V108ShotLanguageSourceBundle` may read the V108 source row only as raw
planning evidence:

- content intent from source row text and source labels
- `style_cluster`
- `scene_category`
- `scene_tag`
- `camera_directing_core`
- sequence context from `sample_type`, `sequence_id`, and `shot_order`
- evidence quality from `covered_points`, `missed_points`, `teaching_note`,
  `ip_abstraction_note`, and `continuity_negative_core`
- blocker state from placeholder, unresolved reference, V3 alignment,
  schema, reserve, and promotion gates

It may also reference the accepted eleven internal style lanes as retrieval and
camera-language metadata. It must not expose lane names as product-facing copy,
visible UI taxonomy, model prompt wording, Seedance overlay wording, exporter
fields, workbook fields, validator enums, repair rules, external reference
handles, or `reference_control_core`.

## Eleven Internal Style Lanes

The current internal lane set is:

| lane | internal name | primary planning use |
| --- | --- | --- |
| `director_01` | `今石洋之` | broad heat, explosive action, impact rhythm, high-energy escalation |
| `director_02` | `荒木哲郎` | broad crowd pressure, crisis depth, compressed space, danger advance |
| `director_03` | `朴性厚` | close-combat clarity, body axis, impact point, readable attack chain |
| `director_04` | `新海诚` | emotional landscape, light/weather mood, reflective transition |
| `director_05` | `山田尚子` | micro-performance, quiet relationship beats, detail observation |
| `director_06` | `汤浅政明` | subjective deformation, emotional distortion, dream/psychological layer |
| `director_07` | `今敏` | suspense, reality displacement, match cut, cognitive transition |
| `director_08` | `内部风格通道：国风武侠/仙侠动作调度` | guofeng wuxia/xianxia action, weapon continuity, qinggong movement axis |
| `director_09` | `内部风格通道：国漫史诗群像/战争场面调度` | Chinese-animation epic crowd, war oath, army formation, battlefield depth |
| `director_10` | `内部风格通道：都市末世/工业压迫调度` | urban apocalypse, infrastructure pressure, evacuation axis, alarm rhythm |
| `director_11` | `内部风格通道：国潮舞台/原创表演调度` | guochao stage, group dance, music beat, performer entrance, final freeze |

The first seven lanes remain the default background model when the row asks for
broad action, crowd pressure, close combat, emotional landscape, daily
micro-performance, subjective deformation, suspense, or transition behavior
without a V108-specific Chinese-animation gap.

The four added lanes intervene only when `style_cluster`, `scene_category`,
`camera_directing_core`, or source evidence indicates that the original seven
lanes are too broad for the V108 row's concrete shot-language problem.

## Selection Inputs

`V108StyleLaneSelectionInput` should consider:

- `style_cluster`: broad source style family, never final product taxonomy
- `scene_category`: scene family evidence, not runtime routing by itself
- `scene_tag`: optional row-local cue, not a canonical registry value
- `camera_directing_core`: strongest source for shot-language intent
- `sample_type`: `single_shot` or `sequence_shot`
- `sequence_id` and `shot_order`: sequence continuity evidence only when present
- `covered_points`: what source evidence claims is covered
- `missed_points`: what source evidence says is still missing
- `teaching_note`: human review rationale
- `ip_abstraction_note`: abstraction/compliance risk signal
- `continuity_negative_core`: continuity locks, negative constraints, empty-word
  or risk evidence

Selection must be conservative. If the source fields are placeholder-heavy,
reference-unresolved, schema-missing, or promotion-blocked, the result remains
raw evidence only.

## Selection Matrix

| content / evidence situation | lane routing plan | allowed planning signal | blocked output |
| --- | --- | --- | --- |
| broad hot-blooded action without V108 guofeng/wuxia markers | original `director_01` plus optional `director_03` | action intensity, impact rhythm, readable attack chain | final fight wording, runtime prompt text |
| close 1v1 or hand-to-hand action where body axis is primary | original `director_03` | body axis, impact point, attack-chain clarity | fake choreography, copied shot text |
| broad crowd pressure or danger advance without Chinese war/crowd-specific evidence | original `director_02` | crowd pressure, space compression, danger advance | final crowd scene prose |
| emotional landscape / reflective transition | original `director_04` | environment-emotion support, light/weather transition | final emotional storyboard words |
| micro-performance or quiet relationship beat | original `director_05` | detail observation, low-viewpoint micro action | dialogue or character acting output |
| dream, subjectivity, psychological distortion | original `director_06` | subjective deformation / emotional layer planning | surreal prompt text |
| suspense, cognitive mismatch, match-cut transition | original `director_07` | match-cut / reality-displacement planning | runtime suspense script |
| `国漫 / 热血打斗` with guofeng, wuxia, xianxia, weapon, qinggong, bamboo, eave, rain, or blade continuity evidence | consider `director_08`; handoff to `director_01` or `director_03` when generic impact or close-combat clarity dominates | weapon continuity, qinggong movement axis, guofeng space relation, burst-and-landing rhythm | generated combat wording, real director imitation |
| `国漫 / 场域追逐` with roof-eave, bamboo, vertical traversal, chase axis, weapon/robe/rain continuity | consider `director_08`; handoff to `director_02` when pressure-space routing dominates | chase axis, vertical movement bridge, action landing, spatial anchor | runtime chase prompt |
| `国漫 / 群像表演` with war oath, army formation, command hierarchy, charge staging, crowd rhythm | consider `director_09`; handoff to `director_02` for generic crisis pressure | formation depth, command center, flag/drum bridge, crowd rhythm | final war or crowd prose |
| `原创 / 群像表演` with stage, group dance, entrance timing, music beat, spotlight, final freeze | consider `director_11`; handoff to `director_05` or `director_04` for quiet relationship or emotional landscape beats | stage axis, formation continuity, music-sync cue, freeze-frame planning | final performance wording |
| urban-apocalypse / industrial-pressure rows with subway, overpass, ruins, evacuation, alarm, infrastructure pressure | consider `director_10`; handoff to `director_02` for broad crisis pressure or `director_07` for suspense/transition | evacuation axis, industrial compression, alarm bridge, obstacle source | final disaster/action script |
| row has placeholder-bearing `camera_directing_core` or prompt / source body | no lane promotion; raw evidence only | blocker note: `placeholder_present` or `prompt_body_blocked_by_placeholder` | selection, prompt, few-shot, import |
| row depends on unresolved `reference_bundle` | no product reference selection; raw reference evidence only | blocker note: `reference_handle_unresolved` | external reference handles, `reference_control_core` |
| row conflicts with V3 meanings or fused fields are not split | raw V108 evidence; optional `v3_alignment_gap` planning note | alignment gap note | DTO field write, old V3 semantic reuse |
| schema/import/promote gates are unopened | no runtime use | blocker note: `schema_field_missing` / `promotion_gate_not_accepted` | product import, positive few-shot, exporter |

## Original Seven-Lane Sufficiency Rule

The original seven lanes are enough when:

- the source row asks for a broad camera-language behavior already covered by
  the background model
- `style_cluster` and `scene_category` do not point to the newly accepted V108
  gaps
- `camera_directing_core` does not require guofeng weapon continuity, Chinese
  war-scale crowd staging, industrial apocalypse routing, or stage/music
  performance blocking
- selection is only needed as internal metadata, not product wording
- blockers keep the row raw and do not require any new lane to rescue it

The selector should not prefer a new lane just because the row says `国漫` or
`原创`. It needs a concrete shot-language need: action axis, crowd blocking,
industrial route, or stage/performance rhythm.

## New Lane Intervention Rule

`director_08` should be considered when V108 evidence combines Chinese-style
action with readable weapon continuity, qinggong or roof/bamboo traversal,
burst-pause rhythm, guofeng space, or blade/robe/rain hard locks.

`director_09` should be considered when V108 evidence requires group hierarchy,
war oath, army formation, battlefield depth, command center, flag/drum bridge,
or crowd-motion rhythm.

`director_10` should be considered when V108 evidence locates danger in
infrastructure failure, subway/overpass/ruins, evacuation blocking, alarm
layers, industrial passage compression, or future battlefield pressure.

`director_11` should be considered when V108 evidence requires stage axis,
performer entrance, synchronized group movement, music beat, spotlight timing,
formation continuity, or final-freeze staging.

The four lanes are not fallback defaults. They are precision lanes for V108
content gaps after source evidence is present and blockers are recorded.

## Boundary Between Metadata, Signals, Words, And Prompts

Internal retrieval metadata is allowed in this gate. It may include lane IDs,
source fields, affinity hints, scene family, and blocker families for future
planning review.

Camera-language planning signal is allowed in this gate. It may state bounded
planning concepts such as `weapon_continuity`, `formation_depth`,
`evacuation_axis`, `stage_axis`, `music_sync_cue`, `movement_landing`,
`match_cut_candidate`, or `crowd_pressure`.

Final storyboard words are closed. This gate must not write the actual
storyboard sentence, shot description, action line, dialogue, beat prose, or
scene text.

Runtime prompt text is closed. This gate must not write Qwen messages,
Seedance prompts, compiled prompt packages, prompt templates, negative prompts,
or model-ready instructions.

## Blocker Semantics

`V108ShotLanguageSelectionBlocker` preserves these blockers:

- `placeholder_present`
- `prompt_body_blocked_by_placeholder`
- `reference_handle_unresolved`
- `v3_alignment_gap`
- `schema_field_missing`
- `promotion_gate_not_accepted`
- `validator_evidence_not_defined`
- `reserve_row`

Blockers do not delete source rows. They keep rows available as raw evidence
while preventing product import, positive few-shot, final words, runtime prompt
text, product-ready references, validator implementation, repair behavior, and
exporter/workbook changes.

## Readiness Matrix

| planning surface | current readiness | reason |
| --- | --- | --- |
| internal style-lane baseline | ready for planning | KB seed now has 11 accepted internal lanes. |
| shot-language selection contract | candidate only | This memo defines selection planning, not runtime selection. |
| camera-language planning signal | allowed as docs-only planning | Signals remain abstract and non-final. |
| final storyboard words | closed | No writer/orchestrator gate is open. |
| runtime prompt text | closed | Qwen and Seedance remain closed. |
| product import | blocked | `rows_ready_for_product_import = 0`. |
| positive few-shot | blocked | `rows_ready_for_positive_fewshot = 0`. |
| external reference handles | blocked | `product_ready_external_reference_handles = 0`. |
| `reference_control_core` | closed | `reference_control_core_coverage = 0`. |

## Future Gate Order

Future work must follow this order unless main control opens a separate
docs-only gate:

1. Accept or reject this `V108ShotLanguageSelectionContract` candidate.
2. If accepted, run a KB-only canonical object registry readiness gate before
   any product-facing reference handle selection.
3. Run a retrieval-selection implementation scope gate naming exact files and
   forbidden files before any DTO, selector, fixture, or debug metadata work.
4. Run validator evidence implementation scope only after selector and
   validator evidence boundaries are both accepted.
5. Run writer/orchestrator and Qwen prompt assembly gates separately before
   final storyboard words or runtime prompts can exist.
6. Run Seedance/exporter/workbook gates separately; current exporter and
   workbook behavior remain frozen.
7. Run desktop/intake gates separately; this candidate does not wake those
   threads.

No gate may skip from internal retrieval metadata directly to final storyboard
words, runtime prompt text, product import, positive few-shot, external
reference handles, exporter output, or `reference_control_core`.

## Explicitly Closed

This candidate does not authorize:

- Rust DTO changes
- validator implementation
- repair implementation
- exporter behavior changes
- workbook shape changes
- IPC changes
- desktop changes
- intake changes
- `hope-kb` edits
- V3 branch edits
- Qwen calls
- Seedance calls
- final storyboard word generation
- runtime prompt text
- V108 row import
- positive few-shot promotion
- `reference_control_core` creation
- invented external references, character appearances, scene designs, or
  registry data
- product-facing style-lane labels
- instructions to imitate any real director

## Candidate Completion

This candidate is complete when committed and pushed to
`codex/contracts-freeze`.

Recommended next label:

```text
Hope主线-Seedance2V108ShotLanguageSelection【候选已完成·待接收】
```

Validation for this package is docs-only. No code tests are required unless a
later gate opens implementation.
