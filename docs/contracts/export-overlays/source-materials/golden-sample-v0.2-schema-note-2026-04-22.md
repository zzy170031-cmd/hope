# Golden Sample v0.2 Schema Note 2026-04-22

This note records why the 2026-04-22 golden sample CSV was not imported into
the current v0.1 `classic_case_examples` seed table.

## Source

- source CSV: `docs/source-exports/golden-sample-library-642bd7876e0d4649814f35fb133b67f3-2026-04-22.csv`
- normalized staging artifact: `docs/normalized-staging/golden-sample-library-642bd7876e0d4649814f35fb133b67f3-2026-04-22.normalized.json`
- source rows: `40`
- source fields: `17`
- source hash: `sha256:2d0ce22f8c85ca59de6abf35a48239aadef9a2bd01bb3f7b4685bf80e06bf843`

## Decision

The CSV is staged as a lossless KB-side normalized artifact only.

It is not imported into `seed/v0.1/classic_case_examples.json`, because the
current v0.1 schema would flatten or discard golden-sample semantics that are
needed for future few-shot and validator use.

## v0.1 Mapping Risk

`classic_case_examples` currently stores broad benchmark/case metadata:

- case title
- case type
- target duration
- recommended committee
- recommended director combination
- scene summary
- structure hint
- validation value
- source metadata

The golden sample library stores row-level training and validation semantics:

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
- source timestamps

Forcing these rows into the v0.1 case table would lose the distinction between
positive few-shot rows, negative examples, validator coverage, V3 core coverage,
and teaching notes.

## v0.2 Gate Needed

A future v0.2 gate should add a dedicated golden-sample schema before seed
import. The minimum required shape should preserve:

- stable `sample_id`
- source title and source timestamps
- core axis (`visual_scene_core`, `motion_performance_core`,
  `camera_directing_core`, `audio_directing_core`, `continuity_lock_core`)
- tier and few-shot eligibility
- covered points and missed points as first-class validator evidence
- empty-word detection as negative-sample signal
- director voice, genre, scene tags, and shot type
- source cut, including intentional empty values
- teaching note and word count
- source CSV hash and import provenance

Until that gate exists, the normalized artifact remains staging-only and should
not be consumed as a v0.1 runtime snapshot table.
