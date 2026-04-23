# Hope Desktop UI

React + TypeScript single-page workbench for the Hope desktop internal trial.

The UI must not bypass Rust commands to touch SQLite directly.

## Current desktop shell scope

- The primary surface is now one Chinese workbench page rather than a multi-route engineering shell.
- The page distinguishes packaged readonly signals, packaged writer/preview snapshots, and editable front-end working state.
- Script expansion, storyboard generation, pagination, row editing, and export are real front-end interactions inside the frozen desktop boundary.
- The workbench does not claim live Qwen, Seedance, KB retrieval, exporter integration, or runtime growth.
