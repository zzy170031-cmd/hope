#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core_domain::{
    ExpandScriptResponse, ExportBundleResponse, ExportStoryboardBankResponse,
    GenerateStoryboardResponse, ListStoryboardShotResultsResponse,
    RemoveStoryboardShotResultResponse, SaveStoryboardShotResultResponse,
    UpdateStoryboardShotResultResponse,
};
use hope_app::{
    desktop_bridge::{invoke_desktop_command, DesktopInvokeRequest, DesktopInvokeResponse},
    ipc::{
        ConfigureTextModelProviderRequest, ExpandScriptRequest, ExportBundleRequest,
        ExportStoryboardBankRequest, GenerateStoryboardRequest, ListStoryboardShotResultsRequest,
        ProjectCreateOrSwitchRequest, RemoveStoryboardShotResultRequest,
        SaveStoryboardShotResultRequest, StoryboardRenderSegmentCutPreviewSnapshotRequest,
        TextModelProviderStatus, UpdateStoryboardRowsRequest, UpdateStoryboardShotResultRequest,
        ValidationExportPanelSnapshotRequest, WriterEntrySnapshotRequest,
        CONFIGURE_TEXT_MODEL_PROVIDER_COMMAND, EXPAND_SCRIPT_COMMAND, EXPORT_BUNDLE_COMMAND,
        EXPORT_STORYBOARD_BANK_COMMAND, GENERATE_STORYBOARD_COMMAND,
        GET_TEXT_MODEL_PROVIDER_STATUS_COMMAND, LIST_STORYBOARD_SHOT_RESULTS_COMMAND,
        PROJECT_CREATE_OR_SWITCH_COMMAND, REMOVE_STORYBOARD_SHOT_RESULT_COMMAND,
        SAVE_STORYBOARD_SHOT_RESULT_COMMAND, STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
        UPDATE_STORYBOARD_ROWS_COMMAND, UPDATE_STORYBOARD_SHOT_RESULT_COMMAND,
        VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND, WRITER_ENTRY_SNAPSHOT_COMMAND,
    },
    runtime::{
        ProjectCreateOrSwitchSnapshot, StoryboardRenderSegmentCutPreviewSnapshot,
        ValidationExportPanelSnapshot, WriterEntrySnapshot,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{self, Command},
};
use tauri::{Manager, WebviewWindow, WebviewWindowBuilder};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Deserialize)]
struct SelectExportSavePathRequest {
    default_file_name: String,
}

#[derive(Debug, Deserialize)]
struct CopyExportArtifactToPathRequest {
    source_path: String,
    target_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImportedStoryDocument {
    file_type: String,
    text: String,
}

const QA_CDP_PORT_ARG: &str = "--hope-qa-cdp-port=";
const QA_BASE_URL_ARG: &str = "--hope-qa-base-url=";
const QA_ENV_FILE_ARG: &str = "--hope-qa-env-file=";
const QA_LAUNCH_ID_ARG: &str = "--hope-qa-launch-id=";
const QA_MODEL_ARG: &str = "--hope-qa-model=";
const QA_MODEL_ENABLED_ARG: &str = "--hope-qa-model-enabled=";
const QA_PID_FILE_ARG: &str = "--hope-qa-pid-file=";
const QA_PROVIDER_ARG: &str = "--hope-qa-provider=";
const QA_WEBVIEW2_USER_DATA_ARG: &str = "--hope-qa-webview2-user-data-folder=";
const WEBVIEW2_DEFAULT_BROWSER_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,\
msSmartScreenProtection --noerrdialogs --disable-crash-reporter --disable-breakpad";

fn main() {
    install_shell_diagnostic_panic_hook();
    append_shell_diagnostic("process_start");
    let args = std::env::args().collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--contracts") {
        print_contract_smoke();
        return;
    }

    append_shell_diagnostic("before_configure_qa_webview2_from_args");
    configure_qa_webview2_from_args(&args);
    append_shell_diagnostic("after_configure_qa_webview2_from_args");
    run_native_host();
}

fn install_shell_diagnostic_panic_hook() {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        append_shell_diagnostic(&format!(
            "panic={}",
            sanitize_diagnostic_value(&panic_info.to_string())
        ));
        previous_hook(panic_info);
    }));
}

fn configure_qa_webview2_from_args(args: &[String]) {
    if let Some(launch_id) = args
        .iter()
        .find_map(|arg| arg.strip_prefix(QA_LAUNCH_ID_ARG))
        .filter(|value| !value.trim().is_empty())
    {
        append_shell_diagnostic(&format!("launch_id={launch_id}"));
    }

    if let Some(env_file) = args
        .iter()
        .find_map(|arg| arg.strip_prefix(QA_ENV_FILE_ARG))
        .filter(|value| !value.trim().is_empty())
    {
        apply_qa_env_file(env_file);
    }

    if let Some(provider) = args
        .iter()
        .find_map(|arg| arg.strip_prefix(QA_PROVIDER_ARG))
        .filter(|value| !value.trim().is_empty())
    {
        set_process_env_var("HOPE_TEXT_MODEL_PROVIDER", provider);
    }

    if let Some(model) = args
        .iter()
        .find_map(|arg| arg.strip_prefix(QA_MODEL_ARG))
        .filter(|value| !value.trim().is_empty())
    {
        set_process_env_var("HOPE_TEXT_MODEL_MODEL", model);
    }

    if let Some(enabled) = args
        .iter()
        .find_map(|arg| arg.strip_prefix(QA_MODEL_ENABLED_ARG))
        .and_then(parse_qa_bool)
    {
        set_process_env_var(
            "HOPE_TEXT_MODEL_ENABLED",
            if enabled { "true" } else { "false" },
        );
    }

    if let Some(base_url) = args
        .iter()
        .find_map(|arg| arg.strip_prefix(QA_BASE_URL_ARG))
        .filter(|value| !value.trim().is_empty())
    {
        set_process_env_var("HOPE_TEXT_MODEL_BASE_URL", base_url);
    }

    if let Some(port) = args
        .iter()
        .find_map(|arg| arg.strip_prefix(QA_CDP_PORT_ARG))
        .filter(|port| is_valid_tcp_port(port))
    {
        set_process_env_var(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            &format!("{WEBVIEW2_DEFAULT_BROWSER_ARGS} --remote-debugging-port={port}"),
        );
    }

    if let Some(user_data_folder) = args
        .iter()
        .find_map(|arg| arg.strip_prefix(QA_WEBVIEW2_USER_DATA_ARG))
        .filter(|value| !value.trim().is_empty())
    {
        set_process_env_var("WEBVIEW2_USER_DATA_FOLDER", user_data_folder);
    }

    if let Some(pid_file) = args
        .iter()
        .find_map(|arg| arg.strip_prefix(QA_PID_FILE_ARG))
        .filter(|value| !value.trim().is_empty())
    {
        let _ = fs::write(pid_file, process::id().to_string());
    }
}

fn apply_qa_env_file(path: &str) {
    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, raw_value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if !matches!(
            key,
            "HOPE_TEXT_MODEL_API_KEY" | "HOPE_TEXT_MODEL_BASE_URL"
        ) {
            continue;
        }
        let value = raw_value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        if !value.is_empty() {
            set_process_env_var(key, &value);
        }
    }
}

fn is_valid_tcp_port(value: &str) -> bool {
    value.parse::<u16>().is_ok_and(|port| port > 0)
}

fn parse_qa_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn set_process_env_var(key: &str, value: &str) {
    // This runs before Tauri/WebView2 starts, so setting process env is bounded to startup.
    unsafe {
        std::env::set_var(key, value);
    }
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
    append_shell_diagnostic("before_builder_run");
    let run_result = tauri::Builder::default()
        .setup(|app| {
            append_shell_diagnostic("setup_enter");
            let has_webview2_cdp_args = std::env::var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS")
                .is_ok_and(|value| value.contains("--remote-debugging-port="));
            let has_webview2_user_data_folder = std::env::var("WEBVIEW2_USER_DATA_FOLDER")
                .is_ok_and(|value| !value.trim().is_empty());
            append_shell_diagnostic(&format!(
                "webview2_remote_debugging_arg_present={has_webview2_cdp_args}"
            ));
            append_shell_diagnostic(&format!(
                "webview2_user_data_folder_present={has_webview2_user_data_folder}"
            ));

            let window = ensure_main_window(app).map_err(|error| {
                append_shell_diagnostic(&format!(
                    "main_window_error={}",
                    sanitize_diagnostic_value(&error.to_string())
                ));
                error
            })?;
            append_shell_diagnostic("main_window_found");
            append_shell_diagnostic("main_window=found");
            let _ = window.show();
            let _ = window.set_focus();
            append_shell_diagnostic("setup_exit");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            project_create_or_switch,
            writer_entry_snapshot,
            storyboard_rendersegment_cut_preview_snapshot,
            validation_export_panel_snapshot,
            expand_script,
            generate_storyboard,
            update_storyboard_rows,
            export_bundle,
            save_storyboard_shot_result,
            list_storyboard_shot_results,
            update_storyboard_shot_result,
            remove_storyboard_shot_result,
            export_storyboard_bank,
            configure_text_model_provider,
            get_text_model_provider_status,
            import_story_document,
            select_export_save_path,
            copy_export_artifact_to_path
        ])
        .run(tauri::generate_context!());

    if let Err(error) = run_result {
        append_shell_diagnostic(&format!(
            "builder_run_error={}",
            sanitize_diagnostic_value(&error.to_string())
        ));
        panic!("failed to run Hope desktop native host: {error}");
    }
}

fn ensure_main_window<R: tauri::Runtime>(app: &mut tauri::App<R>) -> tauri::Result<WebviewWindow<R>> {
    append_shell_diagnostic("before_main_window_get");
    if let Some(window) = app.get_webview_window("main") {
        append_shell_diagnostic("main_window_found=precreated");
        append_shell_diagnostic("main_window=precreated");
        return Ok(window);
    }

    append_shell_diagnostic("main_window_missing");
    append_shell_diagnostic("before_main_window_config_lookup");
    let Some(config) = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .cloned()
        .or_else(|| app.config().app.windows.first().cloned())
    else {
        append_shell_diagnostic("main_window_config=missing");
        return Err(tauri::Error::WindowLabelAlreadyExists("main".to_string()));
    };

    append_shell_diagnostic("before_builder_from_config");
    let mut builder = WebviewWindowBuilder::from_config(app.handle(), &config)?;
    if let Ok(args) = std::env::var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS") {
        if !args.trim().is_empty() {
            append_shell_diagnostic("main_window_additional_browser_args=applied");
            builder = builder.additional_browser_args(&args);
        }
    }
    if let Ok(data_dir) = std::env::var("WEBVIEW2_USER_DATA_FOLDER") {
        if !data_dir.trim().is_empty() {
            append_shell_diagnostic("main_window_data_directory=applied");
            builder = builder.data_directory(PathBuf::from(data_dir));
        }
    }

    append_shell_diagnostic("before_main_window_build");
    let window = match builder.build() {
        Ok(window) => window,
        Err(error) => {
            append_shell_diagnostic(&format!(
                "main_window_build_error={}",
                sanitize_diagnostic_value(&error.to_string())
            ));
            return Err(error);
        }
    };
    append_shell_diagnostic("main_window=created_in_setup");
    Ok(window)
}

fn sanitize_diagnostic_value(value: &str) -> String {
    let mut sanitized = value.replace('\r', " ").replace('\n', " ");
    for sensitive in ["api_key", "apikey", "token", "secret", "authorization"] {
        if sanitized.to_ascii_lowercase().contains(sensitive) {
            sanitized = "<redacted sensitive diagnostic>".to_string();
            break;
        }
    }
    sanitized
}

fn append_shell_diagnostic(line: &str) {
    let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("hope-shell-diagnostic.log")
    else {
        return;
    };
    let _ = writeln!(file, "[hope-shell-diagnostic] {line}");
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

#[tauri::command]
fn expand_script(request: ExpandScriptRequest) -> Result<ExpandScriptResponse, String> {
    match invoke_desktop_command(
        EXPAND_SCRIPT_COMMAND,
        DesktopInvokeRequest::ExpandScript(request),
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::ExpandScript(response) => Ok(response),
        _ => Err("desktop invoke returned an unexpected expand_script response".to_string()),
    }
}

#[tauri::command]
fn generate_storyboard(
    request: GenerateStoryboardRequest,
) -> Result<GenerateStoryboardResponse, String> {
    match invoke_desktop_command(
        GENERATE_STORYBOARD_COMMAND,
        DesktopInvokeRequest::GenerateStoryboard(request),
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::GenerateStoryboard(response) => Ok(response),
        _ => Err("desktop invoke returned an unexpected generate_storyboard response".to_string()),
    }
}

#[tauri::command]
fn update_storyboard_rows(
    request: UpdateStoryboardRowsRequest,
) -> Result<GenerateStoryboardResponse, String> {
    match invoke_desktop_command(
        UPDATE_STORYBOARD_ROWS_COMMAND,
        DesktopInvokeRequest::UpdateStoryboardRows(request),
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::UpdateStoryboardRows(response) => Ok(response),
        _ => {
            Err("desktop invoke returned an unexpected update_storyboard_rows response".to_string())
        }
    }
}

#[tauri::command]
fn export_bundle(request: ExportBundleRequest) -> Result<ExportBundleResponse, String> {
    match invoke_desktop_command(
        EXPORT_BUNDLE_COMMAND,
        DesktopInvokeRequest::ExportBundle(request),
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::ExportBundle(response) => Ok(response),
        _ => Err("desktop invoke returned an unexpected export_bundle response".to_string()),
    }
}

#[tauri::command]
fn save_storyboard_shot_result(
    request: SaveStoryboardShotResultRequest,
) -> Result<SaveStoryboardShotResultResponse, String> {
    match invoke_desktop_command(
        SAVE_STORYBOARD_SHOT_RESULT_COMMAND,
        DesktopInvokeRequest::SaveStoryboardShotResult(request),
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::SaveStoryboardShotResult(response) => Ok(response),
        _ => Err(
            "desktop invoke returned an unexpected save_storyboard_shot_result response"
                .to_string(),
        ),
    }
}

#[tauri::command]
fn list_storyboard_shot_results(
    request: ListStoryboardShotResultsRequest,
) -> Result<ListStoryboardShotResultsResponse, String> {
    match invoke_desktop_command(
        LIST_STORYBOARD_SHOT_RESULTS_COMMAND,
        DesktopInvokeRequest::ListStoryboardShotResults(request),
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::ListStoryboardShotResults(response) => Ok(response),
        _ => Err(
            "desktop invoke returned an unexpected list_storyboard_shot_results response"
                .to_string(),
        ),
    }
}

#[tauri::command]
fn update_storyboard_shot_result(
    request: UpdateStoryboardShotResultRequest,
) -> Result<UpdateStoryboardShotResultResponse, String> {
    match invoke_desktop_command(
        UPDATE_STORYBOARD_SHOT_RESULT_COMMAND,
        DesktopInvokeRequest::UpdateStoryboardShotResult(request),
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::UpdateStoryboardShotResult(response) => Ok(response),
        _ => Err(
            "desktop invoke returned an unexpected update_storyboard_shot_result response"
                .to_string(),
        ),
    }
}

#[tauri::command]
fn remove_storyboard_shot_result(
    request: RemoveStoryboardShotResultRequest,
) -> Result<RemoveStoryboardShotResultResponse, String> {
    match invoke_desktop_command(
        REMOVE_STORYBOARD_SHOT_RESULT_COMMAND,
        DesktopInvokeRequest::RemoveStoryboardShotResult(request),
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::RemoveStoryboardShotResult(response) => Ok(response),
        _ => Err(
            "desktop invoke returned an unexpected remove_storyboard_shot_result response"
                .to_string(),
        ),
    }
}

#[tauri::command]
fn export_storyboard_bank(
    request: ExportStoryboardBankRequest,
) -> Result<ExportStoryboardBankResponse, String> {
    match invoke_desktop_command(
        EXPORT_STORYBOARD_BANK_COMMAND,
        DesktopInvokeRequest::ExportStoryboardBank(request),
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::ExportStoryboardBank(response) => Ok(response),
        _ => {
            Err("desktop invoke returned an unexpected export_storyboard_bank response".to_string())
        }
    }
}

#[tauri::command]
fn configure_text_model_provider(
    request: ConfigureTextModelProviderRequest,
) -> Result<TextModelProviderStatus, String> {
    match invoke_desktop_command(
        CONFIGURE_TEXT_MODEL_PROVIDER_COMMAND,
        DesktopInvokeRequest::ConfigureTextModelProvider(request),
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::ConfigureTextModelProvider(response) => Ok(response),
        _ => Err("desktop invoke returned an unexpected model provider response".to_string()),
    }
}

#[tauri::command]
fn get_text_model_provider_status() -> Result<TextModelProviderStatus, String> {
    match invoke_desktop_command(
        GET_TEXT_MODEL_PROVIDER_STATUS_COMMAND,
        DesktopInvokeRequest::GetTextModelProviderStatus,
    )
    .map_err(|error| error.to_string())?
    {
        DesktopInvokeResponse::GetTextModelProviderStatus(response) => Ok(response),
        _ => {
            Err("desktop invoke returned an unexpected model provider status response".to_string())
        }
    }
}

#[tauri::command]
fn import_story_document() -> Result<Option<ImportedStoryDocument>, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding $false
$OutputEncoding = New-Object System.Text.UTF8Encoding $false
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.IO.Compression.FileSystem
$dialog = New-Object System.Windows.Forms.OpenFileDialog
$dialog.Title = '选择故事文档'
$dialog.Filter = '故事文档 (*.docx;*.txt)|*.docx;*.txt'
$dialog.Multiselect = $false
$result = $dialog.ShowDialog()
if ($result -ne [System.Windows.Forms.DialogResult]::OK) {
  exit 2
}
$ext = [System.IO.Path]::GetExtension($dialog.FileName).ToLowerInvariant()
if ($ext -eq '.txt') {
  try {
    $text = [System.IO.File]::ReadAllText($dialog.FileName, [System.Text.Encoding]::UTF8)
  } catch {
    $text = [System.IO.File]::ReadAllText($dialog.FileName, [System.Text.Encoding]::Default)
  }
} elseif ($ext -eq '.docx') {
  $zip = [System.IO.Compression.ZipFile]::OpenRead($dialog.FileName)
  try {
    $entry = $zip.GetEntry('word/document.xml')
    if ($null -eq $entry) { throw 'document xml missing' }
    $reader = New-Object System.IO.StreamReader($entry.Open())
    try { [xml]$xml = $reader.ReadToEnd() } finally { $reader.Close() }
    $ns = New-Object System.Xml.XmlNamespaceManager($xml.NameTable)
    $ns.AddNamespace('w', 'http://schemas.openxmlformats.org/wordprocessingml/2006/main')
    $paragraphs = New-Object System.Collections.Generic.List[string]
    foreach ($paragraph in $xml.SelectNodes('//w:p', $ns)) {
      $parts = New-Object System.Collections.Generic.List[string]
      foreach ($node in $paragraph.SelectNodes('.//w:t', $ns)) {
        if ($node.InnerText) { [void]$parts.Add($node.InnerText) }
      }
      $line = [string]::Join('', $parts).Trim()
      if ($line.Length -gt 0) { [void]$paragraphs.Add($line) }
    }
    $text = [string]::Join("`n", $paragraphs)
  } finally {
    $zip.Dispose()
  }
} else {
  throw 'unsupported extension'
}
$payload = [pscustomobject]@{
  file_type = $ext.TrimStart('.')
  text = $text
}
$payload | ConvertTo-Json -Compress
"#;

    let mut command = Command::new("powershell.exe");
    command
        .arg("-NoProfile")
        .arg("-STA")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(script);
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = command
        .output()
        .map_err(|_| "文档导入窗口未能打开，请稍后重试。".to_string())?;
    if output.status.code() == Some(2) {
        return Ok(None);
    }
    if !output.status.success() {
        return Err("文档读取失败，请确认文件为 docx 或 txt 后重试。".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        return Ok(None);
    }
    let document = serde_json::from_str::<ImportedStoryDocument>(&stdout)
        .map_err(|_| "文档正文解析失败，请换用 txt 或标准 docx 后重试。".to_string())?;
    if document.text.trim().is_empty() {
        return Err("文档正文为空，请选择包含正文的 docx 或 txt。".to_string());
    }
    Ok(Some(document))
}

#[tauri::command]
fn select_export_save_path(request: SelectExportSavePathRequest) -> Result<Option<String>, String> {
    let default_file_name = sanitize_default_excel_file_name(&request.default_file_name);
    let script = r#"
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding $false
Add-Type -AssemblyName System.Windows.Forms
$dialog = New-Object System.Windows.Forms.SaveFileDialog
$dialog.Title = '选择导出保存位置'
$dialog.Filter = 'Excel 工作簿 (*.xlsx)|*.xlsx'
$dialog.DefaultExt = 'xlsx'
$dialog.AddExtension = $true
$dialog.OverwritePrompt = $true
$dialog.FileName = $env:HOPE_EXPORT_DEFAULT_FILE_NAME
$result = $dialog.ShowDialog()
if ($result -eq [System.Windows.Forms.DialogResult]::OK) {
  Write-Output $dialog.FileName
}
"#;

    let mut command = Command::new("powershell.exe");
    command
        .arg("-NoProfile")
        .arg("-STA")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(script)
        .env("HOPE_EXPORT_DEFAULT_FILE_NAME", default_file_name);
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = command
        .output()
        .map_err(|_| "保存路径选择框未能打开，请稍后重试。".to_string())?;
    if !output.status.success() {
        return Err("保存路径选择框未能打开，请稍后重试。".to_string());
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        Ok(None)
    } else {
        Ok(Some(path))
    }
}

#[tauri::command]
fn copy_export_artifact_to_path(
    request: CopyExportArtifactToPathRequest,
) -> Result<String, String> {
    let source = PathBuf::from(request.source_path);
    let target = ensure_xlsx_extension(PathBuf::from(request.target_path));

    if !source.is_file() {
        return Err("未找到可复制的 Excel 文件，请重新导出。".to_string());
    }
    let Some(parent) = target.parent() else {
        return Err("保存路径不可用，请重新选择保存位置。".to_string());
    };
    if !parent.exists() {
        return Err("保存目录不存在，请重新选择保存位置。".to_string());
    }
    if source == target {
        return Ok(target.display().to_string());
    }

    fs::copy(&source, &target).map_err(|error| format!("复制 Excel 到选择路径失败：{error}"))?;
    Ok(target.display().to_string())
}

fn sanitize_default_excel_file_name(value: &str) -> String {
    let mut name = value
        .trim()
        .chars()
        .map(|character| match character {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect::<String>();
    if name.is_empty() {
        name = "Hope_导出.xlsx".to_string();
    }
    if !name.to_lowercase().ends_with(".xlsx") {
        name.push_str(".xlsx");
    }
    name
}

fn ensure_xlsx_extension(mut path: PathBuf) -> PathBuf {
    let has_xlsx_extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("xlsx"));
    if !has_xlsx_extension {
        path.set_extension("xlsx");
    }
    path
}
