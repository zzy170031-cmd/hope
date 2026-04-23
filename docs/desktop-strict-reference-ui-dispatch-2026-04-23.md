# Desktop Strict Reference UI Dispatch 2026-04-23

## Control Route

- control repo: `E:\codex\hope`
- control branch: `codex/contracts-freeze`
- current control anchor before this dispatch: `7bcec64`
  (`docs: accept V120 full KB ingest route`)
- desktop repo: `E:\codex\hope-desktop-shell`
- desktop branch: `codex/desktop-shell`
- current local desktop anchor visible to control: `7899e4e`
  (`desktop-shell: rebuild single-page workbench`)
- desktop remote anchor: `origin/codex/desktop-shell @ 58c0d40`

Hope main remains on `RC_READY + scope freeze`.

This dispatch is a **desktop-only strict UI correction gate**.

It does not reopen runtime implementation, IPC expansion, bridge changes,
Qwen/Seedance integration, KB runtime consumption, V120 runtime import,
product-ready external references, or `reference_control_core`.

## Why This Gate Is Opened

The user has explicitly rejected the current desktop page as visually off-route.

Main control accepts the desktop thread's own report that:

- `7899e4e` is only an intermediate checkpoint
- the source page is no longer the old skeleton shell
- but the layout, density, control order, spacing, visual weight, and product
  expression still do **not** match the approved reference UI

Therefore the next desktop task is not polish. It is a **strict reference-based
UI rebuild**.

## Required Visual Sources

The desktop thread must use these user-provided local sources:

- `C:\Users\Administrator\Desktop\样本存储区\UI界面参考.png`
- `C:\Users\Administrator\Desktop\样本存储区\桌面合适logo.png`

Working rule:

- `UI界面参考.png` defines layout, proportions, region order, button order,
  table density, and page rhythm
- `桌面合适logo.png` defines product color direction and left-top branding
- previous desktop intermediate UI commits are implementation history only, not
  the new design source of truth

## Non-Negotiable Layout Requirements

The rebuilt desktop page must be a **single-page Chinese workbench** and must
strictly follow the approved reference layout.

Required top structure:

1. left-top brand block:
   - show the Hope logo
   - show the product name beside the logo
2. top control row:
   - model selector
   - `API文档`
   - `API接口`
3. script zone:
   - scene-type selector
   - story synopsis input
   - `扩写脚本`
4. task row:
   - task name input
   - `新建任务`
5. storyboard action row:
   - `导入扩写脚本`
   - duration selector
   - `清空`
   - `开始生成`
6. storyboard output table:
   - fixed column order matching the approved reference
   - dense desktop table styling
   - pagination and jump controls
7. bottom-right export zone:
   - export action placed at the lower-right area as in the reference

## Visual Correction Rules

The desktop thread must correct all of the following:

- do not keep the current oversized cream-card aesthetic
- do not keep the current loose spacing and oversized padding
- do not keep the current "soft rounded dashboard" feel
- do not invent a new composition that drifts away from the approved reference
- do not hide the logo or reduce it to favicon-scale

Instead, the thread must:

- use thin, clear borders
- keep radius restrained
- reduce empty whitespace
- make the whole page read like a compact desktop workbench
- match the reference control order and horizontal relationships
- keep the first viewport focused on the actual working surface

## Color and Branding Rules

The desktop thread must derive the product palette from the approved logo:

- cyan / aqua family from the `H`
- warm gold family from the `P` and star
- dark outline / deep navy shadow for text emphasis and high-contrast buttons

Expected usage:

- logo shown at the upper-left in the application UI
- header / accent / selected states reflect logo colors
- high-priority action button styling may use the logo's dark outline plus warm
  gold or deep accent fill
- supporting accents may use the aqua family
- avoid the current washed beige-heavy visual direction

The result should look like Hope branding, not a generic beige admin panel.

## Interaction Rules

This round is still UI-only, but the page must maintain real interaction
behavior:

- buttons must remain clickable and stateful
- task creation, script import, duration selection, generation trigger,
  pagination, row edit / copy / delete, and export must remain present in the
  UI flow
- the interaction flow must align with the user's approved working order

However, the thread must not falsely imply that the following are already live:

- live KB retrieval
- live Qwen
- live Seedance
- direct V120 runtime import
- product-ready external references

## Allowed File Scope

Allowed:

- `E:\codex\hope-desktop-shell\ui\src\App.tsx`
- `E:\codex\hope-desktop-shell\ui\src\routes.ts`
- `E:\codex\hope-desktop-shell\ui\src\styles.css`
- `E:\codex\hope-desktop-shell\ui\src\components\**`
- `E:\codex\hope-desktop-shell\ui\src\types.ts`
- `E:\codex\hope-desktop-shell\ui\README.md`
- desktop-only docs
- UI asset placement needed for the approved logo

Forbidden:

- no `app/src/runtime.rs`
- no IPC changes
- no bridge contract expansion
- no `crates/**`
- no KB / intake / V3 edits
- no `E:\codex\hope` product-code edits
- no Qwen / Seedance integration
- no V120 runtime import
- no exporter scope expansion beyond the current desktop surface

## Packaging Rule

Do not treat MSI rebuild as the main task of this round.

This round should first fix the source UI layout and visual design. Packaging
validation may happen only after the strict reference UI source is accepted.

Packaging artifacts such as `app/gen/` or opportunistic `Cargo.toml` side
effects must not become the center of this round.

## Required Report Back

The desktop thread must report:

- what old layout decisions from `7899e4e` were replaced
- which UI files changed
- whether the logo is shown at upper-left
- how the logo palette was mapped to the page
- whether the layout now matches the reference control order and proportions
- build result
- final branch / commit
- whether the page is now suitable for renewed desktop install packaging

