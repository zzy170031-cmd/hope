#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use hope_app::{
    desktop_bridge::{DesktopInvokeRequest, DesktopInvokeResponse, invoke_desktop_command},
    ipc::{
        PROJECT_CREATE_OR_SWITCH_COMMAND, ProjectCreateOrSwitchRequest,
        STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
        StoryboardRenderSegmentCutPreviewSnapshotRequest, VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND,
        ValidationExportPanelSnapshotRequest, WRITER_ENTRY_SNAPSHOT_COMMAND,
        WriterEntrySnapshotRequest,
    },
    runtime::{
        ProjectCreateOrSwitchSnapshot, StoryboardRenderSegmentCutPreviewSnapshot,
        ValidationExportPanelSnapshot, WriterEntrySnapshot,
    },
};

fn main() {
    if std::env::args().any(|arg| arg == "--contracts") {
        print_contract_smoke();
        return;
    }

    run_native_host();
}

fn print_contract_smoke() {
    let commands = hope_app::ipc_contract();
    let desktop_commands = hope_app::desktop_invoke_contract();
    println!(
        "Hope app shell native host is ready. IPC contract: {:?}. Desktop invoke contract: {:?}",
        commands, desktop_commands
    );
}

fn run_native_host() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            project_create_or_switch,
            writer_entry_snapshot,
            storyboard_rendersegment_cut_preview_snapshot,
            validation_export_panel_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Hope desktop native host");
}

#[tauri::command]
fn project_create_or_switch(
    request: ProjectCreateOrSwitchRequest,
) -> Result<ProjectCreateOrSwitchSnapshot, String> {
    match invoke_desktop_command(
        PROJECT_CREATE_OR_SWITCH_COMMAND,
        DesktopInvokeRequest::ProjectCreateOrSwitch(request),
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::ProjectCreateOrSwitch(snapshot) => Ok(snapshot),
        _ => Err("desktop invoke returned an unexpected project response".to_string()),
    }
}

#[tauri::command]
fn writer_entry_snapshot(
    request: WriterEntrySnapshotRequest,
) -> Result<WriterEntrySnapshot, String> {
    match invoke_desktop_command(
        WRITER_ENTRY_SNAPSHOT_COMMAND,
        DesktopInvokeRequest::WriterEntrySnapshot(request),
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::WriterEntrySnapshot(snapshot) => Ok(snapshot),
        _ => Err("desktop invoke returned an unexpected writer response".to_string()),
    }
}

#[tauri::command]
fn storyboard_rendersegment_cut_preview_snapshot(
    request: StoryboardRenderSegmentCutPreviewSnapshotRequest,
) -> Result<StoryboardRenderSegmentCutPreviewSnapshot, String> {
    match invoke_desktop_command(
        STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
        DesktopInvokeRequest::StoryboardRenderSegmentCutPreviewSnapshot(request),
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::StoryboardRenderSegmentCutPreviewSnapshot(snapshot) => Ok(snapshot),
        _ => Err("desktop invoke returned an unexpected preview response".to_string()),
    }
}

#[tauri::command]
fn validation_export_panel_snapshot(
    request: ValidationExportPanelSnapshotRequest,
) -> Result<ValidationExportPanelSnapshot, String> {
    match invoke_desktop_command(
        VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND,
        DesktopInvokeRequest::ValidationExportPanelSnapshot(request),
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::ValidationExportPanelSnapshot(snapshot) => Ok(snapshot),
        _ => Err("desktop invoke returned an unexpected validation/export response".to_string()),
    }
}
