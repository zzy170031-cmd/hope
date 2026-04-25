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
    pub selected_total_duration_seconds: Option<u16>,
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
    pub script_id: String,
    pub expanded_script_text: String,
    pub script_hash: String,
    pub warnings: Vec<ProductWarning>,
    pub kb_router_result: KbRouterRuntimeResponse,
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
    #[serde(default)]
    pub model_config_summary: Option<ModelConfigSummary>,
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
