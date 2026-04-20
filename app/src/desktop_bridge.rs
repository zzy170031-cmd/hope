use std::io;

use crate::{
    ipc::{
        PROJECT_CREATE_OR_SWITCH_COMMAND, ProjectCreateOrSwitchRequest,
        WRITER_ENTRY_SNAPSHOT_COMMAND, WriterEntrySnapshotRequest,
    },
    runtime::{
        ProjectCreateOrSwitchSnapshot, WriterEntrySnapshot, build_project_create_or_switch_snapshot,
        build_writer_entry_snapshot,
    },
};

pub const DESKTOP_INVOKE_COMMANDS: &[&str] =
    &[PROJECT_CREATE_OR_SWITCH_COMMAND, WRITER_ENTRY_SNAPSHOT_COMMAND];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopInvokeRequest {
    ProjectCreateOrSwitch(ProjectCreateOrSwitchRequest),
    WriterEntrySnapshot(WriterEntrySnapshotRequest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopInvokeResponse {
    ProjectCreateOrSwitch(ProjectCreateOrSwitchSnapshot),
    WriterEntrySnapshot(WriterEntrySnapshot),
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
        _ => Err(DesktopInvokeError::UnsupportedCommand {
            command: command.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::ipc::{ProjectCreateOrSwitchRequest, WriterEntrySnapshotRequest};

    use super::{
        DESKTOP_INVOKE_COMMANDS, DesktopInvokeRequest, DesktopInvokeResponse,
        PROJECT_CREATE_OR_SWITCH_COMMAND, WRITER_ENTRY_SNAPSHOT_COMMAND, desktop_invoke_contract,
        invoke_desktop_command,
    };

    #[test]
    fn desktop_invoke_contract_registers_phase1_commands_only() {
        assert_eq!(desktop_invoke_contract(), DESKTOP_INVOKE_COMMANDS);
        assert_eq!(
            desktop_invoke_contract(),
            &[PROJECT_CREATE_OR_SWITCH_COMMAND, WRITER_ENTRY_SNAPSHOT_COMMAND]
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
    fn invoke_desktop_command_keeps_preview_and_export_closed() {
        for command in [
            "storyboard_rendersegment_cut_preview_snapshot",
            "validation_export_panel_snapshot",
        ] {
            let error = invoke_desktop_command(
                command,
                DesktopInvokeRequest::ProjectCreateOrSwitch(ProjectCreateOrSwitchRequest {
                    project_id: None,
                    project_name: None,
                }),
            )
            .expect_err("preview and export should stay closed in the writer step");

            assert!(error
                .to_string()
                .contains("desktop invoke command is not registered yet"));
        }
    }
}
