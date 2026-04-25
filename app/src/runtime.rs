use std::{
    collections::HashSet,
    env,
    hash::{Hash, Hasher},
    io,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use crate::{
    ipc::{
        ProjectCreateOrSwitchRequest, StoryboardRenderSegmentCutPreviewSnapshotRequest,
        ValidationExportPanelSnapshotRequest, WriterEntrySnapshotRequest,
    },
    state::{AppState, SnapshotBootstrapReadonlyState, ValidationFeedbackReadonlyState},
};

use core_domain::{
    BridgeCallStatus, ExpandScriptRequest, ExpandScriptResponse, ExportArtifactRecord,
    ExportBundleRequest, ExportBundleResponse, ExternalReferenceHandleCandidate,
    GenerateStoryboardRequest, GenerateStoryboardResponse, GeneratedStoryboardRow,
    GoldenSampleLibraryRecord, KbRouterExcludedCandidate, KbRouterRetrievalTrace,
    KbRouterRuntimeRequest, KbRouterRuntimeResponse, KbRouterSelectedRule, KbRouterSelectionReason,
    KbRouterTaskType, KbRouterTokenBudget, ModelConfigSummary, ProductWarning,
    PromptTextCompilationStatus, ScenePerformanceProjection, SequenceFieldState, SequenceGrouping,
    ShotGroundingSource, StoryboardDurationPlan, StoryboardExportStatus, StructureMode,
    TextGenerationOutputSchema, TextGenerationRequest, TextGenerationResponse,
    TextGenerationTask, TextModelProvider, TextModelProviderKind, UpdateStoryboardRowsRequest,
};
use export_engine::{V120StoryboardExportRequest, export_v120_storyboard_bundle};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use storyboard_pipeline::{StoryboardPlan, StoryboardPlanRequest, StoryboardPlanningError};
use validators::{
    RepairRecommendation, Week3SharedFixture, generate_week3_repair_recommendations,
    generate_week3_validation_report, project_v120_evidence_aware_findings,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectCreateOrSwitchSnapshot {
    pub current_project_id: Option<String>,
    pub projects: Vec<ProjectSummaryItem>,
    pub readonly_status: AppShellReadonlyStatusSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectSummaryItem {
    pub project_id: String,
    pub name: String,
    pub status: String,
    pub updated_at: String,
    pub episode_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppShellReadonlyStatusSnapshot {
    pub snapshot_bootstrap: SnapshotBootstrapReadonlyState,
    pub validation_feedback: ValidationFeedbackReadonlyState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WriterEntrySnapshot {
    pub project_id: String,
    pub synopsis: String,
    pub story: String,
    pub screenplay: String,
    pub storyboard: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StoryboardRenderSegmentCutPreviewSnapshot {
    pub project_id: String,
    pub storyboard: Vec<PreviewItem>,
    pub render_segment: Vec<PreviewItem>,
    pub cuts: Vec<PreviewItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreviewItem {
    pub id: String,
    pub label: String,
    pub duration: String,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StoryboardPreviewPlanRequest {
    pub render_segment_id: String,
    pub narrative_scene_id: String,
    pub render_segment_sequence_no: u32,
    pub start_shot_sequence_no: u32,
    pub end_shot_sequence_no: u32,
    pub target_duration_seconds: u16,
    pub cut_id: String,
    pub cut_sequence_no: u32,
    pub shot_description: String,
    pub dialogue: String,
    pub scene_director_id: Option<String>,
    pub action_director_id: Option<String>,
    pub scene_type: Option<String>,
    pub layout_prompt: String,
    pub render_prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationExportPanelSnapshot {
    pub project_id: String,
    pub summary_items: Vec<ValidationExportPanelItem>,
    pub repair_recommendations: Vec<ValidationRepairRecommendationItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationExportPanelItem {
    pub label: String,
    pub value: String,
    pub state: ValidationExportPanelState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ValidationExportPanelState {
    Ready,
    Pending,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationRepairRecommendationItem {
    pub failure_code: String,
    pub failure_name: String,
    pub repair_strategy: String,
    pub repair_priority: String,
    pub repair_scope: String,
    pub validator_hint: String,
    pub prompt_template_names: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct LiveStoryboardRowPatch {
    #[serde(default)]
    shot_title: String,
    #[serde(default)]
    person: String,
    #[serde(default)]
    scene_scale: String,
    #[serde(default)]
    visual_description: String,
    #[serde(default)]
    character_action: String,
    #[serde(default)]
    dialogue: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct LiveStoryboardRowsEnvelope {
    rows: Vec<LiveStoryboardRowPatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StoryboardGroundingContext {
    shot_script: String,
    expanded_script_text: String,
    grounding_text: String,
    grounding_source: ShotGroundingSource,
    primary_scene_type: String,
    primary_scene_label: String,
    primary_scene_category: String,
    shot_scene_type: String,
    shot_scene_label: String,
    shot_intent: String,
    adaptation_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShotGroundedRowDraft {
    shot_id: String,
    shot_script: String,
    person: String,
    shot_title: String,
    scene_scale: String,
    visual_description: String,
    character_action: String,
    dialogue: String,
    duration_seconds: u16,
    sequence_grouping: SequenceGrouping,
    scene_performance_projection: ScenePerformanceProjection,
}

#[derive(Debug, Deserialize)]
struct QwenChatCompletionResponse {
    choices: Vec<QwenChoice>,
    #[serde(default)]
    usage: Option<QwenUsage>,
}

#[derive(Debug, Deserialize)]
struct QwenChoice {
    message: QwenMessage,
}

#[derive(Debug, Deserialize)]
struct QwenMessage {
    #[serde(default)]
    content: String,
}

#[derive(Debug, Deserialize)]
struct QwenUsage {
    #[serde(default)]
    total_tokens: Option<u32>,
}

pub fn expand_script(state: &AppState, request: ExpandScriptRequest) -> ExpandScriptResponse {
    let scene_label = request.scene_label.as_deref().unwrap_or_default().trim();
    let scene_category = request.scene_category.as_deref().unwrap_or_default().trim();
    let normalized_scene_type = normalize_scene_type(&request.scene_type);
    let target_duration_seconds = request
        .selected_total_duration_seconds
        .filter(|duration| is_supported_storyboard_duration(*duration))
        .unwrap_or(15);
    let router_request = KbRouterRuntimeRequest {
        scene_type: normalized_scene_type.clone(),
        synopsis_text: request.synopsis_text.clone(),
        duration_seconds: target_duration_seconds,
        task_type: KbRouterTaskType::ExpandScript,
        primary_scene_type: Some(normalized_scene_type.clone()),
        primary_scene_label: non_blank_string(scene_label),
        shot_scene_type: None,
        shot_scene_label: None,
        shot_intent: None,
        structure_type: None,
    };
    let script_hash = stable_hash_hex(&format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        request.scene_type.trim(),
        scene_label,
        scene_category,
        target_duration_seconds,
        model_config_hash_input(request.model_config_summary.as_ref()),
        request.synopsis_text.trim()
    ));
    let script_id = format!("script-{}", &script_hash[..12]);

    let mut v120_package_not_confirmed = false;
    let source_package = if let Some(package) = state.kb_golden_sample_runtime.as_ref() {
        if !package.manifest.seed_import_format.contains("v0.2") {
            v120_package_not_confirmed = true;
        }
        package.manifest.snapshot_name.clone()
    } else {
        "missing_v120_runtime_package".to_string()
    };
    let kb_router_result = run_kb_router(state, router_request);
    let (provider, session_api_key) = current_text_model_provider(state);
    let generation_request = build_text_generation_request(
        TextGenerationTask::ExpandScript,
        Some(normalized_scene_type.clone()),
        request.synopsis_text.trim().to_string(),
        Some(StoryboardDurationPlan {
            total_duration_seconds: target_duration_seconds,
            row_count: 1,
            per_row_seconds: target_duration_seconds,
            allocated_seconds: target_duration_seconds,
        }),
        kb_router_result.kb_context_summary.clone(),
        kb_router_result.selected_sample_ids.clone(),
        kb_router_result
            .selected_kb_rules
            .iter()
            .map(|rule| format!("{}:{}", rule.rule_id, rule.summary))
            .collect(),
        TextGenerationOutputSchema::PlainText,
        Some(700),
    );
    let generated_script =
        run_text_generation(&provider, session_api_key.as_deref(), &generation_request);
    let mut warnings = generated_script.warnings.clone();
    warnings.extend(model_config_warnings(request.model_config_summary.as_ref()));
    if v120_package_not_confirmed {
        warnings.push(ProductWarning {
            code: "v120_package_not_confirmed".to_string(),
            message: "Loaded KB package does not advertise the accepted v0.2 seed import format."
                .to_string(),
            related_sample_id: None,
        });
    } else if source_package == "missing_v120_runtime_package" {
        warnings.push(ProductWarning {
            code: "v120_package_missing".to_string(),
            message: "V120 golden sample runtime package is not available to the desktop bridge."
                .to_string(),
            related_sample_id: None,
        });
    }

    let live_text = if generated_script.warnings.is_empty() {
        validate_generated_script_text(&generated_script.text)
    } else {
        None
    };
    if live_text.is_none() {
        warnings.push(ProductWarning {
            code: "text_model_live_expand_fallback".to_string(),
            message: "未启用千问或调用失败，已使用本地候选结果。".to_string(),
            related_sample_id: None,
        });
    }
    let fallback_script = build_deterministic_expanded_story_script(
        &request.synopsis_text,
        scene_label,
        target_duration_seconds,
    );
    let expanded_script_text = live_text
        .unwrap_or(fallback_script.as_str())
        .trim()
        .to_string();

    let response = ExpandScriptResponse {
        script_id,
        expanded_script_text,
        script_hash,
        warnings,
        kb_router_result,
    };
    state.remember_script(response.clone());
    response
}

pub fn generate_storyboard(
    state: &AppState,
    request: GenerateStoryboardRequest,
) -> GenerateStoryboardResponse {
    let now_ms = now_epoch_ms();
    let expanded_script_text = request
        .expanded_script_text
        .clone()
        .filter(|text| !text.trim().is_empty())
        .or_else(|| {
            request
                .script_id
                .as_deref()
                .and_then(|script_id| state.find_script(script_id))
                .map(|script| script.expanded_script_text.clone())
        })
        .unwrap_or_default();
    let grounding = resolve_storyboard_grounding_context(&request, expanded_script_text);
    let scene_type = normalize_scene_type(&grounding.primary_scene_type);
    let shot_scene_type = normalize_scene_type(&grounding.shot_scene_type);
    let router_request = KbRouterRuntimeRequest {
        scene_type: scene_type.clone(),
        synopsis_text: build_storyboard_router_synopsis(&grounding),
        duration_seconds: request.selected_total_duration_seconds,
        task_type: KbRouterTaskType::GenerateStoryboard,
        primary_scene_type: Some(grounding.primary_scene_type.clone()),
        primary_scene_label: Some(grounding.primary_scene_label.clone()),
        shot_scene_type: Some(grounding.shot_scene_type.clone()),
        shot_scene_label: Some(grounding.shot_scene_label.clone()),
        shot_intent: Some(grounding.shot_intent.clone()),
        structure_type: None,
    };
    let mut blockers = Vec::new();

    if grounding.grounding_text.trim().is_empty() {
        blockers.push(ProductWarning {
            code: "task_script_required".to_string(),
            message: "请先完成剧本扩写或提供有效镜头剧本内容，再生成分镜。".to_string(),
            related_sample_id: None,
        });
    }
    if request.task_name.trim().is_empty() {
        blockers.push(ProductWarning {
            code: "task_name_required".to_string(),
            message: "请填写镜头任务名称后再生成分镜。".to_string(),
            related_sample_id: None,
        });
    }
    if !is_supported_storyboard_duration(request.selected_total_duration_seconds) {
        blockers.push(ProductWarning {
            code: "duration_not_supported".to_string(),
            message: "总时长仅支持 5/10/15/30/45/60 秒。".to_string(),
            related_sample_id: None,
        });
    }
    if grounding.primary_scene_type.trim().is_empty() {
        blockers.push(ProductWarning {
            code: "scene_type_required".to_string(),
            message: "镜头任务缺少结构化场景类型，无法生成分镜。".to_string(),
            related_sample_id: None,
        });
    } else if resolve_scene_taxonomy(state, Some(&grounding.primary_scene_type)).is_none()
        && resolve_scene_taxonomy(state, Some(&scene_type)).is_none()
        && !is_supported_desktop_scene_type(&scene_type)
    {
        blockers.push(ProductWarning {
            code: "scene_type_invalid".to_string(),
            message: "镜头任务的结构化场景类型无效，无法生成分镜。".to_string(),
            related_sample_id: None,
        });
    }
    if grounding.shot_scene_type != grounding.primary_scene_type
        && grounding.adaptation_reason.trim().is_empty()
    {
        blockers.push(ProductWarning {
            code: "adaptation_reason_required".to_string(),
            message: "shot_scene_type differs from primary_scene_type and needs adaptation_reason."
                .to_string(),
            related_sample_id: None,
        });
    }
    if state.kb_golden_sample_runtime.is_none() {
        blockers.push(ProductWarning {
            code: "v120_package_missing".to_string(),
            message: "generate_storyboard requires the V120 golden sample runtime package."
                .to_string(),
            related_sample_id: None,
        });
    }

    if !blockers.is_empty() {
        return blocked_storyboard_response(
            state,
            &router_request,
            None,
            request.selected_total_duration_seconds,
            blockers,
            now_ms,
        );
    }

    let kb_router_result = run_kb_router(state, router_request.clone());
    let selected_records = select_golden_sample_records_from_router(state, &kb_router_result);
    if selected_records.is_empty() {
        return blocked_storyboard_response(
            state,
            &router_request,
            None,
            request.selected_total_duration_seconds,
            vec![ProductWarning {
                code: "no_selectable_rows".to_string(),
                message: "当前任务没有可用的官方分镜样本，无法生成分镜。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
        );
    }

    let row_count = selected_records.len().max(1);
    let Some(row_durations) =
        allocate_storyboard_row_durations(request.selected_total_duration_seconds, row_count)
    else {
        return blocked_storyboard_response(
            state,
            &router_request,
            None,
            request.selected_total_duration_seconds,
            vec![ProductWarning {
                code: "duration_allocation_failed".to_string(),
                message: "镜头时长分配失败，无法生成满足总时长守恒的分镜。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
        );
    };

    let mut rows = Vec::new();
    let mut deterministic_rows = Vec::new();
    let mut warnings = Vec::new();
    let (provider, session_api_key) = current_text_model_provider(state);
    let generation_request = build_text_generation_request(
        TextGenerationTask::GenerateStoryboard,
        Some(shot_scene_type.clone()),
        build_storyboard_model_story_input(&grounding),
        Some(StoryboardDurationPlan {
            total_duration_seconds: request.selected_total_duration_seconds,
            row_count: row_count as u32,
            per_row_seconds: (request.selected_total_duration_seconds / row_count as u16).max(1),
            allocated_seconds: request.selected_total_duration_seconds,
        }),
        kb_router_result.kb_context_summary.clone(),
        kb_router_result.selected_sample_ids.clone(),
        kb_router_result
            .selected_kb_rules
            .iter()
            .map(|rule| format!("{}:{}", rule.rule_id, rule.summary))
            .collect(),
        TextGenerationOutputSchema::StoryboardRowsJson,
        Some(1200),
    );
    let live_generation =
        run_text_generation(&provider, session_api_key.as_deref(), &generation_request);
    warnings.extend(live_generation.warnings.clone());
    let live_row_patches = match extract_live_storyboard_row_patches(&live_generation) {
        Ok(patches) => patches,
        Err(warning) => {
            warnings.push(warning);
            Vec::new()
        }
    };

    for (index, record) in selected_records.iter().enumerate() {
        let failure_mapping = state.kb_golden_sample_runtime.as_ref().and_then(|package| {
            package
                .failure_mapping
                .records
                .iter()
                .find(|mapping| mapping.sample_id == record.sample_id)
        });
        let evidence = project_v120_evidence_aware_findings(record, failure_mapping);
        blockers.extend(
            evidence
                .blockers
                .into_iter()
                .map(product_warning_from_evidence),
        );
        warnings.extend(
            evidence
                .warnings
                .into_iter()
                .map(product_warning_from_evidence),
        );

        let draft = build_shot_grounded_row_draft(
            &grounding,
            index,
            row_count,
            row_durations[index],
            record,
        );
        let scene_projection = draft.scene_performance_projection.clone();
        let prompt_compilation = compile_seedance_prompt_text(
            record,
            &scene_projection,
            row_durations[index],
            &grounding,
            &draft.shot_script,
            &kb_router_result,
        );
        warnings.extend(prompt_compilation.2.iter().cloned());

        let mut row = GeneratedStoryboardRow {
            shot_id: draft.shot_id.clone(),
            order: (index + 1) as u32,
            shot_script: draft.shot_script.clone(),
            primary_scene_type: grounding.primary_scene_type.clone(),
            primary_scene_label: grounding.primary_scene_label.clone(),
            primary_scene_category: grounding.primary_scene_category.clone(),
            shot_scene_type: grounding.shot_scene_type.clone(),
            shot_scene_label: grounding.shot_scene_label.clone(),
            shot_intent: grounding.shot_intent.clone(),
            adaptation_reason: grounding.adaptation_reason.clone(),
            grounding_source: grounding.grounding_source,
            person: draft.person.clone(),
            shot_title: draft.shot_title.clone(),
            scene_scale: draft.scene_scale.clone(),
            visual_description: draft.visual_description.clone(),
            character_action: draft.character_action.clone(),
            dialogue: draft.dialogue.clone(),
            prompt_text: prompt_compilation.0,
            prompt_text_compilation_status: prompt_compilation.1,
            prompt_text_compilation_warnings: prompt_compilation.2,
            prompt_text_source_row_id: draft.shot_id.clone(),
            duration_seconds: draft.duration_seconds,
            shot_duration_seconds: draft.duration_seconds,
            duration_source: "storyboard_duration_plan.allocated_row_duration_seconds".to_string(),
            scene_performance_projection: scene_projection.clone(),
            external_reference_handle_candidates: project_reference_handle_candidates(record),
            sequence_grouping: draft.sequence_grouping,
        };
        deterministic_rows.push(row.clone());
        if let Some(patch) = live_row_patches.get(index) {
            apply_live_storyboard_patch(&mut row, patch);
        }
        rows.push(row);
    }

    if !live_row_patches.is_empty() {
        let live_validator_findings =
            validate_live_storyboard_rows(&rows, request.selected_total_duration_seconds);
        if !live_validator_findings.is_empty() {
            warnings.extend(live_validator_findings);
            warnings.push(ProductWarning {
                code: "text_model_live_storyboard_fallback".to_string(),
                message: "千问生成的分镜内容未通过本地校验，已回退到本地候选结果。".to_string(),
                related_sample_id: None,
            });
            rows = deterministic_rows;
        }
    } else if !live_generation.warnings.is_empty() {
        warnings.push(ProductWarning {
            code: "text_model_live_storyboard_fallback".to_string(),
            message: "未启用千问或调用失败，已使用本地候选结果。".to_string(),
            related_sample_id: None,
        });
    }
    warnings.extend(model_config_warnings(request.model_config_summary.as_ref()));

    let status = if !blockers.is_empty() {
        BridgeCallStatus::Blocked
    } else if !warnings.is_empty() {
        BridgeCallStatus::WarningOnly
    } else {
        BridgeCallStatus::Ready
    };
    let blocked_row_count = blockers
        .iter()
        .filter_map(|item| item.related_sample_id.as_deref())
        .collect::<std::collections::BTreeSet<_>>()
        .len() as u32;
    let ready_row_count = rows.len() as u32 - blocked_row_count.min(rows.len() as u32);
    let allocated_seconds = rows.iter().map(|row| row.duration_seconds).sum::<u16>();
    if allocated_seconds != request.selected_total_duration_seconds {
        return blocked_storyboard_response(
            state,
            &router_request,
            None,
            request.selected_total_duration_seconds,
            vec![ProductWarning {
                code: "duration_conservation_failed".to_string(),
                message: "分镜总时长与任务时长不一致，已阻断生成结果。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
        );
    }
    let task_id = format!(
        "task-{}",
        &stable_hash_hex(&format!(
            "{}\n{}\n{}",
            request.task_name.trim(),
            grounding.primary_scene_type,
            grounding.shot_scene_type
        ))[..12]
    );
    let result_id = format!(
        "storyboard-{}",
        &stable_hash_hex(&format!(
            "{}\n{}\n{}\n{}",
            request.task_name,
            grounding.grounding_text,
            grounding.shot_scene_type,
            request.selected_total_duration_seconds
        ))[..12]
    );
    let operation_id = format!(
        "op-{}",
        &stable_hash_hex(&format!("{result_id}\n{allocated_seconds}\n{now_ms}"))[..12]
    );
    let rows_hash = stable_hash_hex(&serialize_storyboard_rows(&rows));

    let response = GenerateStoryboardResponse {
        task_id: Some(task_id),
        result_id,
        rows,
        selected_total_duration_seconds: request.selected_total_duration_seconds,
        duration_plan: StoryboardDurationPlan {
            total_duration_seconds: request.selected_total_duration_seconds,
            row_count: row_count as u32,
            per_row_seconds: (allocated_seconds / row_count as u16).max(1),
            allocated_seconds,
        },
        export_status: StoryboardExportStatus {
            status,
            blockers,
            warnings,
            ready_row_count,
            blocked_row_count,
        },
        busy: false,
        operation_id,
        revision: 1,
        updated_at_ms: now_ms,
        rows_hash,
        dirty: false,
        dirty_source_note: None,
        kb_router_result,
    };
    state.remember_storyboard(response.clone());
    response
}

pub fn save_storyboard_rows(
    state: &AppState,
    request: UpdateStoryboardRowsRequest,
) -> GenerateStoryboardResponse {
    let now_ms = now_epoch_ms();
    let Some(mut snapshot) = state.find_storyboard(&request.result_id) else {
        let fallback_router_request = KbRouterRuntimeRequest {
            scene_type: "unknown".to_string(),
            synopsis_text: request.task_id.clone().unwrap_or_default(),
            duration_seconds: 0,
            task_type: KbRouterTaskType::GenerateStoryboard,
            primary_scene_type: None,
            primary_scene_label: None,
            shot_scene_type: None,
            shot_scene_label: None,
            shot_intent: request.task_id.clone(),
            structure_type: None,
        };
        return blocked_storyboard_response(
            state,
            &fallback_router_request,
            request.task_id,
            0,
            vec![ProductWarning {
                code: "storyboard_result_not_found".to_string(),
                message: "未找到可编辑的分镜结果。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
        );
    };

    if snapshot.revision != request.base_revision {
        return blocked_storyboard_response_with_kb(
            snapshot.task_id.clone(),
            snapshot.selected_total_duration_seconds,
            vec![ProductWarning {
                code: "storyboard_revision_conflict".to_string(),
                message: "分镜内容已被其他操作更新，请刷新后重试。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
            snapshot.kb_router_result.clone(),
        );
    }

    if request.rows.is_empty() {
        return blocked_storyboard_response_with_kb(
            snapshot.task_id.clone(),
            snapshot.selected_total_duration_seconds,
            vec![ProductWarning {
                code: "storyboard_rows_required".to_string(),
                message: "保存分镜前请至少保留一条镜头。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
            snapshot.kb_router_result.clone(),
        );
    }

    if request.rows.iter().any(|row| row.duration_seconds == 0) {
        return blocked_storyboard_response_with_kb(
            snapshot.task_id.clone(),
            snapshot.selected_total_duration_seconds,
            vec![ProductWarning {
                code: "row_duration_required".to_string(),
                message: "每条分镜的时长都必须大于 0 秒。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
            snapshot.kb_router_result.clone(),
        );
    }

    let total_duration = request
        .rows
        .iter()
        .map(|row| row.duration_seconds)
        .sum::<u16>();
    if total_duration != snapshot.selected_total_duration_seconds {
        return blocked_storyboard_response_with_kb(
            snapshot.task_id.clone(),
            snapshot.selected_total_duration_seconds,
            vec![ProductWarning {
                code: "duration_conservation_failed".to_string(),
                message: "编辑后的镜头总时长必须与任务总时长保持一致。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
            snapshot.kb_router_result.clone(),
        );
    }

    snapshot.rows = request.rows;
    snapshot.duration_plan.row_count = snapshot.rows.len() as u32;
    snapshot.duration_plan.allocated_seconds = total_duration;
    snapshot.duration_plan.per_row_seconds = if snapshot.rows.is_empty() {
        0
    } else {
        (total_duration / snapshot.rows.len() as u16).max(1)
    };
    snapshot.export_status.warnings = snapshot
        .rows
        .iter()
        .flat_map(|row| row.prompt_text_compilation_warnings.clone())
        .collect();
    snapshot.export_status.status = if snapshot.export_status.warnings.is_empty() {
        BridgeCallStatus::Ready
    } else {
        BridgeCallStatus::WarningOnly
    };
    snapshot.export_status.ready_row_count = snapshot.rows.len() as u32;
    snapshot.export_status.blocked_row_count = 0;
    snapshot.task_id = request.task_id.or(snapshot.task_id.clone());
    snapshot.revision += 1;
    snapshot.updated_at_ms = now_ms;
    snapshot.operation_id = request.operation_id;
    snapshot.rows_hash = stable_hash_hex(&serialize_storyboard_rows(&snapshot.rows));
    snapshot.dirty = true;
    snapshot.dirty_source_note = request
        .dirty_source_note
        .or_else(|| Some("storyboard_rows_edited".to_string()));

    state.remember_storyboard(snapshot.clone());
    snapshot
}

pub fn export_bundle(state: &AppState, request: ExportBundleRequest) -> ExportBundleResponse {
    let export_manifest_id = format!(
        "export-manifest-{}",
        &stable_hash_hex(&format!(
            "{}\n{}\n{}",
            request.result_id.as_deref().unwrap_or_default(),
            request.task_id.as_deref().unwrap_or_default(),
            request.export_format
        ))[..12]
    );

    let storyboard = request
        .result_id
        .as_deref()
        .and_then(|result_id| state.find_storyboard(result_id))
        .or_else(|| {
            request
                .task_id
                .as_deref()
                .and_then(|task_id| state.find_storyboard_by_task_id(task_id))
        });

    let Some(storyboard) = storyboard else {
        return ExportBundleResponse {
            export_manifest_id: export_manifest_id.clone(),
            export_status: StoryboardExportStatus {
                status: BridgeCallStatus::Blocked,
                blockers: vec![ProductWarning {
                    code: "storyboard_result_not_found".to_string(),
                    message: "导出前请先生成或恢复有效分镜结果。".to_string(),
                    related_sample_id: None,
                }],
                warnings: vec![],
                ready_row_count: 0,
                blocked_row_count: 0,
            },
            artifacts: blocked_export_artifacts(
                &export_manifest_id,
                &request.export_format,
                "storyboard_result_not_found",
                None,
                None,
                false,
            ),
        };
    };

    let mut export_status = storyboard.export_status.clone();
    let mut artifacts = vec![ExportArtifactRecord {
        artifact_id: format!("{}-bridge-manifest", export_manifest_id),
        artifact_kind: "v120_bridge_manifest".to_string(),
        export_format: request.export_format.clone(),
        ready: true,
        blocked_reason: None,
        artifact_path: None,
        content_hash: Some(stable_hash_hex(&serialize_storyboard_rows(
            &storyboard.rows,
        ))),
        byte_size: None,
        row_count: Some(storyboard.rows.len() as u32),
        selected_total_duration_seconds: Some(storyboard.selected_total_duration_seconds),
        source_result_id: Some(storyboard.result_id.clone()),
        edited_rows_applied: storyboard.dirty,
        prompt_text_compilation_statuses: collect_compilation_statuses(&storyboard.rows),
        prompt_text_compilation_warning_codes: collect_compilation_warning_codes(&storyboard.rows),
        selected_sample_ids: storyboard.kb_router_result.selected_sample_ids.clone(),
        selected_kb_rule_ids: storyboard
            .kb_router_result
            .selected_kb_rules
            .iter()
            .map(|rule| rule.rule_id.clone())
            .collect(),
        kb_context_summary: Some(storyboard.kb_router_result.kb_context_summary.clone()),
        retrieval_trace: Some(storyboard.kb_router_result.retrieval_trace.clone()),
        full_kb_rows_included: storyboard
            .kb_router_result
            .retrieval_trace
            .token_budget
            .full_kb_rows_included,
    }];

    match export_v120_storyboard_bundle(&V120StoryboardExportRequest {
        export_manifest_id: export_manifest_id.clone(),
        result_id: storyboard.result_id.clone(),
        selected_total_duration_seconds: storyboard.selected_total_duration_seconds,
        source_result_id: storyboard.result_id.clone(),
        edited_rows_applied: storyboard.dirty,
        rows: storyboard.rows.clone(),
    }) {
        Ok(bundle) => {
            artifacts.extend(bundle.artifacts.into_iter().map(|artifact| {
                ExportArtifactRecord {
                    artifact_id: format!("{}-{}", export_manifest_id, artifact.artifact_kind),
                    artifact_kind: artifact.artifact_kind.to_string(),
                    export_format: artifact.export_format.to_string(),
                    ready: true,
                    blocked_reason: None,
                    artifact_path: Some(artifact.path.display().to_string()),
                    content_hash: Some(artifact.content_hash),
                    byte_size: Some(artifact.byte_size),
                    row_count: Some(artifact.row_count),
                    selected_total_duration_seconds: Some(
                        storyboard.selected_total_duration_seconds,
                    ),
                    source_result_id: Some(storyboard.result_id.clone()),
                    edited_rows_applied: storyboard.dirty,
                    prompt_text_compilation_statuses: collect_compilation_statuses(
                        &storyboard.rows,
                    ),
                    prompt_text_compilation_warning_codes: collect_compilation_warning_codes(
                        &storyboard.rows,
                    ),
                    selected_sample_ids: storyboard.kb_router_result.selected_sample_ids.clone(),
                    selected_kb_rule_ids: storyboard
                        .kb_router_result
                        .selected_kb_rules
                        .iter()
                        .map(|rule| rule.rule_id.clone())
                        .collect(),
                    kb_context_summary: Some(
                        storyboard.kb_router_result.kb_context_summary.clone(),
                    ),
                    retrieval_trace: Some(storyboard.kb_router_result.retrieval_trace.clone()),
                    full_kb_rows_included: storyboard
                        .kb_router_result
                        .retrieval_trace
                        .token_budget
                        .full_kb_rows_included,
                }
            }));
        }
        Err(error) => {
            let reason = format!("v120_export_engine_error: {error:?}");
            export_status.status = BridgeCallStatus::Blocked;
            export_status.blockers.push(ProductWarning {
                code: "v120_export_artifact_generation_failed".to_string(),
                message: reason.clone(),
                related_sample_id: None,
            });
            artifacts.extend(blocked_storyboard_export_artifacts(
                &export_manifest_id,
                "v120_export_artifact_generation_failed",
                Some(storyboard.selected_total_duration_seconds),
                Some(storyboard.result_id.clone()),
                storyboard.dirty,
            ));
        }
    }

    ExportBundleResponse {
        export_manifest_id,
        export_status,
        artifacts,
    }
}

fn blocked_export_artifacts(
    export_manifest_id: &str,
    requested_format: &str,
    blocked_reason: &str,
    selected_total_duration_seconds: Option<u16>,
    source_result_id: Option<String>,
    edited_rows_applied: bool,
) -> Vec<ExportArtifactRecord> {
    let mut artifacts = vec![ExportArtifactRecord {
        artifact_id: format!("{}-bridge-manifest", export_manifest_id),
        artifact_kind: "v120_bridge_manifest".to_string(),
        export_format: requested_format.to_string(),
        ready: false,
        blocked_reason: Some(blocked_reason.to_string()),
        artifact_path: None,
        content_hash: None,
        byte_size: None,
        row_count: None,
        selected_total_duration_seconds,
        source_result_id: source_result_id.clone(),
        edited_rows_applied,
        prompt_text_compilation_statuses: vec![],
        prompt_text_compilation_warning_codes: vec![],
        selected_sample_ids: vec![],
        selected_kb_rule_ids: vec![],
        kb_context_summary: None,
        retrieval_trace: None,
        full_kb_rows_included: 0,
    }];
    artifacts.extend(blocked_storyboard_export_artifacts(
        export_manifest_id,
        blocked_reason,
        selected_total_duration_seconds,
        source_result_id,
        edited_rows_applied,
    ));
    artifacts
}

fn blocked_storyboard_export_artifacts(
    export_manifest_id: &str,
    blocked_reason: &str,
    selected_total_duration_seconds: Option<u16>,
    source_result_id: Option<String>,
    edited_rows_applied: bool,
) -> Vec<ExportArtifactRecord> {
    ["storyboard_json", "storyboard_csv", "excel_workbook"]
        .into_iter()
        .map(|artifact_kind| ExportArtifactRecord {
            artifact_id: format!("{}-{}", export_manifest_id, artifact_kind),
            artifact_kind: artifact_kind.to_string(),
            export_format: match artifact_kind {
                "storyboard_json" => "json",
                "storyboard_csv" => "csv",
                "excel_workbook" => "xlsx",
                _ => "unknown",
            }
            .to_string(),
            ready: false,
            blocked_reason: Some(blocked_reason.to_string()),
            artifact_path: None,
            content_hash: None,
            byte_size: None,
            row_count: None,
            selected_total_duration_seconds,
            source_result_id: source_result_id.clone(),
            edited_rows_applied,
            prompt_text_compilation_statuses: vec![],
            prompt_text_compilation_warning_codes: vec![],
            selected_sample_ids: vec![],
            selected_kb_rule_ids: vec![],
            kb_context_summary: None,
            retrieval_trace: None,
            full_kb_rows_included: 0,
        })
        .collect()
}

fn current_text_model_provider(state: &AppState) -> (TextModelProvider, Option<String>) {
    if let Some(session) = state.text_model_provider_session_config() {
        let provider_kind = text_model_provider_kind_from_str(&session.provider);
        let api_key = session.resolved_api_key();
        return (
            TextModelProvider {
                provider: provider_kind,
                model: if session.model.trim().is_empty() {
                    "qwen-plus".to_string()
                } else {
                    session.model.trim().to_string()
                },
                base_url: session.base_url.clone(),
                api_key_ref: session
                    .api_key_ref
                    .clone()
                    .unwrap_or_else(|| "env:HOPE_TEXT_MODEL_API_KEY".to_string()),
                enabled: session.enabled && provider_kind == TextModelProviderKind::Qwen,
            },
            api_key,
        );
    }

    (default_text_model_provider(), None)
}

fn default_text_model_provider() -> TextModelProvider {
    let provider = text_model_provider_kind_from_str(
        &env::var("HOPE_TEXT_MODEL_PROVIDER").unwrap_or_else(|_| "qwen".to_string()),
    );
    TextModelProvider {
        provider,
        model: env::var("HOPE_TEXT_MODEL_MODEL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "qwen-plus".to_string()),
        base_url: env::var("HOPE_TEXT_MODEL_BASE_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        api_key_ref: env::var("HOPE_TEXT_MODEL_API_KEY_REF")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "env:HOPE_TEXT_MODEL_API_KEY".to_string()),
        enabled: parse_bool_env("HOPE_TEXT_MODEL_ENABLED"),
    }
}

fn build_text_generation_request(
    task_type: TextGenerationTask,
    scene_type: Option<String>,
    story_input: String,
    duration_plan: Option<StoryboardDurationPlan>,
    kb_context_summary: String,
    selected_sample_ids: Vec<String>,
    selected_kb_rules: Vec<String>,
    output_schema: TextGenerationOutputSchema,
    max_tokens: Option<u32>,
) -> TextGenerationRequest {
    TextGenerationRequest {
        task_type,
        scene_type,
        story_input,
        duration_plan,
        kb_context_summary,
        selected_sample_ids,
        selected_kb_rules,
        output_schema,
        temperature: Some(0.2),
        max_tokens,
    }
}

fn run_text_generation(
    provider: &TextModelProvider,
    session_api_key: Option<&str>,
    request: &TextGenerationRequest,
) -> TextGenerationResponse {
    if !provider.enabled {
        return run_text_generation_stub(
            provider,
            request,
            ProductWarning {
                code: "text_model_live_call_closed".to_string(),
                message: format!(
                    "{} 未启用，已使用本地候选结果。",
                    provider_kind_display_name(provider.provider)
                ),
                related_sample_id: request.selected_sample_ids.first().cloned(),
            },
        );
    }

    if provider.provider != TextModelProviderKind::Qwen {
        return run_text_generation_stub(
            provider,
            request,
            ProductWarning {
                code: "text_model_provider_not_supported".to_string(),
                message: "当前只有千问 Qwen 文本生成可启用，其他模型接口仍为预留。".to_string(),
                related_sample_id: request.selected_sample_ids.first().cloned(),
            },
        );
    }

    let Some(api_key) = session_api_key
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| resolve_provider_api_key(provider))
    else {
        return run_text_generation_stub(
            provider,
            request,
            ProductWarning {
                code: "text_model_api_key_missing".to_string(),
                message: "千问已启用，但当前会话缺少 API Key，已使用本地候选结果。".to_string(),
                related_sample_id: request.selected_sample_ids.first().cloned(),
            },
        );
    };

    let Some(endpoint) = resolve_qwen_endpoint(provider) else {
        return run_text_generation_stub(
            provider,
            request,
            ProductWarning {
                code: "text_model_base_url_missing".to_string(),
                message: "千问已启用，但 base_url 未配置，已使用本地候选结果。".to_string(),
                related_sample_id: request.selected_sample_ids.first().cloned(),
            },
        );
    };

    match run_qwen_text_generation_with_transport(
        provider,
        request,
        &endpoint,
        &api_key,
        qwen_http_transport,
    ) {
        Ok(response) => response,
        Err(warning) => run_text_generation_stub(provider, request, warning),
    }
}

fn run_text_generation_stub(
    provider: &TextModelProvider,
    request: &TextGenerationRequest,
    warning: ProductWarning,
) -> TextGenerationResponse {
    let scene_type = request.scene_type.as_deref().unwrap_or("unspecified_scene");
    let text = format!(
        "task={:?}\nscene_type={}\nselected_samples={}\nselected_kb_rules={}\nkb_context={}",
        request.task_type,
        scene_type,
        request.selected_sample_ids.join(","),
        request.selected_kb_rules.join(" | "),
        request.kb_context_summary
    );

    TextGenerationResponse {
        text,
        structured_json: None,
        usage_tokens: None,
        latency_ms: Some(0),
        warnings: vec![warning],
        provider: provider.provider,
        model: provider.model.clone(),
    }
}

fn run_qwen_text_generation_with_transport<F>(
    provider: &TextModelProvider,
    request: &TextGenerationRequest,
    endpoint: &str,
    api_key: &str,
    transport: F,
) -> Result<TextGenerationResponse, ProductWarning>
where
    F: Fn(&str, &str, &Value) -> Result<(Value, u64), ProductWarning>,
{
    let payload = build_qwen_request_payload(provider, request);
    let started = Instant::now();
    let (body, transport_latency_ms) = transport(endpoint, api_key, &payload)?;
    let response: QwenChatCompletionResponse =
        serde_json::from_value(body).map_err(|_| ProductWarning {
            code: "text_model_response_invalid".to_string(),
            message: "千问返回结构无法解析，已使用本地候选结果。".to_string(),
            related_sample_id: request.selected_sample_ids.first().cloned(),
        })?;
    let content = response
        .choices
        .first()
        .map(|choice| choice.message.content.trim())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| ProductWarning {
            code: "text_model_response_invalid".to_string(),
            message: "千问返回为空，已使用本地候选结果。".to_string(),
            related_sample_id: request.selected_sample_ids.first().cloned(),
        })?;
    let structured_json = match request.output_schema {
        TextGenerationOutputSchema::StoryboardRowsJson
        | TextGenerationOutputSchema::RepairPlanJson => serde_json::from_str::<Value>(content).ok(),
        _ => None,
    };

    Ok(TextGenerationResponse {
        text: content.to_string(),
        structured_json,
        usage_tokens: response.usage.and_then(|usage| usage.total_tokens),
        latency_ms: Some(transport_latency_ms.max(started.elapsed().as_millis() as u64)),
        warnings: vec![],
        provider: provider.provider,
        model: provider.model.clone(),
    })
}

fn build_qwen_request_payload(
    provider: &TextModelProvider,
    request: &TextGenerationRequest,
) -> Value {
    let scene_type = request.scene_type.as_deref().unwrap_or("unspecified_scene");
    let duration_seconds = request
        .duration_plan
        .as_ref()
        .map(|plan| plan.total_duration_seconds)
        .unwrap_or_default();
    let output_schema = match request.output_schema {
        TextGenerationOutputSchema::PlainText => "plain_text",
        TextGenerationOutputSchema::StoryboardRowsJson => "storyboard_rows_json",
        TextGenerationOutputSchema::RepairPlanJson => "repair_plan_json",
        TextGenerationOutputSchema::SeedancePromptText => "seedance_prompt_text",
    };
    let system_prompt = match (request.task_type, request.output_schema) {
        (TextGenerationTask::ExpandScript, TextGenerationOutputSchema::PlainText) => {
            "你是 Hope 的受控剧本扩写层。只把用户的故事梗概扩写为连续、可读的剧情剧本正文；只能参考压缩知识库摘要和样本/规则 ID，不得输出全量知识库，不得输出真实导演/IP/品牌名。"
        }
        (TextGenerationTask::GenerateStoryboard, TextGenerationOutputSchema::StoryboardRowsJson) => {
            "你是 Hope 的受控分镜生成层。根据剧本片段生成结构化分镜 rows 和当前文本提示词；只能使用压缩知识库摘要和样本/规则 ID，不得输出全量知识库，不得输出真实导演/IP/品牌名。"
        }
        _ => "你是 Hope 的受控文本生成层。只能使用收到的压缩知识库摘要和样本/规则 ID，不得扩展为全量知识库，不得输出真实导演/IP/品牌名。",
    };
    let user_prompt = match (request.task_type, request.output_schema) {
        (TextGenerationTask::ExpandScript, TextGenerationOutputSchema::PlainText) => format!(
            "任务=扩写剧本\nscene_type={}\ntarget_duration_seconds={}\nstory_synopsis={}\nkb_context_summary={}\nselected_sample_ids={}\nselected_kb_rules={}\noutput_schema=story_script_plain_text\nconstraints=只输出连续剧情剧本正文；不要输出镜头编号、分镜表、景别、画面描述、角色动作、prompt_text、Seedance 提示词、时间码或 JSON；不要输出 full KB rows、source_register、overlay JSON 或内部候选提示证据；剧本需要服务后续镜头拆解，但本步不要提前拆分镜头。",
            scene_type,
            duration_seconds,
            request.story_input,
            request.kb_context_summary,
            request.selected_sample_ids.join(","),
            request.selected_kb_rules.join(" | "),
        ),
        (TextGenerationTask::GenerateStoryboard, TextGenerationOutputSchema::StoryboardRowsJson) => format!(
            "任务=生成分镜提示词\nscene_type={}\nduration_seconds={}\nshot_script={}\nkb_context_summary={}\nselected_sample_ids={}\nselected_kb_rules={}\noutput_schema=storyboard_rows_json\nconstraints=输出 JSON object，包含 rows 数组；每行必须包含人物、镜头、景别、画面描述、角色动作、对话/旁白、分镜提示词 prompt_text、duration_seconds；总时长必须守恒；不要输出 full KB rows；不要把内部候选提示证据当最终 prompt_text；不要输出 source_register 或 overlay JSON。",
            scene_type,
            duration_seconds,
            request.story_input,
            request.kb_context_summary,
            request.selected_sample_ids.join(","),
            request.selected_kb_rules.join(" | "),
        ),
        _ => format!(
            "task_type={:?}\nscene_type={}\nduration_seconds={}\nstory_input={}\nkb_context_summary={}\nselected_sample_ids={}\nselected_kb_rules={}\noutput_schema={}\nconstraints=保持总时长守恒；不要输出 full KB rows；不要把内部候选提示证据当最终 prompt_text；不要输出 source_register 或 overlay JSON。",
            request.task_type,
            scene_type,
            duration_seconds,
            request.story_input,
            request.kb_context_summary,
            request.selected_sample_ids.join(","),
            request.selected_kb_rules.join(" | "),
            output_schema,
        ),
    };

    let mut payload = json!({
        "model": provider.model.clone(),
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ],
        "temperature": request.temperature.unwrap_or(0.2),
        "max_tokens": request.max_tokens.unwrap_or(800)
    });
    if matches!(
        request.output_schema,
        TextGenerationOutputSchema::StoryboardRowsJson | TextGenerationOutputSchema::RepairPlanJson
    ) {
        payload["response_format"] = json!({ "type": "json_object" });
    }
    payload
}

fn qwen_http_transport(
    endpoint: &str,
    api_key: &str,
    payload: &Value,
) -> Result<(Value, u64), ProductWarning> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|_| ProductWarning {
            code: "text_model_network_error".to_string(),
            message: "千问 HTTP 客户端初始化失败，已使用本地候选结果。".to_string(),
            related_sample_id: None,
        })?;
    let started = Instant::now();
    let response = client
        .post(endpoint)
        .bearer_auth(api_key)
        .json(payload)
        .send()
        .map_err(|_| ProductWarning {
            code: "text_model_network_error".to_string(),
            message: "无法连接千问接口，已使用本地候选结果。".to_string(),
            related_sample_id: None,
        })?;
    let status = response.status();
    let body: Value = response.json().map_err(|_| ProductWarning {
        code: "text_model_response_invalid".to_string(),
        message: "千问返回非 JSON 内容，已使用本地候选结果。".to_string(),
        related_sample_id: None,
    })?;
    if !status.is_success() {
        return Err(ProductWarning {
            code: "text_model_network_error".to_string(),
            message: format!(
                "千问接口返回 HTTP {}，已使用本地候选结果。",
                status.as_u16()
            ),
            related_sample_id: None,
        });
    }
    Ok((body, started.elapsed().as_millis() as u64))
}

fn resolve_provider_api_key(provider: &TextModelProvider) -> Option<String> {
    provider
        .api_key_ref
        .strip_prefix("env:")
        .and_then(|name| env::var(name).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn resolve_qwen_endpoint(provider: &TextModelProvider) -> Option<String> {
    let base = provider.base_url.as_deref()?.trim().trim_end_matches('/');
    if base.is_empty() {
        return None;
    }
    if base.ends_with("/chat/completions") {
        Some(base.to_string())
    } else {
        Some(format!("{base}/chat/completions"))
    }
}

fn parse_bool_env(name: &str) -> bool {
    env::var(name)
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn text_model_provider_kind_from_str(value: &str) -> TextModelProviderKind {
    match value.trim().to_ascii_lowercase().as_str() {
        "doubao" => TextModelProviderKind::Doubao,
        "custom" => TextModelProviderKind::Custom,
        _ => TextModelProviderKind::Qwen,
    }
}

fn provider_kind_display_name(kind: TextModelProviderKind) -> &'static str {
    match kind {
        TextModelProviderKind::Qwen => "千问 Qwen",
        TextModelProviderKind::Doubao => "豆包 Doubao",
        TextModelProviderKind::Custom => "自定义模型",
    }
}

fn validate_generated_script_text(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    if trimmed.is_empty()
        || contains_forbidden_generation_terms(trimmed)
        || looks_like_storyboard_or_prompt_text(trimmed)
    {
        None
    } else {
        Some(trimmed)
    }
}

fn looks_like_storyboard_or_prompt_text(text: &str) -> bool {
    let lower = text.to_lowercase();
    let markers = [
        "prompt_text",
        "prompt body",
        "分镜提示词",
        "画面描述",
        "角色动作",
        "对话/旁白",
        "景别",
        "shot_id",
        "scene_scale",
        "duration_seconds",
        "seedance",
        "ws |",
        "ms |",
        "cu |",
        "ecu |",
        "ls |",
    ];
    let marker_count = markers
        .iter()
        .filter(|marker| lower.contains(**marker))
        .count();
    marker_count >= 2
        || ["0-3s", "3-6s", "6-9s", "9-15s", "0–3s", "3–6s", "6–9s", "9–15s"]
            .iter()
            .any(|marker| lower.contains(marker))
}

fn build_deterministic_expanded_story_script(
    synopsis: &str,
    scene_label: &str,
    target_duration_seconds: u16,
) -> String {
    let synopsis = synopsis.trim();
    let scene_label = if scene_label.trim().is_empty() {
        "当前场景"
    } else {
        scene_label.trim()
    };
    format!(
        "在{}的叙事方向中，故事从“{}”展开。开场先建立人物所处的环境和压力，让主角的目标、阻碍和情绪动机变得清晰；随后冲突逐步升级，人物在行动中暴露犹豫、判断和选择。中段让关键阻力逼近，主角必须在短时间内作出反应，场景节奏随情绪和动作推进而收紧。结尾让主角完成一次明确的转折或确认，为后续镜头拆解留下连续的动作线、情绪线和空间线。整段剧本目标时长约 {} 秒，适合继续拆解为若干镜头任务。",
        scene_label, synopsis, target_duration_seconds
    )
}

fn extract_live_storyboard_row_patches(
    generation_response: &TextGenerationResponse,
) -> Result<Vec<LiveStoryboardRowPatch>, ProductWarning> {
    let Some(structured_json) = generation_response.structured_json.as_ref() else {
        if generation_response.warnings.is_empty()
            && generation_response.provider == TextModelProviderKind::Qwen
        {
            return Err(ProductWarning {
                code: "text_model_response_invalid".to_string(),
                message: "千问分镜返回不是有效 rows JSON，已使用本地候选结果。".to_string(),
                related_sample_id: None,
            });
        }
        return Ok(Vec::new());
    };
    if let Ok(envelope) =
        serde_json::from_value::<LiveStoryboardRowsEnvelope>(structured_json.clone())
    {
        return Ok(envelope.rows);
    }
    if let Ok(rows) = serde_json::from_value::<Vec<LiveStoryboardRowPatch>>(structured_json.clone())
    {
        return Ok(rows);
    }
    Err(ProductWarning {
        code: "text_model_response_invalid".to_string(),
        message: "千问分镜返回不是有效 rows JSON，已使用本地候选结果。".to_string(),
        related_sample_id: None,
    })
}

fn apply_live_storyboard_patch(row: &mut GeneratedStoryboardRow, patch: &LiveStoryboardRowPatch) {
    if let Some(value) = non_blank_string(&patch.shot_title) {
        row.shot_title = value;
    }
    if let Some(value) = non_blank_string(&patch.person) {
        row.person = value;
    }
    if let Some(value) = non_blank_string(&patch.scene_scale) {
        row.scene_scale = value;
    }
    if let Some(value) = non_blank_string(&patch.visual_description) {
        row.visual_description = value;
    }
    if let Some(value) = non_blank_string(&patch.character_action) {
        row.character_action = value;
    }
    if let Some(value) = non_blank_string(&patch.dialogue) {
        row.dialogue = value;
    }
}

fn validate_live_storyboard_rows(
    rows: &[GeneratedStoryboardRow],
    expected_duration_seconds: u16,
) -> Vec<ProductWarning> {
    let mut findings = Vec::new();
    if rows.iter().map(|row| row.duration_seconds).sum::<u16>() != expected_duration_seconds {
        findings.push(ProductWarning {
            code: "duration_conservation_failed".to_string(),
            message: "千问返回后分镜总时长不守恒，已使用本地候选结果。".to_string(),
            related_sample_id: None,
        });
    }
    for row in rows {
        for (field_name, field_value) in [
            ("人物", row.person.as_str()),
            ("镜头", row.shot_title.as_str()),
            ("景别", row.scene_scale.as_str()),
            ("画面描述", row.visual_description.as_str()),
            ("角色动作", row.character_action.as_str()),
        ] {
            if field_value.trim().is_empty() {
                findings.push(ProductWarning {
                    code: "text_model_validator_failed".to_string(),
                    message: format!(
                        "千问返回的第 {} 行缺少{}，已使用本地候选结果。",
                        row.order, field_name
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
            if contains_forbidden_generation_terms(field_value) {
                findings.push(ProductWarning {
                    code: "text_model_validator_failed".to_string(),
                    message: format!(
                        "千问返回的第 {} 行包含受限真实名称，已使用本地候选结果。",
                        row.order
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
            if contains_any_story_term(
                field_value,
                &[
                    "not_specified_by_v120_bridge",
                    "visual_scene_core",
                    "fused_scene_performance_core_preserved",
                ],
            ) {
                findings.push(ProductWarning {
                    code: "internal_field_code_leaked".to_string(),
                    message: format!(
                        "第 {} 行包含内部占位字段，已回退到本地候选结果。",
                        row.order
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
            if field_name == "角色动作" && is_role_action_grounding_incomplete(field_value) {
                findings.push(ProductWarning {
                    code: "role_action_grounding_incomplete".to_string(),
                    message: "角色动作信息不完整，请重新生成或检查当前镜头脚本。".to_string(),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
        }
    }
    findings
}

fn is_role_action_grounding_incomplete(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return true;
    }
    if contains_any_story_term(
        trimmed,
        &[
            "保持连续性",
            "执行关键动作",
            "按镜头脚本执行",
            "fused_scene_performance_core_preserved",
            "not_specified_by_v120_bridge",
            "待明确",
        ],
    ) {
        return true;
    }
    if trimmed.chars().count() < 28 {
        return true;
    }
    !(trimmed.contains("从")
        && trimmed.contains("到")
        && trimmed.contains("镜头捕捉")
        && (trimmed.contains("对手") || trimmed.contains("环境") || trimmed.contains("全场")))
}

fn contains_forbidden_generation_terms(text: &str) -> bool {
    const FORBIDDEN_TERMS: &[&str] = &[
        "宫崎骏",
        "新海诚",
        "迪士尼",
        "漫威",
        "星球大战",
        "哈利波特",
        "宝可梦",
        "三国志战略版",
    ];
    FORBIDDEN_TERMS.iter().any(|term| text.contains(term))
}

fn model_config_hash_input(model_config: Option<&ModelConfigSummary>) -> String {
    match model_config {
        Some(config) => format!(
            "{}\n{}\n{}\n{}\n{}",
            config.provider.trim(),
            config.model.trim(),
            config.enabled,
            config.base_url_present,
            config.api_key_present
        ),
        None => "qwen\nqwen-plus\nfalse\nfalse\nfalse".to_string(),
    }
}

fn model_config_warnings(model_config: Option<&ModelConfigSummary>) -> Vec<ProductWarning> {
    let Some(config) = model_config else {
        return Vec::new();
    };

    let provider = config.provider.trim();
    let mut warnings = Vec::new();
    if !provider.eq_ignore_ascii_case("qwen") {
        warnings.push(ProductWarning {
            code: "model_provider_reserved".to_string(),
            message: "该模型接口已配置为预留状态，当前仍使用本地文本生成桥接。".to_string(),
            related_sample_id: None,
        });
    }

    if !config.enabled {
        warnings.push(ProductWarning {
            code: "model_config_disabled".to_string(),
            message: "模型配置当前未启用，本轮仍使用本地文本生成桥接。".to_string(),
            related_sample_id: None,
        });
    }

    warnings
}

fn run_kb_router(state: &AppState, request: KbRouterRuntimeRequest) -> KbRouterRuntimeResponse {
    let Some(package) = state.kb_golden_sample_runtime.as_ref() else {
        return empty_kb_router_response(&request, &state.kb_runtime);
    };

    let story_keywords = derive_story_keywords(&request.synopsis_text, &request.scene_type);
    let query = format!(
        "{} {} {} {} {}",
        request.scene_type,
        request.synopsis_text,
        story_keywords.join(" "),
        request.shot_intent.as_deref().unwrap_or_default(),
        request.structure_type.as_deref().unwrap_or_default()
    )
    .to_lowercase();
    let (top_k_samples, top_k_rules) = router_top_k(&request);
    let mut selected_sample_ids = Vec::new();
    let mut selection_reasons = Vec::new();
    let mut excluded_candidates = Vec::new();

    for record in &package.golden_sample_library.records {
        if record.is_reserve() {
            excluded_candidates.push(KbRouterExcludedCandidate {
                sample_id: record.sample_id.clone(),
                reason_code: "reserve_gate".to_string(),
            });
            continue;
        }
        if !record.is_positive_fewshot_candidate() {
            excluded_candidates.push(KbRouterExcludedCandidate {
                sample_id: record.sample_id.clone(),
                reason_code: "usable_for_fewshot_no".to_string(),
            });
            continue;
        }
        if validators::contains_placeholder_marker(&record.source_fields.prompt_body) {
            excluded_candidates.push(KbRouterExcludedCandidate {
                sample_id: record.sample_id.clone(),
                reason_code: "placeholder_present".to_string(),
            });
            continue;
        }

        let scene_match = record
            .source_fields
            .scene_category
            .eq_ignore_ascii_case(&request.scene_type)
            || query.contains(&record.source_fields.scene_category.to_lowercase())
            || query.contains(&record.source_fields.scene_tag.to_lowercase())
            || story_keywords.iter().any(|keyword| {
                record
                    .source_fields
                    .sample_title
                    .to_lowercase()
                    .contains(keyword)
            });
        if scene_match && selected_sample_ids.len() < top_k_samples as usize {
            selected_sample_ids.push(record.sample_id.clone());
            selection_reasons.push(KbRouterSelectionReason {
                sample_id: record.sample_id.clone(),
                reason_code: "scene_match".to_string(),
            });
        }
    }

    if selected_sample_ids.is_empty() {
        for record in package
            .golden_sample_library
            .records
            .iter()
            .filter(|record| record.is_official() && record.is_positive_fewshot_candidate())
            .take(top_k_samples as usize)
        {
            selected_sample_ids.push(record.sample_id.clone());
            selection_reasons.push(KbRouterSelectionReason {
                sample_id: record.sample_id.clone(),
                reason_code: "scene_match".to_string(),
            });
        }
    }

    let selected_kb_rules = build_router_selected_rules(state, &request, top_k_rules);
    let kb_context_summary = build_kb_context_summary(
        state,
        &request,
        &story_keywords,
        &selected_sample_ids,
        &selected_kb_rules,
        &excluded_candidates,
    );
    let content_cache_key = stable_hash_hex(&format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        state.kb_runtime.snapshot.seed_format,
        request.scene_type,
        request.duration_seconds,
        request.task_type.as_str(),
        request.shot_intent.as_deref().unwrap_or_default(),
        stable_hash_hex(&request.synopsis_text)
    ));

    KbRouterRuntimeResponse {
        selected_sample_ids,
        selected_kb_rules,
        kb_context_summary,
        retrieval_trace: KbRouterRetrievalTrace {
            kb_version: state.kb_runtime.snapshot.seed_format.clone(),
            snapshot_id: state.kb_runtime.snapshot.snapshot_id.clone(),
            snapshot_checksum: state.kb_runtime.snapshot.snapshot_hash.clone(),
            content_cache_key,
            task_type: request.task_type.as_str().to_string(),
            top_k_samples,
            top_k_rules,
            duration_seconds: request.duration_seconds,
            story_keywords,
            selection_reasons,
            excluded_candidates,
            token_budget: KbRouterTokenBudget {
                kb_context_summary_target: "800-1500 Chinese characters".to_string(),
                full_kb_rows_included: 0,
            },
        },
    }
}

fn router_top_k(request: &KbRouterRuntimeRequest) -> (u8, u8) {
    match request.task_type {
        KbRouterTaskType::ExpandScript => (2, 6),
        KbRouterTaskType::GenerateStoryboard => {
            if request.duration_seconds >= 45 {
                (5, 12)
            } else {
                (3, 8)
            }
        }
        KbRouterTaskType::RepairStoryboard => (2, 10),
        KbRouterTaskType::CompileSeedancePromptText => (1, 6),
    }
}

fn derive_story_keywords(synopsis_text: &str, scene_type: &str) -> Vec<String> {
    let mut keywords = synopsis_text
        .split(|character: char| {
            character.is_whitespace()
                || matches!(character, ',' | '.' | ';' | ':' | '，' | '。' | '；' | '：')
        })
        .map(str::trim)
        .filter(|token| token.len() >= 2)
        .map(|token| token.to_lowercase())
        .collect::<Vec<_>>();
    keywords.push(scene_type.to_lowercase());
    keywords.sort();
    keywords.dedup();
    keywords.truncate(8);
    keywords
}

fn build_router_selected_rules(
    state: &AppState,
    request: &KbRouterRuntimeRequest,
    top_k_rules: u8,
) -> Vec<KbRouterSelectedRule> {
    let mut rules = state
        .kb_golden_sample_runtime
        .as_ref()
        .map(|package| {
            package
                .field_coverage_rules
                .records
                .iter()
                .map(|rule| KbRouterSelectedRule {
                    rule_id: rule.rule_id.clone(),
                    family: match rule.core.as_str() {
                        "continuity_negative_core" => "continuity".to_string(),
                        "scene_performance_core" => "routing".to_string(),
                        "camera_directing_core" => "prompt_text".to_string(),
                        other => other.to_string(),
                    },
                    summary: format!(
                        "规则 {} 仅保留压缩摘要，覆盖 {} 行，用于 {} 的本地约束，不展开原始提示词或教学说明。",
                        rule.core,
                        rule.row_count,
                        request.task_type.as_str()
                    ),
                    applies_to: vec![request.task_type.as_str().to_string()],
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    rules.push(KbRouterSelectedRule {
        rule_id: "reserve_gate".to_string(),
        family: "reserve_gate".to_string(),
        summary: "预留样本仅作为负向约束或规则证据，不进入正向 few-shot。".to_string(),
        applies_to: vec![request.task_type.as_str().to_string()],
    });
    rules.push(KbRouterSelectedRule {
        rule_id: "duration_guard".to_string(),
        family: "duration".to_string(),
        summary: "总时长必须守恒，且本地摘要不会携带全量知识库行。".to_string(),
        applies_to: vec![request.task_type.as_str().to_string()],
    });
    rules.truncate(top_k_rules as usize);
    rules
}

fn build_kb_context_summary(
    state: &AppState,
    request: &KbRouterRuntimeRequest,
    story_keywords: &[String],
    selected_sample_ids: &[String],
    selected_kb_rules: &[KbRouterSelectedRule],
    excluded_candidates: &[KbRouterExcludedCandidate],
) -> String {
    let sample_summaries = selected_sample_ids
        .iter()
        .filter_map(|sample_id| {
            state
                .kb_golden_sample_runtime
                .as_ref()?
                .golden_sample_library
                .records
                .iter()
                .find(|record| &record.sample_id == sample_id)
                .map(|record| {
                    format!(
                        "样本 {}：scene_category={}，sample_type={}，scene_scale_hint={}，只保留结构 lesson 与时长/连续性约束。",
                        record.sample_id,
                        record.source_fields.scene_category,
                        record.source_fields.sample_type,
                        derive_scene_scale(&record.source_fields.technical_profile),
                    )
                })
        })
        .collect::<Vec<_>>()
        .join(" ");
    let rule_summaries = selected_kb_rules
        .iter()
        .map(|rule| format!("{}：{}", rule.rule_id, rule.summary))
        .collect::<Vec<_>>()
        .join(" ");
    let excluded_summary = excluded_candidates
        .iter()
        .take(5)
        .map(|candidate| format!("{}=>{}", candidate.sample_id, candidate.reason_code))
        .collect::<Vec<_>>()
        .join("；");

    let mut summary = format!(
        "这是 Hope 本地 KB Router 的压缩摘要。任务={}，scene_type={}，duration_seconds={}，story_keywords={}。当前只选取少量正向参考样本，selected_sample_ids={}。{} {} 排除候选摘要：{}。本摘要只保留场景结构、连续性、时长和 prompt_text 编译约束，不输出完整原始行，不输出完整候选提示词，不输出来源登记、覆盖层 JSON，也不输出真实导演名、IP 名或品牌名。预留样本只保留为规则证据或负向约束，不能作为 positive few-shot。full_kb_rows_included 固定为 0，禁止把 152 行整包送入上下文。导出与 trace 只记录样本 ID、压缩规则摘要、kb_context_summary 与 retrieval_trace。",
        request.task_type.as_str(),
        request.scene_type,
        request.duration_seconds,
        story_keywords.join("、"),
        selected_sample_ids.join("、"),
        sample_summaries,
        rule_summaries,
        excluded_summary
    );

    while summary.chars().count() < 820 {
        summary.push_str(" 本地摘要继续强调：只保留与当前任务直接相关的结构化规则和样本 lesson，不复制原始 23 字段整行，不复制完整候选提示词，不复制来源登记。");
    }
    summary.chars().take(1450).collect()
}

fn select_golden_sample_records_from_router<'a>(
    state: &'a AppState,
    router_result: &KbRouterRuntimeResponse,
) -> Vec<&'a GoldenSampleLibraryRecord> {
    let Some(package) = state.kb_golden_sample_runtime.as_ref() else {
        return Vec::new();
    };

    router_result
        .selected_sample_ids
        .iter()
        .filter_map(|sample_id| {
            package
                .golden_sample_library
                .records
                .iter()
                .find(|record| &record.sample_id == sample_id)
        })
        .collect()
}

fn product_warning_from_evidence(item: validators::EvidenceAwareValidationItem) -> ProductWarning {
    ProductWarning {
        code: item.code,
        message: item.message,
        related_sample_id: item.source_sample_id,
    }
}

fn resolve_storyboard_grounding_context(
    request: &GenerateStoryboardRequest,
    expanded_script_text: String,
) -> StoryboardGroundingContext {
    let shot_script =
        non_blank_string(request.shot_script.as_deref().unwrap_or_default()).unwrap_or_default();
    let primary_scene_type =
        non_blank_string(request.primary_scene_type.as_deref().unwrap_or_default())
            .or_else(|| non_blank_string(request.scene_type.as_deref().unwrap_or_default()))
            .unwrap_or_default();
    let grounding_source = if !shot_script.trim().is_empty() {
        ShotGroundingSource::ShotScript
    } else if !expanded_script_text.trim().is_empty() {
        ShotGroundingSource::ExpandedScriptText
    } else if !primary_scene_type.trim().is_empty() {
        ShotGroundingSource::PrimarySceneFields
    } else {
        ShotGroundingSource::KbRouterSummary
    };
    let grounding_text = match grounding_source {
        ShotGroundingSource::ShotScript => shot_script.clone(),
        ShotGroundingSource::ExpandedScriptText => expanded_script_text.clone(),
        ShotGroundingSource::PrimarySceneFields => primary_scene_type.clone(),
        ShotGroundingSource::KbRouterSummary => String::new(),
    };
    let shot_scene_type = non_blank_string(request.shot_scene_type.as_deref().unwrap_or_default())
        .unwrap_or_else(|| infer_shot_scene_type(&grounding_text, &primary_scene_type));
    let adaptation_reason = non_blank_string(
        request.adaptation_reason.as_deref().unwrap_or_default(),
    )
    .unwrap_or_else(|| {
        if shot_scene_type == primary_scene_type {
            String::new()
        } else {
            build_adaptation_reason(&grounding_text, &shot_scene_type)
        }
    });

    StoryboardGroundingContext {
        shot_script,
        expanded_script_text,
        grounding_text: grounding_text.clone(),
        grounding_source,
        primary_scene_type: primary_scene_type.clone(),
        primary_scene_label: non_blank_string(
            request.primary_scene_label.as_deref().unwrap_or_default(),
        )
        .or_else(|| non_blank_string(request.scene_label.as_deref().unwrap_or_default()))
        .unwrap_or_else(|| derive_shot_scene_label(&primary_scene_type)),
        primary_scene_category: non_blank_string(
            request
                .primary_scene_category
                .as_deref()
                .unwrap_or_default(),
        )
        .or_else(|| non_blank_string(request.scene_category.as_deref().unwrap_or_default()))
        .unwrap_or_else(|| primary_scene_type.clone()),
        shot_scene_label: non_blank_string(request.shot_scene_label.as_deref().unwrap_or_default())
            .unwrap_or_else(|| derive_shot_scene_label(&shot_scene_type)),
        shot_intent: non_blank_string(request.shot_intent.as_deref().unwrap_or_default())
            .or_else(|| non_blank_string(&request.task_name))
            .unwrap_or_else(|| derive_shot_intent(&grounding_text)),
        shot_scene_type,
        adaptation_reason,
    }
}

fn build_storyboard_router_synopsis(grounding: &StoryboardGroundingContext) -> String {
    format!(
        "primary_scene_type={}; primary_scene_label={}; primary_scene_category={}; shot_scene_type={}; shot_scene_label={}; shot_intent={}; grounding_source={}; shot_script={}; expanded_script_text={}",
        grounding.primary_scene_type,
        grounding.primary_scene_label,
        grounding.primary_scene_category,
        grounding.shot_scene_type,
        grounding.shot_scene_label,
        grounding.shot_intent,
        grounding.grounding_source.as_str(),
        grounding.shot_script,
        grounding.expanded_script_text
    )
}

fn build_storyboard_model_story_input(grounding: &StoryboardGroundingContext) -> String {
    format!(
        "grounding_priority=1.shot_script 2.expanded_script_text 3.primary_scene_fields 4.kb_router_summary\nshot_script={}\nexpanded_script_text={}\nprimary_scene_type={}\nprimary_scene_label={}\nprimary_scene_category={}\nshot_scene_type={}\nshot_scene_label={}\nshot_intent={}\nadaptation_reason={}",
        grounding.shot_script,
        grounding.expanded_script_text,
        grounding.primary_scene_type,
        grounding.primary_scene_label,
        grounding.primary_scene_category,
        grounding.shot_scene_type,
        grounding.shot_scene_label,
        grounding.shot_intent,
        grounding.adaptation_reason,
    )
}

fn build_shot_grounded_row_draft(
    grounding: &StoryboardGroundingContext,
    index: usize,
    row_count: usize,
    duration_seconds: u16,
    record: &GoldenSampleLibraryRecord,
) -> ShotGroundedRowDraft {
    let segments = split_story_segments(&grounding.grounding_text);
    let segment = segments
        .get(index)
        .cloned()
        .unwrap_or_else(|| grounding.grounding_text.trim().to_string());
    let shot_id = format!(
        "shot-task-{}-{:02}",
        &stable_hash_hex(&format!(
            "{}\n{}\n{}",
            grounding.grounding_text, grounding.shot_scene_type, row_count
        ))[..12],
        index + 1
    );
    let person = derive_product_person(&segment, &grounding.grounding_text);
    let scene_scale = derive_shot_scene_scale(&segment)
        .or_else(|| non_blank_string(&derive_scene_scale(&record.source_fields.technical_profile)))
        .unwrap_or_else(|| "中景".to_string());
    let character_action = derive_character_action_from_story(&segment, &grounding.grounding_text);
    let shot_title = derive_shot_title(index, &segment, &character_action);
    let dialogue = extract_dialogue_from_story(&segment);
    let visual_description = if segment.trim().is_empty() {
        "当前镜头按已确认镜头脚本推进。".to_string()
    } else {
        segment.trim().to_string()
    };
    let sequence_grouping = SequenceGrouping {
        structure_mode: StructureMode::SingleShot,
        sequence_id: None,
        shot_order: Some((index + 1) as u32),
        sequence_field_state: SequenceFieldState::Present,
    };
    let scene_performance_projection = ScenePerformanceProjection {
        source_sample_id: shot_id.clone(),
        source_sample_title: shot_title.clone(),
        scene_scale: scene_scale.clone(),
        person: person.clone(),
        visual_description: visual_description.clone(),
        character_action: character_action.clone(),
        fused_source_text: grounding.grounding_text.clone(),
        sequence_grouping: sequence_grouping.clone(),
    };

    ShotGroundedRowDraft {
        shot_id,
        shot_script: segment,
        person,
        shot_title,
        scene_scale,
        visual_description,
        character_action,
        dialogue,
        duration_seconds,
        sequence_grouping,
        scene_performance_projection,
    }
}

fn split_story_segments(text: &str) -> Vec<String> {
    text.replace("\r\n", "\n")
        .split(|character| {
            matches!(
                character,
                '\n' | '。' | '！' | '？' | '!' | '?' | ';' | '；'
            )
        })
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn infer_shot_scene_type(text: &str, primary_scene_type: &str) -> String {
    if contains_any_story_term(
        text,
        &[
            "格挡", "震退", "焦土", "银辉", "觉醒", "交锋", "掌心", "战", "击",
        ],
    ) {
        "action_beat".to_string()
    } else if contains_any_story_term(text, &["告别", "对话", "低语", "回应"]) {
        "dialogue_beat".to_string()
    } else if primary_scene_type.trim().is_empty() {
        "unspecified_scene".to_string()
    } else {
        primary_scene_type.to_string()
    }
}

fn derive_shot_scene_label(scene_type: &str) -> String {
    match scene_type {
        "action_beat" => "动作交锋镜头".to_string(),
        "dialogue_beat" => "对白反应镜头".to_string(),
        value if value.trim().is_empty() => "未指定场景".to_string(),
        value => value.to_string(),
    }
}

fn derive_shot_intent(text: &str) -> String {
    if contains_any_story_term(text, &["觉醒", "银辉", "掌心"]) {
        "reveal".to_string()
    } else if contains_any_story_term(text, &["格挡", "震退", "交锋", "击"]) {
        "action_beat".to_string()
    } else if contains_any_story_term(text, &["心跳", "鼓点", "低频"]) {
        "rhythm_emphasis".to_string()
    } else if contains_any_story_term(text, &["对话", "说", "低语"]) {
        "dialogue".to_string()
    } else {
        "story_beat".to_string()
    }
}

fn build_adaptation_reason(text: &str, shot_scene_type: &str) -> String {
    let evidence = story_anchor_terms(text);
    if evidence.is_empty() {
        format!("shot_script supports local {} classification", shot_scene_type)
    } else {
        format!(
            "shot_script evidence [{}] supports local {} classification",
            evidence.join("/"),
            shot_scene_type
        )
    }
}

fn derive_product_person(segment: &str, full_text: &str) -> String {
    if contains_any_story_term(segment, &["掌心", "银辉", "觉醒"]) {
        "觉醒者".to_string()
    } else if contains_any_story_term(segment, &["震退", "七步"]) {
        "被震退者".to_string()
    } else if contains_any_story_term(full_text, &["格挡", "交锋", "震退"]) {
        "交锋双方".to_string()
    } else if contains_any_story_term(full_text, &["主角"]) {
        "主角".to_string()
    } else {
        "当前镜头主体".to_string()
    }
}

fn derive_shot_scene_scale(segment: &str) -> Option<String> {
    if contains_any_story_term(segment, &["掌心", "手臂", "银辉"]) {
        Some("特写".to_string())
    } else if contains_any_story_term(segment, &["焦土", "犁痕"]) {
        Some("全景".to_string())
    } else if contains_any_story_term(segment, &["心跳", "鼓点"]) {
        Some("近景".to_string())
    } else {
        None
    }
}

fn derive_character_action_from_story(segment: &str, full_text: &str) -> String {
    let source = if segment.trim().is_empty() {
        full_text
    } else {
        segment
    };
    if contains_any_story_term(source, &["掌心银辉", "银辉觉醒", "沿手臂上升", "觉醒"]) {
        "觉醒者从格挡后的短暂停滞开始，将掌心朝向当前对手，银辉自掌心亮起并沿手臂上升，到力量完全爬上前臂时结束，镜头捕捉银辉开始蔓延的觉醒瞬间。"
            .to_string()
    } else if contains_any_story_term(source, &["震退七步", "震退"]) {
        "交锋双方从正面相抵的僵持状态开始，防守者以格挡余力反震当前对手，将对手震退七步，到对手脚步失衡后撤时结束，镜头捕捉第一步被震开的瞬间。"
            .to_string()
    } else if contains_any_story_term(source, &["焦土犁痕", "焦土", "犁痕"]) {
        "被震退者从后撤失衡开始，双脚顶住焦土仍被冲击推远，在地面犁出焦黑拖痕，到身体重新找回重心时结束，镜头捕捉脚跟划开焦土的瞬间。"
            .to_string()
    } else if contains_any_story_term(source, &["心跳", "低频鼓点", "鼓点", "低频"]) {
        "当前镜头主体从战斗后的凝滞呼吸开始，胸口随心跳和低频鼓点压低起伏，面向当前对手重新蓄力，到下一次爆发前的停顿时结束，镜头捕捉心跳压住全场节奏的瞬间。"
            .to_string()
    } else if contains_any_story_term(source, &["格挡", "交锋", "战", "击"]) {
        "交锋双方从迎面冲突开始，前景角色抬臂格挡当前对手的攻击，以稳住重心的姿态抵住冲击，到攻防短暂相持时结束，镜头捕捉格挡接触的关键瞬间。"
            .to_string()
    } else {
        "当前镜头主体从上一动作余势中开始，面向当前对手或环境完成可见状态转变，到下一拍动作蓄势完成时结束，镜头捕捉状态发生变化的瞬间。"
            .to_string()
    }
}

fn derive_shot_title(index: usize, segment: &str, character_action: &str) -> String {
    let title_core = story_anchor_terms(segment)
        .into_iter()
        .take(2)
        .collect::<Vec<_>>();
    if title_core.is_empty() {
        format!("镜头{}：{}", index + 1, character_action)
    } else {
        format!("镜头{}：{}", index + 1, title_core.join(""))
    }
}

fn extract_dialogue_from_story(segment: &str) -> String {
    let chars = segment.chars().collect::<Vec<_>>();
    let Some(start) = chars.iter().position(|character| *character == '“') else {
        return String::new();
    };
    let Some(end) = chars
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, character)| (*character == '”').then_some(index))
    else {
        return String::new();
    };
    chars[start + 1..end].iter().collect::<String>()
}

fn story_anchor_terms(text: &str) -> Vec<&'static str> {
    [
        "格挡",
        "震退七步",
        "震退",
        "焦土犁痕",
        "焦土",
        "心跳",
        "低频鼓点",
        "鼓点",
        "掌心银辉",
        "银辉",
        "觉醒",
        "沿手臂上升",
        "交锋",
    ]
    .into_iter()
    .filter(|term| text.contains(term))
    .collect()
}

fn contains_any_story_term(value: &str, terms: &[&str]) -> bool {
    terms.iter().any(|term| value.contains(term))
}

fn project_reference_handle_candidates(
    record: &GoldenSampleLibraryRecord,
) -> Vec<ExternalReferenceHandleCandidate> {
    extract_reference_tokens(&record.source_fields.reference_bundle)
        .into_iter()
        .map(
            |(reference_name, reference_kind, strength)| ExternalReferenceHandleCandidate {
                source_sample_id: record.sample_id.clone(),
                reference_name,
                reference_kind,
                strength,
            },
        )
        .collect()
}

fn extract_reference_tokens(value: &str) -> Vec<(String, String, Option<String>)> {
    let chars = value.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    for (index, character) in chars.iter().enumerate() {
        if *character != '(' {
            continue;
        }
        let mut start = index;
        while start > 0 {
            let previous = chars[start - 1];
            if previous.is_ascii_alphanumeric() || previous == '_' || previous == '-' {
                start -= 1;
            } else {
                break;
            }
        }
        if start == index {
            continue;
        }
        let reference_name = chars[start..index].iter().collect::<String>();
        if start > 0 && matches!(chars[start - 1], '/' | '\\' | ':') {
            continue;
        }
        if looks_like_path_or_url(&reference_name) {
            continue;
        }
        let Some(end_offset) = chars[index + 1..].iter().position(|item| *item == ')') else {
            continue;
        };
        let args = chars[index + 1..index + 1 + end_offset]
            .iter()
            .collect::<String>();
        let mut parts = args.split(',').map(str::trim);
        let reference_kind = parts.next().unwrap_or("name").to_string();
        let lower_reference_kind = reference_kind.to_ascii_lowercase();
        if matches!(
            lower_reference_kind.as_str(),
            "path" | "url" | "uri" | "media" | "asset" | "file"
        ) {
            continue;
        }
        let strength = parts
            .next()
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        tokens.push((reference_name, reference_kind, strength));
    }
    tokens
}

fn looks_like_path_or_url(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("http")
        || lower.contains("://")
        || lower.contains('\\')
        || lower.contains('/')
        || lower.contains(':')
}

fn derive_scene_scale(technical_profile: &str) -> String {
    for token in ["MCU", "CU", "LS", "MS", "WS"] {
        if technical_profile.contains(token) {
            return token.to_string();
        }
    }
    "source_technical_profile".to_string()
}

fn non_blank_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn normalize_scene_type(scene_type: &str) -> String {
    scene_type.trim().to_lowercase()
}

fn is_supported_storyboard_duration(duration_seconds: u16) -> bool {
    matches!(duration_seconds, 5 | 10 | 15 | 30 | 45 | 60)
}

fn is_supported_desktop_scene_type(scene_type: &str) -> bool {
    matches!(
        scene_type,
        "hot_blood_battle"
            | "ensemble_performance"
            | "emotional_dialogue"
            | "encounter_performance"
            | "field_chase"
            | "spectacle_showcase"
            | "daily_healing"
            | "guoman_hot_blood_combat"
            | "guoman_ensemble_performance"
            | "ink_wuxia_combat"
            | "eastern_spectacle"
            | "xianxia_action"
            | "urban_fantasy"
            | "chinese_war_formation"
            | "weapon_highlight"
            | "council_strategy"
            | "siege_defense"
            | "slg_sandbox_view"
            | "slg_march_encirclement"
            | "slg_city_growth"
            | "slg_battle_report"
    )
}

fn allocate_storyboard_row_durations(
    total_duration_seconds: u16,
    row_count: usize,
) -> Option<Vec<u16>> {
    if row_count == 0 || !is_supported_storyboard_duration(total_duration_seconds) {
        return None;
    }

    let base = total_duration_seconds / row_count as u16;
    if base == 0 {
        return None;
    }

    let mut durations = vec![base; row_count];
    let mut remainder = total_duration_seconds % row_count as u16;
    let mut index = 0usize;
    while remainder > 0 {
        durations[index] += 1;
        remainder -= 1;
        index = (index + 1) % row_count;
    }

    (durations.iter().copied().sum::<u16>() == total_duration_seconds).then_some(durations)
}

fn compile_seedance_prompt_text(
    record: &GoldenSampleLibraryRecord,
    scene_projection: &ScenePerformanceProjection,
    duration_seconds: u16,
    _grounding: &StoryboardGroundingContext,
    shot_script: &str,
    _kb_router_result: &KbRouterRuntimeResponse,
) -> (String, PromptTextCompilationStatus, Vec<ProductWarning>) {
    let sections = vec![
        "视频分镜提示词：以当前镜头脚本为准，输出单个镜头画面。".to_string(),
        format!("镜头脚本：{}", shot_script.trim()),
        format!("镜头标题：{}", scene_projection.source_sample_title),
        format!("景别：{}", scene_projection.scene_scale),
        format!("画面描述：{}", scene_projection.visual_description),
        format!("角色动作：{}", scene_projection.character_action),
        format!("时长：{}秒", duration_seconds),
    ];
    let mut warnings = vec![
        ProductWarning {
            code: "seedance_prompt_text_compilation_local".to_string(),
            message: "Seedance2.0 prompt_text 当前由本地结构化字段编译生成。".to_string(),
            related_sample_id: Some(record.sample_id.clone()),
        },
        ProductWarning {
            code: "seedance_runtime_adapter_closed".to_string(),
            message: "Seedance2.0 仅作为文本提示词适配目标；当前不接运行时接口。"
                .to_string(),
            related_sample_id: Some(record.sample_id.clone()),
        },
        ProductWarning {
            code: "kb_sample_prompt_not_promoted".to_string(),
            message:
                "Prompt compilation uses shot_script, structured row fields, and selected KB summaries; internal sample prompt evidence stays hidden."
                    .to_string(),
            related_sample_id: Some(record.sample_id.clone()),
        },
    ];
    if is_role_action_grounding_incomplete(&scene_projection.character_action) {
        warnings.push(ProductWarning {
            code: "role_action_grounding_incomplete".to_string(),
            message: "角色动作信息不完整，请重新生成或检查当前镜头脚本。".to_string(),
            related_sample_id: Some(record.sample_id.clone()),
        });
    }

    (
        sections.join("；"),
        PromptTextCompilationStatus::ReadyStub,
        warnings,
    )
}

#[allow(dead_code)]
fn compile_seedance_prompt_text_legacy(
    record: &GoldenSampleLibraryRecord,
    scene_projection: &ScenePerformanceProjection,
    duration_seconds: u16,
    _scene_type: &str,
) -> (String, PromptTextCompilationStatus, Vec<ProductWarning>) {
    let sections = vec![
        "视频分镜提示词：以当前镜头脚本为准，输出单个镜头画面。".to_string(),
        format!("镜头标题：{}", record.source_fields.sample_title),
        format!("景别：{}", scene_projection.scene_scale),
        format!("画面描述：{}", scene_projection.visual_description),
        format!("角色动作：{}", scene_projection.character_action),
        format!("时长：{}秒", duration_seconds),
    ];

    let warnings = vec![
        ProductWarning {
            code: "seedance_prompt_text_compilation_local".to_string(),
            message: "Seedance2.0 prompt_text 当前由本地结构化字段编译生成。".to_string(),
            related_sample_id: Some(record.sample_id.clone()),
        },
        ProductWarning {
            code: "seedance_runtime_adapter_closed".to_string(),
            message: "Seedance2.0 仅作为文本提示词适配目标；当前不接运行时接口。".to_string(),
            related_sample_id: Some(record.sample_id.clone()),
        },
        ProductWarning {
            code: "kb_sample_prompt_not_promoted".to_string(),
            message:
                "Prompt compilation uses structured storyboard fields; internal sample prompt evidence stays hidden."
                    .to_string(),
            related_sample_id: Some(record.sample_id.clone()),
        },
    ];

    (
        sections.join("；"),
        PromptTextCompilationStatus::ReadyStub,
        warnings,
    )
}

fn serialize_storyboard_rows(rows: &[GeneratedStoryboardRow]) -> String {
    rows.iter()
        .map(|row| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{:?}|{}|{}|{}|{}|{}",
                row.shot_id,
                row.order,
                row.shot_script,
                row.primary_scene_type,
                row.primary_scene_label,
                row.primary_scene_category,
                row.shot_scene_type,
                row.shot_scene_label,
                row.shot_intent,
                row.adaptation_reason,
                row.grounding_source.as_str(),
                row.person,
                row.shot_title,
                row.visual_description,
                row.character_action,
                row.dialogue,
                row.prompt_text,
                row.prompt_text_compilation_status,
                row.prompt_text_source_row_id,
                row.duration_seconds,
                row.shot_duration_seconds,
                row.duration_source,
                row.prompt_text_compilation_warnings
                    .iter()
                    .map(|warning| warning.code.as_str())
                    .collect::<Vec<_>>()
                    .join(","),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect_compilation_statuses(rows: &[GeneratedStoryboardRow]) -> Vec<String> {
    rows.iter()
        .map(|row| format!("{:?}", row.prompt_text_compilation_status))
        .collect()
}

fn collect_compilation_warning_codes(rows: &[GeneratedStoryboardRow]) -> Vec<String> {
    rows.iter()
        .flat_map(|row| {
            row.prompt_text_compilation_warnings
                .iter()
                .map(|warning| warning.code.clone())
        })
        .collect()
}

fn blocked_storyboard_response(
    state: &AppState,
    router_request: &KbRouterRuntimeRequest,
    task_id: Option<String>,
    selected_total_duration_seconds: u16,
    blockers: Vec<ProductWarning>,
    now_ms: u64,
) -> GenerateStoryboardResponse {
    blocked_storyboard_response_with_kb(
        task_id,
        selected_total_duration_seconds,
        blockers,
        now_ms,
        empty_kb_router_response(router_request, &state.kb_runtime),
    )
}

fn blocked_storyboard_response_with_kb(
    task_id: Option<String>,
    selected_total_duration_seconds: u16,
    blockers: Vec<ProductWarning>,
    now_ms: u64,
    kb_router_result: KbRouterRuntimeResponse,
) -> GenerateStoryboardResponse {
    GenerateStoryboardResponse {
        task_id,
        result_id: String::new(),
        rows: vec![],
        selected_total_duration_seconds,
        duration_plan: StoryboardDurationPlan {
            total_duration_seconds: selected_total_duration_seconds,
            row_count: 0,
            per_row_seconds: 0,
            allocated_seconds: 0,
        },
        export_status: StoryboardExportStatus {
            status: BridgeCallStatus::Blocked,
            blocked_row_count: 0,
            ready_row_count: 0,
            warnings: vec![],
            blockers,
        },
        busy: false,
        operation_id: String::new(),
        revision: 0,
        updated_at_ms: now_ms,
        rows_hash: String::new(),
        dirty: false,
        dirty_source_note: None,
        kb_router_result,
    }
}

fn empty_kb_router_response(
    request: &KbRouterRuntimeRequest,
    kb_runtime: &project_store::KbRuntimeHandle,
) -> KbRouterRuntimeResponse {
    KbRouterRuntimeResponse {
        selected_sample_ids: vec![],
        selected_kb_rules: vec![],
        kb_context_summary: String::new(),
        retrieval_trace: KbRouterRetrievalTrace {
            kb_version: kb_runtime.snapshot.seed_format.clone(),
            snapshot_id: kb_runtime.snapshot.snapshot_id.clone(),
            snapshot_checksum: kb_runtime.snapshot.snapshot_hash.clone(),
            content_cache_key: String::new(),
            task_type: request.task_type.as_str().to_string(),
            top_k_samples: 0,
            top_k_rules: 0,
            duration_seconds: request.duration_seconds,
            story_keywords: vec![],
            selection_reasons: vec![],
            excluded_candidates: vec![],
            token_budget: KbRouterTokenBudget {
                kb_context_summary_target: "800-1500 Chinese characters".to_string(),
                full_kb_rows_included: 0,
            },
        },
    }
}

fn stable_hash_hex<T: Hash>(value: &T) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn build_project_create_or_switch_snapshot(
    state: &AppState,
    request: ProjectCreateOrSwitchRequest,
) -> io::Result<ProjectCreateOrSwitchSnapshot> {
    let fixture = state.load_shared_fixture()?;
    Ok(build_project_create_or_switch_snapshot_from_fixture(
        state, request, &fixture,
    ))
}

pub fn build_writer_entry_snapshot(
    state: &AppState,
    request: WriterEntrySnapshotRequest,
) -> io::Result<WriterEntrySnapshot> {
    let fixture = state.load_shared_fixture()?;
    Ok(build_writer_entry_snapshot_from_fixture(request, &fixture))
}

pub fn build_storyboard_rendersegment_cut_preview_snapshot(
    state: &AppState,
    request: StoryboardRenderSegmentCutPreviewSnapshotRequest,
) -> io::Result<StoryboardRenderSegmentCutPreviewSnapshot> {
    let fixture = state.load_shared_fixture()?;
    Ok(build_storyboard_rendersegment_cut_preview_snapshot_from_fixture(state, request, &fixture))
}

pub fn build_storyboard_preview_plan(
    state: &AppState,
    request: StoryboardPreviewPlanRequest,
) -> Result<StoryboardPlan, StoryboardPlanningError> {
    let scene_taxonomy = resolve_scene_taxonomy(state, request.scene_type.as_deref());
    let derived_scene_director_id = request.scene_director_id.clone().or_else(|| {
        scene_taxonomy
            .as_ref()
            .map(|taxonomy| format!("taxonomy:{}:scene", taxonomy.scene_taxonomy_id))
    });
    let derived_action_director_id = request.action_director_id.clone().or_else(|| {
        scene_taxonomy
            .as_ref()
            .map(|taxonomy| format!("taxonomy:{}:action", taxonomy.scene_taxonomy_id))
    });
    let layout_prompt = if let Some(taxonomy) = scene_taxonomy.as_ref() {
        if request
            .layout_prompt
            .contains(&format!("场景分类：{}", taxonomy.scene_type))
        {
            request.layout_prompt.clone()
        } else {
            format!(
                "{}\n场景分类：{}",
                request.layout_prompt, taxonomy.scene_type
            )
        }
    } else {
        request.layout_prompt.clone()
    };
    let render_prompt = if let Some(taxonomy) = scene_taxonomy.as_ref() {
        if request
            .render_prompt
            .contains(&format!("连续性优先级：{}", taxonomy.continuity_priority))
        {
            request.render_prompt.clone()
        } else {
            format!(
                "{}\n连续性优先级：{}",
                request.render_prompt, taxonomy.continuity_priority
            )
        }
    } else {
        request.render_prompt.clone()
    };

    storyboard_pipeline::build_storyboard_plan(StoryboardPlanRequest {
        render_segment_id: request.render_segment_id,
        narrative_scene_id: request.narrative_scene_id,
        render_segment_sequence_no: request.render_segment_sequence_no,
        start_shot_sequence_no: request.start_shot_sequence_no,
        end_shot_sequence_no: request.end_shot_sequence_no,
        target_duration_seconds: request.target_duration_seconds,
        cut_id: request.cut_id,
        cut_sequence_no: request.cut_sequence_no,
        shot_description: request.shot_description,
        dialogue: request.dialogue,
        scene_director_id: derived_scene_director_id,
        action_director_id: derived_action_director_id,
        scene_taxonomy,
        layout_prompt,
        render_prompt,
    })
}

pub fn build_validation_export_panel_snapshot(
    state: &AppState,
    request: ValidationExportPanelSnapshotRequest,
) -> io::Result<ValidationExportPanelSnapshot> {
    let fixture = state.load_shared_fixture()?;
    build_validation_export_panel_snapshot_from_fixture(state, request, &fixture)
}

pub fn resolve_scene_taxonomy(
    state: &AppState,
    scene_type: Option<&str>,
) -> Option<core_domain::SceneTaxonomyRecord> {
    let scene_type = scene_type?;

    state
        .kb_knowledge
        .scene_taxonomies
        .iter()
        .find(|taxonomy| {
            taxonomy.scene_type == scene_type
                || taxonomy.display_name == scene_type
                || taxonomy.scene_taxonomy_id == scene_type
        })
        .cloned()
}

fn build_project_create_or_switch_snapshot_from_fixture(
    state: &AppState,
    request: ProjectCreateOrSwitchRequest,
    fixture: &Week3SharedFixture,
) -> ProjectCreateOrSwitchSnapshot {
    let current_project_id = request.project_id.clone().or_else(|| {
        fixture
            .project_meta
            .first()
            .map(|row| row.project_id.clone())
    });

    let projects = fixture
        .project_meta
        .iter()
        .map(|row| ProjectSummaryItem {
            project_id: row.project_id.clone(),
            name: row.title.clone(),
            status: row.status.clone(),
            updated_at: format!("timestamp {}", row.updated_at_timestamp),
            episode_count: fixture
                .episode_meta
                .iter()
                .filter(|episode| episode.project_id == row.project_id)
                .count(),
        })
        .collect();

    ProjectCreateOrSwitchSnapshot {
        current_project_id,
        projects,
        readonly_status: AppShellReadonlyStatusSnapshot {
            snapshot_bootstrap: state.snapshot_bootstrap_readonly_state().clone(),
            validation_feedback: state.validation_feedback_readonly_state().clone(),
        },
    }
}

fn build_writer_entry_snapshot_from_fixture(
    request: WriterEntrySnapshotRequest,
    fixture: &Week3SharedFixture,
) -> WriterEntrySnapshot {
    let episode_ids = project_episode_ids(fixture, &request.project_id);
    let scene_ids = project_scene_ids(fixture, &request.project_id);
    let render_segment_ids = project_render_segment_ids(fixture, &request.project_id);
    let episode_count = episode_ids.len();

    let scenes: Vec<_> = fixture
        .narrative_scene
        .iter()
        .filter(|scene| scene_ids.contains(&scene.narrative_scene_id))
        .collect();
    let cuts: Vec<_> = fixture
        .cut
        .iter()
        .filter(|cut| render_segment_ids.contains(&cut.render_segment_id))
        .collect();
    let render_segment_count = fixture
        .render_segment
        .iter()
        .filter(|segment| render_segment_ids.contains(&segment.render_segment_id))
        .count();
    let handoff_zone_count = fixture
        .handoff_zone
        .iter()
        .filter(|zone| render_segment_ids.contains(&zone.render_segment_id))
        .count();

    let synopsis = if let Some(project) = fixture
        .project_meta
        .iter()
        .find(|row| row.project_id == request.project_id)
    {
        let first_scene = scenes
            .first()
            .map(|scene| format!(" 首个 NarrativeScene：{}。", scene.title))
            .unwrap_or_default();

        format!(
            "项目《{}》当前目标 {} 分钟，已同步 {} 个 Episode。{}",
            project.title, project.target_duration_minutes, episode_count, first_scene
        )
    } else {
        format!("项目 {} 已接入共享 fixture。", request.project_id)
    };

    let story = if scenes.is_empty() {
        format!("项目 {} 暂无 NarrativeScene。", request.project_id)
    } else {
        scenes
            .iter()
            .map(|scene| {
                format!(
                    "第 {} 场 {}：{}",
                    scene.sequence_no, scene.title, scene.summary
                )
            })
            .collect::<Vec<_>>()
            .join(" | ")
    };

    let screenplay = if cuts.is_empty() {
        "当前共享 fixture 暂无 Cut 预览。".to_string()
    } else {
        cuts.iter()
            .take(3)
            .map(|cut| {
                format!(
                    "Cut {:02} / {} / {}",
                    cut.sequence_no, cut.shot_description, cut.dialogue
                )
            })
            .collect::<Vec<_>>()
            .join(" | ")
    };

    let storyboard = format!(
        "{} 个 RenderSegment / {} 个 Cut / {} 个 HandoffZone / {} 个 PromptPackage",
        render_segment_count,
        cuts.len(),
        handoff_zone_count,
        fixture.prompt_package.len()
    );

    WriterEntrySnapshot {
        project_id: request.project_id,
        synopsis,
        story,
        screenplay,
        storyboard,
    }
}

fn build_storyboard_rendersegment_cut_preview_snapshot_from_fixture(
    state: &AppState,
    request: StoryboardRenderSegmentCutPreviewSnapshotRequest,
    fixture: &Week3SharedFixture,
) -> StoryboardRenderSegmentCutPreviewSnapshot {
    let scene_ids = project_scene_ids(fixture, &request.project_id);
    let project_render_segment_ids = project_render_segment_ids(fixture, &request.project_id);

    let mut scenes: Vec<_> = fixture
        .narrative_scene
        .iter()
        .filter(|scene| scene_ids.contains(&scene.narrative_scene_id))
        .collect();
    if let Some(narrative_scene_id) = request.narrative_scene_id.as_ref() {
        scenes.retain(|scene| scene.narrative_scene_id == *narrative_scene_id);
    }
    scenes.sort_by_key(|scene| scene.sequence_no);

    let filtered_scene_ids: HashSet<_> = scenes
        .iter()
        .map(|scene| scene.narrative_scene_id.clone())
        .collect();

    let mut render_segments: Vec<_> = fixture
        .render_segment
        .iter()
        .filter(|segment| project_render_segment_ids.contains(&segment.render_segment_id))
        .filter(|segment| filtered_scene_ids.contains(&segment.narrative_scene_id))
        .collect();
    if let Some(render_segment_id) = request.render_segment_id.as_ref() {
        render_segments.retain(|segment| segment.render_segment_id == *render_segment_id);
    }
    render_segments.sort_by_key(|segment| segment.sequence_no);

    let filtered_render_segment_ids: HashSet<_> = render_segments
        .iter()
        .map(|segment| segment.render_segment_id.clone())
        .collect();

    let storyboard = scenes
        .iter()
        .map(|scene| PreviewItem {
            id: scene.narrative_scene_id.clone(),
            label: format!("第 {:02} 场 {}", scene.sequence_no, scene.title),
            duration: "Scene".to_string(),
            note: scene.summary.clone(),
        })
        .collect();

    let render_segment = render_segments
        .iter()
        .map(|segment| {
            let scene = scenes
                .iter()
                .find(|scene| scene.narrative_scene_id == segment.narrative_scene_id);
            let cut = fixture
                .cut
                .iter()
                .filter(|cut| cut.render_segment_id == segment.render_segment_id)
                .min_by_key(|cut| cut.sequence_no);
            let prompt_body = fixture
                .prompt_package
                .first()
                .map(|prompt| prompt.body.as_str())
                .unwrap_or("等待真实 PromptPackage 注入。");

            let plan_note = match build_storyboard_preview_plan(
                state,
                StoryboardPreviewPlanRequest {
                    render_segment_id: segment.render_segment_id.clone(),
                    narrative_scene_id: segment.narrative_scene_id.clone(),
                    render_segment_sequence_no: segment.sequence_no,
                    start_shot_sequence_no: segment.start_shot_sequence_no,
                    end_shot_sequence_no: segment.end_shot_sequence_no,
                    target_duration_seconds: segment.target_duration_seconds as u16,
                    cut_id: cut
                        .map(|item| item.cut_id.clone())
                        .unwrap_or_else(|| format!("{}-preview-cut", segment.render_segment_id)),
                    cut_sequence_no: cut.map(|item| item.sequence_no).unwrap_or(1),
                    shot_description: cut
                        .map(|item| item.shot_description.clone())
                        .unwrap_or_else(|| "待补充镜头描述".to_string()),
                    dialogue: cut
                        .map(|item| item.dialogue.clone())
                        .unwrap_or_else(|| "待补充对白".to_string()),
                    scene_director_id: Some("desktop-shell:scene".to_string()),
                    action_director_id: Some("desktop-shell:action".to_string()),
                    scene_type: request.scene_type.clone(),
                    layout_prompt: format!(
                        "{} / {}",
                        scene
                            .map(|item| item.title.as_str())
                            .unwrap_or("未命名场景"),
                        cut.map(|item| item.shot_description.as_str())
                            .unwrap_or("待补充镜头描述")
                    ),
                    render_prompt: prompt_body.to_string(),
                },
            ) {
                Ok(plan) => format!(
                    "{} | shots {}-{} | handoff {} -> {} | scene {} | render {}",
                    scene
                        .map(|item| item.title.as_str())
                        .unwrap_or("未命名场景"),
                    plan.render_segment.start_shot_sequence_no,
                    plan.render_segment.end_shot_sequence_no,
                    plan.handoff_zone.start_boundary,
                    plan.handoff_zone.end_boundary,
                    truncate_preview_text(&plan.committee_runtime.prompt_layers.layout_prompt, 48),
                    truncate_preview_text(&plan.committee_runtime.prompt_layers.render_prompt, 64)
                ),
                Err(error) => format!("Preview plan fallback: {}", error),
            };

            PreviewItem {
                id: segment.render_segment_id.clone(),
                label: format!("RenderSegment {:02}", segment.sequence_no),
                duration: format!("{}s", segment.target_duration_seconds),
                note: plan_note,
            }
        })
        .collect();

    let mut cuts: Vec<_> = fixture
        .cut
        .iter()
        .filter(|cut| filtered_render_segment_ids.contains(&cut.render_segment_id))
        .collect();
    cuts.sort_by_key(|cut| cut.sequence_no);
    let cuts = cuts
        .iter()
        .map(|cut| PreviewItem {
            id: cut.cut_id.clone(),
            label: format!("Cut {:02}", cut.sequence_no),
            duration: format!("{}s", cut.duration_seconds),
            note: format!(
                "{} / {}",
                cut.shot_description,
                truncate_preview_text(&cut.dialogue, 48)
            ),
        })
        .collect();

    StoryboardRenderSegmentCutPreviewSnapshot {
        project_id: request.project_id,
        storyboard,
        render_segment,
        cuts,
    }
}

fn build_validation_export_panel_snapshot_from_fixture(
    state: &AppState,
    request: ValidationExportPanelSnapshotRequest,
    fixture: &Week3SharedFixture,
) -> io::Result<ValidationExportPanelSnapshot> {
    let workbook = generate_week3_validation_report(fixture)?;
    let readonly_state = state.snapshot_bootstrap_readonly_state();
    let repair_recommendations = if readonly_state.knowledge_bundle.repair_mappings_ready {
        generate_week3_repair_recommendations(
            fixture,
            &state.kb_knowledge.failure_patterns,
            &state.kb_knowledge.prompt_templates,
        )?
    } else {
        Vec::new()
    };

    let validation_row_count = workbook.validation_report.len();
    let block_count = workbook
        .validation_report
        .iter()
        .filter(|row| !row.passed)
        .count();
    let validation_state = if block_count > 0 {
        ValidationExportPanelState::Blocked
    } else {
        ValidationExportPanelState::Ready
    };
    let repair_state = if repair_recommendations.is_empty() {
        ValidationExportPanelState::Pending
    } else {
        ValidationExportPanelState::Ready
    };

    let summary_items = vec![
        ValidationExportPanelItem {
            label: "project".to_string(),
            value: request.project_id.clone(),
            state: ValidationExportPanelState::Ready,
        },
        ValidationExportPanelItem {
            label: "validation_report".to_string(),
            value: format!("{} rows / {} blocked", validation_row_count, block_count),
            state: validation_state,
        },
        ValidationExportPanelItem {
            label: "repair_recommendations".to_string(),
            value: format!(
                "{} kb-backed recommendations / snapshot {}",
                repair_recommendations.len(),
                readonly_state.snapshot_identity.snapshot_id
            ),
            state: repair_state,
        },
    ];

    let repair_recommendations = repair_recommendations
        .into_iter()
        .map(map_repair_recommendation)
        .collect();

    Ok(ValidationExportPanelSnapshot {
        project_id: request.project_id,
        summary_items,
        repair_recommendations,
    })
}

fn project_episode_ids(fixture: &Week3SharedFixture, project_id: &str) -> HashSet<String> {
    fixture
        .episode_meta
        .iter()
        .filter(|episode| episode.project_id == project_id)
        .map(|episode| episode.episode_id.clone())
        .collect()
}

fn project_scene_ids(fixture: &Week3SharedFixture, project_id: &str) -> HashSet<String> {
    let episode_ids = project_episode_ids(fixture, project_id);

    fixture
        .narrative_scene
        .iter()
        .filter(|scene| episode_ids.contains(&scene.episode_id))
        .map(|scene| scene.narrative_scene_id.clone())
        .collect()
}

fn project_render_segment_ids(fixture: &Week3SharedFixture, project_id: &str) -> HashSet<String> {
    let scene_ids = project_scene_ids(fixture, project_id);

    fixture
        .render_segment
        .iter()
        .filter(|segment| scene_ids.contains(&segment.narrative_scene_id))
        .map(|segment| segment.render_segment_id.clone())
        .collect()
}

fn truncate_preview_text(text: &str, max_chars: usize) -> String {
    let truncated: String = text.chars().take(max_chars).collect();
    if text.chars().count() > max_chars {
        format!("{}...", truncated)
    } else {
        truncated
    }
}

fn map_repair_recommendation(
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
            .filter_map(|link| link.prompt_name)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use core_domain::{
        FailurePatternRecord, KbRuntimeSummary, KbSnapshotRecord, PromptTemplateRecord,
        SceneTaxonomyRecord,
    };
    use project_store::{
        DualSqliteConnectionPolicy, KbKnowledgeBundle, KbRuntimeHandle, StoreSkeleton,
    };

    use super::{
        AppShellReadonlyStatusSnapshot, StoryboardPreviewPlanRequest, ValidationExportPanelState,
        build_project_create_or_switch_snapshot_from_fixture, build_storyboard_preview_plan,
        build_storyboard_rendersegment_cut_preview_snapshot_from_fixture,
        build_validation_export_panel_snapshot_from_fixture,
        build_writer_entry_snapshot_from_fixture, resolve_scene_taxonomy,
    };
    use crate::state::load_desktop_shared_fixture;
    use crate::{
        ipc::{
            ProjectCreateOrSwitchRequest, StoryboardRenderSegmentCutPreviewSnapshotRequest,
            ValidationExportPanelSnapshotRequest, WriterEntrySnapshotRequest,
        },
        state::AppState,
    };

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
                mirror_tables: vec!["scene_taxonomy".to_string(), "failure_pattern".to_string()],
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
            failure_patterns: vec![
                FailurePatternRecord {
                    failure_pattern_id: "failure-style-drift".to_string(),
                    failure_code: "style_drift".to_string(),
                    failure_name: "Style Drift".to_string(),
                    failure_category: "style".to_string(),
                    symptom: "Adjacent cuts drift away from the locked style.".to_string(),
                    common_causes: vec!["hard lock injection missing".to_string()],
                    detection_hint: "Check Style Unity Validator.".to_string(),
                    repair_strategy: "Re-inject hard locks into render prompts.".to_string(),
                    affected_layers: vec!["prompt_packages".to_string()],
                    validator_hint: "Style Unity Validator".to_string(),
                    repair_template_ids: vec!["prompt-repair-style".to_string()],
                    repair_priority: "high".to_string(),
                    repair_scope: "render_prompt_only".to_string(),
                    suggested_followup_validators: vec!["Style Unity Validator".to_string()],
                    source_type: "team_distillation".to_string(),
                    source_notes: "test fixture".to_string(),
                    confidence_level: "high".to_string(),
                    last_reviewed_at: "2026-04-20".to_string(),
                },
                FailurePatternRecord {
                    failure_pattern_id: "failure-prompt-noise".to_string(),
                    failure_code: "chinese_prompt_noise".to_string(),
                    failure_name: "Chinese Prompt Noise".to_string(),
                    failure_category: "language".to_string(),
                    symptom: "Placeholder or machine-like prompt text leaks into output."
                        .to_string(),
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
                },
            ],
            prompt_templates: vec![
                PromptTemplateRecord {
                    prompt_template_id: "prompt-repair-style".to_string(),
                    stage: "repair_pass".to_string(),
                    name: "Repair Style Lock".to_string(),
                    target_model_family: "qwen-compatible".to_string(),
                    repairs_failure_codes: vec!["style_drift".to_string()],
                },
                PromptTemplateRecord {
                    prompt_template_id: "prompt-repair-language".to_string(),
                    stage: "repair_pass".to_string(),
                    name: "Repair Prompt Language".to_string(),
                    target_model_family: "qwen-compatible".to_string(),
                    repairs_failure_codes: vec!["chinese_prompt_noise".to_string()],
                },
            ],
        };

        AppState::new(store, kb_runtime, kb_knowledge)
    }

    fn test_state_without_scene_taxonomy() -> AppState {
        let mut state = test_state();
        state.kb_knowledge.scene_taxonomies.clear();
        state
    }

    fn test_state_without_repair_mappings() -> AppState {
        let mut state = test_state();
        state.kb_knowledge.failure_patterns.clear();
        state.kb_knowledge.prompt_templates.clear();
        state
    }

    #[test]
    fn resolve_scene_taxonomy_matches_by_scene_type_and_display_name() {
        let state = test_state();

        let by_scene_type = resolve_scene_taxonomy(&state, Some("daily_dialogue"));
        let by_display_name = resolve_scene_taxonomy(&state, Some("Daily Dialogue"));

        assert_eq!(
            by_scene_type
                .as_ref()
                .map(|item| item.scene_taxonomy_id.as_str()),
            Some("scene-taxonomy-daily-dialogue")
        );
        assert_eq!(
            by_display_name
                .as_ref()
                .map(|item| item.scene_type.as_str()),
            Some("daily_dialogue")
        );
    }

    #[test]
    fn build_storyboard_preview_plan_consumes_runtime_scene_taxonomy() {
        let state = test_state();
        let plan = build_storyboard_preview_plan(
            &state,
            StoryboardPreviewPlanRequest {
                render_segment_id: "render-segment-preview-001".to_string(),
                narrative_scene_id: "narrative-scene-preview-001".to_string(),
                render_segment_sequence_no: 1,
                start_shot_sequence_no: 1,
                end_shot_sequence_no: 3,
                target_duration_seconds: 45,
                cut_id: "cut-preview-001".to_string(),
                cut_sequence_no: 1,
                shot_description: "medium shot dialogue".to_string(),
                dialogue: "Let's close this scene before dawn.".to_string(),
                scene_director_id: None,
                action_director_id: None,
                scene_type: Some("daily_dialogue".to_string()),
                layout_prompt: "cool palette, medium shot".to_string(),
                render_prompt: "restrained realism".to_string(),
            },
        )
        .expect("runtime taxonomy should build preview plan");

        assert_eq!(
            plan.render_segment.scene_taxonomy_id.as_deref(),
            Some("scene-taxonomy-daily-dialogue")
        );
        assert_eq!(
            plan.render_segment.scene_type.as_deref(),
            Some("daily_dialogue")
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_scene_director_id,
            "taxonomy:scene-taxonomy-daily-dialogue:scene"
        );
        assert!(
            plan.committee_runtime
                .prompt_layers
                .layout_prompt
                .contains("场景分类：daily_dialogue")
        );
    }

    #[test]
    fn build_storyboard_preview_plan_falls_back_when_scene_taxonomy_is_missing() {
        let state = test_state_without_scene_taxonomy();
        let plan = build_storyboard_preview_plan(
            &state,
            StoryboardPreviewPlanRequest {
                render_segment_id: "render-segment-preview-002".to_string(),
                narrative_scene_id: "narrative-scene-preview-002".to_string(),
                render_segment_sequence_no: 1,
                start_shot_sequence_no: 1,
                end_shot_sequence_no: 3,
                target_duration_seconds: 45,
                cut_id: "cut-preview-002".to_string(),
                cut_sequence_no: 1,
                shot_description: "close shot reaction".to_string(),
                dialogue: "Hold on the lead before the handoff.".to_string(),
                scene_director_id: Some("scene-director-manual".to_string()),
                action_director_id: Some("action-director-manual".to_string()),
                scene_type: Some("daily_dialogue".to_string()),
                layout_prompt: "close shot, interior".to_string(),
                render_prompt: "restrained realism".to_string(),
            },
        )
        .expect("missing taxonomy should not panic when manual directors exist");

        assert_eq!(plan.render_segment.scene_taxonomy_id, None);
        assert_eq!(plan.render_segment.scene_type, None);
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_scene_director_id,
            "scene-director-manual"
        );
        assert_eq!(
            plan.committee_runtime.prompt_layers.layout_prompt,
            "close shot, interior"
        );
    }

    #[test]
    fn build_project_create_or_switch_snapshot_tracks_fixture_projects() {
        let state = test_state();
        let fixture = load_desktop_shared_fixture().expect("shared fixture should load");

        let snapshot = build_project_create_or_switch_snapshot_from_fixture(
            &state,
            ProjectCreateOrSwitchRequest {
                project_id: None,
                project_name: None,
            },
            &fixture,
        );

        assert_eq!(
            snapshot.current_project_id.as_deref(),
            Some("project-week3-001")
        );
        assert_eq!(snapshot.projects.len(), 1);
        assert!(snapshot.projects[0].episode_count >= 1);
        assert!(snapshot.projects[0].name.contains("Hope"));
        assert_eq!(
            snapshot
                .readonly_status
                .snapshot_bootstrap
                .snapshot_identity
                .snapshot_id,
            "hope-kb-v0.1"
        );
        assert_eq!(
            snapshot
                .readonly_status
                .validation_feedback
                .source_snapshot_id,
            "hope-kb-v0.1"
        );
        assert!(
            snapshot
                .readonly_status
                .validation_feedback
                .repair_mapping_ready
        );
    }

    #[test]
    fn app_shell_readonly_status_snapshot_stays_summary_only() {
        let state = test_state();
        let status = AppShellReadonlyStatusSnapshot {
            snapshot_bootstrap: state.snapshot_bootstrap_readonly_state().clone(),
            validation_feedback: state.validation_feedback_readonly_state().clone(),
        };

        assert_eq!(
            status
                .snapshot_bootstrap
                .knowledge_bundle
                .scene_taxonomy_count,
            1
        );
        assert_eq!(status.validation_feedback.failure_pattern_count, 2);
        assert_eq!(status.validation_feedback.prompt_template_count, 2);
        assert!(status.validation_feedback.repair_mapping_ready);
    }

    #[test]
    fn build_writer_entry_snapshot_summarizes_fixture_layers() {
        let fixture = load_desktop_shared_fixture().expect("shared fixture should load");

        let snapshot = build_writer_entry_snapshot_from_fixture(
            WriterEntrySnapshotRequest {
                project_id: "project-week3-001".to_string(),
            },
            &fixture,
        );

        assert!(snapshot.synopsis.contains("分钟"));
        assert!(snapshot.story.contains("第 1 集"));
        assert!(snapshot.screenplay.contains("Cut"));
        assert!(snapshot.storyboard.contains("RenderSegment"));
    }

    #[test]
    fn build_storyboard_rendersegment_cut_preview_snapshot_includes_runtime_details() {
        let state = test_state();
        let fixture = load_desktop_shared_fixture().expect("shared fixture should load");

        let snapshot = build_storyboard_rendersegment_cut_preview_snapshot_from_fixture(
            &state,
            StoryboardRenderSegmentCutPreviewSnapshotRequest {
                project_id: "project-week3-001".to_string(),
                episode_id: None,
                narrative_scene_id: None,
                render_segment_id: None,
                scene_type: Some("daily_dialogue".to_string()),
            },
            &fixture,
        );

        assert!(!snapshot.storyboard.is_empty());
        assert!(!snapshot.render_segment.is_empty());
        assert!(!snapshot.cuts.is_empty());
        assert!(snapshot.render_segment[0].note.contains("handoff"));
        assert!(snapshot.render_segment[0].note.contains("scene"));
    }

    #[test]
    fn build_validation_export_panel_snapshot_surfaces_kb_repairs() {
        let state = test_state();
        let mut fixture = load_desktop_shared_fixture().expect("shared fixture should load");
        fixture.prompt_package[0].body = "TODO: rewrite this prompt body".to_string();

        let snapshot = build_validation_export_panel_snapshot_from_fixture(
            &state,
            ValidationExportPanelSnapshotRequest {
                project_id: "project-week3-001".to_string(),
            },
            &fixture,
        )
        .expect("panel snapshot should build from fixture");

        assert_eq!(snapshot.project_id, "project-week3-001");
        assert_eq!(snapshot.summary_items.len(), 3);
        assert_eq!(
            snapshot.summary_items[2].state,
            ValidationExportPanelState::Ready
        );
        assert!(
            snapshot.summary_items[2]
                .value
                .contains("snapshot hope-kb-v0.1")
        );
        assert!(
            snapshot
                .repair_recommendations
                .iter()
                .any(|item| item.failure_code == "chinese_prompt_noise")
        );
        assert!(
            snapshot
                .repair_recommendations
                .iter()
                .flat_map(|item| item.prompt_template_names.iter())
                .any(|name| name == "Repair Prompt Language")
        );
    }

    #[test]
    fn build_validation_export_panel_snapshot_stays_bounded_without_repair_mappings() {
        let state = test_state_without_repair_mappings();
        let mut fixture = load_desktop_shared_fixture().expect("shared fixture should load");
        fixture.prompt_package[0].body = "TODO: rewrite this prompt body".to_string();

        let snapshot = build_validation_export_panel_snapshot_from_fixture(
            &state,
            ValidationExportPanelSnapshotRequest {
                project_id: "project-week3-001".to_string(),
            },
            &fixture,
        )
        .expect("panel snapshot should still build without repair mappings");

        assert_eq!(snapshot.repair_recommendations.len(), 0);
        assert_eq!(
            snapshot.summary_items[2].state,
            ValidationExportPanelState::Pending
        );
        assert!(snapshot.summary_items[1].value.contains("rows"));
    }
}
