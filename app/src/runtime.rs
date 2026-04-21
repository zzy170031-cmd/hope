use std::io;

use crate::state::AppState;
use project_store::{
    KbRuntimeError, SnapshotBootstrapCheckpointArtifacts, StoreSkeleton,
    bootstrap_verified_kb_context_from_checkpoint,
};

use storyboard_pipeline::{StoryboardPlan, StoryboardPlanRequest, StoryboardPlanningError};
use validators::{
    RepairRecommendation, WEEK3_SHARED_FIXTURE_PATH, Week3SharedFixture,
    generate_week3_repair_recommendations, generate_week3_validation_report,
    load_week3_shared_fixture,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryboardPreviewPlanRequest {
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
    pub scene_type: Option<String>,
    pub layout_prompt: String,
    pub render_prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationExportPanelSnapshotRequest {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationExportPanelSnapshot {
    pub project_id: String,
    pub summary_items: Vec<ValidationExportPanelItem>,
    pub repair_recommendations: Vec<ValidationRepairRecommendationItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationExportPanelItem {
    pub label: String,
    pub value: String,
    pub state: ValidationExportPanelState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationExportPanelState {
    Ready,
    Pending,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationRepairRecommendationItem {
    pub failure_code: String,
    pub failure_name: String,
    pub repair_strategy: String,
    pub repair_priority: String,
    pub repair_scope: String,
    pub validator_hint: String,
    pub prompt_template_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotBootstrapReadonlyState {
    pub snapshot_identity: SnapshotBootstrapSnapshotIdentity,
    pub summary_capabilities: SnapshotBootstrapSummaryCapabilities,
    pub knowledge_bundle: SnapshotBootstrapKnowledgeBundleStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotBootstrapSnapshotIdentity {
    pub snapshot_id: String,
    pub snapshot_hash: String,
    pub seed_format: String,
    pub source_name: String,
    pub created_at_timestamp: i64,
    pub snapshot_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotBootstrapSummaryCapabilities {
    pub has_scene_taxonomy: bool,
    pub has_failure_patterns: bool,
    pub has_repair_template_mapping: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotBootstrapKnowledgeBundleStatus {
    pub scene_taxonomy_count: usize,
    pub failure_pattern_count: usize,
    pub prompt_template_count: usize,
    pub scene_taxonomies_ready: bool,
    pub failure_patterns_ready: bool,
    pub prompt_templates_ready: bool,
    pub repair_mappings_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationFeedbackReadonlyState {
    pub source_snapshot_id: String,
    pub source_snapshot_hash: String,
    pub source_snapshot_path: String,
    pub has_failure_patterns: bool,
    pub has_repair_template_mapping: bool,
    pub failure_pattern_count: usize,
    pub prompt_template_count: usize,
    pub repair_mapping_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoryboardPreviewPlanFromCheckpointError {
    Bootstrap(KbRuntimeError),
    Planning(StoryboardPlanningError),
}

pub fn bootstrap_app_state_from_checkpoint(
    store: StoreSkeleton,
    checkpoint_artifacts: SnapshotBootstrapCheckpointArtifacts,
) -> Result<AppState, KbRuntimeError> {
    let context = bootstrap_verified_kb_context_from_checkpoint(checkpoint_artifacts)?;

    Ok(AppState::new(store, context.runtime, context.bundle))
}

pub fn bootstrap_snapshot_bootstrap_readonly_state_from_checkpoint(
    store: StoreSkeleton,
    checkpoint_artifacts: SnapshotBootstrapCheckpointArtifacts,
) -> Result<SnapshotBootstrapReadonlyState, KbRuntimeError> {
    let state = bootstrap_app_state_from_checkpoint(store, checkpoint_artifacts)?;

    Ok(snapshot_bootstrap_readonly_state_from_verified_app_state(
        &state,
    ))
}

pub fn snapshot_bootstrap_readonly_state_from_verified_app_state(
    state: &AppState,
) -> SnapshotBootstrapReadonlyState {
    let scene_taxonomy_count = state.kb_knowledge.scene_taxonomies.len();
    let failure_pattern_count = state.kb_knowledge.failure_patterns.len();
    let prompt_template_count = state.kb_knowledge.prompt_templates.len();
    let summary = &state.kb_runtime.summary;

    SnapshotBootstrapReadonlyState {
        snapshot_identity: SnapshotBootstrapSnapshotIdentity {
            snapshot_id: state.kb_runtime.snapshot.snapshot_id.clone(),
            snapshot_hash: state.kb_runtime.snapshot.snapshot_hash.clone(),
            seed_format: state.kb_runtime.snapshot.seed_format.clone(),
            source_name: state.kb_runtime.snapshot.source_name.clone(),
            created_at_timestamp: state.kb_runtime.snapshot.created_at_timestamp,
            snapshot_path: summary.snapshot_path.clone(),
        },
        summary_capabilities: SnapshotBootstrapSummaryCapabilities {
            has_scene_taxonomy: summary.has_scene_taxonomy,
            has_failure_patterns: summary.has_failure_patterns,
            has_repair_template_mapping: summary.has_repair_template_mapping,
        },
        knowledge_bundle: SnapshotBootstrapKnowledgeBundleStatus {
            scene_taxonomy_count,
            failure_pattern_count,
            prompt_template_count,
            scene_taxonomies_ready: summary.has_scene_taxonomy && scene_taxonomy_count > 0,
            failure_patterns_ready: summary.has_failure_patterns && failure_pattern_count > 0,
            prompt_templates_ready: summary.has_repair_template_mapping
                && prompt_template_count > 0,
            repair_mappings_ready: summary.has_repair_template_mapping
                && failure_pattern_count > 0
                && prompt_template_count > 0,
        },
    }
}

pub fn bootstrap_validation_feedback_readonly_state_from_checkpoint(
    store: StoreSkeleton,
    checkpoint_artifacts: SnapshotBootstrapCheckpointArtifacts,
) -> Result<ValidationFeedbackReadonlyState, KbRuntimeError> {
    let state = bootstrap_app_state_from_checkpoint(store, checkpoint_artifacts)?;

    Ok(validation_feedback_readonly_state_from_verified_app_state(
        &state,
    ))
}

pub fn validation_feedback_readonly_state_from_verified_app_state(
    state: &AppState,
) -> ValidationFeedbackReadonlyState {
    let failure_pattern_count = state.kb_knowledge.failure_patterns.len();
    let prompt_template_count = state.kb_knowledge.prompt_templates.len();
    let summary = &state.kb_runtime.summary;

    ValidationFeedbackReadonlyState {
        source_snapshot_id: state.kb_runtime.snapshot.snapshot_id.clone(),
        source_snapshot_hash: state.kb_runtime.snapshot.snapshot_hash.clone(),
        source_snapshot_path: summary.snapshot_path.clone(),
        has_failure_patterns: summary.has_failure_patterns,
        has_repair_template_mapping: summary.has_repair_template_mapping,
        failure_pattern_count,
        prompt_template_count,
        repair_mapping_ready: summary.has_repair_template_mapping
            && failure_pattern_count > 0
            && prompt_template_count > 0,
    }
}

pub fn build_storyboard_preview_plan_from_checkpoint(
    store: StoreSkeleton,
    checkpoint_artifacts: SnapshotBootstrapCheckpointArtifacts,
    request: StoryboardPreviewPlanRequest,
) -> Result<StoryboardPlan, StoryboardPreviewPlanFromCheckpointError> {
    let state = bootstrap_app_state_from_checkpoint(store, checkpoint_artifacts)
        .map_err(StoryboardPreviewPlanFromCheckpointError::Bootstrap)?;

    build_storyboard_preview_plan(&state, request)
        .map_err(StoryboardPreviewPlanFromCheckpointError::Planning)
}

pub fn build_storyboard_preview_plan(
    state: &AppState,
    request: StoryboardPreviewPlanRequest,
) -> Result<StoryboardPlan, StoryboardPlanningError> {
    let scene_taxonomy = resolve_scene_taxonomy(state, request.scene_type.as_deref());
    let derived_scene_director_id = request.scene_director_id.clone().or_else(|| {
        scene_taxonomy
            .as_ref()
            .map(|taxonomy| format!("taxonomy:{}:scene", taxonomy.scene_taxonomy_id))
    });
    let derived_action_director_id = request.action_director_id.clone().or_else(|| {
        scene_taxonomy
            .as_ref()
            .map(|taxonomy| format!("taxonomy:{}:action", taxonomy.scene_taxonomy_id))
    });
    let layout_prompt = if let Some(taxonomy) = scene_taxonomy.as_ref() {
        let layout_tag = format!(
            "\u{573A}\u{666F}\u{5206}\u{7C7B}\u{FF1A}{}",
            taxonomy.scene_type
        );

        if request.layout_prompt.contains(&layout_tag) {
            request.layout_prompt.clone()
        } else {
            format!("{}\n{}", request.layout_prompt, layout_tag)
        }
    } else {
        request.layout_prompt.clone()
    };
    let render_prompt = if let Some(taxonomy) = scene_taxonomy.as_ref() {
        let continuity_tag = format!(
            "\u{8FDE}\u{7EED}\u{6027}\u{4F18}\u{5148}\u{7EA7}\u{FF1A}{}",
            taxonomy.continuity_priority
        );

        if request.render_prompt.contains(&continuity_tag) {
            request.render_prompt.clone()
        } else {
            format!("{}\n{}", request.render_prompt, continuity_tag)
        }
    } else {
        request.render_prompt.clone()
    };

    storyboard_pipeline::build_storyboard_plan(StoryboardPlanRequest {
        render_segment_id: request.render_segment_id,
        narrative_scene_id: request.narrative_scene_id,
        render_segment_sequence_no: request.render_segment_sequence_no,
        start_shot_sequence_no: request.start_shot_sequence_no,
        end_shot_sequence_no: request.end_shot_sequence_no,
        target_duration_seconds: request.target_duration_seconds,
        cut_id: request.cut_id,
        cut_sequence_no: request.cut_sequence_no,
        shot_description: request.shot_description,
        dialogue: request.dialogue,
        scene_director_id: derived_scene_director_id,
        action_director_id: derived_action_director_id,
        scene_taxonomy,
        layout_prompt,
        render_prompt,
    })
}

pub fn build_validation_export_panel_snapshot(
    state: &AppState,
    request: ValidationExportPanelSnapshotRequest,
) -> io::Result<ValidationExportPanelSnapshot> {
    let fixture = load_week3_shared_fixture(WEEK3_SHARED_FIXTURE_PATH)?;
    build_validation_export_panel_snapshot_from_fixture(state, request, &fixture)
}

pub fn resolve_scene_taxonomy(
    state: &AppState,
    scene_type: Option<&str>,
) -> Option<core_domain::SceneTaxonomyRecord> {
    let scene_type = scene_type?;

    state
        .kb_knowledge
        .scene_taxonomies
        .iter()
        .find(|taxonomy| {
            taxonomy.scene_type == scene_type
                || taxonomy.display_name == scene_type
                || taxonomy.scene_taxonomy_id == scene_type
        })
        .cloned()
}

fn build_validation_export_panel_snapshot_from_fixture(
    state: &AppState,
    request: ValidationExportPanelSnapshotRequest,
    fixture: &Week3SharedFixture,
) -> io::Result<ValidationExportPanelSnapshot> {
    let workbook = generate_week3_validation_report(fixture)?;
    let repair_recommendations = generate_week3_repair_recommendations(
        fixture,
        &state.kb_knowledge.failure_patterns,
        &state.kb_knowledge.prompt_templates,
    )?;

    let validation_row_count = workbook.validation_report.len();
    let block_count = workbook
        .validation_report
        .iter()
        .filter(|row| !row.passed)
        .count();
    let validation_state = if block_count > 0 {
        ValidationExportPanelState::Blocked
    } else {
        ValidationExportPanelState::Ready
    };
    let repair_state = if repair_recommendations.is_empty() {
        ValidationExportPanelState::Pending
    } else {
        ValidationExportPanelState::Ready
    };

    let summary_items = vec![
        ValidationExportPanelItem {
            label: "project".to_string(),
            value: request.project_id.clone(),
            state: ValidationExportPanelState::Ready,
        },
        ValidationExportPanelItem {
            label: "validation_report".to_string(),
            value: format!("{} rows / {} blocked", validation_row_count, block_count),
            state: validation_state,
        },
        ValidationExportPanelItem {
            label: "repair_recommendations".to_string(),
            value: format!(
                "{} kb-backed recommendations / snapshot {}",
                repair_recommendations.len(),
                state.kb_runtime.snapshot.snapshot_id
            ),
            state: repair_state,
        },
    ];

    let repair_recommendations = repair_recommendations
        .into_iter()
        .map(map_repair_recommendation)
        .collect();

    Ok(ValidationExportPanelSnapshot {
        project_id: request.project_id,
        summary_items,
        repair_recommendations,
    })
}

fn map_repair_recommendation(
    recommendation: RepairRecommendation,
) -> ValidationRepairRecommendationItem {
    ValidationRepairRecommendationItem {
        failure_code: recommendation.failure_code,
        failure_name: recommendation.failure_name,
        repair_strategy: recommendation.repair_strategy,
        repair_priority: recommendation.repair_priority,
        repair_scope: recommendation.repair_scope,
        validator_hint: recommendation.validator_hint,
        prompt_template_names: recommendation
            .prompt_templates
            .into_iter()
            .filter_map(|link| link.prompt_name)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use core_domain::{
        FailurePatternRecord, KbRuntimeSummary, KbSnapshotRecord, PromptTemplateRecord,
        SceneTaxonomyRecord,
    };
    use project_store::{
        DualSqliteConnectionPolicy, KbKnowledgeBundle, KbRuntimeError, KbRuntimeHandle,
        SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH, SnapshotBootstrapCheckpointArtifacts,
        StoreSkeleton,
    };

    use super::{
        StoryboardPreviewPlanFromCheckpointError, StoryboardPreviewPlanRequest,
        ValidationExportPanelSnapshotRequest, ValidationExportPanelState,
        bootstrap_app_state_from_checkpoint,
        bootstrap_snapshot_bootstrap_readonly_state_from_checkpoint,
        bootstrap_validation_feedback_readonly_state_from_checkpoint,
        build_storyboard_preview_plan, build_storyboard_preview_plan_from_checkpoint,
        build_validation_export_panel_snapshot_from_fixture, resolve_scene_taxonomy,
        snapshot_bootstrap_readonly_state_from_verified_app_state,
        validation_feedback_readonly_state_from_verified_app_state,
    };
    use crate::state::AppState;
    use validators::{WEEK3_SHARED_FIXTURE_PATH, load_week3_shared_fixture};

    fn test_state() -> AppState {
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            "E:/codex/hope-kb/snapshots/hope-kb-v0.1.sqlite3".into(),
            "E:/codex/hope/data/hope.sqlite3".into(),
        ));
        let kb_runtime = KbRuntimeHandle {
            snapshot: KbSnapshotRecord {
                snapshot_id: "hope-kb-v0.1".to_string(),
                snapshot_hash: "test-hash".to_string(),
                seed_format: "hope-kb-sqlite-snapshot-v0.1".to_string(),
                source_name: "hope-kb".to_string(),
                created_at_timestamp: 1_714_000_000,
            },
            summary: KbRuntimeSummary {
                snapshot_id: "hope-kb-v0.1".to_string(),
                snapshot_path: "E:/codex/hope-kb/snapshots/hope-kb-v0.1.sqlite3".to_string(),
                mirror_tables: vec!["scene_taxonomy".to_string(), "failure_pattern".to_string()],
                has_scene_taxonomy: true,
                has_failure_patterns: true,
                has_repair_template_mapping: true,
            },
        };
        let kb_knowledge = KbKnowledgeBundle {
            scene_taxonomies: vec![SceneTaxonomyRecord {
                scene_taxonomy_id: "scene-taxonomy-daily-dialogue".to_string(),
                scene_type: "daily_dialogue".to_string(),
                display_name: "Daily Dialogue".to_string(),
                definition: "Stable conversational scene in one space.".to_string(),
                default_duration_band: "30-60s".to_string(),
                typical_committee_roles: vec!["chief".to_string(), "scene".to_string()],
                default_handoff_out: vec!["scene->emotion".to_string()],
                risk_flags: vec!["flat_rhythm".to_string()],
                continuity_priority: "high".to_string(),
                prompt_focus: vec!["micro_expression".to_string(), "blocking".to_string()],
                source_type: "team_distillation".to_string(),
                source_notes: "test fixture".to_string(),
                confidence_level: "high".to_string(),
                last_reviewed_at: "2026-04-20".to_string(),
            }],
            failure_patterns: vec![
                FailurePatternRecord {
                    failure_pattern_id: "failure-style-drift".to_string(),
                    failure_code: "style_drift".to_string(),
                    failure_name: "Style Drift".to_string(),
                    failure_category: "style".to_string(),
                    symptom: "Adjacent cuts drift away from the locked style.".to_string(),
                    common_causes: vec!["hard lock injection missing".to_string()],
                    detection_hint: "Check Style Unity Validator.".to_string(),
                    repair_strategy: "Re-inject hard locks into render prompts.".to_string(),
                    affected_layers: vec!["prompt_packages".to_string()],
                    validator_hint: "Style Unity Validator".to_string(),
                    repair_template_ids: vec!["prompt-repair-style".to_string()],
                    repair_priority: "high".to_string(),
                    repair_scope: "render_prompt_only".to_string(),
                    suggested_followup_validators: vec!["Style Unity Validator".to_string()],
                    source_type: "team_distillation".to_string(),
                    source_notes: "test fixture".to_string(),
                    confidence_level: "high".to_string(),
                    last_reviewed_at: "2026-04-20".to_string(),
                },
                FailurePatternRecord {
                    failure_pattern_id: "failure-prompt-noise".to_string(),
                    failure_code: "chinese_prompt_noise".to_string(),
                    failure_name: "Chinese Prompt Noise".to_string(),
                    failure_category: "language".to_string(),
                    symptom: "Placeholder or machine-like prompt text leaks into output."
                        .to_string(),
                    common_causes: vec!["placeholder marker in prompt".to_string()],
                    detection_hint: "Check prompt body placeholders.".to_string(),
                    repair_strategy: "Rewrite prompt body as natural Chinese guidance.".to_string(),
                    affected_layers: vec!["prompt_packages".to_string()],
                    validator_hint: "Prompt quality review".to_string(),
                    repair_template_ids: vec!["prompt-repair-language".to_string()],
                    repair_priority: "medium".to_string(),
                    repair_scope: "prompt_rendering_layer".to_string(),
                    suggested_followup_validators: vec!["Prompt quality review".to_string()],
                    source_type: "team_distillation".to_string(),
                    source_notes: "test fixture".to_string(),
                    confidence_level: "high".to_string(),
                    last_reviewed_at: "2026-04-20".to_string(),
                },
            ],
            prompt_templates: vec![
                PromptTemplateRecord {
                    prompt_template_id: "prompt-repair-style".to_string(),
                    stage: "repair_pass".to_string(),
                    name: "Repair Style Lock".to_string(),
                    target_model_family: "qwen-compatible".to_string(),
                    repairs_failure_codes: vec!["style_drift".to_string()],
                },
                PromptTemplateRecord {
                    prompt_template_id: "prompt-repair-language".to_string(),
                    stage: "repair_pass".to_string(),
                    name: "Repair Prompt Language".to_string(),
                    target_model_family: "qwen-compatible".to_string(),
                    repairs_failure_codes: vec!["chinese_prompt_noise".to_string()],
                },
            ],
        };

        AppState::new(store, kb_runtime, kb_knowledge)
    }

    fn test_state_without_scene_taxonomy() -> AppState {
        let mut state = test_state();
        state.kb_knowledge.scene_taxonomies.clear();
        state
    }

    fn test_state_without_repair_mappings() -> AppState {
        let mut state = test_state();
        state.kb_knowledge.failure_patterns.clear();
        state.kb_knowledge.prompt_templates.clear();
        state
    }

    #[test]
    fn bootstrap_app_state_from_checkpoint_loads_verified_runtime_into_app_state() {
        let fixture = create_snapshot_bootstrap_fixture();
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            fixture.artifacts.snapshot_path.clone(),
            fixture.root.join("hope.sqlite3"),
        ));

        let state = bootstrap_app_state_from_checkpoint(store.clone(), fixture.artifacts.clone())
            .expect("checkpoint bootstrap should hydrate app state");

        assert_eq!(state.store.connection_policy, store.connection_policy);
        assert_eq!(
            state.kb_runtime.snapshot.snapshot_hash,
            SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
        );
        assert_eq!(
            state.kb_runtime.summary.snapshot_path,
            fixture.artifacts.snapshot_path.display().to_string()
        );
        assert_eq!(state.kb_knowledge.scene_taxonomies.len(), 1);
        assert_eq!(state.kb_knowledge.failure_patterns.len(), 1);
        assert_eq!(state.kb_knowledge.prompt_templates.len(), 1);

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn bootstrap_app_state_from_checkpoint_rejects_missing_trusted_input_without_fallback() {
        let fixture = create_snapshot_bootstrap_fixture();
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            fixture.artifacts.snapshot_path.clone(),
            fixture.root.join("hope.sqlite3"),
        ));
        let missing_path = fixture.packet_root.join("export_templates.json");
        fs::remove_file(&missing_path).expect("trusted input should be removable for the test");

        let error = bootstrap_app_state_from_checkpoint(store, fixture.artifacts.clone())
            .expect_err("missing trusted input should fail without app-side fallback");

        assert_eq!(
            error,
            KbRuntimeError::SnapshotBootstrapInputMissing {
                input_name: "export_template".to_string(),
                path: missing_path,
            }
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn bootstrap_snapshot_bootstrap_readonly_state_from_checkpoint_exposes_verified_contract() {
        let fixture = create_snapshot_bootstrap_fixture();
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            fixture.artifacts.snapshot_path.clone(),
            fixture.root.join("hope.sqlite3"),
        ));

        let readonly_state = bootstrap_snapshot_bootstrap_readonly_state_from_checkpoint(
            store,
            fixture.artifacts.clone(),
        )
        .expect("checkpoint bootstrap should expose readonly state");

        assert_eq!(readonly_state.snapshot_identity.snapshot_id, "hope-kb-v0.1");
        assert_eq!(
            readonly_state.snapshot_identity.snapshot_hash,
            SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
        );
        assert_eq!(
            readonly_state.snapshot_identity.snapshot_path,
            fixture.artifacts.snapshot_path.display().to_string()
        );
        assert_eq!(
            readonly_state.snapshot_identity.seed_format,
            "hope-kb-sqlite-snapshot-v0.1"
        );
        assert_eq!(readonly_state.snapshot_identity.source_name, "hope-kb");
        assert!(readonly_state.summary_capabilities.has_scene_taxonomy);
        assert!(readonly_state.summary_capabilities.has_failure_patterns);
        assert!(
            readonly_state
                .summary_capabilities
                .has_repair_template_mapping
        );
        assert_eq!(readonly_state.knowledge_bundle.scene_taxonomy_count, 1);
        assert_eq!(readonly_state.knowledge_bundle.failure_pattern_count, 1);
        assert_eq!(readonly_state.knowledge_bundle.prompt_template_count, 1);
        assert!(readonly_state.knowledge_bundle.scene_taxonomies_ready);
        assert!(readonly_state.knowledge_bundle.failure_patterns_ready);
        assert!(readonly_state.knowledge_bundle.prompt_templates_ready);
        assert!(readonly_state.knowledge_bundle.repair_mappings_ready);

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn snapshot_bootstrap_readonly_state_from_verified_app_state_uses_verified_state_fields() {
        let fixture = create_snapshot_bootstrap_fixture();
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            fixture.artifacts.snapshot_path.clone(),
            fixture.root.join("hope.sqlite3"),
        ));
        let state = bootstrap_app_state_from_checkpoint(store, fixture.artifacts.clone())
            .expect("checkpoint bootstrap should hydrate app state");

        let readonly_state = snapshot_bootstrap_readonly_state_from_verified_app_state(&state);

        assert_eq!(
            readonly_state.snapshot_identity.snapshot_hash,
            SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
        );
        assert_eq!(
            readonly_state.snapshot_identity.snapshot_path,
            state.kb_runtime.summary.snapshot_path
        );
        assert_eq!(
            readonly_state.knowledge_bundle.scene_taxonomy_count,
            state.kb_knowledge.scene_taxonomies.len()
        );
        assert_eq!(
            readonly_state.knowledge_bundle.failure_pattern_count,
            state.kb_knowledge.failure_patterns.len()
        );
        assert_eq!(
            readonly_state.knowledge_bundle.prompt_template_count,
            state.kb_knowledge.prompt_templates.len()
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn bootstrap_snapshot_bootstrap_readonly_state_from_checkpoint_rejects_missing_trusted_input() {
        let fixture = create_snapshot_bootstrap_fixture();
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            fixture.artifacts.snapshot_path.clone(),
            fixture.root.join("hope.sqlite3"),
        ));
        let missing_path = fixture.packet_root.join("export_templates.json");
        fs::remove_file(&missing_path).expect("trusted input should be removable for the test");

        let error =
            bootstrap_snapshot_bootstrap_readonly_state_from_checkpoint(store, fixture.artifacts)
                .expect_err("readonly state should not add a local fallback");

        assert_eq!(
            error,
            KbRuntimeError::SnapshotBootstrapInputMissing {
                input_name: "export_template".to_string(),
                path: missing_path,
            }
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn bootstrap_validation_feedback_readonly_state_from_checkpoint_exposes_verified_kb_readiness()
    {
        let fixture = create_snapshot_bootstrap_fixture();
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            fixture.artifacts.snapshot_path.clone(),
            fixture.root.join("hope.sqlite3"),
        ));

        let readonly_state = bootstrap_validation_feedback_readonly_state_from_checkpoint(
            store,
            fixture.artifacts.clone(),
        )
        .expect("checkpoint bootstrap should expose validation feedback readiness metadata");

        assert_eq!(readonly_state.source_snapshot_id, "hope-kb-v0.1");
        assert_eq!(
            readonly_state.source_snapshot_hash,
            SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
        );
        assert_eq!(
            readonly_state.source_snapshot_path,
            fixture.artifacts.snapshot_path.display().to_string()
        );
        assert!(readonly_state.has_failure_patterns);
        assert!(readonly_state.has_repair_template_mapping);
        assert_eq!(readonly_state.failure_pattern_count, 1);
        assert_eq!(readonly_state.prompt_template_count, 1);
        assert!(readonly_state.repair_mapping_ready);

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn validation_feedback_readonly_state_from_verified_app_state_uses_verified_state_fields() {
        let fixture = create_snapshot_bootstrap_fixture();
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            fixture.artifacts.snapshot_path.clone(),
            fixture.root.join("hope.sqlite3"),
        ));
        let state = bootstrap_app_state_from_checkpoint(store, fixture.artifacts.clone())
            .expect("checkpoint bootstrap should hydrate verified app state");

        let readonly_state = validation_feedback_readonly_state_from_verified_app_state(&state);

        assert_eq!(
            readonly_state.source_snapshot_hash,
            SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
        );
        assert_eq!(
            readonly_state.source_snapshot_path,
            state.kb_runtime.summary.snapshot_path
        );
        assert_eq!(
            readonly_state.failure_pattern_count,
            state.kb_knowledge.failure_patterns.len()
        );
        assert_eq!(
            readonly_state.prompt_template_count,
            state.kb_knowledge.prompt_templates.len()
        );
        assert_eq!(
            readonly_state.repair_mapping_ready,
            state.kb_runtime.summary.has_repair_template_mapping
                && !state.kb_knowledge.failure_patterns.is_empty()
                && !state.kb_knowledge.prompt_templates.is_empty()
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn bootstrap_validation_feedback_readonly_state_from_checkpoint_rejects_missing_trusted_input()
    {
        let fixture = create_snapshot_bootstrap_fixture();
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            fixture.artifacts.snapshot_path.clone(),
            fixture.root.join("hope.sqlite3"),
        ));
        let missing_path = fixture.packet_root.join("export_templates.json");
        fs::remove_file(&missing_path).expect("trusted input should be removable for the test");

        let error =
            bootstrap_validation_feedback_readonly_state_from_checkpoint(store, fixture.artifacts)
                .expect_err("validation feedback readiness should not add a local fallback");

        assert_eq!(
            error,
            KbRuntimeError::SnapshotBootstrapInputMissing {
                input_name: "export_template".to_string(),
                path: missing_path,
            }
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn build_storyboard_preview_plan_from_checkpoint_uses_verified_app_state() {
        let fixture = create_snapshot_bootstrap_fixture();
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            fixture.artifacts.snapshot_path.clone(),
            fixture.root.join("hope.sqlite3"),
        ));

        let plan = build_storyboard_preview_plan_from_checkpoint(
            store,
            fixture.artifacts.clone(),
            StoryboardPreviewPlanRequest {
                render_segment_id: "render-segment-checkpoint-001".to_string(),
                narrative_scene_id: "narrative-scene-checkpoint-001".to_string(),
                render_segment_sequence_no: 1,
                start_shot_sequence_no: 1,
                end_shot_sequence_no: 3,
                target_duration_seconds: 45,
                cut_id: "cut-checkpoint-001".to_string(),
                cut_sequence_no: 1,
                shot_description: "medium shot dialogue".to_string(),
                dialogue: "Checkpoint-backed taxonomy should hydrate this preview.".to_string(),
                scene_director_id: None,
                action_director_id: None,
                scene_type: Some("daily_dialogue".to_string()),
                layout_prompt: "cool palette, medium shot".to_string(),
                render_prompt: "restrained realism".to_string(),
            },
        )
        .expect("checkpoint-backed storyboard preview should build");

        assert_eq!(
            plan.render_segment.scene_taxonomy_id.as_deref(),
            Some("scene_tax_01")
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_scene_director_id,
            "taxonomy:scene_tax_01:scene"
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_action_director_id,
            "taxonomy:scene_tax_01:action"
        );
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

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn build_storyboard_preview_plan_from_checkpoint_propagates_bootstrap_errors() {
        let fixture = create_snapshot_bootstrap_fixture();
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            fixture.artifacts.snapshot_path.clone(),
            fixture.root.join("hope.sqlite3"),
        ));
        let missing_path = fixture.packet_root.join("export_templates.json");
        fs::remove_file(&missing_path).expect("trusted input should be removable for the test");

        let error = build_storyboard_preview_plan_from_checkpoint(
            store,
            fixture.artifacts.clone(),
            StoryboardPreviewPlanRequest {
                render_segment_id: "render-segment-checkpoint-002".to_string(),
                narrative_scene_id: "narrative-scene-checkpoint-002".to_string(),
                render_segment_sequence_no: 1,
                start_shot_sequence_no: 1,
                end_shot_sequence_no: 3,
                target_duration_seconds: 45,
                cut_id: "cut-checkpoint-002".to_string(),
                cut_sequence_no: 1,
                shot_description: "medium shot dialogue".to_string(),
                dialogue: "Bootstrap errors should surface directly.".to_string(),
                scene_director_id: None,
                action_director_id: None,
                scene_type: Some("daily_dialogue".to_string()),
                layout_prompt: "cool palette, medium shot".to_string(),
                render_prompt: "restrained realism".to_string(),
            },
        )
        .expect_err("checkpoint caller should not add a local fallback");

        assert_eq!(
            error,
            StoryboardPreviewPlanFromCheckpointError::Bootstrap(
                KbRuntimeError::SnapshotBootstrapInputMissing {
                    input_name: "export_template".to_string(),
                    path: missing_path,
                }
            )
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn resolve_scene_taxonomy_matches_by_scene_type_and_display_name() {
        let state = test_state();

        let by_scene_type = resolve_scene_taxonomy(&state, Some("daily_dialogue"));
        let by_display_name = resolve_scene_taxonomy(&state, Some("Daily Dialogue"));

        assert_eq!(
            by_scene_type
                .as_ref()
                .map(|item| item.scene_taxonomy_id.as_str()),
            Some("scene-taxonomy-daily-dialogue")
        );
        assert_eq!(
            by_display_name
                .as_ref()
                .map(|item| item.scene_type.as_str()),
            Some("daily_dialogue")
        );
    }

    #[test]
    fn build_storyboard_preview_plan_consumes_runtime_scene_taxonomy() {
        let state = test_state();
        let plan = build_storyboard_preview_plan(
            &state,
            StoryboardPreviewPlanRequest {
                render_segment_id: "render-segment-preview-001".to_string(),
                narrative_scene_id: "narrative-scene-preview-001".to_string(),
                render_segment_sequence_no: 1,
                start_shot_sequence_no: 1,
                end_shot_sequence_no: 3,
                target_duration_seconds: 45,
                cut_id: "cut-preview-001".to_string(),
                cut_sequence_no: 1,
                shot_description: "medium shot dialogue".to_string(),
                dialogue: "Let's close this scene before dawn.".to_string(),
                scene_director_id: None,
                action_director_id: None,
                scene_type: Some("daily_dialogue".to_string()),
                layout_prompt: "cool palette, medium shot".to_string(),
                render_prompt: "restrained realism".to_string(),
            },
        )
        .expect("runtime taxonomy should build preview plan");

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
        assert!(
            plan.committee_runtime
                .prompt_layers
                .layout_prompt
                .contains("场景分类：daily_dialogue")
        );
    }

    #[test]
    fn build_storyboard_preview_plan_falls_back_when_scene_taxonomy_is_missing() {
        let state = test_state_without_scene_taxonomy();
        let plan = build_storyboard_preview_plan(
            &state,
            StoryboardPreviewPlanRequest {
                render_segment_id: "render-segment-preview-002".to_string(),
                narrative_scene_id: "narrative-scene-preview-002".to_string(),
                render_segment_sequence_no: 1,
                start_shot_sequence_no: 1,
                end_shot_sequence_no: 3,
                target_duration_seconds: 45,
                cut_id: "cut-preview-002".to_string(),
                cut_sequence_no: 1,
                shot_description: "close shot reaction".to_string(),
                dialogue: "Hold on the lead before the handoff.".to_string(),
                scene_director_id: Some("scene-director-manual".to_string()),
                action_director_id: Some("action-director-manual".to_string()),
                scene_type: Some("daily_dialogue".to_string()),
                layout_prompt: "close shot, interior".to_string(),
                render_prompt: "restrained realism".to_string(),
            },
        )
        .expect("missing taxonomy should not panic when manual directors exist");

        assert_eq!(plan.render_segment.scene_taxonomy_id, None);
        assert_eq!(plan.render_segment.scene_type, None);
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_scene_director_id,
            "scene-director-manual"
        );
        assert_eq!(
            plan.committee_runtime.prompt_layers.layout_prompt,
            "close shot, interior"
        );
    }

    #[test]
    fn build_validation_export_panel_snapshot_surfaces_kb_repairs() {
        let state = test_state();
        let mut fixture = load_week3_shared_fixture(WEEK3_SHARED_FIXTURE_PATH)
            .expect("shared fixture should load");
        fixture.prompt_package[0].body = "TODO: rewrite this prompt body".to_string();

        let snapshot = build_validation_export_panel_snapshot_from_fixture(
            &state,
            ValidationExportPanelSnapshotRequest {
                project_id: "project-week3-001".to_string(),
            },
            &fixture,
        )
        .expect("panel snapshot should build from fixture");

        assert_eq!(snapshot.project_id, "project-week3-001");
        assert_eq!(snapshot.summary_items.len(), 3);
        assert_eq!(
            snapshot.summary_items[2].state,
            ValidationExportPanelState::Ready
        );
        assert!(
            snapshot
                .repair_recommendations
                .iter()
                .any(|item| item.failure_code == "chinese_prompt_noise")
        );
        assert!(
            snapshot
                .repair_recommendations
                .iter()
                .flat_map(|item| item.prompt_template_names.iter())
                .any(|name| name == "Repair Prompt Language")
        );
    }

    #[test]
    fn build_validation_export_panel_snapshot_stays_bounded_without_repair_mappings() {
        let state = test_state_without_repair_mappings();
        let mut fixture = load_week3_shared_fixture(WEEK3_SHARED_FIXTURE_PATH)
            .expect("shared fixture should load");
        fixture.prompt_package[0].body = "TODO: rewrite this prompt body".to_string();

        let snapshot = build_validation_export_panel_snapshot_from_fixture(
            &state,
            ValidationExportPanelSnapshotRequest {
                project_id: "project-week3-001".to_string(),
            },
            &fixture,
        )
        .expect("panel snapshot should still build without repair mappings");

        assert_eq!(snapshot.repair_recommendations.len(), 0);
        assert_eq!(
            snapshot.summary_items[2].state,
            ValidationExportPanelState::Pending
        );
        assert!(snapshot.summary_items[1].value.contains("rows"));
    }

    fn create_snapshot_bootstrap_fixture() -> SnapshotBootstrapFixture {
        let root = unique_test_dir("app-runtime-snapshot-bootstrap");
        let repo_root = root.join("hope-kb-runtime");
        let snapshots_dir = repo_root.join("snapshots");
        let packet_root = repo_root.join("seed").join("v0.1");
        fs::create_dir_all(&snapshots_dir).expect("snapshots dir should be creatable");
        fs::create_dir_all(&packet_root).expect("packet root should be creatable");

        let snapshot_path = snapshots_dir.join("hope-kb-v0.1.sqlite3");
        File::create(&snapshot_path).expect("snapshot file should be creatable");

        let manifest_path = packet_root.join("manifest.json");
        fs::write(
            &manifest_path,
            format!(
                r#"{{"content_hash":"{}"}}"#,
                SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
            ),
        )
        .expect("manifest should be writable");

        let validator_result_path = root.join("validator_result.json");
        fs::write(
            &validator_result_path,
            format!(
                r#"{{"status":"passed","content_hash":"{}"}}"#,
                SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
            ),
        )
        .expect("validator result should be writable");

        let snapshot_meta_path = root.join("snapshot_meta.json");
        fs::write(
            &snapshot_meta_path,
            format!(
                r#"{{"content_hash":"{}"}}"#,
                SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
            ),
        )
        .expect("snapshot meta should be writable");

        fs::write(packet_root.join("export_templates.json"), "[]")
            .expect("export templates should be writable");
        fs::write(
            packet_root.join("failure_pattern_library.json"),
            r#"[{
              "machine_id": "failure_01",
              "failure_code": "style_drift",
              "failure_name": "Style drift",
              "failure_category": "style",
              "symptom": "Adjacent cuts drift apart.",
              "common_causes": ["hard lock missing"],
              "detection_hint": "Check Style Unity Validator",
              "repair_strategy": "Reapply hard locks.",
              "affected_layers": ["prompt_packages"],
              "validator_hint": "Style Unity Validator",
              "repair_template_ids": ["prompt_09"],
              "repair_priority": "high",
              "repair_scope": "render_prompt_only",
              "suggested_followup_validator": ["Style Unity Validator"],
              "source_type": "team_distillation",
              "source_notes": "test",
              "confidence_level": "high",
              "last_reviewed_at": "2026-04-20"
            }]"#,
        )
        .expect("failure patterns should be writable");
        fs::write(packet_root.join("degraded_input_examples.json"), "[]")
            .expect("degraded input examples should be writable");
        fs::write(
            packet_root.join("runtime_consume_contracts.json"),
            r#"[{
              "consumer_surface": "snapshot_bootstrap",
              "required_snapshot_tables": [
                "snapshot_meta",
                "export_template",
                "failure_pattern",
                "degraded_input_example",
                "runtime_consume_contract"
              ]
            }]"#,
        )
        .expect("runtime consume contract should be writable");
        fs::write(
            packet_root.join("scene_taxonomy.json"),
            r#"[{
              "machine_id": "scene_tax_01",
              "scene_type": "daily_dialogue",
              "display_name": "Daily Dialogue",
              "definition": "A stable dialogue scene.",
              "default_duration_band": "30-60s",
              "typical_committee_roles": ["chief", "scene", "emotion"],
              "default_handoff_out": ["scene->emotion"],
              "risk_flags": ["pace_flat"],
              "continuity_priority": "high",
              "prompt_focus": ["micro_expression", "blocking"],
              "source_type": "team_distillation",
              "source_notes": "test",
              "confidence_level": "high",
              "last_reviewed_at": "2026-04-20"
            }]"#,
        )
        .expect("scene taxonomy should be writable");
        fs::write(
            packet_root.join("prompt_templates.json"),
            r#"[{
              "machine_id": "prompt_09",
              "stage": "repair_pass",
              "name": "Structure Repair Loop",
              "target_model_family": "qwen-compatible",
              "repairs_failure_codes": ["style_drift"]
            }]"#,
        )
        .expect("prompt templates should be writable");

        SnapshotBootstrapFixture {
            root,
            packet_root,
            artifacts: SnapshotBootstrapCheckpointArtifacts {
                snapshot_path,
                manifest_path,
                validator_result_path,
                snapshot_meta_path,
            },
        }
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("current time should be after unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("hope-{prefix}-{unique_suffix}"))
    }

    struct SnapshotBootstrapFixture {
        root: PathBuf,
        packet_root: PathBuf,
        artifacts: SnapshotBootstrapCheckpointArtifacts,
    }
}
