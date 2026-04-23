# Desktop Internal Trial Notes 2026-04-23

## Trial Basis

- desktop repo: `E:\codex\hope-desktop-shell`
- desktop branch: `codex/desktop-shell`
- accepted packaging anchor before this note: `58c0d40`
- accepted MVP installer: `target/release/bundle/msi/Hope Desktop Shell_0.1.0_x64_en-US.msi`

## Internal Trial Checks

- MSI is present in the packaged bundle path.
- Installed desktop executable is present at `C:\Users\Administrator\AppData\Local\Hope Desktop Shell\hope-app.exe`.
- Start Menu shortcut is present.
- Desktop shortcut is present.
- The installed app can be launched again after installation acceptance.

## Trial Feedback Captured

- The packaged app is installable and launchable for internal trial use.
- The shell still read too much like a staged engineering skeleton rather than a product workbench.
- Navigation labels exposed implementation layers instead of user-facing workflow steps.
- Local Qwen key wording could be mistaken for real connectivity, even though no live call exists.

## Follow-up In This Package

- Remove obvious skeleton and Track B wording from the desktop shell.
- Align route labels and workspace copy to the packaged MVP workflow.
- Keep the frozen runtime boundary unchanged: no live Qwen, no Seedance, no V108 import, no exporter or IPC widening.
