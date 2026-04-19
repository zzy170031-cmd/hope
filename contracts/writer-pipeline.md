# Writer Pipeline Contract

Track: `D`

Scope:

- `Synopsis -> Story -> Screenplay`
- structured outputs default to non-thinking + JSON
- Story layer owns duration adaptation
- stale propagation freezes trigger points and propagation contract only
- `NarrativeScene` and `DialogueTurn` are the smallest generation units in this track

Out of scope:

- storyboard runtime
- handoff zone logic
- exporter implementation
- validator implementation

Frozen hard rules used by this track:

- `RenderSegment` target duration is `30s-90s`
- `RenderSegment` cannot cross `NarrativeScene`
- `Project` longer than `45min` must split into `Episode`
- `Episode` default target duration is `22-24min`
- project `hard_locks` are global across all episodes, segments, and cuts
- all token fields are Chinese-only

Stage shapes:

`SynopsisInput`

- wrapper around the synopsis source document
- this track treats synopsis as the upstream input to Story generation

`StoryInput`

- contains the synopsis wrapper
- contains the frozen duration policy used by Story generation
- this is the only stage that receives duration adaptation

`StoryOutput`

- structured JSON document
- stale propagation marker

`ScreenplayInput`

- consumes the Story output as the only upstream input

`ScreenplayOutput`

- structured JSON document mode
- `NarrativeScene` draft list
- `DialogueTurn` draft list
- default output mode is JSON-only

`NarrativeSceneDraft`

- opaque JSON draft
- must stay inside one `NarrativeScene`
- must not encode any `RenderSegment` boundary rule here

`DialogueTurnDraft`

- opaque JSON draft
- minimal unit of dialogue generation
- must remain in JSON form until Track A shared domain structs land

Stale propagation contract:

- trigger points freeze at `SynopsisChanged`, `StoryChanged`, and `ScreenplayChanged`
- this track only freezes trigger semantics and downstream propagation intent
- implementation of propagation is out of scope

Track A dependencies this track is waiting on:

- shared domain identity names for `Project`, `Episode`, `NarrativeScene`, `RenderSegment`, and `Cut`
- the frozen `hard_locks` container type
- the final `Tauri IPC` payload envelope name, if Track A changes the transport shape
- the final shared duration and hierarchy structs, so this module can replace the local minimal wrappers without changing the contract

Local minimal contract types in `crates/writer-pipeline`:

- `SynopsisInput`
- `StoryInput`
- `StoryOutput`
- `ScreenplayInput`
- `ScreenplayOutput`
- `NarrativeSceneDraft`
- `DialogueTurnDraft`
- `DurationPolicy`
- `StalePropagation`
- `StaleTrigger`

