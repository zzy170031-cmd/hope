#![forbid(unsafe_code)]

use std::fmt;

pub use core_domain::{Exporter, LLMProvider, PromptRenderer, SceneTaxonomyRecord, Validator};

pub const RENDER_SEGMENT_TARGET_MIN_SECONDS: u16 = 30;
pub const RENDER_SEGMENT_TARGET_MAX_SECONDS: u16 = 90;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryboardPlanRequest {
    pub render_segment_id: String,
    pub narrative_scene_id: String,
    pub render_segment_sequence_no: u32,
    pub start_shot_sequence_no: u32,
    pub end_shot_sequence_no: u32,
    pub target_duration_seconds: u16,
    pub cut_id: String,
    pub cut_sequence_no: u32,
    pub shot_description: String,
    pub dialogue: String,
    pub scene_director_id: Option<String>,
    pub action_director_id: Option<String>,
    pub scene_taxonomy: Option<SceneTaxonomyRecord>,
    pub layout_prompt: String,
    pub render_prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryboardPlan {
    pub render_segment: RenderSegmentPlan,
    pub cut: CutPlan,
    pub handoff_zone: HandoffZonePlan,
    pub committee_runtime: CommitteeRuntimePlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSegmentPlan {
    pub render_segment_id: String,
    pub narrative_scene_id: String,
    pub sequence_no: u32,
    pub start_shot_sequence_no: u32,
    pub end_shot_sequence_no: u32,
    pub target_duration_seconds: u16,
    pub actual_duration_seconds: Option<u16>,
    pub scene_taxonomy_id: Option<String>,
    pub scene_type: Option<String>,
    pub continuity_priority: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutPlan {
    pub cut_id: String,
    pub render_segment_id: String,
    pub sequence_no: u32,
    pub shot_description: String,
    pub dialogue: String,
    pub duration_seconds: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffZonePlan {
    pub handoff_zone_id: String,
    pub render_segment_id: String,
    pub start_boundary: String,
    pub end_boundary: String,
    pub boundary_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptLayers {
    pub layout_prompt: String,
    pub render_prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorAssignment {
    pub primary_scene_director_id: String,
    pub primary_action_director_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitteeRuntimePlan {
    pub render_segment_id: String,
    pub scene_taxonomy_id: Option<String>,
    pub scene_type: Option<String>,
    pub director_assignment: DirectorAssignment,
    pub prompt_layers: PromptLayers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneTaxonomyContext {
    pub scene_taxonomy_id: String,
    pub scene_type: String,
    pub default_duration_band: String,
    pub continuity_priority: String,
    pub typical_committee_roles: Vec<String>,
    pub default_handoff_out: Vec<String>,
    pub prompt_focus: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoryboardPlanningError {
    RenderSegmentDurationOutOfRange {
        target_duration_seconds: u16,
        min_seconds: u16,
        max_seconds: u16,
    },
    RenderSegmentShotRangeInvalid {
        start_shot_sequence_no: u32,
        end_shot_sequence_no: u32,
    },
    MissingDirectorAssignment,
}

impl fmt::Display for StoryboardPlanningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RenderSegmentDurationOutOfRange {
                target_duration_seconds,
                min_seconds,
                max_seconds,
            } => write!(
                f,
                "render segment duration {}s is outside frozen range {}s-{}s",
                target_duration_seconds, min_seconds, max_seconds
            ),
            Self::RenderSegmentShotRangeInvalid {
                start_shot_sequence_no,
                end_shot_sequence_no,
            } => write!(
                f,
                "render segment shot range is invalid: {} > {}",
                start_shot_sequence_no, end_shot_sequence_no
            ),
            Self::MissingDirectorAssignment => f.write_str("at least one director id is required"),
        }
    }
}

impl std::error::Error for StoryboardPlanningError {}

pub fn build_storyboard_plan(
    request: StoryboardPlanRequest,
) -> Result<StoryboardPlan, StoryboardPlanningError> {
    validate_render_segment_window(
        request.target_duration_seconds,
        request.start_shot_sequence_no,
        request.end_shot_sequence_no,
    )?;

    let taxonomy_context = request
        .scene_taxonomy
        .as_ref()
        .map(derive_scene_taxonomy_context);
    let director_assignment = aggregate_director_assignment(
        request.scene_director_id,
        request.action_director_id,
        taxonomy_context.as_ref(),
    )?;
    let prompt_layers = build_prompt_layers(
        request.layout_prompt,
        request.render_prompt,
        taxonomy_context.as_ref(),
    );

    let render_segment = RenderSegmentPlan {
        render_segment_id: request.render_segment_id.clone(),
        narrative_scene_id: request.narrative_scene_id.clone(),
        sequence_no: request.render_segment_sequence_no,
        start_shot_sequence_no: request.start_shot_sequence_no,
        end_shot_sequence_no: request.end_shot_sequence_no,
        target_duration_seconds: request.target_duration_seconds,
        actual_duration_seconds: Some(request.target_duration_seconds),
        scene_taxonomy_id: taxonomy_context
            .as_ref()
            .map(|context| context.scene_taxonomy_id.clone()),
        scene_type: taxonomy_context
            .as_ref()
            .map(|context| context.scene_type.clone()),
        continuity_priority: taxonomy_context
            .as_ref()
            .map(|context| context.continuity_priority.clone()),
    };

    let cut = CutPlan {
        cut_id: request.cut_id,
        render_segment_id: request.render_segment_id.clone(),
        sequence_no: request.cut_sequence_no,
        shot_description: request.shot_description,
        dialogue: request.dialogue,
        duration_seconds: request.target_duration_seconds,
    };

    let handoff_zone = derive_handoff_zone(&render_segment, taxonomy_context.as_ref());
    let committee_runtime = CommitteeRuntimePlan {
        render_segment_id: request.render_segment_id,
        scene_taxonomy_id: render_segment.scene_taxonomy_id.clone(),
        scene_type: render_segment.scene_type.clone(),
        director_assignment,
        prompt_layers,
    };

    Ok(StoryboardPlan {
        render_segment,
        cut,
        handoff_zone,
        committee_runtime,
    })
}

pub fn validate_render_segment_window(
    target_duration_seconds: u16,
    start_shot_sequence_no: u32,
    end_shot_sequence_no: u32,
) -> Result<(), StoryboardPlanningError> {
    if !(RENDER_SEGMENT_TARGET_MIN_SECONDS..=RENDER_SEGMENT_TARGET_MAX_SECONDS)
        .contains(&target_duration_seconds)
    {
        return Err(StoryboardPlanningError::RenderSegmentDurationOutOfRange {
            target_duration_seconds,
            min_seconds: RENDER_SEGMENT_TARGET_MIN_SECONDS,
            max_seconds: RENDER_SEGMENT_TARGET_MAX_SECONDS,
        });
    }

    if start_shot_sequence_no > end_shot_sequence_no {
        return Err(StoryboardPlanningError::RenderSegmentShotRangeInvalid {
            start_shot_sequence_no,
            end_shot_sequence_no,
        });
    }

    Ok(())
}

pub fn derive_scene_taxonomy_context(scene_taxonomy: &SceneTaxonomyRecord) -> SceneTaxonomyContext {
    SceneTaxonomyContext {
        scene_taxonomy_id: scene_taxonomy.scene_taxonomy_id.clone(),
        scene_type: scene_taxonomy.scene_type.clone(),
        default_duration_band: scene_taxonomy.default_duration_band.clone(),
        continuity_priority: scene_taxonomy.continuity_priority.clone(),
        typical_committee_roles: scene_taxonomy.typical_committee_roles.clone(),
        default_handoff_out: scene_taxonomy.default_handoff_out.clone(),
        prompt_focus: scene_taxonomy.prompt_focus.clone(),
    }
}

pub fn aggregate_director_assignment(
    scene_director_id: Option<String>,
    action_director_id: Option<String>,
    scene_taxonomy: Option<&SceneTaxonomyContext>,
) -> Result<DirectorAssignment, StoryboardPlanningError> {
    match (scene_director_id, action_director_id) {
        (Some(scene_director_id), Some(action_director_id)) => Ok(DirectorAssignment {
            primary_scene_director_id: scene_director_id,
            primary_action_director_id: action_director_id,
        }),
        (Some(scene_director_id), None) => {
            let primary_action_director_id = scene_taxonomy
                .map(|taxonomy| taxonomy_default_director_id(taxonomy, "action"))
                .unwrap_or_else(|| scene_director_id.clone());
            Ok(DirectorAssignment {
                primary_scene_director_id: scene_director_id,
                primary_action_director_id,
            })
        }
        (None, Some(action_director_id)) => {
            let primary_scene_director_id = scene_taxonomy
                .map(|taxonomy| taxonomy_default_director_id(taxonomy, "scene"))
                .unwrap_or_else(|| action_director_id.clone());
            Ok(DirectorAssignment {
                primary_scene_director_id,
                primary_action_director_id: action_director_id,
            })
        }
        (None, None) => {
            if let Some(taxonomy) = scene_taxonomy {
                Ok(DirectorAssignment {
                    primary_scene_director_id: taxonomy_default_director_id(taxonomy, "scene"),
                    primary_action_director_id: taxonomy_default_director_id(taxonomy, "action"),
                })
            } else {
                Err(StoryboardPlanningError::MissingDirectorAssignment)
            }
        }
    }
}

pub fn build_prompt_layers(
    layout_prompt: String,
    render_prompt: String,
    scene_taxonomy: Option<&SceneTaxonomyContext>,
) -> PromptLayers {
    if let Some(scene_taxonomy) = scene_taxonomy {
        let prompt_focus = if scene_taxonomy.prompt_focus.is_empty() {
            String::new()
        } else {
            format!("；焦点：{}", scene_taxonomy.prompt_focus.join("、"))
        };

        PromptLayers {
            layout_prompt: format!(
                "{}；场景分类：{}{}",
                layout_prompt, scene_taxonomy.scene_type, prompt_focus
            ),
            render_prompt: format!(
                "{}；连续性优先级：{}",
                render_prompt, scene_taxonomy.continuity_priority
            ),
        }
    } else {
        PromptLayers {
            layout_prompt,
            render_prompt,
        }
    }
}

pub fn derive_handoff_zone(
    render_segment: &RenderSegmentPlan,
    scene_taxonomy: Option<&SceneTaxonomyContext>,
) -> HandoffZonePlan {
    let boundary_type = if let Some(scene_taxonomy) = scene_taxonomy {
        if scene_taxonomy
            .default_handoff_out
            .iter()
            .any(|value| value == "action")
        {
            "taxonomy_action_transition".to_string()
        } else if scene_taxonomy
            .continuity_priority
            .eq_ignore_ascii_case("high")
        {
            "taxonomy_continuity_guard".to_string()
        } else {
            "render_segment_boundary".to_string()
        }
    } else {
        "render_segment_boundary".to_string()
    };

    HandoffZonePlan {
        handoff_zone_id: format!("handoff-zone-{}", render_segment.render_segment_id),
        render_segment_id: render_segment.render_segment_id.clone(),
        start_boundary: render_segment.start_shot_sequence_no.to_string(),
        end_boundary: render_segment.end_shot_sequence_no.to_string(),
        boundary_type,
    }
}

fn taxonomy_default_director_id(scene_taxonomy: &SceneTaxonomyContext, role: &str) -> String {
    format!("taxonomy:{}:{}", scene_taxonomy.scene_taxonomy_id, role)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WEEK3_SHARED_FIXTURE: &str =
        include_str!("../../../contracts/fixtures/week3-shared-fixture.json");

    fn daily_dialogue_taxonomy() -> SceneTaxonomyRecord {
        SceneTaxonomyRecord {
            scene_taxonomy_id: "scene-taxonomy-daily-dialogue".to_string(),
            scene_type: "daily_dialogue".to_string(),
            display_name: "日常对白".to_string(),
            definition: "以人物对白和细微表情推进关系。".to_string(),
            default_duration_band: "30s-60s".to_string(),
            typical_committee_roles: vec!["scene".to_string(), "emotion".to_string()],
            default_handoff_out: vec!["emotion".to_string()],
            risk_flags: vec!["performance_drift".to_string()],
            continuity_priority: "high".to_string(),
            prompt_focus: vec!["人物关系".to_string(), "表情细节".to_string()],
            source_type: "team_curated".to_string(),
            source_notes: "week4 fixture".to_string(),
            confidence_level: "high".to_string(),
            last_reviewed_at: "2026-04-20".to_string(),
        }
    }

    #[test]
    fn shared_fixture_is_the_only_sample_source() {
        assert!(WEEK3_SHARED_FIXTURE.contains("\"prompt_package\""));
        assert!(WEEK3_SHARED_FIXTURE.contains("\"handoff_zone\""));
    }

    #[test]
    fn build_storyboard_plan_rejects_out_of_range_duration() {
        let err = validate_render_segment_window(29, 1, 2).expect_err("duration should fail");

        assert!(matches!(
            err,
            StoryboardPlanningError::RenderSegmentDurationOutOfRange { .. }
        ));
    }

    #[test]
    fn scene_taxonomy_drives_default_assignment_and_handoff() {
        let plan = build_storyboard_plan(StoryboardPlanRequest {
            render_segment_id: "render-segment-taxonomy-001".to_string(),
            narrative_scene_id: "narrative-scene-taxonomy-001".to_string(),
            render_segment_sequence_no: 1,
            start_shot_sequence_no: 10,
            end_shot_sequence_no: 12,
            target_duration_seconds: 45,
            cut_id: "cut-taxonomy-001".to_string(),
            cut_sequence_no: 1,
            shot_description: "中景对话".to_string(),
            dialogue: "我们得先把这一段收口。".to_string(),
            scene_director_id: None,
            action_director_id: None,
            scene_taxonomy: Some(daily_dialogue_taxonomy()),
            layout_prompt: "冷色调，中景".to_string(),
            render_prompt: "克制写实".to_string(),
        })
        .expect("taxonomy should provide default assignment");

        assert_eq!(
            plan.render_segment.scene_taxonomy_id.as_deref(),
            Some("scene-taxonomy-daily-dialogue")
        );
        assert_eq!(
            plan.render_segment.scene_type.as_deref(),
            Some("daily_dialogue")
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_scene_director_id,
            "taxonomy:scene-taxonomy-daily-dialogue:scene"
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_action_director_id,
            "taxonomy:scene-taxonomy-daily-dialogue:action"
        );
        assert_eq!(plan.handoff_zone.boundary_type, "taxonomy_continuity_guard");
        assert!(
            plan.committee_runtime
                .prompt_layers
                .layout_prompt
                .contains("场景分类：daily_dialogue")
        );
        assert!(
            plan.committee_runtime
                .prompt_layers
                .render_prompt
                .contains("连续性优先级：high")
        );
    }
}
