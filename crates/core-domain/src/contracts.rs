use crate::domain::HierarchyRef;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelChannel {
    QwenCompatible,
    GptCompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptPackageRecord {
    pub prompt_package_id: String,
    pub source: HierarchyRef,
    pub body: String,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelRequest {
    pub request_id: String,
    pub channel: ModelChannel,
    pub prompt_body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelResponse {
    pub request_id: String,
    pub body: String,
    pub raw_response: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextModelProviderKind {
    Qwen,
    Doubao,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextModelProvider {
    pub provider: TextModelProviderKind,
    pub model: String,
    pub base_url: Option<String>,
    pub api_key_ref: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextGenerationTask {
    ExpandScript,
    GenerateStoryboard,
    RepairStoryboard,
    CompileSeedancePromptText,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextGenerationOutputSchema {
    PlainText,
    StoryboardRowsJson,
    RepairPlanJson,
    SeedancePromptText,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextGenerationRequest {
    pub task_type: TextGenerationTask,
    pub scene_type: Option<String>,
    pub story_input: String,
    pub duration_plan: Option<StoryboardDurationPlan>,
    pub kb_context_summary: String,
    pub selected_sample_ids: Vec<String>,
    pub selected_kb_rules: Vec<String>,
    pub output_schema: TextGenerationOutputSchema,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextGenerationResponse {
    pub text: String,
    pub structured_json: Option<Value>,
    pub usage_tokens: Option<u32>,
    pub latency_ms: Option<u64>,
    pub warnings: Vec<ProductWarning>,
    pub provider: TextModelProviderKind,
    pub model: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PromptTextCompilationStatus {
    ReadyStub,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationSummary {
    pub subject_id: String,
    pub passed: bool,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportSummary {
    pub export_id: String,
    pub workbook_machine_name: String,
    pub workbook_chinese_name: String,
    pub sheet_machine_names: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StructureMode {
    SingleShot,
    SequenceShot,
}

impl StructureMode {
    pub fn from_sample_type(value: &str) -> Self {
        match value {
            "sequence_shot" => Self::SequenceShot,
            _ => Self::SingleShot,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SingleShot => "single_shot",
            Self::SequenceShot => "sequence_shot",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SequenceFieldState {
    NotApplicable,
    Present,
    Missing,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BridgeCallStatus {
    Ready,
    WarningOnly,
    Blocked,
    Gated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductWarning {
    pub code: String,
    pub message: String,
    pub related_sample_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceStoryFacts {
    #[serde(default)]
    pub character_names: Vec<String>,
    #[serde(default)]
    pub character_relationships: Vec<String>,
    #[serde(default)]
    pub core_events: Vec<String>,
    #[serde(default)]
    pub event_order: Vec<String>,
    #[serde(default)]
    pub timeline_facts: Vec<String>,
    #[serde(default)]
    pub prop_state: Vec<String>,
    #[serde(default)]
    pub location_facts: Vec<String>,
    #[serde(default)]
    pub emotional_progression: Vec<String>,
    #[serde(default)]
    pub conflict_progression: Vec<String>,
    #[serde(default)]
    pub ending_state: String,
}

fn default_target_duration_mode() -> String {
    "fixed_seconds".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SequenceGrouping {
    pub structure_mode: StructureMode,
    pub sequence_id: Option<String>,
    pub shot_order: Option<u32>,
    pub sequence_field_state: SequenceFieldState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleSelectorInput {
    pub scene_type: String,
    pub desired_duration_seconds: u16,
    pub require_prompt_candidate: bool,
    pub allow_reserve_samples: bool,
    pub preferred_quality_grade: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleGateDecision {
    pub sample_id: String,
    pub decision: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleSelectionResult {
    pub selected_rows: Vec<String>,
    pub excluded_rows: Vec<GoldenSampleGateDecision>,
    pub gate_decisions: Vec<GoldenSampleGateDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalReferenceHandleCandidate {
    pub source_sample_id: String,
    pub reference_name: String,
    pub reference_kind: String,
    pub strength: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenePerformanceProjection {
    pub source_sample_id: String,
    pub source_sample_title: String,
    pub scene_scale: String,
    pub person: String,
    pub visual_description: String,
    pub character_action: String,
    pub fused_source_text: String,
    pub sequence_grouping: SequenceGrouping,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShotGroundingSource {
    ShotScript,
    ExpandedScriptText,
    PrimarySceneFields,
    KbRouterSummary,
}

impl ShotGroundingSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ShotScript => "shot_script",
            Self::ExpandedScriptText => "expanded_script_text",
            Self::PrimarySceneFields => "primary_scene_fields",
            Self::KbRouterSummary => "kb_router_summary",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpandScriptRequest {
    pub scene_type: String,
    #[serde(default)]
    pub scene_label: Option<String>,
    #[serde(default)]
    pub scene_category: Option<String>,
    #[serde(default)]
    pub model_config_summary: Option<ModelConfigSummary>,
    #[serde(default)]
    pub target_duration_seconds: Option<u16>,
    #[serde(default)]
    pub selected_total_duration_seconds: Option<u16>,
    #[serde(default = "default_target_duration_mode")]
    pub target_duration_mode: String,
    #[serde(default)]
    pub story_length_profile: String,
    #[serde(default)]
    pub source_material_length_chars: u32,
    #[serde(default)]
    pub auto_segment_strategy: String,
    #[serde(default)]
    pub source_input_type: String,
    #[serde(default)]
    pub authoring_mode: String,
    #[serde(default)]
    pub source_material_summary: String,
    #[serde(default)]
    pub source_story_facts: SourceStoryFacts,
    #[serde(default)]
    pub preserved_fact_summary: String,
    #[serde(default)]
    pub changed_for_screenplay_summary: String,
    #[serde(default)]
    pub omitted_detail_summary: String,
    pub synopsis_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelConfigSummary {
    pub provider: String,
    pub model: String,
    pub enabled: bool,
    #[serde(default)]
    pub base_url_present: bool,
    pub api_key_present: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KbRouterTaskType {
    ExpandScript,
    GenerateStoryboard,
    RepairStoryboard,
    CompileSeedancePromptText,
}

impl KbRouterTaskType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExpandScript => "expand_script",
            Self::GenerateStoryboard => "generate_storyboard",
            Self::RepairStoryboard => "repair_storyboard",
            Self::CompileSeedancePromptText => "compile_seedance_prompt_text",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KbRouterRuntimeRequest {
    pub scene_type: String,
    pub synopsis_text: String,
    pub duration_seconds: u16,
    pub task_type: KbRouterTaskType,
    #[serde(default)]
    pub primary_scene_type: Option<String>,
    #[serde(default)]
    pub primary_scene_label: Option<String>,
    #[serde(default)]
    pub shot_scene_type: Option<String>,
    #[serde(default)]
    pub shot_scene_label: Option<String>,
    pub shot_intent: Option<String>,
    pub structure_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KbRouterSelectedRule {
    pub rule_id: String,
    pub family: String,
    pub summary: String,
    pub applies_to: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KbRouterSelectionReason {
    pub sample_id: String,
    pub reason_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KbRouterExcludedCandidate {
    pub sample_id: String,
    pub reason_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KbRouterTokenBudget {
    pub kb_context_summary_target: String,
    pub full_kb_rows_included: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KbRouterRetrievalTrace {
    pub kb_version: String,
    pub snapshot_id: String,
    pub snapshot_checksum: String,
    pub content_cache_key: String,
    pub task_type: String,
    pub top_k_samples: u8,
    pub top_k_rules: u8,
    pub duration_seconds: u16,
    pub story_keywords: Vec<String>,
    pub selection_reasons: Vec<KbRouterSelectionReason>,
    pub excluded_candidates: Vec<KbRouterExcludedCandidate>,
    pub token_budget: KbRouterTokenBudget,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KbRouterRuntimeResponse {
    pub selected_sample_ids: Vec<String>,
    pub selected_kb_rules: Vec<KbRouterSelectedRule>,
    pub kb_context_summary: String,
    pub retrieval_trace: KbRouterRetrievalTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpandScriptResponse {
    #[serde(default = "default_bridge_call_status")]
    pub status: BridgeCallStatus,
    pub script_id: String,
    pub expanded_script_text: String,
    pub script_hash: String,
    #[serde(default)]
    pub blockers: Vec<ProductWarning>,
    pub warnings: Vec<ProductWarning>,
    #[serde(default)]
    pub source_input_type: String,
    #[serde(default)]
    pub authoring_mode: String,
    #[serde(default)]
    pub source_material_summary: String,
    #[serde(default)]
    pub source_story_facts: SourceStoryFacts,
    #[serde(default)]
    pub preserved_fact_summary: String,
    #[serde(default)]
    pub changed_for_screenplay_summary: String,
    #[serde(default)]
    pub omitted_detail_summary: String,
    #[serde(default)]
    pub continuity_warnings: Vec<ProductWarning>,
    #[serde(default = "default_target_duration_mode")]
    pub target_duration_mode: String,
    #[serde(default)]
    pub story_length_profile: String,
    #[serde(default)]
    pub source_material_length_chars: u32,
    #[serde(default)]
    pub auto_segment_strategy: String,
    #[serde(default)]
    pub estimated_total_story_duration_seconds: u16,
    #[serde(default)]
    pub generated_shot_task_count: u32,
    #[serde(default)]
    pub duration_plan_summary: String,
    pub kb_router_result: KbRouterRuntimeResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InferredSceneFactBinding {
    pub fact: String,
    #[serde(default)]
    pub inference_reason: String,
    #[serde(default)]
    pub inference_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcceptedRewriteSnapshotBinding {
    pub current_case_id: String,
    pub source_text_hash: String,
    pub accepted_rewrite_hash: String,
    pub scene_type: String,
    pub scene_label: String,
    #[serde(default)]
    pub scene_category: String,
    pub duration_seconds: u16,
    #[serde(default = "default_target_duration_mode")]
    pub target_duration_mode: String,
    #[serde(default)]
    pub accepted_confirmation_body: String,
    pub story_fact_frame_hash: String,
    #[serde(default)]
    pub explicit_facts: Vec<String>,
    #[serde(default)]
    pub inferred_scene_facts: Vec<InferredSceneFactBinding>,
    #[serde(default)]
    pub must_keep_facts: Vec<String>,
    #[serde(default)]
    pub forbidden_facts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerateStoryboardRequest {
    pub task_name: String,
    pub script_id: Option<String>,
    #[serde(default)]
    pub scene_type: Option<String>,
    #[serde(default)]
    pub scene_label: Option<String>,
    #[serde(default)]
    pub scene_category: Option<String>,
    #[serde(default)]
    pub shot_script: Option<String>,
    #[serde(default)]
    pub primary_scene_type: Option<String>,
    #[serde(default)]
    pub primary_scene_label: Option<String>,
    #[serde(default)]
    pub primary_scene_category: Option<String>,
    #[serde(default)]
    pub shot_scene_type: Option<String>,
    #[serde(default)]
    pub shot_scene_label: Option<String>,
    #[serde(default)]
    pub shot_intent: Option<String>,
    #[serde(default)]
    pub adaptation_reason: Option<String>,
    pub expanded_script_text: Option<String>,
    pub selected_total_duration_seconds: u16,
    #[serde(default = "default_target_duration_mode")]
    pub target_duration_mode: String,
    #[serde(default)]
    pub auto_segment_strategy: String,
    #[serde(default)]
    pub model_config_summary: Option<ModelConfigSummary>,
    #[serde(default)]
    pub accepted_rewrite_snapshot: Option<AcceptedRewriteSnapshotBinding>,
    #[serde(default)]
    pub current_case_id: String,
    #[serde(default)]
    pub source_text_hash: String,
    #[serde(default)]
    pub accepted_rewrite_hash: String,
    #[serde(default)]
    pub task_script_hash: String,
    #[serde(default)]
    pub story_fact_frame_hash: String,
    #[serde(default)]
    pub source_profile: String,
    #[serde(default)]
    pub must_keep_facts: Vec<String>,
    #[serde(default)]
    pub forbidden_facts: Vec<String>,
}

impl Default for GenerateStoryboardRequest {
    fn default() -> Self {
        Self {
            task_name: String::new(),
            script_id: None,
            scene_type: None,
            scene_label: None,
            scene_category: None,
            shot_script: None,
            primary_scene_type: None,
            primary_scene_label: None,
            primary_scene_category: None,
            shot_scene_type: None,
            shot_scene_label: None,
            shot_intent: None,
            adaptation_reason: None,
            expanded_script_text: None,
            selected_total_duration_seconds: 0,
            target_duration_mode: default_target_duration_mode(),
            auto_segment_strategy: String::new(),
            model_config_summary: None,
            accepted_rewrite_snapshot: None,
            current_case_id: String::new(),
            source_text_hash: String::new(),
            accepted_rewrite_hash: String::new(),
            task_script_hash: String::new(),
            story_fact_frame_hash: String::new(),
            source_profile: String::new(),
            must_keep_facts: Vec::new(),
            forbidden_facts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneratedStoryboardRow {
    pub shot_id: String,
    pub order: u32,
    pub shot_script: String,
    pub primary_scene_type: String,
    pub primary_scene_label: String,
    pub primary_scene_category: String,
    pub shot_scene_type: String,
    pub shot_scene_label: String,
    pub shot_intent: String,
    pub adaptation_reason: String,
    pub grounding_source: ShotGroundingSource,
    pub person: String,
    pub shot_title: String,
    pub scene_scale: String,
    pub visual_description: String,
    pub character_action: String,
    #[serde(default)]
    pub camera_movement: String,
    pub dialogue: String,
    pub prompt_text: String,
    #[serde(default = "default_prompt_text_compilation_status")]
    pub prompt_text_compilation_status: PromptTextCompilationStatus,
    #[serde(default)]
    pub prompt_text_compilation_warnings: Vec<ProductWarning>,
    #[serde(default)]
    pub prompt_text_source_row_id: String,
    pub duration_seconds: u16,
    #[serde(default)]
    pub shot_duration_seconds: u16,
    #[serde(default)]
    pub duration_source: String,
    pub scene_performance_projection: ScenePerformanceProjection,
    pub external_reference_handle_candidates: Vec<ExternalReferenceHandleCandidate>,
    pub sequence_grouping: SequenceGrouping,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryboardDurationPlan {
    pub total_duration_seconds: u16,
    pub row_count: u32,
    pub per_row_seconds: u16,
    pub allocated_seconds: u16,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryboardBindingEvidence {
    #[serde(default)]
    pub current_case_id: String,
    #[serde(default)]
    pub source_text_hash: String,
    #[serde(default)]
    pub accepted_rewrite_hash: String,
    #[serde(default)]
    pub task_script_hash: String,
    #[serde(default)]
    pub story_fact_frame_hash: String,
    #[serde(default)]
    pub source_profile: String,
    #[serde(default)]
    pub scene_type: String,
    #[serde(default)]
    pub duration_seconds: u16,
    #[serde(default)]
    pub duration_plan_hash: String,
    #[serde(default)]
    pub storyboard_rows_hash: String,
    #[serde(default)]
    pub must_keep_facts: Vec<String>,
    #[serde(default)]
    pub missing_source_facts: Vec<String>,
    #[serde(default)]
    pub forbidden_facts: Vec<String>,
    #[serde(default)]
    pub forbidden_fact_hits: Vec<String>,
    #[serde(default)]
    pub stale_binding_detected: bool,
    #[serde(default)]
    pub kb_rule_pack_ids: Vec<String>,
    #[serde(default)]
    pub kb_snapshot_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryboardExportStatus {
    pub status: BridgeCallStatus,
    pub blockers: Vec<ProductWarning>,
    pub warnings: Vec<ProductWarning>,
    pub ready_row_count: u32,
    pub blocked_row_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerateStoryboardResponse {
    #[serde(default)]
    pub task_id: Option<String>,
    pub result_id: String,
    pub rows: Vec<GeneratedStoryboardRow>,
    #[serde(default)]
    pub selected_total_duration_seconds: u16,
    #[serde(default = "default_target_duration_mode")]
    pub target_duration_mode: String,
    #[serde(default)]
    pub story_length_profile: String,
    #[serde(default)]
    pub source_material_length_chars: u32,
    #[serde(default)]
    pub auto_segment_strategy: String,
    #[serde(default)]
    pub estimated_total_story_duration_seconds: u16,
    #[serde(default)]
    pub generated_shot_task_count: u32,
    #[serde(default)]
    pub duration_plan_summary: String,
    pub duration_plan: StoryboardDurationPlan,
    pub export_status: StoryboardExportStatus,
    #[serde(default)]
    pub busy: bool,
    #[serde(default)]
    pub operation_id: String,
    #[serde(default)]
    pub revision: u32,
    #[serde(default)]
    pub updated_at_ms: u64,
    #[serde(default)]
    pub rows_hash: String,
    #[serde(default)]
    pub dirty: bool,
    #[serde(default)]
    pub dirty_source_note: Option<String>,
    pub kb_router_result: KbRouterRuntimeResponse,
    #[serde(default)]
    pub binding_evidence: StoryboardBindingEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FinalizedStoryboardShotResult {
    pub project_id: String,
    pub script_id: String,
    pub shot_task_id: String,
    pub result_id: String,
    pub shot_order: u32,
    pub shot_task_name: String,
    pub rows: Vec<GeneratedStoryboardRow>,
    pub prompt_text: String,
    pub shot_duration_seconds: u16,
    pub duration_source: String,
    pub confirmed: bool,
    pub updated_at_ms: u64,
    pub rows_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaveStoryboardShotResultRequest {
    pub project_id: String,
    pub script_id: String,
    pub shot_task_id: String,
    pub result_id: String,
    pub shot_order: u32,
    pub shot_task_name: String,
    pub rows: Vec<GeneratedStoryboardRow>,
    pub prompt_text: String,
    pub shot_duration_seconds: u16,
    pub duration_source: String,
    pub confirmed: bool,
    #[serde(default)]
    pub updated_at_ms: u64,
    #[serde(default)]
    pub rows_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaveStoryboardShotResultResponse {
    pub status: BridgeCallStatus,
    pub shot: Option<FinalizedStoryboardShotResult>,
    pub blockers: Vec<ProductWarning>,
    pub warnings: Vec<ProductWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListStoryboardShotResultsRequest {
    pub project_id: String,
    #[serde(default)]
    pub script_id: Option<String>,
    #[serde(default)]
    pub confirmed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListStoryboardShotResultsResponse {
    pub project_id: String,
    pub script_id: Option<String>,
    pub shots: Vec<FinalizedStoryboardShotResult>,
    pub warnings: Vec<ProductWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateStoryboardShotResultRequest {
    pub project_id: String,
    pub result_id: String,
    pub shot_order: u32,
    pub shot_task_name: String,
    pub rows: Vec<GeneratedStoryboardRow>,
    pub prompt_text: String,
    pub shot_duration_seconds: u16,
    pub duration_source: String,
    pub confirmed: bool,
    #[serde(default)]
    pub updated_at_ms: u64,
    #[serde(default)]
    pub rows_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateStoryboardShotResultResponse {
    pub status: BridgeCallStatus,
    pub shot: Option<FinalizedStoryboardShotResult>,
    pub blockers: Vec<ProductWarning>,
    pub warnings: Vec<ProductWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoveStoryboardShotResultRequest {
    pub project_id: String,
    pub result_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoveStoryboardShotResultResponse {
    pub status: BridgeCallStatus,
    pub project_id: String,
    pub result_id: String,
    pub removed: bool,
    pub warnings: Vec<ProductWarning>,
    pub blockers: Vec<ProductWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportStoryboardBankRequest {
    pub project_id: String,
    #[serde(default)]
    pub script_id: Option<String>,
    pub export_format: String,
    #[serde(default)]
    pub include_unconfirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportStoryboardBankResponse {
    pub export_manifest_id: String,
    pub project_id: String,
    pub script_id: Option<String>,
    pub status: BridgeCallStatus,
    pub no_export: bool,
    pub confirmed_shot_count: u32,
    pub exported_result_ids: Vec<String>,
    pub total_shot_duration_seconds: u16,
    pub artifacts: Vec<ExportArtifactRecord>,
    pub warnings: Vec<ProductWarning>,
    pub blockers: Vec<ProductWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportBundleRequest {
    #[serde(default)]
    pub result_id: Option<String>,
    #[serde(default)]
    pub task_id: Option<String>,
    pub export_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateStoryboardRowsRequest {
    pub result_id: String,
    pub task_id: Option<String>,
    pub rows: Vec<GeneratedStoryboardRow>,
    pub operation_id: String,
    pub base_revision: u32,
    pub dirty_source_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportArtifactRecord {
    pub artifact_id: String,
    pub artifact_kind: String,
    pub export_format: String,
    pub ready: bool,
    pub blocked_reason: Option<String>,
    pub artifact_path: Option<String>,
    pub content_hash: Option<String>,
    pub byte_size: Option<u64>,
    pub row_count: Option<u32>,
    #[serde(default)]
    pub selected_total_duration_seconds: Option<u16>,
    #[serde(default)]
    pub source_result_id: Option<String>,
    #[serde(default)]
    pub edited_rows_applied: bool,
    #[serde(default)]
    pub prompt_text_compilation_statuses: Vec<String>,
    #[serde(default)]
    pub prompt_text_compilation_warning_codes: Vec<String>,
    #[serde(default)]
    pub selected_sample_ids: Vec<String>,
    #[serde(default)]
    pub selected_kb_rule_ids: Vec<String>,
    #[serde(default)]
    pub kb_context_summary: Option<String>,
    #[serde(default)]
    pub retrieval_trace: Option<KbRouterRetrievalTrace>,
    #[serde(default)]
    pub full_kb_rows_included: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportBundleResponse {
    pub export_manifest_id: String,
    pub export_status: StoryboardExportStatus,
    pub artifacts: Vec<ExportArtifactRecord>,
}

fn default_prompt_text_compilation_status() -> PromptTextCompilationStatus {
    PromptTextCompilationStatus::Blocked
}

fn default_bridge_call_status() -> BridgeCallStatus {
    BridgeCallStatus::Ready
}
