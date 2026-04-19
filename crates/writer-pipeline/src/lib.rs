#![forbid(unsafe_code)]

pub use core_domain::{Exporter, LLMProvider, PromptRenderer, Validator};

/// Writer pipeline keeps the stage contract as small as possible until Track A
/// publishes the shared domain structs.

pub const EPISODE_TARGET_MIN_SECONDS: u32 = 22 * 60;
pub const EPISODE_TARGET_MAX_SECONDS: u32 = 24 * 60;
pub const RENDER_SEGMENT_TARGET_MIN_SECONDS: u32 = 30;
pub const RENDER_SEGMENT_TARGET_MAX_SECONDS: u32 = 90;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StructuredOutputMode {
    #[default]
    JsonOnly,
}

pub const DEFAULT_STRUCTURED_OUTPUT_MODE: StructuredOutputMode =
    StructuredOutputMode::JsonOnly;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsonDocument {
    pub json: String,
}

impl JsonDocument {
    pub fn new(json: impl Into<String>) -> Self {
        Self { json: json.into() }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DurationPolicy {
    pub episode_target_min_seconds: u32,
    pub episode_target_max_seconds: u32,
    pub render_segment_target_min_seconds: u32,
    pub render_segment_target_max_seconds: u32,
}

impl DurationPolicy {
    pub const fn frozen() -> Self {
        Self {
            episode_target_min_seconds: EPISODE_TARGET_MIN_SECONDS,
            episode_target_max_seconds: EPISODE_TARGET_MAX_SECONDS,
            render_segment_target_min_seconds: RENDER_SEGMENT_TARGET_MIN_SECONDS,
            render_segment_target_max_seconds: RENDER_SEGMENT_TARGET_MAX_SECONDS,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SynopsisInput {
    pub document: JsonDocument,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoryInput {
    pub synopsis: SynopsisInput,
    pub duration_policy: DurationPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoryOutput {
    pub document: JsonDocument,
    pub stale: StalePropagation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreenplayInput {
    pub story: StoryOutput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreenplayOutput {
    pub narrative_scenes: Vec<NarrativeSceneDraft>,
    pub dialogue_turns: Vec<DialogueTurnDraft>,
    pub output_mode: StructuredOutputMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NarrativeSceneDraft {
    pub document: JsonDocument,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogueTurnDraft {
    pub document: JsonDocument,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaleTrigger {
    SynopsisChanged,
    StoryChanged,
    ScreenplayChanged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StalePropagation {
    pub trigger: StaleTrigger,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_policy_is_frozen_to_the_contract() {
        let policy = DurationPolicy::frozen();

        assert_eq!(policy.episode_target_min_seconds, EPISODE_TARGET_MIN_SECONDS);
        assert_eq!(policy.episode_target_max_seconds, EPISODE_TARGET_MAX_SECONDS);
        assert_eq!(
            policy.render_segment_target_min_seconds,
            RENDER_SEGMENT_TARGET_MIN_SECONDS
        );
        assert_eq!(
            policy.render_segment_target_max_seconds,
            RENDER_SEGMENT_TARGET_MAX_SECONDS
        );
    }

    #[test]
    fn structured_outputs_default_to_json_only() {
        assert_eq!(
            DEFAULT_STRUCTURED_OUTPUT_MODE,
            StructuredOutputMode::JsonOnly
        );
    }
}
