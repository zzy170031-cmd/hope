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

pub const QWEN_COMPATIBLE_MODEL_FAMILY: &str = "qwen-compatible";

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
pub struct ModelStoryboardDraftRequest {
    pub request_id: String,
    pub user_requirement: String,
    pub kb_context: ModelKbContextBoundary,
    pub target_model_family: String,
    pub storyboard_output_intent: StoryboardOutputIntent,
    pub validation_expectations: Vec<ModelValidationExpectation>,
    pub seedance_overlay_proposal: SeedanceV1ReadonlyProposalBoundary,
    pub prompt_package_draft: Option<PromptPackageDraftBoundary>,
    pub secret_source: ModelSecretSource,
    pub runtime_secret_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelKbContextBoundary {
    pub context_reference: String,
    pub capability_tags: Vec<String>,
    pub scene_taxonomy_refs: Vec<String>,
    pub failure_pattern_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryboardOutputIntent {
    pub intent_name: String,
    pub draft_scope: String,
    pub expected_sections: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelValidationExpectation {
    pub validator_name: String,
    pub expectation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedanceV1ReadonlyProposalBoundary {
    pub proposal_name: String,
    pub proposal_version: String,
    pub readonly: bool,
    pub export_overlay_only: bool,
    pub v01_workbook_frozen: bool,
    pub exporter_change_allowed: bool,
    pub sheet_change_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptPackageDraftBoundary {
    pub enabled: bool,
    pub payload_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelStoryboardDraftResponse {
    pub model_output_draft: ModelOutputDraft,
    pub structured_storyboard_draft_boundary: StructuredStoryboardDraftBoundary,
    pub seedance_overlay_proposal: SeedanceV1ReadonlyProposalBoundary,
    pub model_metadata: ModelInvocationMetadata,
    pub terminal_state: ModelInvocationTerminalState,
    pub connection_state: ModelConnectionState,
    pub secret_display_hint: ModelSecretDisplayHint,
    pub no_hardcoded_content: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelOutputDraft {
    pub draft_text: Option<String>,
    pub draft_generated: bool,
    pub generated_storyboard_outline: Option<String>,
    pub generated_shot_list: Option<Vec<String>>,
    pub generated_prompt_package: Option<String>,
    pub prompt_package_draft: Option<PromptPackageDraftBoundary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredStoryboardDraftBoundary {
    pub output_intent_name: String,
    pub draft_scope: String,
    pub expected_sections: Vec<String>,
    pub draft_payload_present: bool,
    pub generated_storyboard_outline_present: bool,
    pub generated_shot_list_present: bool,
    pub generated_prompt_package_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInvocationMetadata {
    pub request_model_family: String,
    pub boundary_model_family: String,
    pub invocation_mode: ModelInvocationMode,
    pub secret_source: ModelSecretSource,
    pub attempted_live_call: bool,
    pub contract_shell_only: bool,
    pub live_invocation_disabled: bool,
    pub configuration_reference_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInvocationBoundary {
    pub mode: ModelInvocationMode,
    pub target_model_family: String,
    pub configuration_present: bool,
    pub environment_reference: ModelEnvironmentReferenceBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelEnvironmentReferenceBoundary {
    pub api_key_env_var: Option<String>,
    pub endpoint_env_var: Option<String>,
    pub model_name_env_var: Option<String>,
    pub local_secret_name: Option<String>,
    pub runtime_config_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelSecretSource {
    DesktopRuntimeInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelInvocationMode {
    Mock,
    Env,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelInvocationTerminalState {
    Completed,
    Mocked,
    ContractOnly,
    LiveDisabled,
    MissingRuntimeSecret,
    Connecting,
    Connected,
    Failed { message: String },
    Refused { reason: String },
    TimedOut { timeout_ms: u64 },
    Misconfigured { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelConnectionState {
    Completed,
    Mocked,
    ContractOnly,
    LiveDisabled,
    MissingRuntimeSecret,
    Connecting,
    Connected,
    Failed { message: String },
    Refused { reason: String },
    TimedOut { timeout_ms: u64 },
    Misconfigured { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelSecretDisplayHint {
    LastFive { value: String },
    Redacted { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelStoryboardDraftRequestError {
    MissingRequestId,
    MissingUserRequirement,
    MissingKbContextReference,
    MissingKbCapabilityTags,
    MissingTargetModelFamily,
    UnsupportedTargetModelFamily { target_model_family: String },
    MissingStoryboardOutputIntent,
    MissingValidationExpectations,
    MissingValidationExpectationDetail,
    InvalidSeedanceOverlayBoundary,
    PromptPackageDraftMustRemainDisabled,
    MissingRuntimeSecret,
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

pub fn validate_model_storyboard_draft_request(
    request: &ModelStoryboardDraftRequest,
) -> Result<(), ModelStoryboardDraftRequestError> {
    if request.request_id.trim().is_empty() {
        return Err(ModelStoryboardDraftRequestError::MissingRequestId);
    }
    if request.user_requirement.trim().is_empty() {
        return Err(ModelStoryboardDraftRequestError::MissingUserRequirement);
    }
    if request.kb_context.context_reference.trim().is_empty() {
        return Err(ModelStoryboardDraftRequestError::MissingKbContextReference);
    }
    if request.kb_context.capability_tags.is_empty()
        || request
            .kb_context
            .capability_tags
            .iter()
            .any(|tag| tag.trim().is_empty())
    {
        return Err(ModelStoryboardDraftRequestError::MissingKbCapabilityTags);
    }
    if request.target_model_family.trim().is_empty() {
        return Err(ModelStoryboardDraftRequestError::MissingTargetModelFamily);
    }
    if request.target_model_family != QWEN_COMPATIBLE_MODEL_FAMILY {
        return Err(
            ModelStoryboardDraftRequestError::UnsupportedTargetModelFamily {
                target_model_family: request.target_model_family.clone(),
            },
        );
    }
    if request
        .storyboard_output_intent
        .intent_name
        .trim()
        .is_empty()
        || request
            .storyboard_output_intent
            .draft_scope
            .trim()
            .is_empty()
        || request
            .storyboard_output_intent
            .expected_sections
            .is_empty()
    {
        return Err(ModelStoryboardDraftRequestError::MissingStoryboardOutputIntent);
    }
    if request.validation_expectations.is_empty() {
        return Err(ModelStoryboardDraftRequestError::MissingValidationExpectations);
    }
    if request.validation_expectations.iter().any(|expectation| {
        expectation.validator_name.trim().is_empty() || expectation.expectation.trim().is_empty()
    }) {
        return Err(ModelStoryboardDraftRequestError::MissingValidationExpectationDetail);
    }
    if !request.seedance_overlay_proposal.readonly
        || !request.seedance_overlay_proposal.export_overlay_only
        || !request.seedance_overlay_proposal.v01_workbook_frozen
        || request.seedance_overlay_proposal.exporter_change_allowed
        || request.seedance_overlay_proposal.sheet_change_allowed
    {
        return Err(ModelStoryboardDraftRequestError::InvalidSeedanceOverlayBoundary);
    }
    if request
        .prompt_package_draft
        .as_ref()
        .is_some_and(|draft| draft.enabled || draft.payload_present)
    {
        return Err(ModelStoryboardDraftRequestError::PromptPackageDraftMustRemainDisabled);
    }
    if !request.runtime_secret_present {
        return Err(ModelStoryboardDraftRequestError::MissingRuntimeSecret);
    }

    Ok(())
}

pub fn invoke_model_storyboard_draft(
    request: ModelStoryboardDraftRequest,
    boundary: ModelInvocationBoundary,
    runtime_secret: Option<&str>,
) -> ModelStoryboardDraftResponse {
    if let Err(error) = validate_model_storyboard_draft_request(&request) {
        return empty_model_storyboard_draft_response(
            &request,
            &boundary,
            runtime_secret,
            terminal_and_connection_state_for_request_error(error),
        );
    }
    if runtime_secret.is_none() {
        return empty_model_storyboard_draft_response(
            &request,
            &boundary,
            runtime_secret,
            ModelResponseState {
                terminal_state: ModelInvocationTerminalState::ContractOnly,
                connection_state: ModelConnectionState::MissingRuntimeSecret,
            },
        );
    }
    if boundary.target_model_family != request.target_model_family {
        return empty_model_storyboard_draft_response(
            &request,
            &boundary,
            runtime_secret,
            ModelResponseState {
                terminal_state: ModelInvocationTerminalState::ContractOnly,
                connection_state: ModelConnectionState::Misconfigured {
                    message: "model boundary family does not match request family".to_string(),
                },
            },
        );
    }
    if !model_environment_reference_boundary_is_valid(&boundary.environment_reference) {
        return empty_model_storyboard_draft_response(
            &request,
            &boundary,
            runtime_secret,
            ModelResponseState {
                terminal_state: ModelInvocationTerminalState::ContractOnly,
                connection_state: ModelConnectionState::Misconfigured {
                    message: "model environment references must be names, not values".to_string(),
                },
            },
        );
    }

    match boundary.mode {
        ModelInvocationMode::Mock => empty_model_storyboard_draft_response(
            &request,
            &boundary,
            runtime_secret,
            ModelResponseState {
                terminal_state: ModelInvocationTerminalState::Mocked,
                connection_state: ModelConnectionState::Mocked,
            },
        ),
        ModelInvocationMode::Env if !boundary.configuration_present => {
            empty_model_storyboard_draft_response(
                &request,
                &boundary,
                runtime_secret,
                ModelResponseState {
                    terminal_state: ModelInvocationTerminalState::ContractOnly,
                    connection_state: ModelConnectionState::Misconfigured {
                        message: "model environment is not configured".to_string(),
                    },
                },
            )
        }
        ModelInvocationMode::Env => empty_model_storyboard_draft_response(
            &request,
            &boundary,
            runtime_secret,
            ModelResponseState {
                terminal_state: ModelInvocationTerminalState::LiveDisabled,
                connection_state: ModelConnectionState::LiveDisabled,
            },
        ),
    }
}

fn model_environment_reference_boundary_is_valid(
    boundary: &ModelEnvironmentReferenceBoundary,
) -> bool {
    [
        boundary.api_key_env_var.as_deref(),
        boundary.endpoint_env_var.as_deref(),
        boundary.model_name_env_var.as_deref(),
        boundary.local_secret_name.as_deref(),
        boundary.runtime_config_name.as_deref(),
    ]
    .into_iter()
    .flatten()
    .all(|reference| {
        let reference = reference.trim();
        !reference.is_empty()
            && !reference.contains("://")
            && !reference.contains('=')
            && !reference.contains('\n')
            && !reference.contains('\r')
    })
}

fn terminal_and_connection_state_for_request_error(
    error: ModelStoryboardDraftRequestError,
) -> ModelResponseState {
    match error {
        ModelStoryboardDraftRequestError::MissingRuntimeSecret => ModelResponseState {
            terminal_state: ModelInvocationTerminalState::ContractOnly,
            connection_state: ModelConnectionState::MissingRuntimeSecret,
        },
        other_error => ModelResponseState {
            terminal_state: ModelInvocationTerminalState::ContractOnly,
            connection_state: ModelConnectionState::Misconfigured {
                message: format!("invalid model request boundary: {:?}", other_error),
            },
        },
    }
}

fn empty_model_storyboard_draft_response(
    request: &ModelStoryboardDraftRequest,
    boundary: &ModelInvocationBoundary,
    runtime_secret: Option<&str>,
    response_state: ModelResponseState,
) -> ModelStoryboardDraftResponse {
    let live_invocation_disabled = matches!(
        response_state.terminal_state,
        ModelInvocationTerminalState::LiveDisabled
    );

    ModelStoryboardDraftResponse {
        model_output_draft: ModelOutputDraft {
            draft_text: None,
            draft_generated: false,
            generated_storyboard_outline: None,
            generated_shot_list: None,
            generated_prompt_package: None,
            prompt_package_draft: None,
        },
        structured_storyboard_draft_boundary: StructuredStoryboardDraftBoundary {
            output_intent_name: request.storyboard_output_intent.intent_name.clone(),
            draft_scope: request.storyboard_output_intent.draft_scope.clone(),
            expected_sections: request.storyboard_output_intent.expected_sections.clone(),
            draft_payload_present: false,
            generated_storyboard_outline_present: false,
            generated_shot_list_present: false,
            generated_prompt_package_present: false,
        },
        seedance_overlay_proposal: request.seedance_overlay_proposal.clone(),
        model_metadata: ModelInvocationMetadata {
            request_model_family: request.target_model_family.clone(),
            boundary_model_family: boundary.target_model_family.clone(),
            invocation_mode: boundary.mode,
            secret_source: request.secret_source,
            attempted_live_call: false,
            contract_shell_only: true,
            live_invocation_disabled,
            configuration_reference_present: boundary.configuration_present
                && model_environment_reference_boundary_is_valid(&boundary.environment_reference),
        },
        terminal_state: response_state.terminal_state,
        connection_state: response_state.connection_state,
        secret_display_hint: runtime_secret
            .map(secret_display_hint_from_runtime_input)
            .unwrap_or_else(|| ModelSecretDisplayHint::Redacted {
                reason: "runtime secret is not available".to_string(),
            }),
        no_hardcoded_content: true,
    }
}

fn secret_display_hint_from_runtime_input(runtime_secret: &str) -> ModelSecretDisplayHint {
    let secret_chars = runtime_secret.chars().collect::<Vec<_>>();
    if secret_chars.len() <= 5 {
        return ModelSecretDisplayHint::Redacted {
            reason: "runtime secret is too short for a safe display hint".to_string(),
        };
    }

    ModelSecretDisplayHint::LastFive {
        value: secret_chars[secret_chars.len() - 5..].iter().collect(),
    }
}

struct ModelResponseState {
    terminal_state: ModelInvocationTerminalState,
    connection_state: ModelConnectionState,
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
        ModelConnectionState, ModelEnvironmentReferenceBoundary, ModelInvocationBoundary,
        ModelInvocationMode, ModelInvocationTerminalState, ModelKbContextBoundary,
        ModelSecretDisplayHint, ModelSecretSource, ModelStoryboardDraftRequest,
        ModelStoryboardDraftRequestError, ModelValidationExpectation, PromptPackageDraftBoundary,
        QWEN_COMPATIBLE_MODEL_FAMILY, SeedanceV1ReadonlyProposalBoundary, StoryboardOutputIntent,
        StoryboardPreviewPlanFromCheckpointError, StoryboardPreviewPlanRequest,
        ValidationExportPanelSnapshotRequest, ValidationExportPanelState,
        bootstrap_app_state_from_checkpoint,
        bootstrap_snapshot_bootstrap_readonly_state_from_checkpoint,
        bootstrap_validation_feedback_readonly_state_from_checkpoint,
        build_storyboard_preview_plan, build_storyboard_preview_plan_from_checkpoint,
        build_validation_export_panel_snapshot_from_fixture, invoke_model_storyboard_draft,
        resolve_scene_taxonomy, snapshot_bootstrap_readonly_state_from_verified_app_state,
        validate_model_storyboard_draft_request,
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

    fn valid_model_storyboard_draft_request() -> ModelStoryboardDraftRequest {
        ModelStoryboardDraftRequest {
            request_id: "qwen-contract-request-001".to_string(),
            user_requirement: "Need a bounded storyboard draft from user requirements.".to_string(),
            kb_context: ModelKbContextBoundary {
                context_reference: "kb-capability:snapshot-bootstrap".to_string(),
                capability_tags: vec![
                    "scene-taxonomy-ready".to_string(),
                    "repair-mapping-ready".to_string(),
                ],
                scene_taxonomy_refs: vec!["scene-taxonomy-ref".to_string()],
                failure_pattern_refs: vec!["failure-pattern-ref".to_string()],
            },
            target_model_family: QWEN_COMPATIBLE_MODEL_FAMILY.to_string(),
            storyboard_output_intent: StoryboardOutputIntent {
                intent_name: "storyboard_draft".to_string(),
                draft_scope: "bounded_model_contract".to_string(),
                expected_sections: vec![
                    "requirement_summary".to_string(),
                    "storyboard_outline_boundary".to_string(),
                ],
            },
            validation_expectations: vec![ModelValidationExpectation {
                validator_name: "readonly_contract_check".to_string(),
                expectation: "Response must stay bounded and avoid final script content."
                    .to_string(),
            }],
            seedance_overlay_proposal: seedance_v1_readonly_proposal_boundary(),
            prompt_package_draft: None,
            secret_source: ModelSecretSource::DesktopRuntimeInput,
            runtime_secret_present: true,
        }
    }

    fn seedance_v1_readonly_proposal_boundary() -> SeedanceV1ReadonlyProposalBoundary {
        SeedanceV1ReadonlyProposalBoundary {
            proposal_name: "seedance-v1-export-overlay".to_string(),
            proposal_version: "v0.2-proposal".to_string(),
            readonly: true,
            export_overlay_only: true,
            v01_workbook_frozen: true,
            exporter_change_allowed: false,
            sheet_change_allowed: false,
        }
    }

    fn env_reference_boundary() -> ModelEnvironmentReferenceBoundary {
        ModelEnvironmentReferenceBoundary {
            api_key_env_var: Some("QWEN_API_KEY".to_string()),
            endpoint_env_var: Some("QWEN_ENDPOINT".to_string()),
            model_name_env_var: Some("QWEN_MODEL_NAME".to_string()),
            local_secret_name: Some("desktop-qwen-runtime-secret".to_string()),
            runtime_config_name: Some("desktop-qwen-runtime-config".to_string()),
        }
    }

    fn mock_model_boundary() -> ModelInvocationBoundary {
        ModelInvocationBoundary {
            mode: ModelInvocationMode::Mock,
            target_model_family: QWEN_COMPATIBLE_MODEL_FAMILY.to_string(),
            configuration_present: false,
            environment_reference: ModelEnvironmentReferenceBoundary {
                api_key_env_var: None,
                endpoint_env_var: None,
                model_name_env_var: None,
                local_secret_name: None,
                runtime_config_name: None,
            },
        }
    }

    fn desktop_runtime_secret_for_test() -> String {
        ["desktop", "runtime", "value", "12345"].join("_")
    }

    #[test]
    fn model_storyboard_draft_request_requires_bounded_fields() {
        let request = valid_model_storyboard_draft_request();
        assert_eq!(validate_model_storyboard_draft_request(&request), Ok(()));
        assert_eq!(
            request.secret_source,
            ModelSecretSource::DesktopRuntimeInput
        );
        assert!(request.runtime_secret_present);

        let mut missing_request_id = request.clone();
        missing_request_id.request_id = " ".to_string();
        assert_eq!(
            validate_model_storyboard_draft_request(&missing_request_id),
            Err(ModelStoryboardDraftRequestError::MissingRequestId)
        );

        let mut missing_user_requirement = request.clone();
        missing_user_requirement.user_requirement = " ".to_string();
        assert_eq!(
            validate_model_storyboard_draft_request(&missing_user_requirement),
            Err(ModelStoryboardDraftRequestError::MissingUserRequirement)
        );

        let mut missing_context_reference = request.clone();
        missing_context_reference
            .kb_context
            .context_reference
            .clear();
        assert_eq!(
            validate_model_storyboard_draft_request(&missing_context_reference),
            Err(ModelStoryboardDraftRequestError::MissingKbContextReference)
        );

        let mut missing_capability_tags = request.clone();
        missing_capability_tags.kb_context.capability_tags.clear();
        assert_eq!(
            validate_model_storyboard_draft_request(&missing_capability_tags),
            Err(ModelStoryboardDraftRequestError::MissingKbCapabilityTags)
        );

        let mut unsupported_family = request.clone();
        unsupported_family.target_model_family = "other-model-family".to_string();
        assert_eq!(
            validate_model_storyboard_draft_request(&unsupported_family),
            Err(
                ModelStoryboardDraftRequestError::UnsupportedTargetModelFamily {
                    target_model_family: "other-model-family".to_string(),
                }
            )
        );

        let mut missing_output_intent = request.clone();
        missing_output_intent
            .storyboard_output_intent
            .expected_sections
            .clear();
        assert_eq!(
            validate_model_storyboard_draft_request(&missing_output_intent),
            Err(ModelStoryboardDraftRequestError::MissingStoryboardOutputIntent)
        );

        let mut missing_validation = request;
        missing_validation.validation_expectations.clear();
        assert_eq!(
            validate_model_storyboard_draft_request(&missing_validation),
            Err(ModelStoryboardDraftRequestError::MissingValidationExpectations)
        );

        let mut invalid_seedance_boundary = valid_model_storyboard_draft_request();
        invalid_seedance_boundary
            .seedance_overlay_proposal
            .exporter_change_allowed = true;
        assert_eq!(
            validate_model_storyboard_draft_request(&invalid_seedance_boundary),
            Err(ModelStoryboardDraftRequestError::InvalidSeedanceOverlayBoundary)
        );

        let mut enabled_prompt_package_draft = valid_model_storyboard_draft_request();
        enabled_prompt_package_draft.prompt_package_draft = Some(PromptPackageDraftBoundary {
            enabled: true,
            payload_present: false,
        });
        assert_eq!(
            validate_model_storyboard_draft_request(&enabled_prompt_package_draft),
            Err(ModelStoryboardDraftRequestError::PromptPackageDraftMustRemainDisabled)
        );

        let mut missing_runtime_secret = valid_model_storyboard_draft_request();
        missing_runtime_secret.runtime_secret_present = false;
        assert_eq!(
            validate_model_storyboard_draft_request(&missing_runtime_secret),
            Err(ModelStoryboardDraftRequestError::MissingRuntimeSecret)
        );
    }

    #[test]
    fn mock_model_storyboard_draft_response_contains_no_hardcoded_content() {
        let request = valid_model_storyboard_draft_request();
        let runtime_secret = desktop_runtime_secret_for_test();

        let response =
            invoke_model_storyboard_draft(request, mock_model_boundary(), Some(&runtime_secret));

        assert_eq!(
            response.terminal_state,
            ModelInvocationTerminalState::Mocked
        );
        assert_eq!(response.connection_state, ModelConnectionState::Mocked);
        assert!(response.no_hardcoded_content);
        assert_eq!(response.model_output_draft.draft_text, None);
        assert!(!response.model_output_draft.draft_generated);
        assert_eq!(
            response.model_output_draft.generated_storyboard_outline,
            None
        );
        assert_eq!(response.model_output_draft.generated_shot_list, None);
        assert_eq!(response.model_output_draft.generated_prompt_package, None);
        assert_eq!(response.model_output_draft.prompt_package_draft, None);
        assert!(
            !response
                .structured_storyboard_draft_boundary
                .draft_payload_present
        );
        assert!(
            !response
                .structured_storyboard_draft_boundary
                .generated_storyboard_outline_present
        );
        assert!(
            !response
                .structured_storyboard_draft_boundary
                .generated_shot_list_present
        );
        assert!(
            !response
                .structured_storyboard_draft_boundary
                .generated_prompt_package_present
        );
        assert!(!response.model_metadata.attempted_live_call);
        assert!(response.model_metadata.contract_shell_only);
        assert!(!response.model_metadata.live_invocation_disabled);
        assert!(!response.model_metadata.configuration_reference_present);
        assert_eq!(
            response.model_metadata.request_model_family,
            QWEN_COMPATIBLE_MODEL_FAMILY
        );
        assert_eq!(
            response.model_metadata.secret_source,
            ModelSecretSource::DesktopRuntimeInput
        );
        assert_eq!(
            response.secret_display_hint,
            ModelSecretDisplayHint::LastFive {
                value: "12345".to_string(),
            }
        );
        assert_ne!(
            response.secret_display_hint,
            ModelSecretDisplayHint::LastFive {
                value: runtime_secret,
            }
        );
        assert_eq!(
            response.seedance_overlay_proposal,
            seedance_v1_readonly_proposal_boundary()
        );
        assert!(response.seedance_overlay_proposal.readonly);
        assert!(response.seedance_overlay_proposal.export_overlay_only);
        assert!(response.seedance_overlay_proposal.v01_workbook_frozen);
        assert!(!response.seedance_overlay_proposal.exporter_change_allowed);
        assert!(!response.seedance_overlay_proposal.sheet_change_allowed);
    }

    #[test]
    fn model_storyboard_draft_boundary_requires_desktop_runtime_secret() {
        let request = valid_model_storyboard_draft_request();
        let boundary = ModelInvocationBoundary {
            mode: ModelInvocationMode::Env,
            target_model_family: QWEN_COMPATIBLE_MODEL_FAMILY.to_string(),
            configuration_present: false,
            environment_reference: env_reference_boundary(),
        };

        let response = invoke_model_storyboard_draft(request, boundary, None);

        assert_eq!(
            response.terminal_state,
            ModelInvocationTerminalState::ContractOnly
        );
        assert_eq!(
            response.connection_state,
            ModelConnectionState::MissingRuntimeSecret
        );
        assert!(response.no_hardcoded_content);
        assert_eq!(response.model_output_draft.draft_text, None);
        assert!(!response.model_output_draft.draft_generated);
        assert!(!response.model_metadata.attempted_live_call);
        assert!(matches!(
            response.secret_display_hint,
            ModelSecretDisplayHint::Redacted { .. }
        ));
    }

    #[test]
    fn env_model_storyboard_draft_boundary_returns_misconfigured_when_not_configured() {
        let request = valid_model_storyboard_draft_request();
        let runtime_secret = desktop_runtime_secret_for_test();
        let boundary = ModelInvocationBoundary {
            mode: ModelInvocationMode::Env,
            target_model_family: QWEN_COMPATIBLE_MODEL_FAMILY.to_string(),
            configuration_present: false,
            environment_reference: env_reference_boundary(),
        };

        let response = invoke_model_storyboard_draft(request, boundary, Some(&runtime_secret));

        assert_eq!(
            response.terminal_state,
            ModelInvocationTerminalState::ContractOnly
        );
        assert_eq!(
            response.connection_state,
            ModelConnectionState::Misconfigured {
                message: "model environment is not configured".to_string(),
            }
        );
        assert!(response.no_hardcoded_content);
        assert_eq!(response.model_output_draft.draft_text, None);
        assert!(!response.model_output_draft.draft_generated);
        assert!(!response.model_metadata.attempted_live_call);
        assert!(response.model_metadata.contract_shell_only);
        assert!(!response.model_metadata.live_invocation_disabled);
        assert!(!response.model_metadata.configuration_reference_present);
        assert_eq!(
            response.secret_display_hint,
            ModelSecretDisplayHint::LastFive {
                value: "12345".to_string(),
            }
        );
    }

    #[test]
    fn env_model_storyboard_draft_boundary_is_live_disabled_even_when_configured() {
        let request = valid_model_storyboard_draft_request();
        let runtime_secret = desktop_runtime_secret_for_test();
        let boundary = ModelInvocationBoundary {
            mode: ModelInvocationMode::Env,
            target_model_family: QWEN_COMPATIBLE_MODEL_FAMILY.to_string(),
            configuration_present: true,
            environment_reference: env_reference_boundary(),
        };

        let response = invoke_model_storyboard_draft(request, boundary, Some(&runtime_secret));

        assert_eq!(
            response.terminal_state,
            ModelInvocationTerminalState::LiveDisabled
        );
        assert_eq!(
            response.connection_state,
            ModelConnectionState::LiveDisabled
        );
        assert!(response.model_metadata.contract_shell_only);
        assert!(response.model_metadata.live_invocation_disabled);
        assert!(response.model_metadata.configuration_reference_present);
        assert!(!response.model_metadata.attempted_live_call);
        assert_eq!(response.model_output_draft.draft_text, None);
        assert!(!response.model_output_draft.draft_generated);
        assert_eq!(
            response.model_output_draft.generated_storyboard_outline,
            None
        );
        assert_eq!(response.model_output_draft.generated_shot_list, None);
        assert_eq!(response.model_output_draft.generated_prompt_package, None);
        assert_eq!(response.model_output_draft.prompt_package_draft, None);
        assert!(response.no_hardcoded_content);
    }

    #[test]
    fn env_model_storyboard_draft_boundary_rejects_secret_values_as_references() {
        let request = valid_model_storyboard_draft_request();
        let runtime_secret = desktop_runtime_secret_for_test();
        let boundary = ModelInvocationBoundary {
            mode: ModelInvocationMode::Env,
            target_model_family: QWEN_COMPATIBLE_MODEL_FAMILY.to_string(),
            configuration_present: true,
            environment_reference: ModelEnvironmentReferenceBoundary {
                api_key_env_var: Some("QWEN_API_KEY=not-a-name".to_string()),
                endpoint_env_var: Some("QWEN_ENDPOINT=not-a-name".to_string()),
                model_name_env_var: Some("QWEN_MODEL_NAME".to_string()),
                local_secret_name: Some("desktop-qwen-runtime-secret".to_string()),
                runtime_config_name: Some("desktop-qwen-runtime-config".to_string()),
            },
        };

        let response = invoke_model_storyboard_draft(request, boundary, Some(&runtime_secret));

        assert_eq!(
            response.terminal_state,
            ModelInvocationTerminalState::ContractOnly
        );
        assert_eq!(
            response.connection_state,
            ModelConnectionState::Misconfigured {
                message: "model environment references must be names, not values".to_string(),
            }
        );
        assert!(!response.model_metadata.configuration_reference_present);
        assert!(!response.model_metadata.attempted_live_call);
        assert_eq!(response.model_output_draft.draft_text, None);
        assert!(!response.model_output_draft.draft_generated);
        assert!(response.no_hardcoded_content);
    }

    #[test]
    fn short_desktop_runtime_secret_never_exposes_full_value() {
        let request = valid_model_storyboard_draft_request();
        let short_runtime_secret = "tiny";

        let response = invoke_model_storyboard_draft(
            request,
            mock_model_boundary(),
            Some(short_runtime_secret),
        );

        assert_eq!(
            response.secret_display_hint,
            ModelSecretDisplayHint::Redacted {
                reason: "runtime secret is too short for a safe display hint".to_string(),
            }
        );
        assert_ne!(
            response.secret_display_hint,
            ModelSecretDisplayHint::LastFive {
                value: short_runtime_secret.to_string(),
            }
        );
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
