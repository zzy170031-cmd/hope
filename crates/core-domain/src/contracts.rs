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
pub enum KbRouterTaskType {
    GenerateNovelChapter,
    AdaptChapterToScript,
    SplitScriptToShotTasks,
    ExpandScript,
    GenerateStoryboard,
    RepairStoryboard,
    CompileSeedancePromptText,
    UpdateContinuityState,
}

impl KbRouterTaskType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GenerateNovelChapter => "generate_novel_chapter",
            Self::AdaptChapterToScript => "adapt_chapter_to_script",
            Self::SplitScriptToShotTasks => "split_script_to_shot_tasks",
            Self::ExpandScript => "expand_script",
            Self::GenerateStoryboard => "generate_storyboard",
            Self::RepairStoryboard => "repair_storyboard",
            Self::CompileSeedancePromptText => "compile_seedance_prompt_text",
            Self::UpdateContinuityState => "update_continuity_state",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KbRouterRuntimeRequest {
    pub scene_type: String,
    pub synopsis_text: String,
    pub duration_seconds: u16,
    pub task_type: KbRouterTaskType,
    pub primary_scene_type: Option<String>,
    pub primary_scene_label: Option<String>,
    pub shot_scene_type: Option<String>,
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
pub struct PromptTextCompilationRow {
    pub shot_id: String,
    pub shot_title: String,
    pub scene_scale: String,
    pub visual_description: String,
    pub character_action: String,
    #[serde(default)]
    pub camera_movement: String,
    pub dialogue: String,
    pub duration_seconds: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptTextCompilationRequest {
    pub row: PromptTextCompilationRow,
    pub shot_script: Option<String>,
    pub scene_type: String,
    pub scene_label: String,
    pub kb_context_summary: String,
    pub selected_kb_rules: Vec<String>,
    pub continuity_negative_core: String,
    pub target_profile: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PromptTextCompilationStatus {
    ReadyStub,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptTextCompilationResponse {
    pub prompt_text: String,
    pub target_profile: String,
    pub compilation_status: PromptTextCompilationStatus,
    pub source_row_id: String,
    pub warnings: Vec<ProductWarning>,
    pub provider: TextModelProviderKind,
    pub model: String,
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
pub struct GenerateNovelChapterRequest {
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_order: u32,
    pub user_topic_or_synopsis: String,
    pub story_length_profile: String,
    pub authoring_craft_summary: String,
    pub continuity_context_summary: String,
    pub kb_context_summary: String,
    pub selected_sample_ids: Vec<String>,
    pub selected_kb_rules: Vec<String>,
    pub retrieval_trace_user_summary: String,
    pub full_kb_rows_included: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerateNovelChapterResponse {
    pub status: BridgeCallStatus,
    pub blockers: Vec<ProductWarning>,
    pub warnings: Vec<ProductWarning>,
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_order: u32,
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
    pub chapter_title: String,
    pub chapter_text: String,
    pub chapter_summary: String,
    pub authoring_craft_summary: String,
    pub premise_hook: String,
    pub character_desire: String,
    pub character_pressure: String,
    pub conflict_engine: String,
    pub emotional_turn: String,
    pub suspense_setup: String,
    pub payoff_setup: String,
    pub scene_purpose: String,
    pub visualizable_action: String,
    pub chapter_cliffhanger: String,
    pub director_bridge: String,
    pub character_motivation_summary: String,
    pub conflict_progression_summary: String,
    pub emotional_progression_summary: String,
    pub timeline_continuity_summary: String,
    pub prop_state_summary: String,
    pub next_scene_bridge: String,
    pub continuity_warnings: Vec<ProductWarning>,
    pub continuity_delta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChapterAcceptedState {
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_order: u32,
    pub chapter_title: String,
    pub chapter_text: String,
    pub chapter_summary: String,
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
    pub continuity_delta: String,
    pub accepted_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdaptChapterToScriptRequest {
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_order: u32,
    pub chapter_text: String,
    pub chapter_summary: String,
    #[serde(default)]
    pub source_input_type: String,
    #[serde(default)]
    pub authoring_mode: String,
    #[serde(default)]
    pub source_story_facts: SourceStoryFacts,
    #[serde(default)]
    pub preserved_fact_summary: String,
    #[serde(default)]
    pub source_material_summary: String,
    pub screenwriting_adaptation_summary: String,
    pub continuity_context_summary: String,
    pub kb_context_summary: String,
    pub selected_sample_ids: Vec<String>,
    pub selected_kb_rules: Vec<String>,
    pub retrieval_trace_user_summary: String,
    pub full_kb_rows_included: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdaptChapterToScriptResponse {
    pub status: BridgeCallStatus,
    pub blockers: Vec<ProductWarning>,
    pub warnings: Vec<ProductWarning>,
    pub script_id: String,
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_order: u32,
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
    pub script_text: String,
    pub script_summary: String,
    pub screenwriting_adaptation_summary: String,
    pub scene_beats: Vec<String>,
    pub dialogue_intent: String,
    pub action_blocks: Vec<String>,
    pub turning_points: Vec<String>,
    pub scene_purpose: String,
    #[serde(default)]
    pub continuity_warnings: Vec<ProductWarning>,
    pub continuity_delta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptAcceptedState {
    pub script_id: String,
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_order: u32,
    pub script_text: String,
    pub script_summary: String,
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
    pub continuity_delta: String,
    pub accepted_at_ms: u64,
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
pub struct SplitScriptToShotTasksRequest {
    pub script_id: Option<String>,
    pub expanded_script_text: String,
    pub selected_total_duration_seconds: u16,
    #[serde(default = "default_target_duration_mode")]
    pub target_duration_mode: String,
    #[serde(default)]
    pub auto_segment_strategy: String,
    pub primary_scene_type: String,
    pub primary_scene_label: Option<String>,
    pub primary_scene_category: Option<String>,
    pub task_type: Option<String>,
    pub shot_count_hint: Option<u32>,
    pub structure_type: Option<String>,
    pub kb_context_summary: Option<String>,
    pub selected_kb_rules: Vec<String>,
    pub selected_sample_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShotTask {
    pub shot_task_id: String,
    pub shot_order: u32,
    pub shot_script: String,
    pub duration_seconds: u16,
    pub shot_scene_type: String,
    pub shot_scene_label: String,
    pub shot_intent: String,
    pub adaptation_reason: String,
    pub grounding_source: ShotGroundingSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SplitScriptToShotTasksResponse {
    pub script_id: Option<String>,
    pub shot_tasks: Vec<ShotTask>,
    pub warnings: Vec<ProductWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShotTaskPlan {
    pub script_id: String,
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_order: u32,
    pub shot_tasks: Vec<ShotTask>,
    pub shot_task_count: u32,
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
    pub duration_plan_summary: String,
    pub continuity_delta: String,
    pub warnings: Vec<ProductWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpandScriptRequest {
    pub scene_type: String,
    pub synopsis_text: String,
    #[serde(default)]
    pub target_duration_seconds: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpandScriptResponse {
    pub status: BridgeCallStatus,
    pub script_id: String,
    pub expanded_script_text: String,
    pub script_hash: String,
    pub blockers: Vec<ProductWarning>,
    pub warnings: Vec<ProductWarning>,
    pub kb_router_result: KbRouterRuntimeResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerateStoryboardRequest {
    pub task_name: String,
    pub script_id: Option<String>,
    pub shot_script: Option<String>,
    pub expanded_script_text: Option<String>,
    pub primary_scene_type: Option<String>,
    pub primary_scene_label: Option<String>,
    pub primary_scene_category: Option<String>,
    pub shot_scene_type: Option<String>,
    pub shot_scene_label: Option<String>,
    pub shot_intent: Option<String>,
    pub adaptation_reason: Option<String>,
    pub selected_total_duration_seconds: u16,
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
    pub prompt_text_compilation_status: PromptTextCompilationStatus,
    pub prompt_text_compilation_warnings: Vec<ProductWarning>,
    pub prompt_text_source_row_id: String,
    pub duration_seconds: u16,
    pub shot_duration_seconds: u16,
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
    pub task_id: Option<String>,
    pub result_id: String,
    pub rows: Vec<GeneratedStoryboardRow>,
    pub selected_total_duration_seconds: u16,
    pub kb_router_result: KbRouterRuntimeResponse,
    pub duration_plan: StoryboardDurationPlan,
    pub export_status: StoryboardExportStatus,
    pub busy: bool,
    pub operation_id: String,
    pub revision: u32,
    pub updated_at_ms: u64,
    pub rows_hash: String,
    pub dirty: bool,
    pub dirty_source_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryboardResult {
    pub status: BridgeCallStatus,
    pub blockers: Vec<ProductWarning>,
    pub warnings: Vec<ProductWarning>,
    pub storyboard_result_id: String,
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_order: u32,
    pub script_id: String,
    pub shot_task_id: String,
    pub shot_order: u32,
    pub rows: Vec<GeneratedStoryboardRow>,
    pub prompt_text: String,
    pub shot_duration_seconds: u16,
    pub duration_source: String,
    pub director_intent_summary: String,
    pub performance_focus: String,
    pub blocking_hint: String,
    pub rhythm_hint: String,
    pub visual_focus: String,
    pub continuity_note: String,
    pub directing_kb_context_summary: String,
    pub continuity_delta: String,
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
pub struct FinalizedStoryboardRef {
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_order: u32,
    pub script_id: String,
    pub shot_task_id: String,
    pub storyboard_result_id: String,
    pub finalized_result_id: String,
    pub shot_order: u32,
    pub shot_task_name: String,
    pub confirmed: bool,
    pub updated_at_ms: u64,
    pub rows_hash: String,
    pub shot_duration_seconds: u16,
    pub label: String,
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
    pub updated_at_ms: u64,
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
    pub script_id: Option<String>,
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
    pub updated_at_ms: u64,
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
    pub script_id: Option<String>,
    pub export_format: String,
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
pub struct UpdateContinuityStateRequest {
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_order: u32,
    pub chapter_summary: String,
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
    pub character_state_summary: String,
    pub location_state_summary: String,
    pub prop_state_summary: String,
    pub timeline_state_summary: String,
    pub unresolved_threads: Vec<String>,
    pub style_bible_summary: String,
    pub finalized_storyboard_refs: Vec<FinalizedStoryboardRef>,
    pub continuity_delta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryContinuityState {
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_order: u32,
    pub continuity_context_summary: String,
    pub chapter_summary: String,
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
    pub character_state_summary: String,
    pub location_state_summary: String,
    pub prop_state_summary: String,
    pub timeline_state_summary: String,
    pub unresolved_threads: Vec<String>,
    pub style_bible_summary: String,
    pub finalized_storyboard_refs: Vec<FinalizedStoryboardRef>,
    pub continuity_delta: String,
    pub updated_at_ms: u64,
    pub warnings: Vec<ProductWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateContinuityStateResponse {
    pub status: BridgeCallStatus,
    pub blockers: Vec<ProductWarning>,
    pub warnings: Vec<ProductWarning>,
    pub continuity_state: Option<StoryContinuityState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContinuityDeltaLog {
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_order: u32,
    pub stage: String,
    pub continuity_delta: String,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunV0StoryToStoryboardChainRequest {
    pub story_id: String,
    pub chapter_id: String,
    pub chapter_order: u32,
    pub user_topic_or_synopsis: String,
    pub story_length_profile: String,
    pub selected_total_duration_seconds: u16,
    #[serde(default = "default_target_duration_mode")]
    pub target_duration_mode: String,
    #[serde(default)]
    pub source_material_length_chars: u32,
    #[serde(default)]
    pub auto_segment_strategy: String,
    #[serde(default)]
    pub estimated_total_story_duration_seconds: u16,
    pub primary_scene_type: String,
    pub primary_scene_label: Option<String>,
    pub primary_scene_category: Option<String>,
    pub authoring_craft_summary: String,
    pub screenwriting_adaptation_summary: String,
    pub directing_kb_context_summary: String,
    pub continuity_context_summary: String,
    pub kb_context_summary: String,
    pub selected_sample_ids: Vec<String>,
    pub selected_kb_rules: Vec<String>,
    pub retrieval_trace_user_summary: String,
    pub full_kb_rows_included: u32,
    pub confirm_storyboard_results: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunV0StoryToStoryboardChainResponse {
    pub status: BridgeCallStatus,
    pub blockers: Vec<ProductWarning>,
    pub warnings: Vec<ProductWarning>,
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
    pub chapter: Option<GenerateNovelChapterResponse>,
    pub script: Option<AdaptChapterToScriptResponse>,
    pub shot_task_plan: Option<ShotTaskPlan>,
    pub storyboard_results: Vec<StoryboardResult>,
    pub finalized_storyboard_refs: Vec<FinalizedStoryboardRef>,
    pub continuity_state: Option<StoryContinuityState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportBundleRequest {
    pub result_id: Option<String>,
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
    pub selected_total_duration_seconds: Option<u16>,
    pub source_result_id: Option<String>,
    pub edited_rows_applied: bool,
    pub prompt_text_compilation_statuses: Vec<String>,
    pub prompt_text_compilation_warning_codes: Vec<String>,
    pub selected_sample_ids: Vec<String>,
    pub selected_kb_rule_ids: Vec<String>,
    pub kb_context_summary: Option<String>,
    pub retrieval_trace: Option<KbRouterRetrievalTrace>,
    pub full_kb_rows_included: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportBundleResponse {
    pub export_manifest_id: String,
    pub export_status: StoryboardExportStatus,
    pub artifacts: Vec<ExportArtifactRecord>,
}
