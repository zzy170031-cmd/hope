# KB Golden Sample Intake Review 2026-04-22

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor before this memo: `6056210` (`docs: record RC publication confirmation smoke`)
- route: `RC_READY` plus scope freeze
- review type: control-thread acceptance of an independent `hope-kb` package

This memo is docs-only. It does not import KB data into `hope`, does not reopen
runtime integration, and does not promote V3/V4 fields to implementation.

## Reviewed KB Result

Reviewed repo:

- repo: `E:\codex\hope-kb`
- branch: `codex/contracts-freeze`
- commit: `21d5641` (`Normalize golden sample intake staging`)
- status: clean and aligned with `origin/codex/contracts-freeze`

Changed files in the KB commit:

- `docs/golden-sample-v0.2-schema-note-2026-04-22.md`
- `docs/live-progress.md`
- `docs/normalized-staging/golden-sample-library-642bd7876e0d4649814f35fb133b67f3-2026-04-22.normalized.json`

No seed bundle, manifest, import map, migration, runtime, desktop, or `hope`
file was changed by the KB package.

## Acceptance Decision

`KB_GOLDEN_SAMPLE_STAGING_ACCEPTED`

The KB thread made the correct boundary decision:

- the 40-row golden sample CSV was normalized as KB-only staging
- all 17 source fields were preserved
- all 40 rows were preserved
- no missing `source_cut` values were invented
- the data was not forced into the v0.1 `classic_case_examples` seed table

This preserves future few-shot, validator, negative-sample, and V3 core
semantics instead of flattening them into the wrong v0.1 schema.

## Counts And Hashes

Seed import status:

- `classic_case_examples`: 28 before / 28 after
- `degraded_input_examples`: 26 before / 26 after
- seed import: none

Bundle hash:

- before / after: `bundle-sha256:5f042c10ada3726bdbd71d5f4cbad2187b3895dd85e1e5195d6938a3a22d20b6`

Normalized staging artifact:

- `row_count`: 40
- actual normalized rows reviewed by controller: 40
- `field_order` count: 17
- `imported_into_v0_1_seed`: false
- `preserves_all_source_fields`: true

## Validation Evidence

The control thread reran KB validation after reviewing the package:

```text
powershell -ExecutionPolicy Bypass -File E:\codex\hope-kb\scripts\validate-seed-bundle.ps1
```

Result:

- `Seed bundle validation passed.`
- `classic_case_example`: 28
- `degraded_input_example`: 26

The control thread also reran the snapshot builder:

```text
python E:\codex\hope-kb\scripts\build-kb-snapshot.py --repo-root E:\codex\hope-kb
```

Result:

- builder passed
- generated ignored local artifact:
  `E:\codex\hope-kb\snapshots\hope-kb-v0.1.rebuilt-11.sqlite3`
- snapshot version: `v0.1`
- director cut samples: 35

The snapshot rebuild does not change Git state because SQLite snapshot outputs
are ignored artifacts.

## v0.2 Schema Gate Input

The KB result creates a concrete planning input for the V3/V4 field proposal,
but it does not authorize implementation.

Future v0.2 golden-sample schema planning must preserve at least:

- `sample_id`
- `sample_title`
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
- source CSV hash and import provenance

The v0.1 `classic_case_examples` schema must remain unchanged for this RC line.

## Five-Thread State After Review

Hope main:

- remains `RC_READY`
- continue publication confirmation and baseline protection
- no new product-code work opened by this review

Hope KB:

- package accepted at `21d5641`
- return to standby after golden-sample staging
- wait for future v0.2 schema gate feedback

V3/V4 field thread:

- next eligible work is docs-only proposal planning for a v0.2 golden-sample
  schema overlay
- must read the KB schema note and normalized artifact
- must not modify `hope` product code, exporter, IPC, Rust DTOs, validators,
  desktop UI, workbook, Qwen integration, or Seedance integration

Desktop:

- remains waiting at `66d687d`

Controlled intake:

- remains waiting at `d74bd11`

## Next Dispatch Candidate

Send the V3/V4 field thread a docs-only proposal task:

```text
Use existing V3 field thread `019db40b-4299-75c3-8ea7-d6b1ff3a8175`.

Do not implement product code. Do not modify exporter / IPC / Rust DTOs /
validators / desktop / workbook / Qwen / Seedance integration.

Review these KB outputs:
- E:\codex\hope-kb\docs\golden-sample-v0.2-schema-note-2026-04-22.md
- E:\codex\hope-kb\docs\normalized-staging\golden-sample-library-642bd7876e0d4649814f35fb133b67f3-2026-04-22.normalized.json

Update the V3/V4 proposal branch with a docs-only v0.2 golden-sample schema
overlay proposal. It must preserve the 17 CSV fields, few-shot eligibility,
validator coverage, negative-sample signals, V3 core coverage, timestamps, and
source provenance.

Keep this as proposal-only planning input. Commit and push, then report branch,
commit, changed files, and the proposed schema fields back to main control.
```
