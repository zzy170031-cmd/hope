# Seedance2 V108 Canonical Object Registry Readiness Dispatch 2026-04-23

## Dispatch Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- package type: KB-only docs readiness dispatch
- accepted shot-language selection review:
  `docs/seedance2-v108-shot-language-selection-contract-review-2026-04-23.md`
- accepted validator evidence review:
  `docs/seedance2-v108-validator-evidence-contract-review-2026-04-23.md`
- accepted KB director-style expansion review:
  `docs/seedance2-v108-director-style-expansion-control-review-2026-04-23.md`

The selector and validator evidence contracts both keep product-ready external
references blocked. This dispatch opens the next KB-only readiness gate for the
canonical object registry boundary.

## Thread Label

```text
Hope-KB-V108CanonicalObjectRegistryReadiness【Reference边界评估中】
```

## Target Repo

```text
E:\codex\hope-kb / codex/contracts-freeze
```

Before working, the KB thread must sync with the fixed Git path:

```text
& 'C:\Program Files\Git\cmd\git.exe' -C E:\codex\hope-kb pull --ff-only
& 'C:\Program Files\Git\cmd\git.exe' -C E:\codex\hope-kb status --short --branch
```

## Task

Create a KB-only docs readiness review for a future canonical object registry.

The review must answer what is needed before V108 `reference_bundle` evidence
can become product-facing external reference handles or support
`reference_control_core`.

This is not a registry implementation gate. It must not create product-ready
handles, canonical object rows, reference IDs, image URLs, asset IDs,
character appearances, scene designs, or `reference_control_core`.

## Required Inputs

Read Hope control files as read-only route evidence:

- `E:\codex\hope\docs\seedance2-v108-canonical-object-registry-readiness-dispatch-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-shot-language-selection-contract-review-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-shot-language-selection-contract-candidate-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-validator-evidence-contract-review-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-vnext-schema-decision-2026-04-23.md`

Read KB evidence:

- `docs/seedance2-v108-import-dry-run-report-2026-04-23.md`
- `docs/seedance2-v108-field-mapping-review-2026-04-23.md`
- `docs/seedance2-v108-v3-alignment-report-2026-04-23.md`
- `docs/seedance2-v108-external-reference-handles-canonical-name-review-2026-04-23.md`
- `docs/normalized-staging/seedance2-v108-fused-golden-sample.normalized.json`
- `seed/v0.1/source_register.json`
- `seed/v0.2/source_register.json`
- `seed/v0.2/import_map.json`

If a listed file is absent, report it as an input gap instead of inventing
registry data.

## Required Output

Create:

```text
docs/seedance2-v108-canonical-object-registry-readiness-2026-04-23.md
```

The document must include:

- current reference evidence summary from V108 staging
- count of rows with reference-related source fields
- distinction between raw `reference_bundle` evidence, canonical object names,
  product-ready external reference handles, and `reference_control_core`
- unresolved risks:
  - placeholder content
  - ambiguous source names
  - IP / abstraction risk
  - missing object registry schema
  - missing import map
  - missing manifest/content hash path
  - V3 alignment gaps
- proposed registry readiness checklist
- proposed future registry object fields, if any, as planning names only
- gate order before handles can become product-facing
- explicit recommendation: ready / not ready / ready only for docs-only
  registry schema proposal

## Required Assertions

Keep these assertions unchanged:

```text
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
```

Also keep:

```text
qwen_calls = 0
seedance_calls = 0
hope_product_code_changes = 0
desktop_or_intake_changes = 0
v3_branch_edits = 0
rust_dto_validator_exporter_workbook_ipc_changes = 0
```

## Forbidden Scope

Do not:

- change `E:\codex\hope`
- change V3, desktop, or intake
- import V108 rows
- promote positive few-shot
- create product-ready external reference handles
- create `reference_control_core`
- invent image URLs, asset IDs, character appearances, scene designs, or
  canonical object records
- change seed manifests, import maps, snapshots, or seed bundles unless main
  control opens a later implementation gate
- call Qwen or Seedance

## Validation

This is docs-only. If no seed files are changed, code/seed validation is not
required. If the KB thread changes any seed or manifest file by mistake, stop
and report drift.

## Completion

Commit and push to `origin/codex/contracts-freeze`.

Report back to main control:

- final branch / commit
- changed files
- reference evidence counts
- registry readiness recommendation
- unresolved blockers
- unchanged assertions
- validation result; if docs-only and no tests were run, say so
