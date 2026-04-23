# Seedance2 V108 Retrieval Selection Implementation Scope Dispatch 2026-04-23

## Dispatch Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- package type: docs-only main-thread implementation scope dispatch
- accepted shot-language selector contract:
  `docs/seedance2-v108-shot-language-selection-contract-review-2026-04-23.md`
- accepted registry schema proposal:
  `docs/seedance2-v108-canonical-object-registry-schema-proposal-control-review-2026-04-23.md`
- accepted validator evidence contract:
  `docs/seedance2-v108-validator-evidence-contract-review-2026-04-23.md`

The planning contracts are now sufficient to scope an internal retrieval /
shot-language selection implementation candidate. This dispatch does not open
implementation yet. It asks the main thread to name exact files, tests, and
boundaries for the next code gate.

## Thread Label

```text
Hope主线-Seedance2V108RetrievalSelectionScope【实现范围候选执行中】
```

## Sync Requirement

Before working, use the fixed Git path:

```text
& 'C:\Program Files\Git\cmd\git.exe' -C E:\codex\hope pull --ff-only
& 'C:\Program Files\Git\cmd\git.exe' -C E:\codex\hope status --short --branch
```

Expected repo:

```text
E:\codex\hope / codex/contracts-freeze
```

## Task

Create a docs-only implementation scope candidate for V108 internal retrieval /
shot-language selection.

The candidate must identify the exact files and tests that would be touched in
a later implementation gate. It must also list all forbidden files and
behaviors.

This scope candidate must not edit product code. It must not implement DTOs,
selectors, validators, repair, exporter, workbook, IPC, Qwen, Seedance,
desktop, intake, or runtime prompt generation.

## Required Inputs

Read:

- `docs/seedance2-v108-retrieval-selection-implementation-scope-dispatch-2026-04-23.md`
- `docs/seedance2-v108-shot-language-selection-contract-review-2026-04-23.md`
- `docs/seedance2-v108-shot-language-selection-contract-candidate-2026-04-23.md`
- `docs/seedance2-v108-canonical-object-registry-schema-proposal-control-review-2026-04-23.md`
- `docs/seedance2-v108-canonical-object-registry-readiness-control-review-2026-04-23.md`
- `docs/seedance2-v108-validator-evidence-contract-review-2026-04-23.md`
- `docs/seedance2-v108-validator-evidence-contract-candidate-2026-04-23.md`
- `docs/generation-field-rule-contract-review-2026-04-22.md`
- `docs/generation-field-rule-contract-candidate-2026-04-22.md`
- `docs/seedance2-v108-vnext-schema-decision-2026-04-23.md`

Read KB evidence as read-only:

- `E:\codex\hope-kb\docs\seedance2-v108-director-style-web-research-2026-04-23.md`
- `E:\codex\hope-kb\docs\seedance2-v108-canonical-object-registry-schema-proposal-2026-04-23.md`
- `E:\codex\hope-kb\seed\v0.1\director_profiles.json`
- `E:\codex\hope-kb\seed\v0.1\director_rules.json`
- `E:\codex\hope-kb\seed\v0.1\director_reference_sets.json`
- `E:\codex\hope-kb\seed\v0.1\director_scene_affinity.json`

## Required Output

Create:

```text
docs/seedance2-v108-retrieval-selection-implementation-scope-candidate-2026-04-23.md
```

## Required Scope Candidate Content

The candidate must include:

- current accepted planning baseline
- exact implementation goal for an internal selector, stated without product
  prompt generation
- exact candidate files to inspect and possibly edit in the later code gate
- exact test files or test commands that would verify the later code gate
- exact forbidden files
- data boundaries for the eleven internal style lanes
- boundary for registry schema: schema may be referenced as blocked metadata,
  but registry rows and product handles remain absent
- blocker behavior for:
  - `placeholder_present`
  - `prompt_body_blocked_by_placeholder`
  - `reference_handle_unresolved`
  - `canonical_name_missing`
  - `v3_alignment_gap`
  - `schema_field_missing`
  - `promotion_gate_not_accepted`
  - `reference_control_core_closed`
- proposed minimal implementation slices, ordered from safest to riskiest
- rollback / validation plan for the later implementation gate

The candidate must explicitly say whether implementation should start with:

- a pure planning/debug metadata module
- fixture-only tests
- a non-runtime internal selector
- or no implementation yet

It must justify the choice.

## Required Assertions

Keep these assertions unchanged:

```text
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
qwen_calls = 0
seedance_calls = 0
desktop_or_intake_changes = 0
v3_branch_edits = 0
```

## Forbidden Scope

Do not:

- edit Rust / app / runtime / exporter / workbook / IPC code in this gate
- edit `hope-kb`
- edit V3, desktop, or intake
- import V108 rows
- promote positive few-shot
- create product-ready external reference handles
- create registry rows
- create `reference_control_core`
- write final storyboard words
- write runtime prompt text
- call Qwen or Seedance
- change seed manifests, import maps, snapshots, normalized staging artifacts,
  or seed bundles

## Validation

This dispatch is docs-only. No code tests are required unless the candidate
chooses to inspect the test suite without modifying it. If any code file changes
in this gate, stop and report drift.

## Completion

Commit and push to `origin/codex/contracts-freeze`.

Report back to main control:

- final branch / commit
- changed files
- candidate implementation file list
- candidate test list
- forbidden file list
- recommended first implementation slice
- unresolved blockers
- unchanged assertions
- validation result; if docs-only and no tests were run, say so
