use std::env;
use std::hash::{Hash, Hasher};
use std::io;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::state::AppState;

use core_domain::{
    BridgeCallStatus, ExpandScriptRequest, ExpandScriptResponse, ExportArtifactRecord,
    ExportBundleRequest, ExportBundleResponse, ExternalReferenceHandleCandidate,
    GenerateStoryboardRequest, GenerateStoryboardResponse, GeneratedStoryboardRow,
    GoldenSampleLibraryRecord, KbRouterExcludedCandidate, KbRouterRetrievalTrace,
    KbRouterRuntimeRequest, KbRouterRuntimeResponse, KbRouterSelectedRule, KbRouterSelectionReason,
    KbRouterTaskType, KbRouterTokenBudget, ProductWarning, PromptBodyCandidate,
    PromptTextCompilationRequest, PromptTextCompilationResponse, PromptTextCompilationRow,
    PromptTextCompilationStatus, ScenePerformanceProjection, SequenceFieldState, SequenceGrouping,
    StoryboardDurationPlan, StoryboardExportStatus, StructureMode, TextGenerationOutputSchema,
    TextGenerationRequest, TextGenerationResponse, TextGenerationTask, TextModelProvider,
    TextModelProviderKind, UpdateStoryboardRowsRequest,
};
use export_engine::{V120StoryboardExportRequest, export_v120_storyboard_bundle};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use storyboard_pipeline::{StoryboardPlan, StoryboardPlanRequest, StoryboardPlanningError};
use validators::{
    RepairRecommendation, WEEK3_SHARED_FIXTURE_PATH, Week3SharedFixture,
    generate_week3_repair_recommendations, generate_week3_validation_report,
    load_week3_shared_fixture, project_v120_evidence_aware_findings,
};

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationExportPanelSnapshotRequest {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationExportPanelSnapshot {
    pub project_id: String,
    pub summary_items: Vec<ValidationExportPanelItem>,
    pub repair_recommendations: Vec<ValidationRepairRecommendationItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationExportPanelItem {
    pub label: String,
    pub value: String,
    pub state: ValidationExportPanelState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationExportPanelState {
    Ready,
    Pending,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationRepairRecommendationItem {
    pub failure_code: String,
    pub failure_name: String,
    pub repair_strategy: String,
    pub repair_priority: String,
    pub repair_scope: String,
    pub validator_hint: String,
    pub prompt_template_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct LiveStoryboardRowsEnvelope {
    rows: Vec<LiveStoryboardRowPatch>,
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
    let mut blockers = Vec::new();
    let normalized_scene_type = normalize_scene_type(&request.scene_type);
    let router_request = KbRouterRuntimeRequest {
        scene_type: normalized_scene_type.clone(),
        synopsis_text: request.synopsis_text.clone(),
        duration_seconds: 0,
        task_type: KbRouterTaskType::ExpandScript,
        shot_intent: None,
        structure_type: None,
    };
    if request.synopsis_text.trim().is_empty() {
        blockers.push(ProductWarning {
            code: "synopsis_required".to_string(),
            message: "请输入故事梗概后再扩写脚本。".to_string(),
            related_sample_id: None,
        });
    }
    if request.scene_type.trim().is_empty() {
        blockers.push(ProductWarning {
            code: "scene_type_required".to_string(),
            message: "请选择有效的场景类型后再扩写脚本。".to_string(),
            related_sample_id: None,
        });
    } else if resolve_scene_taxonomy(state, Some(&request.scene_type)).is_none()
        && resolve_scene_taxonomy(state, Some(&normalized_scene_type)).is_none()
    {
        blockers.push(ProductWarning {
            code: "scene_type_invalid".to_string(),
            message: "当前场景类型无效，无法扩写脚本。".to_string(),
            related_sample_id: None,
        });
    }

    if !blockers.is_empty() {
        return ExpandScriptResponse {
            status: BridgeCallStatus::Blocked,
            script_id: String::new(),
            expanded_script_text: String::new(),
            script_hash: String::new(),
            blockers,
            warnings: vec![],
            kb_router_result: empty_kb_router_response(&router_request, &state.kb_runtime),
        };
    }

    let script_hash = stable_hash_hex(&format!(
        "{}\n{}",
        normalized_scene_type,
        request.synopsis_text.trim()
    ));
    let script_id = format!("script-{}", &script_hash[..12]);

    let kb_router_result = run_kb_router(state, router_request);
    let provider = default_text_model_provider();
    let generation_request = build_text_generation_request(
        TextGenerationTask::ExpandScript,
        Some(normalized_scene_type.clone()),
        request.synopsis_text.trim().to_string(),
        None,
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
    let generated_script = run_text_generation(&provider, &generation_request);
    let mut warnings = generated_script.warnings.clone();
    if !state
        .kb_golden_sample_runtime
        .manifest
        .seed_import_format
        .contains("v0.2")
    {
        warnings.push(ProductWarning {
            code: "v120_package_not_confirmed".to_string(),
            message: "Loaded KB package does not advertise the accepted v0.2 seed import format."
                .to_string(),
            related_sample_id: None,
        });
    }
    let expanded_script_text = if generated_script.warnings.is_empty() {
        match validate_generated_script_text(&generated_script.text) {
            Some(text) => text.to_string(),
            None => {
                warnings.push(ProductWarning {
                    code: "text_model_validator_failed".to_string(),
                    message:
                        "The live expanded script was empty or contained forbidden real-name style content, so Hope rejected it."
                            .to_string(),
                    related_sample_id: None,
                });
                warnings.push(ProductWarning {
                    code: "text_model_live_expand_fallback".to_string(),
                    message:
                        "Expanded script fell back to the deterministic bridge envelope because the live result was missing or failed validation."
                            .to_string(),
                    related_sample_id: None,
                });
                deterministic_expanded_script_text(
                    &normalized_scene_type,
                    request.synopsis_text.trim(),
                    &state.kb_golden_sample_runtime.manifest.snapshot_name,
                )
            }
        }
    } else {
        warnings.push(ProductWarning {
            code: "text_model_live_expand_fallback".to_string(),
            message:
                "Expanded script stayed on the deterministic bridge envelope because live Qwen was unavailable or not configured."
                    .to_string(),
            related_sample_id: None,
        });
        deterministic_expanded_script_text(
            &normalized_scene_type,
            request.synopsis_text.trim(),
            &state.kb_golden_sample_runtime.manifest.snapshot_name,
        )
    };
    let status = if warnings.is_empty() {
        BridgeCallStatus::Ready
    } else {
        BridgeCallStatus::WarningOnly
    };

    let response = ExpandScriptResponse {
        status,
        script_id,
        expanded_script_text,
        script_hash,
        blockers: vec![],
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
    let script_text = request
        .expanded_script_text
        .clone()
        .or_else(|| {
            request
                .script_id
                .as_deref()
                .and_then(|script_id| state.find_script(script_id))
                .map(|script| script.expanded_script_text)
        })
        .unwrap_or_default();
    let scene_type = extract_scene_type(&script_text);
    let normalized_scene_type = normalize_scene_type(&scene_type);
    let synopsis_text = extract_synopsis_text(&script_text);
    let router_request = KbRouterRuntimeRequest {
        scene_type: normalized_scene_type.clone(),
        synopsis_text,
        duration_seconds: request.selected_total_duration_seconds,
        task_type: KbRouterTaskType::GenerateStoryboard,
        shot_intent: None,
        structure_type: None,
    };
    let mut blockers = Vec::new();

    if script_text.trim().is_empty() {
        blockers.push(ProductWarning {
            code: "task_script_required".to_string(),
            message: "请先完成脚本扩写或提供有效脚本内容，再生成分镜。".to_string(),
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
    if scene_type.trim().is_empty() {
        blockers.push(ProductWarning {
            code: "scene_type_required".to_string(),
            message: "脚本中缺少有效场景类型，无法生成分镜。".to_string(),
            related_sample_id: None,
        });
    } else if resolve_scene_taxonomy(state, Some(&scene_type)).is_none()
        && resolve_scene_taxonomy(state, Some(&normalized_scene_type)).is_none()
    {
        blockers.push(ProductWarning {
            code: "scene_type_invalid".to_string(),
            message: "脚本中的场景类型无效，无法生成分镜。".to_string(),
            related_sample_id: None,
        });
    }

    if !blockers.is_empty() {
        return blocked_storyboard_response(
            None,
            request.selected_total_duration_seconds,
            blockers,
            now_ms,
            &router_request,
            &state.kb_runtime,
        );
    }

    let kb_router_result = run_kb_router(state, router_request.clone());
    let selected_records = select_golden_sample_records_from_router(state, &kb_router_result);
    if selected_records.is_empty() {
        return blocked_storyboard_response(
            None,
            request.selected_total_duration_seconds,
            vec![ProductWarning {
                code: "no_selectable_rows".to_string(),
                message: "当前任务没有可用的官方分镜样本，无法生成分镜。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
            &router_request,
            &state.kb_runtime,
        );
    }
    let row_count = selected_records.len().max(1);
    let mut rows = Vec::new();
    let mut warnings = Vec::new();
    let row_durations =
        match allocate_storyboard_row_durations(request.selected_total_duration_seconds, row_count)
        {
            Some(durations) => durations,
            None => {
                return blocked_storyboard_response(
                    None,
                    request.selected_total_duration_seconds,
                    vec![ProductWarning {
                        code: "duration_allocation_failed".to_string(),
                        message: "镜头时长分配失败，无法生成满足总时长守恒的分镜。".to_string(),
                        related_sample_id: None,
                    }],
                    now_ms,
                    &router_request,
                    &state.kb_runtime,
                );
            }
        };
    let provider = default_text_model_provider();
    let generation_request = build_text_generation_request(
        TextGenerationTask::GenerateStoryboard,
        Some(normalized_scene_type.clone()),
        script_text.clone(),
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
    let live_generation = run_text_generation(&provider, &generation_request);
    warnings.extend(live_generation.warnings.clone());
    let live_row_patches = match extract_live_storyboard_row_patches(&live_generation) {
        Ok(patches) => patches,
        Err(warning) => {
            warnings.push(warning);
            Vec::new()
        }
    };

    for (index, record) in selected_records.iter().enumerate() {
        let failure_mapping = state
            .kb_golden_sample_runtime
            .failure_mapping
            .records
            .iter()
            .find(|mapping| mapping.sample_id == record.sample_id);
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

        let prompt_candidate = project_prompt_body_candidate(record);
        let scene_projection = project_scene_performance(record);
        if prompt_candidate.candidate_text.is_some() {
            warnings.push(ProductWarning {
                code: "prompt_body_candidate_not_compiled".to_string(),
                message: "V120 prompt_body is held as candidate evidence and is not compiled into runtime prompt_text.".to_string(),
                related_sample_id: Some(record.sample_id.clone()),
            });
        }
        let prompt_compilation = compile_seedance_prompt_text(
            state,
            build_prompt_text_compilation_request(
                record,
                &scene_projection,
                row_durations[index],
                &kb_router_result,
            ),
        );
        warnings.extend(prompt_compilation.warnings.iter().cloned());

        rows.push(GeneratedStoryboardRow {
            shot_id: record.source_fields.shot_id.clone(),
            order: (index + 1) as u32,
            person: scene_projection.person.clone(),
            shot_title: record.source_fields.sample_title.clone(),
            scene_scale: scene_projection.scene_scale.clone(),
            visual_description: scene_projection.visual_description.clone(),
            character_action: scene_projection.character_action.clone(),
            dialogue: String::new(),
            prompt_text: prompt_compilation.prompt_text.clone(),
            prompt_text_compilation_status: prompt_compilation.compilation_status,
            prompt_text_compilation_warnings: prompt_compilation.warnings.clone(),
            prompt_text_source_row_id: prompt_compilation.source_row_id.clone(),
            duration_seconds: row_durations[index],
            prompt_body_candidate: prompt_candidate,
            scene_performance_projection: scene_projection.clone(),
            external_reference_handle_candidates: project_reference_handle_candidates(record),
            sequence_grouping: scene_projection.sequence_grouping,
        });

        if let Some(patch) = live_row_patches.get(index) {
            let row = rows
                .last_mut()
                .expect("storyboard row must exist immediately after push");
            apply_live_storyboard_patch(row, patch);
        }
    }

    let validator_findings =
        validate_storyboard_rows(&rows, request.selected_total_duration_seconds);
    if !validator_findings.is_empty() {
        warnings.extend(validator_findings.iter().cloned());
        if live_generation.provider == TextModelProviderKind::Qwen && provider.enabled {
            warnings.push(ProductWarning {
                code: "text_model_live_storyboard_fallback".to_string(),
                message:
                    "Live storyboard content failed local validation, so Hope kept the deterministic bridge rows."
                        .to_string(),
                related_sample_id: None,
            });
            rows.clear();
            for (index, record) in selected_records.iter().enumerate() {
                let prompt_candidate = project_prompt_body_candidate(record);
                let scene_projection = project_scene_performance(record);
                let prompt_compilation = compile_seedance_prompt_text(
                    state,
                    build_prompt_text_compilation_request(
                        record,
                        &scene_projection,
                        row_durations[index],
                        &kb_router_result,
                    ),
                );
                warnings.extend(prompt_compilation.warnings.iter().cloned());
                rows.push(GeneratedStoryboardRow {
                    shot_id: record.source_fields.shot_id.clone(),
                    order: (index + 1) as u32,
                    person: scene_projection.person.clone(),
                    shot_title: record.source_fields.sample_title.clone(),
                    scene_scale: scene_projection.scene_scale.clone(),
                    visual_description: scene_projection.visual_description.clone(),
                    character_action: scene_projection.character_action.clone(),
                    dialogue: String::new(),
                    prompt_text: prompt_compilation.prompt_text.clone(),
                    prompt_text_compilation_status: prompt_compilation.compilation_status,
                    prompt_text_compilation_warnings: prompt_compilation.warnings.clone(),
                    prompt_text_source_row_id: prompt_compilation.source_row_id.clone(),
                    duration_seconds: row_durations[index],
                    prompt_body_candidate: prompt_candidate,
                    scene_performance_projection: scene_projection.clone(),
                    external_reference_handle_candidates: project_reference_handle_candidates(
                        record,
                    ),
                    sequence_grouping: scene_projection.sequence_grouping,
                });
            }
        }
    }

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
            None,
            request.selected_total_duration_seconds,
            vec![ProductWarning {
                code: "duration_conservation_failed".to_string(),
                message: "分镜总时长与任务时长不一致，已阻断生成结果。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
            &router_request,
            &state.kb_runtime,
        );
    }
    let task_id = format!(
        "task-{}",
        &stable_hash_hex(&format!(
            "{}\n{}",
            request.task_name.trim(),
            normalized_scene_type
        ))[..12]
    );
    let result_id = format!(
        "storyboard-{}",
        &stable_hash_hex(&format!(
            "{}\n{}\n{}",
            request.task_name, script_text, request.selected_total_duration_seconds
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
        kb_router_result,
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
    };
    state.remember_storyboard(response.clone());
    response
}

pub fn save_storyboard_rows(
    state: &AppState,
    request: UpdateStoryboardRowsRequest,
) -> GenerateStoryboardResponse {
    let now_ms = now_epoch_ms();
    let missing_router_request = KbRouterRuntimeRequest {
        scene_type: String::new(),
        synopsis_text: String::new(),
        duration_seconds: 0,
        task_type: KbRouterTaskType::GenerateStoryboard,
        shot_intent: None,
        structure_type: None,
    };
    let Some(mut snapshot) = state.find_storyboard(&request.result_id) else {
        return blocked_storyboard_response(
            request.task_id,
            0,
            vec![ProductWarning {
                code: "storyboard_result_not_found".to_string(),
                message: "未找到可编辑的分镜结果。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
            &missing_router_request,
            &state.kb_runtime,
        );
    };

    let snapshot_router_request = KbRouterRuntimeRequest {
        scene_type: snapshot
            .kb_router_result
            .retrieval_trace
            .story_keywords
            .first()
            .cloned()
            .unwrap_or_default(),
        synopsis_text: snapshot.kb_router_result.kb_context_summary.clone(),
        duration_seconds: snapshot.selected_total_duration_seconds,
        task_type: KbRouterTaskType::GenerateStoryboard,
        shot_intent: None,
        structure_type: None,
    };

    if snapshot.revision != request.base_revision {
        return blocked_storyboard_response(
            snapshot.task_id.clone(),
            snapshot.selected_total_duration_seconds,
            vec![ProductWarning {
                code: "storyboard_revision_conflict".to_string(),
                message: "分镜内容已被其他操作更新，请刷新后重试。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
            &snapshot_router_request,
            &state.kb_runtime,
        );
    }

    if request.rows.is_empty() {
        return blocked_storyboard_response(
            snapshot.task_id.clone(),
            snapshot.selected_total_duration_seconds,
            vec![ProductWarning {
                code: "storyboard_rows_required".to_string(),
                message: "保存分镜前请至少保留一条镜头。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
            &snapshot_router_request,
            &state.kb_runtime,
        );
    }

    if request.rows.iter().any(|row| row.duration_seconds == 0) {
        return blocked_storyboard_response(
            snapshot.task_id.clone(),
            snapshot.selected_total_duration_seconds,
            vec![ProductWarning {
                code: "row_duration_required".to_string(),
                message: "每条分镜的时长都必须大于 0 秒。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
            &snapshot_router_request,
            &state.kb_runtime,
        );
    }

    let total_duration = request
        .rows
        .iter()
        .map(|row| row.duration_seconds)
        .sum::<u16>();
    if total_duration != snapshot.selected_total_duration_seconds {
        return blocked_storyboard_response(
            snapshot.task_id.clone(),
            snapshot.selected_total_duration_seconds,
            vec![ProductWarning {
                code: "duration_conservation_failed".to_string(),
                message: "编辑后的镜头总时长必须与任务总时长保持一致。".to_string(),
                related_sample_id: None,
            }],
            now_ms,
            &snapshot_router_request,
            &state.kb_runtime,
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
                message: reason,
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

fn run_kb_router(state: &AppState, request: KbRouterRuntimeRequest) -> KbRouterRuntimeResponse {
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

    for record in &state.kb_golden_sample_runtime.golden_sample_library.records {
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
        for record in state
            .kb_golden_sample_runtime
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
                "规则 {} 仅保留压缩摘要，覆盖 {} 行，用于 {} 的本地约束，不展开原始 prompt_body 或 teaching_note。",
                rule.core,
                rule.row_count,
                request.task_type.as_str()
            ),
            applies_to: vec![request.task_type.as_str().to_string()],
        })
        .collect::<Vec<_>>();

    rules.push(KbRouterSelectedRule {
        rule_id: "reserve_gate".to_string(),
        family: "reserve_gate".to_string(),
        summary:
            "reserve 与 usable_for_fewshot=No 仅作为规则证据或负向约束，不进入 positive few-shot。"
                .to_string(),
        applies_to: vec![request.task_type.as_str().to_string()],
    });
    rules.push(KbRouterSelectedRule {
        rule_id: "duration_guard".to_string(),
        family: "duration".to_string(),
        summary: "总时长必须守恒，且模型侧 full_kb_rows_included 永远保持 0。".to_string(),
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
                .golden_sample_library
                .records
                .iter()
                .find(|record| &record.sample_id == sample_id)
                .map(|record| {
                    format!(
                        "样本 {}：scene_category={}，sample_type={}，scene_scale_hint={}，只保留结构 lesson 与时长/连续性约束，不暴露 prompt_body、teaching_note、source_register。",
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
        "这是 Hope 本地 KB Router 的压缩摘要。任务={}，scene_type={}，duration_seconds={}，story_keywords={}。正向样本仅从 official 且 usable_for_fewshot=Yes 中选取，当前 selected_sample_ids={}。{} {} 排除候选摘要：{}。本摘要只保留场景结构、连续性、时长和 prompt_text 编译约束，不输出完整 prompt_body、不输出完整 teaching_note、不输出 source_register/provenance/overlay JSON，也不输出真实导演名、IP 名或品牌名。reserve 与 usable_for_fewshot=No 仅保留为规则证据或负向约束，不能作为 positive few-shot。full_kb_rows_included 固定为 0，禁止把 152 行整包送入上下文。导出与 trace 只记录 selected_sample_ids、压缩规则摘要、kb_context_summary 与 retrieval_trace。",
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
        summary.push_str(" 本地摘要继续强调：只保留与当前任务直接相关的结构化规则和样本 lesson，不复制原始 23 字段整行，不复制 prompt_body，不复制 teaching_note，不复制 provenance。");
    }
    summary.chars().take(1450).collect()
}

fn select_golden_sample_records_from_router<'a>(
    state: &'a AppState,
    router_result: &KbRouterRuntimeResponse,
) -> Vec<&'a GoldenSampleLibraryRecord> {
    router_result
        .selected_sample_ids
        .iter()
        .filter_map(|sample_id| {
            state
                .kb_golden_sample_runtime
                .golden_sample_library
                .records
                .iter()
                .find(|record| &record.sample_id == sample_id)
        })
        .collect()
}

#[cfg(test)]
fn select_golden_sample_records<'a>(
    state: &'a AppState,
    script_text: &str,
    request: &GenerateStoryboardRequest,
) -> Vec<&'a GoldenSampleLibraryRecord> {
    let normalized_scene_type = normalize_scene_type(&extract_scene_type(script_text));
    let query = format!(
        "{} {} {} {}",
        request.task_name,
        script_text,
        normalized_scene_type,
        request.expanded_script_text.as_deref().unwrap_or_default()
    )
    .to_lowercase();
    let target_rows = ((request.selected_total_duration_seconds / 8).max(1) as usize).clamp(1, 8);
    let records = &state.kb_golden_sample_runtime.golden_sample_library.records;

    let mut selected = records
        .iter()
        .filter(|record| record.is_official())
        .filter(|record| {
            let fields = &record.source_fields;
            query.contains(&fields.scene_category.to_lowercase())
                || query.contains(&fields.scene_tag.to_lowercase())
                || query.contains(&fields.style_cluster.to_lowercase())
        })
        .take(target_rows)
        .collect::<Vec<_>>();

    if selected.is_empty() {
        selected = records
            .iter()
            .filter(|record| record.is_official())
            .take(target_rows)
            .collect();
    }

    selected
}

fn product_warning_from_evidence(item: validators::EvidenceAwareValidationItem) -> ProductWarning {
    ProductWarning {
        code: item.code,
        message: item.message,
        related_sample_id: item.source_sample_id,
    }
}

fn default_text_model_provider() -> TextModelProvider {
    let provider = match env::var("HOPE_TEXT_MODEL_PROVIDER")
        .unwrap_or_else(|_| "qwen".to_string())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "doubao" => TextModelProviderKind::Doubao,
        "custom" => TextModelProviderKind::Custom,
        _ => TextModelProviderKind::Qwen,
    };
    TextModelProvider {
        provider,
        model: env::var("HOPE_TEXT_MODEL_MODEL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "qwen-default-text".to_string()),
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
    request: &TextGenerationRequest,
) -> TextGenerationResponse {
    if !provider.enabled {
        return run_text_generation_stub(
            provider,
            request,
            ProductWarning {
                code: "text_model_live_call_closed".to_string(),
                message: format!(
                    "Text generation stays deterministic until the {} provider is enabled in runtime config.",
                    provider_kind_label(provider.provider)
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
                message:
                    "Only the Qwen live text provider is wired in this MVP; other providers stay stubbed."
                        .to_string(),
                related_sample_id: request.selected_sample_ids.first().cloned(),
            },
        );
    }
    let Some(api_key) = resolve_provider_api_key(provider) else {
        return run_text_generation_stub(
            provider,
            request,
            ProductWarning {
                code: "text_model_api_key_missing".to_string(),
                message:
                    "Qwen live text generation is enabled but the configured API key reference could not be resolved."
                        .to_string(),
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
                message:
                    "Qwen live text generation needs a configurable base_url or endpoint before Hope can call it."
                        .to_string(),
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
    let response: QwenChatCompletionResponse = serde_json::from_value(body).map_err(|_| ProductWarning {
        code: "text_model_response_invalid".to_string(),
        message: "Qwen returned a payload that Hope could not parse into the expected response shape.".to_string(),
        related_sample_id: request.selected_sample_ids.first().cloned(),
    })?;
    let content = response
        .choices
        .first()
        .map(|choice| choice.message.content.trim())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| ProductWarning {
            code: "text_model_response_invalid".to_string(),
            message:
                "Qwen returned an empty text body, so Hope kept the local deterministic fallback."
                    .to_string(),
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
    let system_prompt = "You are Hope's controlled text-generation layer. Use only the compressed KB context you receive. Never invent real director names, IP names, brand names, or external asset bindings. Do not expand to full KB rows.";
    let user_prompt = format!(
        "task_type={:?}\nscene_type={}\nduration_seconds={}\nstory_input={}\nkb_context_summary={}\nselected_sample_ids={}\nselected_kb_rules={}\noutput_schema={}\nconstraints=keep total duration conserved; do not emit full KB; do not emit raw prompt_body as final prompt_text; do not use real director/IP/brand names.",
        request.task_type,
        scene_type,
        duration_seconds,
        request.story_input,
        request.kb_context_summary,
        request.selected_sample_ids.join(","),
        request.selected_kb_rules.join(" | "),
        output_schema,
    );

    let mut payload = json!({
        "model": provider.model,
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
            message: "Hope could not initialize the Qwen HTTP client for this runtime session."
                .to_string(),
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
            message: "Hope could not reach the configured Qwen endpoint, so it kept the local deterministic fallback.".to_string(),
            related_sample_id: None,
        })?;
    let status = response.status();
    let body: Value = response.json().map_err(|_| ProductWarning {
        code: "text_model_response_invalid".to_string(),
        message: "Qwen returned a non-JSON body that Hope could not interpret.".to_string(),
        related_sample_id: None,
    })?;
    if !status.is_success() {
        return Err(ProductWarning {
            code: "text_model_network_error".to_string(),
            message: format!(
                "Qwen endpoint returned HTTP {}, so Hope kept the local deterministic fallback.",
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

fn provider_kind_label(kind: TextModelProviderKind) -> &'static str {
    match kind {
        TextModelProviderKind::Qwen => "qwen",
        TextModelProviderKind::Doubao => "doubao",
        TextModelProviderKind::Custom => "custom",
    }
}

fn validate_generated_script_text(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    if trimmed.is_empty() || contains_forbidden_generation_terms(trimmed) {
        None
    } else {
        Some(trimmed)
    }
}

fn extract_live_storyboard_row_patches(
    generation_response: &TextGenerationResponse,
) -> Result<Vec<LiveStoryboardRowPatch>, ProductWarning> {
    let Some(structured_json) = generation_response.structured_json.as_ref() else {
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
        message:
            "Qwen storyboard output was not valid row JSON, so Hope kept the deterministic bridge rows."
                .to_string(),
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

fn validate_storyboard_rows(
    rows: &[GeneratedStoryboardRow],
    expected_duration_seconds: u16,
) -> Vec<ProductWarning> {
    let mut findings = Vec::new();
    if rows.iter().map(|row| row.duration_seconds).sum::<u16>() != expected_duration_seconds {
        findings.push(ProductWarning {
            code: "duration_conservation_failed".to_string(),
            message: "Storyboard row durations no longer match the requested total duration."
                .to_string(),
            related_sample_id: None,
        });
    }
    for row in rows {
        for (field_name, field_value) in [
            ("shot_title", row.shot_title.as_str()),
            ("scene_scale", row.scene_scale.as_str()),
            ("visual_description", row.visual_description.as_str()),
            ("character_action", row.character_action.as_str()),
        ] {
            if field_value.trim().is_empty() {
                findings.push(ProductWarning {
                    code: "text_model_validator_failed".to_string(),
                    message: format!(
                        "Storyboard row {} is missing required field {} after live generation.",
                        row.shot_id, field_name
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
            if contains_forbidden_generation_terms(field_value) {
                findings.push(ProductWarning {
                    code: "text_model_validator_failed".to_string(),
                    message: format!(
                        "Storyboard row {} includes forbidden real-name or brand-like content in {}.",
                        row.shot_id, field_name
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
        }
    }
    findings
}

fn contains_forbidden_generation_terms(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        "director_style_ref",
        "christopher nolan",
        "marvel",
        "disney",
        "nike",
        "harry potter",
    ]
    .iter()
    .any(|term| lower.contains(term))
}

fn deterministic_expanded_script_text(
    normalized_scene_type: &str,
    synopsis_text: &str,
    snapshot_name: &str,
) -> String {
    format!(
        "scene_type: {}\nsynopsis: {}\nsource_package: {}",
        normalized_scene_type, synopsis_text, snapshot_name
    )
}

fn build_prompt_text_compilation_request(
    record: &GoldenSampleLibraryRecord,
    scene_projection: &ScenePerformanceProjection,
    duration_seconds: u16,
    kb_router_result: &KbRouterRuntimeResponse,
) -> PromptTextCompilationRequest {
    let scene_type = record.source_fields.scene_category.clone();
    let scene_label = record
        .classification
        .scene_tags
        .first()
        .cloned()
        .unwrap_or_else(|| record.source_fields.scene_tag.clone());

    PromptTextCompilationRequest {
        row: PromptTextCompilationRow {
            shot_id: record.source_fields.shot_id.clone(),
            shot_title: record.source_fields.sample_title.clone(),
            scene_scale: scene_projection.scene_scale.clone(),
            visual_description: scene_projection.visual_description.clone(),
            character_action: scene_projection.character_action.clone(),
            dialogue: String::new(),
            duration_seconds,
        },
        scene_type,
        scene_label,
        kb_context_summary: kb_router_result.kb_context_summary.clone(),
        selected_kb_rules: kb_router_result
            .selected_kb_rules
            .iter()
            .map(|rule| format!("{}:{}", rule.rule_id, rule.summary))
            .collect(),
        continuity_negative_core: record.source_fields.continuity_negative_core.clone(),
        target_profile: "seedance2.0".to_string(),
    }
}

fn compile_seedance_prompt_text(
    state: &AppState,
    mut request: PromptTextCompilationRequest,
) -> PromptTextCompilationResponse {
    if request.selected_kb_rules.is_empty() {
        request.selected_kb_rules = select_prompt_compilation_rules(state, &request);
    }
    let provider = default_text_model_provider();
    let generation_request = build_text_generation_request(
        TextGenerationTask::CompileSeedancePromptText,
        Some(request.scene_type.clone()),
        format!(
            "{} | {} | {} | {}",
            request.scene_label,
            request.row.shot_title,
            request.row.visual_description,
            request.row.character_action
        ),
        Some(StoryboardDurationPlan {
            total_duration_seconds: request.row.duration_seconds,
            row_count: 1,
            per_row_seconds: request.row.duration_seconds,
            allocated_seconds: request.row.duration_seconds,
        }),
        request.kb_context_summary.clone(),
        vec![request.row.shot_id.clone()],
        request.selected_kb_rules.clone(),
        TextGenerationOutputSchema::SeedancePromptText,
        Some(220),
    );
    let stub_response = run_text_generation(
        &TextModelProvider {
            enabled: false,
            ..provider.clone()
        },
        &generation_request,
    );

    let mut prompt_sections = vec![
        format!("场景类型：{}", request.scene_type),
        format!("场景标签：{}", request.scene_label),
        format!("镜头标题：{}", request.row.shot_title),
        format!("景别：{}", request.row.scene_scale),
        format!("画面描述：{}", request.row.visual_description),
        format!("角色动作：{}", request.row.character_action),
    ];
    if !request.row.dialogue.trim().is_empty() {
        prompt_sections.push(format!("对白/旁白：{}", request.row.dialogue.trim()));
    }
    prompt_sections.push(format!("时长：{}秒", request.row.duration_seconds));
    if !request.selected_kb_rules.is_empty() {
        prompt_sections.push(format!("KB摘要：{}", request.selected_kb_rules.join("；")));
    }
    prompt_sections.push(format!("KB上下文：{}", request.kb_context_summary));
    if !request.continuity_negative_core.trim().is_empty() {
        prompt_sections.push(format!(
            "连续性约束：{}",
            request.continuity_negative_core.trim()
        ));
    }
    prompt_sections.push("目标适配：Seedance2.0 文本提示词".to_string());

    let mut warnings = stub_response.warnings;
    warnings.push(ProductWarning {
        code: "seedance_video_generation_closed".to_string(),
        message:
            "Seedance2.0 is a prompt_text adaptation target in MVP; in-app video generation remains gated."
                .to_string(),
        related_sample_id: Some(request.row.shot_id.clone()),
    });
    warnings.push(ProductWarning {
        code: "prompt_body_candidate_not_promoted".to_string(),
        message:
            "Prompt compilation uses structured storyboard fields plus selected KB summaries; raw prompt_body does not become final prompt_text."
                .to_string(),
        related_sample_id: Some(request.row.shot_id.clone()),
    });

    PromptTextCompilationResponse {
        prompt_text: prompt_sections.join("；"),
        target_profile: request.target_profile,
        compilation_status: PromptTextCompilationStatus::ReadyStub,
        source_row_id: request.row.shot_id,
        warnings,
        provider: stub_response.provider,
        model: stub_response.model,
    }
}

fn select_prompt_compilation_rules(
    state: &AppState,
    request: &PromptTextCompilationRequest,
) -> Vec<String> {
    let is_sequence = request
        .row
        .shot_id
        .to_ascii_lowercase()
        .contains("sequence");
    let preferred_cores = if is_sequence {
        [
            "scene_performance_core",
            "continuity_negative_core",
            "camera_directing_core",
        ]
    } else {
        [
            "scene_performance_core",
            "camera_directing_core",
            "continuity_negative_core",
        ]
    };

    preferred_cores
        .iter()
        .filter_map(|core| {
            state
                .kb_golden_sample_runtime
                .field_coverage_rules
                .records
                .iter()
                .find(|rule| rule.core == *core)
                .map(|rule| {
                    format!(
                        "{} summary only ({} rows, fewshot positive gated to official rows)",
                        rule.core, rule.row_count
                    )
                })
        })
        .take(3)
        .collect()
}
fn project_prompt_body_candidate(record: &GoldenSampleLibraryRecord) -> PromptBodyCandidate {
    let blocked = validators::contains_placeholder_marker(&record.source_fields.prompt_body)
        || record.validator_evidence.has_placeholder_signal;
    PromptBodyCandidate {
        source_sample_id: record.sample_id.clone(),
        source_prompt_body: record.source_fields.prompt_body.clone(),
        candidate_text: (!blocked).then(|| record.source_fields.prompt_body.clone()),
        blocked,
        blocker_codes: blocked
            .then(|| vec!["prompt_body_blocked_by_placeholder".to_string()])
            .unwrap_or_default(),
    }
}

fn project_scene_performance(record: &GoldenSampleLibraryRecord) -> ScenePerformanceProjection {
    ScenePerformanceProjection {
        source_sample_id: record.sample_id.clone(),
        source_sample_title: record.source_fields.sample_title.clone(),
        scene_scale: derive_scene_scale(&record.source_fields.technical_profile),
        person: "not_specified_by_v120_bridge".to_string(),
        visual_description: record.source_fields.scene_performance_core.clone(),
        character_action: "fused_scene_performance_core_preserved".to_string(),
        fused_source_text: record.source_fields.scene_performance_core.clone(),
        sequence_grouping: project_sequence_grouping(record),
    }
}

fn project_sequence_grouping(record: &GoldenSampleLibraryRecord) -> SequenceGrouping {
    let structure_mode = StructureMode::from_sample_type(&record.source_fields.sample_type);
    match structure_mode {
        StructureMode::SingleShot => SequenceGrouping {
            structure_mode,
            sequence_id: None,
            shot_order: None,
            sequence_field_state: SequenceFieldState::NotApplicable,
        },
        StructureMode::SequenceShot => {
            let sequence_id = non_blank_string(&record.source_fields.sequence_id);
            let shot_order = record.source_fields.shot_order.trim().parse::<u32>().ok();
            SequenceGrouping {
                structure_mode,
                sequence_id,
                shot_order,
                sequence_field_state: if shot_order.is_some() {
                    SequenceFieldState::Present
                } else {
                    SequenceFieldState::Missing
                },
            }
        }
    }
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
            content_cache_key: stable_hash_hex(&format!(
                "{}\n{}\n{}\n{}\n{}\n{}",
                request.scene_type,
                request.synopsis_text,
                request.duration_seconds,
                request.task_type.as_str(),
                request.shot_intent.as_deref().unwrap_or_default(),
                request.structure_type.as_deref().unwrap_or_default()
            )),
            task_type: request.task_type.as_str().to_string(),
            top_k_samples: 0,
            top_k_rules: 0,
            duration_seconds: request.duration_seconds,
            story_keywords: derive_story_keywords(&request.synopsis_text, &request.scene_type),
            selection_reasons: vec![],
            excluded_candidates: vec![],
            token_budget: KbRouterTokenBudget {
                kb_context_summary_target: "800-1500 Chinese characters".to_string(),
                full_kb_rows_included: 0,
            },
        },
    }
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn extract_scene_type(script_text: &str) -> String {
    script_text
        .lines()
        .find_map(|line| {
            line.strip_prefix("scene_type:")
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_default()
}

fn extract_synopsis_text(script_text: &str) -> String {
    script_text
        .lines()
        .find_map(|line| {
            line.strip_prefix("synopsis:")
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| script_text.trim().to_string())
}

fn is_supported_storyboard_duration(duration_seconds: u16) -> bool {
    matches!(duration_seconds, 5 | 10 | 15 | 30 | 45 | 60)
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

fn serialize_storyboard_rows(rows: &[GeneratedStoryboardRow]) -> String {
    rows.iter()
        .map(|row| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{:?}|{}|{}|{}",
                row.shot_id,
                row.order,
                row.person,
                row.shot_title,
                row.visual_description,
                row.character_action,
                row.dialogue,
                row.prompt_text,
                row.prompt_text_compilation_status,
                row.prompt_text_source_row_id,
                row.duration_seconds,
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
    task_id: Option<String>,
    selected_total_duration_seconds: u16,
    blockers: Vec<ProductWarning>,
    now_ms: u64,
    router_request: &KbRouterRuntimeRequest,
    kb_runtime: &project_store::KbRuntimeHandle,
) -> GenerateStoryboardResponse {
    GenerateStoryboardResponse {
        task_id,
        result_id: String::new(),
        rows: vec![],
        selected_total_duration_seconds,
        kb_router_result: empty_kb_router_response(router_request, kb_runtime),
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
    }
}

fn stable_hash_hex<T: Hash>(value: &T) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn build_storyboard_preview_plan(
    state: &AppState,
    request: StoryboardPreviewPlanRequest,
) -> Result<StoryboardPlan, StoryboardPlanningError> {
    let scene_taxonomy = resolve_scene_taxonomy(state, request.scene_type.as_deref());

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
        scene_director_id: request.scene_director_id,
        action_director_id: request.action_director_id,
        scene_taxonomy,
        layout_prompt: request.layout_prompt,
        render_prompt: request.render_prompt,
    })
}

pub fn build_validation_export_panel_snapshot(
    state: &AppState,
    request: ValidationExportPanelSnapshotRequest,
) -> io::Result<ValidationExportPanelSnapshot> {
    let fixture = load_week3_shared_fixture(WEEK3_SHARED_FIXTURE_PATH)?;
    build_validation_export_panel_snapshot_from_fixture(state, request, &fixture)
}

pub fn resolve_scene_taxonomy(
    state: &AppState,
    scene_type: Option<&str>,
) -> Option<core_domain::SceneTaxonomyRecord> {
    let scene_type = scene_type?;
    let normalized_scene_type = normalize_scene_type(scene_type);

    state
        .kb_knowledge
        .scene_taxonomies
        .iter()
        .find(|taxonomy| {
            taxonomy.scene_type == scene_type
                || taxonomy.display_name == scene_type
                || taxonomy.scene_taxonomy_id == scene_type
                || taxonomy.scene_type == normalized_scene_type
                || taxonomy.display_name == normalized_scene_type
                || taxonomy.scene_taxonomy_id == normalized_scene_type
        })
        .cloned()
}

fn normalize_scene_type(scene_type: &str) -> String {
    match scene_type.trim() {
        "war_formation" | "weapon_highlight" | "council_strategy" | "slg_sandbox_view"
        | "slg_city_growth" | "battle_report_ui" | "multi_army_siege" => {
            "daily_dialogue".to_string()
        }
        other => other.to_string(),
    }
}

fn build_validation_export_panel_snapshot_from_fixture(
    state: &AppState,
    request: ValidationExportPanelSnapshotRequest,
    fixture: &Week3SharedFixture,
) -> io::Result<ValidationExportPanelSnapshot> {
    let workbook = generate_week3_validation_report(fixture)?;
    let repair_recommendations = generate_week3_repair_recommendations(
        fixture,
        &state.kb_knowledge.failure_patterns,
        &state.kb_knowledge.prompt_templates,
    )?;

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
                state.kb_runtime.snapshot.snapshot_id
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
    use std::collections::BTreeMap;
    use std::env;
    use std::fs;

    use core_domain::{
        BridgeCallStatus, ExpandScriptRequest, ExportBundleRequest, FailurePatternRecord,
        GenerateStoryboardRequest, GoldenSampleAssetSource, GoldenSampleClassification,
        GoldenSampleComparisonBaseline, GoldenSampleCoverageSummary,
        GoldenSampleFailureCodeDefinition, GoldenSampleFailureMappingAsset,
        GoldenSampleFailureMappingRecord, GoldenSampleFailureValidatorEvidence,
        GoldenSampleFewshotGate, GoldenSampleFewshotState, GoldenSampleFieldCoverageRuleAsset,
        GoldenSampleFieldCoverageRuleRecord, GoldenSampleLibraryAsset,
        GoldenSampleLibraryProvenance, GoldenSampleLibraryRecord, GoldenSampleNegativeSample,
        GoldenSampleNegativeSampleGate, GoldenSampleRepairInputs, GoldenSampleRepairMappingAsset,
        GoldenSampleRepairMappingPlanning, GoldenSampleRepairMappingRecord,
        GoldenSampleSourceContext, GoldenSampleSourceFields, GoldenSampleSourceRegister,
        GoldenSampleSourceRegisterEntry, GoldenSampleSourceRegisterProvenance,
        GoldenSampleV3CoreCoverage, GoldenSampleValidatorEvidence, KbBundleManifestRecord,
        KbBundleRecordCounts, KbGoldenSampleRuntimePackage, KbRouterRuntimeRequest,
        KbRouterTaskType, KbRuntimeSummary, KbSnapshotRecord, PromptTemplateRecord,
        PromptTextCompilationStatus, SceneTaxonomyRecord, StoryboardDurationPlan,
        TextGenerationOutputSchema, TextGenerationTask, TextModelProviderKind,
        UpdateStoryboardRowsRequest,
    };
    use project_store::{
        DualSqliteConnectionPolicy, KbKnowledgeBundle, KbRuntimeHandle, StoreSkeleton,
    };
    use serde_json::json;

    use super::{
        StoryboardPreviewPlanRequest, ValidationExportPanelSnapshotRequest,
        ValidationExportPanelState, build_prompt_text_compilation_request,
        build_qwen_request_payload, build_storyboard_preview_plan, build_text_generation_request,
        build_validation_export_panel_snapshot_from_fixture, compile_seedance_prompt_text,
        default_text_model_provider, expand_script, export_bundle, generate_storyboard,
        project_scene_performance, resolve_scene_taxonomy, run_kb_router,
        run_qwen_text_generation_with_transport, save_storyboard_rows,
        select_golden_sample_records,
    };
    use crate::state::AppState;
    use validators::{WEEK3_SHARED_FIXTURE_PATH, load_week3_shared_fixture};

    fn test_state() -> AppState {
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            "E:/codex/hope-kb/snapshots/hope-kb-v0.1.sqlite3".into(),
            "E:/codex/hope/data/hope.sqlite3".into(),
        ));
        let golden_sample_package = test_golden_sample_package();
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
                has_golden_sample_v120_package: true,
                golden_sample_record_count: golden_sample_package
                    .manifest
                    .record_counts
                    .golden_sample_library,
                golden_sample_source_count: golden_sample_package
                    .manifest
                    .record_counts
                    .golden_sample_sources,
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

        AppState::new(store, kb_runtime, kb_knowledge, golden_sample_package)
    }

    #[derive(Debug)]
    struct EnvGuard {
        name: &'static str,
        original: Option<String>,
    }

    impl EnvGuard {
        fn set(name: &'static str, value: &str) -> Self {
            let original = env::var(name).ok();
            unsafe { env::set_var(name, value) };
            Self { name, original }
        }

        fn unset(name: &'static str) -> Self {
            let original = env::var(name).ok();
            unsafe { env::remove_var(name) };
            Self { name, original }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(value) => unsafe { env::set_var(self.name, value) },
                None => unsafe { env::remove_var(self.name) },
            }
        }
    }

    fn test_golden_sample_package() -> KbGoldenSampleRuntimePackage {
        let official_count = 108usize;
        let reserve_count = 44usize;
        let total_count = official_count + reserve_count;
        let mut records = Vec::with_capacity(total_count);
        let mut failure_records = Vec::with_capacity(total_count);
        let mut repair_records = Vec::with_capacity(total_count);

        for index in 0..total_count {
            let is_official = index < official_count;
            let record = test_golden_sample_record(index + 1, is_official);
            failure_records.push(test_failure_mapping_record(index + 1, &record));
            repair_records.push(test_repair_mapping_record(index + 1, &record));
            records.push(record);
        }

        KbGoldenSampleRuntimePackage {
            manifest: KbBundleManifestRecord {
                snapshot_version: "v0.2".to_string(),
                snapshot_name: "hope-kb-golden-sample-library-v0.2".to_string(),
                content_hash_algo: "sha256".to_string(),
                content_hash: "bundle-sha256:test".to_string(),
                seed_import_format: "hope-kb-golden-sample-seed-bundle-v0.2".to_string(),
                imported_at: "2026-04-23".to_string(),
                primary_key: "machine_id".to_string(),
                bundle_order: vec![
                    "seed/v0.2/manifest.json".to_string(),
                    "seed/v0.2/golden_sample_library.json".to_string(),
                    "seed/v0.2/golden_sample_field_coverage_rules.json".to_string(),
                    "seed/v0.2/golden_sample_failure_mapping.json".to_string(),
                    "seed/v0.2/golden_sample_repair_mapping.json".to_string(),
                    "seed/v0.2/source_register.json".to_string(),
                ],
                record_counts: KbBundleRecordCounts {
                    golden_sample_library: total_count,
                    golden_sample_field_coverage_rules: 5,
                    golden_sample_failure_mapping: total_count,
                    golden_sample_repair_mapping: total_count,
                    golden_sample_sources: 14,
                    golden_sample_provenance_entries: 4,
                },
            },
            golden_sample_library: GoldenSampleLibraryAsset {
                schema_version: "golden_sample_library.v0.2".to_string(),
                asset_name: "golden_sample_library".to_string(),
                generated_at: "2026-04-23".to_string(),
                source: GoldenSampleAssetSource {
                    primary_workbook: "workbook.xlsx".to_string(),
                    primary_workbook_sha256: "hash".to_string(),
                    control_memo: "memo.docx".to_string(),
                    control_memo_sha256: "hash".to_string(),
                    control_review: "review.md".to_string(),
                    control_dispatch: "dispatch.md".to_string(),
                    comparison_baseline_source_id: "v108".to_string(),
                },
                record_count: total_count,
                source_field_order: vec![
                    "shot_id".to_string(),
                    "library_status".to_string(),
                    "sample_type".to_string(),
                    "scene_category".to_string(),
                    "prompt_body".to_string(),
                ],
                records,
            },
            field_coverage_rules: GoldenSampleFieldCoverageRuleAsset {
                schema_version: "golden_sample_field_coverage_rules.v0.2".to_string(),
                asset_name: "golden_sample_field_coverage_rules".to_string(),
                generated_at: "2026-04-23".to_string(),
                source_context: GoldenSampleSourceContext {
                    primary_source_id: "v120".to_string(),
                    comparison_source_id: "v108".to_string(),
                },
                record_count: 5,
                records: test_field_coverage_rules(total_count as u32, official_count as u32),
            },
            failure_mapping: GoldenSampleFailureMappingAsset {
                schema_version: "golden_sample_failure_mapping.v0.2".to_string(),
                asset_name: "golden_sample_failure_mapping".to_string(),
                generated_at: "2026-04-23".to_string(),
                source_context: GoldenSampleSourceContext {
                    primary_source_id: "v120".to_string(),
                    comparison_source_id: "v108".to_string(),
                },
                failure_code_definitions: vec![GoldenSampleFailureCodeDefinition {
                    failure_code: "golden_coverage_gap".to_string(),
                    source_fields: vec!["missed_points".to_string()],
                    meaning: "coverage gap".to_string(),
                }],
                record_count: total_count,
                records: failure_records,
            },
            repair_mapping: GoldenSampleRepairMappingAsset {
                schema_version: "golden_sample_repair_mapping.v0.2".to_string(),
                asset_name: "golden_sample_repair_mapping".to_string(),
                generated_at: "2026-04-23".to_string(),
                source_context: GoldenSampleSourceContext {
                    primary_source_id: "v120".to_string(),
                    comparison_source_id: "v108".to_string(),
                },
                record_count: total_count,
                records: repair_records,
            },
            source_register: GoldenSampleSourceRegister {
                schema_version: "hope-kb-v0.2-source-register".to_string(),
                register_name: "hope-kb-v0.2-source-register".to_string(),
                updated_at: "2026-04-23".to_string(),
                sources: (1..=14)
                    .map(|index| GoldenSampleSourceRegisterEntry {
                        source_id: format!("v120_source_{index:02}"),
                        source_type: "immutable_raw_xlsx".to_string(),
                        label: format!("V120 Source {index:02}"),
                        path: format!("sources/workbook_{index:02}.xlsx"),
                        sha256: Some(format!("hash-{index:02}")),
                        applies_to: vec!["golden_sample_library".to_string()],
                        notes: "test source".to_string(),
                    })
                    .collect(),
                provenance_entries: (1..=4)
                    .map(|index| GoldenSampleSourceRegisterProvenance {
                        provenance_id: format!("v120_package_{index:02}"),
                        source_ids: vec![format!("v120_source_{index:02}")],
                        generated_files: vec![format!(
                            "seed/v0.2/golden_sample_library_part_{index:02}.json"
                        )],
                        preservation_contract: BTreeMap::from([(
                            "imports_into_live_hope_runtime".to_string(),
                            false,
                        )]),
                    })
                    .collect(),
            },
        }
    }

    fn test_golden_sample_record(index: usize, is_official: bool) -> GoldenSampleLibraryRecord {
        let sample_id = if index == 1 {
            "GS-BRIDGE-01".to_string()
        } else {
            format!("GS-BRIDGE-{index:03}")
        };
        let machine_id = if index == 1 {
            "golden_sample_bridge_01".to_string()
        } else {
            format!("golden_sample_bridge_{index:03}")
        };
        let library_status = if is_official { "official" } else { "reserve" };
        let reserve_reason = if is_official {
            String::new()
        } else {
            "new_kb152_candidate".to_string()
        };
        let usable_for_fewshot = if is_official { "Yes" } else { "No" };
        let reference_bundle = if index == 1 {
            "scene_room(layout,strong) char_lead(face,reference) plate_url(url,strong) plate_uri(uri,strong) plate_media(media,strong) plate_asset(asset,strong) plate_file(file,strong) C:/refs/chair(layout,strong)".to_string()
        } else {
            format!("scene_room_{index}(layout,strong) char_lead_{index}(face,reference)")
        };

        GoldenSampleLibraryRecord {
            machine_id,
            sample_id: sample_id.clone(),
            schema_version: "golden_sample_library.v0.2".to_string(),
            source_fields: GoldenSampleSourceFields {
                shot_id: sample_id.clone(),
                library_status: library_status.to_string(),
                reserve_reason: reserve_reason.clone(),
                sample_type: "single_shot".to_string(),
                sequence_id: String::new(),
                shot_order: String::new(),
                sample_title: format!("Bridge dialogue sample {index:03}"),
                style_cluster: "dialogue".to_string(),
                scene_category: "daily_dialogue".to_string(),
                scene_tag: "daily_dialogue,interior".to_string(),
                quality_grade: "good".to_string(),
                usable_for_fewshot: usable_for_fewshot.to_string(),
                technical_profile: "duration 8s / MCU / 24fps".to_string(),
                scene_performance_core: format!(
                    "Two people hold a restrained dialogue beat {index:03}."
                ),
                camera_directing_core: "Hold a stable medium close composition.".to_string(),
                audio_directing_core: "Room tone and breath stay low.".to_string(),
                continuity_negative_core: "Do not drift eyeline or prop handoff.".to_string(),
                reference_bundle,
                ip_abstraction_note: "abstracted".to_string(),
                covered_points: "dialogue / eyeline".to_string(),
                missed_points: String::new(),
                teaching_note: "Use as bridge evidence only.".to_string(),
                prompt_body: format!(
                    "Compose a restrained dialogue shot with stable eyeline {index:03}."
                ),
            },
            provenance: GoldenSampleLibraryProvenance {
                source_workbook: format!("workbook_{index:03}.xlsx"),
                source_workbook_sha256: format!("hash-{index:03}"),
                source_control_memo: "memo.docx".to_string(),
                source_control_memo_sha256: "hash".to_string(),
                source_row_index: index as u32,
                source_library_status: library_status.to_string(),
                source_sample_type: "single_shot".to_string(),
                comparison_baseline_source_id: "v108".to_string(),
                control_review_doc: "review.md".to_string(),
                control_dispatch_doc: "dispatch.md".to_string(),
            },
            classification: GoldenSampleClassification {
                core: "full_stack_single_shot_sample".to_string(),
                library_status: library_status.to_string(),
                sample_type: "single_shot".to_string(),
                sequence_id: String::new(),
                shot_order: String::new(),
                style_cluster: "dialogue".to_string(),
                scene_category: "daily_dialogue".to_string(),
                scene_tags: vec!["daily_dialogue".to_string()],
                quality_grade: "good".to_string(),
                usable_for_fewshot: is_official,
                coverage_surfaces: vec![
                    "scene_performance_core".to_string(),
                    "continuity_negative_core".to_string(),
                ],
            },
            fewshot: GoldenSampleFewshotState {
                eligible: is_official,
                source_value: usable_for_fewshot.to_string(),
                retrieval_status: if is_official {
                    "candidate_positive_fewshot".to_string()
                } else {
                    "reserve_candidate_only".to_string()
                },
            },
            validator_evidence: GoldenSampleValidatorEvidence {
                covered_points: "dialogue / eyeline".to_string(),
                missed_points: String::new(),
                teaching_note: "Use as bridge evidence only.".to_string(),
                source_quality_grade: "good".to_string(),
                source_library_status: library_status.to_string(),
                source_sample_type: "single_shot".to_string(),
                has_coverage_gap: false,
                has_placeholder_signal: false,
                surface_completeness: BTreeMap::from([
                    ("scene_performance_core".to_string(), true),
                    ("continuity_negative_core".to_string(), true),
                ]),
                reference_bundle_present: true,
            },
            negative_sample: GoldenSampleNegativeSample {
                is_negative_sample: false,
                signal_codes: vec![],
                reserve_reason,
            },
            v3_core_coverage: GoldenSampleV3CoreCoverage {
                core: "full_stack_single_shot_sample".to_string(),
                coverage_rule_ids: vec!["gs_field_rule_scene_performance_core".to_string()],
                source_coverage_statement: "covered".to_string(),
                source_missing_statement: String::new(),
                source_field_presence: BTreeMap::from([
                    ("scene_performance_core".to_string(), true),
                    ("continuity_negative_core".to_string(), true),
                ]),
                comparison_baseline: GoldenSampleComparisonBaseline {
                    primary_source_id: "v120".to_string(),
                    comparison_source_id: "v108".to_string(),
                },
            },
            repair_mapping_planning: GoldenSampleRepairMappingPlanning {
                failure_mapping_id: format!("failure_map_{index:03}"),
                repair_mapping_id: format!("repair_map_{index:03}"),
                planning_only: true,
            },
        }
    }

    fn test_field_coverage_rules(
        total_count: u32,
        official_count: u32,
    ) -> Vec<GoldenSampleFieldCoverageRuleRecord> {
        let reserve_count = total_count - official_count;
        [
            "scene_performance_core",
            "camera_directing_core",
            "audio_directing_core",
            "continuity_negative_core",
            "prompt_body",
        ]
        .into_iter()
        .map(|core| GoldenSampleFieldCoverageRuleRecord {
            rule_id: format!("gs_field_rule_{core}"),
            core: core.to_string(),
            row_count: total_count as usize,
            source_sample_ids: vec!["GS-BRIDGE-01".to_string()],
            required_source_fields: vec![core.to_string()],
            validator_evidence_fields: vec!["covered_points".to_string()],
            fewshot_gate: GoldenSampleFewshotGate {
                eligible_source_field: "usable_for_fewshot".to_string(),
                positive_value: "Yes".to_string(),
                negative_value: "No".to_string(),
                library_status_field: "library_status".to_string(),
                official_value: "official".to_string(),
                reserve_value: "reserve".to_string(),
                reserve_rows_excluded_from_positive_fewshot: true,
                negative_quality_value: "bad".to_string(),
            },
            negative_sample_gate: GoldenSampleNegativeSampleGate {
                quality_field: "quality_grade".to_string(),
                negative_quality_value: "bad".to_string(),
                library_status_field: "library_status".to_string(),
                reserve_value: "reserve".to_string(),
                reserve_reason_field: "reserve_reason".to_string(),
            },
            coverage_summary: GoldenSampleCoverageSummary {
                library_status: BTreeMap::from([
                    ("official".to_string(), official_count),
                    ("reserve".to_string(), reserve_count),
                ]),
                quality_grade: BTreeMap::from([("good".to_string(), total_count)]),
                sample_type: BTreeMap::from([("single_shot".to_string(), total_count)]),
                scene_category: BTreeMap::from([("daily_dialogue".to_string(), total_count)]),
                sequence_groups: BTreeMap::new(),
                style_cluster: BTreeMap::from([("dialogue".to_string(), total_count)]),
                surface_completeness: BTreeMap::from([(core.to_string(), total_count)]),
                usable_for_fewshot: BTreeMap::from([
                    ("Yes".to_string(), official_count),
                    ("No".to_string(), reserve_count),
                ]),
            },
            future_gate_owner: vec!["Hope runtime ingest planning".to_string()],
            planning_only: true,
        })
        .collect()
    }

    fn test_failure_mapping_record(
        index: usize,
        record: &GoldenSampleLibraryRecord,
    ) -> GoldenSampleFailureMappingRecord {
        GoldenSampleFailureMappingRecord {
            mapping_id: format!("failure_map_{index:03}"),
            sample_id: record.sample_id.clone(),
            core: record.classification.core.clone(),
            tier: record.classification.quality_grade.clone(),
            usable_for_fewshot: record.source_fields.usable_for_fewshot.clone(),
            negative_sample_signal: false,
            planned_failure_codes: vec![],
            validator_evidence: GoldenSampleFailureValidatorEvidence {
                covered_points: record.validator_evidence.covered_points.clone(),
                missed_points: record.validator_evidence.missed_points.clone(),
                teaching_note: record.validator_evidence.teaching_note.clone(),
                library_status: record.classification.library_status.clone(),
                sample_type: record.classification.sample_type.clone(),
                sequence_id: record.classification.sequence_id.clone(),
                shot_order: record.classification.shot_order.clone(),
                reference_bundle_present: record.validator_evidence.reference_bundle_present,
            },
            source_field_refs: vec!["covered_points".to_string()],
        }
    }

    fn test_repair_mapping_record(
        index: usize,
        record: &GoldenSampleLibraryRecord,
    ) -> GoldenSampleRepairMappingRecord {
        GoldenSampleRepairMappingRecord {
            mapping_id: format!("repair_map_{index:03}"),
            sample_id: record.sample_id.clone(),
            core: record.classification.core.clone(),
            repair_planning_mode: if record.is_positive_fewshot_candidate() {
                "positive_fewshot_candidate".to_string()
            } else {
                "reserve_candidate_only".to_string()
            },
            linked_failure_mapping_id: format!("failure_map_{index:03}"),
            planned_repair_inputs: GoldenSampleRepairInputs {
                library_status: record.classification.library_status.clone(),
                missed_points: record.validator_evidence.missed_points.clone(),
                reserve_reason: record.negative_sample.reserve_reason.clone(),
                sample_type: record.classification.sample_type.clone(),
                scene_category: record.classification.scene_category.clone(),
                sequence_id: record.classification.sequence_id.clone(),
                style_cluster: record.classification.style_cluster.clone(),
                teaching_note: record.validator_evidence.teaching_note.clone(),
                usable_for_fewshot: record.source_fields.usable_for_fewshot.clone(),
            },
            future_repair_gate: "fewshot_retrieval_gate".to_string(),
            planning_only: true,
        }
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
    fn resolve_scene_taxonomy_accepts_desktop_scene_type_aliases() {
        let state = test_state();

        for scene_type in [
            "war_formation",
            "weapon_highlight",
            "council_strategy",
            "slg_sandbox_view",
            "slg_city_growth",
            "battle_report_ui",
            "multi_army_siege",
        ] {
            let resolved = resolve_scene_taxonomy(&state, Some(scene_type));
            assert_eq!(
                resolved.as_ref().map(|item| item.scene_type.as_str()),
                Some("daily_dialogue"),
                "{scene_type} should resolve into the current MVP canonical bucket",
            );
        }
    }

    #[test]
    fn kb_router_runtime_keeps_top_k_summary_and_reserve_gate_bounded() {
        let state = test_state();
        let router_result = run_kb_router(
            &state,
            KbRouterRuntimeRequest {
                scene_type: "daily_dialogue".to_string(),
                synopsis_text: "Two leads negotiate quietly before dawn and keep continuity tight."
                    .to_string(),
                duration_seconds: 60,
                task_type: KbRouterTaskType::GenerateStoryboard,
                shot_intent: Some("dialogue".to_string()),
                structure_type: Some("single_shot".to_string()),
            },
        );

        assert!((3..=5).contains(&router_result.selected_sample_ids.len()));
        assert_eq!(
            router_result
                .retrieval_trace
                .token_budget
                .full_kb_rows_included,
            0
        );
        assert!(router_result.kb_context_summary.chars().count() >= 800);
        assert!(router_result.kb_context_summary.chars().count() <= 1500);
        assert!(
            !router_result
                .kb_context_summary
                .contains("TODO: fill prompt")
        );
        assert!(
            !router_result
                .kb_context_summary
                .contains("reserve evidence")
        );
        assert!(
            router_result
                .retrieval_trace
                .excluded_candidates
                .iter()
                .any(|candidate| candidate.reason_code == "reserve_gate")
        );
    }

    #[test]
    fn v120_consumer_accepts_152_package_without_promoting_reserve_rows() {
        let state = test_state();
        let package = &state.kb_golden_sample_runtime;

        assert_eq!(state.kb_runtime.summary.golden_sample_record_count, 152);
        assert_eq!(package.manifest_golden_sample_record_count(), 152);
        assert_eq!(package.official_record_count(), 108);
        assert_eq!(package.reserve_record_count(), 44);
        assert_eq!(package.positive_fewshot_record_count(), 108);
        assert!(
            package
                .golden_sample_library
                .records
                .iter()
                .filter(|record| record.is_reserve())
                .all(|record| record.source_fields.usable_for_fewshot == "No")
        );

        let selected = select_golden_sample_records(
            &state,
            "daily_dialogue bridge script",
            &GenerateStoryboardRequest {
                task_name: "daily_dialogue_scene".to_string(),
                script_id: Some("script-test".to_string()),
                expanded_script_text: None,
                selected_total_duration_seconds: 15,
            },
        );

        assert_eq!(selected.len(), 1);
        assert!(selected.iter().all(|record| record.is_official()));
    }

    #[test]
    fn text_model_and_prompt_compilation_boundaries_stay_stubbed_and_qwen_first() {
        let _enabled = EnvGuard::set("HOPE_TEXT_MODEL_ENABLED", "false");
        let state = test_state();
        let record = &state.kb_golden_sample_runtime.golden_sample_library.records[0];
        let scene_projection = project_scene_performance(record);
        let provider = default_text_model_provider();

        assert_eq!(provider.provider, TextModelProviderKind::Qwen);
        assert_eq!(provider.model, "qwen-default-text");
        assert!(!provider.enabled);
        let kb_router_result = run_kb_router(
            &state,
            KbRouterRuntimeRequest {
                scene_type: "daily_dialogue".to_string(),
                synopsis_text: "Two leads negotiate quietly before dawn.".to_string(),
                duration_seconds: 8,
                task_type: KbRouterTaskType::CompileSeedancePromptText,
                shot_intent: None,
                structure_type: None,
            },
        );

        let response = compile_seedance_prompt_text(
            &state,
            build_prompt_text_compilation_request(record, &scene_projection, 8, &kb_router_result),
        );

        assert_eq!(response.target_profile, "seedance2.0");
        assert_eq!(
            response.compilation_status,
            PromptTextCompilationStatus::ReadyStub
        );
        assert_eq!(response.source_row_id, "GS-BRIDGE-01");
        assert_eq!(response.provider, TextModelProviderKind::Qwen);
        assert_eq!(response.model, "qwen-default-text");
        assert!(response.prompt_text.contains("镜头标题"));
        assert!(
            response
                .prompt_text
                .contains("目标适配：Seedance2.0 文本提示词")
        );
        assert!(
            response
                .prompt_text
                .contains("连续性约束：Do not drift eyeline or prop handoff.")
        );
        assert!(
            !response
                .prompt_text
                .contains("Compose a restrained dialogue shot with stable eyeline")
        );
        assert!(
            response
                .warnings
                .iter()
                .any(|warning| warning.code == "text_model_live_call_closed")
        );
        assert!(
            response
                .warnings
                .iter()
                .any(|warning| warning.code == "seedance_video_generation_closed")
        );
    }

    #[test]
    fn qwen_payload_uses_router_summary_instead_of_full_kb_rows() {
        let provider = core_domain::TextModelProvider {
            provider: TextModelProviderKind::Qwen,
            model: "qwen-test".to_string(),
            base_url: Some("https://example.invalid/v1".to_string()),
            api_key_ref: "env:HOPE_TEXT_MODEL_API_KEY".to_string(),
            enabled: true,
        };
        let request = build_text_generation_request(
            TextGenerationTask::GenerateStoryboard,
            Some("daily_dialogue".to_string()),
            "scene_type: daily_dialogue\nsynopsis: quiet negotiation".to_string(),
            Some(StoryboardDurationPlan {
                total_duration_seconds: 10,
                row_count: 3,
                per_row_seconds: 3,
                allocated_seconds: 10,
            }),
            "compressed kb summary only".to_string(),
            vec!["GS-BRIDGE-01".to_string(), "GS-BRIDGE-02".to_string()],
            vec![
                "coverage-scene_performance:preserve action core".to_string(),
                "reserve_gate:reserve rows stay evidence-only".to_string(),
            ],
            TextGenerationOutputSchema::StoryboardRowsJson,
            Some(512),
        );

        let payload = build_qwen_request_payload(&provider, &request);
        let payload_text = payload.to_string();

        assert!(payload_text.contains("compressed kb summary only"));
        assert!(payload_text.contains("coverage-scene_performance"));
        assert!(payload_text.contains("reserve_gate"));
        assert!(!payload_text.contains("source_register"));
        assert!(!payload_text.contains("\"full_kb_rows_included\":152"));
        assert!(!payload_text.contains("Compose a restrained dialogue shot with stable eyeline"));
    }

    #[test]
    fn qwen_provider_mock_transport_returns_structured_storyboard_rows() {
        let provider = core_domain::TextModelProvider {
            provider: TextModelProviderKind::Qwen,
            model: "qwen-test".to_string(),
            base_url: Some("https://example.invalid/v1".to_string()),
            api_key_ref: "env:HOPE_TEXT_MODEL_API_KEY".to_string(),
            enabled: true,
        };
        let request = build_text_generation_request(
            TextGenerationTask::GenerateStoryboard,
            Some("daily_dialogue".to_string()),
            "scene_type: daily_dialogue\nsynopsis: quiet negotiation".to_string(),
            Some(StoryboardDurationPlan {
                total_duration_seconds: 10,
                row_count: 3,
                per_row_seconds: 3,
                allocated_seconds: 10,
            }),
            "compressed kb summary only".to_string(),
            vec!["GS-BRIDGE-01".to_string()],
            vec!["coverage-scene_performance:preserve action core".to_string()],
            TextGenerationOutputSchema::StoryboardRowsJson,
            Some(512),
        );

        let response = run_qwen_text_generation_with_transport(
            &provider,
            &request,
            "https://example.invalid/v1/chat/completions",
            "test-key",
            |endpoint, api_key, payload| {
                assert_eq!(endpoint, "https://example.invalid/v1/chat/completions");
                assert_eq!(api_key, "test-key");
                let body_text = payload.to_string();
                assert!(body_text.contains("compressed kb summary only"));
                assert!(body_text.contains("GS-BRIDGE-01"));
                assert!(body_text.contains("coverage-scene_performance"));
                assert!(!body_text.contains("test-key"));
                Ok((
                    json!({
                        "choices": [{
                            "message": {
                                "content": "{\"rows\":[{\"shot_title\":\"Live title\",\"person\":\"Lead\",\"scene_scale\":\"MCU\",\"visual_description\":\"Live visual\",\"character_action\":\"Live action\",\"dialogue\":\"Live dialogue\"}]}"
                            }
                        }],
                        "usage": {
                            "total_tokens": 321
                        }
                    }),
                    12,
                ))
            },
        )
        .expect("mock transport should succeed");

        assert_eq!(response.provider, TextModelProviderKind::Qwen);
        assert_eq!(response.model, "qwen-test");
        assert_eq!(response.usage_tokens, Some(321));
        assert!(response.structured_json.is_some());
        assert!(response.text.contains("\"rows\""));
    }

    #[test]
    fn enabled_qwen_without_api_key_falls_back_to_stub() {
        let _enabled = EnvGuard::set("HOPE_TEXT_MODEL_ENABLED", "true");
        let _provider = EnvGuard::set("HOPE_TEXT_MODEL_PROVIDER", "qwen");
        let _base_url = EnvGuard::set("HOPE_TEXT_MODEL_BASE_URL", "https://example.invalid/v1");
        let _api_key_ref =
            EnvGuard::set("HOPE_TEXT_MODEL_API_KEY_REF", "env:HOPE_TEXT_MODEL_API_KEY");
        let _missing_key = EnvGuard::unset("HOPE_TEXT_MODEL_API_KEY");

        let state = test_state();
        let script = expand_script(
            &state,
            ExpandScriptRequest {
                scene_type: "daily_dialogue".to_string(),
                synopsis_text: "Two leads negotiate quietly before dawn.".to_string(),
            },
        );

        assert_eq!(script.status, BridgeCallStatus::WarningOnly);
        assert!(script.expanded_script_text.contains("source_package:"));
        assert!(
            script
                .warnings
                .iter()
                .any(|warning| warning.code == "text_model_api_key_missing")
        );
        assert!(
            script
                .warnings
                .iter()
                .any(|warning| warning.code == "text_model_live_expand_fallback")
        );
    }

    #[test]
    fn v120_bridge_expands_generates_and_exports_without_live_model() {
        let _enabled = EnvGuard::set("HOPE_TEXT_MODEL_ENABLED", "false");
        let state = test_state();
        let script = expand_script(
            &state,
            ExpandScriptRequest {
                scene_type: "daily_dialogue".to_string(),
                synopsis_text: "Two leads negotiate quietly before dawn.".to_string(),
            },
        );

        assert!(script.script_id.starts_with("script-"));
        assert!(script.expanded_script_text.contains("daily_dialogue"));
        assert!(
            script
                .warnings
                .iter()
                .any(|warning| warning.code == "text_model_live_call_closed")
        );

        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "daily_dialogue_scene".to_string(),
                script_id: Some(script.script_id.clone()),
                expanded_script_text: None,
                selected_total_duration_seconds: 10,
            },
        );

        assert_eq!(state.kb_runtime.summary.golden_sample_record_count, 152);
        assert_eq!(storyboard.rows.len(), 3);
        assert_eq!(storyboard.kb_router_result.selected_sample_ids.len(), 3);
        assert_eq!(
            storyboard
                .kb_router_result
                .retrieval_trace
                .token_budget
                .full_kb_rows_included,
            0
        );
        assert_eq!(storyboard.rows[0].shot_id, "GS-BRIDGE-01");
        assert_eq!(
            storyboard.rows[0].prompt_text_compilation_status,
            PromptTextCompilationStatus::ReadyStub
        );
        assert_eq!(storyboard.rows[0].prompt_text_source_row_id, "GS-BRIDGE-01");
        assert!(
            storyboard.rows[0]
                .prompt_text
                .contains("目标适配：Seedance2.0 文本提示词")
        );
        assert!(
            !storyboard.rows[0]
                .prompt_text
                .contains("Compose a restrained dialogue shot with stable eyeline")
        );
        assert!(
            storyboard.rows[0]
                .prompt_body_candidate
                .candidate_text
                .is_some()
        );
        assert_eq!(
            storyboard.rows[0].sequence_grouping.sequence_field_state,
            core_domain::SequenceFieldState::NotApplicable
        );
        assert!(
            storyboard.rows[0]
                .external_reference_handle_candidates
                .iter()
                .any(|candidate| candidate.reference_name == "scene_room")
        );
        assert!(
            storyboard.rows[0]
                .external_reference_handle_candidates
                .iter()
                .all(|candidate| !matches!(
                    candidate.reference_kind.as_str(),
                    "path" | "url" | "uri" | "media" | "asset" | "file"
                ))
        );
        for forbidden_name in [
            "plate_url",
            "plate_uri",
            "plate_media",
            "plate_asset",
            "plate_file",
            "chair",
        ] {
            assert!(
                storyboard.rows[0]
                    .external_reference_handle_candidates
                    .iter()
                    .all(|candidate| candidate.reference_name != forbidden_name)
            );
        }
        assert_eq!(
            storyboard.export_status.status,
            BridgeCallStatus::WarningOnly
        );
        assert!(
            storyboard
                .export_status
                .warnings
                .iter()
                .any(|warning| warning.code == "text_model_live_call_closed")
        );
        assert!(
            storyboard
                .export_status
                .warnings
                .iter()
                .any(|warning| warning.code == "seedance_video_generation_closed")
        );
        assert!(
            storyboard.rows[0]
                .prompt_text_compilation_warnings
                .iter()
                .any(|warning| warning.code == "text_model_live_call_closed")
        );

        let export = export_bundle(
            &state,
            ExportBundleRequest {
                result_id: Some(storyboard.result_id.clone()),
                task_id: storyboard.task_id.clone(),
                export_format: "xlsx".to_string(),
            },
        );

        assert_eq!(export.export_status.status, BridgeCallStatus::WarningOnly);
        assert_eq!(export.artifacts.len(), 4);
        assert!(
            export
                .artifacts
                .iter()
                .any(|artifact| artifact.artifact_kind == "v120_bridge_manifest" && artifact.ready)
        );
        for artifact_kind in ["storyboard_json", "storyboard_csv", "excel_workbook"] {
            let artifact = export
                .artifacts
                .iter()
                .find(|artifact| artifact.artifact_kind == artifact_kind)
                .expect("storyboard export artifact should be present");
            assert!(artifact.ready, "{artifact_kind} should be ready");
            assert!(artifact.blocked_reason.is_none());
            assert_eq!(artifact.row_count, Some(3));
            assert!(artifact.byte_size.unwrap_or_default() > 0);
            assert_eq!(artifact.full_kb_rows_included, 0);
            assert_eq!(artifact.selected_sample_ids.len(), 3);
            let path = artifact
                .artifact_path
                .as_deref()
                .expect("ready artifact should expose a file path");
            assert!(std::path::Path::new(path).is_file());
        }

        let json_artifact = export
            .artifacts
            .iter()
            .find(|artifact| artifact.artifact_kind == "storyboard_json")
            .expect("json artifact should be present");
        let json_text = fs::read_to_string(json_artifact.artifact_path.as_ref().unwrap())
            .expect("json artifact should be readable");
        assert!(json_text.contains("ReadyStub"));
        assert!(json_text.contains("text_model_live_call_closed"));
        assert!(json_text.contains("分镜提示词"));
        assert!(!json_text.contains("Compose a restrained dialogue shot with stable eyeline."));
    }

    #[test]
    fn export_bundle_missing_storyboard_result_stays_blocked() {
        let state = test_state();

        let export = export_bundle(
            &state,
            ExportBundleRequest {
                result_id: Some("storyboard-missing".to_string()),
                task_id: None,
                export_format: "xlsx".to_string(),
            },
        );

        assert_eq!(export.export_status.status, BridgeCallStatus::Blocked);
        assert!(
            export
                .export_status
                .blockers
                .iter()
                .any(|blocker| blocker.code == "storyboard_result_not_found")
        );
        assert!(export.artifacts.iter().all(|artifact| !artifact.ready));
        assert!(
            export
                .artifacts
                .iter()
                .all(|artifact| artifact.blocked_reason.as_deref()
                    == Some("storyboard_result_not_found"))
        );
    }

    #[test]
    fn expand_script_blocks_empty_synopsis_without_creating_script_id() {
        let state = test_state();

        let response = expand_script(
            &state,
            ExpandScriptRequest {
                scene_type: "daily_dialogue".to_string(),
                synopsis_text: "   ".to_string(),
            },
        );

        assert_eq!(response.status, BridgeCallStatus::Blocked);
        assert!(response.script_id.is_empty());
        assert!(
            response
                .blockers
                .iter()
                .any(|item| item.code == "synopsis_required")
        );
    }

    #[test]
    fn generate_storyboard_blocks_empty_script_and_invalid_duration() {
        let state = test_state();

        let empty_script = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "empty-script".to_string(),
                script_id: None,
                expanded_script_text: Some("".to_string()),
                selected_total_duration_seconds: 10,
            },
        );
        assert_eq!(empty_script.export_status.status, BridgeCallStatus::Blocked);
        assert!(empty_script.result_id.is_empty());
        assert!(empty_script.rows.is_empty());
        assert!(
            empty_script
                .export_status
                .blockers
                .iter()
                .any(|item| item.code == "task_script_required")
        );

        for duration in [7u16, 20u16, 0u16] {
            let response = generate_storyboard(
                &state,
                GenerateStoryboardRequest {
                    task_name: format!("invalid-duration-{duration}"),
                    script_id: None,
                    expanded_script_text: Some(
                        "scene_type: daily_dialogue\nsynopsis: valid synopsis".to_string(),
                    ),
                    selected_total_duration_seconds: duration,
                },
            );
            assert_eq!(response.export_status.status, BridgeCallStatus::Blocked);
            assert!(response.result_id.is_empty());
            assert!(
                response
                    .export_status
                    .blockers
                    .iter()
                    .any(|item| item.code == "duration_not_supported")
            );
        }
    }

    #[test]
    fn desktop_scene_type_aliases_stay_valid_without_opening_new_schema_paths() {
        let state = test_state();

        for scene_type in [
            "war_formation",
            "weapon_highlight",
            "council_strategy",
            "slg_sandbox_view",
            "slg_city_growth",
            "battle_report_ui",
            "multi_army_siege",
        ] {
            let script = expand_script(
                &state,
                ExpandScriptRequest {
                    scene_type: scene_type.to_string(),
                    synopsis_text: format!("synopsis for {scene_type}"),
                },
            );
            assert_ne!(script.status, BridgeCallStatus::Blocked);
            assert!(
                script
                    .expanded_script_text
                    .contains("scene_type: daily_dialogue")
            );

            let storyboard = generate_storyboard(
                &state,
                GenerateStoryboardRequest {
                    task_name: format!("task-{scene_type}"),
                    script_id: Some(script.script_id.clone()),
                    expanded_script_text: None,
                    selected_total_duration_seconds: 10,
                },
            );
            assert_ne!(storyboard.export_status.status, BridgeCallStatus::Blocked);
            assert_eq!(
                storyboard
                    .rows
                    .iter()
                    .map(|row| row.duration_seconds)
                    .sum::<u16>(),
                10
            );
        }
    }

    #[test]
    fn generate_storyboard_keeps_duration_sum_for_supported_values() {
        let state = test_state();

        for duration in [5u16, 10u16, 15u16, 30u16, 45u16, 60u16] {
            let response = generate_storyboard(
                &state,
                GenerateStoryboardRequest {
                    task_name: format!("duration-{duration}"),
                    script_id: None,
                    expanded_script_text: Some(format!(
                        "scene_type: daily_dialogue\nsynopsis: duration test {duration}"
                    )),
                    selected_total_duration_seconds: duration,
                },
            );

            assert_ne!(response.result_id, "");
            assert_eq!(
                response
                    .rows
                    .iter()
                    .map(|row| row.duration_seconds)
                    .sum::<u16>(),
                duration
            );
            assert_eq!(response.duration_plan.allocated_seconds, duration);
        }
    }

    #[test]
    fn save_storyboard_rows_marks_snapshot_dirty_and_export_uses_latest_rows() {
        let state = test_state();
        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "editable-storyboard".to_string(),
                script_id: None,
                expanded_script_text: Some(
                    "scene_type: daily_dialogue\nsynopsis: editable export".to_string(),
                ),
                selected_total_duration_seconds: 10,
            },
        );

        let mut edited_rows = storyboard.rows.clone();
        edited_rows[0].visual_description = "Edited bridge visual description".to_string();
        edited_rows[0].prompt_text = "Edited Seedance2.0 prompt text".to_string();
        edited_rows[0].duration_seconds = 4;
        edited_rows[1].duration_seconds = 3;
        edited_rows[2].duration_seconds = 3;

        let saved = save_storyboard_rows(
            &state,
            UpdateStoryboardRowsRequest {
                result_id: storyboard.result_id.clone(),
                task_id: storyboard.task_id.clone(),
                rows: edited_rows,
                operation_id: "op-edit-save-001".to_string(),
                base_revision: storyboard.revision,
                dirty_source_note: Some("desktop_table_edit".to_string()),
            },
        );

        assert!(saved.dirty);
        assert_eq!(saved.revision, storyboard.revision + 1);
        assert_eq!(saved.duration_plan.allocated_seconds, 10);
        assert_eq!(
            saved.rows[0].visual_description,
            "Edited bridge visual description"
        );

        let export = export_bundle(
            &state,
            ExportBundleRequest {
                result_id: Some(storyboard.result_id),
                task_id: saved.task_id.clone(),
                export_format: "xlsx".to_string(),
            },
        );

        let json_artifact = export
            .artifacts
            .iter()
            .find(|artifact| artifact.artifact_kind == "storyboard_json")
            .expect("json artifact should exist");
        assert!(json_artifact.edited_rows_applied);
        assert_eq!(json_artifact.selected_total_duration_seconds, Some(10));
        assert_eq!(json_artifact.full_kb_rows_included, 0);
        assert_eq!(json_artifact.selected_sample_ids.len(), 3);
        assert!(
            json_artifact
                .prompt_text_compilation_statuses
                .iter()
                .any(|status| status == "ReadyStub")
        );
        assert!(json_artifact.kb_context_summary.is_some());
        assert!(json_artifact.retrieval_trace.is_some());

        let json_text = fs::read_to_string(json_artifact.artifact_path.as_ref().unwrap())
            .expect("json export should be readable");
        assert!(json_text.contains("Edited bridge visual description"));
        assert!(json_text.contains("Edited Seedance2.0 prompt text"));
    }

    #[test]
    fn save_storyboard_rows_blocks_duration_mismatch() {
        let state = test_state();
        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "duration-mismatch".to_string(),
                script_id: None,
                expanded_script_text: Some(
                    "scene_type: daily_dialogue\nsynopsis: duration mismatch".to_string(),
                ),
                selected_total_duration_seconds: 10,
            },
        );

        let mut edited_rows = storyboard.rows.clone();
        edited_rows[0].duration_seconds = 9;

        let saved = save_storyboard_rows(
            &state,
            UpdateStoryboardRowsRequest {
                result_id: storyboard.result_id,
                task_id: storyboard.task_id,
                rows: edited_rows,
                operation_id: "op-edit-save-002".to_string(),
                base_revision: storyboard.revision,
                dirty_source_note: None,
            },
        );

        assert_eq!(saved.export_status.status, BridgeCallStatus::Blocked);
        assert!(
            saved
                .export_status
                .blockers
                .iter()
                .any(|item| item.code == "duration_conservation_failed")
        );
        assert!(saved.result_id.is_empty());
    }

    #[test]
    fn build_storyboard_preview_plan_keeps_runtime_directors_with_scene_taxonomy_metadata() {
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
                scene_director_id: Some("scene-director-preview".to_string()),
                action_director_id: Some("action-director-preview".to_string()),
                scene_type: Some("daily_dialogue".to_string()),
                layout_prompt: "cool palette, medium shot".to_string(),
                render_prompt: "restrained realism".to_string(),
            },
        )
        .expect("runtime directors with taxonomy metadata should build preview plan");

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
            "scene-director-preview"
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_action_director_id,
            "action-director-preview"
        );
        assert_eq!(
            plan.committee_runtime.prompt_layers.layout_prompt,
            "cool palette, medium shot"
        );
        assert_eq!(
            plan.committee_runtime.prompt_layers.render_prompt,
            "restrained realism"
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
    fn build_validation_export_panel_snapshot_surfaces_kb_repairs() {
        let state = test_state();
        let mut fixture = load_week3_shared_fixture(WEEK3_SHARED_FIXTURE_PATH)
            .expect("shared fixture should load");
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
        let mut fixture = load_week3_shared_fixture(WEEK3_SHARED_FIXTURE_PATH)
            .expect("shared fixture should load");
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
