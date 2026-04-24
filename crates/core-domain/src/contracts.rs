use crate::domain::HierarchyRef;

use serde::{Deserialize, Serialize};

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
pub struct PromptBodyCandidate {
    pub source_sample_id: String,
    pub source_prompt_body: String,
    pub candidate_text: Option<String>,
    pub blocked: bool,
    pub blocker_codes: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpandScriptRequest {
    pub scene_type: String,
    #[serde(default)]
    pub scene_label: Option<String>,
    #[serde(default)]
    pub scene_category: Option<String>,
    pub synopsis_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpandScriptResponse {
    pub script_id: String,
    pub expanded_script_text: String,
    pub script_hash: String,
    pub warnings: Vec<ProductWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerateStoryboardRequest {
    pub task_name: String,
    pub script_id: Option<String>,
    pub expanded_script_text: Option<String>,
    pub selected_total_duration_seconds: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneratedStoryboardRow {
    pub shot_id: String,
    pub order: u32,
    pub person: String,
    pub shot_title: String,
    pub scene_scale: String,
    pub visual_description: String,
    pub character_action: String,
    pub dialogue: String,
    pub prompt_text: String,
    pub duration_seconds: u16,
    pub prompt_body_candidate: PromptBodyCandidate,
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
    pub result_id: String,
    pub rows: Vec<GeneratedStoryboardRow>,
    pub duration_plan: StoryboardDurationPlan,
    pub export_status: StoryboardExportStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportBundleRequest {
    pub result_id: String,
    pub export_format: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportBundleResponse {
    pub export_manifest_id: String,
    pub export_status: StoryboardExportStatus,
    pub artifacts: Vec<ExportArtifactRecord>,
}
