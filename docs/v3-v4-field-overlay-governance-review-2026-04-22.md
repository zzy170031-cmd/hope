# V3/V4 Field Overlay Governance Review 2026-04-22

## Route

This memo is a docs-only control-thread review on the frozen Hope main thread.

- repo: `E:\codex\hope`
- frozen branch: `codex/contracts-freeze`
- frozen branch anchor before this memo: `c3d6142` (`docs: accept seedance v1 as v0.2 export overlay proposal`)
- reviewed side branch: `codex/v3-field-overlay-proposal`
- reviewed side branch anchor: `13f8d7d` (`docs: add v3 field overlay proposal handoff`)
- review type: governance / route intake only

This memo does not merge the side branch, does not implement field changes, does not modify the frozen v0.1 workbook contract, and does not authorize exporter, IPC, Rust, desktop, Seedance, or `hope-kb` implementation work.

## Decision

`ACCEPT_AS_DOCS_ONLY_V0_2_OVERLAY_PROPOSAL_KEEP_SEPARATE`

The V3/V4 field proposal is accepted as future v0.2 overlay planning input only.

It is not accepted as a frozen contract.
It is not accepted as an implementation package.
It does not change the current v0.1 17-sheet workbook contract.

## Reviewed Evidence

The side branch `codex/v3-field-overlay-proposal` adds only docs:

- `docs/contracts/export-overlays/v3-field-overlay-proposal-2026-04-22.md`
- `docs/contracts/export-overlays/source-materials/Hope_V4_Notion建库清单.md`
- `docs/contracts/export-overlays/source-materials/Hope_V4_结构化分镜Schema_Seedance优先版.md`

No code files, top-level frozen contract files, exporter files, IPC files, desktop files, or `hope-kb` files are changed by the reviewed side branch.

## Accepted Control Interpretation

The useful engineering correction from the side branch is model-boundary naming:

```text
content_generation_model = Qwen text generation
target_video_model = Seedance 2.0 video execution target
adapter_profile = Seedance adapter / compile profile
```

The ambiguous `primary_model` wording should not be promoted as future Hope truth.

The V3/V4 field material should be read as:

- Qwen output shape guidance
- Seedance-ready overlay planning
- future adapter/export planning material
- future validator and KB sample planning material

It should not be read as:

- a replacement for v0.1 entities
- a replacement for the current 17-sheet workbook
- an immediate exporter contract
- an immediate desktop payload editor
- a reason to reopen `hope-kb` merge-readiness

## Current Integration Rhythm

### Stage 0: Current main thread intake

Status: accepted by this memo.

Allowed now:

- keep the V3/V4 side branch visible as planning input
- reference side branch anchor `13f8d7d` from control-thread decisions
- use the model-boundary split in future planning language

Not allowed now:

- copy the V3/V4 field table into frozen contracts
- add v0.2 DTOs
- add exporter sheets
- add desktop editor surfaces
- add real Qwen or Seedance calls

### Stage 1: Future v0.2 overlay planning gate

Open only after an explicit scope gate.

Allowed then:

- define the minimum v0.2 overlay subset
- decide whether V3 source snapshots should be committed as source material
- decide whether source-material whitespace should be normalized
- choose which fields are KB sample work, validator work, adapter work, or desktop review work

Not allowed by default:

- implementation
- merge with `hope-kb`
- mutation of frozen v0.1 workbook sheets

### Stage 2: KB / sample hardening

Open only as an independent `hope-kb` hardening package.

Candidate work:

- golden sample schema
- sample usage taxonomy: few-shot / validator / repair / regression
- scene taxonomy aliases
- failure-code to repair-template mapping
- negative-boundary markers

This remains separate from `hope` main-thread implementation.

### Stage 3: Engineering implementation

Open only after RC release confirmation or an explicit scope reopen.

Required prerequisites:

- v0.2 overlay contract gate accepted
- minimal field subset selected
- validator ownership defined
- exporter profile ownership defined
- desktop review boundary defined
- `hope-kb` merge-readiness explicitly reopened if KB runtime changes are needed

## Thread Assignments

Main Hope control thread:

- keep RC freeze
- keep V3/V4 as proposal-only planning input
- decide whether future scope gates are opened

V3/V4 field thread:

- stop implementation work
- optionally add docs-only cleanup or source snapshots
- keep branch separate

`hope-kb`:

- remain checkpoint standby unless a KB hardening package is explicitly opened
- prepare future sample / taxonomy / repair support only after authorization

Desktop thread:

- keep four existing surfaces
- future work may show read-only readiness only
- no fifth surface
- no Seedance payload editor

Controlled intake thread:

- keep Qwen/storyboard boundary as contract shell
- no live Qwen invocation from this proposal
- no prompt package generation from this proposal

Exporter / validator:

- v0.1 remains frozen
- v0.2 overlay sheets and validators require a future gate

## Still Prohibited

- do not merge `codex/v3-field-overlay-proposal` into `codex/contracts-freeze`
- do not modify frozen v0.1 contract files
- do not modify exporter behavior
- do not add Excel overlay sheets now
- do not modify IPC
- do not modify Rust DTOs or validators now
- do not modify desktop business UI
- do not connect to real Seedance
- do not merge with `hope-kb`
- do not treat V3/V4 field tables as frozen truth

## Conclusion

Current result: `ACCEPT_AS_DOCS_ONLY_V0_2_OVERLAY_PROPOSAL_KEEP_SEPARATE`.

Hope may carry the V3/V4 model-boundary correction forward in planning language now, but the actual field overlay must wait for a future v0.2 scope gate. The active Hope route remains `RC_READY` plus scope freeze.
