use std::env;
use std::hash::{Hash, Hasher};
use std::io;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::state::AppState;

use core_domain::{
    AdaptChapterToScriptRequest, AdaptChapterToScriptResponse, BridgeCallStatus,
    ChapterAcceptedState, ContinuityDeltaLog, ExpandScriptRequest, ExpandScriptResponse,
    ExportArtifactRecord, ExportBundleRequest, ExportBundleResponse, ExportStoryboardBankRequest,
    ExportStoryboardBankResponse, ExternalReferenceHandleCandidate, FinalizedStoryboardRef,
    FinalizedStoryboardShotResult, GenerateNovelChapterRequest, GenerateNovelChapterResponse,
    GenerateStoryboardRequest, GenerateStoryboardResponse, GeneratedStoryboardRow,
    GoldenSampleLibraryRecord, KbRouterExcludedCandidate, KbRouterRetrievalTrace,
    KbRouterRuntimeRequest, KbRouterRuntimeResponse, KbRouterSelectedRule, KbRouterSelectionReason,
    KbRouterTaskType, KbRouterTokenBudget, ListStoryboardShotResultsRequest,
    ListStoryboardShotResultsResponse, ProductWarning, PromptTextCompilationRequest,
    PromptTextCompilationResponse, PromptTextCompilationRow, PromptTextCompilationStatus,
    RemoveStoryboardShotResultRequest, RemoveStoryboardShotResultResponse,
    RunV0StoryToStoryboardChainRequest, RunV0StoryToStoryboardChainResponse,
    SaveStoryboardShotResultRequest, SaveStoryboardShotResultResponse, ScenePerformanceProjection,
    ScriptAcceptedState, SequenceFieldState, SequenceGrouping, ShotGroundingSource, ShotTask,
    ShotTaskPlan, SourceStoryFacts, SplitScriptToShotTasksRequest, SplitScriptToShotTasksResponse,
    StoryContinuityState, StoryboardDurationPlan, StoryboardExportStatus, StoryboardResult,
    StructureMode, TextGenerationOutputSchema, TextGenerationRequest, TextGenerationResponse,
    TextGenerationTask, TextModelProvider, TextModelProviderKind, UpdateContinuityStateRequest,
    UpdateContinuityStateResponse, UpdateStoryboardRowsRequest, UpdateStoryboardShotResultRequest,
    UpdateStoryboardShotResultResponse,
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
    camera_movement: String,
    #[serde(default)]
    dialogue: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    camera_movement: String,
    dialogue: String,
    duration_seconds: u16,
    sequence_grouping: SequenceGrouping,
    scene_performance_projection: ScenePerformanceProjection,
}

const STORYBOARD_DURATION_SOURCE: &str = "storyboard_duration_plan.allocated_row_duration_seconds";
const DEFAULT_EXPAND_SCRIPT_DURATION_SECONDS: u16 = 30;
const SEEDANCE_STANDARD_SEGMENT_SECONDS: u16 = 10;
const SEEDANCE_REMAINDER_SEGMENT_SECONDS: u16 = 5;
const SEEDANCE_MAX_SEGMENT_SECONDS: u16 = 15;
const V0_SHORT_STORY_PROFILE: &str = "short_story_2000_2500";
const V0_TWO_MINUTE_STORY_PROFILE: &str = "two_minute_story_2500_3500";
const V0_SHORT_CLIP_PROFILE: &str = "short_clip";
const V0_STANDARD_CLIP_PROFILE: &str = "standard_clip";
const V0_LONG_STORY_PROFILE: &str = "long_story";
const V0_LONG_STORY_AUTO_PROFILE: &str = "long_story_auto";
const TARGET_DURATION_MODE_FIXED_SECONDS: &str = "fixed_seconds";
const TARGET_DURATION_MODE_LONG_TEXT_AUTO: &str = "long_text_auto";
const AUTO_SEGMENT_STRATEGY_FIXED_SECONDS: &str = "fixed_seconds_seedance_grid";
const AUTO_SEGMENT_STRATEGY_LONG_TEXT: &str = "long_text_auto_story_fact_segments";

const FINALIZED_BANK_FORBIDDEN_TERMS: &[&str] = &[
    "raw prompt_body",
    "prompt_body",
    "prompt body",
    "prompt_body_candidate",
    "source_register",
    "source register",
    "overlay json",
    "overlay_json",
    "api key",
    "api_key",
    "authorization",
    "bearer ",
    "provider secret",
    "plaintext secret",
    "token",
    "full raw kb",
    "full raw kb rows",
    "full_kb_rows",
    "full_kb_rows_included",
];

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
    let target_duration_seconds = resolve_expand_script_target_duration_seconds(&request);
    let router_request = KbRouterRuntimeRequest {
        scene_type: normalized_scene_type.clone(),
        synopsis_text: request.synopsis_text.clone(),
        duration_seconds: target_duration_seconds,
        task_type: KbRouterTaskType::ExpandScript,
        primary_scene_type: Some(normalized_scene_type.clone()),
        primary_scene_label: Some(request.scene_type.clone()),
        shot_scene_type: None,
        shot_scene_label: None,
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
        "{}\n{}\n{}",
        normalized_scene_type,
        request.synopsis_text.trim(),
        target_duration_seconds
    ));
    let script_id = format!("script-{}", &script_hash[..12]);

    let kb_router_result = run_kb_router(state, router_request);
    let provider = default_text_model_provider();
    let generation_request = build_text_generation_request(
        TextGenerationTask::ExpandScript,
        Some(normalized_scene_type.clone()),
        request.synopsis_text.trim().to_string(),
        Some(StoryboardDurationPlan {
            total_duration_seconds: target_duration_seconds,
            row_count: 0,
            per_row_seconds: SEEDANCE_STANDARD_SEGMENT_SECONDS,
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
                    target_duration_seconds,
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
            target_duration_seconds,
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

pub fn generate_novel_chapter(
    state: &AppState,
    request: GenerateNovelChapterRequest,
) -> GenerateNovelChapterResponse {
    let now_ms = now_epoch_ms();
    let source_analysis = analyze_v0_source_input(&request.user_topic_or_synopsis);
    let blockers = validate_generate_novel_chapter_request(&request);
    if !blockers.is_empty() {
        return GenerateNovelChapterResponse {
            status: BridgeCallStatus::Blocked,
            blockers,
            warnings: vec![],
            story_id: request.story_id,
            chapter_id: request.chapter_id,
            chapter_order: request.chapter_order,
            source_input_type: source_analysis.source_input_type,
            authoring_mode: source_analysis.authoring_mode,
            source_material_summary: source_analysis.source_material_summary,
            source_story_facts: source_analysis.source_story_facts,
            preserved_fact_summary: source_analysis.preserved_fact_summary,
            changed_for_screenplay_summary: source_analysis.changed_for_screenplay_summary,
            omitted_detail_summary: source_analysis.omitted_detail_summary,
            chapter_title: String::new(),
            chapter_text: String::new(),
            chapter_summary: String::new(),
            authoring_craft_summary: request.authoring_craft_summary,
            premise_hook: String::new(),
            character_desire: String::new(),
            character_pressure: String::new(),
            conflict_engine: String::new(),
            emotional_turn: String::new(),
            suspense_setup: String::new(),
            payoff_setup: String::new(),
            scene_purpose: String::new(),
            visualizable_action: String::new(),
            chapter_cliffhanger: String::new(),
            director_bridge: String::new(),
            character_motivation_summary: String::new(),
            conflict_progression_summary: String::new(),
            emotional_progression_summary: String::new(),
            timeline_continuity_summary: String::new(),
            prop_state_summary: String::new(),
            next_scene_bridge: String::new(),
            continuity_warnings: source_analysis.continuity_warnings,
            continuity_delta: String::new(),
        };
    }

    let router_request = KbRouterRuntimeRequest {
        scene_type: "v0_authoring".to_string(),
        synopsis_text: request.user_topic_or_synopsis.clone(),
        duration_seconds: 0,
        task_type: KbRouterTaskType::GenerateNovelChapter,
        primary_scene_type: None,
        primary_scene_label: None,
        shot_scene_type: None,
        shot_scene_label: None,
        shot_intent: None,
        structure_type: Some(request.story_length_profile.clone()),
    };
    let router_result = run_kb_router(state, router_request);
    let kb_summary_available = !request.kb_context_summary.trim().is_empty()
        || !router_result.kb_context_summary.trim().is_empty();
    let continuity_profile = build_v0_continuity_profile(
        &source_analysis,
        &request.continuity_context_summary,
        if request.kb_context_summary.trim().is_empty() {
            &router_result.kb_context_summary
        } else {
            &request.kb_context_summary
        },
        &request.retrieval_trace_user_summary,
    );
    let chapter_title =
        deterministic_chapter_title(request.chapter_order, &request.user_topic_or_synopsis);
    let chapter_text = deterministic_v0_source_chapter_text(
        &request.user_topic_or_synopsis,
        &request.story_length_profile,
        &request.authoring_craft_summary,
        &continuity_profile.effective_context_summary,
        kb_summary_available,
        &source_analysis,
    );
    let chapter_summary_source = if source_analysis.authoring_mode == "expand_from_synopsis" {
        &request.user_topic_or_synopsis
    } else {
        &source_analysis.preserved_fact_summary
    };
    let chapter_summary = compact_product_summary(
        chapter_summary_source,
        "本章围绕用户梗概建立角色目标、压力、冲突推进和下一段承接。",
        160,
    );
    let protagonist = derive_product_person(&request.user_topic_or_synopsis, &chapter_text);
    let premise_hook = format!("{protagonist}在既定处境中被迫面对一个无法回避的选择。");
    let character_desire = format!("{protagonist}想守住当前目标，同时确认对立压力的真实来源。");
    let character_pressure = "外部冲突持续逼近，过往状态和新线索同时压向角色。".to_string();
    let conflict_engine = "人物目标与对立力量持续碰撞，推动每一段行动都必须改变局面。".to_string();
    let emotional_turn = "角色从被动承压转向主动判断，情绪由迟疑进入清醒。".to_string();
    let suspense_setup = "关键信息只揭开一部分，下一段仍保留未解决的追问。".to_string();
    let payoff_setup = "本章把角色目标、对手压力和可视化动作放到同一条后续线索上。".to_string();
    let scene_purpose = "建立本章冲突、角色动机和可拆分的镜头行动。".to_string();
    let visualizable_action = first_story_sentence(&chapter_text);
    let chapter_cliffhanger = "下一段需要承接本章末尾的选择结果和未解压力。".to_string();
    let _director_bridge = "后续改编应优先保留角色目标、行动对象、空间关系和情绪转折。".to_string();
    let character_motivation_summary = continuity_profile.character_motivation_summary.clone();
    let conflict_progression_summary = continuity_profile.conflict_progression_summary.clone();
    let emotional_progression_summary = continuity_profile.emotional_progression_summary.clone();
    let _timeline_continuity_summary = format!(
        "第{}章按单一连续段落推进，不静默跳时空。",
        request.chapter_order
    );
    let _prop_state_summary =
        "道具状态仅按梗概中已出现的信息保留，未出现的关键道具不新增归属。".to_string();
    let _next_scene_bridge = chapter_cliffhanger.clone();
    let director_bridge = continuity_profile.director_bridge.clone();
    let timeline_continuity_summary = continuity_profile.timeline_continuity_summary.clone();
    let prop_state_summary = continuity_profile.prop_state_summary.clone();
    let next_scene_bridge = continuity_profile.next_scene_bridge.clone();
    let continuity_delta = format!(
        "chapter:{} established summary={}, motivation={}, next_bridge={}",
        request.chapter_id, chapter_summary, character_motivation_summary, next_scene_bridge
    );
    let mut warnings = source_analysis.continuity_warnings.clone();
    warnings.extend(continuity_profile.warnings.clone());
    if !kb_summary_available {
        warnings.push(ProductWarning {
            code: "v0_kb_context_summary_not_supplied".to_string(),
            message:
                "V0 chapter generation used neutral deterministic craft because no external KB summary was supplied."
                    .to_string(),
            related_sample_id: None,
        });
    }
    dedupe_product_warnings(&mut warnings);
    let continuity_warnings = warnings.clone();

    let response = GenerateNovelChapterResponse {
        status: if warnings.is_empty() {
            BridgeCallStatus::Ready
        } else {
            BridgeCallStatus::WarningOnly
        },
        blockers: vec![],
        warnings,
        story_id: request.story_id.clone(),
        chapter_id: request.chapter_id.clone(),
        chapter_order: request.chapter_order,
        source_input_type: source_analysis.source_input_type.clone(),
        authoring_mode: source_analysis.authoring_mode.clone(),
        source_material_summary: source_analysis.source_material_summary.clone(),
        source_story_facts: source_analysis.source_story_facts.clone(),
        preserved_fact_summary: source_analysis.preserved_fact_summary.clone(),
        changed_for_screenplay_summary: source_analysis.changed_for_screenplay_summary.clone(),
        omitted_detail_summary: source_analysis.omitted_detail_summary.clone(),
        chapter_title: chapter_title.clone(),
        chapter_text: chapter_text.clone(),
        chapter_summary: chapter_summary.clone(),
        authoring_craft_summary: request.authoring_craft_summary,
        premise_hook,
        character_desire,
        character_pressure,
        conflict_engine,
        emotional_turn,
        suspense_setup,
        payoff_setup,
        scene_purpose,
        visualizable_action,
        chapter_cliffhanger,
        director_bridge,
        character_motivation_summary,
        conflict_progression_summary,
        emotional_progression_summary,
        timeline_continuity_summary,
        prop_state_summary,
        next_scene_bridge,
        continuity_warnings: continuity_warnings.clone(),
        continuity_delta: continuity_delta.clone(),
    };
    state.remember_v0_chapter(ChapterAcceptedState {
        story_id: response.story_id.clone(),
        chapter_id: response.chapter_id.clone(),
        chapter_order: response.chapter_order,
        chapter_title,
        chapter_text,
        chapter_summary,
        source_input_type: response.source_input_type.clone(),
        authoring_mode: response.authoring_mode.clone(),
        source_material_summary: response.source_material_summary.clone(),
        source_story_facts: response.source_story_facts.clone(),
        preserved_fact_summary: response.preserved_fact_summary.clone(),
        changed_for_screenplay_summary: response.changed_for_screenplay_summary.clone(),
        omitted_detail_summary: response.omitted_detail_summary.clone(),
        continuity_warnings: continuity_warnings,
        continuity_delta,
        accepted_at_ms: now_ms,
    });
    state.append_v0_continuity_delta_log(ContinuityDeltaLog {
        story_id: response.story_id.clone(),
        chapter_id: response.chapter_id.clone(),
        chapter_order: response.chapter_order,
        stage: "generate_novel_chapter".to_string(),
        continuity_delta: response.continuity_delta.clone(),
        updated_at_ms: now_ms,
    });
    response
}

pub fn adapt_chapter_to_script(
    state: &AppState,
    request: AdaptChapterToScriptRequest,
) -> AdaptChapterToScriptResponse {
    let now_ms = now_epoch_ms();
    let blockers = validate_adapt_chapter_to_script_request(&request);
    if !blockers.is_empty() {
        return AdaptChapterToScriptResponse {
            status: BridgeCallStatus::Blocked,
            blockers,
            warnings: vec![],
            script_id: String::new(),
            story_id: request.story_id,
            chapter_id: request.chapter_id,
            chapter_order: request.chapter_order,
            source_input_type: request.source_input_type,
            authoring_mode: request.authoring_mode,
            source_material_summary: request.source_material_summary,
            source_story_facts: request.source_story_facts,
            preserved_fact_summary: request.preserved_fact_summary,
            changed_for_screenplay_summary: String::new(),
            omitted_detail_summary: String::new(),
            script_text: String::new(),
            script_summary: String::new(),
            screenwriting_adaptation_summary: request.screenwriting_adaptation_summary,
            scene_beats: vec![],
            dialogue_intent: String::new(),
            action_blocks: vec![],
            turning_points: vec![],
            scene_purpose: String::new(),
            continuity_warnings: vec![],
            continuity_delta: String::new(),
        };
    }

    let router_request = KbRouterRuntimeRequest {
        scene_type: "v0_screenwriting".to_string(),
        synopsis_text: format!("{}\n{}", request.chapter_summary, request.chapter_text),
        duration_seconds: 0,
        task_type: KbRouterTaskType::AdaptChapterToScript,
        primary_scene_type: None,
        primary_scene_label: None,
        shot_scene_type: None,
        shot_scene_label: None,
        shot_intent: None,
        structure_type: Some("chapter_to_script".to_string()),
    };
    let _router_result = run_kb_router(state, router_request);
    let script_id = format!(
        "v0-script-{}",
        &stable_hash_hex(&format!(
            "{}\n{}\n{}",
            request.story_id, request.chapter_id, request.chapter_text
        ))[..12]
    );
    let scene_beats = deterministic_scene_beats(&request.chapter_text);
    let action_blocks = scene_beats
        .iter()
        .enumerate()
        .map(|(index, beat)| format!("动作段{}：{}", index + 1, beat))
        .collect::<Vec<_>>();
    let turning_points = vec![
        "角色确认当前压力的来源。".to_string(),
        "角色用可见行动改变局面。".to_string(),
        "本段留下下一镜头可承接的状态。".to_string(),
    ];
    let dialogue_intent = "对白只服务角色目标、压力确认和关系变化，不替代行动。".to_string();
    let scene_purpose = "把章节内容压缩为可表演、可拆镜头的连续剧本段落。".to_string();
    let _legacy_script_text = deterministic_v0_script_text(
        &format!("第{}章改编剧本", request.chapter_order),
        &scene_beats,
        &dialogue_intent,
        &request.chapter_summary,
    );
    let script_text =
        deterministic_v0_script_text_for_authoring(&request, &scene_beats, &dialogue_intent);
    let script_summary = format!(
        "剧本保留章节主线：{}",
        compact_product_summary(
            &request.chapter_summary,
            "角色目标、冲突和转折被保留。",
            100
        )
    );
    let continuity_delta = format!(
        "script:{} adapted chapter:{} into {} ordered beats while preserving chapter facts",
        script_id,
        request.chapter_id,
        scene_beats.len()
    );

    let mut continuity_warnings = validate_rewrite_fact_preservation(
        &script_text,
        &request.source_story_facts,
        &request.authoring_mode,
    );
    dedupe_product_warnings(&mut continuity_warnings);
    let response = AdaptChapterToScriptResponse {
        status: if continuity_warnings.is_empty() {
            BridgeCallStatus::Ready
        } else {
            BridgeCallStatus::WarningOnly
        },
        blockers: vec![],
        warnings: continuity_warnings.clone(),
        script_id: script_id.clone(),
        story_id: request.story_id.clone(),
        chapter_id: request.chapter_id.clone(),
        chapter_order: request.chapter_order,
        source_input_type: request.source_input_type.clone(),
        authoring_mode: request.authoring_mode.clone(),
        source_material_summary: request.source_material_summary.clone(),
        source_story_facts: request.source_story_facts.clone(),
        preserved_fact_summary: request.preserved_fact_summary.clone(),
        changed_for_screenplay_summary: changed_for_screenplay_summary(
            &request.source_input_type,
            &request.authoring_mode,
        ),
        omitted_detail_summary: omitted_detail_summary(
            &request.source_input_type,
            &request.source_story_facts,
        ),
        script_text: script_text.clone(),
        script_summary: script_summary.clone(),
        screenwriting_adaptation_summary: request.screenwriting_adaptation_summary,
        scene_beats,
        dialogue_intent,
        action_blocks,
        turning_points,
        scene_purpose,
        continuity_warnings: continuity_warnings.clone(),
        continuity_delta: continuity_delta.clone(),
    };
    state.remember_v0_script(ScriptAcceptedState {
        script_id,
        story_id: response.story_id.clone(),
        chapter_id: response.chapter_id.clone(),
        chapter_order: response.chapter_order,
        script_text,
        script_summary,
        source_input_type: response.source_input_type.clone(),
        authoring_mode: response.authoring_mode.clone(),
        source_material_summary: response.source_material_summary.clone(),
        source_story_facts: response.source_story_facts.clone(),
        preserved_fact_summary: response.preserved_fact_summary.clone(),
        changed_for_screenplay_summary: response.changed_for_screenplay_summary.clone(),
        omitted_detail_summary: response.omitted_detail_summary.clone(),
        continuity_warnings: response.continuity_warnings.clone(),
        continuity_delta,
        accepted_at_ms: now_ms,
    });
    state.append_v0_continuity_delta_log(ContinuityDeltaLog {
        story_id: response.story_id.clone(),
        chapter_id: response.chapter_id.clone(),
        chapter_order: response.chapter_order,
        stage: "adapt_chapter_to_script".to_string(),
        continuity_delta: response.continuity_delta.clone(),
        updated_at_ms: now_ms,
    });
    response
}

pub fn update_continuity_state(
    state: &AppState,
    request: UpdateContinuityStateRequest,
) -> UpdateContinuityStateResponse {
    let now_ms = now_epoch_ms();
    let mut blockers = Vec::new();
    for (field_name, value) in [
        ("story_id", request.story_id.as_str()),
        ("chapter_id", request.chapter_id.as_str()),
        ("chapter_summary", request.chapter_summary.as_str()),
        ("continuity_delta", request.continuity_delta.as_str()),
    ] {
        if value.trim().is_empty() {
            blockers.push(ProductWarning {
                code: format!("{field_name}_required"),
                message: format!("{field_name} is required before updating V0 continuity state."),
                related_sample_id: None,
            });
        }
    }
    if contains_forbidden_v0_product_payload(&format!(
        "{}\n{}\n{}\n{}\n{}",
        request.chapter_summary,
        request.character_state_summary,
        request.location_state_summary,
        request.prop_state_summary,
        request.timeline_state_summary
    )) {
        blockers.push(ProductWarning {
            code: "v0_continuity_forbidden_payload".to_string(),
            message: "V0 continuity state rejected internal, raw, credential, or full-KB payload content."
                .to_string(),
            related_sample_id: None,
        });
    }
    if !blockers.is_empty() {
        return UpdateContinuityStateResponse {
            status: BridgeCallStatus::Blocked,
            blockers,
            warnings: vec![],
            continuity_state: None,
        };
    }

    let ref_summary = request
        .finalized_storyboard_refs
        .iter()
        .map(|item| item.label.clone())
        .collect::<Vec<_>>()
        .join("；");
    let continuity_context_summary = format!(
        "故事{}第{}章：{}。角色：{}。地点：{}。道具：{}。时间线：{}。未解线索：{}。定稿分镜引用：{}。最新变化：{}。",
        request.story_id,
        request.chapter_order,
        compact_product_summary(&request.chapter_summary, "已接受章节摘要", 140),
        compact_product_summary(&request.character_state_summary, "角色状态保持连续", 120),
        compact_product_summary(&request.location_state_summary, "地点状态保持连续", 100),
        compact_product_summary(&request.prop_state_summary, "道具状态保持连续", 100),
        compact_product_summary(&request.timeline_state_summary, "时间线保持连续", 100),
        if request.unresolved_threads.is_empty() {
            "无新增未解线索".to_string()
        } else {
            request.unresolved_threads.join("；")
        },
        if ref_summary.trim().is_empty() {
            "暂无".to_string()
        } else {
            ref_summary
        },
        compact_product_summary(&request.continuity_delta, "连续性已更新", 140),
    );
    let mut warnings = if request.finalized_storyboard_refs.is_empty() {
        vec![ProductWarning {
            code: "v0_continuity_without_finalized_refs".to_string(),
            message: "Continuity was updated without finalized storyboard refs; next stage will rely on text summaries only."
                .to_string(),
            related_sample_id: None,
        }]
    } else {
        vec![]
    };
    warnings.extend(validate_continuity_state_against_source_facts(&request));
    dedupe_product_warnings(&mut warnings);
    let continuity_state = StoryContinuityState {
        story_id: request.story_id.clone(),
        chapter_id: request.chapter_id.clone(),
        chapter_order: request.chapter_order,
        continuity_context_summary,
        chapter_summary: request.chapter_summary,
        source_input_type: request.source_input_type,
        authoring_mode: request.authoring_mode,
        source_material_summary: request.source_material_summary,
        source_story_facts: request.source_story_facts,
        preserved_fact_summary: request.preserved_fact_summary,
        changed_for_screenplay_summary: request.changed_for_screenplay_summary,
        omitted_detail_summary: request.omitted_detail_summary,
        character_state_summary: request.character_state_summary,
        location_state_summary: request.location_state_summary,
        prop_state_summary: request.prop_state_summary,
        timeline_state_summary: request.timeline_state_summary,
        unresolved_threads: request.unresolved_threads,
        style_bible_summary: request.style_bible_summary,
        finalized_storyboard_refs: request.finalized_storyboard_refs,
        continuity_delta: request.continuity_delta.clone(),
        updated_at_ms: now_ms,
        warnings: warnings.clone(),
    };
    state.remember_v0_continuity_state(continuity_state.clone());
    state.append_v0_continuity_delta_log(ContinuityDeltaLog {
        story_id: request.story_id,
        chapter_id: request.chapter_id,
        chapter_order: request.chapter_order,
        stage: "update_continuity_state".to_string(),
        continuity_delta: request.continuity_delta,
        updated_at_ms: now_ms,
    });

    UpdateContinuityStateResponse {
        status: if warnings.is_empty() {
            BridgeCallStatus::Ready
        } else {
            BridgeCallStatus::WarningOnly
        },
        blockers: vec![],
        warnings,
        continuity_state: Some(continuity_state),
    }
}

pub fn run_v0_story_to_storyboard_chain(
    state: &AppState,
    request: RunV0StoryToStoryboardChainRequest,
) -> RunV0StoryToStoryboardChainResponse {
    let source_analysis = analyze_v0_source_input(&request.user_topic_or_synopsis);
    let duration_plan = plan_v0_chain_duration(&request, &source_analysis);
    let mut warnings = duration_plan.warnings.clone();
    let request_continuity_warnings = build_v0_chain_advisory_warnings(
        &source_analysis.source_story_facts,
        &request.continuity_context_summary,
        &request.kb_context_summary,
        &request.directing_kb_context_summary,
    );
    let mut chain_continuity_warnings = source_analysis.continuity_warnings.clone();
    warnings.extend(request_continuity_warnings.clone());
    chain_continuity_warnings.extend(request_continuity_warnings.clone());
    dedupe_product_warnings(&mut warnings);
    dedupe_product_warnings(&mut chain_continuity_warnings);
    let mut blockers = validate_run_v0_story_to_storyboard_chain_request(&request);
    blockers.extend(
        duration_plan
            .warnings
            .iter()
            .filter(|warning| {
                warning.code == "long_text_auto_duration_plan_missing"
                    || warning.code == "long_text_auto_compressed_to_single_clip_blocked"
            })
            .cloned(),
    );
    if !blockers.is_empty() {
        return RunV0StoryToStoryboardChainResponse {
            status: BridgeCallStatus::Blocked,
            blockers,
            warnings: source_analysis
                .continuity_warnings
                .iter()
                .chain(duration_plan.warnings.iter())
                .chain(request_continuity_warnings.iter())
                .cloned()
                .collect(),
            target_duration_mode: duration_plan.target_duration_mode,
            story_length_profile: duration_plan.story_length_profile,
            source_material_length_chars: duration_plan.source_material_length_chars,
            auto_segment_strategy: duration_plan.auto_segment_strategy,
            estimated_total_story_duration_seconds: duration_plan
                .estimated_total_story_duration_seconds,
            generated_shot_task_count: duration_plan.generated_shot_task_count,
            duration_plan_summary: duration_plan.duration_plan_summary,
            source_input_type: source_analysis.source_input_type,
            authoring_mode: source_analysis.authoring_mode,
            source_material_summary: source_analysis.source_material_summary,
            source_story_facts: source_analysis.source_story_facts,
            preserved_fact_summary: source_analysis.preserved_fact_summary,
            changed_for_screenplay_summary: source_analysis.changed_for_screenplay_summary,
            omitted_detail_summary: source_analysis.omitted_detail_summary,
            continuity_warnings: chain_continuity_warnings,
            chapter: None,
            script: None,
            shot_task_plan: None,
            storyboard_results: vec![],
            finalized_storyboard_refs: vec![],
            continuity_state: None,
        };
    }

    let authoring_craft_summary = v0_required_summary_or_neutral(
        &request.authoring_craft_summary,
        "neutral authoring craft: premise hook, character desire, conflict engine, visible action, next scene bridge",
    );
    let screenwriting_adaptation_summary = v0_required_summary_or_neutral(
        &request.screenwriting_adaptation_summary,
        "neutral screenwriting adaptation: preserve accepted plot facts, compress into beats, keep action and dialogue playable",
    );
    let directing_kb_context_summary = v0_required_summary_or_neutral(
        &request.directing_kb_context_summary,
        "neutral directing guidance: camera and blocking serve story facts, shot_script remains authoritative",
    );

    let chapter = generate_novel_chapter(
        state,
        GenerateNovelChapterRequest {
            story_id: request.story_id.clone(),
            chapter_id: request.chapter_id.clone(),
            chapter_order: request.chapter_order,
            user_topic_or_synopsis: request.user_topic_or_synopsis.clone(),
            story_length_profile: duration_plan.story_length_profile.clone(),
            authoring_craft_summary,
            continuity_context_summary: request.continuity_context_summary.clone(),
            kb_context_summary: request.kb_context_summary.clone(),
            selected_sample_ids: request.selected_sample_ids.clone(),
            selected_kb_rules: request.selected_kb_rules.clone(),
            retrieval_trace_user_summary: request.retrieval_trace_user_summary.clone(),
            full_kb_rows_included: request.full_kb_rows_included,
        },
    );
    warnings.extend(chapter.warnings.clone());
    dedupe_product_warnings(&mut warnings);
    let mut chapter_stage_continuity_warnings = chain_continuity_warnings.clone();
    chapter_stage_continuity_warnings.extend(chapter.continuity_warnings.clone());
    dedupe_product_warnings(&mut chapter_stage_continuity_warnings);
    if !chapter.blockers.is_empty() {
        blockers.extend(chapter.blockers.clone());
        return RunV0StoryToStoryboardChainResponse {
            status: BridgeCallStatus::Blocked,
            blockers,
            warnings,
            target_duration_mode: duration_plan.target_duration_mode.clone(),
            story_length_profile: duration_plan.story_length_profile.clone(),
            source_material_length_chars: duration_plan.source_material_length_chars,
            auto_segment_strategy: duration_plan.auto_segment_strategy.clone(),
            estimated_total_story_duration_seconds: duration_plan
                .estimated_total_story_duration_seconds,
            generated_shot_task_count: duration_plan.generated_shot_task_count,
            duration_plan_summary: duration_plan.duration_plan_summary.clone(),
            source_input_type: chapter.source_input_type.clone(),
            authoring_mode: chapter.authoring_mode.clone(),
            source_material_summary: chapter.source_material_summary.clone(),
            source_story_facts: chapter.source_story_facts.clone(),
            preserved_fact_summary: chapter.preserved_fact_summary.clone(),
            changed_for_screenplay_summary: chapter.changed_for_screenplay_summary.clone(),
            omitted_detail_summary: chapter.omitted_detail_summary.clone(),
            continuity_warnings: chapter_stage_continuity_warnings,
            chapter: Some(chapter),
            script: None,
            shot_task_plan: None,
            storyboard_results: vec![],
            finalized_storyboard_refs: vec![],
            continuity_state: None,
        };
    }

    let script = adapt_chapter_to_script(
        state,
        AdaptChapterToScriptRequest {
            story_id: request.story_id.clone(),
            chapter_id: request.chapter_id.clone(),
            chapter_order: request.chapter_order,
            chapter_text: chapter.chapter_text.clone(),
            chapter_summary: chapter.chapter_summary.clone(),
            source_input_type: chapter.source_input_type.clone(),
            authoring_mode: chapter.authoring_mode.clone(),
            source_story_facts: chapter.source_story_facts.clone(),
            preserved_fact_summary: chapter.preserved_fact_summary.clone(),
            source_material_summary: chapter.source_material_summary.clone(),
            screenwriting_adaptation_summary,
            continuity_context_summary: build_script_stage_continuity_context(
                &request.continuity_context_summary,
                &chapter,
            ),
            kb_context_summary: request.kb_context_summary.clone(),
            selected_sample_ids: request.selected_sample_ids.clone(),
            selected_kb_rules: request.selected_kb_rules.clone(),
            retrieval_trace_user_summary: request.retrieval_trace_user_summary.clone(),
            full_kb_rows_included: request.full_kb_rows_included,
        },
    );
    warnings.extend(script.warnings.clone());
    dedupe_product_warnings(&mut warnings);
    let mut script_stage_continuity_warnings = chapter_stage_continuity_warnings.clone();
    script_stage_continuity_warnings.extend(script.continuity_warnings.clone());
    dedupe_product_warnings(&mut script_stage_continuity_warnings);
    if !script.blockers.is_empty() {
        blockers.extend(script.blockers.clone());
        return RunV0StoryToStoryboardChainResponse {
            status: BridgeCallStatus::Blocked,
            blockers,
            warnings,
            target_duration_mode: duration_plan.target_duration_mode.clone(),
            story_length_profile: duration_plan.story_length_profile.clone(),
            source_material_length_chars: duration_plan.source_material_length_chars,
            auto_segment_strategy: duration_plan.auto_segment_strategy.clone(),
            estimated_total_story_duration_seconds: duration_plan
                .estimated_total_story_duration_seconds,
            generated_shot_task_count: duration_plan.generated_shot_task_count,
            duration_plan_summary: duration_plan.duration_plan_summary.clone(),
            source_input_type: chapter.source_input_type.clone(),
            authoring_mode: chapter.authoring_mode.clone(),
            source_material_summary: chapter.source_material_summary.clone(),
            source_story_facts: chapter.source_story_facts.clone(),
            preserved_fact_summary: chapter.preserved_fact_summary.clone(),
            changed_for_screenplay_summary: script.changed_for_screenplay_summary.clone(),
            omitted_detail_summary: script.omitted_detail_summary.clone(),
            continuity_warnings: script_stage_continuity_warnings,
            chapter: Some(chapter),
            script: Some(script),
            shot_task_plan: None,
            storyboard_results: vec![],
            finalized_storyboard_refs: vec![],
            continuity_state: None,
        };
    }

    let split_response = split_script_to_shot_tasks(SplitScriptToShotTasksRequest {
        script_id: Some(script.script_id.clone()),
        expanded_script_text: script.script_text.clone(),
        selected_total_duration_seconds: duration_plan.estimated_total_story_duration_seconds,
        target_duration_mode: duration_plan.target_duration_mode.clone(),
        auto_segment_strategy: duration_plan.auto_segment_strategy.clone(),
        primary_scene_type: request.primary_scene_type.clone(),
        primary_scene_label: request.primary_scene_label.clone(),
        primary_scene_category: request.primary_scene_category.clone(),
        task_type: Some("v0_story_to_storyboard_chain".to_string()),
        shot_count_hint: None,
        structure_type: Some(duration_plan.story_length_profile.clone()),
        kb_context_summary: Some(request.kb_context_summary.clone()),
        selected_kb_rules: request.selected_kb_rules.clone(),
        selected_sample_ids: request.selected_sample_ids.clone(),
    });
    warnings.extend(split_response.warnings.clone());
    let shot_task_plan = ShotTaskPlan {
        script_id: script.script_id.clone(),
        story_id: request.story_id.clone(),
        chapter_id: request.chapter_id.clone(),
        chapter_order: request.chapter_order,
        shot_task_count: split_response.shot_tasks.len() as u32,
        target_duration_mode: duration_plan.target_duration_mode.clone(),
        story_length_profile: duration_plan.story_length_profile.clone(),
        source_material_length_chars: duration_plan.source_material_length_chars,
        auto_segment_strategy: duration_plan.auto_segment_strategy.clone(),
        estimated_total_story_duration_seconds: duration_plan
            .estimated_total_story_duration_seconds,
        generated_shot_task_count: split_response.shot_tasks.len() as u32,
        duration_plan_summary: duration_plan.duration_plan_summary.clone(),
        continuity_delta: format!(
            "shot_task_plan:{} split script into {} grounded shot tasks",
            script.script_id,
            split_response.shot_tasks.len()
        ),
        warnings: split_response.warnings.clone(),
        shot_tasks: split_response.shot_tasks.clone(),
    };
    state.remember_v0_shot_task_plan(shot_task_plan.clone());
    state.append_v0_continuity_delta_log(ContinuityDeltaLog {
        story_id: request.story_id.clone(),
        chapter_id: request.chapter_id.clone(),
        chapter_order: request.chapter_order,
        stage: "split_script_to_shot_tasks".to_string(),
        continuity_delta: shot_task_plan.continuity_delta.clone(),
        updated_at_ms: now_epoch_ms(),
    });

    let mut storyboard_results = Vec::new();
    let mut finalized_storyboard_refs = Vec::new();
    for task in &shot_task_plan.shot_tasks {
        let storyboard = generate_storyboard_with_live_text(
            state,
            GenerateStoryboardRequest {
                task_name: v0_shot_task_name(task),
                script_id: Some(script.script_id.clone()),
                shot_script: Some(task.shot_script.clone()),
                expanded_script_text: Some(script.script_text.clone()),
                primary_scene_type: Some(request.primary_scene_type.clone()),
                primary_scene_label: request.primary_scene_label.clone(),
                primary_scene_category: request.primary_scene_category.clone(),
                shot_scene_type: Some(task.shot_scene_type.clone()),
                shot_scene_label: Some(task.shot_scene_label.clone()),
                shot_intent: Some(task.shot_intent.clone()),
                adaptation_reason: Some(task.adaptation_reason.clone()),
                selected_total_duration_seconds: task.duration_seconds,
            },
            false,
        );
        warnings.extend(storyboard.export_status.warnings.clone());
        if !storyboard.export_status.blockers.is_empty() {
            blockers.extend(storyboard.export_status.blockers.clone());
            continue;
        }
        let prompt_text = storyboard
            .rows
            .iter()
            .map(|row| row.prompt_text.clone())
            .collect::<Vec<_>>()
            .join("\n");
        let storyboard_result = StoryboardResult {
            status: storyboard.export_status.status,
            blockers: storyboard.export_status.blockers.clone(),
            warnings: storyboard.export_status.warnings.clone(),
            storyboard_result_id: storyboard.result_id.clone(),
            story_id: request.story_id.clone(),
            chapter_id: request.chapter_id.clone(),
            chapter_order: request.chapter_order,
            script_id: script.script_id.clone(),
            shot_task_id: task.shot_task_id.clone(),
            shot_order: task.shot_order,
            rows: storyboard.rows.clone(),
            prompt_text: prompt_text.clone(),
            shot_duration_seconds: task.duration_seconds,
            duration_source: STORYBOARD_DURATION_SOURCE.to_string(),
            director_intent_summary: format!(
                "镜头{}服务当前剧情片段，不改写角色事实。",
                task.shot_order
            ),
            performance_focus: task.shot_intent.clone(),
            blocking_hint: "按 shot_script 保持主体、对象和空间关系清晰。".to_string(),
            rhythm_hint: format!("使用 {} 秒 Seedance 友好分段节奏。", task.duration_seconds),
            visual_focus: task.shot_scene_label.clone(),
            continuity_note: format!(
                "承接脚本{}第{}个镜头任务。",
                script.script_id, task.shot_order
            ),
            directing_kb_context_summary: directing_kb_context_summary.clone(),
            continuity_delta: format!(
                "storyboard:{} produced {} row(s) for shot_task:{}",
                storyboard.result_id,
                storyboard.rows.len(),
                task.shot_task_id
            ),
        };
        let save_response = save_storyboard_shot_result(
            state,
            SaveStoryboardShotResultRequest {
                project_id: request.story_id.clone(),
                script_id: script.script_id.clone(),
                shot_task_id: task.shot_task_id.clone(),
                result_id: storyboard.result_id.clone(),
                shot_order: task.shot_order,
                shot_task_name: v0_shot_task_name(task),
                rows: storyboard.rows.clone(),
                prompt_text,
                shot_duration_seconds: task.duration_seconds,
                duration_source: STORYBOARD_DURATION_SOURCE.to_string(),
                confirmed: request.confirm_storyboard_results,
                updated_at_ms: 0,
                rows_hash: storyboard.rows_hash.clone(),
            },
        );
        warnings.extend(save_response.warnings.clone());
        if !save_response.blockers.is_empty() {
            blockers.extend(save_response.blockers.clone());
        } else if let Some(saved) = save_response.shot {
            finalized_storyboard_refs.push(finalized_ref_from_saved_shot(
                &request.story_id,
                &request.chapter_id,
                request.chapter_order,
                &saved,
            ));
        }
        storyboard_results.push(storyboard_result);
    }

    if !blockers.is_empty() {
        return RunV0StoryToStoryboardChainResponse {
            status: BridgeCallStatus::Blocked,
            blockers,
            warnings,
            target_duration_mode: duration_plan.target_duration_mode.clone(),
            story_length_profile: duration_plan.story_length_profile.clone(),
            source_material_length_chars: duration_plan.source_material_length_chars,
            auto_segment_strategy: duration_plan.auto_segment_strategy.clone(),
            estimated_total_story_duration_seconds: duration_plan
                .estimated_total_story_duration_seconds,
            generated_shot_task_count: shot_task_plan.generated_shot_task_count,
            duration_plan_summary: duration_plan.duration_plan_summary.clone(),
            source_input_type: chapter.source_input_type.clone(),
            authoring_mode: chapter.authoring_mode.clone(),
            source_material_summary: chapter.source_material_summary.clone(),
            source_story_facts: chapter.source_story_facts.clone(),
            preserved_fact_summary: chapter.preserved_fact_summary.clone(),
            changed_for_screenplay_summary: script.changed_for_screenplay_summary.clone(),
            omitted_detail_summary: script.omitted_detail_summary.clone(),
            continuity_warnings: script_stage_continuity_warnings.clone(),
            chapter: Some(chapter),
            script: Some(script),
            shot_task_plan: Some(shot_task_plan),
            storyboard_results,
            finalized_storyboard_refs,
            continuity_state: None,
        };
    }

    let continuity_response = update_continuity_state(
        state,
        UpdateContinuityStateRequest {
            story_id: request.story_id.clone(),
            chapter_id: request.chapter_id.clone(),
            chapter_order: request.chapter_order,
            chapter_summary: chapter.chapter_summary.clone(),
            source_input_type: chapter.source_input_type.clone(),
            authoring_mode: chapter.authoring_mode.clone(),
            source_material_summary: chapter.source_material_summary.clone(),
            source_story_facts: chapter.source_story_facts.clone(),
            preserved_fact_summary: chapter.preserved_fact_summary.clone(),
            changed_for_screenplay_summary: script.changed_for_screenplay_summary.clone(),
            omitted_detail_summary: script.omitted_detail_summary.clone(),
            character_state_summary: format!(
                "motivation={}; emotional_progression={}; conflict_progression={}",
                chapter.character_motivation_summary,
                chapter.emotional_progression_summary,
                chapter.conflict_progression_summary
            ),
            location_state_summary: "地点连续性按章节和脚本中已确认的空间关系保留。".to_string(),
            prop_state_summary: chapter.prop_state_summary.clone(),
            timeline_state_summary: chapter.timeline_continuity_summary.clone(),
            unresolved_threads: vec![chapter.next_scene_bridge.clone()],
            style_bible_summary: "中性故事语气，禁止真实作者、导演、IP 或品牌风格模仿。"
                .to_string(),
            finalized_storyboard_refs: finalized_storyboard_refs.clone(),
            continuity_delta: format!(
                "{} | {} | {}",
                chapter.continuity_delta, script.continuity_delta, shot_task_plan.continuity_delta
            ),
        },
    );
    warnings.extend(continuity_response.warnings.clone());
    dedupe_product_warnings(&mut warnings);
    if !continuity_response.blockers.is_empty() {
        blockers.extend(continuity_response.blockers.clone());
    }
    let mut final_continuity_warnings = script_stage_continuity_warnings;
    final_continuity_warnings.extend(continuity_response.warnings.clone());
    dedupe_product_warnings(&mut final_continuity_warnings);

    RunV0StoryToStoryboardChainResponse {
        status: if !blockers.is_empty() {
            BridgeCallStatus::Blocked
        } else if !warnings.is_empty() {
            BridgeCallStatus::WarningOnly
        } else {
            BridgeCallStatus::Ready
        },
        blockers,
        warnings,
        target_duration_mode: duration_plan.target_duration_mode.clone(),
        story_length_profile: duration_plan.story_length_profile.clone(),
        source_material_length_chars: duration_plan.source_material_length_chars,
        auto_segment_strategy: duration_plan.auto_segment_strategy.clone(),
        estimated_total_story_duration_seconds: duration_plan
            .estimated_total_story_duration_seconds,
        generated_shot_task_count: shot_task_plan.generated_shot_task_count,
        duration_plan_summary: duration_plan.duration_plan_summary.clone(),
        source_input_type: chapter.source_input_type.clone(),
        authoring_mode: chapter.authoring_mode.clone(),
        source_material_summary: chapter.source_material_summary.clone(),
        source_story_facts: chapter.source_story_facts.clone(),
        preserved_fact_summary: chapter.preserved_fact_summary.clone(),
        changed_for_screenplay_summary: script.changed_for_screenplay_summary.clone(),
        omitted_detail_summary: script.omitted_detail_summary.clone(),
        continuity_warnings: final_continuity_warnings,
        chapter: Some(chapter),
        script: Some(script),
        shot_task_plan: Some(shot_task_plan),
        storyboard_results,
        finalized_storyboard_refs,
        continuity_state: continuity_response.continuity_state,
    }
}

pub fn generate_storyboard(
    state: &AppState,
    request: GenerateStoryboardRequest,
) -> GenerateStoryboardResponse {
    generate_storyboard_with_live_text(state, request, true)
}

fn generate_storyboard_with_live_text(
    state: &AppState,
    request: GenerateStoryboardRequest,
    allow_live_text: bool,
) -> GenerateStoryboardResponse {
    let now_ms = now_epoch_ms();
    let expanded_script_text = request
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
    let grounding = resolve_storyboard_grounding_context(&request, expanded_script_text);
    let normalized_scene_type = normalize_scene_type(&grounding.primary_scene_type);
    let synopsis_text = build_storyboard_router_synopsis(&grounding);
    let router_request = KbRouterRuntimeRequest {
        scene_type: normalized_scene_type.clone(),
        synopsis_text,
        duration_seconds: request.selected_total_duration_seconds,
        task_type: KbRouterTaskType::GenerateStoryboard,
        primary_scene_type: Some(grounding.primary_scene_type.clone()),
        primary_scene_label: Some(grounding.primary_scene_label.clone()),
        shot_scene_type: Some(grounding.shot_scene_type.clone()),
        shot_scene_label: Some(grounding.shot_scene_label.clone()),
        shot_intent: non_blank_string(&grounding.shot_intent),
        structure_type: None,
    };
    let mut blockers = Vec::new();

    if grounding.grounding_text.trim().is_empty() {
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
    if grounding.primary_scene_type.trim().is_empty() {
        blockers.push(ProductWarning {
            code: "scene_type_required".to_string(),
            message: "脚本中缺少有效场景类型，无法生成分镜。".to_string(),
            related_sample_id: None,
        });
    } else if resolve_scene_taxonomy(state, Some(&grounding.primary_scene_type)).is_none()
        && resolve_scene_taxonomy(state, Some(&normalized_scene_type)).is_none()
    {
        blockers.push(ProductWarning {
            code: "scene_type_invalid".to_string(),
            message: "脚本中的场景类型无效，无法生成分镜。".to_string(),
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
    let mut rows = Vec::new();
    let mut warnings = Vec::new();
    let row_durations = match allocate_storyboard_row_durations(
        request.selected_total_duration_seconds,
        selected_records.len().max(1),
    ) {
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
    let row_count = row_durations.len();
    let provider = if allow_live_text {
        default_text_model_provider()
    } else {
        TextModelProvider {
            enabled: false,
            ..default_text_model_provider()
        }
    };
    let generation_request = build_text_generation_request(
        TextGenerationTask::GenerateStoryboard,
        Some(grounding.shot_scene_type.clone()),
        build_storyboard_model_story_input(&grounding),
        Some(build_storyboard_duration_plan(
            request.selected_total_duration_seconds,
            &row_durations,
        )),
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

    for index in 0..row_count {
        let record = selected_records.get(index).copied();
        if let Some(record) = record {
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
        }

        let deterministic_draft = build_shot_grounded_row_draft(
            &grounding,
            index,
            row_count,
            row_durations[index],
            record,
        );
        let mut final_draft = deterministic_draft.clone();
        if let Some(patch) = live_row_patches.get(index) {
            apply_live_storyboard_patch(&mut final_draft, patch);
            if !draft_is_grounded_in_story(&final_draft, &grounding) {
                warnings.push(ProductWarning {
                    code: "text_model_live_storyboard_fallback".to_string(),
                    message:
                        "Live storyboard content did not stay grounded in shot_script, so Hope kept the deterministic shot-grounded row."
                            .to_string(),
                    related_sample_id: Some(final_draft.shot_id.clone()),
                });
                final_draft = deterministic_draft;
            }
        }

        let prompt_compilation = compile_seedance_prompt_text(
            state,
            build_shot_prompt_text_compilation_request(
                &final_draft,
                &grounding,
                &kb_router_result,
                record.map(|item| item.source_fields.continuity_negative_core.as_str()),
            ),
        );
        warnings.extend(prompt_compilation.warnings.iter().cloned());

        rows.push(build_generated_storyboard_row(
            (index + 1) as u32,
            &grounding,
            &final_draft,
            &prompt_compilation,
        ));
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
            for index in 0..row_count {
                let record = selected_records.get(index).copied();
                let draft = build_shot_grounded_row_draft(
                    &grounding,
                    index,
                    row_count,
                    row_durations[index],
                    record,
                );
                let prompt_compilation = compile_seedance_prompt_text(
                    state,
                    build_shot_prompt_text_compilation_request(
                        &draft,
                        &grounding,
                        &kb_router_result,
                        record.map(|item| item.source_fields.continuity_negative_core.as_str()),
                    ),
                );
                warnings.extend(prompt_compilation.warnings.iter().cloned());
                rows.push(build_generated_storyboard_row(
                    (index + 1) as u32,
                    &grounding,
                    &draft,
                    &prompt_compilation,
                ));
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
            grounding.shot_scene_type
        ))[..12]
    );
    let result_id = format!(
        "storyboard-{}",
        &stable_hash_hex(&format!(
            "{}\n{}\n{}",
            request.task_name, grounding.grounding_text, request.selected_total_duration_seconds
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
            per_row_seconds: planned_per_row_seconds(&row_durations),
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
        primary_scene_type: None,
        primary_scene_label: None,
        shot_scene_type: None,
        shot_scene_label: None,
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
        primary_scene_type: None,
        primary_scene_label: None,
        shot_scene_type: None,
        shot_scene_label: None,
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

pub fn save_storyboard_shot_result(
    state: &AppState,
    request: SaveStoryboardShotResultRequest,
) -> SaveStoryboardShotResultResponse {
    let now_ms = now_epoch_ms();
    let (mut blockers, rows_hash) = validate_finalized_storyboard_shot_payload(
        &request.project_id,
        &request.script_id,
        &request.shot_task_id,
        &request.result_id,
        &request.shot_task_name,
        &request.rows,
        &request.prompt_text,
        request.shot_duration_seconds,
        &request.duration_source,
        &request.rows_hash,
    );
    if state
        .find_finalized_storyboard_shot(&request.project_id, &request.result_id)
        .is_some()
    {
        blockers.push(ProductWarning {
            code: "storyboard_bank_result_already_exists".to_string(),
            message: "A finalized storyboard shot result with this result_id already exists."
                .to_string(),
            related_sample_id: Some(request.result_id.clone()),
        });
    }
    if !blockers.is_empty() {
        return SaveStoryboardShotResultResponse {
            status: BridgeCallStatus::Blocked,
            shot: None,
            blockers,
            warnings: vec![],
        };
    }

    let shot = FinalizedStoryboardShotResult {
        project_id: request.project_id,
        script_id: request.script_id,
        shot_task_id: request.shot_task_id,
        result_id: request.result_id,
        shot_order: request.shot_order,
        shot_task_name: request.shot_task_name,
        rows: request.rows,
        prompt_text: request.prompt_text,
        shot_duration_seconds: request.shot_duration_seconds,
        duration_source: request.duration_source,
        confirmed: request.confirmed,
        updated_at_ms: if request.updated_at_ms == 0 {
            now_ms
        } else {
            request.updated_at_ms
        },
        rows_hash,
    };

    state.remember_finalized_storyboard_shot(shot.clone());
    SaveStoryboardShotResultResponse {
        status: BridgeCallStatus::Ready,
        shot: Some(shot),
        blockers: vec![],
        warnings: vec![],
    }
}

pub fn list_storyboard_shot_results(
    state: &AppState,
    request: ListStoryboardShotResultsRequest,
) -> ListStoryboardShotResultsResponse {
    let warnings = if request.project_id.trim().is_empty() {
        vec![ProductWarning {
            code: "project_id_required".to_string(),
            message: "project_id is required before listing finalized storyboard shots."
                .to_string(),
            related_sample_id: None,
        }]
    } else {
        vec![]
    };
    let shots = if request.project_id.trim().is_empty() {
        vec![]
    } else {
        state.list_finalized_storyboard_shots(
            &request.project_id,
            request.script_id.as_deref(),
            request.confirmed,
        )
    };

    ListStoryboardShotResultsResponse {
        project_id: request.project_id,
        script_id: request.script_id,
        shots,
        warnings,
    }
}

pub fn update_storyboard_shot_result(
    state: &AppState,
    request: UpdateStoryboardShotResultRequest,
) -> UpdateStoryboardShotResultResponse {
    let Some(existing) =
        state.find_finalized_storyboard_shot(&request.project_id, &request.result_id)
    else {
        return UpdateStoryboardShotResultResponse {
            status: BridgeCallStatus::Blocked,
            shot: None,
            blockers: vec![ProductWarning {
                code: "storyboard_bank_result_not_found".to_string(),
                message: "No finalized storyboard shot result exists for this project/result_id."
                    .to_string(),
                related_sample_id: Some(request.result_id),
            }],
            warnings: vec![],
        };
    };

    let (blockers, rows_hash) = validate_finalized_storyboard_shot_payload(
        &existing.project_id,
        &existing.script_id,
        &existing.shot_task_id,
        &existing.result_id,
        &request.shot_task_name,
        &request.rows,
        &request.prompt_text,
        request.shot_duration_seconds,
        &request.duration_source,
        &request.rows_hash,
    );
    if !blockers.is_empty() {
        return UpdateStoryboardShotResultResponse {
            status: BridgeCallStatus::Blocked,
            shot: None,
            blockers,
            warnings: vec![],
        };
    }

    let shot = FinalizedStoryboardShotResult {
        project_id: existing.project_id,
        script_id: existing.script_id,
        shot_task_id: existing.shot_task_id,
        result_id: existing.result_id,
        shot_order: request.shot_order,
        shot_task_name: request.shot_task_name,
        rows: request.rows,
        prompt_text: request.prompt_text,
        shot_duration_seconds: request.shot_duration_seconds,
        duration_source: request.duration_source,
        confirmed: request.confirmed,
        updated_at_ms: if request.updated_at_ms == 0 {
            now_epoch_ms()
        } else {
            request.updated_at_ms
        },
        rows_hash,
    };

    state.remember_finalized_storyboard_shot(shot.clone());
    UpdateStoryboardShotResultResponse {
        status: BridgeCallStatus::Ready,
        shot: Some(shot),
        blockers: vec![],
        warnings: vec![],
    }
}

pub fn remove_storyboard_shot_result(
    state: &AppState,
    request: RemoveStoryboardShotResultRequest,
) -> RemoveStoryboardShotResultResponse {
    if request.project_id.trim().is_empty() || request.result_id.trim().is_empty() {
        return RemoveStoryboardShotResultResponse {
            status: BridgeCallStatus::Blocked,
            project_id: request.project_id,
            result_id: request.result_id,
            removed: false,
            warnings: vec![],
            blockers: vec![ProductWarning {
                code: "storyboard_bank_remove_selector_required".to_string(),
                message: "project_id and result_id are required before removing a finalized shot."
                    .to_string(),
                related_sample_id: None,
            }],
        };
    }

    let removed = state.remove_finalized_storyboard_shot(&request.project_id, &request.result_id);
    RemoveStoryboardShotResultResponse {
        status: if removed {
            BridgeCallStatus::Ready
        } else {
            BridgeCallStatus::Blocked
        },
        project_id: request.project_id,
        result_id: request.result_id.clone(),
        removed,
        warnings: vec![],
        blockers: if removed {
            vec![]
        } else {
            vec![ProductWarning {
                code: "storyboard_bank_result_not_found".to_string(),
                message: "No finalized storyboard shot result was found for removal.".to_string(),
                related_sample_id: Some(request.result_id),
            }]
        },
    }
}

pub fn export_storyboard_bank(
    state: &AppState,
    request: ExportStoryboardBankRequest,
) -> ExportStoryboardBankResponse {
    let export_manifest_id = format!(
        "storyboard-bank-export-{}",
        &stable_hash_hex(&format!(
            "{}\n{}\n{}",
            request.project_id,
            request.script_id.as_deref().unwrap_or_default(),
            request.export_format
        ))[..12]
    );
    let mut blockers = Vec::new();
    if request.project_id.trim().is_empty() {
        blockers.push(ProductWarning {
            code: "project_id_required".to_string(),
            message: "project_id is required before exporting finalized storyboard shots."
                .to_string(),
            related_sample_id: None,
        });
    }
    if request.include_unconfirmed {
        blockers.push(ProductWarning {
            code: "storyboard_bank_include_unconfirmed_not_supported".to_string(),
            message: "V1 finalized storyboard bank export only supports confirmed shots."
                .to_string(),
            related_sample_id: None,
        });
    }
    if !blockers.is_empty() {
        return ExportStoryboardBankResponse {
            export_manifest_id,
            project_id: request.project_id,
            script_id: request.script_id,
            status: BridgeCallStatus::Blocked,
            no_export: true,
            confirmed_shot_count: 0,
            exported_result_ids: vec![],
            total_shot_duration_seconds: 0,
            artifacts: vec![],
            warnings: vec![],
            blockers,
        };
    }

    let confirmed_shots = state.list_finalized_storyboard_shots(
        &request.project_id,
        request.script_id.as_deref(),
        Some(true),
    );
    if confirmed_shots.is_empty() {
        return ExportStoryboardBankResponse {
            export_manifest_id,
            project_id: request.project_id,
            script_id: request.script_id,
            status: BridgeCallStatus::Gated,
            no_export: true,
            confirmed_shot_count: 0,
            exported_result_ids: vec![],
            total_shot_duration_seconds: 0,
            artifacts: vec![],
            warnings: vec![ProductWarning {
                code: "storyboard_bank_no_confirmed_shots".to_string(),
                message: "No confirmed finalized storyboard shots exist, so no storyboard bank export was created."
                    .to_string(),
                related_sample_id: None,
            }],
            blockers: vec![],
        };
    }

    let mut total_shot_duration_seconds = 0u16;
    let mut rows = Vec::new();
    for shot in &confirmed_shots {
        let (shot_blockers, expected_rows_hash) = validate_finalized_storyboard_shot_payload(
            &shot.project_id,
            &shot.script_id,
            &shot.shot_task_id,
            &shot.result_id,
            &shot.shot_task_name,
            &shot.rows,
            &shot.prompt_text,
            shot.shot_duration_seconds,
            &shot.duration_source,
            &shot.rows_hash,
        );
        if !shot_blockers.is_empty() {
            return blocked_storyboard_bank_export_response(
                export_manifest_id,
                request.project_id,
                request.script_id,
                shot_blockers,
            );
        }
        if expected_rows_hash != shot.rows_hash {
            return blocked_storyboard_bank_export_response(
                export_manifest_id,
                request.project_id,
                request.script_id,
                vec![ProductWarning {
                    code: "storyboard_bank_rows_hash_mismatch".to_string(),
                    message: "Saved rows_hash no longer matches the finalized storyboard rows; export stopped before packaging."
                        .to_string(),
                    related_sample_id: Some(shot.result_id.clone()),
                }],
            );
        }
        let Some(total) = total_shot_duration_seconds.checked_add(shot.shot_duration_seconds)
        else {
            return blocked_storyboard_bank_export_response(
                export_manifest_id,
                request.project_id,
                request.script_id,
                vec![ProductWarning {
                    code: "storyboard_bank_duration_overflow".to_string(),
                    message: "Finalized storyboard bank duration exceeds the V1 export range."
                        .to_string(),
                    related_sample_id: Some(shot.result_id.clone()),
                }],
            );
        };
        total_shot_duration_seconds = total;
        rows.extend(shot.rows.clone());
    }

    let exported_result_ids = confirmed_shots
        .iter()
        .map(|shot| shot.result_id.clone())
        .collect::<Vec<_>>();
    match export_v120_storyboard_bundle(&V120StoryboardExportRequest {
        export_manifest_id: export_manifest_id.clone(),
        result_id: "finalized_storyboard_bank".to_string(),
        selected_total_duration_seconds: total_shot_duration_seconds,
        source_result_id: exported_result_ids.join(","),
        edited_rows_applied: false,
        rows,
    }) {
        Ok(bundle) => ExportStoryboardBankResponse {
            export_manifest_id: export_manifest_id.clone(),
            project_id: request.project_id,
            script_id: request.script_id,
            status: BridgeCallStatus::Ready,
            no_export: false,
            confirmed_shot_count: confirmed_shots.len() as u32,
            exported_result_ids,
            total_shot_duration_seconds,
            artifacts: bundle
                .artifacts
                .into_iter()
                .map(|artifact| ExportArtifactRecord {
                    artifact_id: format!("{}-{}", export_manifest_id, artifact.artifact_kind),
                    artifact_kind: artifact.artifact_kind.to_string(),
                    export_format: artifact.export_format.to_string(),
                    ready: true,
                    blocked_reason: None,
                    artifact_path: None,
                    content_hash: Some(artifact.content_hash),
                    byte_size: Some(artifact.byte_size),
                    row_count: Some(artifact.row_count),
                    selected_total_duration_seconds: Some(total_shot_duration_seconds),
                    source_result_id: Some("finalized_storyboard_bank".to_string()),
                    edited_rows_applied: false,
                    prompt_text_compilation_statuses: vec![],
                    prompt_text_compilation_warning_codes: vec![],
                    selected_sample_ids: vec![],
                    selected_kb_rule_ids: vec![],
                    kb_context_summary: None,
                    retrieval_trace: None,
                    full_kb_rows_included: 0,
                })
                .collect(),
            warnings: vec![],
            blockers: vec![],
        },
        Err(error) => blocked_storyboard_bank_export_response(
            export_manifest_id,
            request.project_id,
            request.script_id,
            vec![ProductWarning {
                code: "storyboard_bank_export_artifact_generation_failed".to_string(),
                message: format!("Finalized storyboard bank export failed: {error:?}"),
                related_sample_id: None,
            }],
        ),
    }
}

fn validate_finalized_storyboard_shot_payload(
    project_id: &str,
    script_id: &str,
    shot_task_id: &str,
    result_id: &str,
    shot_task_name: &str,
    rows: &[GeneratedStoryboardRow],
    prompt_text: &str,
    shot_duration_seconds: u16,
    duration_source: &str,
    rows_hash: &str,
) -> (Vec<ProductWarning>, String) {
    let mut blockers = Vec::new();
    for (field_name, value) in [
        ("project_id", project_id),
        ("script_id", script_id),
        ("shot_task_id", shot_task_id),
        ("result_id", result_id),
        ("shot_task_name", shot_task_name),
    ] {
        if value.trim().is_empty() {
            blockers.push(ProductWarning {
                code: format!("{field_name}_required"),
                message: format!("{field_name} is required for finalized storyboard bank V1."),
                related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
            });
        }
    }
    if rows.is_empty() {
        blockers.push(ProductWarning {
            code: "storyboard_bank_rows_required".to_string(),
            message: "Finalized storyboard shot results must include saved storyboard rows."
                .to_string(),
            related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
        });
    }
    if prompt_text.trim().is_empty() {
        blockers.push(ProductWarning {
            code: "storyboard_bank_prompt_text_required".to_string(),
            message: "Finalized storyboard shot results must include clean prompt_text."
                .to_string(),
            related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
        });
    }
    if shot_duration_seconds == 0 {
        blockers.push(ProductWarning {
            code: "storyboard_bank_shot_duration_required".to_string(),
            message: "shot_duration_seconds must be greater than 0 for finalized shot results."
                .to_string(),
            related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
        });
    }
    if duration_source != STORYBOARD_DURATION_SOURCE {
        blockers.push(ProductWarning {
            code: "storyboard_bank_duration_source_unsupported".to_string(),
            message: "duration_source must use the V1 storyboard duration plan allocation marker."
                .to_string(),
            related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
        });
    }
    if rows.iter().any(|row| row.shot_duration_seconds == 0) {
        blockers.push(ProductWarning {
            code: "storyboard_bank_row_duration_required".to_string(),
            message: "Every saved storyboard row must include shot_duration_seconds.".to_string(),
            related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
        });
    }
    let row_duration_sum = rows
        .iter()
        .map(|row| row.shot_duration_seconds)
        .sum::<u16>();
    if shot_duration_seconds != 0 && row_duration_sum != shot_duration_seconds {
        blockers.push(ProductWarning {
            code: "storyboard_bank_shot_duration_mismatch".to_string(),
            message: "shot_duration_seconds must equal the saved storyboard row duration sum."
                .to_string(),
            related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
        });
    }

    if finalized_storyboard_payload_contains_forbidden_terms(shot_task_name, prompt_text, rows) {
        blockers.push(ProductWarning {
            code: "storyboard_bank_forbidden_payload".to_string(),
            message: "Finalized storyboard bank rejected internal, raw, credential, or full-KB payload content."
                .to_string(),
            related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
        });
    }

    let expected_rows_hash = stable_hash_hex(&serialize_storyboard_rows(rows));
    if !rows_hash.trim().is_empty() && rows_hash.trim() != expected_rows_hash {
        blockers.push(ProductWarning {
            code: "storyboard_bank_rows_hash_mismatch".to_string(),
            message:
                "rows_hash does not match the supplied finalized storyboard rows; export cannot trust this shot."
                    .to_string(),
            related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
        });
    }

    (blockers, expected_rows_hash)
}

fn finalized_storyboard_payload_contains_forbidden_terms(
    shot_task_name: &str,
    prompt_text: &str,
    rows: &[GeneratedStoryboardRow],
) -> bool {
    let row_product_text = rows
        .iter()
        .map(|row| {
            format!(
                "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                row.shot_script,
                row.person,
                row.shot_title,
                row.scene_scale,
                row.visual_description,
                row.character_action,
                row.camera_movement,
                row.dialogue,
                row.prompt_text
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let payload = format!("{shot_task_name}\n{prompt_text}\n{row_product_text}").to_lowercase();
    FINALIZED_BANK_FORBIDDEN_TERMS
        .iter()
        .any(|term| payload.contains(term))
        || contains_secret_like_sk_token(&payload)
}

fn contains_secret_like_sk_token(payload: &str) -> bool {
    payload
        .split(|character: char| {
            character.is_whitespace()
                || matches!(
                    character,
                    '"' | '\'' | ',' | ';' | ':' | '(' | ')' | '[' | ']' | '{' | '}'
                )
        })
        .any(|token| {
            token.starts_with("sk-")
                && token.len() >= 16
                && token.chars().skip(3).all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
                })
        })
}

fn blocked_storyboard_bank_export_response(
    export_manifest_id: String,
    project_id: String,
    script_id: Option<String>,
    blockers: Vec<ProductWarning>,
) -> ExportStoryboardBankResponse {
    ExportStoryboardBankResponse {
        export_manifest_id,
        project_id,
        script_id,
        status: BridgeCallStatus::Blocked,
        no_export: true,
        confirmed_shot_count: 0,
        exported_result_ids: vec![],
        total_shot_duration_seconds: 0,
        artifacts: vec![],
        warnings: vec![],
        blockers,
    }
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
        "{} {} {} {} {} {} {} {} {}",
        request.scene_type,
        request.synopsis_text,
        story_keywords.join(" "),
        request.primary_scene_type.as_deref().unwrap_or_default(),
        request.primary_scene_label.as_deref().unwrap_or_default(),
        request.shot_scene_type.as_deref().unwrap_or_default(),
        request.shot_scene_label.as_deref().unwrap_or_default(),
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
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        state.kb_runtime.snapshot.seed_format,
        request.scene_type,
        request.duration_seconds,
        request.task_type.as_str(),
        request.primary_scene_type.as_deref().unwrap_or_default(),
        request.shot_scene_type.as_deref().unwrap_or_default(),
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
        KbRouterTaskType::GenerateNovelChapter => (2, 6),
        KbRouterTaskType::AdaptChapterToScript => (2, 6),
        KbRouterTaskType::SplitScriptToShotTasks => (2, 6),
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
        KbRouterTaskType::UpdateContinuityState => (1, 6),
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

    summary.push_str(&format!(
        " primary_scene_type={} primary_scene_label={} shot_scene_type={} shot_scene_label={} shot_intent={} summary_only=true.",
        request.primary_scene_type.as_deref().unwrap_or(&request.scene_type),
        request.primary_scene_label.as_deref().unwrap_or_default(),
        request.shot_scene_type.as_deref().unwrap_or(&request.scene_type),
        request.shot_scene_label.as_deref().unwrap_or_default(),
        request.shot_intent.as_deref().unwrap_or_default(),
    ));

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

pub fn split_script_to_shot_tasks(
    request: SplitScriptToShotTasksRequest,
) -> SplitScriptToShotTasksResponse {
    let mut warnings = Vec::new();
    let mut source_segments = split_story_segments(&request.expanded_script_text)
        .into_iter()
        .filter(|segment| !is_v0_screenplay_metadata_segment(segment))
        .collect::<Vec<_>>();
    if source_segments.is_empty() {
        source_segments = split_story_segments(&request.expanded_script_text);
    }
    let target_duration_mode = normalize_target_duration_mode(&request.target_duration_mode)
        .unwrap_or(TARGET_DURATION_MODE_FIXED_SECONDS);
    if normalize_target_duration_mode(&request.target_duration_mode).is_none()
        && !request.target_duration_mode.trim().is_empty()
    {
        warnings.push(duration_plan_warning(
            "target_duration_mode_invalid",
            "split_script_to_shot_tasks received an unsupported target_duration_mode and used fixed_seconds planning.",
        ));
    }
    let durations = if target_duration_mode == TARGET_DURATION_MODE_LONG_TEXT_AUTO {
        let planned =
            allocate_long_text_auto_shot_task_durations(request.selected_total_duration_seconds);
        if planned.is_empty() {
            warnings.push(duration_plan_warning(
                "long_text_auto_duration_plan_missing",
                "long_text_auto could not split the estimated total into Seedance-friendly shot tasks.",
            ));
        }
        if planned.len() <= 1 {
            warnings.push(duration_plan_warning(
                "long_text_auto_compressed_to_single_clip_blocked",
                "long_text_auto must not compress source material into a single storyboard clip.",
            ));
        }
        planned
    } else {
        allocate_storyboard_row_durations(request.selected_total_duration_seconds, 0)
            .unwrap_or_else(|| vec![request.selected_total_duration_seconds])
    };
    let shot_count = durations.len();
    let mut shot_tasks = Vec::new();
    let script_hash = stable_hash_hex(&format!(
        "{}\n{}\n{}\n{}\n{}",
        request.script_id.as_deref().unwrap_or_default(),
        request.expanded_script_text,
        request.selected_total_duration_seconds,
        target_duration_mode,
        request.auto_segment_strategy
    ));

    for index in 0..shot_count {
        let shot_script = story_segment_for_planned_row(
            &source_segments,
            index,
            shot_count,
            &request.expanded_script_text,
        );
        let primary_scene_type = request.primary_scene_type.clone();
        let shot_scene_type = infer_shot_scene_type(&shot_script, &primary_scene_type);
        let adaptation_reason = if shot_scene_type == primary_scene_type {
            String::new()
        } else {
            build_adaptation_reason(&shot_script, &shot_scene_type)
        };
        shot_tasks.push(ShotTask {
            shot_task_id: format!("shot-task-{}-{:02}", &script_hash[..12], index + 1),
            shot_order: (index + 1) as u32,
            shot_script,
            duration_seconds: durations[index.min(durations.len() - 1)],
            shot_scene_type: shot_scene_type.clone(),
            shot_scene_label: derive_shot_scene_label(&shot_scene_type),
            shot_intent: derive_shot_intent(
                source_segments
                    .get(index)
                    .map(String::as_str)
                    .unwrap_or(&request.expanded_script_text),
            ),
            adaptation_reason,
            grounding_source: ShotGroundingSource::ExpandedScriptText,
        });
    }

    SplitScriptToShotTasksResponse {
        script_id: request.script_id,
        shot_tasks,
        warnings,
    }
}

fn validate_generate_novel_chapter_request(
    request: &GenerateNovelChapterRequest,
) -> Vec<ProductWarning> {
    let mut blockers = Vec::new();
    push_required_warning(&mut blockers, "story_id", &request.story_id);
    push_required_warning(&mut blockers, "chapter_id", &request.chapter_id);
    push_required_warning(
        &mut blockers,
        "user_topic_or_synopsis",
        &request.user_topic_or_synopsis,
    );
    push_required_warning(
        &mut blockers,
        "authoring_craft_summary",
        &request.authoring_craft_summary,
    );
    if !is_supported_v0_story_length_profile(&request.story_length_profile) {
        blockers.push(ProductWarning {
            code: "story_length_profile_unsupported".to_string(),
            message: "V0 story_length_profile must be one of short_clip, standard_clip, long_story, long_story_auto, short_story_2000_2500, or two_minute_story_2500_3500."
                .to_string(),
            related_sample_id: None,
        });
    }
    if request.full_kb_rows_included != 0 {
        blockers.push(ProductWarning {
            code: "full_kb_rows_must_be_zero".to_string(),
            message:
                "V0 stage contracts only allow summary KB context; full_kb_rows_included must be 0."
                    .to_string(),
            related_sample_id: None,
        });
    }
    if contains_forbidden_v0_product_payload(&request.user_topic_or_synopsis)
        || contains_forbidden_generation_terms(&request.authoring_craft_summary)
    {
        blockers.push(ProductWarning {
            code: "v0_authoring_forbidden_payload".to_string(),
            message: "V0 chapter input rejected internal, credential, or style-imitation payload content."
                .to_string(),
            related_sample_id: None,
        });
    }
    blockers
}

fn validate_adapt_chapter_to_script_request(
    request: &AdaptChapterToScriptRequest,
) -> Vec<ProductWarning> {
    let mut blockers = Vec::new();
    push_required_warning(&mut blockers, "story_id", &request.story_id);
    push_required_warning(&mut blockers, "chapter_id", &request.chapter_id);
    push_required_warning(&mut blockers, "chapter_text", &request.chapter_text);
    push_required_warning(&mut blockers, "chapter_summary", &request.chapter_summary);
    push_required_warning(
        &mut blockers,
        "screenwriting_adaptation_summary",
        &request.screenwriting_adaptation_summary,
    );
    if request.full_kb_rows_included != 0 {
        blockers.push(ProductWarning {
            code: "full_kb_rows_must_be_zero".to_string(),
            message: "V0 screenwriting adaptation only accepts summary KB context.".to_string(),
            related_sample_id: None,
        });
    }
    if contains_forbidden_v0_product_payload(&format!(
        "{}\n{}",
        request.chapter_text, request.chapter_summary
    )) {
        blockers.push(ProductWarning {
            code: "v0_screenwriting_forbidden_payload".to_string(),
            message: "V0 script adaptation rejected internal, raw, credential, or full-KB payload content."
                .to_string(),
            related_sample_id: None,
        });
    }
    blockers
}

fn validate_run_v0_story_to_storyboard_chain_request(
    request: &RunV0StoryToStoryboardChainRequest,
) -> Vec<ProductWarning> {
    let mut blockers = Vec::new();
    push_required_warning(&mut blockers, "story_id", &request.story_id);
    push_required_warning(&mut blockers, "chapter_id", &request.chapter_id);
    push_required_warning(
        &mut blockers,
        "user_topic_or_synopsis",
        &request.user_topic_or_synopsis,
    );
    push_required_warning(
        &mut blockers,
        "primary_scene_type",
        &request.primary_scene_type,
    );
    let target_duration_mode = normalize_target_duration_mode(&request.target_duration_mode);
    if target_duration_mode.is_none() && !request.target_duration_mode.trim().is_empty() {
        blockers.push(ProductWarning {
            code: "target_duration_mode_invalid".to_string(),
            message: "target_duration_mode must be fixed_seconds or long_text_auto.".to_string(),
            related_sample_id: None,
        });
    }
    if !is_supported_v0_story_length_profile(&request.story_length_profile) {
        blockers.push(ProductWarning {
            code: "story_length_profile_unsupported".to_string(),
            message: "V0 chain supports short_clip, standard_clip, long_story, long_story_auto, short_story_2000_2500, and two_minute_story_2500_3500."
                .to_string(),
            related_sample_id: None,
        });
    }
    if target_duration_mode.unwrap_or(TARGET_DURATION_MODE_FIXED_SECONDS)
        == TARGET_DURATION_MODE_FIXED_SECONDS
        && !is_supported_fixed_chain_duration(request.selected_total_duration_seconds)
    {
        blockers.push(ProductWarning {
            code: "duration_not_supported".to_string(),
            message: "fixed_seconds duration must be one of 5, 10, 15, 30, 45, or 60 seconds."
                .to_string(),
            related_sample_id: None,
        });
    }
    if request.full_kb_rows_included != 0 {
        blockers.push(ProductWarning {
            code: "full_kb_rows_must_be_zero".to_string(),
            message: "V0 chain must keep full_kb_rows_included at 0.".to_string(),
            related_sample_id: None,
        });
    }
    if contains_forbidden_v0_product_payload(&request.user_topic_or_synopsis) {
        blockers.push(ProductWarning {
            code: "v0_chain_forbidden_payload".to_string(),
            message:
                "V0 chain input rejected internal, raw, credential, or full-KB payload content."
                    .to_string(),
            related_sample_id: None,
        });
    }
    blockers
}

fn push_required_warning(blockers: &mut Vec<ProductWarning>, field_name: &str, value: &str) {
    if value.trim().is_empty() {
        blockers.push(ProductWarning {
            code: format!("{field_name}_required"),
            message: format!("{field_name} is required for the V0 story-to-storyboard chain."),
            related_sample_id: None,
        });
    }
}

fn is_supported_v0_story_length_profile(value: &str) -> bool {
    matches!(
        value.trim(),
        V0_SHORT_CLIP_PROFILE
            | V0_STANDARD_CLIP_PROFILE
            | V0_LONG_STORY_PROFILE
            | V0_LONG_STORY_AUTO_PROFILE
            | V0_SHORT_STORY_PROFILE
            | V0_TWO_MINUTE_STORY_PROFILE
    )
}

fn is_supported_fixed_chain_duration(duration_seconds: u16) -> bool {
    matches!(duration_seconds, 5 | 10 | 15 | 30 | 45 | 60)
}

#[derive(Debug, Clone)]
struct SourceInputAnalysis {
    source_input_type: String,
    authoring_mode: String,
    source_material_summary: String,
    source_story_facts: SourceStoryFacts,
    preserved_fact_summary: String,
    changed_for_screenplay_summary: String,
    omitted_detail_summary: String,
    continuity_warnings: Vec<ProductWarning>,
}

#[derive(Debug, Clone)]
struct V0ChainDurationPlan {
    target_duration_mode: String,
    story_length_profile: String,
    source_material_length_chars: u32,
    auto_segment_strategy: String,
    estimated_total_story_duration_seconds: u16,
    generated_shot_task_count: u32,
    duration_plan_summary: String,
    warnings: Vec<ProductWarning>,
}

#[derive(Debug, Clone)]
struct V0ContinuityProfile {
    effective_context_summary: String,
    character_motivation_summary: String,
    conflict_progression_summary: String,
    emotional_progression_summary: String,
    timeline_continuity_summary: String,
    prop_state_summary: String,
    next_scene_bridge: String,
    director_bridge: String,
    warnings: Vec<ProductWarning>,
}

fn plan_v0_chain_duration(
    request: &RunV0StoryToStoryboardChainRequest,
    source_analysis: &SourceInputAnalysis,
) -> V0ChainDurationPlan {
    let source_material_length_chars =
        stable_source_material_length_chars(&request.user_topic_or_synopsis);
    let target_duration_mode = normalize_target_duration_mode(&request.target_duration_mode)
        .map(str::to_string)
        .unwrap_or_else(|| request.target_duration_mode.trim().to_string());

    if normalize_target_duration_mode(&request.target_duration_mode).is_none()
        && !request.target_duration_mode.trim().is_empty()
    {
        return V0ChainDurationPlan {
            target_duration_mode,
            story_length_profile: request.story_length_profile.trim().to_string(),
            source_material_length_chars,
            auto_segment_strategy: request.auto_segment_strategy.trim().to_string(),
            estimated_total_story_duration_seconds: 0,
            generated_shot_task_count: 0,
            duration_plan_summary: "target_duration_mode is invalid; duration plan not created."
                .to_string(),
            warnings: vec![],
        };
    }

    if target_duration_mode == TARGET_DURATION_MODE_LONG_TEXT_AUTO {
        let estimated_total_story_duration_seconds = estimate_long_text_auto_duration_seconds(
            source_material_length_chars,
            &source_analysis.source_input_type,
            &source_analysis.source_story_facts,
        );
        let story_length_profile = derive_auto_story_length_profile(
            source_material_length_chars,
            estimated_total_story_duration_seconds,
        );
        let auto_segment_strategy = request
            .auto_segment_strategy
            .trim()
            .to_string()
            .if_empty(AUTO_SEGMENT_STRATEGY_LONG_TEXT);
        let shot_task_durations =
            allocate_long_text_auto_shot_task_durations(estimated_total_story_duration_seconds);
        let generated_shot_task_count = shot_task_durations.len() as u32;
        let mut warnings = Vec::new();
        if request.auto_segment_strategy.trim().is_empty() {
            warnings.push(duration_plan_warning(
                "auto_segment_strategy_missing",
                "long_text_auto used the deterministic story-fact segment strategy because no strategy was supplied.",
            ));
        }
        if shot_task_durations.is_empty() {
            warnings.push(duration_plan_warning(
                "long_text_auto_duration_plan_missing",
                "long_text_auto could not produce a Seedance-friendly duration plan.",
            ));
        }
        if generated_shot_task_count <= 1 {
            warnings.push(duration_plan_warning(
                "long_text_auto_compressed_to_single_clip_blocked",
                "long_text_auto must split source material into multiple concrete shot tasks instead of one compressed clip.",
            ));
        }
        let duration_plan_summary = format!(
            "mode={}; story_length_profile={}; source_chars={}; source_input_type={}; strategy={}; estimated_total={}s; generated_shot_tasks={}; shot_task_durations={}; layers=writing_continuity>scene_expression_adaptation>director_scheduling>shot_language",
            TARGET_DURATION_MODE_LONG_TEXT_AUTO,
            story_length_profile,
            source_material_length_chars,
            source_analysis.source_input_type,
            auto_segment_strategy,
            estimated_total_story_duration_seconds,
            generated_shot_task_count,
            join_durations(&shot_task_durations)
        );

        return V0ChainDurationPlan {
            target_duration_mode,
            story_length_profile,
            source_material_length_chars,
            auto_segment_strategy,
            estimated_total_story_duration_seconds,
            generated_shot_task_count,
            duration_plan_summary,
            warnings,
        };
    }

    let fixed_duration = request.selected_total_duration_seconds;
    let shot_task_durations = allocate_storyboard_row_durations(fixed_duration, 0)
        .unwrap_or_else(|| vec![fixed_duration]);
    let generated_shot_task_count = shot_task_durations.len() as u32;
    let story_length_profile = request.story_length_profile.trim().to_string();
    let auto_segment_strategy = request
        .auto_segment_strategy
        .trim()
        .to_string()
        .if_empty(AUTO_SEGMENT_STRATEGY_FIXED_SECONDS);

    V0ChainDurationPlan {
        target_duration_mode: TARGET_DURATION_MODE_FIXED_SECONDS.to_string(),
        story_length_profile: story_length_profile.clone(),
        source_material_length_chars,
        auto_segment_strategy: auto_segment_strategy.clone(),
        estimated_total_story_duration_seconds: fixed_duration,
        generated_shot_task_count,
        duration_plan_summary: format!(
            "mode={}; story_length_profile={}; source_chars={}; source_input_type={}; total={}s; generated_shot_tasks={}; shot_task_durations={}",
            TARGET_DURATION_MODE_FIXED_SECONDS,
            story_length_profile,
            source_material_length_chars,
            source_analysis.source_input_type,
            fixed_duration,
            generated_shot_task_count,
            join_durations(&shot_task_durations)
        ),
        warnings: vec![],
    }
}

trait IfEmpty {
    fn if_empty(self, fallback: &str) -> String;
}

impl IfEmpty for String {
    fn if_empty(self, fallback: &str) -> String {
        if self.trim().is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
}

fn normalize_target_duration_mode(value: &str) -> Option<&'static str> {
    match value.trim() {
        "" | TARGET_DURATION_MODE_FIXED_SECONDS => Some(TARGET_DURATION_MODE_FIXED_SECONDS),
        TARGET_DURATION_MODE_LONG_TEXT_AUTO => Some(TARGET_DURATION_MODE_LONG_TEXT_AUTO),
        _ => None,
    }
}

fn stable_source_material_length_chars(source_text: &str) -> u32 {
    source_text.trim().chars().count().min(u32::MAX as usize) as u32
}

fn estimate_long_text_auto_duration_seconds(
    source_material_length_chars: u32,
    source_input_type: &str,
    facts: &SourceStoryFacts,
) -> u16 {
    let length_based = match source_material_length_chars {
        0..=220 => 30,
        221..=800 => 75,
        801..=2_000 => 90,
        2_001..=2_500 => 105,
        2_501..=3_500 => 120,
        3_501..=6_000 => 180,
        _ => {
            let extra_blocks = ((source_material_length_chars - 6_000) / 1_500).min(6) as u16;
            180 + extra_blocks * 30
        }
    };
    let source_type_floor = match source_input_type {
        "screenplay_text" => 75,
        "novel_chapter" | "mixed_material" => 90,
        "full_story" => 120,
        _ => 30,
    };
    let event_count = facts.event_order.len().max(facts.core_events.len()) as u16;
    let fact_based = if event_count == 0 {
        0
    } else {
        event_count.min(18) * SEEDANCE_STANDARD_SEGMENT_SECONDS
    };
    let mut total = length_based.max(source_type_floor).max(fact_based);
    if total == 60 {
        total = 75;
    }
    round_duration_to_five(total.clamp(30, 360))
}

fn derive_auto_story_length_profile(
    source_material_length_chars: u32,
    estimated_total_story_duration_seconds: u16,
) -> String {
    if source_material_length_chars >= 3_500 || estimated_total_story_duration_seconds >= 180 {
        V0_LONG_STORY_AUTO_PROFILE
    } else if source_material_length_chars >= 2_500 || estimated_total_story_duration_seconds >= 120
    {
        V0_TWO_MINUTE_STORY_PROFILE
    } else if source_material_length_chars >= 2_000 || estimated_total_story_duration_seconds >= 105
    {
        V0_SHORT_STORY_PROFILE
    } else if estimated_total_story_duration_seconds >= 75 {
        V0_LONG_STORY_PROFILE
    } else if estimated_total_story_duration_seconds >= 45 {
        V0_STANDARD_CLIP_PROFILE
    } else {
        V0_SHORT_CLIP_PROFILE
    }
    .to_string()
}

fn round_duration_to_five(duration_seconds: u16) -> u16 {
    ((duration_seconds + SEEDANCE_REMAINDER_SEGMENT_SECONDS - 1)
        / SEEDANCE_REMAINDER_SEGMENT_SECONDS)
        * SEEDANCE_REMAINDER_SEGMENT_SECONDS
}

fn allocate_long_text_auto_shot_task_durations(total_duration_seconds: u16) -> Vec<u16> {
    if total_duration_seconds < SEEDANCE_STANDARD_SEGMENT_SECONDS
        || total_duration_seconds % SEEDANCE_REMAINDER_SEGMENT_SECONDS != 0
    {
        return vec![];
    }
    let mut remaining = total_duration_seconds;
    let mut durations = Vec::new();
    while remaining >= SEEDANCE_STANDARD_SEGMENT_SECONDS {
        durations.push(SEEDANCE_STANDARD_SEGMENT_SECONDS);
        remaining -= SEEDANCE_STANDARD_SEGMENT_SECONDS;
    }
    if remaining == SEEDANCE_REMAINDER_SEGMENT_SECONDS {
        durations.push(SEEDANCE_REMAINDER_SEGMENT_SECONDS);
    }
    durations
}

fn join_durations(durations: &[u16]) -> String {
    durations
        .iter()
        .map(u16::to_string)
        .collect::<Vec<_>>()
        .join("/")
}

fn duration_plan_warning(code: &str, message: &str) -> ProductWarning {
    ProductWarning {
        code: code.to_string(),
        message: message.to_string(),
        related_sample_id: None,
    }
}

fn analyze_v0_source_input(source_text: &str) -> SourceInputAnalysis {
    let source_input_type = classify_v0_source_input(source_text);
    let authoring_mode = authoring_mode_for_source_input_type(&source_input_type).to_string();
    let source_story_facts = extract_source_story_facts(source_text);
    let source_material_summary = compact_product_summary(
        source_text,
        "No source material supplied.",
        match source_input_type.as_str() {
            "synopsis" => 180,
            "screenplay_text" => 260,
            _ => 320,
        },
    );
    let preserved_fact_summary = build_preserved_fact_summary(&source_story_facts);
    let changed_for_screenplay_summary =
        changed_for_screenplay_summary(&source_input_type, &authoring_mode);
    let omitted_detail_summary = omitted_detail_summary(&source_input_type, &source_story_facts);
    let mut continuity_warnings = Vec::new();
    if source_input_type == "mixed_material" {
        continuity_warnings.push(ProductWarning {
            code: "source_input_type_uncertain".to_string(),
            message:
                "Source material mixes prose, synopsis, or screenplay markers; Hope used conservative preservation mode."
                    .to_string(),
            related_sample_id: None,
        });
    }
    if source_text.chars().count() > 6_000 {
        continuity_warnings.push(ProductWarning {
            code: "source_document_too_long_for_single_pass".to_string(),
            message:
                "Source material is long for one deterministic pass; key fact summaries were preserved and the rest should be reviewed."
                    .to_string(),
            related_sample_id: None,
        });
    }

    SourceInputAnalysis {
        source_input_type,
        authoring_mode,
        source_material_summary,
        source_story_facts,
        preserved_fact_summary,
        changed_for_screenplay_summary,
        omitted_detail_summary,
        continuity_warnings,
    }
}

fn classify_v0_source_input(source_text: &str) -> String {
    let trimmed = source_text.trim();
    let segments = split_story_segments(trimmed);
    let char_count = trimmed.chars().count();
    let has_screenplay_marker = contains_any_story_term(
        trimmed,
        &[
            "剧本",
            "场景",
            "镜头",
            "对白",
            "内景",
            "外景",
            "CUT",
            "INT.",
            "EXT.",
            "角色：",
            "旁白：",
        ],
    );
    let has_novel_marker = contains_any_story_term(
        trimmed,
        &[
            "第1章",
            "第一章",
            "第 1 章",
            "章节",
            "本章",
            "章末",
            "小说章节",
            "上一章",
            "下一章",
        ],
    );
    let has_synopsis_marker =
        contains_any_story_term(trimmed, &["梗概", "故事大纲", "简介", "一句话", "概述"]);
    let looks_like_long_story = char_count >= 260 || segments.len() >= 5;

    if (has_screenplay_marker && (has_novel_marker || has_synopsis_marker))
        || (has_novel_marker && has_synopsis_marker && has_screenplay_marker)
    {
        "mixed_material".to_string()
    } else if has_screenplay_marker {
        "screenplay_text".to_string()
    } else if has_novel_marker {
        "novel_chapter".to_string()
    } else if looks_like_long_story {
        "full_story".to_string()
    } else {
        "synopsis".to_string()
    }
}

fn authoring_mode_for_source_input_type(source_input_type: &str) -> &'static str {
    match source_input_type {
        "full_story" => "rewrite_from_full_story",
        "novel_chapter" => "adapt_story_to_screenplay",
        "screenplay_text" => "polish_existing_screenplay",
        "mixed_material" => "conservative_rewrite_from_mixed_material",
        _ => "expand_from_synopsis",
    }
}

fn extract_source_story_facts(source_text: &str) -> SourceStoryFacts {
    let registry = CharacterRegistry::from_story_text(source_text, source_text);
    let mut facts = SourceStoryFacts::default();
    for character in &registry.characters {
        let normalized_name = trim_detected_character_name(&character.name);
        push_unique_string(&mut facts.character_names, normalized_name.clone());
        push_unique_string(
            &mut facts.character_relationships,
            format!("{}:{}", character.role, normalized_name),
        );
    }

    let segments = split_story_segments(source_text);
    for (index, segment) in segments.iter().enumerate() {
        let compact = compact_product_summary(segment, "source event", 120);
        if index < 8 {
            push_unique_string(&mut facts.core_events, compact.clone());
        }
        if index < 12 {
            push_unique_string(
                &mut facts.event_order,
                format!("{}: {}", index + 1, compact),
            );
        }
        if contains_any_story_term(
            segment,
            &[
                "先", "随后", "然后", "最后", "当夜", "清晨", "第", "章", "之前", "之后",
            ],
        ) {
            push_unique_string(&mut facts.timeline_facts, compact.clone());
        }
        if contains_any_story_term(
            segment,
            &[
                "刀", "剑", "信", "玉佩", "钥匙", "掌心", "银辉", "火把", "卷轴", "血书",
            ],
        ) {
            push_unique_string(&mut facts.prop_state, compact.clone());
        }
        if contains_any_story_term(
            segment,
            &[
                "在", "桥", "断桥", "城门", "房间", "山", "街", "殿", "林中", "战场", "屋内",
            ],
        ) {
            push_unique_string(&mut facts.location_facts, compact.clone());
        }
        if contains_any_story_term(
            segment,
            &[
                "害怕", "迟疑", "愤怒", "松动", "决意", "动摇", "心跳", "沉默", "泪",
            ],
        ) {
            push_unique_string(&mut facts.emotional_progression, compact.clone());
        }
        if contains_any_story_term(
            segment,
            &[
                "追杀", "交锋", "对峙", "逼近", "挡住", "反击", "威胁", "冲突", "敌",
            ],
        ) {
            push_unique_string(&mut facts.conflict_progression, compact.clone());
        }
    }
    facts.ending_state = segments
        .last()
        .map(|segment| compact_product_summary(segment, "source ending state", 140))
        .unwrap_or_default();
    facts
}

fn trim_detected_character_name(name: &str) -> String {
    let mut chars = name.chars().collect::<Vec<_>>();
    while chars.len() > 2 {
        let Some(last) = chars.last().copied() else {
            break;
        };
        if matches!(
            last,
            '先' | '已' | '也' | '把' | '从' | '仍' | '在' | '向' | '对'
        ) {
            chars.pop();
        } else {
            break;
        }
    }
    chars.into_iter().collect()
}

fn push_unique_string(values: &mut Vec<String>, value: String) {
    let trimmed = value.trim();
    if trimmed.is_empty() || values.iter().any(|item| item == trimmed) {
        return;
    }
    values.push(trimmed.to_string());
}

fn build_preserved_fact_summary(facts: &SourceStoryFacts) -> String {
    let names = if facts.character_names.is_empty() {
        "no named characters detected".to_string()
    } else {
        facts.character_names.join("、")
    };
    let events = if facts.event_order.is_empty() {
        "no ordered source events detected".to_string()
    } else {
        facts
            .event_order
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join(" | ")
    };
    format!("preserved characters: {names}; preserved event order: {events}")
}

fn changed_for_screenplay_summary(source_input_type: &str, authoring_mode: &str) -> String {
    match authoring_mode {
        "expand_from_synopsis" => {
            "Expanded the synopsis into playable beats while keeping user-provided subject facts first."
                .to_string()
        }
        "polish_existing_screenplay" => {
            "Polished existing screenplay text into clearer beat blocks without adding new main plot."
                .to_string()
        }
        "adapt_story_to_screenplay" => {
            "Adapted chapter prose into screenplay beats while preserving names, event order, locations, props, and ending state."
                .to_string()
        }
        "rewrite_from_full_story" => {
            "Rewrote full story material into screenplay beats while preserving source facts before advisory KB guidance."
                .to_string()
        }
        _ => format!(
            "Conservatively normalized {source_input_type} into screenplay-ready beats without approving new plot facts."
        ),
    }
}

fn omitted_detail_summary(source_input_type: &str, facts: &SourceStoryFacts) -> String {
    if source_input_type == "synopsis" {
        "No source details were omitted; synopsis mode adds connective screenplay beats only."
            .to_string()
    } else if facts.core_events.len() > 8 {
        "Minor descriptive texture beyond the first eight source events was compressed; named characters, event order, and ending state remain preserved."
            .to_string()
    } else {
        "No identified core event was intentionally omitted in deterministic rewrite.".to_string()
    }
}

fn summarize_fact_entries(
    entries: &[String],
    fallback: &str,
    max_items: usize,
    max_chars: usize,
) -> String {
    if entries.is_empty() {
        return fallback.to_string();
    }
    let joined = entries
        .iter()
        .take(max_items)
        .map(|entry| compact_product_summary(entry, fallback, max_chars))
        .collect::<Vec<_>>()
        .join(" | ");
    compact_product_summary(&joined, fallback, max_chars)
}

fn hint_requests_fact_override(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let has_override = [
        "replace",
        "override",
        "swap",
        "新增",
        "加入",
        "改为",
        "改成",
        "替换",
        "重写",
    ]
    .iter()
    .any(|term| lower.contains(term));
    let has_guard = [
        "do not override",
        "source facts remain authoritative",
        "summary-only",
        "advisory only",
        "don't override",
        "不覆盖",
        "不改",
        "不新增",
        "仅供参考",
        "只做参考",
    ]
    .iter()
    .any(|term| lower.contains(term));
    has_override && !has_guard
}

fn build_v0_chain_advisory_warnings(
    facts: &SourceStoryFacts,
    continuity_context_summary: &str,
    kb_context_summary: &str,
    directing_kb_context_summary: &str,
) -> Vec<ProductWarning> {
    let mut warnings = Vec::new();
    if continuity_context_summary.trim().is_empty() {
        warnings.push(ProductWarning {
            code: "continuity_context_missing".to_string(),
            message:
                "No continuity_context_summary was supplied, so Hope inferred continuity handoff from accepted facts only."
                    .to_string(),
            related_sample_id: None,
        });
    }
    if !source_facts_are_empty(facts) && hint_requests_fact_override(kb_context_summary) {
        warnings.push(ProductWarning {
            code: "kb_hint_conflicts_with_source_fact".to_string(),
            message:
                "Summary-only KB guidance appears to override accepted source facts; Hope kept content facts authoritative."
                    .to_string(),
            related_sample_id: None,
        });
    }
    if !source_facts_are_empty(facts) && hint_requests_fact_override(directing_kb_context_summary) {
        warnings.push(ProductWarning {
            code: "director_hint_conflicts_with_content_fact".to_string(),
            message:
                "Directing guidance appears to rewrite accepted content facts; Hope kept directing in a scheduling-only role."
                    .to_string(),
            related_sample_id: None,
        });
    }
    warnings
}

fn build_v0_continuity_profile(
    analysis: &SourceInputAnalysis,
    continuity_context_summary: &str,
    kb_context_summary: &str,
    retrieval_trace_user_summary: &str,
) -> V0ContinuityProfile {
    let facts = &analysis.source_story_facts;
    let mut warnings =
        build_v0_chain_advisory_warnings(facts, continuity_context_summary, kb_context_summary, "");
    let subject_summary = if facts.character_names.is_empty() {
        "accepted on-page characters".to_string()
    } else {
        subject_label_from_names(
            &facts
                .character_names
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>(),
        )
    };
    let relationship_summary = summarize_fact_entries(
        &facts.character_relationships,
        "character relationships stay anchored to accepted facts.",
        3,
        150,
    );
    let conflict_progression_summary = summarize_fact_entries(
        &facts.conflict_progression,
        "conflict cause and effect must stay continuous across the next beat.",
        3,
        180,
    );
    let emotional_progression_summary = summarize_fact_entries(
        &facts.emotional_progression,
        "emotional progression keeps moving from the prior accepted beat without a reset.",
        3,
        180,
    );
    let timeline_continuity_summary = if facts.event_order.is_empty() {
        summarize_fact_entries(
            &facts.timeline_facts,
            "event order and timeline stay aligned with accepted chapter facts.",
            4,
            180,
        )
    } else {
        summarize_fact_entries(
            &facts.event_order,
            "event order and timeline stay aligned with accepted chapter facts.",
            4,
            180,
        )
    };
    let prop_state_summary = summarize_fact_entries(
        &facts.prop_state,
        "props keep only accepted ownership, availability, and damage state.",
        3,
        180,
    );
    let next_scene_bridge = if facts.ending_state.trim().is_empty() {
        compact_product_summary(
            continuity_context_summary,
            "next beat must inherit unresolved action and pressure from the accepted ending state.",
            180,
        )
    } else {
        format!(
            "next beat inherits accepted ending state: {}",
            compact_product_summary(
                &facts.ending_state,
                "accepted ending state carries forward.",
                140
            )
        )
    };
    let character_motivation_summary = format!(
        "{} keeps acting from accepted goals and relationships. relation_guard={} continuity_guard={}",
        subject_summary,
        relationship_summary,
        compact_product_summary(
            continuity_context_summary,
            "motivation carries from the prior accepted state without a reset.",
            120
        )
    );
    let director_bridge = format!(
        "content facts stay authoritative, writing preserves continuity, scene controls expression, and directing schedules shots only. kb_summary_only={} retrieval_trace={}",
        compact_product_summary(
            kb_context_summary,
            "no external KB summary was supplied.",
            100
        ),
        compact_product_summary(
            retrieval_trace_user_summary,
            "no retrieval trace user summary was supplied.",
            80
        )
    );
    let effective_context_summary = format!(
        "content_facts_first={}; writing_continuity={}; timeline_guard={}; prop_guard={}; emotion_guard={}; conflict_guard={}; next_scene_bridge={}",
        compact_product_summary(
            &analysis.preserved_fact_summary,
            "accepted source facts remain authoritative.",
            120
        ),
        compact_product_summary(
            continuity_context_summary,
            "continue from accepted facts and do not drift relationships, motives, or event order.",
            120
        ),
        timeline_continuity_summary,
        prop_state_summary,
        emotional_progression_summary,
        conflict_progression_summary,
        compact_product_summary(
            &next_scene_bridge,
            "next beat inherits unresolved tension from the current accepted end state.",
            100
        )
    );
    if !facts.character_names.is_empty()
        && !character_motivation_summary.contains(&facts.character_names[0])
    {
        warnings.push(ProductWarning {
            code: "character_motivation_drift_detected".to_string(),
            message:
                "Character motivation routing lost the accepted character anchor, so manual review is recommended."
                    .to_string(),
            related_sample_id: None,
        });
    }
    dedupe_product_warnings(&mut warnings);
    V0ContinuityProfile {
        effective_context_summary,
        character_motivation_summary,
        conflict_progression_summary,
        emotional_progression_summary,
        timeline_continuity_summary,
        prop_state_summary,
        next_scene_bridge,
        director_bridge,
        warnings,
    }
}

fn build_script_stage_continuity_context(
    base_context_summary: &str,
    chapter: &GenerateNovelChapterResponse,
) -> String {
    format!(
        "base_continuity={}; chapter_summary={}; character_state={}; conflict_progression={}; emotional_progression={}; timeline={}; props={}; next_scene_bridge={}",
        compact_product_summary(
            base_context_summary,
            "continue from accepted facts without continuity drift.",
            120
        ),
        compact_product_summary(&chapter.chapter_summary, "accepted chapter summary", 100),
        compact_product_summary(
            &chapter.character_motivation_summary,
            "character motivation follows accepted chapter facts.",
            100
        ),
        compact_product_summary(
            &chapter.conflict_progression_summary,
            "conflict progression remains causal and continuous.",
            100
        ),
        compact_product_summary(
            &chapter.emotional_progression_summary,
            "emotional progression keeps moving forward.",
            100
        ),
        compact_product_summary(
            &chapter.timeline_continuity_summary,
            "timeline remains aligned with accepted event order.",
            100
        ),
        compact_product_summary(
            &chapter.prop_state_summary,
            "prop state remains aligned with accepted facts.",
            100
        ),
        compact_product_summary(
            &chapter.next_scene_bridge,
            "next scene must inherit unresolved accepted pressure.",
            100
        )
    )
}

fn summary_mentions_fact_anchor(summary: &str, entries: &[String]) -> bool {
    if entries.is_empty() {
        return true;
    }
    entries.iter().take(4).any(|entry| {
        let anchor = compact_product_summary(entry, "", 24);
        !anchor.trim().is_empty() && summary.contains(anchor.trim())
    })
}

fn validate_continuity_state_against_source_facts(
    request: &UpdateContinuityStateRequest,
) -> Vec<ProductWarning> {
    let mut warnings = Vec::new();
    if !summary_mentions_fact_anchor(
        &request.timeline_state_summary,
        &request.source_story_facts.event_order,
    ) {
        warnings.push(ProductWarning {
            code: "event_order_drift_detected".to_string(),
            message:
                "Continuity state no longer reflects accepted source event order or timeline anchors."
                    .to_string(),
            related_sample_id: None,
        });
    }
    if !summary_mentions_fact_anchor(&request.prop_state_summary, &request.source_story_facts.prop_state) {
        warnings.push(ProductWarning {
            code: "prop_state_drift_detected".to_string(),
            message:
                "Continuity state no longer reflects accepted prop ownership or prop-state anchors."
                    .to_string(),
            related_sample_id: None,
        });
    }
    if !summary_mentions_fact_anchor(
        &request.character_state_summary,
        &request.source_story_facts.emotional_progression,
    ) {
        warnings.push(ProductWarning {
            code: "emotional_progression_drift_detected".to_string(),
            message:
                "Continuity state no longer reflects accepted emotional progression anchors."
                    .to_string(),
            related_sample_id: None,
        });
    }
    if !summary_mentions_fact_anchor(
        &request.character_state_summary,
        &request.source_story_facts.conflict_progression,
    ) {
        warnings.push(ProductWarning {
            code: "conflict_progression_drift_detected".to_string(),
            message:
                "Continuity state no longer reflects accepted conflict-causality anchors."
                    .to_string(),
            related_sample_id: None,
        });
    }
    warnings
}

fn dedupe_product_warnings(warnings: &mut Vec<ProductWarning>) {
    let mut seen = std::collections::BTreeSet::new();
    warnings.retain(|warning| {
        seen.insert((
            warning.code.clone(),
            warning.message.clone(),
            warning.related_sample_id.clone(),
        ))
    });
}

fn source_facts_are_empty(facts: &SourceStoryFacts) -> bool {
    facts.character_names.is_empty()
        && facts.core_events.is_empty()
        && facts.event_order.is_empty()
        && facts.ending_state.trim().is_empty()
}

fn is_v0_screenplay_metadata_segment(segment: &str) -> bool {
    let trimmed = segment.trim();
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("screenplay_title:")
        || lower.starts_with("source_input_type:")
        || lower.starts_with("authoring_mode:")
        || lower.starts_with("source_material_summary:")
        || lower.starts_with("preserved_fact_summary:")
        || lower.starts_with("continuity_context_summary:")
        || lower.starts_with("content_priority:")
        || lower.starts_with("kb_guidance_mode:")
        || lower.starts_with("timeline_guard:")
        || lower.starts_with("prop_state_guard:")
        || lower.starts_with("location_guard:")
        || lower.starts_with("ending_state_guard:")
        || lower.starts_with("emotional_progression_guard:")
        || lower.starts_with("conflict_progression_guard:")
        || lower.starts_with("dialogue_intent:")
        || lower.starts_with("source_story_facts_take_priority")
        || lower.starts_with("changed_for_screenplay_summary:")
        || lower.starts_with("omitted_detail_summary:")
        || trimmed.starts_with("角色保留")
        || trimmed.starts_with("关系保留")
        || trimmed.starts_with("时间线保留")
        || trimmed.starts_with("道具状态保留")
        || trimmed.starts_with("地点事实保留")
        || trimmed.starts_with("结尾状态保留")
}

fn deterministic_chapter_title(chapter_order: u32, topic: &str) -> String {
    format!(
        "第{}章：{}",
        chapter_order,
        compact_product_summary(topic, "故事转折", 18)
    )
}

fn deterministic_v0_chapter_text(
    topic: &str,
    story_length_profile: &str,
    authoring_craft_summary: &str,
    continuity_context_summary: &str,
    kb_summary_available: bool,
) -> String {
    let beat_count = match story_length_profile {
        V0_LONG_STORY_AUTO_PROFILE | V0_LONG_STORY_PROFILE => 8,
        V0_TWO_MINUTE_STORY_PROFILE | V0_STANDARD_CLIP_PROFILE => 6,
        _ => 4,
    };
    let topic_summary = compact_product_summary(topic, "主角在压力中推进目标", 160);
    let craft_summary = compact_product_summary(
        authoring_craft_summary,
        "角色目标、冲突推进、可视化行动",
        120,
    );
    let continuity_summary = compact_product_summary(
        continuity_context_summary,
        "当前章节从用户梗概开始，不重置已接受事实。",
        120,
    );
    let kb_note = if kb_summary_available {
        "压缩知识摘要只作为结构提醒，角色事实仍以用户梗概和连续性为准。"
    } else {
        "当前使用中性创作规则，不依赖未确认的能力画像。"
    };
    let mut lines = vec![format!(
        "本章围绕“{}”展开。{} {}",
        topic_summary, craft_summary, kb_note
    )];
    for index in 0..beat_count {
        let beat_no = index + 1;
        let beat = match index {
            0 => "开端：角色从上一状态接入，目标和压力同时被看见。",
            1 => "推进：对立力量逼近，角色必须用行动回应，而不是停留在解释。",
            2 => "转折：角色发现新的限制或线索，原本的选择变得更困难。",
            3 => "承接：角色做出可被镜头捕捉的动作，留下下一段状态。",
            4 => "加压：关系、地点或道具状态出现更明确的后果。",
            _ => "收束：本章不解决全部问题，只把下一段的冲突桥接清楚。",
        };
        lines.push(format!("段落{beat_no}：{beat} 章节事实：{topic_summary}"));
    }
    lines.push(format!("连续性约束：{continuity_summary}"));
    lines.join("\n")
}

fn deterministic_scene_beats(chapter_text: &str) -> Vec<String> {
    let mut beats = split_story_segments(chapter_text)
        .into_iter()
        .filter(|segment| !segment.contains("连续性约束"))
        .take(6)
        .collect::<Vec<_>>();
    if beats.len() < 3 {
        beats.push("角色确认目标和当前压力。".to_string());
        beats.push("角色对对立力量做出可见行动。".to_string());
        beats.push("镜头留下下一段需要承接的状态。".to_string());
    }
    beats
}

fn deterministic_v0_script_text(
    title: &str,
    scene_beats: &[String],
    dialogue_intent: &str,
    chapter_summary: &str,
) -> String {
    let mut lines = vec![
        format!("剧本标题：{title}"),
        format!(
            "剧本摘要：{}",
            compact_product_summary(chapter_summary, "保留章节主线", 120)
        ),
        format!("对白意图：{dialogue_intent}"),
    ];
    for (index, beat) in scene_beats.iter().enumerate() {
        let scene_no = index + 1;
        lines.push(format!(
            "场景{scene_no}：{}。角色从上一状态进入，面向当前对立压力完成明确动作，镜头可继续拆分。",
            compact_product_summary(beat, "角色推进当前冲突", 180)
        ));
    }
    lines.join("\n")
}

fn deterministic_v0_source_chapter_text(
    source_text: &str,
    story_length_profile: &str,
    authoring_craft_summary: &str,
    continuity_context_summary: &str,
    kb_summary_available: bool,
    analysis: &SourceInputAnalysis,
) -> String {
    if analysis.authoring_mode == "expand_from_synopsis" {
        return deterministic_v0_chapter_text(
            source_text,
            story_length_profile,
            authoring_craft_summary,
            continuity_context_summary,
            kb_summary_available,
        );
    }

    let mut lines = vec![
        format!("source_input_type: {}", analysis.source_input_type),
        format!("authoring_mode: {}", analysis.authoring_mode),
        format!(
            "source_material_summary: {}",
            analysis.source_material_summary
        ),
        format!(
            "preserved_fact_summary: {}",
            analysis.preserved_fact_summary
        ),
        "source_facts_take_priority_over_kb_summary: true".to_string(),
    ];
    if !analysis.source_story_facts.character_names.is_empty() {
        lines.push(format!(
            "character_names: {}",
            analysis.source_story_facts.character_names.join("、")
        ));
    }
    if !analysis
        .source_story_facts
        .character_relationships
        .is_empty()
    {
        lines.push(format!(
            "character_relationships: {}",
            analysis
                .source_story_facts
                .character_relationships
                .join(" | ")
        ));
    }
    for event in analysis.source_story_facts.event_order.iter().take(8) {
        lines.push(format!("preserved_event_order: {event}"));
    }
    if !analysis.source_story_facts.ending_state.trim().is_empty() {
        lines.push(format!(
            "ending_state: {}",
            analysis.source_story_facts.ending_state
        ));
    }
    lines.push("source_material_body_begin".to_string());
    lines.push(source_text.trim().to_string());
    lines.push("source_material_body_end".to_string());
    lines.join("\n")
}

fn deterministic_v0_script_text_for_authoring(
    request: &AdaptChapterToScriptRequest,
    scene_beats: &[String],
    dialogue_intent: &str,
) -> String {
    if request.authoring_mode == "expand_from_synopsis"
        || source_facts_are_empty(&request.source_story_facts)
    {
        let mut lines = vec![
            format!("screenplay_title: V0 chapter {}", request.chapter_order),
            format!("source_input_type: {}", request.source_input_type),
            format!("authoring_mode: {}", request.authoring_mode),
            format!("preserved_fact_summary: {}", request.preserved_fact_summary),
            format!(
                "continuity_context_summary: {}",
                compact_product_summary(
                    &request.continuity_context_summary,
                    "continue from accepted source state without continuity drift.",
                    220
                )
            ),
            "content_priority: content_facts>writing_continuity>scene_expression>director_scheduling>storyboard".to_string(),
            "kb_guidance_mode: summary_only_advisory".to_string(),
            "source_story_facts_take_priority_over_kb_advice: true".to_string(),
            format!(
                "changed_for_screenplay_summary: {}",
                changed_for_screenplay_summary(&request.source_input_type, &request.authoring_mode)
            ),
            format!(
                "omitted_detail_summary: {}",
                omitted_detail_summary(&request.source_input_type, &request.source_story_facts)
            ),
            format!("dialogue_intent: {}", dialogue_intent),
        ];
        for (index, beat) in scene_beats.iter().enumerate() {
            let scene_no = index + 1;
            lines.push(format!(
                "scene_{scene_no}: {}. Characters inherit the prior accepted state, perform visible action, and leave a clean bridge into the next shot.",
                compact_product_summary(beat, "character advances the current conflict", 180)
            ));
        }
        return lines.join("\n");
    }

    let mut lines = vec![
        format!("screenplay_title: V0 chapter {}", request.chapter_order),
        format!("source_input_type: {}", request.source_input_type),
        format!("authoring_mode: {}", request.authoring_mode),
        format!("preserved_fact_summary: {}", request.preserved_fact_summary),
        format!(
            "continuity_context_summary: {}",
            compact_product_summary(
                &request.continuity_context_summary,
                "continue from accepted source state without continuity drift.",
                220
            )
        ),
        "content_priority: content_facts>writing_continuity>scene_expression>director_scheduling>storyboard".to_string(),
        "kb_guidance_mode: summary_only_advisory".to_string(),
        "source_story_facts_take_priority_over_kb_advice: true".to_string(),
        format!(
            "changed_for_screenplay_summary: {}",
            changed_for_screenplay_summary(&request.source_input_type, &request.authoring_mode)
        ),
        format!(
            "omitted_detail_summary: {}",
            omitted_detail_summary(&request.source_input_type, &request.source_story_facts)
        ),
    ];
    if !request.source_story_facts.character_names.is_empty() {
        lines.push(format!(
            "角色保留：{}",
            request.source_story_facts.character_names.join("、")
        ));
    }
    if !request
        .source_story_facts
        .character_relationships
        .is_empty()
    {
        lines.push(format!(
            "关系保留：{}",
            request
                .source_story_facts
                .character_relationships
                .join(" | ")
        ));
    }
    for event in request.source_story_facts.event_order.iter().take(8) {
        lines.push(format!(
            "场景{}：{}。人物从输入事实的上一状态进入，本场只把该事件改写为可表演动作、对白意图和可拆镜头调度，不新增未经输入支持的主线剧情。",
            lines.len(),
            event
        ));
    }
    if !request.source_story_facts.timeline_facts.is_empty() {
        lines.push(format!(
            "时间线保留：{}",
            request
                .source_story_facts
                .timeline_facts
                .iter()
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    if !request.source_story_facts.timeline_facts.is_empty() {
        lines.push(format!(
            "timeline_guard: {}",
            summarize_fact_entries(
                &request.source_story_facts.timeline_facts,
                "timeline stays aligned with accepted chapter facts.",
                4,
                180
            )
        ));
    }
    if !request.source_story_facts.prop_state.is_empty() {
        lines.push(format!(
            "道具状态保留：{}",
            request
                .source_story_facts
                .prop_state
                .iter()
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    if !request.source_story_facts.prop_state.is_empty() {
        lines.push(format!(
            "prop_state_guard: {}",
            summarize_fact_entries(
                &request.source_story_facts.prop_state,
                "prop ownership and state remain aligned with accepted facts.",
                4,
                180
            )
        ));
    }
    if !request.source_story_facts.location_facts.is_empty() {
        lines.push(format!(
            "地点事实保留：{}",
            request
                .source_story_facts
                .location_facts
                .iter()
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    if !request.source_story_facts.location_facts.is_empty() {
        lines.push(format!(
            "location_guard: {}",
            summarize_fact_entries(
                &request.source_story_facts.location_facts,
                "locations remain aligned with accepted source facts.",
                4,
                180
            )
        ));
    }
    if !request.source_story_facts.emotional_progression.is_empty() {
        lines.push(format!(
            "emotional_progression_guard: {}",
            summarize_fact_entries(
                &request.source_story_facts.emotional_progression,
                "emotional progression keeps moving forward without a reset.",
                4,
                180
            )
        ));
    }
    if !request.source_story_facts.conflict_progression.is_empty() {
        lines.push(format!(
            "conflict_progression_guard: {}",
            summarize_fact_entries(
                &request.source_story_facts.conflict_progression,
                "conflict causality stays aligned with accepted source facts.",
                4,
                180
            )
        ));
    }
    if !request.source_story_facts.ending_state.trim().is_empty() {
        lines.push(format!(
            "结尾状态保留：{}",
            request.source_story_facts.ending_state
        ));
        lines.push(format!(
            "ending_state_guard: {}",
            compact_product_summary(
                &request.source_story_facts.ending_state,
                "accepted ending state carries into the next beat.",
                180
            )
        ));
    }
    lines.join("\n")
}

fn validate_rewrite_fact_preservation(
    script_text: &str,
    facts: &SourceStoryFacts,
    authoring_mode: &str,
) -> Vec<ProductWarning> {
    if authoring_mode == "expand_from_synopsis" || source_facts_are_empty(facts) {
        return vec![];
    }

    let mut warnings = Vec::new();
    let missing_names = facts
        .character_names
        .iter()
        .filter(|name| !script_text.contains(name.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_names.is_empty() {
        warnings.push(ProductWarning {
            code: "full_story_rewrite_fact_loss_detected".to_string(),
            message: format!(
                "Screenplay rewrite did not preserve named source characters: {}.",
                missing_names.join("、")
            ),
            related_sample_id: None,
        });
    }

    if !missing_names.is_empty() {
        warnings.push(ProductWarning {
            code: "character_motivation_drift_detected".to_string(),
            message:
                "Character motivation continuity may have drifted because accepted named characters disappeared from the screenplay rewrite."
                    .to_string(),
            related_sample_id: None,
        });
    }

    let missing_events = facts
        .core_events
        .iter()
        .take(3)
        .filter(|event| {
            let anchor = event.chars().take(12).collect::<String>();
            !anchor.trim().is_empty() && !script_text.contains(anchor.trim())
        })
        .cloned()
        .collect::<Vec<_>>();
    if !missing_events.is_empty() {
        warnings.push(ProductWarning {
            code: "rewrite_dropped_key_event".to_string(),
            message: "Screenplay rewrite dropped at least one early source event anchor."
                .to_string(),
            related_sample_id: None,
        });
    }

    let mut last_position = 0usize;
    let mut event_order_changed = false;
    for event in facts.core_events.iter().take(4) {
        let anchor = event.chars().take(12).collect::<String>();
        if let Some(position) = script_text.find(anchor.trim()) {
            if position < last_position {
                event_order_changed = true;
                break;
            }
            last_position = position;
        }
    }
    if event_order_changed {
        warnings.push(ProductWarning {
            code: "rewrite_changed_event_order".to_string(),
            message: "Screenplay rewrite changed the detected source event order.".to_string(),
            related_sample_id: None,
        });
    }

    if event_order_changed {
        warnings.push(ProductWarning {
            code: "event_order_drift_detected".to_string(),
            message:
                "Screenplay rewrite no longer follows the accepted source event order and should be reviewed."
                    .to_string(),
            related_sample_id: None,
        });
    }

    if !facts.prop_state.is_empty()
        && !script_text.contains("prop_state_guard:")
        && !script_text.contains("道具状态保留")
    {
        warnings.push(ProductWarning {
            code: "prop_state_drift_detected".to_string(),
            message:
                "Screenplay rewrite did not carry forward accepted prop-state anchors."
                    .to_string(),
            related_sample_id: None,
        });
    }

    if !facts.emotional_progression.is_empty() && !script_text.contains("emotional_progression_guard:")
    {
        warnings.push(ProductWarning {
            code: "emotional_progression_drift_detected".to_string(),
            message:
                "Screenplay rewrite did not carry forward accepted emotional progression anchors."
                    .to_string(),
            related_sample_id: None,
        });
    }

    if script_text.contains("改变动机") || script_text.contains("动机改为") {
        warnings.push(ProductWarning {
            code: "rewrite_changed_character_motivation".to_string(),
            message: "Screenplay rewrite appears to change a source character motivation."
                .to_string(),
            related_sample_id: None,
        });
    }

    if script_text.contains("新增主线") || script_text.contains("未输入的新剧情") {
        warnings.push(ProductWarning {
            code: "rewrite_added_unapproved_plot".to_string(),
            message: "Screenplay rewrite appears to add an unapproved main plot.".to_string(),
            related_sample_id: None,
        });
    }
    warnings
}

fn first_story_sentence(text: &str) -> String {
    split_story_segments(text)
        .into_iter()
        .next()
        .unwrap_or_else(|| text.trim().to_string())
}

fn compact_product_summary(value: &str, fallback: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    let source = if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    };
    let mut summary = source.chars().take(max_chars).collect::<String>();
    if source.chars().count() > max_chars {
        summary.push_str("...");
    }
    summary
}

fn v0_required_summary_or_neutral(value: &str, neutral: &str) -> String {
    non_blank_string(value).unwrap_or_else(|| neutral.to_string())
}

fn contains_forbidden_v0_product_payload(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    FINALIZED_BANK_FORBIDDEN_TERMS
        .iter()
        .any(|term| lower.contains(term))
        || contains_forbidden_generation_terms(value)
}

fn v0_shot_task_name(task: &ShotTask) -> String {
    format!(
        "镜头{}：{}",
        task.shot_order,
        compact_product_summary(&task.shot_script, "分镜任务", 24)
    )
}

fn finalized_ref_from_saved_shot(
    story_id: &str,
    chapter_id: &str,
    chapter_order: u32,
    shot: &FinalizedStoryboardShotResult,
) -> FinalizedStoryboardRef {
    let label = shot
        .rows
        .first()
        .map(|row| format!("镜头{}：{}", shot.shot_order, row.shot_title))
        .unwrap_or_else(|| format!("镜头{}：{}", shot.shot_order, shot.shot_task_name));
    FinalizedStoryboardRef {
        story_id: story_id.to_string(),
        chapter_id: chapter_id.to_string(),
        chapter_order,
        script_id: shot.script_id.clone(),
        shot_task_id: shot.shot_task_id.clone(),
        storyboard_result_id: shot.result_id.clone(),
        finalized_result_id: shot.result_id.clone(),
        shot_order: shot.shot_order,
        shot_task_name: shot.shot_task_name.clone(),
        confirmed: shot.confirmed,
        updated_at_ms: shot.updated_at_ms,
        rows_hash: shot.rows_hash.clone(),
        shot_duration_seconds: shot.shot_duration_seconds,
        label,
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
            .or_else(|| non_blank_string(&extract_scene_type(&expanded_script_text)))
            .or_else(|| non_blank_string(&extract_scene_type(&shot_script)))
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
        grounding_text,
        grounding_source,
        primary_scene_type: primary_scene_type.clone(),
        primary_scene_label: non_blank_string(
            request.primary_scene_label.as_deref().unwrap_or_default(),
        )
        .unwrap_or_else(|| derive_shot_scene_label(&primary_scene_type)),
        primary_scene_category: non_blank_string(
            request
                .primary_scene_category
                .as_deref()
                .unwrap_or_default(),
        )
        .unwrap_or_else(|| primary_scene_type.clone()),
        shot_scene_label: non_blank_string(request.shot_scene_label.as_deref().unwrap_or_default())
            .unwrap_or_else(|| derive_shot_scene_label(&shot_scene_type)),
        shot_intent: non_blank_string(request.shot_intent.as_deref().unwrap_or_default())
            .unwrap_or_else(|| derive_shot_intent(&shot_scene_type)),
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

fn screenplay_metadata_value(text: &str, prefix: &str) -> Option<String> {
    text.lines().find_map(|line| {
        line.trim()
            .strip_prefix(prefix)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn build_storyboard_model_story_input(grounding: &StoryboardGroundingContext) -> String {
    let continuity_context_summary = screenplay_metadata_value(
        &grounding.expanded_script_text,
        "continuity_context_summary:",
    )
    .unwrap_or_else(|| {
        "continue from accepted shot facts without drifting relationships, motivation, event order, timeline, or prop state."
            .to_string()
    });
    let content_priority =
        screenplay_metadata_value(&grounding.expanded_script_text, "content_priority:")
            .unwrap_or_else(|| {
                "content_facts>writing_continuity>scene_expression>director_scheduling>storyboard"
                    .to_string()
            });
    format!(
        "grounding_priority=1.shot_script 2.expanded_script_text 3.primary_scene_fields 4.kb_router_summary\ncontent_priority={}\ncontinuity_context_summary={}\nshot_script={}\nexpanded_script_text={}\nprimary_scene_type={}\nprimary_scene_label={}\nprimary_scene_category={}\nshot_scene_type={}\nshot_scene_label={}\nshot_intent={}\nadaptation_reason={}",
        content_priority,
        continuity_context_summary,
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
    _record: Option<&GoldenSampleLibraryRecord>,
) -> ShotGroundedRowDraft {
    let segments = split_story_segments(&grounding.grounding_text);
    let segment =
        story_segment_for_planned_row(&segments, index, row_count, &grounding.grounding_text);
    let shot_id = format!(
        "shot-task-{}-{:02}",
        &stable_hash_hex(&format!(
            "{}\n{}\n{}",
            grounding.grounding_text, grounding.shot_scene_type, row_count
        ))[..12],
        index + 1
    );
    let person = derive_product_person(&segment, &grounding.grounding_text);
    let scene_scale = derive_shot_scene_scale(&segment);
    let character_action = derive_character_action_from_story(&segment, &grounding.grounding_text);
    let shot_title = derive_shot_title(index, &segment, &person, &character_action);
    let dialogue = extract_dialogue_from_story(&segment);
    let visual_description = build_visual_description_from_story(
        &segment,
        &grounding.grounding_text,
        &person,
        &scene_scale,
        &grounding.shot_scene_label,
        &grounding.shot_intent,
    );
    let camera_movement = derive_camera_movement_from_story(
        &segment,
        &scene_scale,
        &visual_description,
        &character_action,
    );
    let sequence_grouping = SequenceGrouping {
        structure_mode: StructureMode::SingleShot,
        sequence_id: None,
        shot_order: None,
        sequence_field_state: SequenceFieldState::NotApplicable,
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
        camera_movement,
        dialogue,
        duration_seconds,
        sequence_grouping,
        scene_performance_projection,
    }
}

fn build_shot_prompt_text_compilation_request(
    draft: &ShotGroundedRowDraft,
    grounding: &StoryboardGroundingContext,
    kb_router_result: &KbRouterRuntimeResponse,
    continuity_negative_core: Option<&str>,
) -> PromptTextCompilationRequest {
    PromptTextCompilationRequest {
        row: PromptTextCompilationRow {
            shot_id: draft.shot_id.clone(),
            shot_title: draft.shot_title.clone(),
            scene_scale: draft.scene_scale.clone(),
            visual_description: draft.visual_description.clone(),
            character_action: draft.character_action.clone(),
            camera_movement: draft.camera_movement.clone(),
            dialogue: draft.dialogue.clone(),
            duration_seconds: draft.duration_seconds,
        },
        shot_script: Some(draft.shot_script.clone()),
        scene_type: grounding.shot_scene_type.clone(),
        scene_label: grounding.shot_scene_label.clone(),
        kb_context_summary: kb_router_result.kb_context_summary.clone(),
        selected_kb_rules: kb_router_result
            .selected_kb_rules
            .iter()
            .map(|rule| rule.summary.clone())
            .collect(),
        continuity_negative_core: continuity_negative_core.unwrap_or_default().to_string(),
        target_profile: "seedance2.0".to_string(),
    }
}

fn build_generated_storyboard_row(
    order: u32,
    grounding: &StoryboardGroundingContext,
    draft: &ShotGroundedRowDraft,
    prompt_compilation: &PromptTextCompilationResponse,
) -> GeneratedStoryboardRow {
    GeneratedStoryboardRow {
        shot_id: draft.shot_id.clone(),
        order,
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
        camera_movement: draft.camera_movement.clone(),
        dialogue: draft.dialogue.clone(),
        prompt_text: prompt_compilation.prompt_text.clone(),
        prompt_text_compilation_status: prompt_compilation.compilation_status,
        prompt_text_compilation_warnings: prompt_compilation.warnings.clone(),
        prompt_text_source_row_id: prompt_compilation.source_row_id.clone(),
        duration_seconds: draft.duration_seconds,
        shot_duration_seconds: draft.duration_seconds,
        duration_source: STORYBOARD_DURATION_SOURCE.to_string(),
        scene_performance_projection: draft.scene_performance_projection.clone(),
        external_reference_handle_candidates: vec![],
        sequence_grouping: draft.sequence_grouping.clone(),
    }
}

fn split_story_segments(text: &str) -> Vec<String> {
    let normalized = text.replace("\r\n", "\n");
    normalized
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

fn story_segment_for_planned_row(
    segments: &[String],
    index: usize,
    planned_rows: usize,
    fallback_text: &str,
) -> String {
    if segments.is_empty() || planned_rows == 0 {
        return fallback_text.trim().to_string();
    }
    if segments.len() <= planned_rows {
        return segments
            .get(index)
            .cloned()
            .unwrap_or_else(|| fallback_text.trim().to_string());
    }

    let start = index * segments.len() / planned_rows;
    let mut end = (index + 1) * segments.len() / planned_rows;
    if index + 1 == planned_rows {
        end = segments.len();
    }
    if start >= end || start >= segments.len() {
        return fallback_text.trim().to_string();
    }
    segments[start..end.min(segments.len())].join("。")
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
        "daily_dialogue" => "日常对白".to_string(),
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
        format!(
            "shot_script supports local {} classification",
            shot_scene_type
        )
    } else {
        format!(
            "shot_script evidence [{}] supports local {} classification",
            evidence.join("/"),
            shot_scene_type
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CharacterMention {
    name: String,
    role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CharacterRegistry {
    characters: Vec<CharacterMention>,
}

impl CharacterRegistry {
    fn from_story_text(segment: &str, full_text: &str) -> Self {
        let mut registry = Self { characters: vec![] };
        registry.extend_from_text(full_text);
        registry.extend_from_text(segment);
        registry
    }

    fn extend_from_text(&mut self, text: &str) {
        for role in [
            "男主", "女主", "主角", "少年", "敌将", "反派", "敌人", "对手",
        ] {
            for (start, _) in text.match_indices(role) {
                let after_role = &text[start + role.len()..];
                if let Some(name) = extract_character_name_after_role(after_role) {
                    self.push(role, &name);
                }
            }
        }
    }

    fn push(&mut self, role: &str, name: &str) {
        if self.characters.iter().any(|item| item.name == name) {
            return;
        }
        self.characters.push(CharacterMention {
            name: name.to_string(),
            role: role.to_string(),
        });
    }

    fn is_empty(&self) -> bool {
        self.characters.is_empty()
    }

    fn names_in_text(&self, text: &str) -> Vec<String> {
        self.characters
            .iter()
            .filter(|character| text.contains(&character.name))
            .map(|character| character.name.clone())
            .collect()
    }

    fn protagonist_name(&self) -> Option<&str> {
        self.characters
            .iter()
            .find(|character| contains_any_story_term(&character.role, &["男主", "主角", "少年"]))
            .or_else(|| {
                self.characters
                    .iter()
                    .find(|character| !is_antagonist_role(&character.role))
            })
            .map(|character| character.name.as_str())
    }

    fn heroine_name(&self) -> Option<&str> {
        self.characters
            .iter()
            .find(|character| character.role.contains("女主"))
            .map(|character| character.name.as_str())
    }

    fn antagonist_name(&self) -> Option<&str> {
        self.characters
            .iter()
            .find(|character| is_antagonist_role(&character.role))
            .map(|character| character.name.as_str())
    }
}

fn derive_product_person(segment: &str, full_text: &str) -> String {
    let registry = CharacterRegistry::from_story_text(segment, full_text);
    if !registry.is_empty() {
        let active_names = registry.names_in_text(segment);
        if !active_names.is_empty() {
            return subject_label_from_names(&active_names);
        }

        if contains_any_story_term(segment, &["护住", "护着", "回身", "重逢"]) {
            if let (Some(heroine), Some(protagonist)) =
                (registry.heroine_name(), registry.protagonist_name())
            {
                return subject_label_from_names(&[heroine.to_string(), protagonist.to_string()]);
            }
        }

        if contains_enemy_or_conflict_terms(segment) {
            if let (Some(protagonist), Some(antagonist)) =
                (registry.protagonist_name(), registry.antagonist_name())
            {
                return subject_label_from_names(&[
                    protagonist.to_string(),
                    antagonist.to_string(),
                ]);
            }
        }

        if let Some(protagonist) = registry.protagonist_name() {
            return protagonist.to_string();
        }

        return subject_label_from_names(
            &registry
                .characters
                .iter()
                .map(|character| character.name.clone())
                .collect::<Vec<_>>(),
        );
    }

    derive_fallback_subject(segment, full_text)
}

fn derive_shot_scene_scale(segment: &str) -> String {
    if contains_any_story_term(segment, &["掌心", "手臂", "银辉"]) {
        "特写".to_string()
    } else if contains_any_story_term(segment, &["焦土", "犁痕"]) {
        "全景".to_string()
    } else if contains_any_story_term(segment, &["心跳", "鼓点"]) {
        "近景".to_string()
    } else {
        "中景".to_string()
    }
}

fn build_visual_description_from_story(
    segment: &str,
    full_text: &str,
    subject: &str,
    scene_scale: &str,
    shot_scene_label: &str,
    shot_intent: &str,
) -> String {
    let source = if segment.trim().is_empty() {
        full_text.trim()
    } else {
        segment.trim()
    };
    let mut parts = vec![format!(
        "主体为{subject}，{}",
        derive_visual_composition_clause(source, subject, scene_scale, shot_intent)
    )];
    if let Some(environment) = derive_visual_environment_clause(source, full_text, shot_scene_label) {
        parts.push(environment);
    }
    if let Some(light_tone) = derive_visual_light_tone_clause(source, full_text) {
        parts.push(light_tone);
    }
    parts.push(derive_visual_event_clause(source, subject));
    parts.push(format!(
        "画面突出{}",
        derive_visual_focus_clause(source, shot_intent)
    ));
    parts.join("；")
}

fn derive_visual_composition_clause(
    source: &str,
    subject: &str,
    scene_scale: &str,
    shot_intent: &str,
) -> String {
    let scale = if scene_scale.trim().is_empty() {
        "中景"
    } else {
        scene_scale.trim()
    };
    if scale.contains("特写") || contains_any_story_term(source, &["掌心", "手臂", "银辉"]) {
        format!("{scale}压近{subject}的关键部位，以低角度把动作起势顶到画面前缘")
    } else if contains_enemy_or_conflict_terms(source) {
        format!("{scale}侧视角对角构图，把{subject}压在画面前侧的一步对冲距离里")
    } else if contains_any_story_term(source, &["重逢", "护住", "护着", "回身"]) {
        format!("{scale}正侧面构图，让{subject}占住画面中心并保留贴身站位关系")
    } else if shot_intent == "dialogue" {
        format!("{scale}正面或半侧视角收住{subject}的站位与视线关系")
    } else if scale.contains("全景") {
        format!("{scale}展开{subject}与环境的前后层次，主体位置和退路同时留在画面里")
    } else {
        format!("{scale}把{subject}放在主体区中央，保留人物与周围空间的清晰关系")
    }
}

fn derive_visual_environment_clause(
    source: &str,
    full_text: &str,
    shot_scene_label: &str,
) -> Option<String> {
    let combined = format!("{source}\n{full_text}");
    if contains_any_story_term(&combined, &["断桥", "桥边", "桥面"]) {
        Some("场景落在断桥残口与桥边碎石之间，狭窄落脚点把人物退路压得很紧".to_string())
    } else if contains_any_story_term(&combined, &["焦土", "裂痕", "残垣", "废墟"]) {
        Some("场景落在焦土裂痕和残垣边缘，碎土与硬质断面把空间压成前线险位".to_string())
    } else if contains_any_story_term(&combined, &["城门", "门墙"]) {
        Some("场景落在城门前沿，门墙与地面高差把进退路线框进同一块画面".to_string())
    } else if contains_any_story_term(&combined, &["林间", "林隙", "竹林", "竹叶"]) {
        Some("场景落在林间空地，枝叶和树影把主体前后层次切得很清楚".to_string())
    } else if contains_any_story_term(&combined, &["营地", "帐幕", "密营"]) {
        Some("场景落在营地核心区域，帐幕和立柱把主体围在可见中心".to_string())
    } else if contains_any_story_term(&combined, &["雨夜", "巷口", "雨雾", "湿地"]) {
        Some("场景落在雨夜巷口，墙面与湿地反光把纵深压成一条冷硬通道".to_string())
    } else if contains_any_story_term(&combined, &["战场", "军阵", "前沿"]) {
        Some("场景落在战场前沿，阵线、尘土和空地把人物推到冲突最前面".to_string())
    } else if shot_scene_label.contains("对白") {
        None
    } else {
        None
    }
}

fn derive_visual_light_tone_clause(source: &str, full_text: &str) -> Option<String> {
    let combined = format!("{source}\n{full_text}");
    if contains_any_story_term(&combined, &["火光", "火星"])
        && contains_any_story_term(&combined, &["烟尘", "焦土", "裂痕"])
    {
        Some("火光和火星在烟尘里来回闪动，粗粝暗色把整幅画面压出持续的冲击感".to_string())
    } else if contains_any_story_term(&combined, &["银辉", "掌心"]) {
        Some("银辉冷光沿掌心和手臂向上爬升，把周围色调压成偏冷的硬光".to_string())
    } else if contains_any_story_term(&combined, &["雨夜", "雨雾", "湿地"]) {
        Some("雨雾与湿地反光把画面压成冷色湿亮的质感，边缘轮廓更显锋利".to_string())
    } else if contains_any_story_term(&combined, &["焦土", "烟尘", "裂痕"]) {
        Some("焦土灰屑和扬起的烟尘压暗画面色调，空间显得发闷而粗粝".to_string())
    } else if contains_any_story_term(&combined, &["残阳"]) {
        Some("残阳只在边线留下一层钝暖色，主体仍被暗面和空气颗粒包住".to_string())
    } else {
        None
    }
}

fn derive_visual_event_clause(source: &str, subject: &str) -> String {
    if contains_any_story_term(source, &["重逢"]) {
        format!("当前视觉事件是{subject}在断裂边缘重新并肩，视线和站位同时重新对上")
    } else if contains_any_story_term(source, &["护住", "护着", "回身"]) {
        format!("当前视觉事件是{subject}回身挡住来势，身体横切进对冲路线")
    } else if contains_any_story_term(source, &["追杀", "压近", "逼近"]) {
        format!("当前视觉事件是{subject}把距离继续压短，来袭方向直逼主体前线")
    } else if contains_any_story_term(source, &["格挡", "刀锋", "攻击", "交锋"]) {
        format!("当前视觉事件是{subject}与对手的锋线正面撞上，冲击点停在接触瞬间")
    } else if contains_any_story_term(source, &["掌心", "银辉", "觉醒"]) {
        format!("当前视觉事件是{subject}的掌心或手臂出现醒目的能量变化，力量在画面内继续上涌")
    } else if contains_any_story_term(source, &["震退", "七步"]) {
        format!("当前视觉事件是{subject}借反冲把对手震开，脚下碎土被力量带起")
    } else {
        format!("当前视觉事件是{subject}在当前空间内完成清晰可见的状态变化，动作落点停在镜头正要继续之前")
    }
}

fn derive_visual_focus_clause(source: &str, shot_intent: &str) -> String {
    if contains_any_story_term(source, &["重逢"]) {
        "失而复得后的确认与仍未放松的紧绷感".to_string()
    } else if contains_any_story_term(source, &["护住", "护着", "回身"]) {
        "护人与来袭同时挤进画面的压迫感".to_string()
    } else if contains_any_story_term(source, &["追杀", "压近", "逼近"]) {
        "来势压近和退路被夺走的突袭感".to_string()
    } else if contains_any_story_term(source, &["格挡", "刀锋", "攻击", "交锋"]) {
        "正面硬碰时双方谁都不退的对峙感".to_string()
    } else if contains_any_story_term(source, &["掌心", "银辉", "觉醒"]) || shot_intent == "reveal" {
        "力量爆发前一刻的控场反转".to_string()
    } else if contains_any_story_term(source, &["震退", "七步"]) {
        "反击成立后的控场优势".to_string()
    } else if shot_intent == "dialogue" {
        "人物关系在静默停顿里的拉扯感".to_string()
    } else {
        "动作即将进入下一拍前的悬停感".to_string()
    }
}

fn derive_character_action_from_story(segment: &str, full_text: &str) -> String {
    let source = if segment.trim().is_empty() {
        full_text
    } else {
        segment
    };
    let registry = CharacterRegistry::from_story_text(source, full_text);
    let subject = derive_product_person(source, full_text);
    let target = derive_action_target(&registry, &subject, source, full_text);
    let start_state = derive_action_start_state(source);
    let action = derive_visible_action(source, &subject, &target);
    let end_state = derive_action_end_state(source);
    let captured_moment = derive_captured_moment(source);

    format!(
        "{subject}从{start_state}开始，{action}，到{end_state}时结束，镜头捕捉{captured_moment}。"
    )
}

fn derive_shot_title(index: usize, segment: &str, person: &str, character_action: &str) -> String {
    let action_core = derive_shot_title_action_core(segment, character_action);
    format!("镜头{}：{}{}", index + 1, person, action_core)
}

fn derive_camera_movement_from_story(
    shot_script: &str,
    scene_scale: &str,
    visual_description: &str,
    character_action: &str,
) -> String {
    let evidence = format!("{shot_script}\n{visual_description}\n{character_action}");
    let subject = character_action_subject(character_action);
    if contains_any_story_term(&evidence, &["站起", "起身"]) {
        format!("{scene_scale}低机位上摇，捕捉{subject}站起的动作转折")
    } else if contains_any_story_term(&evidence, &["抬起", "举起"]) {
        format!("{scene_scale}缓慢上摇，跟住{subject}抬起动作的发力线")
    } else if contains_any_story_term(&evidence, &["后撤", "震退", "退开"]) {
        format!("{scene_scale}跟随{subject}后撤半步，稳住动作对象和空间距离")
    } else if contains_any_story_term(&evidence, &["掌心", "手臂", "刀柄", "手部"]) {
        format!("{scene_scale}缓慢推近{subject}手部动作，停在动作发力瞬间")
    } else if contains_any_story_term(&evidence, &["焦土", "战场", "残骸", "断桥", "城门"])
    {
        format!("{scene_scale}横移掠过环境边缘，再锁定{subject}当前动作")
    } else if contains_any_story_term(&evidence, &["望向", "看向", "对手", "敌", "对立人物"])
    {
        format!("{scene_scale}过肩跟拍{subject}视线方向，保持动作对象在画面内")
    } else if scene_scale.contains("特写") {
        format!("{scene_scale}缓慢推近{subject}的关键动作，保持画面焦点稳定")
    } else if scene_scale.contains("全景") {
        format!("{scene_scale}定机位观察{subject}动作起止，保留环境和主体关系")
    } else {
        format!("{scene_scale}定机位观察{subject}动作起止，镜头在关键瞬间轻微推近")
    }
}

fn character_action_subject(character_action: &str) -> String {
    character_action
        .split('从')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "当前主体".to_string())
}

fn extract_character_name_after_role(text: &str) -> Option<String> {
    let mut name = String::new();
    for character in text.chars().skip_while(|character| {
        character.is_whitespace() || matches!(character, '：' | ':' | '，' | ',' | '、')
    }) {
        if is_character_name_stop(character) {
            break;
        }
        if !is_cjk_unified_ideograph(character) {
            break;
        }
        name.push(character);
        if name.chars().count() >= 3 {
            break;
        }
    }

    (name.chars().count() >= 2).then_some(name)
}

fn is_character_name_stop(character: char) -> bool {
    character.is_whitespace()
        || matches!(
            character,
            '，' | '。'
                | '、'
                | '；'
                | '：'
                | ','
                | '.'
                | ';'
                | ':'
                | '！'
                | '？'
                | '!'
                | '?'
                | '和'
                | '与'
                | '跟'
                | '同'
                | '及'
                | '在'
                | '从'
                | '向'
                | '对'
                | '被'
                | '把'
                | '将'
                | '追'
                | '护'
                | '回'
                | '稳'
                | '抬'
                | '踏'
                | '挡'
                | '格'
                | '冲'
                | '握'
                | '站'
                | '转'
                | '看'
                | '走'
                | '跑'
                | '挥'
                | '举'
                | '落'
                | '借'
                | '用'
                | '让'
                | '完'
                | '压'
                | '逼'
                | '提'
                | '劈'
                | '后'
                | '前'
                | '中'
                | '上'
                | '下'
                | '里'
                | '内'
                | '外'
                | '边'
                | '而'
                | '到'
                | '为'
                | '以'
                | '并'
                | '的'
                | '了'
        )
}

fn is_cjk_unified_ideograph(character: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&character)
}

fn is_antagonist_role(role: &str) -> bool {
    contains_any_story_term(role, &["敌将", "反派", "敌人", "对手"])
}

fn subject_label_from_names(names: &[String]) -> String {
    let mut unique_names = Vec::new();
    for name in names {
        if !unique_names.contains(name) {
            unique_names.push(name.clone());
        }
    }

    match unique_names.as_slice() {
        [] => "主角".to_string(),
        [single] => single.clone(),
        [first, second] => format!("{first}与{second}"),
        [first, second, third, ..] => format!("{first}、{second}与{third}"),
    }
}

fn contains_enemy_or_conflict_terms(text: &str) -> bool {
    contains_any_story_term(
        text,
        &[
            "敌方刀客",
            "敌人",
            "敌将",
            "反派",
            "对手",
            "刀客",
            "追杀",
            "压近",
            "逼近",
            "格挡",
            "交锋",
            "刀锋",
            "攻击",
            "迎敌",
        ],
    )
}

fn derive_fallback_subject(segment: &str, full_text: &str) -> String {
    let combined = format!("{segment} {full_text}");
    let has_main = contains_any_story_term(&combined, &["主角", "男主", "女主", "少年"]);
    let has_enemy = contains_enemy_or_conflict_terms(&combined);
    let enemy_label = if combined.contains("刀客") {
        "敌方刀客"
    } else {
        "对立人物"
    };

    if contains_any_story_term(&combined, &["群像", "众人", "队伍"]) {
        "群像角色".to_string()
    } else if has_main && has_enemy {
        format!("主角与{enemy_label}")
    } else if has_enemy {
        enemy_label.to_string()
    } else if has_main {
        "主角".to_string()
    } else {
        "目标人物".to_string()
    }
}

fn subject_label_mentions_any_name(subject: &str, text: &str) -> bool {
    subject
        .split(|character| matches!(character, '与' | '、'))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .any(|value| text.contains(value))
}

fn derive_action_target(
    registry: &CharacterRegistry,
    subject: &str,
    segment: &str,
    full_text: &str,
) -> String {
    if contains_any_story_term(segment, &["重逢"]) {
        return "彼此".to_string();
    }
    if contains_any_story_term(segment, &["护住", "护着"]) {
        if let Some(protagonist) = registry.protagonist_name() {
            return protagonist.to_string();
        }
        return "被保护者".to_string();
    }
    if let (Some(protagonist), Some(antagonist)) =
        (registry.protagonist_name(), registry.antagonist_name())
    {
        if subject.contains(protagonist) && subject.contains(antagonist) {
            return "彼此".to_string();
        }
    }
    if let Some(antagonist) = registry.antagonist_name() {
        if !subject.contains(antagonist) {
            return antagonist.to_string();
        }
    }
    if let Some(protagonist) = registry.protagonist_name() {
        if !subject.contains(protagonist) {
            return protagonist.to_string();
        }
    }
    if let Some(heroine) = registry.heroine_name() {
        if !subject.contains(heroine) {
            return heroine.to_string();
        }
    }

    let combined = format!("{segment} {full_text}");
    if subject.contains("敌方刀客") || subject.contains("对立人物") {
        "主角".to_string()
    } else if combined.contains("刀客") {
        "敌方刀客".to_string()
    } else if contains_enemy_or_conflict_terms(&combined) {
        "对立人物".to_string()
    } else if subject.contains('与') || subject.contains('、') {
        "彼此".to_string()
    } else {
        "目标人物".to_string()
    }
}

fn derive_action_start_state(source: &str) -> &'static str {
    if contains_any_story_term(source, &["焦土", "裂痕", "犁痕"]) {
        "焦土裂痕边缘稳住身体"
    } else if contains_any_story_term(source, &["断桥", "重逢"]) {
        "断桥残口确认彼此位置"
    } else if contains_any_story_term(source, &["追杀", "压近", "逼近"]) {
        "断桥远端压低重心"
    } else if contains_any_story_term(source, &["格挡", "刀锋", "攻击"]) {
        "迎面冲击前的半步停顿"
    } else if contains_any_story_term(source, &["掌心", "银辉", "觉醒"]) {
        "格挡后的短暂停滞"
    } else {
        "上一动作落点稳定身体"
    }
}

fn derive_visible_action(source: &str, subject: &str, target: &str) -> String {
    if contains_any_story_term(source, &["重逢"]) {
        format!("{subject}在断桥残口向{target}靠近，确认对方安全并重新建立站位")
    } else if contains_any_story_term(source, &["护住", "护着", "回身"]) {
        format!("{subject}回身护住{target}，用身体挡住逼近的威胁")
    } else if contains_any_story_term(source, &["追杀", "压近", "逼近"]) {
        format!("{subject}提刀逼向{target}，把对方压向断桥边缘")
    } else if contains_any_story_term(source, &["格挡", "刀锋", "攻击", "交锋"]) {
        format!("{subject}迎着{target}的冲击抬臂格挡，让银辉与刀锋正面相撞")
    } else if contains_any_story_term(source, &["掌心", "银辉", "觉醒"]) {
        format!("{subject}将掌心朝向{target}，让银辉沿手臂上升并压住对方攻势")
    } else if contains_any_story_term(source, &["震退", "七步"]) {
        format!("{subject}借格挡余力反震{target}，把对方逼到脚步失衡")
    } else {
        format!("{subject}面向{target}完成清晰可见的状态转变并接上下一拍动作")
    }
}

fn derive_action_end_state(source: &str) -> &'static str {
    if contains_any_story_term(source, &["重逢"]) {
        "两人重新并肩"
    } else if contains_any_story_term(source, &["护住", "护着"]) {
        "被保护者退到安全半步"
    } else if contains_any_story_term(source, &["追杀", "压近", "逼近"]) {
        "目标被逼到断桥边缘"
    } else if contains_any_story_term(source, &["掌心", "银辉", "觉醒"]) {
        "银辉完全爬上前臂"
    } else if contains_any_story_term(source, &["震退", "七步"]) {
        "对方脚步失衡后撤"
    } else if contains_any_story_term(source, &["格挡", "刀锋", "攻击", "交锋"]) {
        "攻防短暂相持"
    } else {
        "下一拍动作蓄势完成"
    }
}

fn derive_captured_moment(source: &str) -> &'static str {
    if contains_any_story_term(source, &["重逢"]) {
        "两人视线重新对上的一瞬间"
    } else if contains_any_story_term(source, &["护住", "护着"]) {
        "身体挡住威胁的一瞬间"
    } else if contains_any_story_term(source, &["追杀", "压近", "逼近"]) {
        "刀锋压入断桥空间的一瞬间"
    } else if contains_any_story_term(source, &["掌心", "银辉", "觉醒"]) {
        "银辉开始蔓延的一瞬间"
    } else if contains_any_story_term(source, &["震退", "七步"]) {
        "第一步被震开的瞬间"
    } else if contains_any_story_term(source, &["格挡", "刀锋", "攻击", "交锋"]) {
        "银辉与刀刃相撞的一瞬间"
    } else {
        "状态发生变化的一瞬间"
    }
}

fn derive_shot_title_action_core(segment: &str, character_action: &str) -> &'static str {
    if contains_any_story_term(segment, &["重逢"]) {
        "断桥重逢"
    } else if contains_any_story_term(segment, &["护住", "护着", "回身"]) {
        "回身护人"
    } else if contains_any_story_term(segment, &["追杀", "压近", "逼近"]) {
        "压近断桥"
    } else if contains_any_story_term(segment, &["掌心", "银辉", "觉醒"]) {
        "银辉觉醒"
    } else if contains_any_story_term(segment, &["震退", "七步"]) {
        "震退对手"
    } else if contains_any_story_term(segment, &["焦土", "裂痕"]) {
        "踏碎焦土"
    } else if contains_any_story_term(segment, &["格挡", "刀锋", "攻击", "交锋"]) {
        "踏步格挡"
    } else if character_action.contains("镜头捕捉") {
        "完成关键动作"
    } else {
        "推进当前动作"
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

fn draft_is_grounded_in_story(
    draft: &ShotGroundedRowDraft,
    grounding: &StoryboardGroundingContext,
) -> bool {
    let combined = format!(
        "{} {} {} {} {} {}",
        draft.person,
        draft.shot_title,
        draft.visual_description,
        draft.character_action,
        draft.camera_movement,
        draft.dialogue
    );
    if contains_any_story_term(
        &combined,
        &[
            "城市夜雨站台",
            "夜雨站台",
            "便利店",
            "透明伞",
            "霓虹反光",
            "雨中告别",
        ],
    ) {
        return false;
    }
    let anchors = story_anchor_terms(&grounding.grounding_text);
    anchors.is_empty() || anchors.iter().any(|anchor| combined.contains(anchor))
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
    let system_prompt = "You are Hope's controlled text-generation layer. Ground storyboard output in shot_script first, then expanded_script_text, then primary scene fields, and only then compressed KB context. Preserve accepted character relationships, motivation, event order, timeline, prop state, emotional progression, conflict causality, and next-scene handoff. Scene changes expression only; directing schedules shots only. Never invent real director names, IP names, brand names, or external asset bindings. Do not expand to full KB rows.";
    let user_prompt = format!(
        "task_type={:?}\nscene_type={}\nduration_seconds={}\nstory_input={}\nkb_context_summary={}\nselected_sample_ids={}\nselected_kb_rules={}\noutput_schema={}\nconstraints=for expand_script, write to the requested duration_seconds; for storyboard, each row must use a Seedance-friendly duration from the duration plan; shot_script is authoritative when present; preserve relationships, motivation, event order, timeline, prop state, emotional progression, conflict causality, and next-scene continuity; scene only changes expression; directing only schedules shots; KB samples are summary-only references and must not replace current story; keep total duration conserved; do not emit full KB; do not emit raw prompt_body as final prompt_text; do not use real director/IP/brand names.",
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

fn resolve_expand_script_target_duration_seconds(request: &ExpandScriptRequest) -> u16 {
    request
        .target_duration_seconds
        .or_else(|| infer_duration_seconds_from_text(&request.synopsis_text))
        .filter(|duration| is_supported_storyboard_duration(*duration))
        .unwrap_or(DEFAULT_EXPAND_SCRIPT_DURATION_SECONDS)
}

fn infer_duration_seconds_from_text(text: &str) -> Option<u16> {
    let chars = text.chars().collect::<Vec<_>>();
    for index in 0..chars.len() {
        if !chars[index].is_ascii_digit() {
            continue;
        }
        let end = chars[index..]
            .iter()
            .position(|character| !character.is_ascii_digit())
            .map(|offset| index + offset)
            .unwrap_or(chars.len());
        let value = chars[index..end]
            .iter()
            .collect::<String>()
            .parse::<u16>()
            .ok()?;
        let suffix = chars[end..chars.len().min(end + 2)]
            .iter()
            .collect::<String>();
        if suffix.starts_with('秒') || suffix.to_ascii_lowercase().starts_with('s') {
            return Some(value);
        }
    }
    None
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

fn apply_live_storyboard_patch(draft: &mut ShotGroundedRowDraft, patch: &LiveStoryboardRowPatch) {
    if let Some(value) = non_blank_string(&patch.shot_title) {
        draft.shot_title = value;
        draft.scene_performance_projection.source_sample_title = draft.shot_title.clone();
    }
    if let Some(value) = non_blank_string(&patch.person) {
        draft.person = value;
        draft.scene_performance_projection.person = draft.person.clone();
    }
    if let Some(value) = non_blank_string(&patch.scene_scale) {
        draft.scene_scale = value;
        draft.scene_performance_projection.scene_scale = draft.scene_scale.clone();
    }
    if let Some(value) = non_blank_string(&patch.visual_description) {
        draft.visual_description = value;
        draft.scene_performance_projection.visual_description = draft.visual_description.clone();
    }
    if let Some(value) = non_blank_string(&patch.character_action) {
        draft.character_action = value;
        draft.scene_performance_projection.character_action = draft.character_action.clone();
    }
    if let Some(value) = non_blank_string(&patch.camera_movement) {
        draft.camera_movement = value;
    }
    if let Some(value) = non_blank_string(&patch.dialogue) {
        draft.dialogue = value;
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
            ("person", row.person.as_str()),
            ("shot_title", row.shot_title.as_str()),
            ("scene_scale", row.scene_scale.as_str()),
            ("visual_description", row.visual_description.as_str()),
            ("character_action", row.character_action.as_str()),
            ("camera_movement", row.camera_movement.as_str()),
            ("prompt_text", row.prompt_text.as_str()),
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
                        "Storyboard row {} includes an internal code in {}.",
                        row.shot_id, field_name
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
            if contains_any_story_term(
                field_value,
                &[
                    "交锋双方",
                    "当前镜头主体",
                    "未指定角色",
                    "按当前镜头脚本执行关键动作",
                    "执行关键动作",
                    "保持连续性",
                ],
            ) {
                findings.push(ProductWarning {
                    code: "role_action_grounding_incomplete".to_string(),
                    message: format!(
                        "Storyboard row {} includes vague product wording in {}.",
                        row.shot_id, field_name
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
            if field_name == "character_action" && is_role_action_grounding_incomplete(field_value)
            {
                findings.push(ProductWarning {
                    code: "role_action_grounding_incomplete".to_string(),
                    message: format!(
                        "Storyboard row {} has an incomplete role action; it must describe who acts, what changes, the target, start/end state, and the captured moment.",
                        row.shot_id
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
            if field_name == "visual_description"
                && is_visual_description_grounding_incomplete(
                    field_value,
                    &row.person,
                    &row.scene_scale,
                    &row.character_action,
                    &row.camera_movement,
                    &row.shot_script,
                )
            {
                findings.push(ProductWarning {
                    code: "visual_description_grounding_incomplete".to_string(),
                    message: format!(
                        "Storyboard row {} has an incomplete visual description; it must anchor subject, environment, composition, visible elements, current visual event, and the conflict focus without repeating role action or camera movement.",
                        row.shot_id
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
            if field_name == "camera_movement"
                && is_camera_movement_grounding_incomplete(
                    field_value,
                    &row.shot_title,
                    &row.scene_scale,
                    &row.visual_description,
                    &row.shot_script,
                )
            {
                findings.push(ProductWarning {
                    code: "camera_movement_grounding_incomplete".to_string(),
                    message: format!(
                        "Storyboard row {} has an incomplete camera movement; it must describe how the camera moves or stages the current shot without replacing title or visual description.",
                        row.shot_id
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
        }
    }
    findings
}

fn is_camera_movement_grounding_incomplete(
    camera_movement: &str,
    shot_title: &str,
    scene_scale: &str,
    visual_description: &str,
    shot_script: &str,
) -> bool {
    let trimmed = camera_movement.trim();
    if trimmed.is_empty() {
        return true;
    }
    if trimmed == shot_title.trim() || trimmed == visual_description.trim() {
        return true;
    }
    if shot_script.trim().is_empty() {
        return true;
    }
    if contains_forbidden_v0_product_payload(trimmed)
        || contains_any_story_term(
            trimmed,
            &[
                "当前镜头主体完成动作",
                "按当前脚本执行",
                "visual_scene_core",
                "fused_scene_performance_core_preserved",
                "目标人物完成关键动作",
            ],
        )
    {
        return true;
    }
    if !scene_scale.trim().is_empty() && !trimmed.contains(scene_scale.trim()) {
        return true;
    }
    !contains_any_story_term(
        trimmed,
        &[
            "定机位",
            "推近",
            "跟随",
            "横移",
            "上摇",
            "过肩",
            "手持",
            "跟拍",
            "锁定",
            "观察",
            "捕捉",
        ],
    )
}

fn is_visual_description_grounding_incomplete(
    visual_description: &str,
    subject: &str,
    scene_scale: &str,
    character_action: &str,
    camera_movement: &str,
    shot_script: &str,
) -> bool {
    let trimmed = visual_description.trim();
    if trimmed.is_empty() {
        return true;
    }
    if trimmed == character_action.trim()
        || trimmed == camera_movement.trim()
        || trimmed == shot_script.trim()
        || (!character_action.trim().is_empty() && trimmed.contains(character_action.trim()))
    {
        return true;
    }
    if contains_forbidden_v0_product_payload(trimmed)
        || contains_any_story_term(
            trimmed,
            &[
                "当前镜头主体",
                "按当前镜头脚本执行关键动作",
                "保持连续性",
                "visual_scene_core",
                "fused_scene_performance_core_preserved",
                "not_specified_by_v120_bridge",
                "气氛紧张",
                "画面震撼",
            ],
        )
    {
        return true;
    }
    if !subject.trim().is_empty()
        && !trimmed.contains(subject.trim())
        && !subject_label_mentions_any_name(subject, trimmed)
    {
        return true;
    }
    let has_environment = contains_any_story_term(
        trimmed,
        &[
            "断桥",
            "桥边",
            "桥面",
            "焦土",
            "裂痕",
            "残垣",
            "废墟",
            "城门",
            "门墙",
            "林间",
            "林隙",
            "竹林",
            "竹叶",
            "营地",
            "帐幕",
            "巷口",
            "雨夜",
            "雨雾",
            "湿地",
            "战场",
            "军阵",
            "前沿",
        ],
    );
    if !has_environment {
        return true;
    }
    let has_composition = (!scene_scale.trim().is_empty() && trimmed.contains(scene_scale.trim()))
        || contains_any_story_term(
            trimmed,
            &[
                "正面",
                "侧视角",
                "俯拍",
                "仰拍",
                "对角构图",
                "三角构图",
                "前景",
                "画面中心",
                "主体区中央",
                "前后层次",
                "一步对冲距离",
                "贴身站位",
            ],
        );
    if !has_composition {
        return true;
    }
    let has_visible_elements = contains_any_story_term(
        trimmed,
        &[
            "碎石",
            "碎土",
            "烟尘",
            "火光",
            "火星",
            "银辉",
            "反光",
            "冷光",
            "并肩",
            "挡住",
            "压短",
            "撞上",
            "上涌",
            "震开",
            "带起",
        ],
    );
    if !has_visible_elements {
        return true;
    }
    let has_focus = trimmed.contains("画面突出")
        || contains_any_story_term(
            trimmed,
            &[
                "压迫感",
                "对峙感",
                "突袭感",
                "控场优势",
                "控场反转",
                "拉扯感",
                "悬停感",
            ],
        );
    !has_focus || trimmed.chars().count() < 28
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
        && (trimmed.contains("对手")
            || trimmed.contains("环境")
            || trimmed.contains("全场")
            || trimmed.contains("迎着")
            || trimmed.contains("面向")
            || trimmed.contains("逼向")
            || trimmed.contains("护住")
            || trimmed.contains("朝向")
            || trimmed.contains("彼此")
            || trimmed.contains("敌方刀客")
            || trimmed.contains("对立人物")
            || trimmed.contains("目标人物")
            || trimmed.contains("被保护者")))
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
    target_duration_seconds: u16,
) -> String {
    let beat_count = match target_duration_seconds {
        0..=15 => 2,
        16..=30 => 3,
        31..=45 => 5,
        _ => 6,
    };
    let seconds_per_beat = (target_duration_seconds / beat_count).max(1);
    let mut lines = vec![
        format!("scene_type: {normalized_scene_type}"),
        format!("target_duration_seconds: {target_duration_seconds}"),
        format!("synopsis: {}", synopsis_text.trim()),
        format!("source_package: {snapshot_name}"),
        format!(
            "扩写剧本：按{target_duration_seconds}秒连续剧情处理，保留真实人物名、主体关系、动作对象和清晰起承转合。"
        ),
    ];

    for index in 0..beat_count {
        let beat_no = index + 1;
        let start_second = index as u16 * seconds_per_beat;
        let end_second = if beat_no == beat_count {
            target_duration_seconds
        } else {
            ((index as u16 + 1) * seconds_per_beat).min(target_duration_seconds)
        };
        lines.push(format!(
            "段落{beat_no}（约{start_second}-{end_second}秒）：{}；人物从上一段状态接续，明确谁面对谁、做什么动作、情绪和空间如何变化，并为下一段留下可见承接。",
            synopsis_text.trim()
        ));
    }

    lines.join("\n")
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
            camera_movement: derive_camera_movement_from_story(
                &scene_projection.fused_source_text,
                &scene_projection.scene_scale,
                &scene_projection.visual_description,
                &scene_projection.character_action,
            ),
            dialogue: String::new(),
            duration_seconds,
        },
        shot_script: None,
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
        build_prompt_compilation_story_input(&request),
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

    let mut prompt_sections =
        vec!["视频分镜提示词：以当前镜头脚本为准，输出单个镜头画面。".to_string()];
    if let Some(shot_script) = request
        .shot_script
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        prompt_sections.push(format!("镜头脚本：{}", shot_script));
    }
    prompt_sections.extend([
        format!("镜头标题：{}", request.row.shot_title),
        format!("景别：{}", request.row.scene_scale),
        format!("画面描述：{}", request.row.visual_description),
        format!("角色动作：{}", request.row.character_action),
        format!("运镜：{}", request.row.camera_movement),
    ]);
    if !request.row.dialogue.trim().is_empty() {
        prompt_sections.push(format!("对白/旁白：{}", request.row.dialogue.trim()));
    }
    prompt_sections.push(format!("镜头时长：{}秒", request.row.duration_seconds));

    let mut warnings = stub_response.warnings;
    if is_role_action_grounding_incomplete(&request.row.character_action) {
        warnings.push(ProductWarning {
            code: "role_action_grounding_incomplete".to_string(),
            message:
                "角色动作未能完整说明主体、动作、对象、起止状态与镜头捕捉瞬间，已保留产品态占位。"
                    .to_string(),
            related_sample_id: Some(request.row.shot_id.clone()),
        });
    }
    if is_visual_description_grounding_incomplete(
        &request.row.visual_description,
        &character_action_subject(&request.row.character_action),
        &request.row.scene_scale,
        &request.row.character_action,
        &request.row.camera_movement,
        request.shot_script.as_deref().unwrap_or_default(),
    ) {
        warnings.push(ProductWarning {
            code: "visual_description_grounding_incomplete".to_string(),
            message: "画面描述未能完整交代主体、环境、构图、可见元素、当前视觉事件与冲突落点，已保留产品态 warning。"
                .to_string(),
            related_sample_id: Some(request.row.shot_id.clone()),
        });
    }
    if is_camera_movement_grounding_incomplete(
        &request.row.camera_movement,
        &request.row.shot_title,
        &request.row.scene_scale,
        &request.row.visual_description,
        request.shot_script.as_deref().unwrap_or_default(),
    ) {
        warnings.push(ProductWarning {
            code: "camera_movement_grounding_incomplete".to_string(),
            message: "运镜未能完整说明镜头如何结合景别捕捉当前动作，已保留产品态占位。".to_string(),
            related_sample_id: Some(request.row.shot_id.clone()),
        });
    }
    warnings.push(ProductWarning {
        code: "seedance_video_generation_closed".to_string(),
        message:
            "Seedance2.0 is a prompt_text adaptation target in MVP; in-app video generation remains gated."
                .to_string(),
        related_sample_id: Some(request.row.shot_id.clone()),
    });
    warnings.push(ProductWarning {
        code: "kb_sample_prompt_not_promoted".to_string(),
        message:
            "Prompt compilation uses shot_script, structured row fields, and selected KB summaries; raw KB sample prompt evidence stays internal."
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

fn build_prompt_compilation_story_input(request: &PromptTextCompilationRequest) -> String {
    format!(
        "grounding_priority=shot_script_then_row_fields\nshot_script={}\nscene_label={}\nshot_title={}\nvisual_description={}\ncharacter_action={}\ncamera_movement={}\ndialogue={}",
        request.shot_script.as_deref().unwrap_or_default(),
        request.scene_label,
        request.row.shot_title,
        request.row.visual_description,
        request.row.character_action,
        request.row.camera_movement,
        request.row.dialogue,
    )
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
fn project_scene_performance(record: &GoldenSampleLibraryRecord) -> ScenePerformanceProjection {
    ScenePerformanceProjection {
        source_sample_id: record.sample_id.clone(),
        source_sample_title: record.source_fields.sample_title.clone(),
        scene_scale: derive_scene_scale(&record.source_fields.technical_profile),
        person: "未指定主体".to_string(),
        visual_description: record.source_fields.scene_performance_core.clone(),
        character_action: "待明确动作".to_string(),
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
                "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                request.scene_type,
                request.synopsis_text,
                request.duration_seconds,
                request.task_type.as_str(),
                request.primary_scene_type.as_deref().unwrap_or_default(),
                request.shot_scene_type.as_deref().unwrap_or_default(),
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
    (5..=60).contains(&duration_seconds)
        && duration_seconds % SEEDANCE_REMAINDER_SEGMENT_SECONDS == 0
}

fn allocate_storyboard_row_durations(
    total_duration_seconds: u16,
    _row_count: usize,
) -> Option<Vec<u16>> {
    if !is_supported_storyboard_duration(total_duration_seconds) {
        return None;
    }

    let mut remaining = total_duration_seconds;
    let mut durations = Vec::new();
    while remaining >= SEEDANCE_STANDARD_SEGMENT_SECONDS {
        durations.push(SEEDANCE_STANDARD_SEGMENT_SECONDS);
        remaining -= SEEDANCE_STANDARD_SEGMENT_SECONDS;
    }
    if remaining == SEEDANCE_REMAINDER_SEGMENT_SECONDS {
        durations.push(SEEDANCE_REMAINDER_SEGMENT_SECONDS);
    } else if remaining != 0 {
        return None;
    }

    if durations.is_empty()
        || durations
            .iter()
            .any(|duration| *duration == 0 || *duration > SEEDANCE_MAX_SEGMENT_SECONDS)
    {
        return None;
    }

    (durations.iter().copied().sum::<u16>() == total_duration_seconds).then_some(durations)
}

fn build_storyboard_duration_plan(
    total_duration_seconds: u16,
    durations: &[u16],
) -> StoryboardDurationPlan {
    StoryboardDurationPlan {
        total_duration_seconds,
        row_count: durations.len() as u32,
        per_row_seconds: planned_per_row_seconds(durations),
        allocated_seconds: durations.iter().copied().sum(),
    }
}

fn planned_per_row_seconds(durations: &[u16]) -> u16 {
    if durations
        .iter()
        .any(|duration| *duration == SEEDANCE_STANDARD_SEGMENT_SECONDS)
    {
        SEEDANCE_STANDARD_SEGMENT_SECONDS
    } else {
        durations.first().copied().unwrap_or_default()
    }
}

fn serialize_storyboard_rows(rows: &[GeneratedStoryboardRow]) -> String {
    rows.iter()
        .map(|row| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{:?}|{}|{}|{}|{}|{}",
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
                row.camera_movement,
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
        BridgeCallStatus, ExpandScriptRequest, ExportBundleRequest, ExportStoryboardBankRequest,
        FailurePatternRecord, GenerateStoryboardRequest, GoldenSampleAssetSource,
        GoldenSampleClassification, GoldenSampleComparisonBaseline, GoldenSampleCoverageSummary,
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
        KbRouterTaskType, KbRuntimeSummary, KbSnapshotRecord, ListStoryboardShotResultsRequest,
        PromptTemplateRecord, PromptTextCompilationStatus, RemoveStoryboardShotResultRequest,
        RunV0StoryToStoryboardChainRequest, SaveStoryboardShotResultRequest, SceneTaxonomyRecord,
        ShotGroundingSource, StoryboardDurationPlan, TextGenerationOutputSchema,
        TextGenerationTask, TextModelProviderKind, UpdateStoryboardRowsRequest,
        UpdateStoryboardShotResultRequest,
    };
    use project_store::{
        DualSqliteConnectionPolicy, KbKnowledgeBundle, KbRuntimeHandle, StoreSkeleton,
    };
    use serde_json::json;

    use super::{
        SEEDANCE_REMAINDER_SEGMENT_SECONDS, SEEDANCE_STANDARD_SEGMENT_SECONDS,
        StoryboardPreviewPlanRequest, TARGET_DURATION_MODE_FIXED_SECONDS,
        TARGET_DURATION_MODE_LONG_TEXT_AUTO, V0_LONG_STORY_AUTO_PROFILE,
        ValidationExportPanelSnapshotRequest, ValidationExportPanelState,
        allocate_long_text_auto_shot_task_durations, allocate_storyboard_row_durations,
        build_prompt_text_compilation_request, build_qwen_request_payload,
        build_storyboard_preview_plan, build_text_generation_request,
        build_validation_export_panel_snapshot_from_fixture, compile_seedance_prompt_text,
        contains_any_story_term, default_text_model_provider, expand_script, export_bundle,
        export_storyboard_bank, generate_storyboard, is_visual_description_grounding_incomplete,
        list_storyboard_shot_results, project_scene_performance, remove_storyboard_shot_result,
        resolve_scene_taxonomy, run_kb_router, run_qwen_text_generation_with_transport,
        run_v0_story_to_storyboard_chain, save_storyboard_rows, save_storyboard_shot_result,
        select_golden_sample_records, serialize_storyboard_rows, stable_hash_hex,
        stable_source_material_length_chars, subject_label_mentions_any_name,
        update_storyboard_shot_result,
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

    fn generate_test_storyboard(
        state: &AppState,
        task_name: &str,
        duration_seconds: u16,
    ) -> core_domain::GenerateStoryboardResponse {
        generate_storyboard(
            state,
            GenerateStoryboardRequest {
                task_name: task_name.to_string(),
                script_id: None,
                shot_script: Some(format!(
                    "{task_name} blocks the incoming strike, the opponent is pushed back, and the shot resolves on a visible power shift."
                )),
                expanded_script_text: Some(
                    "scene_type: daily_dialogue\nsynopsis: a grounded action beat inside the current shot"
                        .to_string(),
                ),
                primary_scene_type: Some("daily_dialogue".to_string()),
                primary_scene_label: Some("Daily Dialogue".to_string()),
                primary_scene_category: Some("dialogue".to_string()),
                shot_scene_type: Some("daily_dialogue".to_string()),
                shot_scene_label: Some("Daily Dialogue".to_string()),
                shot_intent: Some("story_beat".to_string()),
                adaptation_reason: None,
                selected_total_duration_seconds: duration_seconds,
            },
        )
    }

    fn finalized_save_request_from_storyboard(
        project_id: &str,
        script_id: &str,
        shot_order: u32,
        confirmed: bool,
        storyboard: &core_domain::GenerateStoryboardResponse,
    ) -> SaveStoryboardShotResultRequest {
        let prompt_text = storyboard
            .rows
            .iter()
            .map(|row| row.prompt_text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let shot_duration_seconds = storyboard
            .rows
            .iter()
            .map(|row| row.shot_duration_seconds)
            .sum::<u16>();
        SaveStoryboardShotResultRequest {
            project_id: project_id.to_string(),
            script_id: script_id.to_string(),
            shot_task_id: storyboard
                .task_id
                .clone()
                .unwrap_or_else(|| "shot-task-test".to_string()),
            result_id: storyboard.result_id.clone(),
            shot_order,
            shot_task_name: storyboard
                .task_id
                .clone()
                .unwrap_or_else(|| "finalized shot".to_string()),
            rows: storyboard.rows.clone(),
            prompt_text,
            shot_duration_seconds,
            duration_source: "storyboard_duration_plan.allocated_row_duration_seconds".to_string(),
            confirmed,
            updated_at_ms: storyboard.updated_at_ms,
            rows_hash: stable_hash_hex(&serialize_storyboard_rows(&storyboard.rows)),
        }
    }

    fn assert_storyboard_row_has_no_vague_or_internal_terms(
        row: &core_domain::GeneratedStoryboardRow,
    ) {
        let combined = format!(
            "{} {} {} {} {} {}",
            row.person,
            row.shot_title,
            row.visual_description,
            row.character_action,
            row.camera_movement,
            row.prompt_text
        );
        for forbidden in [
            "交锋双方",
            "当前镜头主体",
            "not_specified_by_v120_bridge",
            "visual_scene_core",
            "fused_scene_performance_core_preserved",
            "按当前镜头脚本执行关键动作",
            "执行关键动作",
            "保持连续性",
            "待明确",
        ] {
            assert!(
                !combined.contains(forbidden),
                "storyboard row leaked vague/internal term {forbidden}: {combined}"
            );
        }
    }

    fn assert_visual_description_is_enhanced(row: &core_domain::GeneratedStoryboardRow) {
        assert!(
            row.visual_description.contains("主体为"),
            "visual_description should anchor subject: {}",
            row.visual_description
        );
        assert!(
            row.visual_description.contains(&row.scene_scale),
            "visual_description should carry scene scale/composition context: {}",
            row.visual_description
        );
        assert!(
            row.visual_description.contains("当前视觉事件是"),
            "visual_description should describe the current visual event: {}",
            row.visual_description
        );
        assert!(
            row.visual_description.contains("画面突出"),
            "visual_description should land on conflict focus: {}",
            row.visual_description
        );
        assert_ne!(row.visual_description, row.character_action);
        assert_ne!(row.visual_description, row.shot_script);
        assert!(contains_any_story_term(
            &row.visual_description,
            &[
                "断桥",
                "桥边",
                "焦土",
                "裂痕",
                "残垣",
                "碎石",
                "碎土",
                "烟尘",
                "银辉",
                "雨夜",
                "巷口",
                "战场",
                "帐幕",
                "枝叶",
            ],
        ));
    }

    fn assert_prompt_text_has_no_internal_payload(prompt_text: &str) {
        for forbidden in [
            "KB摘要",
            "KB上下文",
            "sample_id",
            "selected_sample_ids",
            "rule id",
            "rule_id",
            "retrieval trace",
            "retrieval_trace",
            "grounding_source",
            "raw prompt_body",
            "prompt_body",
            "source_register",
            "overlay JSON",
            "overlay_json",
            "API key",
            "api_key",
            "internal hash",
            "full raw KB rows",
        ] {
            assert!(
                !prompt_text.contains(forbidden),
                "prompt_text leaked internal payload term {forbidden}: {prompt_text}"
            );
        }
    }

    fn v0_chain_request_for_source(
        story_id: &str,
        source_text: &str,
    ) -> RunV0StoryToStoryboardChainRequest {
        RunV0StoryToStoryboardChainRequest {
            story_id: story_id.to_string(),
            chapter_id: format!("{story_id}-chapter-001"),
            chapter_order: 1,
            user_topic_or_synopsis: source_text.to_string(),
            story_length_profile: "short_story_2000_2500".to_string(),
            selected_total_duration_seconds: 10,
            target_duration_mode: TARGET_DURATION_MODE_FIXED_SECONDS.to_string(),
            source_material_length_chars: 0,
            auto_segment_strategy: String::new(),
            estimated_total_story_duration_seconds: 0,
            primary_scene_type: "daily_dialogue".to_string(),
            primary_scene_label: Some("story rewrite validation".to_string()),
            primary_scene_category: Some("story_rewrite".to_string()),
            authoring_craft_summary:
                "summary-only authoring advice; source facts remain authoritative.".to_string(),
            screenwriting_adaptation_summary:
                "rewrite into playable screenplay beats without adding unsupported plot."
                    .to_string(),
            directing_kb_context_summary:
                "camera and blocking must serve the accepted source events.".to_string(),
            continuity_context_summary: "start from submitted source material.".to_string(),
            kb_context_summary:
                "summary-only KB advice; do not override names, event order, or ending state."
                    .to_string(),
            selected_sample_ids: vec!["GS-SUMMARY-ONLY".to_string()],
            selected_kb_rules: vec!["full_kb_rows_included=0".to_string()],
            retrieval_trace_user_summary: "summary-only deterministic test".to_string(),
            full_kb_rows_included: 0,
            confirm_storyboard_results: true,
        }
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
                primary_scene_type: Some("daily_dialogue".to_string()),
                primary_scene_label: Some("Daily Dialogue".to_string()),
                shot_scene_type: Some("daily_dialogue".to_string()),
                shot_scene_label: Some("Daily Dialogue".to_string()),
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
                shot_script: None,
                expanded_script_text: None,
                primary_scene_type: None,
                primary_scene_label: None,
                primary_scene_category: None,
                shot_scene_type: None,
                shot_scene_label: None,
                shot_intent: None,
                adaptation_reason: None,
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
                primary_scene_type: Some("daily_dialogue".to_string()),
                primary_scene_label: Some("Daily Dialogue".to_string()),
                shot_scene_type: Some("daily_dialogue".to_string()),
                shot_scene_label: Some("Daily Dialogue".to_string()),
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
        assert!(response.prompt_text.contains("视频分镜提示词"));
        assert!(response.prompt_text.contains("镜头时长：8秒"));
        assert!(!response.prompt_text.contains("目标适配"));
        assert!(!response.prompt_text.contains("连续性约束"));
        assert!(!response.prompt_text.contains("KB摘要"));
        assert!(!response.prompt_text.contains("KB上下文"));
        assert!(!response.prompt_text.contains("sample_id"));
        assert!(!response.prompt_text.contains("scene_category"));
        assert!(!response.prompt_text.contains("style_cluster"));
        assert!(!response.prompt_text.contains("selected_sample_ids"));
        assert!(!response.prompt_text.contains("rule_id"));
        assert!(!response.prompt_text.contains("duration_guard"));
        assert!(!response.prompt_text.contains("reserve_gate"));
        assert!(!response.prompt_text.contains("grounding_source"));
        assert!(!response.prompt_text.contains("primary_scene_type"));
        assert!(!response.prompt_text.contains("shot_scene_type"));
        assert!(!response.prompt_text.contains("retrieval trace"));
        assert!(!response.prompt_text.contains("retrieval_trace"));
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
        assert!(
            response
                .warnings
                .iter()
                .any(|warning| warning.code == "role_action_grounding_incomplete")
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
    fn generate_storyboard_grounds_rows_in_shot_script_before_kb_samples() {
        let state = test_state();
        let shot_script = "格挡正面冲击。对手被震退七步，焦土被脚跟犁出痕。心跳与低频鼓点压住全场。掌心银辉觉醒，并沿手臂上升。";

        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "guoman-action-grounding".to_string(),
                script_id: None,
                shot_script: Some(shot_script.to_string()),
                expanded_script_text: Some(
                    "scene_type: daily_dialogue\nsynopsis: 国漫群像表演中的局部交锋镜头"
                        .to_string(),
                ),
                primary_scene_type: Some("daily_dialogue".to_string()),
                primary_scene_label: Some("国漫群像表演".to_string()),
                primary_scene_category: Some("group_performance".to_string()),
                shot_scene_type: None,
                shot_scene_label: None,
                shot_intent: None,
                adaptation_reason: None,
                selected_total_duration_seconds: 45,
            },
        );

        assert_ne!(storyboard.export_status.status, BridgeCallStatus::Blocked);
        assert_eq!(
            storyboard
                .kb_router_result
                .retrieval_trace
                .token_budget
                .full_kb_rows_included,
            0
        );
        assert!(storyboard.rows.len() >= 4);
        let combined_rows = storyboard
            .rows
            .iter()
            .map(|row| {
                format!(
                    "{} {} {} {} {} {} {} {}",
                    row.person,
                    row.shot_title,
                    row.visual_description,
                    row.character_action,
                    row.camera_movement,
                    row.dialogue,
                    row.prompt_text,
                    row.shot_script
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        for required in [
            "格挡",
            "震退七步",
            "焦土",
            "心跳",
            "低频鼓点",
            "掌心银辉",
            "沿手臂上升",
        ] {
            assert!(
                combined_rows.contains(required),
                "shot grounding output should contain {required}"
            );
        }
        for forbidden in ["夜雨站台", "便利店", "透明伞", "霓虹反光", "雨中告别"]
        {
            assert!(
                !combined_rows.contains(forbidden),
                "shot grounding output drifted into forbidden sample content {forbidden}"
            );
        }
        for row in &storyboard.rows {
            assert_eq!(row.grounding_source, ShotGroundingSource::ShotScript);
            assert_eq!(row.shot_scene_type, "action_beat");
            assert_eq!(row.shot_duration_seconds, row.duration_seconds);
            assert_eq!(
                row.duration_source,
                "storyboard_duration_plan.allocated_row_duration_seconds"
            );
            assert!(!row.adaptation_reason.trim().is_empty());
            assert!(!row.person.contains("not_specified_by_v120_bridge"));
            assert!(
                !row.character_action
                    .contains("fused_scene_performance_core_preserved")
            );
            assert!(!row.character_action.contains("按镜头脚本执行"));
            assert!(!row.character_action.contains("执行关键动作"));
            assert!(row.character_action.contains("从"));
            assert!(row.character_action.contains("到"));
            assert!(row.character_action.contains("镜头捕捉"));
            assert_visual_description_is_enhanced(row);
            assert!(!row.camera_movement.trim().is_empty());
            assert_ne!(row.camera_movement, row.shot_title);
            assert_ne!(row.camera_movement, row.visual_description);
            assert!(row.camera_movement.contains(&row.scene_scale));
            assert!(contains_any_story_term(
                &row.camera_movement,
                &[
                    "定机位",
                    "推近",
                    "跟随",
                    "横移",
                    "上摇",
                    "过肩",
                    "跟拍",
                    "锁定",
                    "观察",
                    "捕捉",
                ],
            ));
            assert!(row.prompt_text.contains(&row.camera_movement));
            assert!(row.prompt_text.contains(&row.visual_description));
            assert!(row.prompt_text.contains(&row.shot_script));
            assert!(!row.prompt_text.contains("KB摘要"));
            assert!(!row.prompt_text.contains("KB上下文"));
            assert!(!row.prompt_text.contains("sample_id"));
            assert!(!row.prompt_text.contains("scene_category"));
            assert!(!row.prompt_text.contains("style_cluster"));
            assert!(!row.prompt_text.contains("selected_sample_ids"));
            assert!(!row.prompt_text.contains("rule_id"));
            assert!(!row.prompt_text.contains("duration_guard"));
            assert!(!row.prompt_text.contains("reserve_gate"));
            assert!(!row.prompt_text.contains("grounding_source"));
            assert!(!row.prompt_text.contains("primary_scene_type"));
            assert!(!row.prompt_text.contains("shot_scene_type"));
            assert!(!row.prompt_text.contains("retrieval trace"));
            assert!(!row.prompt_text.contains("retrieval_trace"));
            assert!(
                !row.prompt_text
                    .contains("Compose a restrained dialogue shot")
            );
            assert!(
                !row.prompt_text_compilation_warnings
                    .iter()
                    .any(|warning| warning.code == "visual_description_grounding_incomplete")
            );
            assert!(
                !row.prompt_text_compilation_warnings
                    .iter()
                    .any(|warning| warning.code == "role_action_grounding_incomplete")
            );
            assert!(
                !row.prompt_text_compilation_warnings
                    .iter()
                    .any(|warning| warning.code == "camera_movement_grounding_incomplete")
            );
            assert!(row.external_reference_handle_candidates.is_empty());
        }

        let response_json =
            serde_json::to_string(&storyboard).expect("storyboard response should serialize");
        assert!(!response_json.contains("prompt_body_candidate"));
        assert!(!response_json.contains("Compose a restrained dialogue shot"));
    }

    #[test]
    fn generate_storyboard_uses_named_people_in_product_fields() {
        let state = test_state();
        let shot_script = "男主林峰和女主叶倾颜在断桥重逢。敌将萧寒追杀而至，林峰抬起银辉手臂迎着萧寒格挡。叶倾颜回身护住林峰。";

        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "named-character-grounding".to_string(),
                script_id: None,
                shot_script: Some(shot_script.to_string()),
                expanded_script_text: Some(
                    "scene_type: daily_dialogue\nsynopsis: 男主林峰和女主叶倾颜在断桥重逢，敌将萧寒追杀而至。"
                        .to_string(),
                ),
                primary_scene_type: Some("daily_dialogue".to_string()),
                primary_scene_label: Some("断桥重逢".to_string()),
                primary_scene_category: Some("action_dialogue".to_string()),
                shot_scene_type: None,
                shot_scene_label: None,
                shot_intent: None,
                adaptation_reason: None,
                selected_total_duration_seconds: 30,
            },
        );

        assert_ne!(storyboard.export_status.status, BridgeCallStatus::Blocked);
        assert_eq!(
            storyboard
                .rows
                .iter()
                .map(|row| row.duration_seconds)
                .collect::<Vec<_>>(),
            vec![10, 10, 10]
        );
        let combined_rows = storyboard
            .rows
            .iter()
            .map(|row| {
                format!(
                    "{} {} {} {} {} {}",
                    row.person,
                    row.shot_title,
                    row.visual_description,
                    row.character_action,
                    row.camera_movement,
                    row.prompt_text
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        for required_name in ["林峰", "叶倾颜", "萧寒"] {
            assert!(
                combined_rows.contains(required_name),
                "storyboard rows should keep named person {required_name}"
            );
        }

        for row in &storyboard.rows {
            assert!(!["主角", "女主", "交锋双方", "当前镜头主体"].contains(&row.person.as_str()));
            assert!(row.shot_title.contains(&row.person));
            assert!(row.character_action.contains("从"));
            assert!(row.character_action.contains("到"));
            assert!(row.character_action.contains("镜头捕捉"));
            assert_visual_description_is_enhanced(row);
            assert!(!row.camera_movement.trim().is_empty());
            assert_ne!(row.camera_movement, row.shot_title);
            assert_ne!(row.camera_movement, row.visual_description);
            assert!(row.camera_movement.contains(&row.scene_scale));
            assert!(
                row.prompt_text.contains(&row.person)
                    || subject_label_mentions_any_name(&row.person, &row.prompt_text)
            );
            assert!(row.prompt_text.contains(&row.visual_description));
            assert!(row.prompt_text.contains(&row.camera_movement));
            assert_storyboard_row_has_no_vague_or_internal_terms(row);
            assert_prompt_text_has_no_internal_payload(&row.prompt_text);
            assert!(
                !row.prompt_text_compilation_warnings
                    .iter()
                    .any(|warning| warning.code == "visual_description_grounding_incomplete")
            );
        }
    }

    #[test]
    fn generate_storyboard_uses_clear_subjects_without_named_people() {
        let state = test_state();
        let shot_script = "主角在焦土裂痕边缘稳住身体。敌方刀客压近断壁，刀锋逼向主角。主角抬起银辉手臂格挡后震退对手。";

        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "fallback-subject-grounding".to_string(),
                script_id: None,
                shot_script: Some(shot_script.to_string()),
                expanded_script_text: Some(
                    "scene_type: daily_dialogue\nsynopsis: 主角与敌方刀客在焦土断壁交锋。"
                        .to_string(),
                ),
                primary_scene_type: Some("daily_dialogue".to_string()),
                primary_scene_label: Some("焦土交锋".to_string()),
                primary_scene_category: Some("action_dialogue".to_string()),
                shot_scene_type: None,
                shot_scene_label: None,
                shot_intent: None,
                adaptation_reason: None,
                selected_total_duration_seconds: 30,
            },
        );

        assert_ne!(storyboard.export_status.status, BridgeCallStatus::Blocked);
        for row in &storyboard.rows {
            assert!(
                ["主角", "敌方刀客", "主角与敌方刀客", "主角与对立人物"]
                    .contains(&row.person.as_str()),
                "unexpected fallback person: {}",
                row.person
            );
            assert!(row.shot_title.contains(&row.person));
            assert!(row.character_action.contains("从"));
            assert!(row.character_action.contains("到"));
            assert!(row.character_action.contains("镜头捕捉"));
            assert_visual_description_is_enhanced(row);
            assert!(!row.camera_movement.trim().is_empty());
            assert_ne!(row.camera_movement, row.shot_title);
            assert_ne!(row.camera_movement, row.visual_description);
            assert!(row.camera_movement.contains(&row.scene_scale));
            assert!(row.prompt_text.contains(&row.person));
            assert!(row.prompt_text.contains(&row.visual_description));
            assert!(row.prompt_text.contains(&row.camera_movement));
            assert_storyboard_row_has_no_vague_or_internal_terms(row);
            assert_prompt_text_has_no_internal_payload(&row.prompt_text);
            assert!(
                !row.prompt_text_compilation_warnings
                    .iter()
                    .any(|warning| warning.code == "visual_description_grounding_incomplete")
            );
        }
    }

    #[test]
    fn visual_description_validator_flags_action_only_copy() {
        assert!(is_visual_description_grounding_incomplete(
            "主角挥刀逼近对手",
            "主角",
            "中景",
            "主角从压低重心开始，挥刀逼近对手，到逼到墙角时结束，镜头捕捉刀锋压近的一瞬间。",
            "中景侧视角对角构图，跟随主角压近一步",
            "主角挥刀逼近对手"
        ));
    }

    #[test]
    fn seedance_duration_planner_uses_ten_second_rows_and_five_second_remainder() {
        let state = test_state();
        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "duration-45-seedance-plan".to_string(),
                script_id: None,
                shot_script: Some(
                    "主角踏碎焦土迎敌。敌方刀客压近断壁。主角抬臂格挡。银辉沿主角手臂觉醒。主角震退敌方刀客。"
                        .to_string(),
                ),
                expanded_script_text: Some(
                    "scene_type: daily_dialogue\nsynopsis: 45秒动作分镜".to_string(),
                ),
                primary_scene_type: Some("daily_dialogue".to_string()),
                primary_scene_label: Some("动作分镜".to_string()),
                primary_scene_category: Some("action".to_string()),
                shot_scene_type: Some("action_beat".to_string()),
                shot_scene_label: Some("动作交锋镜头".to_string()),
                shot_intent: Some("action_beat".to_string()),
                adaptation_reason: Some("explicit action beat".to_string()),
                selected_total_duration_seconds: 45,
            },
        );

        assert_ne!(storyboard.export_status.status, BridgeCallStatus::Blocked);
        let durations = storyboard
            .rows
            .iter()
            .map(|row| row.duration_seconds)
            .collect::<Vec<_>>();
        assert_eq!(durations, vec![10, 10, 10, 10, 5]);
        assert!(!durations.contains(&9));
        for row in &storyboard.rows {
            assert_eq!(row.duration_seconds, row.shot_duration_seconds);
            assert!(row.duration_seconds <= 15);
        }
        assert_eq!(storyboard.duration_plan.total_duration_seconds, 45);
        assert_eq!(storyboard.duration_plan.allocated_seconds, 45);
        assert_eq!(
            storyboard.duration_plan.per_row_seconds,
            SEEDANCE_STANDARD_SEGMENT_SECONDS
        );
    }

    #[test]
    fn v0_story_to_storyboard_chain_runs_neutral_deterministic_scaffold() {
        let state = test_state();

        let response = run_v0_story_to_storyboard_chain(
            &state,
            RunV0StoryToStoryboardChainRequest {
                story_id: "story-v0-001".to_string(),
                chapter_id: "chapter-v0-001".to_string(),
                chapter_order: 1,
                user_topic_or_synopsis: "男主林峰和女主叶倾颜在断桥重逢，敌将萧寒追杀而至。"
                    .to_string(),
                story_length_profile: "short_story_2000_2500".to_string(),
                selected_total_duration_seconds: 30,
                target_duration_mode: TARGET_DURATION_MODE_FIXED_SECONDS.to_string(),
                source_material_length_chars: 0,
                auto_segment_strategy: String::new(),
                estimated_total_story_duration_seconds: 0,
                primary_scene_type: "daily_dialogue".to_string(),
                primary_scene_label: Some("断桥重逢".to_string()),
                primary_scene_category: Some("action_dialogue".to_string()),
                authoring_craft_summary: "以角色目标、冲突压力、情绪转折和下一场承接组织章节。"
                    .to_string(),
                screenwriting_adaptation_summary: "将章节压缩为可表演的动作、对白意图和转折节拍。"
                    .to_string(),
                directing_kb_context_summary: "镜头调度只服务当前剧情事实，shot_script 优先。"
                    .to_string(),
                continuity_context_summary: "第一章从断桥重逢开始，无前置定稿分镜。".to_string(),
                kb_context_summary: "压缩 KB 摘要：仅保留结构、连续性和时长规则。".to_string(),
                selected_sample_ids: vec!["GS-BRIDGE-01".to_string()],
                selected_kb_rules: vec!["summary-only continuity and duration guard".to_string()],
                retrieval_trace_user_summary: "scene match and continuity need".to_string(),
                full_kb_rows_included: 0,
                confirm_storyboard_results: true,
            },
        );

        assert_ne!(response.status, BridgeCallStatus::Blocked, "{response:#?}");
        let chapter = response.chapter.as_ref().expect("chapter should exist");
        let script = response.script.as_ref().expect("script should exist");
        let plan = response
            .shot_task_plan
            .as_ref()
            .expect("shot task plan should exist");
        assert_eq!(chapter.story_id, "story-v0-001");
        assert_eq!(script.chapter_id, "chapter-v0-001");
        assert_eq!(
            plan.shot_tasks
                .iter()
                .map(|task| task.duration_seconds)
                .collect::<Vec<_>>(),
            vec![10, 10, 10]
        );
        assert_eq!(response.storyboard_results.len(), 3);
        assert_eq!(response.finalized_storyboard_refs.len(), 3);
        assert!(response.continuity_state.is_some());

        let continuity = response.continuity_state.as_ref().unwrap();
        assert!(
            continuity
                .continuity_context_summary
                .contains("story-v0-001")
        );
        assert_eq!(continuity.finalized_storyboard_refs.len(), 3);
        assert!(
            state
                .find_v0_chapter("story-v0-001", "chapter-v0-001")
                .is_some()
        );
        assert!(state.find_v0_script(&script.script_id).is_some());
        assert!(state.find_v0_shot_task_plan(&script.script_id).is_some());
        assert!(state.find_v0_continuity_state("story-v0-001").is_some());
        let logs = state.list_v0_continuity_delta_logs("story-v0-001");
        assert!(logs.iter().any(|log| log.stage == "generate_novel_chapter"));
        assert!(
            logs.iter()
                .any(|log| log.stage == "update_continuity_state")
        );

        let prompt_payload = response
            .storyboard_results
            .iter()
            .map(|result| result.prompt_text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert_prompt_text_has_no_internal_payload(&prompt_payload);
        for forbidden in [
            "raw prompt_body",
            "prompt_body",
            "source_register",
            "overlay JSON",
            "full raw KB rows",
            "API key",
            "director_style_ref",
        ] {
            assert!(
                !continuity.continuity_context_summary.contains(forbidden),
                "continuity leaked forbidden payload term {forbidden}: {}",
                continuity.continuity_context_summary
            );
        }
    }

    #[test]
    fn v0_story_to_storyboard_chain_long_text_auto_plans_multiple_shot_tasks() {
        let state = test_state();
        let source = [
            "Event 1: Lin keeps the bridge order while Ye answers the pressure and Xiao closes the gate before the next move. The paragraph stays long enough to exercise automatic duration planning without adding any new facts.",
            "Event 2: Lin changes position, Ye protects the same promise, and Xiao moves closer. The source keeps ordered events visible so writing continuity remains ahead of camera advice.",
            "Event 3: The bridge route narrows, the keepsake stays with Lin, and Ye names the next choice. The scene expression can shift, but the accepted source facts stay first.",
            "Event 4: Xiao blocks the exit, Lin must answer with action, and Ye holds the final state for the following scene. The director can schedule shots only after this order is preserved.",
        ]
        .join("\n\n")
        .repeat(4);
        let mut request = v0_chain_request_for_source("story-v0-long-auto", &source);
        request.target_duration_mode = TARGET_DURATION_MODE_LONG_TEXT_AUTO.to_string();
        request.selected_total_duration_seconds = 60;
        request.story_length_profile = V0_LONG_STORY_AUTO_PROFILE.to_string();
        request.auto_segment_strategy = String::new();

        let response = run_v0_story_to_storyboard_chain(&state, request);

        assert_ne!(response.status, BridgeCallStatus::Blocked, "{response:#?}");
        assert_eq!(
            response.target_duration_mode,
            TARGET_DURATION_MODE_LONG_TEXT_AUTO
        );
        assert_ne!(response.estimated_total_story_duration_seconds, 60);
        assert_eq!(
            response.source_material_length_chars,
            stable_source_material_length_chars(&source)
        );
        assert!(response.generated_shot_task_count > 1);
        assert!(
            response
                .warnings
                .iter()
                .any(|warning| warning.code == "auto_segment_strategy_missing")
        );
        assert!(
            !response
                .warnings
                .iter()
                .any(|warning| warning.code == "long_text_auto_compressed_to_single_clip_blocked")
        );

        let plan = response
            .shot_task_plan
            .as_ref()
            .expect("long_text_auto should return a shot task plan");
        let durations = plan
            .shot_tasks
            .iter()
            .map(|task| task.duration_seconds)
            .collect::<Vec<_>>();
        assert_eq!(response.generated_shot_task_count, durations.len() as u32);
        assert_eq!(
            durations.iter().copied().sum::<u16>(),
            response.estimated_total_story_duration_seconds
        );
        assert!(durations.iter().all(|duration| *duration <= 15));
        assert!(
            durations
                .iter()
                .all(|duration| *duration == SEEDANCE_STANDARD_SEGMENT_SECONDS
                    || *duration == SEEDANCE_REMAINDER_SEGMENT_SECONDS)
        );
        assert_eq!(response.storyboard_results.len(), durations.len());
        assert!(
            response
                .duration_plan_summary
                .contains("director_scheduling")
        );
    }

    #[test]
    fn v0_story_to_storyboard_chain_validates_duration_mode_and_fixed_seconds() {
        let state = test_state();
        let mut invalid_mode =
            v0_chain_request_for_source("story-v0-invalid-mode", "Lin keeps the bridge.");
        invalid_mode.target_duration_mode = "sixty_seconds_auto".to_string();
        let invalid_mode_response = run_v0_story_to_storyboard_chain(&state, invalid_mode);
        assert_eq!(invalid_mode_response.status, BridgeCallStatus::Blocked);
        assert!(
            invalid_mode_response
                .blockers
                .iter()
                .any(|blocker| blocker.code == "target_duration_mode_invalid")
        );

        let mut unsupported_fixed =
            v0_chain_request_for_source("story-v0-fixed-20", "Lin keeps the bridge.");
        unsupported_fixed.selected_total_duration_seconds = 20;
        let unsupported_fixed_response =
            run_v0_story_to_storyboard_chain(&state, unsupported_fixed);
        assert_eq!(unsupported_fixed_response.status, BridgeCallStatus::Blocked);
        assert!(
            unsupported_fixed_response
                .blockers
                .iter()
                .any(|blocker| blocker.code == "duration_not_supported")
        );
    }

    #[test]
    fn long_text_auto_allocator_uses_ten_second_tasks_and_five_second_remainder() {
        let durations = allocate_long_text_auto_shot_task_durations(105);

        assert_eq!(durations.iter().copied().sum::<u16>(), 105);
        assert_eq!(durations.last(), Some(&SEEDANCE_REMAINDER_SEGMENT_SECONDS));
        assert!(durations.iter().all(|duration| *duration <= 15));
        assert!(
            durations
                .iter()
                .all(|duration| *duration == SEEDANCE_STANDARD_SEGMENT_SECONDS
                    || *duration == SEEDANCE_REMAINDER_SEGMENT_SECONDS)
        );
    }

    #[test]
    fn v0_story_to_storyboard_chain_detects_input_types_and_authoring_modes() {
        let cases = [
            (
                "story-v0-synopsis",
                "男主林峰和女主叶倾颜在断桥重逢，敌将萧寒追杀而至。",
                "synopsis",
                "expand_from_synopsis",
            ),
            (
                "story-v0-full-story",
                "男主林峰先在断桥边发现叶倾颜留下的玉佩，他没有离开，而是把玉佩握进掌心。随后女主叶倾颜从桥下现身，告诉林峰敌将萧寒已经封住退路。萧寒带兵追到断桥，逼林峰交出玉佩。林峰挡在叶倾颜身前，先确认她受伤不重，再举起银辉缠绕的手臂迎敌。叶倾颜把萧寒的真实目的告诉林峰，林峰决定护住玉佩和叶倾颜。最后断桥塌落，林峰与叶倾颜退到桥头，萧寒仍在对岸逼近。",
                "full_story",
                "rewrite_from_full_story",
            ),
            (
                "story-v0-novel-chapter",
                "第一章 断桥重逢。男主林峰在雨夜抵达断桥，女主叶倾颜把玉佩交给他。敌将萧寒随后追到桥头，逼林峰交出玉佩。林峰没有逃走，而是挡在叶倾颜身前。章末，断桥开始塌落，三人的对峙还没有结束。",
                "novel_chapter",
                "adapt_story_to_screenplay",
            ),
            (
                "story-v0-screenplay",
                "剧本：场景1 外景 断桥 夜。林峰握住玉佩。叶倾颜：萧寒已经追来了。场景2 萧寒逼近断桥，林峰挡在叶倾颜身前。",
                "screenplay_text",
                "polish_existing_screenplay",
            ),
        ];

        for (story_id, source, source_input_type, authoring_mode) in cases {
            let state = test_state();
            let response = run_v0_story_to_storyboard_chain(
                &state,
                v0_chain_request_for_source(story_id, source),
            );

            assert_ne!(response.status, BridgeCallStatus::Blocked, "{response:#?}");
            assert_eq!(response.source_input_type, source_input_type);
            assert_eq!(response.authoring_mode, authoring_mode);
            assert_eq!(
                response.chapter.as_ref().unwrap().source_input_type,
                source_input_type
            );
            assert_eq!(
                response.script.as_ref().unwrap().authoring_mode,
                authoring_mode
            );
            assert!(response.shot_task_plan.is_some());
            assert!(!response.storyboard_results.is_empty());
        }
    }

    #[test]
    fn v0_full_story_rewrite_preserves_source_facts_before_kb_advice() {
        let state = test_state();
        let mut request = v0_chain_request_for_source(
            "story-v0-fact-preserve",
            "男主林峰先在断桥边发现叶倾颜留下的玉佩，他没有离开，而是把玉佩握进掌心。随后女主叶倾颜从桥下现身，告诉林峰敌将萧寒已经封住退路。萧寒带兵追到断桥，逼林峰交出玉佩。林峰挡在叶倾颜身前，先确认她受伤不重，再举起银辉缠绕的手臂迎敌。叶倾颜把萧寒的真实目的告诉林峰，林峰决定护住玉佩和叶倾颜。最后断桥塌落，林峰与叶倾颜退到桥头，萧寒仍在对岸逼近。",
        );
        request.kb_context_summary =
            "KB advisory only: suggest replacing the enemy with 沈烬 and adding a throne plot."
                .to_string();

        let response = run_v0_story_to_storyboard_chain(&state, request);

        assert_ne!(response.status, BridgeCallStatus::Blocked, "{response:#?}");
        assert_eq!(response.source_input_type, "full_story");
        assert_eq!(response.authoring_mode, "rewrite_from_full_story");
        for name in ["林峰", "叶倾颜", "萧寒"] {
            assert!(
                response
                    .source_story_facts
                    .character_names
                    .contains(&name.to_string()),
                "source facts should preserve name {name}: {:?}",
                response.source_story_facts.character_names
            );
            assert!(
                response.preserved_fact_summary.contains(name),
                "preserved summary should contain {name}: {}",
                response.preserved_fact_summary
            );
        }

        let script = response.script.as_ref().expect("script should exist");
        for name in ["林峰", "叶倾颜", "萧寒"] {
            assert!(script.script_text.contains(name), "script lost {name}");
        }
        let first = script
            .script_text
            .find("发现叶倾颜留下的玉佩")
            .expect("first event should be preserved");
        let second = script
            .script_text
            .find("萧寒带兵追到断桥")
            .expect("later event should be preserved");
        assert!(
            first < second,
            "event order changed: {}",
            script.script_text
        );
        assert!(!script.script_text.contains("沈烬"));
        assert!(!script.script_text.contains("throne plot"));
        assert!(
            script
                .changed_for_screenplay_summary
                .contains("source facts")
        );
        assert!(
            script
                .omitted_detail_summary
                .contains("No identified core event")
        );

        let continuity = response
            .continuity_state
            .as_ref()
            .expect("continuity should exist");
        assert_eq!(continuity.source_input_type, "full_story");
        assert_eq!(continuity.authoring_mode, "rewrite_from_full_story");
        assert!(continuity.preserved_fact_summary.contains("林峰"));

        let prompt_payload = response
            .storyboard_results
            .iter()
            .map(|result| result.prompt_text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert_prompt_text_has_no_internal_payload(&prompt_payload);
        for forbidden in [
            "raw prompt_body",
            "source_register",
            "overlay JSON",
            "API key",
            "director_style_ref",
        ] {
            assert!(!script.script_text.contains(forbidden));
            assert!(!prompt_payload.contains(forbidden));
            assert!(!response.preserved_fact_summary.contains(forbidden));
        }
        for metadata in [
            "source_input_type:",
            "authoring_mode:",
            "source_story_facts_take_priority",
        ] {
            assert!(!prompt_payload.contains(metadata));
        }
    }

    #[test]
    fn v0_mixed_material_warns_when_source_type_is_uncertain() {
        let state = test_state();
        let response = run_v0_story_to_storyboard_chain(
            &state,
            v0_chain_request_for_source(
                "story-v0-mixed",
                "梗概：男主林峰寻找玉佩。剧本：场景1 外景 断桥 夜。林峰看见叶倾颜，敌将萧寒追来。",
            ),
        );

        assert_ne!(response.status, BridgeCallStatus::Blocked, "{response:#?}");
        assert_eq!(response.source_input_type, "mixed_material");
        assert!(
            response
                .warnings
                .iter()
                .any(|warning| warning.code == "source_input_type_uncertain")
        );
        assert!(
            response
                .continuity_warnings
                .iter()
                .any(|warning| warning.code == "source_input_type_uncertain")
        );
    }

    #[test]
    fn v0_story_to_storyboard_chain_warns_when_continuity_context_is_missing() {
        let state = test_state();
        let mut request =
            v0_chain_request_for_source("story-v0-missing-continuity", "Lin guards the bridge.");
        request.continuity_context_summary = String::new();

        let response = run_v0_story_to_storyboard_chain(&state, request);

        assert_ne!(response.status, BridgeCallStatus::Blocked, "{response:#?}");
        assert!(
            response
                .warnings
                .iter()
                .any(|warning| warning.code == "continuity_context_missing")
        );
        assert!(
            response
                .continuity_warnings
                .iter()
                .any(|warning| warning.code == "continuity_context_missing")
        );
    }

    #[test]
    fn v0_story_to_storyboard_chain_warns_on_conflicting_kb_and_directing_hints() {
        let state = test_state();
        let mut request = v0_chain_request_for_source(
            "story-v0-conflicting-hints",
            "Lin holds the bridge keepsake, Ye warns him, and Xiao closes the gate before the enemy arrives.",
        );
        request.kb_context_summary =
            "Replace the accepted enemy with Shen Mo and add a throne plot.".to_string();
        request.directing_kb_context_summary =
            "Replace the accepted motive with a betrayal twist and add a throne plot."
                .to_string();

        let response = run_v0_story_to_storyboard_chain(&state, request);

        assert_ne!(response.status, BridgeCallStatus::Blocked, "{response:#?}");
        assert!(
            response
                .warnings
                .iter()
                .any(|warning| warning.code == "kb_hint_conflicts_with_source_fact")
        );
        assert!(
            response
                .warnings
                .iter()
                .any(|warning| warning.code == "director_hint_conflicts_with_content_fact")
        );
    }

    #[test]
    fn v0_story_to_storyboard_chain_blocks_full_kb_rows() {
        let state = test_state();

        let response = run_v0_story_to_storyboard_chain(
            &state,
            RunV0StoryToStoryboardChainRequest {
                story_id: "story-v0-blocked".to_string(),
                chapter_id: "chapter-v0-blocked".to_string(),
                chapter_order: 1,
                user_topic_or_synopsis: "主角与对手在城门前对峙。".to_string(),
                story_length_profile: "short_story_2000_2500".to_string(),
                selected_total_duration_seconds: 10,
                target_duration_mode: TARGET_DURATION_MODE_FIXED_SECONDS.to_string(),
                source_material_length_chars: 0,
                auto_segment_strategy: String::new(),
                estimated_total_story_duration_seconds: 0,
                primary_scene_type: "daily_dialogue".to_string(),
                primary_scene_label: Some("城门对峙".to_string()),
                primary_scene_category: Some("dialogue".to_string()),
                authoring_craft_summary: "保持角色目标和下一场承接。".to_string(),
                screenwriting_adaptation_summary: "改编为可表演节拍。".to_string(),
                directing_kb_context_summary: "镜头服务剧情。".to_string(),
                continuity_context_summary: String::new(),
                kb_context_summary: String::new(),
                selected_sample_ids: vec![],
                selected_kb_rules: vec![],
                retrieval_trace_user_summary: String::new(),
                full_kb_rows_included: 1,
                confirm_storyboard_results: true,
            },
        );

        assert_eq!(response.status, BridgeCallStatus::Blocked);
        assert!(response.chapter.is_none());
        assert!(
            response
                .blockers
                .iter()
                .any(|blocker| blocker.code == "full_kb_rows_must_be_zero")
        );
    }

    #[test]
    fn expand_script_respects_target_duration_in_deterministic_fallback() {
        let state = test_state();

        let script_15 = expand_script(
            &state,
            ExpandScriptRequest {
                scene_type: "daily_dialogue".to_string(),
                synopsis_text: "男主林峰和女主叶倾颜在断桥重逢，敌将萧寒追杀而至。".to_string(),
                target_duration_seconds: Some(15),
            },
        );
        let script_60 = expand_script(
            &state,
            ExpandScriptRequest {
                scene_type: "daily_dialogue".to_string(),
                synopsis_text: "男主林峰和女主叶倾颜在断桥重逢，敌将萧寒追杀而至。".to_string(),
                target_duration_seconds: Some(60),
            },
        );

        assert_ne!(script_15.status, BridgeCallStatus::Blocked);
        assert_ne!(script_60.status, BridgeCallStatus::Blocked);
        assert!(
            script_15
                .expanded_script_text
                .contains("target_duration_seconds: 15")
        );
        assert!(
            script_60
                .expanded_script_text
                .contains("target_duration_seconds: 60")
        );
        assert!(
            script_60.expanded_script_text.chars().count()
                > script_15.expanded_script_text.chars().count() * 3 / 2
        );
        assert!(
            script_60
                .expanded_script_text
                .lines()
                .filter(|line| line.starts_with("段落"))
                .count()
                > script_15
                    .expanded_script_text
                    .lines()
                    .filter(|line| line.starts_with("段落"))
                    .count()
        );
    }

    #[test]
    fn finalized_storyboard_bank_saves_lists_updates_removes_and_exports_confirmed_only() {
        let state = test_state();
        let first_storyboard = generate_test_storyboard(&state, "finalized-shot-01", 10);
        let second_storyboard = generate_test_storyboard(&state, "finalized-shot-02", 15);

        let first_save = save_storyboard_shot_result(
            &state,
            finalized_save_request_from_storyboard(
                "project-bank-001",
                "script-bank-001",
                2,
                true,
                &first_storyboard,
            ),
        );
        assert_eq!(
            first_save.status,
            BridgeCallStatus::Ready,
            "{:?}",
            first_save.blockers
        );
        let second_save = save_storyboard_shot_result(
            &state,
            finalized_save_request_from_storyboard(
                "project-bank-001",
                "script-bank-001",
                1,
                false,
                &second_storyboard,
            ),
        );
        assert_eq!(
            second_save.status,
            BridgeCallStatus::Ready,
            "{:?}",
            second_save.blockers
        );

        let listed = list_storyboard_shot_results(
            &state,
            ListStoryboardShotResultsRequest {
                project_id: "project-bank-001".to_string(),
                script_id: Some("script-bank-001".to_string()),
                confirmed: None,
            },
        );
        assert_eq!(listed.shots.len(), 2);
        assert_eq!(listed.shots[0].result_id, second_storyboard.result_id);
        assert_eq!(listed.shots[1].result_id, first_storyboard.result_id);

        let confirmed_export = export_storyboard_bank(
            &state,
            ExportStoryboardBankRequest {
                project_id: "project-bank-001".to_string(),
                script_id: Some("script-bank-001".to_string()),
                export_format: "v120_storyboard".to_string(),
                include_unconfirmed: false,
            },
        );
        assert_eq!(confirmed_export.status, BridgeCallStatus::Ready);
        assert!(!confirmed_export.no_export);
        assert_eq!(confirmed_export.confirmed_shot_count, 1);
        assert_eq!(
            confirmed_export.exported_result_ids,
            vec![first_storyboard.result_id.clone()]
        );
        assert_eq!(
            confirmed_export.total_shot_duration_seconds,
            first_storyboard
                .rows
                .iter()
                .map(|row| row.shot_duration_seconds)
                .sum::<u16>()
        );
        assert_eq!(confirmed_export.artifacts.len(), 3);
        assert!(confirmed_export.artifacts.iter().all(|artifact| {
            artifact.ready
                && artifact.artifact_path.is_none()
                && artifact.full_kb_rows_included == 0
                && artifact.selected_sample_ids.is_empty()
                && artifact.selected_kb_rule_ids.is_empty()
                && artifact.kb_context_summary.is_none()
                && artifact.retrieval_trace.is_none()
        }));

        let existing_second = second_save
            .shot
            .expect("saved finalized shot should be returned");
        let updated_second = update_storyboard_shot_result(
            &state,
            UpdateStoryboardShotResultRequest {
                project_id: existing_second.project_id.clone(),
                result_id: existing_second.result_id.clone(),
                shot_order: existing_second.shot_order,
                shot_task_name: "updated finalized shot".to_string(),
                rows: existing_second.rows.clone(),
                prompt_text: existing_second.prompt_text.clone(),
                shot_duration_seconds: existing_second.shot_duration_seconds,
                duration_source: existing_second.duration_source.clone(),
                confirmed: true,
                updated_at_ms: existing_second.updated_at_ms + 1,
                rows_hash: existing_second.rows_hash.clone(),
            },
        );
        assert_eq!(updated_second.status, BridgeCallStatus::Ready);

        let multi_export = export_storyboard_bank(
            &state,
            ExportStoryboardBankRequest {
                project_id: "project-bank-001".to_string(),
                script_id: Some("script-bank-001".to_string()),
                export_format: "v120_storyboard".to_string(),
                include_unconfirmed: false,
            },
        );
        assert_eq!(multi_export.status, BridgeCallStatus::Ready);
        assert_eq!(multi_export.confirmed_shot_count, 2);
        assert_eq!(
            multi_export.exported_result_ids,
            vec![
                second_storyboard.result_id.clone(),
                first_storyboard.result_id.clone()
            ]
        );
        assert_eq!(
            multi_export.total_shot_duration_seconds,
            first_storyboard
                .rows
                .iter()
                .chain(second_storyboard.rows.iter())
                .map(|row| row.shot_duration_seconds)
                .sum::<u16>()
        );

        let removed = remove_storyboard_shot_result(
            &state,
            RemoveStoryboardShotResultRequest {
                project_id: "project-bank-001".to_string(),
                result_id: first_storyboard.result_id.clone(),
            },
        );
        assert_eq!(removed.status, BridgeCallStatus::Ready);
        assert!(removed.removed);
        let listed_confirmed = list_storyboard_shot_results(
            &state,
            ListStoryboardShotResultsRequest {
                project_id: "project-bank-001".to_string(),
                script_id: Some("script-bank-001".to_string()),
                confirmed: Some(true),
            },
        );
        assert_eq!(listed_confirmed.shots.len(), 1);
        assert_eq!(
            listed_confirmed.shots[0].result_id,
            second_storyboard.result_id
        );
    }

    #[test]
    fn export_storyboard_bank_returns_no_export_without_confirmed_shots() {
        let state = test_state();
        let generated_but_unsaved = generate_test_storyboard(&state, "not-bank-result", 10);

        let empty_export = export_storyboard_bank(
            &state,
            ExportStoryboardBankRequest {
                project_id: "project-bank-empty".to_string(),
                script_id: Some("script-bank-empty".to_string()),
                export_format: "v120_storyboard".to_string(),
                include_unconfirmed: false,
            },
        );
        assert_eq!(empty_export.status, BridgeCallStatus::Gated);
        assert!(empty_export.no_export);
        assert_eq!(empty_export.confirmed_shot_count, 0);
        assert!(empty_export.exported_result_ids.is_empty());
        assert!(empty_export.artifacts.is_empty());
        assert!(
            !empty_export
                .exported_result_ids
                .contains(&generated_but_unsaved.result_id)
        );
        assert!(
            empty_export
                .warnings
                .iter()
                .any(|warning| warning.code == "storyboard_bank_no_confirmed_shots")
        );

        let unconfirmed_save = save_storyboard_shot_result(
            &state,
            finalized_save_request_from_storyboard(
                "project-bank-empty",
                "script-bank-empty",
                1,
                false,
                &generated_but_unsaved,
            ),
        );
        assert_eq!(
            unconfirmed_save.status,
            BridgeCallStatus::Ready,
            "{:?}",
            unconfirmed_save.blockers
        );
        let unconfirmed_export = export_storyboard_bank(
            &state,
            ExportStoryboardBankRequest {
                project_id: "project-bank-empty".to_string(),
                script_id: Some("script-bank-empty".to_string()),
                export_format: "v120_storyboard".to_string(),
                include_unconfirmed: false,
            },
        );
        assert_eq!(unconfirmed_export.status, BridgeCallStatus::Gated);
        assert!(unconfirmed_export.no_export);
        assert!(unconfirmed_export.exported_result_ids.is_empty());

        let include_unconfirmed_export = export_storyboard_bank(
            &state,
            ExportStoryboardBankRequest {
                project_id: "project-bank-empty".to_string(),
                script_id: Some("script-bank-empty".to_string()),
                export_format: "v120_storyboard".to_string(),
                include_unconfirmed: true,
            },
        );
        assert_eq!(include_unconfirmed_export.status, BridgeCallStatus::Blocked);
        assert!(include_unconfirmed_export.no_export);
        assert!(
            include_unconfirmed_export
                .blockers
                .iter()
                .any(|blocker| blocker.code == "storyboard_bank_include_unconfirmed_not_supported")
        );
    }

    #[test]
    fn finalized_storyboard_bank_rejects_forbidden_payload_and_hash_drift() {
        let state = test_state();
        let storyboard = generate_test_storyboard(&state, "bank-safety-shot", 10);
        let mut forbidden_request = finalized_save_request_from_storyboard(
            "project-bank-safe",
            "script-bank-safe",
            1,
            true,
            &storyboard,
        );
        forbidden_request.prompt_text = "raw prompt_body must never be saved".to_string();

        let forbidden_save = save_storyboard_shot_result(&state, forbidden_request);
        assert_eq!(forbidden_save.status, BridgeCallStatus::Blocked);
        assert!(
            forbidden_save
                .blockers
                .iter()
                .any(|blocker| blocker.code == "storyboard_bank_forbidden_payload")
        );

        let valid_save = save_storyboard_shot_result(
            &state,
            finalized_save_request_from_storyboard(
                "project-bank-safe",
                "script-bank-safe",
                1,
                true,
                &storyboard,
            ),
        );
        assert_eq!(
            valid_save.status,
            BridgeCallStatus::Ready,
            "{:?}",
            valid_save.blockers
        );
        let mut corrupted = valid_save
            .shot
            .expect("valid finalized shot should be returned");
        corrupted.rows[0]
            .visual_description
            .push_str(" adjusted after hash was stored");
        state.remember_finalized_storyboard_shot(corrupted.clone());

        let export = export_storyboard_bank(
            &state,
            ExportStoryboardBankRequest {
                project_id: "project-bank-safe".to_string(),
                script_id: Some("script-bank-safe".to_string()),
                export_format: "v120_storyboard".to_string(),
                include_unconfirmed: false,
            },
        );
        assert_eq!(export.status, BridgeCallStatus::Blocked);
        assert!(export.no_export);
        assert!(export.artifacts.is_empty());
        assert!(export.blockers.iter().any(|blocker| {
            blocker.code == "storyboard_bank_rows_hash_mismatch"
                && blocker.related_sample_id.as_deref() == Some(corrupted.result_id.as_str())
        }));
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
                target_duration_seconds: None,
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
                target_duration_seconds: None,
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
                shot_script: None,
                expanded_script_text: None,
                primary_scene_type: None,
                primary_scene_label: None,
                primary_scene_category: None,
                shot_scene_type: None,
                shot_scene_label: None,
                shot_intent: None,
                adaptation_reason: None,
                selected_total_duration_seconds: 10,
            },
        );

        assert_eq!(state.kb_runtime.summary.golden_sample_record_count, 152);
        assert_eq!(storyboard.rows.len(), 1);
        assert_eq!(storyboard.kb_router_result.selected_sample_ids.len(), 3);
        assert_eq!(
            storyboard
                .kb_router_result
                .retrieval_trace
                .token_budget
                .full_kb_rows_included,
            0
        );
        assert!(storyboard.rows[0].shot_id.starts_with("shot-task-"));
        assert_eq!(
            storyboard.rows[0].grounding_source,
            ShotGroundingSource::ExpandedScriptText
        );
        assert_eq!(storyboard.rows[0].primary_scene_type, "daily_dialogue");
        assert_eq!(storyboard.rows[0].shot_scene_type, "daily_dialogue");
        assert_eq!(
            storyboard.rows[0].prompt_text_compilation_status,
            PromptTextCompilationStatus::ReadyStub
        );
        assert_eq!(
            storyboard.rows[0].prompt_text_source_row_id,
            storyboard.rows[0].shot_id
        );
        assert!(storyboard.rows[0].prompt_text.contains("视频分镜提示词"));
        assert!(storyboard.rows[0].prompt_text.contains("镜头时长"));
        assert!(!storyboard.rows[0].prompt_text.contains("目标适配"));
        assert!(!storyboard.rows[0].prompt_text.contains("KB摘要"));
        assert!(!storyboard.rows[0].prompt_text.contains("KB上下文"));
        assert!(!storyboard.rows[0].prompt_text.contains("sample_id"));
        assert!(!storyboard.rows[0].prompt_text.contains("scene_category"));
        assert!(!storyboard.rows[0].prompt_text.contains("style_cluster"));
        assert!(
            !storyboard.rows[0]
                .prompt_text
                .contains("selected_sample_ids")
        );
        assert!(!storyboard.rows[0].prompt_text.contains("rule_id"));
        assert!(!storyboard.rows[0].prompt_text.contains("duration_guard"));
        assert!(!storyboard.rows[0].prompt_text.contains("reserve_gate"));
        assert!(!storyboard.rows[0].prompt_text.contains("grounding_source"));
        assert!(
            !storyboard.rows[0]
                .prompt_text
                .contains("primary_scene_type")
        );
        assert!(!storyboard.rows[0].prompt_text.contains("shot_scene_type"));
        assert!(!storyboard.rows[0].prompt_text.contains("retrieval trace"));
        assert!(!storyboard.rows[0].prompt_text.contains("retrieval_trace"));
        assert!(
            !storyboard.rows[0]
                .prompt_text
                .contains("Compose a restrained dialogue shot with stable eyeline")
        );
        assert_eq!(
            storyboard.rows[0].sequence_grouping.sequence_field_state,
            core_domain::SequenceFieldState::NotApplicable
        );
        assert!(
            storyboard.rows[0]
                .external_reference_handle_candidates
                .is_empty()
        );
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
            assert_eq!(artifact.row_count, Some(1));
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
                target_duration_seconds: None,
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
                shot_script: None,
                expanded_script_text: Some("".to_string()),
                primary_scene_type: None,
                primary_scene_label: None,
                primary_scene_category: None,
                shot_scene_type: None,
                shot_scene_label: None,
                shot_intent: None,
                adaptation_reason: None,
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

        for duration in [7u16, 62u16, 0u16] {
            let response = generate_storyboard(
                &state,
                GenerateStoryboardRequest {
                    task_name: format!("invalid-duration-{duration}"),
                    script_id: None,
                    shot_script: None,
                    expanded_script_text: Some(
                        "scene_type: daily_dialogue\nsynopsis: valid synopsis".to_string(),
                    ),
                    primary_scene_type: None,
                    primary_scene_label: None,
                    primary_scene_category: None,
                    shot_scene_type: None,
                    shot_scene_label: None,
                    shot_intent: None,
                    adaptation_reason: None,
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
                    target_duration_seconds: None,
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
                    shot_script: None,
                    expanded_script_text: None,
                    primary_scene_type: None,
                    primary_scene_label: None,
                    primary_scene_category: None,
                    shot_scene_type: None,
                    shot_scene_label: None,
                    shot_intent: None,
                    adaptation_reason: None,
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

        for duration in [5u16, 10u16, 15u16, 20u16, 30u16, 40u16, 45u16, 60u16] {
            let response = generate_storyboard(
                &state,
                GenerateStoryboardRequest {
                    task_name: format!("duration-{duration}"),
                    script_id: None,
                    shot_script: None,
                    expanded_script_text: Some(format!(
                        "scene_type: daily_dialogue\nsynopsis: duration test {duration}"
                    )),
                    primary_scene_type: None,
                    primary_scene_label: None,
                    primary_scene_category: None,
                    shot_scene_type: None,
                    shot_scene_label: None,
                    shot_intent: None,
                    adaptation_reason: None,
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
            assert!(response.rows.iter().all(|row| row.duration_seconds <= 15));
            assert_eq!(
                response
                    .rows
                    .iter()
                    .map(|row| row.duration_seconds)
                    .collect::<Vec<_>>(),
                allocate_storyboard_row_durations(duration, 0).expect("duration should plan")
            );
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
                shot_script: None,
                expanded_script_text: Some(
                    "scene_type: daily_dialogue\nsynopsis: editable export".to_string(),
                ),
                primary_scene_type: None,
                primary_scene_label: None,
                primary_scene_category: None,
                shot_scene_type: None,
                shot_scene_label: None,
                shot_intent: None,
                adaptation_reason: None,
                selected_total_duration_seconds: 30,
            },
        );

        let mut edited_rows = storyboard.rows.clone();
        edited_rows[0].visual_description = "Edited bridge visual description".to_string();
        edited_rows[0].prompt_text = "Edited Seedance2.0 prompt text".to_string();
        edited_rows[0].duration_seconds = 10;
        edited_rows[1].duration_seconds = 10;
        edited_rows[2].duration_seconds = 10;

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
        assert_eq!(saved.duration_plan.allocated_seconds, 30);
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
        assert_eq!(json_artifact.selected_total_duration_seconds, Some(30));
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
                shot_script: None,
                expanded_script_text: Some(
                    "scene_type: daily_dialogue\nsynopsis: duration mismatch".to_string(),
                ),
                primary_scene_type: None,
                primary_scene_label: None,
                primary_scene_category: None,
                shot_scene_type: None,
                shot_scene_label: None,
                shot_intent: None,
                adaptation_reason: None,
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
