# Desktop Anime Long Sample Library Plan

Status: guidance third-targeted-small-fix post-review candidate, still needs human re-review.
Thread: （运行）Hope桌面端-文本资源线程-【27样本指导性小修】
Repo: `E:\codex\hope-desktop-shell`.
Branch: `codex/desktop-shell`.
JSON index: `tests/qa/desktop-anime-scene-long-samples.index.json`.

## Review Verdict

- `review_verdict=not_ready_to_approve`
- `cannot_replace_human_review=true`
- `approved_samples=0`
- `revision_round=kb_aligned_27_sample_guidance_third_targeted_small_fix_2026_05_02`
- This package is still a QA reference candidate. It must not be marked approved until a later explicit human re-review accepts it.

## Library Counts

- `scene_index=21`
- `long_samples=27`
- Original scene samples retained: 21
- Supplemental samples retained: 6
- All 27 long samples are `qa_only=true`, `reference_only=true`, `not_source_of_truth=true`, and `raw_sample_text_for_runtime=false`.

## Word Count Strategy

The library no longer treats 3000+ characters as a mechanical requirement.

Shorter bodies are allowed when they already show the scene mechanics, visible action, camera/action rhythm, QA constraints, and checkable drift boundaries. Longer bodies are reserved for genuinely complex war formations, SLG layouts, group staging, spectacle staging, and chase/combat geography.

Padding with review notes, runtime boundary prose, field explanations, or validation claims is forbidden inside `sample_text`.

## Sample Text Boundary

`sample_text` is natural scene-facing prose only. It may describe visible staging, camera movement, action beats, subject continuity, and editing rhythm when those are part of the scene guidance. It must not contain audit recap text, validation conclusions, runtime-source claims, field names, raw extract policy, approval notes, or QA handoff language.

The current revision cleaned the top-level `sample_text` and `content.sample_text` bodies for all 27 samples. Runtime and QA boundary language now lives in structured fields and this document, not in the prose bodies.

## Structured Guidance Added

Every long sample now includes:

- `content.storyboard_rows[]` with Hope-facing row fields: `person`, `visual_description`, `character_action`, `camera_movement`
- `content.expected_prompt_oracle`
- `content.expected_row_oracle`
- `content.video_failure_criteria`
- `content.forbidden_drift_tags`
- synchronized top-level `storyboard_rows`, `expected_prompt_oracle`, `expected_row_oracle`, and `video_failure_criteria`

Rows are intentionally partial examples. They cover the key capability of the scene type rather than the full prose body.

## Smoke Extract Policy

`content.smoke_extracts` is no longer a list of raw body substrings. Each entry is a paraphrase/fact anchor with `anchor_id`, `kind`, `fact_ref`, `summary`, and `original_text_substring=false`.

Smoke anchors are QA comparison references only. They must not be joined into `prompt_text`, copied into generated output, or treated as source material.

## Urban Supplemental Rewrite

`sample_supplemental_urban_last_elevator_blank_floor_001` was rewritten away from the main urban sample.

The supplemental now focuses on a closed elevator cabin, floor looping, sound cues coming from the wrong level, delayed mirror reflection, repeated wheel marks, and a half-visible stair platform. It no longer reuses the main sample's file-room path as its core.

## KB Alias And IP Boundary

KB usage remains structure-only: craft pattern, visible action, director scheduling, camera movement, rhythm, scene taxonomy, compression anchors, and drift prevention.

The public-facing sample fields should expose only irreversible craft tags, director rule tags, scene taxonomy tags, and negative drift tags. `writer_*`, `director_*`, `work_*`, source IDs, and KB case IDs are internal trace anchors only. They must not enter runtime prompts and must never become imitation targets.

Modern references and `source_register` rows are structure-only. The library must not copy character names, worldviews, dialogue, plot events, recognizable expression, protected settings, brands, or proprietary text. Public-domain classics remain secondary structure references only.

## Runtime Boundary

Runtime may consume only compact structured abstractions when explicitly routed by future work:

- sample ID and taxonomy tags
- compact sample summary
- craft/rule tags
- camera and motion anchors
- duration/compression anchors
- checkable negative drift tags

Runtime must not consume:

- prose body text
- smoke anchor summaries as prompt prose
- sample character names, props, places, or worldview
- raw KB rows or source-register payloads
- real-person, director, author, work, brand, or IP imitation targets
- modern copyrighted text or recognizable expression

The sample library must never override user source, accepted snapshot, StoryFactFrame, scene type, or duration.

## Validation Snapshot

Current mechanical checks are scoped and do not claim whole-file marker cleanliness:

- JSON parses successfully.
- `scene_index=21`, `long_samples=27`, `supplemental=6`.
- all runtime flags are green: `qa_only=true`, `reference_only=true`, `not_source_of_truth=true`, `raw_sample_text_for_runtime=false`.
- top-level `sample_text` and `content.sample_text` are synchronized for all 27 samples.
- scoped `sample_text` marker scan is clean for the required audit/runtime/meta phrases.
- all 27 samples include storyboard rows, prompt oracle, row oracle, and video failure criteria.
- smoke anchors are paraphrase/fact anchors and are not raw body substrings.
- `sample_supplemental_urban_last_elevator_blank_floor_001` is distinct from `sample_urban_fantasy_001` in characters, setting, motion mechanics, and failure tags.

This validation is not approval. It is only evidence that the current candidate is ready for human re-review.

## Next Step

Human re-review of all 27 long samples remains required before any approval metadata, runtime consumption change, 403-case, packaging, push, or release step.

关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。
