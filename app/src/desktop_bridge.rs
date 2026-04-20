use std::io;

use crate::{
    ipc::{
        PROJECT_CREATE_OR_SWITCH_COMMAND, ProjectCreateOrSwitchRequest,
        STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
        StoryboardRenderSegmentCutPreviewSnapshotRequest, WRITER_ENTRY_SNAPSHOT_COMMAND,
        WriterEntrySnapshotRequest,
    },
    runtime::{
        ProjectCreateOrSwitchSnapshot, StoryboardRenderSegmentCutPreviewSnapshot,
        WriterEntrySnapshot, build_project_create_or_switch_snapshot,
        build_storyboard_rendersegment_cut_preview_snapshot, build_writer_entry_snapshot,
    },
    state::AppState,
};
use core_domain::{KbRuntimeSummary, KbSnapshotRecord, SceneTaxonomyRecord};
use project_store::{DualSqliteConnectionPolicy, KbKnowledgeBundle, KbRuntimeHandle, StoreSkeleton};

pub const DESKTOP_INVOKE_COMMANDS: &[&str] =
    &[
        PROJECT_CREATE_OR_SWITCH_COMMAND,
        WRITER_ENTRY_SNAPSHOT_COMMAND,
        STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
    ];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopInvokeRequest {
    ProjectCreateOrSwitch(ProjectCreateOrSwitchRequest),
    WriterEntrySnapshot(WriterEntrySnapshotRequest),
    StoryboardRenderSegmentCutPreviewSnapshot(StoryboardRenderSegmentCutPreviewSnapshotRequest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopInvokeResponse {
    ProjectCreateOrSwitch(ProjectCreateOrSwitchSnapshot),
    WriterEntrySnapshot(WriterEntrySnapshot),
    StoryboardRenderSegmentCutPreviewSnapshot(StoryboardRenderSegmentCutPreviewSnapshot),
}

#[derive(Debug)]
pub enum DesktopInvokeError {
    UnsupportedCommand { command: String },
    Io(io::Error),
}

impl std::fmt::Display for DesktopInvokeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedCommand { command } => {
                write!(f, "desktop invoke command is not registered yet: {}", command)
            }
            Self::Io(error) => write!(f, "desktop invoke failed: {}", error),
        }
    }
}

impl std::error::Error for DesktopInvokeError {}

impl From<io::Error> for DesktopInvokeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn desktop_invoke_contract() -> &'static [&'static str] {
    DESKTOP_INVOKE_COMMANDS
}

fn desktop_preview_state() -> AppState {
    let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
        "E:/codex/hope-desktop-shell/data/desktop-preview-hope-kb.sqlite3".into(),
        "E:/codex/hope-desktop-shell/data/desktop-preview-hope.sqlite3".into(),
    ));
    let kb_runtime = KbRuntimeHandle {
        snapshot: KbSnapshotRecord {
            snapshot_id: "desktop-shell-preview".to_string(),
            snapshot_hash: "desktop-preview-hash".to_string(),
            seed_format: "hope-kb-sqlite-snapshot-v0.1".to_string(),
            source_name: "desktop-shell".to_string(),
            created_at_timestamp: 1_777_000_000,
        },
        summary: KbRuntimeSummary {
            snapshot_id: "desktop-shell-preview".to_string(),
            snapshot_path: "E:/codex/hope-desktop-shell/data/desktop-preview-hope-kb.sqlite3"
                .to_string(),
            mirror_tables: vec!["scene_taxonomy".to_string()],
            has_scene_taxonomy: true,
            has_failure_patterns: false,
            has_repair_template_mapping: false,
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
            source_type: "desktop-shell-fixture".to_string(),
            source_notes: "Preview desktop invoke baseline".to_string(),
            confidence_level: "high".to_string(),
            last_reviewed_at: "2026-04-20".to_string(),
        }],
        failure_patterns: vec![],
        prompt_templates: vec![],
    };

    AppState::new(store, kb_runtime, kb_knowledge)
}

pub fn invoke_desktop_command(
    command: &str,
    request: DesktopInvokeRequest,
) -> Result<DesktopInvokeResponse, DesktopInvokeError> {
    match (command, request) {
        (PROJECT_CREATE_OR_SWITCH_COMMAND, DesktopInvokeRequest::ProjectCreateOrSwitch(request)) => {
            let snapshot = build_project_create_or_switch_snapshot(request)?;
            Ok(DesktopInvokeResponse::ProjectCreateOrSwitch(snapshot))
        }
        (WRITER_ENTRY_SNAPSHOT_COMMAND, DesktopInvokeRequest::WriterEntrySnapshot(request)) => {
            let snapshot = build_writer_entry_snapshot(request)?;
            Ok(DesktopInvokeResponse::WriterEntrySnapshot(snapshot))
        }
        (
            STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
            DesktopInvokeRequest::StoryboardRenderSegmentCutPreviewSnapshot(request),
        ) => {
            let state = desktop_preview_state();
            let snapshot = build_storyboard_rendersegment_cut_preview_snapshot(&state, request)?;
            Ok(DesktopInvokeResponse::StoryboardRenderSegmentCutPreviewSnapshot(
                snapshot,
            ))
        }
        _ => Err(DesktopInvokeError::UnsupportedCommand {
            command: command.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::ipc::{
        ProjectCreateOrSwitchRequest, StoryboardRenderSegmentCutPreviewSnapshotRequest,
        WriterEntrySnapshotRequest,
    };

    use super::{
        DESKTOP_INVOKE_COMMANDS, DesktopInvokeRequest, DesktopInvokeResponse,
        PROJECT_CREATE_OR_SWITCH_COMMAND, STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
        WRITER_ENTRY_SNAPSHOT_COMMAND, desktop_invoke_contract, invoke_desktop_command,
    };

    #[test]
    fn desktop_invoke_contract_registers_phase1_commands_only() {
        assert_eq!(desktop_invoke_contract(), DESKTOP_INVOKE_COMMANDS);
        assert_eq!(
            desktop_invoke_contract(),
            &[
                PROJECT_CREATE_OR_SWITCH_COMMAND,
                WRITER_ENTRY_SNAPSHOT_COMMAND,
                STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
            ]
        );
    }

    #[test]
    fn invoke_desktop_command_returns_real_project_snapshot() {
        let response = invoke_desktop_command(
            PROJECT_CREATE_OR_SWITCH_COMMAND,
            DesktopInvokeRequest::ProjectCreateOrSwitch(ProjectCreateOrSwitchRequest {
                project_id: None,
                project_name: None,
            }),
        )
        .expect("project desktop invoke should succeed");

        match response {
            DesktopInvokeResponse::ProjectCreateOrSwitch(snapshot) => {
                assert_eq!(snapshot.current_project_id.as_deref(), Some("project-week3-001"));
                assert!(!snapshot.projects.is_empty());
            }
            _ => panic!("project invoke should return the project snapshot variant"),
        }
    }

    #[test]
    fn invoke_desktop_command_returns_real_writer_snapshot() {
        let response = invoke_desktop_command(
            WRITER_ENTRY_SNAPSHOT_COMMAND,
            DesktopInvokeRequest::WriterEntrySnapshot(WriterEntrySnapshotRequest {
                project_id: "project-week3-001".to_string(),
            }),
        )
        .expect("writer desktop invoke should succeed");

        match response {
            DesktopInvokeResponse::WriterEntrySnapshot(snapshot) => {
                assert_eq!(snapshot.project_id, "project-week3-001");
                assert!(snapshot.synopsis.contains("分钟"));
                assert!(snapshot.story.contains("第 1 集"));
            }
            _ => panic!("writer invoke should return the writer snapshot variant"),
        }
    }

    #[test]
    fn invoke_desktop_command_returns_real_preview_snapshot() {
        let response = invoke_desktop_command(
            STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
            DesktopInvokeRequest::StoryboardRenderSegmentCutPreviewSnapshot(
                StoryboardRenderSegmentCutPreviewSnapshotRequest {
                    project_id: "project-week3-001".to_string(),
                    episode_id: None,
                    narrative_scene_id: None,
                    render_segment_id: None,
                    scene_type: Some("daily_dialogue".to_string()),
                },
            ),
        )
        .expect("preview desktop invoke should succeed");

        match response {
            DesktopInvokeResponse::StoryboardRenderSegmentCutPreviewSnapshot(snapshot) => {
                assert_eq!(snapshot.project_id, "project-week3-001");
                assert!(!snapshot.storyboard.is_empty());
                assert!(!snapshot.render_segment.is_empty());
                assert!(!snapshot.cuts.is_empty());
                assert!(snapshot.render_segment[0].note.contains("handoff"));
            }
            _ => panic!("preview invoke should return the preview snapshot variant"),
        }
    }

    #[test]
    fn invoke_desktop_command_keeps_export_closed() {
        let error = invoke_desktop_command(
            "validation_export_panel_snapshot",
            DesktopInvokeRequest::ProjectCreateOrSwitch(ProjectCreateOrSwitchRequest {
                project_id: None,
                project_name: None,
            }),
        )
        .expect_err("export should stay closed in the preview step");

        assert!(error
            .to_string()
            .contains("desktop invoke command is not registered yet"));
    }
}
