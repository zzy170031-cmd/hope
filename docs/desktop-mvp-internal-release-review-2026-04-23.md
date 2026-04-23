# Desktop MVP Internal Release Review 2026-04-23

## Review Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- current main anchor: `612f908` (`docs: accept V108 retrieval selector`)
- current control dispatch: `422db05`
  (`docs: dispatch desktop MVP release acceptance`)
- reviewed desktop repo: `E:\codex\hope-desktop-shell`
- accepted desktop anchor: `58c0d40`
  (`desktop-shell: fix MSI packaging acceptance`)
- related intake anchor: `d74bd11`
  (`qwen-storyboard: tighten contract boundary`)
- related KB anchor: `513c681`
  (`docs: propose V108 canonical object registry schema`)

This review records the five-agent control decision for the current Hope desktop
package candidate.

## Decision

`HOPE_DESKTOP_MVP_INTERNAL_RELEASE_ACCEPTED`

Main control accepts the current desktop package candidate for **internal MVP
release**.

This is **not** a declaration that the desktop product has reached the final
public product UI or final v0.2 scope. It is a bounded release decision inside
the current `RC_READY + scope freeze` route.

## Accepted Release Basis

The current acceptance is based on the following lane conclusions:

### Hope Main

- current route remains `RC_READY + scope freeze`
- main repo is clean and aligned with `origin/codex/contracts-freeze`
- accepted retrieval selector remains non-runtime and metadata-only
- no main-thread blocker remains for the current desktop MVP release

### Desktop

- desktop repo fix `58c0d40` is now pushed and available on
  `origin/codex/desktop-shell`
- MSI packaging configuration is now committed into the repo
- per-user install mode is formalized through the WiX template
- install, launch, Start Menu shortcut, and desktop shortcut were all verified
- package reproduction no longer depends on temporary Tauri override config

### Intake

- intake remains green standby
- current role is still contract shell / snapshot bootstrap app
- no live Qwen call, no Seedance integration, no V108 import
- intake is not a blocker for the desktop MVP release

### KB / V3

- KB remains independent standby and does not block the release
- V3 remains archived proposal-only input for a future v0.2 scope gate
- neither lane should be reopened during the current MVP release step

## Frozen Boundary Still Required

The accepted desktop MVP release must continue to preserve:

- no live Qwen call
- no Seedance integration
- no V108 row import
- no product-ready external reference handles
- no `reference_control_core`
- no new frozen contracts
- no workbook / exporter / validator / IPC expansion
- no `hope-kb` merge into `hope`
- no V3/V4 engineering implementation before a written v0.2 gate

## Important Product Note

The five-agent review also confirms that the **current UI is still a Track B /
UI skeleton product shell**, not the previously discussed final detailed product
UI.

Therefore:

- this package is accepted as an **internal MVP / release candidate**
- this package is **not yet accepted as the final public desktop product**

## Release Call

Main control call for the current milestone:

- `desktop MVP candidate ready for internal release`: **yes**
- `desktop final public product ready`: **no**

## Exact Next Step

Use the current desktop package for internal MVP release / trial distribution.

After that release step, the first reopened product gate should be a bounded
desktop UI alignment review that compares:

- current Track B skeleton shell
- previously discussed target desktop product workflow

That post-release gate may define the next product UI package. It must not be
mixed into the current MVP release acceptance.
