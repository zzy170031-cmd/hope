#![forbid(unsafe_code)]

pub mod shot_language_selection;

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
    _scene_taxonomy: Option<&SceneTaxonomyContext>,
) -> Result<DirectorAssignment, StoryboardPlanningError> {
    match (scene_director_id, action_director_id) {
        (Some(scene_director_id), Some(action_director_id)) => Ok(DirectorAssignment {
            primary_scene_director_id: scene_director_id,
            primary_action_director_id: action_director_id,
        }),
        (Some(scene_director_id), None) => Ok(DirectorAssignment {
            primary_scene_director_id: scene_director_id.clone(),
            primary_action_director_id: scene_director_id,
        }),
        (None, Some(action_director_id)) => Ok(DirectorAssignment {
            primary_scene_director_id: action_director_id.clone(),
            primary_action_director_id: action_director_id,
        }),
        (None, None) => Err(StoryboardPlanningError::MissingDirectorAssignment),
    }
}

pub fn build_prompt_layers(
    layout_prompt: String,
    render_prompt: String,
    _scene_taxonomy: Option<&SceneTaxonomyContext>,
) -> PromptLayers {
    PromptLayers {
        layout_prompt,
        render_prompt,
    }
}

pub fn derive_handoff_zone(
    render_segment: &RenderSegmentPlan,
    _scene_taxonomy: Option<&SceneTaxonomyContext>,
) -> HandoffZonePlan {
    HandoffZonePlan {
        handoff_zone_id: format!("handoff-zone-{}", render_segment.render_segment_id),
        render_segment_id: render_segment.render_segment_id.clone(),
        start_boundary: render_segment.start_shot_sequence_no.to_string(),
        end_boundary: render_segment.end_shot_sequence_no.to_string(),
        boundary_type: "render_segment_boundary".to_string(),
    }
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

    fn overlapping_signal_taxonomy() -> SceneTaxonomyRecord {
        SceneTaxonomyRecord {
            scene_taxonomy_id: "scene-taxonomy-overlap-001".to_string(),
            scene_type: "knowledge_overlap".to_string(),
            display_name: "knowledge overlap".to_string(),
            definition: "knowledge-side signals overlap with runtime inputs".to_string(),
            default_duration_band: "30s-90s".to_string(),
            typical_committee_roles: vec![
                "scene".to_string(),
                "action".to_string(),
                "knowledge".to_string(),
            ],
            default_handoff_out: vec!["action".to_string()],
            risk_flags: vec!["signal_collision".to_string()],
            continuity_priority: "high".to_string(),
            prompt_focus: vec![
                "taxonomy emphasis".to_string(),
                "knowledge signal".to_string(),
            ],
            source_type: "team_curated".to_string(),
            source_notes: "collision test fixture".to_string(),
            confidence_level: "high".to_string(),
            last_reviewed_at: "2026-04-20".to_string(),
        }
    }

    fn conflicting_signal_taxonomy() -> SceneTaxonomyRecord {
        SceneTaxonomyRecord {
            scene_taxonomy_id: "scene-taxonomy-conflict-001".to_string(),
            scene_type: "knowledge_conflict".to_string(),
            display_name: "knowledge conflict".to_string(),
            definition: "knowledge-side signals attempt to conflict with runtime inputs"
                .to_string(),
            default_duration_band: "90s-120s".to_string(),
            typical_committee_roles: vec![
                "taxonomy_override".to_string(),
                "knowledge".to_string(),
                "scene".to_string(),
            ],
            default_handoff_out: vec!["knowledge".to_string()],
            risk_flags: vec![
                "signal_collision".to_string(),
                "runtime_override_attempt".to_string(),
            ],
            continuity_priority: "low".to_string(),
            prompt_focus: vec![
                "taxonomy override attempt".to_string(),
                "conflicting knowledge emphasis".to_string(),
            ],
            source_type: "team_curated".to_string(),
            source_notes: "conflict test fixture".to_string(),
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
    fn scene_taxonomy_stays_metadata_when_runtime_inputs_are_explicit() {
        let provided_director = "director-profile-week3-001".to_string();
        let layout_prompt = "冷色调，中景".to_string();
        let render_prompt = "克制写实".to_string();
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
            scene_director_id: Some(provided_director.clone()),
            action_director_id: None,
            scene_taxonomy: Some(daily_dialogue_taxonomy()),
            layout_prompt: layout_prompt.clone(),
            render_prompt: render_prompt.clone(),
        })
        .expect("explicit runtime inputs should stay valid");

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
            provided_director
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_action_director_id,
            "director-profile-week3-001"
        );
        assert!(
            plan.committee_runtime.prompt_layers.layout_prompt == layout_prompt
        );
        assert!(
            plan.committee_runtime.prompt_layers.render_prompt == render_prompt
        );
        assert_eq!(plan.handoff_zone.boundary_type, "render_segment_boundary");
    }

    #[test]
    fn precedence_keeps_distinct_runtime_directors_authoritative() {
        let scene_director = "director-scene-runtime-001".to_string();
        let action_director = "director-action-runtime-001".to_string();
        let plan = build_storyboard_plan(StoryboardPlanRequest {
            render_segment_id: "render-segment-precedence-001".to_string(),
            narrative_scene_id: "narrative-scene-precedence-001".to_string(),
            render_segment_sequence_no: 2,
            start_shot_sequence_no: 20,
            end_shot_sequence_no: 22,
            target_duration_seconds: 60,
            cut_id: "cut-precedence-001".to_string(),
            cut_sequence_no: 1,
            shot_description: "medium shot dialogue".to_string(),
            dialogue: "runtime inputs should stay in control".to_string(),
            scene_director_id: Some(scene_director.clone()),
            action_director_id: Some(action_director.clone()),
            scene_taxonomy: Some(daily_dialogue_taxonomy()),
            layout_prompt: "runtime layout stays frozen".to_string(),
            render_prompt: "runtime render stays frozen".to_string(),
        })
        .expect("distinct runtime directors should remain authoritative");

        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_scene_director_id,
            scene_director
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_action_director_id,
            action_director
        );
        assert_eq!(
            plan.committee_runtime.scene_taxonomy_id.as_deref(),
            Some("scene-taxonomy-daily-dialogue")
        );
        assert_eq!(
            plan.committee_runtime.scene_type.as_deref(),
            Some("daily_dialogue")
        );
    }

    #[test]
    fn precedence_duplicates_runtime_action_director_before_taxonomy_can_interfere() {
        let action_director = "director-action-runtime-002".to_string();
        let plan = build_storyboard_plan(StoryboardPlanRequest {
            render_segment_id: "render-segment-precedence-002".to_string(),
            narrative_scene_id: "narrative-scene-precedence-002".to_string(),
            render_segment_sequence_no: 3,
            start_shot_sequence_no: 30,
            end_shot_sequence_no: 33,
            target_duration_seconds: 60,
            cut_id: "cut-precedence-002".to_string(),
            cut_sequence_no: 1,
            shot_description: "tracked motion beat".to_string(),
            dialogue: "action runtime input should duplicate to both slots".to_string(),
            scene_director_id: None,
            action_director_id: Some(action_director.clone()),
            scene_taxonomy: Some(daily_dialogue_taxonomy()),
            layout_prompt: "layout prompt stays runtime-owned".to_string(),
            render_prompt: "render prompt stays runtime-owned".to_string(),
        })
        .expect("single runtime action director should satisfy the frozen contract");

        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_scene_director_id,
            action_director
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_action_director_id,
            "director-action-runtime-002"
        );
    }

    #[test]
    fn precedence_keeps_runtime_prompt_layers_and_handoff_boundaries() {
        let layout_prompt = "runtime layout prompt remains authoritative".to_string();
        let render_prompt = "runtime render prompt remains authoritative".to_string();
        let plan = build_storyboard_plan(StoryboardPlanRequest {
            render_segment_id: "render-segment-precedence-003".to_string(),
            narrative_scene_id: "narrative-scene-precedence-003".to_string(),
            render_segment_sequence_no: 4,
            start_shot_sequence_no: 41,
            end_shot_sequence_no: 44,
            target_duration_seconds: 60,
            cut_id: "cut-precedence-003".to_string(),
            cut_sequence_no: 1,
            shot_description: "wide to medium transition".to_string(),
            dialogue: "knowledge-side signals must not rewrite runtime boundaries".to_string(),
            scene_director_id: Some("director-scene-runtime-003".to_string()),
            action_director_id: Some("director-action-runtime-003".to_string()),
            scene_taxonomy: Some(daily_dialogue_taxonomy()),
            layout_prompt: layout_prompt.clone(),
            render_prompt: render_prompt.clone(),
        })
        .expect("runtime prompt layers and handoff boundaries should stay frozen");

        assert_eq!(plan.committee_runtime.prompt_layers.layout_prompt, layout_prompt);
        assert_eq!(plan.committee_runtime.prompt_layers.render_prompt, render_prompt);
        assert_eq!(plan.handoff_zone.start_boundary, "41");
        assert_eq!(plan.handoff_zone.end_boundary, "44");
        assert_eq!(plan.handoff_zone.boundary_type, "render_segment_boundary");
    }

    #[test]
    fn collision_keeps_runtime_director_assignment_authoritative() {
        let scene_director = "director-scene-runtime-collision-001".to_string();
        let action_director = "director-action-runtime-collision-001".to_string();
        let plan = build_storyboard_plan(StoryboardPlanRequest {
            render_segment_id: "render-segment-collision-001".to_string(),
            narrative_scene_id: "narrative-scene-collision-001".to_string(),
            render_segment_sequence_no: 5,
            start_shot_sequence_no: 50,
            end_shot_sequence_no: 54,
            target_duration_seconds: 60,
            cut_id: "cut-collision-001".to_string(),
            cut_sequence_no: 1,
            shot_description: "runtime and taxonomy signals overlap".to_string(),
            dialogue: "runtime directors must remain authoritative".to_string(),
            scene_director_id: Some(scene_director.clone()),
            action_director_id: Some(action_director.clone()),
            scene_taxonomy: Some(overlapping_signal_taxonomy()),
            layout_prompt: "runtime-owned layout prompt".to_string(),
            render_prompt: "runtime-owned render prompt".to_string(),
        })
        .expect("runtime directors should remain stable under overlapping signals");

        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_scene_director_id,
            scene_director
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_action_director_id,
            action_director
        );
        assert_eq!(
            plan.committee_runtime.scene_taxonomy_id.as_deref(),
            Some("scene-taxonomy-overlap-001")
        );
        assert_eq!(
            plan.committee_runtime.scene_type.as_deref(),
            Some("knowledge_overlap")
        );
    }

    #[test]
    fn collision_keeps_runtime_prompt_layers_stable() {
        let layout_prompt = "runtime layout survives collision".to_string();
        let render_prompt = "runtime render survives collision".to_string();
        let plan = build_storyboard_plan(StoryboardPlanRequest {
            render_segment_id: "render-segment-collision-002".to_string(),
            narrative_scene_id: "narrative-scene-collision-002".to_string(),
            render_segment_sequence_no: 6,
            start_shot_sequence_no: 60,
            end_shot_sequence_no: 63,
            target_duration_seconds: 60,
            cut_id: "cut-collision-002".to_string(),
            cut_sequence_no: 1,
            shot_description: "taxonomy prompt hints overlap with runtime text".to_string(),
            dialogue: "prompt layers should stay literal and stable".to_string(),
            scene_director_id: Some("director-scene-runtime-collision-002".to_string()),
            action_director_id: Some("director-action-runtime-collision-002".to_string()),
            scene_taxonomy: Some(overlapping_signal_taxonomy()),
            layout_prompt: layout_prompt.clone(),
            render_prompt: render_prompt.clone(),
        })
        .expect("runtime prompt layers should stay authoritative under collisions");

        assert_eq!(plan.committee_runtime.prompt_layers.layout_prompt, layout_prompt);
        assert_eq!(plan.committee_runtime.prompt_layers.render_prompt, render_prompt);
    }

    #[test]
    fn collision_keeps_runtime_handoff_boundary_stable() {
        let plan = build_storyboard_plan(StoryboardPlanRequest {
            render_segment_id: "render-segment-collision-003".to_string(),
            narrative_scene_id: "narrative-scene-collision-003".to_string(),
            render_segment_sequence_no: 7,
            start_shot_sequence_no: 70,
            end_shot_sequence_no: 75,
            target_duration_seconds: 75,
            cut_id: "cut-collision-003".to_string(),
            cut_sequence_no: 1,
            shot_description: "continuity and handoff hints overlap".to_string(),
            dialogue: "handoff boundary must remain runtime-stable".to_string(),
            scene_director_id: Some("director-scene-runtime-collision-003".to_string()),
            action_director_id: None,
            scene_taxonomy: Some(overlapping_signal_taxonomy()),
            layout_prompt: "handoff layout stays runtime-owned".to_string(),
            render_prompt: "handoff render stays runtime-owned".to_string(),
        })
        .expect("runtime handoff boundary should stay stable under collisions");

        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_scene_director_id,
            "director-scene-runtime-collision-003"
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_action_director_id,
            "director-scene-runtime-collision-003"
        );
        assert_eq!(plan.handoff_zone.start_boundary, "70");
        assert_eq!(plan.handoff_zone.end_boundary, "75");
        assert_eq!(plan.handoff_zone.boundary_type, "render_segment_boundary");
    }

    #[test]
    fn taxonomy_variants_only_change_metadata_visibility() {
        let variants = vec![
            (
                "none",
                None,
                None,
                None,
                None,
            ),
            (
                "daily",
                Some(daily_dialogue_taxonomy()),
                Some("scene-taxonomy-daily-dialogue"),
                Some("daily_dialogue"),
                Some("high"),
            ),
            (
                "overlap",
                Some(overlapping_signal_taxonomy()),
                Some("scene-taxonomy-overlap-001"),
                Some("knowledge_overlap"),
                Some("high"),
            ),
            (
                "conflict",
                Some(conflicting_signal_taxonomy()),
                Some("scene-taxonomy-conflict-001"),
                Some("knowledge_conflict"),
                Some("low"),
            ),
        ];

        for (
            label,
            taxonomy,
            expected_taxonomy_id,
            expected_scene_type,
            expected_continuity_priority,
        ) in variants
        {
            let plan = build_storyboard_plan(StoryboardPlanRequest {
                render_segment_id: format!("render-segment-variant-{label}"),
                narrative_scene_id: format!("narrative-scene-variant-{label}"),
                render_segment_sequence_no: 11,
                start_shot_sequence_no: 110,
                end_shot_sequence_no: 114,
                target_duration_seconds: 60,
                cut_id: format!("cut-variant-{label}"),
                cut_sequence_no: 1,
                shot_description: "runtime request should stay stable across taxonomy variants"
                    .to_string(),
                dialogue: format!("taxonomy variant {label} must not change runtime output"),
                scene_director_id: Some("director-scene-runtime-variant-001".to_string()),
                action_director_id: Some("director-action-runtime-variant-001".to_string()),
                scene_taxonomy: taxonomy,
                layout_prompt: "runtime layout stays identical across variants".to_string(),
                render_prompt: "runtime render stays identical across variants".to_string(),
            })
            .expect("taxonomy variant should preserve runtime-authoritative outputs");

            assert_eq!(
                plan.committee_runtime
                    .director_assignment
                    .primary_scene_director_id,
                "director-scene-runtime-variant-001",
                "scene director changed under taxonomy variant {label}"
            );
            assert_eq!(
                plan.committee_runtime
                    .director_assignment
                    .primary_action_director_id,
                "director-action-runtime-variant-001",
                "action director changed under taxonomy variant {label}"
            );
            assert_eq!(
                plan.committee_runtime.prompt_layers.layout_prompt,
                "runtime layout stays identical across variants",
                "layout prompt changed under taxonomy variant {label}"
            );
            assert_eq!(
                plan.committee_runtime.prompt_layers.render_prompt,
                "runtime render stays identical across variants",
                "render prompt changed under taxonomy variant {label}"
            );
            assert_eq!(
                plan.handoff_zone.start_boundary, "110",
                "handoff start changed under taxonomy variant {label}"
            );
            assert_eq!(
                plan.handoff_zone.end_boundary, "114",
                "handoff end changed under taxonomy variant {label}"
            );
            assert_eq!(
                plan.handoff_zone.boundary_type, "render_segment_boundary",
                "handoff boundary type changed under taxonomy variant {label}"
            );
            assert_eq!(
                plan.render_segment.target_duration_seconds, 60,
                "target duration changed under taxonomy variant {label}"
            );
            assert_eq!(
                plan.render_segment.actual_duration_seconds,
                Some(60),
                "actual duration changed under taxonomy variant {label}"
            );
            assert_eq!(
                plan.render_segment.start_shot_sequence_no, 110,
                "shot window start changed under taxonomy variant {label}"
            );
            assert_eq!(
                plan.render_segment.end_shot_sequence_no, 114,
                "shot window end changed under taxonomy variant {label}"
            );
            assert_eq!(
                plan.render_segment.scene_taxonomy_id.as_deref(),
                expected_taxonomy_id,
                "taxonomy id visibility mismatch for variant {label}"
            );
            assert_eq!(
                plan.render_segment.scene_type.as_deref(),
                expected_scene_type,
                "scene type visibility mismatch for variant {label}"
            );
            assert_eq!(
                plan.render_segment.continuity_priority.as_deref(),
                expected_continuity_priority,
                "continuity priority visibility mismatch for variant {label}"
            );
            assert_eq!(
                plan.committee_runtime.scene_taxonomy_id.as_deref(),
                expected_taxonomy_id,
                "runtime taxonomy id visibility mismatch for variant {label}"
            );
            assert_eq!(
                plan.committee_runtime.scene_type.as_deref(),
                expected_scene_type,
                "runtime scene type visibility mismatch for variant {label}"
            );
        }
    }

    #[test]
    fn bad_duration_input_still_errors_when_taxonomy_exists() {
        let variants = vec![
            ("daily", daily_dialogue_taxonomy()),
            ("overlap", overlapping_signal_taxonomy()),
            ("conflict", conflicting_signal_taxonomy()),
        ];

        for (label, taxonomy) in variants {
            let err = build_storyboard_plan(StoryboardPlanRequest {
                render_segment_id: format!("render-segment-invalid-duration-{label}"),
                narrative_scene_id: format!("narrative-scene-invalid-duration-{label}"),
                render_segment_sequence_no: 12,
                start_shot_sequence_no: 120,
                end_shot_sequence_no: 124,
                target_duration_seconds: 91,
                cut_id: format!("cut-invalid-duration-{label}"),
                cut_sequence_no: 1,
                shot_description: "duration boundary must remain strict".to_string(),
                dialogue: "taxonomy must not wash out invalid duration".to_string(),
                scene_director_id: Some("director-scene-runtime-invalid-duration".to_string()),
                action_director_id: Some("director-action-runtime-invalid-duration".to_string()),
                scene_taxonomy: Some(taxonomy),
                layout_prompt: "runtime layout invalid duration".to_string(),
                render_prompt: "runtime render invalid duration".to_string(),
            })
            .expect_err("invalid duration should still fail when taxonomy exists");

            assert!(matches!(
                err,
                StoryboardPlanningError::RenderSegmentDurationOutOfRange {
                    target_duration_seconds: 91,
                    min_seconds: RENDER_SEGMENT_TARGET_MIN_SECONDS,
                    max_seconds: RENDER_SEGMENT_TARGET_MAX_SECONDS,
                }
            ));
        }
    }

    #[test]
    fn bad_shot_window_still_errors_when_taxonomy_exists() {
        let variants = vec![
            ("daily", daily_dialogue_taxonomy()),
            ("overlap", overlapping_signal_taxonomy()),
            ("conflict", conflicting_signal_taxonomy()),
        ];

        for (label, taxonomy) in variants {
            let err = build_storyboard_plan(StoryboardPlanRequest {
                render_segment_id: format!("render-segment-invalid-shot-{label}"),
                narrative_scene_id: format!("narrative-scene-invalid-shot-{label}"),
                render_segment_sequence_no: 13,
                start_shot_sequence_no: 133,
                end_shot_sequence_no: 132,
                target_duration_seconds: 60,
                cut_id: format!("cut-invalid-shot-{label}"),
                cut_sequence_no: 1,
                shot_description: "shot window boundary must remain strict".to_string(),
                dialogue: "taxonomy must not wash out invalid shot windows".to_string(),
                scene_director_id: Some("director-scene-runtime-invalid-shot".to_string()),
                action_director_id: Some("director-action-runtime-invalid-shot".to_string()),
                scene_taxonomy: Some(taxonomy),
                layout_prompt: "runtime layout invalid shot window".to_string(),
                render_prompt: "runtime render invalid shot window".to_string(),
            })
            .expect_err("invalid shot window should still fail when taxonomy exists");

            assert!(matches!(
                err,
                StoryboardPlanningError::RenderSegmentShotRangeInvalid {
                    start_shot_sequence_no: 133,
                    end_shot_sequence_no: 132,
                }
            ));
        }
    }

    #[test]
    fn negative_boundary_rejects_missing_runtime_directors_without_taxonomy_fallback() {
        let err = build_storyboard_plan(StoryboardPlanRequest {
            render_segment_id: "render-segment-negative-001".to_string(),
            narrative_scene_id: "narrative-scene-negative-001".to_string(),
            render_segment_sequence_no: 8,
            start_shot_sequence_no: 80,
            end_shot_sequence_no: 84,
            target_duration_seconds: 60,
            cut_id: "cut-negative-001".to_string(),
            cut_sequence_no: 1,
            shot_description: "missing directors with knowledge-side overlap".to_string(),
            dialogue: "taxonomy signals must not become runtime fallback".to_string(),
            scene_director_id: None,
            action_director_id: None,
            scene_taxonomy: Some(overlapping_signal_taxonomy()),
            layout_prompt: "runtime layout missing directors".to_string(),
            render_prompt: "runtime render missing directors".to_string(),
        })
        .expect_err("missing runtime directors should fail instead of falling back");

        assert!(matches!(err, StoryboardPlanningError::MissingDirectorAssignment));
    }

    #[test]
    fn negative_boundary_keeps_conflicting_runtime_prompts_literal_and_boundary_frozen() {
        let layout_prompt =
            "runtime layout says hold close-up while taxonomy suggests other emphasis".to_string();
        let render_prompt =
            "runtime render says suppress taxonomy wording and keep runtime output literal"
                .to_string();
        let plan = build_storyboard_plan(StoryboardPlanRequest {
            render_segment_id: "render-segment-negative-002".to_string(),
            narrative_scene_id: "narrative-scene-negative-002".to_string(),
            render_segment_sequence_no: 9,
            start_shot_sequence_no: 90,
            end_shot_sequence_no: 94,
            target_duration_seconds: 60,
            cut_id: "cut-negative-002".to_string(),
            cut_sequence_no: 1,
            shot_description: "conflicting runtime and knowledge-side prompt signals".to_string(),
            dialogue: "conflicting text must stay runtime-owned".to_string(),
            scene_director_id: Some("director-scene-runtime-negative-002".to_string()),
            action_director_id: Some("director-action-runtime-negative-002".to_string()),
            scene_taxonomy: Some(overlapping_signal_taxonomy()),
            layout_prompt: layout_prompt.clone(),
            render_prompt: render_prompt.clone(),
        })
        .expect("conflicting runtime prompts should still remain authoritative");

        assert_eq!(plan.committee_runtime.prompt_layers.layout_prompt, layout_prompt);
        assert_eq!(plan.committee_runtime.prompt_layers.render_prompt, render_prompt);
        assert_eq!(plan.handoff_zone.start_boundary, "90");
        assert_eq!(plan.handoff_zone.end_boundary, "94");
        assert_eq!(plan.handoff_zone.boundary_type, "render_segment_boundary");
    }

    #[test]
    fn negative_boundary_keeps_degraded_runtime_prompts_without_taxonomy_rewrite() {
        let layout_prompt = String::new();
        let render_prompt = " ".to_string();
        let plan = build_storyboard_plan(StoryboardPlanRequest {
            render_segment_id: "render-segment-negative-003".to_string(),
            narrative_scene_id: "narrative-scene-negative-003".to_string(),
            render_segment_sequence_no: 10,
            start_shot_sequence_no: 100,
            end_shot_sequence_no: 102,
            target_duration_seconds: 45,
            cut_id: "cut-negative-003".to_string(),
            cut_sequence_no: 1,
            shot_description: "degraded runtime prompts under knowledge-side overlap".to_string(),
            dialogue: "degraded prompt input must not trigger taxonomy rewrite".to_string(),
            scene_director_id: Some("director-scene-runtime-negative-003".to_string()),
            action_director_id: None,
            scene_taxonomy: Some(overlapping_signal_taxonomy()),
            layout_prompt: layout_prompt.clone(),
            render_prompt: render_prompt.clone(),
        })
        .expect("degraded runtime prompts should still stay literal");

        assert_eq!(plan.committee_runtime.prompt_layers.layout_prompt, layout_prompt);
        assert_eq!(plan.committee_runtime.prompt_layers.render_prompt, render_prompt);
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_scene_director_id,
            "director-scene-runtime-negative-003"
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_action_director_id,
            "director-scene-runtime-negative-003"
        );
        assert_eq!(plan.handoff_zone.boundary_type, "render_segment_boundary");
    }

    #[test]
    fn storyboard_plan_rejects_missing_directors_even_with_taxonomy() {
        let err = build_storyboard_plan(StoryboardPlanRequest {
            render_segment_id: "render-segment-taxonomy-002".to_string(),
            narrative_scene_id: "narrative-scene-taxonomy-002".to_string(),
            render_segment_sequence_no: 1,
            start_shot_sequence_no: 10,
            end_shot_sequence_no: 12,
            target_duration_seconds: 45,
            cut_id: "cut-taxonomy-002".to_string(),
            cut_sequence_no: 1,
            shot_description: "中景对话".to_string(),
            dialogue: "这一段需要保持冻结契约。".to_string(),
            scene_director_id: None,
            action_director_id: None,
            scene_taxonomy: Some(daily_dialogue_taxonomy()),
            layout_prompt: "冷色调，中景".to_string(),
            render_prompt: "克制写实".to_string(),
        })
        .expect_err("missing directors should fail under the frozen contract");

        assert!(matches!(err, StoryboardPlanningError::MissingDirectorAssignment));
    }
}
