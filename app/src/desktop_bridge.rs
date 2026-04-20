use std::io;

use crate::{
    ipc::{PROJECT_CREATE_OR_SWITCH_COMMAND, ProjectCreateOrSwitchRequest},
    runtime::{ProjectCreateOrSwitchSnapshot, build_project_create_or_switch_snapshot},
};

pub const DESKTOP_INVOKE_COMMANDS: &[&str] = &[PROJECT_CREATE_OR_SWITCH_COMMAND];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopInvokeRequest {
    ProjectCreateOrSwitch(ProjectCreateOrSwitchRequest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopInvokeResponse {
    ProjectCreateOrSwitch(ProjectCreateOrSwitchSnapshot),
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
        _ => Err(DesktopInvokeError::UnsupportedCommand {
            command: command.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::ipc::ProjectCreateOrSwitchRequest;

    use super::{
        DESKTOP_INVOKE_COMMANDS, DesktopInvokeRequest, DesktopInvokeResponse,
        PROJECT_CREATE_OR_SWITCH_COMMAND, desktop_invoke_contract, invoke_desktop_command,
    };

    #[test]
    fn desktop_invoke_contract_registers_projects_first() {
        assert_eq!(desktop_invoke_contract(), DESKTOP_INVOKE_COMMANDS);
        assert_eq!(desktop_invoke_contract(), &[PROJECT_CREATE_OR_SWITCH_COMMAND]);
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
        }
    }

    #[test]
    fn invoke_desktop_command_rejects_unregistered_commands() {
        let error = invoke_desktop_command(
            "writer_entry_snapshot",
            DesktopInvokeRequest::ProjectCreateOrSwitch(ProjectCreateOrSwitchRequest {
                project_id: None,
                project_name: None,
            }),
        )
        .expect_err("writer is not enabled in the first Projects-only pass");

        assert!(error
            .to_string()
            .contains("desktop invoke command is not registered yet"));
    }
}
