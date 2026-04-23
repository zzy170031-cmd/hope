# Hope Desktop UI

React + TypeScript desktop workbench UI for Hope.

The UI must not bypass Rust commands to touch SQLite directly.

## Current desktop shell scope

- The surface is a single-page Chinese workbench aligned to the approved reference layout.
- The visible workflow is fixed in this order: top bar, script area, task bar, storyboard action bar, table, pagination, export.
- Script expansion, task creation, storyboard generation, row editing, pagination, and export remain real front-end interactions.
- The workbench does not claim live Qwen, live Seedance, runtime import of V120, or product-ready external references.
