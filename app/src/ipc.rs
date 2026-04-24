use serde::{Deserialize, Serialize};

pub type ExpandScriptRequest = core_domain::ExpandScriptRequest;
pub type GenerateStoryboardRequest = core_domain::GenerateStoryboardRequest;
pub type ExportBundleRequest = core_domain::ExportBundleRequest;
pub type UpdateStoryboardRowsRequest = core_domain::UpdateStoryboardRowsRequest;

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

pub const IPC_COMMANDS: &[&str] = &[
    PROJECT_CREATE_OR_SWITCH_COMMAND,
    WRITER_ENTRY_SNAPSHOT_COMMAND,
    STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
    VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND,
    EXPAND_SCRIPT_COMMAND,
    GENERATE_STORYBOARD_COMMAND,
    UPDATE_STORYBOARD_ROWS_COMMAND,
    EXPORT_BUNDLE_COMMAND,
];
