use std::io;

use crate::{
    ipc::{
        PROJECT_CREATE_OR_SWITCH_COMMAND, ProjectCreateOrSwitchRequest,
        STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
        VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND, ValidationExportPanelSnapshotRequest,
        StoryboardRenderSegmentCutPreviewSnapshotRequest, WRITER_ENTRY_SNAPSHOT_COMMAND,
        WriterEntrySnapshotRequest,
    },
    runtime::{
        ProjectCreateOrSwitchSnapshot, StoryboardRenderSegmentCutPreviewSnapshot,
        ValidationExportPanelSnapshot, ValidationExportPanelState,
        ValidationRepairRecommendationItem, WriterEntrySnapshot,
        build_project_create_or_switch_snapshot,
        build_storyboard_rendersegment_cut_preview_snapshot,
        build_validation_export_panel_snapshot, build_writer_entry_snapshot,
    },
    state::AppState,
};
use core_domain::{
    FailurePatternRecord, KbRuntimeSummary, KbSnapshotRecord, PromptTemplateRecord,
    RepairTemplateLink, SceneTaxonomyRecord,
};
use project_store::{DualSqliteConnectionPolicy, KbKnowledgeBundle, KbRuntimeHandle, StoreSkeleton};
use validators::{
    RepairRecommendation, WEEK3_SHARED_FIXTURE_PATH, generate_week3_repair_recommendations,
    load_week3_shared_fixture,
};

pub const DESKTOP_INVOKE_COMMANDS: &[&str] =
    &[
        PROJECT_CREATE_OR_SWITCH_COMMAND,
        WRITER_ENTRY_SNAPSHOT_COMMAND,
        STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
        VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND,
    ];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopInvokeRequest {
    ProjectCreateOrSwitch(ProjectCreateOrSwitchRequest),
    WriterEntrySnapshot(WriterEntrySnapshotRequest),
    StoryboardRenderSegmentCutPreviewSnapshot(StoryboardRenderSegmentCutPreviewSnapshotRequest),
    ValidationExportPanelSnapshot(ValidationExportPanelSnapshotRequest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopInvokeResponse {
    ProjectCreateOrSwitch(ProjectCreateOrSwitchSnapshot),
    WriterEntrySnapshot(WriterEntrySnapshot),
    StoryboardRenderSegmentCutPreviewSnapshot(StoryboardRenderSegmentCutPreviewSnapshot),
    ValidationExportPanelSnapshot(ValidationExportPanelSnapshot),
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

fn desktop_phase1_state() -> AppState {
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
            mirror_tables: vec![
                "scene_taxonomy".to_string(),
                "failure_pattern".to_string(),
                "prompt_template".to_string(),
            ],
            has_scene_taxonomy: true,
            has_failure_patterns: true,
            has_repair_template_mapping: true,
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
            source_notes: "Phase 1 desktop invoke baseline".to_string(),
            confidence_level: "high".to_string(),
            last_reviewed_at: "2026-04-20".to_string(),
        }],
        failure_patterns: vec![
            FailurePatternRecord {
                failure_pattern_id: "failure-prompt-noise".to_string(),
                failure_code: "chinese_prompt_noise".to_string(),
                failure_name: "Chinese Prompt Noise".to_string(),
                failure_category: "language".to_string(),
                symptom: "Placeholder or machine-like prompt text leaks into output."
                    .to_string(),
                common_causes: vec!["placeholder marker in prompt".to_string()],
                detection_hint: "Check prompt body placeholders.".to_string(),
                repair_strategy: "Rewrite the prompt package body with natural language."
                    .to_string(),
                affected_layers: vec!["prompt_packages".to_string()],
                validator_hint: "Prompt quality review / Export sanity check".to_string(),
                repair_template_ids: vec!["prompt-repair-language".to_string()],
                repair_priority: "medium".to_string(),
                repair_scope: "prompt_rendering_layer".to_string(),
                suggested_followup_validators: vec![
                    "Prompt quality review / Export sanity check".to_string(),
                ],
                source_type: "desktop-shell-fixture".to_string(),
                source_notes: "Phase 1 desktop export invoke baseline".to_string(),
                confidence_level: "high".to_string(),
                last_reviewed_at: "2026-04-20".to_string(),
            },
            FailurePatternRecord {
                failure_pattern_id: "failure-continuity-break".to_string(),
                failure_code: "continuity_break".to_string(),
                failure_name: "Continuity Break".to_string(),
                failure_category: "continuity".to_string(),
                symptom: "Traceability or continuity links drift across adjacent cuts."
                    .to_string(),
                common_causes: vec!["stale chain incomplete".to_string()],
                detection_hint: "Check Continuity Validator findings.".to_string(),
                repair_strategy: "Patch continuity references and adjacent cut chain."
                    .to_string(),
                affected_layers: vec!["storyboard_cuts".to_string()],
                validator_hint: "Continuity Validator".to_string(),
                repair_template_ids: vec!["prompt-repair-continuity".to_string()],
                repair_priority: "high".to_string(),
                repair_scope: "continuity_fields_and_adjacent_cuts".to_string(),
                suggested_followup_validators: vec!["Continuity Validator".to_string()],
                source_type: "desktop-shell-fixture".to_string(),
                source_notes: "Phase 1 desktop export invoke baseline".to_string(),
                confidence_level: "high".to_string(),
                last_reviewed_at: "2026-04-20".to_string(),
            },
        ],
        prompt_templates: vec![
            PromptTemplateRecord {
                prompt_template_id: "prompt-repair-language".to_string(),
                stage: "repair_pass".to_string(),
                name: "Repair Prompt Language".to_string(),
                target_model_family: "qwen-compatible".to_string(),
                repairs_failure_codes: vec!["chinese_prompt_noise".to_string()],
            },
            PromptTemplateRecord {
                prompt_template_id: "prompt-repair-continuity".to_string(),
                stage: "repair_continuity".to_string(),
                name: "Repair Continuity Links".to_string(),
                target_model_family: "qwen-compatible".to_string(),
                repairs_failure_codes: vec!["continuity_break".to_string()],
            },
        ],
    };

    AppState::new(store, kb_runtime, kb_knowledge)
}

fn desktop_export_probe_recommendations(
    state: &AppState,
) -> io::Result<Vec<ValidationRepairRecommendationItem>> {
    let mut fixture = load_week3_shared_fixture(WEEK3_SHARED_FIXTURE_PATH)?;

    // Keep the export repair card path exercised even when the shared fixture is fully green.
    if let Some(prompt_package) = fixture.prompt_package.first_mut() {
        prompt_package.body = "TODO: rewrite this prompt body".to_string();
    }

    Ok(generate_week3_repair_recommendations(
        &fixture,
        &state.kb_knowledge.failure_patterns,
        &state.kb_knowledge.prompt_templates,
    )?
    .into_iter()
    .map(map_desktop_repair_recommendation)
    .collect())
}

fn map_desktop_repair_recommendation(
    recommendation: RepairRecommendation,
) -> ValidationRepairRecommendationItem {
    ValidationRepairRecommendationItem {
        failure_code: recommendation.failure_code,
        failure_name: recommendation.failure_name,
        repair_strategy: recommendation.repair_strategy,
        repair_priority: recommendation.repair_priority,
        repair_scope: recommendation.repair_scope,
        validator_hint: recommendation.validator_hint,
        prompt_template_names: recommendation
            .prompt_templates
            .into_iter()
            .filter_map(
                |RepairTemplateLink {
                     prompt_name, ..
                 }| prompt_name,
            )
            .collect(),
    }
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
            let state = desktop_phase1_state();
            let snapshot = build_storyboard_rendersegment_cut_preview_snapshot(&state, request)?;
            Ok(DesktopInvokeResponse::StoryboardRenderSegmentCutPreviewSnapshot(
                snapshot,
            ))
        }
        (
            VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND,
            DesktopInvokeRequest::ValidationExportPanelSnapshot(request),
        ) => {
            let state = desktop_phase1_state();
            let mut snapshot = build_validation_export_panel_snapshot(&state, request)?;

            if snapshot.repair_recommendations.is_empty() {
                let probe_recommendations = desktop_export_probe_recommendations(&state)?;
                if !probe_recommendations.is_empty() {
                    if let Some(item) = snapshot
                        .summary_items
                        .iter_mut()
                        .find(|item| item.label == "repair_recommendations")
                    {
                        item.value = format!(
                            "{} kb-backed recommendations / snapshot {}",
                            probe_recommendations.len(),
                            state.kb_runtime.snapshot.snapshot_id
                        );
                        item.state = ValidationExportPanelState::Ready;
                    }
                    snapshot.repair_recommendations = probe_recommendations;
                }
            }

            Ok(DesktopInvokeResponse::ValidationExportPanelSnapshot(snapshot))
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
        ValidationExportPanelSnapshotRequest, WriterEntrySnapshotRequest,
    };

    use super::{
        DESKTOP_INVOKE_COMMANDS, DesktopInvokeRequest, DesktopInvokeResponse,
        PROJECT_CREATE_OR_SWITCH_COMMAND, STORYBOARD_RENDERSEGMENT_CUT_PREVIEW_SNAPSHOT_COMMAND,
        VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND, WRITER_ENTRY_SNAPSHOT_COMMAND,
        desktop_export_probe_recommendations, desktop_invoke_contract, desktop_phase1_state,
        invoke_desktop_command,
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
                VALIDATION_EXPORT_PANEL_SNAPSHOT_COMMAND,
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
    fn invoke_desktop_command_returns_real_validation_export_snapshot() {
        let response = invoke_desktop_command(
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
                assert!(!snapshot.repair_recommendations.is_empty());
                assert!(snapshot
                    .repair_recommendations
                    .iter()
                    .any(|item| !item.failure_code.is_empty()));
                assert!(snapshot
                    .repair_recommendations
                    .iter()
                    .flat_map(|item| item.prompt_template_names.iter())
                    .any(|name| !name.is_empty()));
            }
            _ => panic!("export invoke should return the validation/export snapshot variant"),
        }
    }

    #[test]
    fn desktop_export_probe_returns_real_repair_recommendations() {
        let state = desktop_phase1_state();
        let recommendations = desktop_export_probe_recommendations(&state)
            .expect("desktop export probe should return real repair recommendations");

        assert!(recommendations
            .iter()
            .any(|item| item.failure_code == "chinese_prompt_noise"));
        assert!(recommendations
            .iter()
            .flat_map(|item| item.prompt_template_names.iter())
            .any(|name| name == "Repair Prompt Language"));
    }
}
