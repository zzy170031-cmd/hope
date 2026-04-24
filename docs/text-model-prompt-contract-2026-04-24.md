# Text Model Prompt Contract 2026-04-24

## Scope

This gate defines the current Hope MVP boundary for text generation and
`prompt_text` compilation.

It does not:

- call live Qwen
- call live Doubao
- call Seedance runtime
- generate video inside the app
- introduce real API keys or network calls

## Current MVP Product Exit

Hope currently ships text outputs only:

1. expanded script text
2. storyboard table rows
3. per-shot `prompt_text` adapted for Seedance2.0 wording
4. exported JSON / CSV / Excel files

Seedance2.0 is the prompt adaptation target.
It is not the current in-app video generation interface.

## Text Model Provider Contract

The current provider contract is replaceable and desktop-bindable later.

- `provider`: `qwen | doubao | custom`
- `model`
- `base_url` optional
- `api_key_ref`
- `enabled`

Current default target:

- provider: `qwen`
- model: `qwen-default-text`
- enabled: `false`

Future providers:

- `doubao` reserved as a first future provider slot
- `custom` reserved for enterprise / self-hosted text endpoints

## Text Generation Tasks

Hope now carries one bounded task enum:

- `expand_script`
- `generate_storyboard`
- `repair_storyboard`
- `compile_seedance_prompt_text`

These tasks define the request/response shape even when live provider calls stay
closed.

## PromptTextCompilation Boundary

Input:

- structured storyboard row
- selected KB rule summaries
- `continuity_negative_core`

Output:

- Seedance2.0-ready `prompt_text`

Rules:

- compilation may be deterministic in MVP
- raw `prompt_body` remains candidate evidence only
- raw `prompt_body` must not be declared as final compiled `prompt_text`
- Seedance video generation remains a future gate

## Token Compression Rules

Hope must not send the entire 152-row package to a text model.

Compression policy:

1. choose a small set of rules by scene/shot intent, not the full KB package
2. keep KB input as summary lines, not raw long-form source rows
3. pass selected sample IDs only, not the full library payload
4. repair tasks should send only validator failures and local repair context
5. reserve rows remain evidence/planning support and are not promoted into
   positive few-shot context

## Desktop Binding Note

Desktop top-bar model selection and API endpoint settings are future UI work.
When that gate opens, the desktop layer should bind to this provider contract
instead of inventing a separate runtime model schema.

## Remaining Gates

- live Qwen
- live Doubao
- Seedance video generation
- prompt repair via live provider calls
- desktop provider settings UI / IPC binding
