# Seedance V1 v0.2 Export Overlay Proposal Gate 2026-04-22

## Thread Label

`Hope-main-thread-SeedanceV1ContractGate-round5-pending-review`

## Route

This is a docs-only contract gate record on the frozen Hope main thread.

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor commit at gate start: `5b45a7b` (`Document round 4 stable stop gate`)
- gate type: Seedance V1 export overlay proposal review

This memo does not implement exporter behavior, does not change Rust, does not change IPC, does not change desktop code, does not touch `hope-kb`, and does not reopen merge-readiness.

## Gate Decision

`SEEDANCE_V1_ACCEPTED_AS_V0_2_EXPORT_OVERLAY_PROPOSAL_ONLY`

Seedance V1 is accepted as a docs-only v0.2 export overlay proposal target.

It is not accepted as a frozen contract in this gate.

## v0.1 / v0.2 Boundary

The current v0.1 17-sheet workbook remains the stable main contract.

The Seedance V1 field set should be treated as a future v0.2 export overlay proposal because it requires a complete one-shot export profile and should not silently replace the current v0.1 workbook.

Boundary rules:

- v0.1 remains the current frozen 17-sheet workbook contract
- v0.2 may propose a Seedance-specific export profile / overlay
- v0.2 proposal must be additive as a proposal, not a direct replacement of v0.1
- no new sheet is added by this gate
- no exporter implementation is authorized by this gate
- no fixture, IPC, desktop, or KB implementation change is authorized by this gate

## Proposal Document Path

The docs-only proposal path is:

`docs/contracts/export-overlays/seedance-v1-v0.2-export-overlay-proposal-2026-04-22.md`

Future revisions should stay under:

`docs/contracts/export-overlays/`

This path is intentionally outside the frozen top-level `contracts/` source-of-truth set so the proposal is not mistaken for a frozen contract.

## Required Proposal Content

The future v0.2 proposal should cover:

- Seedance-specific export profile identity
- complete one-shot export target shape
- explicit field mapping from the v0.1 17-sheet contract
- overlay-only fields that do not mutate v0.1
- `character_lock` conditional materialization
- `scene_asset_lock` conditional materialization
- same-character multi-form modeling through one `character_id` plus explicit `form` / `state`
- readiness semantics for any field that is intentionally unavailable in v0.1

## Future Track Ownership

These items belong to future Track A contract change discussion:

- domain entity shape for Seedance-specific export profile
- conditional entity rules for `character_lock`
- conditional entity rules for `scene_asset_lock`
- identity rules for `character_id` plus `form` / `state`
- schema or type changes required to carry v0.2 overlay fields

These items belong to future Track E contract change discussion:

- render-segment / cut mapping into Seedance-ready timing or shot fields
- segment and cut projection rules needed by the overlay
- handoff or continuity implications if the overlay needs explicit transition payloads

These items belong to future Track G contract change discussion:

- export profile selection
- Excel / JSON / Markdown overlay serialization strategy
- one-shot Seedance export completeness rules
- fixture and manifest updates for v0.2 proposal verification
- exporter tests required before any proposal can become a frozen contract

## Desktop P3 Boundary

Desktop P3 may only prepare semantic-zone and readiness placeholders.

Allowed now:

- display a readonly semantic zone for future Seedance readiness
- display readiness placeholder states such as not-started / proposal-only / unavailable
- link to docs-only proposal language
- avoid exposing any editable contract fields

Not allowed now:

- full Seedance field contract implementation
- new business panel
- fifth surface
- exporter execution
- IPC expansion
- desktop-side schema ownership
- any UI that implies v0.2 is already frozen

## Still Prohibited

- do not modify Rust implementation
- do not modify the Excel exporter
- do not modify IPC
- do not modify desktop code
- do not modify `hope-kb`
- do not add a new sheet
- do not replace the v0.1 17-sheet contract
- do not reopen merge-readiness
- do not merge KB work
- do not treat this proposal as a frozen contract
- do not open Round 5 implementation automatically from this gate

## Next Thread Split

Main thread:

- keep frozen supervision
- keep this as docs-only proposal gating
- wait for an explicit scope discussion gate before any new package

Intake thread:

- may refine the Seedance V1 field proposal as docs-only material only
- must not implement exporter or app behavior from this gate

Desktop thread:

- may prepare P3 semantic-zone / readiness placeholder language only
- must not implement full Seedance field contract

Hope-KB thread:

- stays in checkpoint standby
- no KB merge or KB schema expansion is authorized by this gate

## Conclusion

Current result: `PROPOSAL_ACCEPTED_KEEP_V0_1_FROZEN`.

Seedance V1 is accepted for v0.2 export overlay proposal work only. The v0.1 17-sheet workbook remains the current stable main contract.
