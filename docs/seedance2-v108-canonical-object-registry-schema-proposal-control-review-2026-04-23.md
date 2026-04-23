# Seedance2 V108 Canonical Object Registry Schema Proposal Control Review 2026-04-23

## Review Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- prior control dispatch:
  `22162ea` (`docs: dispatch V108 registry schema proposal`)
- reviewed KB repo: `E:\codex\hope-kb`
- reviewed KB branch: `codex/contracts-freeze`
- reviewed KB commit:
  `513c681` (`docs: propose V108 canonical object registry schema`)
- reviewed KB memo:
  `docs/seedance2-v108-canonical-object-registry-schema-proposal-2026-04-23.md`
- review scope: main-control acceptance of KB-only docs schema proposal

This review does not modify `hope-kb`, V3, desktop, intake, Rust DTOs,
validators, repair, exporter, workbook, IPC, Qwen, Seedance, product import,
positive few-shot promotion, product-ready external reference handles, canonical
object records, or `reference_control_core`.

## Decision

`SEEDANCE2_V108_CANONICAL_OBJECT_REGISTRY_SCHEMA_PROPOSAL_ACCEPTED`

Main control accepts the KB schema proposal as the planning schema baseline for
a future canonical object registry.

Acceptance is schema-only. It does not authorize registry seed rows, source
object names, product-ready handles, Qwen / Seedance context, V108 import,
positive few-shot promotion, or `reference_control_core`.

## Accepted Planning Objects

The following planning names are accepted:

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

They are not accepted as seed files, JSON records, database tables, Rust DTOs,
IPC payloads, workbook sheets, UI taxonomy, validator enums, Qwen prompts, or
Seedance prompt structures.

## Accepted Categories

The following categories are accepted as planning-only registry categories:

- `character`
- `scene`
- `prop`
- `style`
- `continuity_object`

No current V108 candidate stem is accepted into any category.

## Accepted Planning Fields

The following fields are accepted as planning-only future record fields:

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

These fields are not added to any seed file, import map, manifest, snapshot,
Rust DTO, workbook, exporter, UI, or runtime prompt in this gate.

## Accepted Rules

Main control accepts these schema rules:

- aliases are not canonical names
- V108 candidate stems remain non-canonical evidence
- canonical names must come from a later accepted source object package
- prompt prose, media labels, V108 titles, candidate stems, and old V3
  assumptions cannot create canonical names
- placeholder aliases preserve `placeholder_present`
- ambiguous aliases preserve `ambiguous_source_name`
- provenance must come from accepted source registers, staging row references,
  accepted review docs, or future accepted registry sources
- `reference_control_core` remains closed
- product-ready handles require a later Hope-side reference handle contract

## Assertions Remain Frozen

```text
raw_source_rows_ready_for_future_dry_run = 115
rows_ready_for_product_import = 0
rows_ready_for_positive_fewshot = 0
product_ready_external_reference_handles = 0
reference_control_core_coverage = 0
qwen_calls = 0
seedance_calls = 0
hope_product_code_changes = 0
desktop_or_intake_changes = 0
v3_branch_edits = 0
rust_dto_validator_exporter_workbook_ipc_changes = 0
```

## Remaining Blockers

The following blockers remain unresolved:

- no accepted source object names
- no registry seed package
- no import_map entry for registry data
- no manifest / content hash / snapshot path for registry data
- no Hope-side product handle contract
- no `reference_control_core` coverage gate
- placeholder and ambiguous alias blockers remain active
- V3 alignment gap remains active

## Route Impact

The reference boundary is now planned but not implemented.

Because product-ready handles and registry rows remain closed, the next safe
main-thread gate is not registry implementation. The next gate is a
retrieval-selection implementation scope candidate that names exact files,
tests, forbidden files, and residual blockers before any code work begins.

## Validation

This was a docs-only control review. No code tests were required or run.
