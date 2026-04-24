# Desktop Packaged Router Runtime Boundary 2026-04-24

## Scope

This is a docs-only packaging and local-runtime boundary for the future KB
Router inside the desktop product.

It does not:

- modify desktop UI
- modify desktop IPC
- modify runtime code
- modify `hope-kb`
- modify intake
- connect live Qwen
- connect live Doubao
- connect Seedance runtime
- generate video
- change the 23-field KB schema
- promote reserve rows or rows with `usable_for_fewshot = No`

This contract defines how a packaged desktop product should hold a read-only KB
snapshot, local route cache, credentials, logs, and export traceability once a
later implementation gate opens.

## Packaged Snapshot Boundary

The installer may include a read-only `kb_runtime_snapshot`.

Required package properties:

- versioned with `kb_version`
- identified with `snapshot_id`
- checksum-verifiable
- immutable during a product run
- readable by the local Router
- not writable by normal route-cache operations

The packaged snapshot may include the accepted KB runtime package, currently
treated as V120 plus V148 safe-plus with 152 rows and the same 23-field schema.

The packaged snapshot must not be silently rewritten by:

- user input
- route cache writes
- provider responses
- validation repair attempts
- export operations

Snapshot updates require an app update, a verified data update, or a separate
accepted package-refresh gate.

## Multi-User Boundary

The installed read-only snapshot may be shared by all OS users.

User-specific data must be isolated per OS user:

- route cache
- logs
- provider configuration
- credential references
- local export history
- recent project state

One user's route cache, provider settings, logs, or exports must not become
another user's default context.

## Route Cache Boundary

The route cache belongs in a user-local app data directory, for example:

```text
%LOCALAPPDATA%\Hope\router-cache\{kb_version}\
```

This path is descriptive, not an implementation in this docs-only gate.

Required cache key:

```text
kb_version + scene_type + shot_intent + structure_type + duration + synopsis_hash
```

`duration` means `duration_seconds`.

The route cache may store:

- `selected_sample_ids`
- selected rule IDs and compressed summaries
- `kb_context_summary`
- `retrieval_trace`
- token-budget metadata
- validator preflight result

The route cache must not store:

- plaintext API keys
- full 152-row KB payloads
- raw source workbook rows
- provider request secrets
- provider authentication headers
- user local file paths inside prompt text
- debug stack traces inside prompt text

The cache may contain user-derived summaries, so it must stay local to the
current OS user and should be clearable by product settings once a UI gate
opens.

## API Key Boundary

API keys must not be stored as plaintext files.

Allowed storage:

- Windows Credential Manager
- DPAPI-protected secret storage

Allowed product references:

- `api_key_ref`
- provider name
- model name
- endpoint/base URL when configured
- enabled/disabled state

Forbidden storage and output:

- plaintext API key in config files
- plaintext API key in route cache
- plaintext API key in logs
- plaintext API key in exports
- plaintext API key in `prompt_text`
- plaintext API key in provider messages

The current product gate still keeps providers disabled by default. This
boundary does not open live Qwen, Doubao, or custom provider calls.

## Logging Boundary

Logs may record operational metadata:

- route ID
- timestamp
- `kb_version`
- `snapshot_id`
- selected sample IDs
- selected rule IDs
- top-k values
- token-budget estimates
- validator status
- provider disabled/enabled state
- error codes

Logs must not record:

- plaintext API keys
- authorization headers
- full KB rows
- raw source workbook rows
- full user synopsis text unless a later explicit privacy gate opens it
- full provider request/response bodies unless a later debug gate opens it
- user local paths inside product prompt text
- overlay JSON as normal product logs

Logs may record hashes or stable trace IDs for correlation.

## Export Traceability Boundary

Normal product exports may include traceability that helps reproduce why a row
was generated without leaking internal runtime state.

Allowed export trace fields:

- `kb_version`
- `snapshot_id`
- `snapshot_checksum`
- `selected_sample_ids`
- selected rule IDs
- short selected rule summaries
- `router_trace_id`
- `prompt_text_compiler_version`
- duration plan
- validator status

Forbidden export trace fields:

- full 152-row KB
- full selected row payloads
- raw source workbook rows
- raw `prompt_body`
- `committee_signal_overlay`
- `retrieval_signal_overlay`
- local cache path
- user home path
- API keys
- provider authorization metadata
- debug stack traces

Final `prompt_text` may be exported only as the Seedance2.0 text-storyboard
prompt for the row. It must not contain internal fields, overlay JSON, real
director names, concrete IP names, brand names, API keys, user paths, cache
paths, or debug information.

## Packaged Runtime Request Boundary

Desktop passes the same bounded business input to the local Router:

```json
{
  "scene_type": "string",
  "synopsis_text": "string",
  "duration_seconds": 60,
  "task_type": "generate_storyboard",
  "shot_intent": "optional string",
  "structure_type": "optional string"
}
```

The packaged runtime may call local Router logic and local validators.

It must not require desktop to send:

- raw KB rows
- all KB samples
- source workbook files
- overlay JSON
- API keys
- local user paths
- media files
- Seedance runtime payloads

## Provider Request Boundary

A later provider gate may allow a text request to Qwen, Doubao, or a custom
endpoint. Until that gate opens, provider calls remain disabled.

When a later provider gate opens, the provider request may carry only:

- business input
- compressed `kb_context_summary`
- `selected_sample_ids`
- compressed selected rule summaries
- validator failure context for repair tasks
- requested duration

The provider request must not carry:

- full KB snapshot
- all 152 rows
- raw source workbook rows
- full 23-field selected row payloads
- reserve rows as positive examples
- rows with `usable_for_fewshot = No` as positive examples
- overlay JSON
- API keys inside messages
- user local paths
- cache paths
- debug logs

Provider preflight must reject any request where:

```text
full_kb_rows_included != 0
```

## Validator Runtime Boundary

Packaged validation should apply the checks defined by
`docs/kb-retrieval-router-contract-2026-04-24.md` before provider I/O and
before export.

Required checks:

- forbidden terms in model request text and final `prompt_text`
- duration conservation
- reserve gate
- `usable_for_fewshot = No` gate
- prompt-text source provenance
- no full-KB request payload
- trace/version consistency

Validation failures must block provider I/O or export when they would leak
forbidden data, violate duration, use ineligible rows as positive context, or
turn raw source text into final `prompt_text`.

## Installation And Update Boundary

Installer may place:

- application binaries
- read-only `kb_runtime_snapshot`
- static contract metadata
- default provider definitions with providers disabled

Installer must not place:

- user API keys
- user route cache
- user logs
- user exports
- mutable KB seed files
- live provider credentials

Package update may replace the read-only snapshot only when the new snapshot
has:

- explicit `kb_version`
- manifest
- checksum
- accepted package provenance

Package update must not migrate reserve rows into few-shot eligibility or add
main-table fields without a separate accepted gate.

## Still Gated

The following remain closed:

- desktop UI changes
- desktop IPC implementation
- provider settings UI
- live Qwen
- live Doubao
- live Seedance
- video generation
- code validators for this contract
- route-cache implementation
- credential-manager implementation
- exporter trace implementation
- `hope-kb` seed changes
- reserve row promotion
- 23-field schema changes

## Completion Standard

This boundary is complete when it is committed as docs-only on
`codex/contracts-freeze`.

No tests are required for this docs-only gate unless a later implementation
gate opens code changes.
