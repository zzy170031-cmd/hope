# Golden Sample v0.2 Validation And Snapshot Readiness 2026-04-22

This note records the validation posture for the v0.2
`golden_sample_library` schema package.

## Scope

- repo: `E:\codex\hope-kb`
- branch: `codex/contracts-freeze`
- source checkpoint before this package: `21d5641`
- source artifact:
  `docs/normalized-staging/golden-sample-library-642bd7876e0d4649814f35fb133b67f3-2026-04-22.normalized.json`

This package does not modify `hope`, desktop, intake, Qwen, Seedance, or the
v0.1 snapshot contract.

## v0.2 Seed Files

- `seed/v0.2/golden_sample_library.json`
- `seed/v0.2/golden_sample_field_coverage_rules.json`
- `seed/v0.2/golden_sample_failure_mapping.json`
- `seed/v0.2/golden_sample_repair_mapping.json`
- `seed/v0.2/source_register.json`

## Validation Result

The v0.2 structure was checked for:

- `golden_sample_library` has `40` records
- every library record preserves all `17` source fields under `source_fields`
- `golden_sample_field_coverage_rules` has `5` records
- `golden_sample_failure_mapping` has `40` records
- `golden_sample_repair_mapping` has `40` records
- `source_register` has `3` source entries and `1` provenance entry
- few-shot eligibility remains `Yes=30 / No=10`
- negative-sample signal remains `10` records

Result: `passed`.

## Snapshot Readiness

The v0.2 package is seed/schema ready but not snapshot-import ready.

It intentionally does not change:

- `seed/v0.1/manifest.json`
- `seed/v0.1/import_map.json`
- `migrations/0001_init_kb.sql`
- `snapshots/hope-kb-v0.1.sqlite3`
- `scripts/build-kb-snapshot.py`

A future v0.2 snapshot gate still needs:

- a v0.2 import map
- a v0.2 migration/table definition
- a v0.2 manifest and content hash
- a v0.2 snapshot builder or explicit versioned builder mode
- Hope-side validator/export/desktop gates before product consumption

## Fields Needing Future Hope Gates

The following fields are preserved in KB but still need downstream planning
before Hope product use:

- `core`
- `covered_points`
- `missed_points`
- `tier`
- `usable_for_fewshot`
- `empty_words_detected`
- `director_voice`
- `genre`
- `scene_tag`
- `shot_type`
- `source_cut`
- `teaching_note`
- `word_count`
- `created_at`
- `updated_at`

These fields require future validator/export/desktop gates before any runtime or
UI consumption.
