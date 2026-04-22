# Generation Field Rule Contract Review 2026-04-22

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- reviewed commit: `84fbdf5` (`docs: freeze generation field rule candidate`)
- previous planning anchor: `94e3933` (`docs: plan qwen kb v3 field rules`)
- route: `RC_READY` plus controlled v0.2 readiness planning
- review type: main-control acceptance of docs-only generation field rule contract candidate

This review is docs-only. It does not open product implementation.

## Reviewed Result

Changed file in `84fbdf5`:

- `docs/generation-field-rule-contract-candidate-2026-04-22.md`

No Rust DTO, validator, exporter, workbook, IPC, desktop, intake, Qwen,
Seedance, V3 proposal branch, or `hope-kb` file changed.

## Acceptance Decision

`GENERATION_FIELD_RULE_CONTRACT_CANDIDATE_ACCEPTED`

Main control accepts the candidate as the canonical docs-only field-rule
contract for controlled v0.2 readiness planning.

Accepted points:

- It freezes the generation phase list from input preflight through export
  adapter planning.
- It defines `allowed_write_fields` and `forbidden_fields` for every phase.
- It routes each phase to bounded KB sources instead of allowing broad KB
  leakage into runtime prompts.
- It defines golden-sample positive retrieval and exclusion rules.
- It keeps negative or unusable samples out of positive few-shot prompt context.
- It limits V3 core writing to the five covered cores:
  `visual_scene_core`, `motion_performance_core`, `camera_directing_core`,
  `audio_directing_core`, and `continuity_lock_core`.
- It preserves `reference_control_core` as uncovered and forbids invented
  coverage.
- It maps future validator gate ownership without defining implementation
  thresholds or Rust types.
- It scopes repair to validator-named fields and forbids unrelated rewrites.
- It preserves the future implementation gate order before any writer,
  orchestrator, validator, intake/Qwen, exporter, desktop, or Seedance work.

## Remaining Closed Gates

This acceptance does not authorize:

- Rust DTO changes
- validator implementation
- repair implementation
- writer or orchestrator implementation
- exporter behavior changes
- workbook changes
- IPC changes
- desktop changes
- intake changes
- Qwen calls or prompt assembly implementation
- Seedance calls or adapter implementation
- `hope-kb` edits
- V3 branch edits
- merging `hope-kb` into `hope`
- importing golden samples into v0.1 assets
- using negative samples as positive few-shot examples
- inventing `reference_control_core` coverage

## Next Control State

The current state is:

- Hope main: `RC_READY` plus controlled v0.2 readiness planning.
- KB: accepted v0.2 snapshot-import readiness at `818c098`, waiting.
- V3: accepted freeze candidate at `69c645c`, archived / waiting.
- Desktop: `66d687d`, waiting.
- Intake: `d74bd11`, waiting.
- Qwen and Seedance: closed.

If the user continues, the next eligible docs-only gate is one of:

1. `KB support update if needed`, only if the field-rule contract exposes missing
   KB metadata or new sample requirements.
2. `V3 proposal refresh if needed`, only after a KB update is completed and
   accepted by main control.
3. `Hope validator evidence contract`, if no KB/V3 update is needed and main
   control explicitly opens validator planning.

Do not proceed directly to implementation from this acceptance.

## Thread Label

Recommended next label:

```text
Hope主线-GenerationFieldRuleContract【候选已接收·待下一Gate】
```

关键节点提醒：请立即刷新线程标签、锚点提交、工作树状态和边界说明。
