use std::{io, sync::OnceLock};

use crate::{
    ipc::{
        EXPAND_SCRIPT_COMMAND, EXPORT_BUNDLE_COMMAND, ExpandScriptRequest, ExportBundleRequest,
        GENERATE_STORYBOARD_COMMAND, GenerateStoryboardRequest, PROJECT_CREATE_OR_SWITCH_COMMAND,
        ProjectCreateOrSwitchRequest, STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
        UPDATE_STORYBOARD_ROWS_COMMAND, UpdateStoryboardRowsRequest,
        StoryboardRenderSegmentCutPreviewSnapshotRequest, VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND,
        ValidationExportPanelSnapshotRequest, WRITER_ENTRY_SNAPSHOT_COMMAND,
        WriterEntrySnapshotRequest,
    },
    runtime::{
        ProjectCreateOrSwitchSnapshot, StoryboardRenderSegmentCutPreviewSnapshot,
        ValidationExportPanelSnapshot, WriterEntrySnapshot,
        build_project_create_or_switch_snapshot,
        build_storyboard_rendersegment_cut_preview_snapshot,
        build_validation_export_panel_snapshot, build_writer_entry_snapshot, expand_script,
        export_bundle, generate_storyboard, save_storyboard_rows,
    },
    state::AppState,
};

pub const DESKTOP_INVOKE_COMMANDS: &[&str] = &[
    PROJECT_CREATE_OR_SWITCH_COMMAND,
    WRITER_ENTRY_SNAPSHOT_COMMAND,
    STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
    VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND,
    EXPAND_SCRIPT_COMMAND,
    GENERATE_STORYBOARD_COMMAND,
    UPDATE_STORYBOARD_ROWS_COMMAND,
    EXPORT_BUNDLE_COMMAND,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopInvokeRequest {
    ProjectCreateOrSwitch(ProjectCreateOrSwitchRequest),
    WriterEntrySnapshot(WriterEntrySnapshotRequest),
    StoryboardRenderSegmentCutPreviewSnapshot(StoryboardRenderSegmentCutPreviewSnapshotRequest),
    ValidationExportPanelSnapshot(ValidationExportPanelSnapshotRequest),
    ExpandScript(ExpandScriptRequest),
    GenerateStoryboard(GenerateStoryboardRequest),
    UpdateStoryboardRows(UpdateStoryboardRowsRequest),
    ExportBundle(ExportBundleRequest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopInvokeResponse {
    ProjectCreateOrSwitch(ProjectCreateOrSwitchSnapshot),
    WriterEntrySnapshot(WriterEntrySnapshot),
    StoryboardRenderSegmentCutPreviewSnapshot(StoryboardRenderSegmentCutPreviewSnapshot),
    ValidationExportPanelSnapshot(ValidationExportPanelSnapshot),
    ExpandScript(core_domain::ExpandScriptResponse),
    GenerateStoryboard(core_domain::GenerateStoryboardResponse),
    UpdateStoryboardRows(core_domain::GenerateStoryboardResponse),
    ExportBundle(core_domain::ExportBundleResponse),
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
                write!(
                    f,
                    "desktop invoke command is not registered yet: {}",
                    command
                )
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

fn command_accepts_request(command: &str, request: &DesktopInvokeRequest) -> bool {
    matches!(
        (command, request),
        (
            PROJECT_CREATE_OR_SWITCH_COMMAND,
            DesktopInvokeRequest::ProjectCreateOrSwitch(_)
        ) | (
            WRITER_ENTRY_SNAPSHOT_COMMAND,
            DesktopInvokeRequest::WriterEntrySnapshot(_)
        ) | (
            STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
            DesktopInvokeRequest::StoryboardRenderSegmentCutPreviewSnapshot(_)
        ) | (
            VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND,
            DesktopInvokeRequest::ValidationExportPanelSnapshot(_)
        ) | (EXPAND_SCRIPT_COMMAND, DesktopInvokeRequest::ExpandScript(_))
            | (
                GENERATE_STORYBOARD_COMMAND,
                DesktopInvokeRequest::GenerateStoryboard(_)
            )
            | (
                UPDATE_STORYBOARD_ROWS_COMMAND,
                DesktopInvokeRequest::UpdateStoryboardRows(_)
            )
            | (EXPORT_BUNDLE_COMMAND, DesktopInvokeRequest::ExportBundle(_))
    )
}

fn invoke_desktop_command_with_state(
    state: &AppState,
    command: &str,
    request: DesktopInvokeRequest,
) -> Result<DesktopInvokeResponse, DesktopInvokeError> {
    match (command, request) {
        (
            PROJECT_CREATE_OR_SWITCH_COMMAND,
            DesktopInvokeRequest::ProjectCreateOrSwitch(request),
        ) => {
            let snapshot = build_project_create_or_switch_snapshot(state, request)?;
            Ok(DesktopInvokeResponse::ProjectCreateOrSwitch(snapshot))
        }
        (WRITER_ENTRY_SNAPSHOT_COMMAND, DesktopInvokeRequest::WriterEntrySnapshot(request)) => {
            let snapshot = build_writer_entry_snapshot(state, request)?;
            Ok(DesktopInvokeResponse::WriterEntrySnapshot(snapshot))
        }
        (
            STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
            DesktopInvokeRequest::StoryboardRenderSegmentCutPreviewSnapshot(request),
        ) => {
            let snapshot = build_storyboard_rendersegment_cut_preview_snapshot(state, request)?;
            Ok(DesktopInvokeResponse::StoryboardRenderSegmentCutPreviewSnapshot(snapshot))
        }
        (
            VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND,
            DesktopInvokeRequest::ValidationExportPanelSnapshot(request),
        ) => {
            let snapshot = build_validation_export_panel_snapshot(state, request)?;
            Ok(DesktopInvokeResponse::ValidationExportPanelSnapshot(
                snapshot,
            ))
        }
        (EXPAND_SCRIPT_COMMAND, DesktopInvokeRequest::ExpandScript(request)) => Ok(
            DesktopInvokeResponse::ExpandScript(expand_script(state, request)),
        ),
        (GENERATE_STORYBOARD_COMMAND, DesktopInvokeRequest::GenerateStoryboard(request)) => Ok(
            DesktopInvokeResponse::GenerateStoryboard(generate_storyboard(state, request)),
        ),
        (UPDATE_STORYBOARD_ROWS_COMMAND, DesktopInvokeRequest::UpdateStoryboardRows(request)) => {
            Ok(DesktopInvokeResponse::UpdateStoryboardRows(
                save_storyboard_rows(state, request),
            ))
        }
        (EXPORT_BUNDLE_COMMAND, DesktopInvokeRequest::ExportBundle(request)) => Ok(
            DesktopInvokeResponse::ExportBundle(export_bundle(state, request)),
        ),
        _ => Err(DesktopInvokeError::UnsupportedCommand {
            command: command.to_string(),
        }),
    }
}

static DESKTOP_RUNTIME_STATE: OnceLock<Result<AppState, String>> = OnceLock::new();

fn desktop_runtime_state() -> Result<&'static AppState, DesktopInvokeError> {
    let state = DESKTOP_RUNTIME_STATE
        .get_or_init(|| AppState::load_desktop_runtime().map_err(|error| error.to_string()));

    match state {
        Ok(state) => Ok(state),
        Err(message) => Err(DesktopInvokeError::Io(io::Error::new(
            io::ErrorKind::Other,
            message.clone(),
        ))),
    }
}

pub fn invoke_desktop_command(
    command: &str,
    request: DesktopInvokeRequest,
) -> Result<DesktopInvokeResponse, DesktopInvokeError> {
    if !command_accepts_request(command, &request) {
        return Err(DesktopInvokeError::UnsupportedCommand {
            command: command.to_string(),
        });
    }

    invoke_desktop_command_with_state(desktop_runtime_state()?, command, request)
}

#[cfg(test)]
mod tests {
    use core_domain::{
        ExpandScriptRequest, ExportBundleRequest, FailurePatternRecord, GenerateStoryboardRequest,
        KbRuntimeSummary, KbSnapshotRecord, ModelConfigSummary, PromptTemplateRecord,
        SceneTaxonomyRecord, UpdateStoryboardRowsRequest,
    };
    use project_store::{
        DualSqliteConnectionPolicy, KbKnowledgeBundle, KbRuntimeHandle, StoreSkeleton,
    };

    use crate::ipc::{
        EXPAND_SCRIPT_COMMAND, EXPORT_BUNDLE_COMMAND, GENERATE_STORYBOARD_COMMAND,
        ProjectCreateOrSwitchRequest, StoryboardRenderSegmentCutPreviewSnapshotRequest,
        UPDATE_STORYBOARD_ROWS_COMMAND, ValidationExportPanelSnapshotRequest,
        WriterEntrySnapshotRequest,
    };

    use super::{
        DESKTOP_INVOKE_COMMANDS, DesktopInvokeRequest, DesktopInvokeResponse,
        PROJECT_CREATE_OR_SWITCH_COMMAND, STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
        VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND, WRITER_ENTRY_SNAPSHOT_COMMAND,
        desktop_invoke_contract, invoke_desktop_command, invoke_desktop_command_with_state,
    };
    use crate::{runtime::ValidationExportPanelState, state::AppState};

    #[test]
    fn desktop_invoke_contract_registers_phase1_commands_only() {
        assert_eq!(desktop_invoke_contract(), DESKTOP_INVOKE_COMMANDS);
        assert_eq!(
            desktop_invoke_contract(),
            &[
                PROJECT_CREATE_OR_SWITCH_COMMAND,
                WRITER_ENTRY_SNAPSHOT_COMMAND,
                STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
                VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND,
                EXPAND_SCRIPT_COMMAND,
                GENERATE_STORYBOARD_COMMAND,
                UPDATE_STORYBOARD_ROWS_COMMAND,
                EXPORT_BUNDLE_COMMAND,
            ]
        );
    }

    #[test]
    fn invoke_desktop_command_rejects_mismatched_request_before_loading_state() {
        let error = invoke_desktop_command(
            VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND,
            DesktopInvokeRequest::ProjectCreateOrSwitch(ProjectCreateOrSwitchRequest {
                project_id: None,
                project_name: None,
            }),
        )
        .expect_err("mismatched request variant should be rejected before state load");

        assert!(
            error
                .to_string()
                .contains("desktop invoke command is not registered yet")
        );
    }

    #[test]
    fn invoke_desktop_command_returns_real_project_snapshot() {
        let response = invoke_desktop_command_with_state(
            &test_state(),
            PROJECT_CREATE_OR_SWITCH_COMMAND,
            DesktopInvokeRequest::ProjectCreateOrSwitch(ProjectCreateOrSwitchRequest {
                project_id: None,
                project_name: None,
            }),
        )
        .expect("project desktop invoke should succeed");

        match response {
            DesktopInvokeResponse::ProjectCreateOrSwitch(snapshot) => {
                assert_eq!(
                    snapshot.current_project_id.as_deref(),
                    Some("project-week3-001")
                );
                assert!(!snapshot.projects.is_empty());
            }
            _ => panic!("project invoke should return the project snapshot variant"),
        }
    }

    #[test]
    fn invoke_desktop_command_returns_real_writer_snapshot() {
        let response = invoke_desktop_command_with_state(
            &test_state(),
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
        let response = invoke_desktop_command_with_state(
            &test_state(),
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
    fn invoke_desktop_command_returns_real_validation_export_snapshot() {
        let response = invoke_desktop_command_with_state(
            &test_state(),
            VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND,
            DesktopInvokeRequest::ValidationExportPanelSnapshot(
                ValidationExportPanelSnapshotRequest {
                    project_id: "project-week3-001".to_string(),
                },
            ),
        )
        .expect("validation/export desktop invoke should succeed");

        match response {
            DesktopInvokeResponse::ValidationExportPanelSnapshot(snapshot) => {
                assert_eq!(snapshot.project_id, "project-week3-001");
                assert!(!snapshot.summary_items.is_empty());
                assert!(
                    snapshot
                        .summary_items
                        .iter()
                        .any(|item| item.label == "repair_recommendations")
                );
                assert_eq!(snapshot.repair_recommendations.len(), 0);
                assert!(snapshot.summary_items.iter().any(|item| {
                    item.label == "repair_recommendations"
                        && item.state == ValidationExportPanelState::Pending
                }));
            }
            _ => panic!("export invoke should return the validation/export snapshot variant"),
        }
    }

    #[test]
    fn v120_bridge_commands_smoke_against_desktop_runtime() {
        let state = AppState::load_desktop_runtime()
            .expect("desktop runtime with V120 package should load");

        let expand = invoke_desktop_command_with_state(
            &state,
            EXPAND_SCRIPT_COMMAND,
            DesktopInvokeRequest::ExpandScript(ExpandScriptRequest {
                scene_type: "slg_sandbox_view".to_string(),
                scene_label: Some("沙盘战略视口".to_string()),
                scene_category: Some("三国 / 国战 / SLG".to_string()),
                model_config_summary: Some(ModelConfigSummary {
                    provider: "qwen".to_string(),
                    model: "qwen-plus".to_string(),
                    enabled: true,
                    base_url_present: true,
                    api_key_present: true,
                }),
                synopsis_text: "主公在沙盘上观察敌军行军轨迹，调度两翼完成合围。".to_string(),
            }),
        )
        .expect("expand_script bridge command should succeed");

        let (script_id, expanded_script_text) = match expand {
            DesktopInvokeResponse::ExpandScript(response) => {
                assert!(response.script_id.starts_with("script-"));
                assert!(response.expanded_script_text.contains("source_package"));
                assert!(
                    response
                        .expanded_script_text
                        .contains("scene_type: slg_sandbox_view")
                );
                assert!(
                    response
                        .expanded_script_text
                        .contains("scene_label: 沙盘战略视口")
                );
                assert!(
                    response
                        .expanded_script_text
                        .contains("model_provider: qwen")
                );
                assert!(response.expanded_script_text.contains("model: qwen-plus"));
                assert!(
                    response
                        .expanded_script_text
                        .contains("api_key_present: true")
                );
                assert!(!response.expanded_script_text.contains("sk-"));
                assert!(!response.kb_router_result.selected_sample_ids.is_empty());
                assert!(!response.kb_router_result.selected_kb_rules.is_empty());
                assert_eq!(
                    response
                        .kb_router_result
                        .retrieval_trace
                        .token_budget
                        .full_kb_rows_included,
                    0
                );
                println!(
                    "expand_script smoke: script_id={} warnings={} kb_samples={} kb_rules={}",
                    response.script_id,
                    response.warnings.len(),
                    response.kb_router_result.selected_sample_ids.len(),
                    response.kb_router_result.selected_kb_rules.len()
                );
                (response.script_id, response.expanded_script_text)
            }
            _ => panic!("expand_script should return expand response"),
        };

        let storyboard = invoke_desktop_command_with_state(
            &state,
            GENERATE_STORYBOARD_COMMAND,
            DesktopInvokeRequest::GenerateStoryboard(GenerateStoryboardRequest {
                task_name: "desktop bridge smoke".to_string(),
                script_id: Some(script_id),
                expanded_script_text: Some(expanded_script_text),
                selected_total_duration_seconds: 30,
                model_config_summary: Some(ModelConfigSummary {
                    provider: "doubao".to_string(),
                    model: "doubao-seed-reserved".to_string(),
                    enabled: true,
                    base_url_present: true,
                    api_key_present: false,
                }),
            }),
        )
        .expect("generate_storyboard bridge command should succeed");

        let (result_id, task_id, mut rows, base_revision) = match storyboard {
            DesktopInvokeResponse::GenerateStoryboard(response) => {
                assert!(!response.rows.is_empty());
                assert!(response.rows.iter().all(|row| !row.prompt_text.is_empty()));
                assert!(
                    response
                        .export_status
                        .warnings
                        .iter()
                        .any(|warning| warning.code == "model_provider_reserved")
                );
                assert!((3..=5).contains(&response.kb_router_result.selected_sample_ids.len()));
                assert!(!response.kb_router_result.selected_kb_rules.is_empty());
                assert_eq!(
                    response
                        .kb_router_result
                        .retrieval_trace
                        .token_budget
                        .full_kb_rows_included,
                    0
                );
                println!(
                    "generate_storyboard smoke: result_id={} rows={} status={:?} kb_samples={} kb_rules={}",
                    response.result_id,
                    response.rows.len(),
                    response.export_status.status,
                    response.kb_router_result.selected_sample_ids.len(),
                    response.kb_router_result.selected_kb_rules.len()
                );
                (
                    response.result_id,
                    response.task_id,
                    response.rows,
                    response.revision,
                )
            }
            _ => panic!("generate_storyboard should return storyboard response"),
        };

        rows[0].prompt_text = "desktop smoke edited prompt_text".to_string();
        let updated = invoke_desktop_command_with_state(
            &state,
            UPDATE_STORYBOARD_ROWS_COMMAND,
            DesktopInvokeRequest::UpdateStoryboardRows(UpdateStoryboardRowsRequest {
                result_id: result_id.clone(),
                task_id: task_id.clone(),
                rows: rows.clone(),
                operation_id: "desktop-smoke-update-rows".to_string(),
                base_revision,
                dirty_source_note: Some("desktop_smoke_row_edit".to_string()),
            }),
        )
        .expect("update_storyboard_rows bridge command should succeed");

        match updated {
            DesktopInvokeResponse::UpdateStoryboardRows(response) => {
                assert_eq!(response.result_id, result_id);
                assert!(response.dirty);
                assert!(response.revision > base_revision);
                assert_eq!(response.rows[0].prompt_text, "desktop smoke edited prompt_text");
                println!(
                    "update_storyboard_rows smoke: result_id={} revision={} rows_hash={}",
                    response.result_id, response.revision, response.rows_hash
                );
            }
            _ => panic!("update_storyboard_rows should return storyboard response"),
        }

        let export = invoke_desktop_command_with_state(
            &state,
            EXPORT_BUNDLE_COMMAND,
            DesktopInvokeRequest::ExportBundle(ExportBundleRequest {
                result_id: Some(result_id.clone()),
                task_id,
                export_format: "storyboard_words".to_string(),
            }),
        )
        .expect("export_bundle bridge command should succeed");

        match export {
            DesktopInvokeResponse::ExportBundle(response) => {
                let ready_artifact = response
                    .artifacts
                    .iter()
                    .find(|artifact| artifact.ready && artifact.artifact_path.is_some())
                    .expect("export_bundle should return at least one ready file artifact");
                println!(
                    "export_bundle smoke: manifest={} artifact_kind={} ready={} artifact_path={} content_hash={} row_count={}",
                    response.export_manifest_id,
                    ready_artifact.artifact_kind,
                    ready_artifact.ready,
                    ready_artifact.artifact_path.as_deref().unwrap_or_default(),
                    ready_artifact.content_hash.as_deref().unwrap_or_default(),
                    ready_artifact.row_count.unwrap_or_default()
                );
                assert_eq!(ready_artifact.source_result_id.as_deref(), Some(result_id.as_str()));
                assert_eq!(ready_artifact.edited_rows_applied, true);
                assert!(
                    ready_artifact
                        .prompt_text_compilation_statuses
                        .iter()
                        .any(|status| status == "ReadyStub")
                );
                assert!(!ready_artifact.selected_sample_ids.is_empty());
                assert!(!ready_artifact.selected_kb_rule_ids.is_empty());
                assert_eq!(ready_artifact.full_kb_rows_included, 0);
                assert!(ready_artifact.kb_context_summary.is_some());
                assert!(ready_artifact.retrieval_trace.is_some());
            }
            _ => panic!("export_bundle should return export response"),
        }
    }

    fn test_state() -> AppState {
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            "E:/codex/hope-kb/snapshots/hope-kb-v0.1.sqlite3".into(),
            "E:/codex/hope/data/hope.sqlite3".into(),
        ));
        let kb_runtime = KbRuntimeHandle {
            snapshot: KbSnapshotRecord {
                snapshot_id: "hope-kb-v0.1".to_string(),
                snapshot_hash: "test-hash".to_string(),
                seed_format: "hope-kb-sqlite-snapshot-v0.1".to_string(),
                source_name: "hope-kb".to_string(),
                created_at_timestamp: 1_714_000_000,
            },
            summary: KbRuntimeSummary {
                snapshot_id: "hope-kb-v0.1".to_string(),
                snapshot_path: "E:/codex/hope-kb/snapshots/hope-kb-v0.1.sqlite3".to_string(),
                mirror_tables: vec![
                    "scene_taxonomy".to_string(),
                    "failure_pattern".to_string(),
                    "prompt_template".to_string(),
                ],
                has_scene_taxonomy: true,
                has_failure_patterns: true,
                has_repair_template_mapping: true,
                has_golden_sample_v120_package: false,
                golden_sample_record_count: 0,
                golden_sample_source_count: 0,
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
                source_type: "team_distillation".to_string(),
                source_notes: "test fixture".to_string(),
                confidence_level: "high".to_string(),
                last_reviewed_at: "2026-04-20".to_string(),
            }],
            failure_patterns: vec![FailurePatternRecord {
                failure_pattern_id: "failure-prompt-noise".to_string(),
                failure_code: "chinese_prompt_noise".to_string(),
                failure_name: "Chinese Prompt Noise".to_string(),
                failure_category: "language".to_string(),
                symptom: "Placeholder or machine-like prompt text leaks into output.".to_string(),
                common_causes: vec!["placeholder marker in prompt".to_string()],
                detection_hint: "Check prompt body placeholders.".to_string(),
                repair_strategy: "Rewrite prompt body as natural Chinese guidance.".to_string(),
                affected_layers: vec!["prompt_packages".to_string()],
                validator_hint: "Prompt quality review".to_string(),
                repair_template_ids: vec!["prompt-repair-language".to_string()],
                repair_priority: "medium".to_string(),
                repair_scope: "prompt_rendering_layer".to_string(),
                suggested_followup_validators: vec!["Prompt quality review".to_string()],
                source_type: "team_distillation".to_string(),
                source_notes: "test fixture".to_string(),
                confidence_level: "high".to_string(),
                last_reviewed_at: "2026-04-20".to_string(),
            }],
            prompt_templates: vec![PromptTemplateRecord {
                prompt_template_id: "prompt-repair-language".to_string(),
                stage: "repair_pass".to_string(),
                name: "Repair Prompt Language".to_string(),
                target_model_family: "qwen-compatible".to_string(),
                repairs_failure_codes: vec!["chinese_prompt_noise".to_string()],
            }],
        };

        AppState::new(store, kb_runtime, kb_knowledge)
    }
}
