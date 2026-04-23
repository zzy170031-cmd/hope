# Desktop MVP Release Acceptance Dispatch 2026-04-23

## Control Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- current main anchor: `612f908` (`docs: accept V108 retrieval selector`)
- current desktop anchor: `66d687d` (`desktop-shell: add writer qwen input readiness boundary`)
- current intake anchor: `d74bd11` (`qwen-storyboard: tighten contract boundary`)

The replaced remote intake branch
`origin/codex/controlled-intake-snapshot-bootstrap` has now been retired. The
active intake route remains
`origin/codex/controlled-intake-snapshot-bootstrap-app`.

All active MVP lanes are green enough to move from build recovery into release
acceptance:

- Hope main: accepted non-runtime retrieval metadata selector
- Desktop: MSI package reproduced successfully
- Intake: smoke green under `1.95.0-x86_64-pc-windows-msvc`
- KB: frozen standby
- V3: archived standby

This dispatch keeps RC scope frozen. It does not open Qwen / Seedance, product
import, registry rows, product-ready handles, or `reference_control_core`.

## Thread Label

```text
Hope桌面端-DesktopMVPReleaseAcceptance【安装验收中】
```

## Target

```text
E:\codex\hope-desktop-shell / codex/desktop-shell
```

Expected package candidate:

```text
E:\codex\hope-desktop-shell\target\release\bundle\msi\Hope Desktop Shell_0.1.0_x64_en-US.msi
```

## Task

Run the narrow desktop MVP release-acceptance pass.

Focus only on package installability, app launch, and current frozen-boundary
behavior. Do not reopen feature work.

Minimum acceptance checks:

1. Confirm the MSI file exists and can be launched for installation.
2. Confirm the installed desktop app can launch without widening scope.
3. Confirm current shell behavior still matches the frozen boundary:
   - no Qwen live call
   - no Seedance integration
   - no V108 row import
   - no product-ready external references
4. Confirm the package is still reproducible with the known-good toolchain path:

```text
rustup run 1.95.0-x86_64-pc-windows-msvc cargo tauri build -b msi --no-sign
```

If the installation pass finds a packaging-only defect inside
`hope-desktop-shell`, a tiny desktop-only fix is allowed. Do not widen product
scope.

## Forbidden

Do not:

- change `E:\codex\hope`
- change KB / intake / V3
- add Qwen / Seedance integration
- expose V108 internal selector details as product UI
- open registry rows, external handles, or `reference_control_core`
- start local cache cleanup during acceptance unless a packaging issue requires
  it

## Report Back

Report to main control:

- install result
- launch result
- acceptance notes for frozen boundary behavior
- whether MSI remains reproducible
- final branch / commit if any desktop-only fix was needed
- any remaining blocker for calling the desktop MVP candidate ready for release
