use serde::{Deserialize, Serialize};

pub type ExpandScriptRequest = core_domain::ExpandScriptRequest;
pub type GenerateStoryboardRequest = core_domain::GenerateStoryboardRequest;
pub type ExportBundleRequest = core_domain::ExportBundleRequest;
pub type UpdateStoryboardRowsRequest = core_domain::UpdateStoryboardRowsRequest;
pub type SaveStoryboardShotResultRequest = core_domain::SaveStoryboardShotResultRequest;
pub type ListStoryboardShotResultsRequest = core_domain::ListStoryboardShotResultsRequest;
pub type UpdateStoryboardShotResultRequest = core_domain::UpdateStoryboardShotResultRequest;
pub type RemoveStoryboardShotResultRequest = core_domain::RemoveStoryboardShotResultRequest;
pub type ExportStoryboardBankRequest = core_domain::ExportStoryboardBankRequest;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ConfigureTextModelProviderRequest {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_key_ref: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TextModelProviderStatus {
    pub provider: String,
    pub model: String,
    pub enabled: bool,
    pub base_url_present: bool,
    pub api_key_present: bool,
    pub live_ready: bool,
    pub status: String,
    pub message: String,
    pub storage: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectCreateOrSwitchRequest {
    pub project_id: Option<String>,
    pub project_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WriterEntrySnapshotRequest {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct StoryboardRenderSegmentCutPreviewSnapshotRequest {
    pub project_id: String,
    pub episode_id: Option<String>,
    pub narrative_scene_id: Option<String>,
    pub render_segment_id: Option<String>,
    pub scene_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ValidationExportPanelSnapshotRequest {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct EmptyResponse;

pub const PROJECT_CREATE_OR_SWITCH_COMMAND: &str = "project_create_or_switch";
pub const WRITER_ENTRY_SNAPSHOT_COMMAND: &str = "writer_entry_snapshot";
pub const STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND: &str =
    "storyboard_rendersegment_cut_preview_snapshot";
pub const VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND: &str = "validation_export_panel_snapshot";
pub const EXPAND_SCRIPT_COMMAND: &str = "expand_script";
pub const GENERATE_STORYBOARD_COMMAND: &str = "generate_storyboard";
pub const UPDATE_STORYBOARD_ROWS_COMMAND: &str = "update_storyboard_rows";
pub const EXPORT_BUNDLE_COMMAND: &str = "export_bundle";
pub const CONFIGURE_TEXT_MODEL_PROVIDER_COMMAND: &str = "configure_text_model_provider";
pub const SAVE_STORYBOARD_SHOT_RESULT_COMMAND: &str = "save_storyboard_shot_result";
pub const LIST_STORYBOARD_SHOT_RESULTS_COMMAND: &str = "list_storyboard_shot_results";
pub const UPDATE_STORYBOARD_SHOT_RESULT_COMMAND: &str = "update_storyboard_shot_result";
pub const REMOVE_STORYBOARD_SHOT_RESULT_COMMAND: &str = "remove_storyboard_shot_result";
pub const EXPORT_STORYBOARD_BANK_COMMAND: &str = "export_storyboard_bank";

pub const IPC_COMMANDS: &[&str] = &[
    PROJECT_CREATE_OR_SWITCH_COMMAND,
    WRITER_ENTRY_SNAPSHOT_COMMAND,
    STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
    VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND,
    EXPAND_SCRIPT_COMMAND,
    GENERATE_STORYBOARD_COMMAND,
    UPDATE_STORYBOARD_ROWS_COMMAND,
    EXPORT_BUNDLE_COMMAND,
    CONFIGURE_TEXT_MODEL_PROVIDER_COMMAND,
    SAVE_STORYBOARD_SHOT_RESULT_COMMAND,
    LIST_STORYBOARD_SHOT_RESULTS_COMMAND,
    UPDATE_STORYBOARD_SHOT_RESULT_COMMAND,
    REMOVE_STORYBOARD_SHOT_RESULT_COMMAND,
    EXPORT_STORYBOARD_BANK_COMMAND,
];
