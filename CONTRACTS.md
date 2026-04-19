# Contract Change Control

Day 1-3 is reserved for contract freeze.

## Frozen contract set

- product DB schema
- knowledge DB schema
- Rust domain structs
- TypeScript mirror types
- JSON schemas
- Tauri IPC contracts
- PromptRenderer trait
- LLMProvider trait
- Validator trait
- Exporter trait
- Excel workbook contract
- stale propagation contract
- segment boundary contract
- handoff zone contract

## Change rule after freeze

After Day 3:

- no silent contract edits
- all contract changes must be reviewed from the integration thread
- every contract change must list impacted tracks
- contract changes must update examples and tests in the same change

## Default policy

- prefer adapting implementation to the contract
- do not expand contracts for convenience
- if a contract must change, record why the previous contract failed
