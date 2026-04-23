# Seedance2 V108 Canonical Object Registry Schema Proposal Dispatch 2026-04-23

## Dispatch Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- target repo: `E:\codex\hope-kb`
- target branch: `codex/contracts-freeze`
- accepted readiness review:
  `docs/seedance2-v108-canonical-object-registry-readiness-control-review-2026-04-23.md`
- reviewed KB readiness commit:
  `8cd8335` (`docs: assess V108 canonical object registry readiness`)
- package type: KB-only docs schema proposal dispatch

V108 is only ready for a docs-only canonical object registry schema proposal.
No registry implementation or product-facing reference handle gate is open.

## Thread Label

```text
Hope-KB-V108CanonicalObjectRegistrySchemaProposal【Schema候选执行中】
```

## Sync Requirement

Before working, use the fixed Git path:

```text
& 'C:\Program Files\Git\cmd\git.exe' -C E:\codex\hope-kb pull --ff-only
& 'C:\Program Files\Git\cmd\git.exe' -C E:\codex\hope-kb status --short --branch
```

Expected target repo:

```text
E:\codex\hope-kb / codex/contracts-freeze
```

## Task

Create a KB-only docs schema proposal for a future canonical object registry.

The proposal must define how a later implementation could represent canonical
objects without adding any object rows in this gate.

This is schema planning only. It must not create product-ready handles,
canonical object records, source object names, image URLs, asset IDs, character
appearances, scene designs, or `reference_control_core`.

## Required Inputs

Read Hope control files as read-only route evidence:

- `E:\codex\hope\docs\seedance2-v108-canonical-object-registry-schema-proposal-dispatch-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-canonical-object-registry-readiness-control-review-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-canonical-object-registry-readiness-dispatch-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-shot-language-selection-contract-review-2026-04-23.md`
- `E:\codex\hope\docs\seedance2-v108-validator-evidence-contract-review-2026-04-23.md`

Read KB evidence:

- `docs/seedance2-v108-canonical-object-registry-readiness-2026-04-23.md`
- `docs/seedance2-v108-external-reference-handles-canonical-name-review-2026-04-23.md`
- `docs/seedance2-v108-field-mapping-review-2026-04-23.md`
- `docs/seedance2-v108-v3-alignment-report-2026-04-23.md`
- `docs/normalized-staging/seedance2-v108-fused-golden-sample.normalized.json`
- `seed/v0.1/source_register.json`
- `seed/v0.2/source_register.json`
- `seed/v0.2/import_map.json`

If a listed file is absent, record it as an input gap. Do not invent data.

## Required Output

Create:

```text
docs/seedance2-v108-canonical-object-registry-schema-proposal-2026-04-23.md
```

Optionally update `docs/live-progress.md` to reflect this docs-only package.
Do not change seed files, manifests, import maps, snapshots, or normalized
staging artifacts.

## Required Proposal Content

The proposal must define planning-only schema names for:

- `CanonicalObjectRegistrySchema`
- `CanonicalObjectRecord`
- `CanonicalObjectCategory`
- `CanonicalObjectAliasRule`
- `CanonicalObjectProvenance`
- `CanonicalObjectUsePolicy`
- `CanonicalObjectBlocker`
- `CanonicalObjectImportMapRequirement`
- `CanonicalObjectManifestRequirement`
- `CanonicalObjectReviewStatus`

The proposal must define object categories as planning names only, using or
revising the readiness candidates:

- `character`
- `scene`
- `prop`
- `style`
- `continuity_object`

The proposal must define planning fields, using or revising the readiness
candidates:

- `canonical_object_id`
- `canonical_object_name`
- `object_category`
- `source_aliases`
- `source_row_refs`
- `source_register_refs`
- `ip_abstraction_status`
- `allowed_reference_use`
- `blocked_reference_use`
- `continuity_notes`
- `registry_provenance`
- `registry_review_status`

The proposal must also specify:

- how aliases differ from canonical object names
- how candidate stems from V108 remain non-canonical until accepted
- how source provenance is recorded without deriving names from prompt prose
- how placeholder content blocks registry entry creation
- how IP / abstraction status controls allowed and blocked use
- how future v0.2 import map, manifest counts, content hash, and snapshot path
  would be required in a later implementation gate
- how `reference_control_core` remains closed
- what exact future gate would be needed before registry rows can be added

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
- create canonical object rows
- choose source object names
- invent image URLs, asset IDs, character appearances, scene designs, or
  canonical object records
- change seed manifests, import maps, snapshots, normalized staging artifacts,
  or seed bundles
- call Qwen or Seedance

## Validation

This is docs-only. If no seed files are changed, code/seed validation is not
required. If any seed, manifest, import map, snapshot, or normalized staging
file changes, stop and report drift instead of committing.

## Completion

Commit and push to `origin/codex/contracts-freeze`.

Report back to main control:

- final branch / commit
- changed files
- schema planning objects
- category list
- field list
- alias / provenance / blocker rules
- future implementation gate needed
- unchanged assertions
- validation result; if docs-only and no tests were run, say so
