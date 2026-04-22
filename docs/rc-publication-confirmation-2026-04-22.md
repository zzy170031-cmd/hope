# RC Publication Confirmation 2026-04-22

## Route

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor before this memo: `5f30231` (`docs: add second control thread handoff`)
- route: `RC_READY` plus scope freeze
- decision: `RC_PUBLICATION_CONFIRMATION_CONTINUE`

This memo records a control-thread progress step. It does not reopen product
scope, does not change contracts, and does not merge any side thread.

## Current Conclusion

Hope remains `RC_READY`.

The main thread has now completed a release-confirmation smoke pass after the
latest RC validation recovery and after the KB golden-sample dispatch was handed
to the existing KB thread.

No new regression signal appeared.

## Current Anchors

Hope main:

- repo: `E:\codex\hope`
- branch: `codex/contracts-freeze`
- anchor: `5f30231` (`docs: add second control thread handoff`)
- state before this memo: clean and aligned with `origin/codex/contracts-freeze`

Hope KB:

- repo: `E:\codex\hope-kb`
- branch: `codex/contracts-freeze`
- anchor: `aa8610e` (`docs: dispatch golden sample intake to KB thread`)
- state: clean and aligned with `origin/codex/contracts-freeze`
- active task: existing KB thread
  `019da89f-49a2-7e11-bd2f-c0138165fdd9` owns the golden-sample intake package

Desktop:

- repo: `E:\codex\hope-desktop-shell`
- branch: `codex/desktop-shell`
- anchor: `66d687d` (`desktop-shell: add writer qwen input readiness boundary`)
- state: clean and aligned with `origin/codex/desktop-shell`

Controlled intake:

- repo: `E:\codex\hope-intake-app`
- branch: `codex/controlled-intake-snapshot-bootstrap-app`
- anchor: `d74bd11` (`qwen-storyboard: tighten contract boundary`)
- state: clean and aligned with
  `origin/codex/controlled-intake-snapshot-bootstrap-app`

V3/V4 proposal:

- branch: `origin/codex/v3-field-overlay-proposal`
- anchor: `13f8d7d` (`docs: add v3 field overlay proposal handoff`)
- state: v0.2 overlay proposal planning input only

## Validation Evidence

The full RC benchmark ladder was not reopened because there was no new product
code change after `b8f200a` and no regression signal.

The control thread ran a targeted publication-confirmation smoke pass with an
isolated Cargo target directory to avoid Windows target-file locks:

```text
cargo test --target-dir E:\codex\hope\target\codex-rc-smoke -p export-engine --lib
```

Result:

- `engine::tests::render_face_detail_clause_is_stronger_than_layout_clause`: pass
- `engine::tests::appearance_constraint_appends_face_cleanup_defaults`: pass
- `engine::tests::external_prompt_body_removes_machine_identifier_and_preserves_traceability_outside_body`: pass
- summary: 3 passed, 0 failed

```text
cargo test --target-dir E:\codex\hope\target\codex-rc-smoke -p hope-app
```

Result:

- `runtime::tests::build_storyboard_preview_plan_falls_back_when_scene_taxonomy_is_missing`: pass
- `runtime::tests::resolve_scene_taxonomy_matches_by_scene_type_and_display_name`: pass
- `runtime::tests::build_storyboard_preview_plan_keeps_runtime_directors_with_scene_taxonomy_metadata`: pass
- `runtime::tests::build_validation_export_panel_snapshot_surfaces_kb_repairs`: pass
- `runtime::tests::build_validation_export_panel_snapshot_stays_bounded_without_repair_mappings`: pass
- summary: 5 passed, 0 failed

Initial attempt with the shared default Cargo target directory hit Windows
`os error 5` while removing existing `.rlib` files. The isolated target run
passed and is the release-confirmation evidence for this memo.

## Five-Thread Dispatch

Hope main:

```text
Continue RC publication confirmation and baseline protection. Do not reopen
feature scope. Rerun the full benchmark ladder only if a product-code change or
regression signal appears.
```

Hope KB:

```text
Existing KB thread `019da89f-49a2-7e11-bd2f-c0138165fdd9` should continue from
`E:\codex\hope-kb` commit `aa8610e` and execute
`docs/golden-sample-intake-dispatch-2026-04-22.md`.

Normalize the committed 40-row golden-sample CSV without losing fields. Import
into seed files only if the v0.1 schema can preserve meaning safely; otherwise
stop at normalized staging plus a v0.2 schema note.
```

V3/V4:

```text
Remain proposal-only at `13f8d7d`. V3/V4 is expected to enter mainline later,
but only after RC publication follow-through and a written v0.2 scope gate.
No exporter, IPC, Rust DTO, validator, desktop UI, workbook, Qwen, or Seedance
implementation is open now.
```

Desktop:

```text
Remain accepted and waiting at `66d687d`. Do not open another desktop package
without a new control-thread scope gate.
```

Controlled intake:

```text
Remain accepted and waiting at `d74bd11`. Do not expand into live Qwen, prompt
package generation, segment/cut handoff, or runtime integration without a new
control-thread scope gate.
```

## Next Control Action

Wait for the existing KB thread to report the golden-sample intake result.

When it reports back, the control thread should review:

- changed files
- whether the CSV was staged only, normalized, or imported into seed files
- record counts before and after
- bundle hash before and after, if seed files changed
- validation result
- snapshot-build result
- any fields that require a future v0.2 schema gate

Until that report returns, the Hope main thread stays on RC publication
confirmation and baseline protection.
