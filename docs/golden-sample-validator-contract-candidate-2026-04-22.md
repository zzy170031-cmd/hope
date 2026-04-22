# Golden Sample Validator Contract Candidate 2026-04-22

## Route

This is a docs-only validator contract planning memo for the Hope main thread.

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- planning anchor: `29f4e88` (`docs: dispatch golden sample validator planning`)
- route: `RC_READY` plus controlled v0.2 readiness planning
- decision scope: validator / repair / exporter-debug metadata planning only

This memo does not open implementation. It does not modify Rust DTOs,
validators, exporters, workbook shape, desktop UI, intake, Qwen, Seedance,
`hope-kb` seed files, or the v0.1 snapshot contract.

## Reviewed Inputs

Hope-side inputs:

- `docs/golden-sample-validator-planning-dispatch-2026-04-22.md`
- `docs/kb-golden-sample-v0.2-package-review-2026-04-22.md`
- `docs/v3-golden-sample-contract-freeze-review-2026-04-22.md`
- `docs/v0.2-scope-gate-readiness-2026-04-22.md`

KB-side inputs:

- `E:\codex\hope-kb\docs\golden-sample-v0.2-validation-readiness-2026-04-22.md`
- `E:\codex\hope-kb\seed\v0.2\golden_sample_library.json`
- `E:\codex\hope-kb\seed\v0.2\golden_sample_failure_mapping.json`
- `E:\codex\hope-kb\seed\v0.2\golden_sample_repair_mapping.json`
- `E:\codex\hope-kb\seed\v0.2\source_register.json`
- `E:\codex\hope-kb\seed\v0.2\golden_sample_field_coverage_rules.json`

Referenced package anchors:

- KB v0.2 package: `445c452` (`Add golden sample v0.2 schema package`)
- V3 freeze candidate: `69c645c` (`docs: promote golden sample overlay to freeze candidate`)

## Candidate Decision

`GOLDEN_SAMPLE_VALIDATOR_CONTRACT_CANDIDATE_PLANNED_KEEP_SEPARATE`

The v0.2 `golden_sample_library` may become future validator evidence, repair
planning input, exporter/debug metadata, and Qwen retrieval metadata only after
separate implementation gates.

It is not product-consumable yet because v0.2 snapshot import readiness remains
open.

## Evidence Package Shape

The accepted KB v0.2 package currently provides:

- `golden_sample_library`: 40 records
- 5 V3 core groups, 8 rows each:
  - `visual_scene_core`
  - `motion_performance_core`
  - `camera_directing_core`
  - `audio_directing_core`
  - `continuity_lock_core`
- tier distribution: 10 each for `优`, `好`, `中`, `差`
- few-shot eligibility: 30 positive candidates, 10 excluded samples
- negative sample signal: 10 records
- source preservation: all 40 rows and all 17 source fields retained

The 17 preserved source fields are:

- `sample_title`
- `core`
- `covered_points`
- `created_at`
- `director_voice`
- `empty_words_detected`
- `genre`
- `missed_points`
- `sample_id`
- `scene_tag`
- `shot_type`
- `source_cut`
- `teaching_note`
- `tier`
- `updated_at`
- `usable_for_fewshot`
- `word_count`

## Validator Evidence Use

Future Hope validator use should treat `golden_sample_library` as evidence, not
as model output and not as exporter sheet data.

Candidate validator behavior:

- use `core` to route a sample to the relevant validator family
- use `covered_points` as positive coverage evidence
- use `missed_points` as gap evidence
- use `empty_words_detected` as vague-language / empty-word evidence
- use `word_count` as debug and heuristic metadata, with thresholds deferred to
  a later validator implementation gate
- use `tier` as quality-band metadata and negative-sample support
- use `usable_for_fewshot` only to govern retrieval eligibility, not validation
  pass/fail by itself
- use `negative_sample.is_negative_sample` to exclude positive few-shot retrieval
  and to provide explicit validator negative examples
- use provenance fields only for traceability and debug metadata

The validator should not treat a golden sample as a mandatory pass/fail answer.
It should use the record as a comparison fixture and evidence source for future
coverage, empty-word, negative-sample, and provenance checks.

## Validator Signal Families

Candidate Hope validator signal families:

- `golden_coverage_gap`
  Triggered by non-empty or meaningful `missed_points`, especially when a core's
  expected checklist is materially incomplete.
- `golden_empty_word_noise`
  Triggered by non-empty `empty_words_detected` or vague-language evidence.
- `golden_negative_sample`
  Triggered by `negative_sample.is_negative_sample = true`, `tier = 差`, or
  `usable_for_fewshot = No`.
- `golden_unusable_fewshot`
  Hope-side planning signal for records that must be excluded from positive
  retrieval even if they remain useful as validator negative evidence.
- `golden_source_provenance_gap`
  Hope-side planning signal for missing or inconsistent source references,
  source hashes, row indexes, or provenance entries.

The KB failure mapping currently defines these concrete failure codes:

- `golden_coverage_gap`
- `golden_empty_word_noise`
- `golden_negative_sample`

The other two families remain Hope-side validator planning candidates until a
future implementation gate defines concrete failure codes.

## Field-Level Future Use

`covered_points`

- positive evidence for what a sample already satisfies
- future debug metadata for showing why a row is considered a good reference
- should not be used alone to mark generated output valid

`missed_points`

- primary source for `golden_coverage_gap`
- future repair input for targeted checklist completion
- should stay inspectable in debug metadata

`empty_words_detected`

- primary source for `golden_empty_word_noise`
- future validator evidence for empty/vague prompt wording
- future repair input for replacing generic words with concrete anchors

`word_count`

- future heuristic and debug metadata
- may help detect suspiciously thin or bloated samples after a separate threshold
  gate
- must not define pass/fail without a future validator threshold decision

`tier`

- quality-band metadata
- supports negative-sample classification and review sorting
- should not alone override explicit `usable_for_fewshot` or negative-sample
  signals

`usable_for_fewshot`

- controls positive retrieval eligibility
- `No` records must be excluded from positive few-shot prompt context
- excluded records can still be used as validator negative examples

`negative_sample`

- explicit negative fixture signal
- must never enter positive few-shot retrieval
- may enter validator negative-example checks and repair planning

## Failure Mapping Candidate

The v0.2 failure mapping has 40 records and maps planned evidence to failure
codes.

Current distribution:

- `golden_coverage_gap`: 26 mapped appearances
- `golden_empty_word_noise`: 14 mapped appearances
- `golden_negative_sample`: 10 mapped appearances

Future Hope validator behavior should:

- load failure mapping only after v0.2 snapshot import is defined
- preserve the one-to-one relationship between `sample_id` and mapping records
- report the mapped failure family as evidence metadata, not as generated
  content
- keep mapping source refs visible for debug traceability

## Repair Mapping Candidate

The v0.2 repair mapping has 40 records and remains `planning_only = true`.

Current repair planning modes:

- `positive_fewshot_candidate`: 14 records
- `gap_repair_reference`: 16 records
- `negative_fixture_exclude_from_positive_fewshot`: 10 records

Current future repair gates:

- `fewshot_retrieval_gate`: 14 records
- `coverage_gap_repair_gate`: 16 records
- `validator_negative_fixture_gate`: 10 records

Future repair planning should consume:

- `missed_points`
- `empty_words_detected`
- `teaching_note`
- `director_voice`
- `shot_type`
- linked failure mapping IDs

It must not auto-rewrite prompts, Qwen requests, workbook rows, or exporter
fields before a separate repair implementation gate.

## Exporter And Debug Metadata Impact

Exporter behavior remains closed.

Future exporter/debug planning may add read-only provenance metadata after a
separate gate, such as:

- golden sample `sample_id`
- validator signal family
- source SHA / source row index
- coverage evidence summary
- repair planning mode

This future debug metadata must not:

- add workbook sheets under the current v0.1 contract
- change the 17-sheet workbook
- change Excel exporter behavior
- promote v0.2 seed/schema into a frozen workbook contract
- leak KB internal rows into user-facing export output without a separate gate

## Fixture And Benchmark Impact

Expected future implementation impact:

- add targeted validator fixtures for positive few-shot candidates, gap samples,
  empty-word samples, and negative samples
- add no-op fixture checks that v0.1 RC exports remain unchanged
- add golden-sample debug metadata fixtures only after exporter/debug metadata
  scope is opened
- add retrieval-boundary fixtures only after intake / Qwen retrieval scope is
  opened

Expected non-impact for this memo:

- no benchmark fixture refresh
- no workbook fixture refresh
- no exporter output change
- no runtime snapshot import

## Future Gate Order

Future gates should run in this order:

1. `v0.2 snapshot import readiness`
   Define import map, migration/table shape, manifest/content hash, and versioned
   snapshot builder mode.
2. `Hope validator evidence contract`
   Define Rust/domain DTOs and validator behavior for golden coverage,
   empty-word, negative-sample, unusable-fewshot, and provenance-gap signals.
3. `Hope repair mapping contract`
   Define how failure mapping and repair mapping become repair suggestions or
   planning evidence.
4. `Exporter/debug metadata planning`
   Decide whether and how read-only provenance metadata may appear without
   changing the v0.1 workbook contract.
5. `Desktop read-only provenance planning`
   Define any UI-only provenance display. No KB internal data should be exposed.
6. `Intake / Qwen retrieval boundary`
   Define positive few-shot retrieval and negative-sample exclusion before any
   prompt/model integration.

## Unresolved Gaps

`reference_control_core`

- still has no golden-sample coverage
- must remain open as a future intake/schema-extension gap
- must not be invented from current rows

v0.2 snapshot import readiness

- no v0.2 import map yet
- no v0.2 migration/table definition yet
- no v0.2 manifest/content hash yet
- no v0.2 snapshot builder or explicit versioned builder mode yet

Product implementation gaps:

- Hope validator contract remains closed
- Hope repair implementation remains closed
- exporter/debug metadata remains closed
- desktop read-only provenance remains closed
- intake / Qwen retrieval boundary remains closed
- Qwen and Seedance integration remain closed

## Non-Implementation Boundary

This memo does not authorize:

- Rust DTO changes
- validator implementation
- repair implementation
- exporter behavior changes
- workbook sheet changes
- IPC changes
- desktop UI changes
- intake UI changes
- Qwen prompt/model calls
- Seedance adapters
- `hope-kb` seed edits
- v0.1 seed, manifest, import map, migration, or snapshot contract changes

## Completion State

This docs-only candidate is complete when committed and pushed on
`codex/contracts-freeze`.

Recommended completion label:

`GOLDEN_SAMPLE_VALIDATOR_CONTRACT_CANDIDATE_DOCS_ONLY_COMPLETE`
