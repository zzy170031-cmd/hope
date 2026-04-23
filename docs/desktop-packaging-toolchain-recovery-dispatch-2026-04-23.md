# Desktop Packaging Toolchain Recovery Dispatch 2026-04-23

## Dispatch Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- desktop target repo: `E:\codex\hope-desktop-shell`
- desktop target branch: `codex/desktop-shell`
- accepted main implementation review:
  `docs/seedance2-v108-retrieval-selection-implementation-fixup-review-2026-04-23.md`
- intake smoke note:
  isolated toolchain `1.95.0-x86_64-pc-windows-msvc` is confirmed usable for
  local smoke recovery on this machine

Desktop debug smoke already passed earlier. The current MVP blocker is release /
package toolchain recovery.

## Thread Label

```text
Hope桌面端-PackagingToolchainRecovery【Release打包修复中】
```

## Sync Requirement

Before working:

```text
& 'C:\Program Files\Git\cmd\git.exe' -C E:\codex\hope-desktop-shell pull --ff-only
& 'C:\Program Files\Git\cmd\git.exe' -C E:\codex\hope-desktop-shell status --short --branch
```

Expected target:

```text
E:\codex\hope-desktop-shell / codex/desktop-shell
```

## Task

Recover the desktop release/package toolchain path without changing product
scope.

Use the same isolated toolchain recovery strategy that worked for intake:

- prefer `1.95.0-x86_64-pc-windows-msvc`
- do not keep fighting the damaged default `stable` toolchain

## Required Diagnostic / Recovery Commands

At minimum inspect:

```text
node --version
npm --version
cargo --version
rustc --version
rustup show
rustup toolchain list
cargo install --list
```

If `cargo-tauri` is missing, install it in the least invasive way:

```text
cargo install tauri-cli --locked
```

Then prioritize these commands with the isolated toolchain:

```text
npm run build
rustup run 1.95.0-x86_64-pc-windows-msvc cargo test -p hope-app --lib
rustup run 1.95.0-x86_64-pc-windows-msvc cargo build -p hope-app --release
rustup run 1.95.0-x86_64-pc-windows-msvc cargo tauri build
```

If `cargo tauri build` still resolves to the damaged default toolchain, report
the first real error and stop widening scope.

## Forbidden Scope

Do not:

- change `E:\codex\hope`
- change KB / intake / V3
- add Qwen / Seedance integration
- expose V108 style lanes as desktop UI text
- add registry rows, product-ready handles, or `reference_control_core`
- widen UI / IPC / exporter behavior

If the recovery is environment-only, no code commit is required.

## Completion

Report back to main control:

- whether `cargo tauri --version` is available
- recovery actions taken
- `cargo build -p hope-app --release` result
- `cargo tauri build` result
- whether an installable package was generated
- generated path, if any
- first real blocker, if package is still not ready
