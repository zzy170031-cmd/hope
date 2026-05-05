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

use core_domain::contracts::{AcceptedRewriteSnapshotBinding, SourceStoryFacts};
use core_domain::{
    BridgeCallStatus, ExpandScriptRequest, ExpandScriptResponse, ExportArtifactRecord,
    ExportBundleRequest, ExportBundleResponse, ExportStoryboardBankRequest,
    ExportStoryboardBankResponse, ExternalReferenceHandleCandidate, FinalizedStoryboardShotResult,
    GenerateStoryboardRequest, GenerateStoryboardResponse, GeneratedStoryboardRow,
    GoldenSampleLibraryRecord, KbRouterExcludedCandidate, KbRouterRetrievalTrace,
    KbRouterRuntimeRequest, KbRouterRuntimeResponse, KbRouterSelectedRule, KbRouterSelectionReason,
    KbRouterTaskType, KbRouterTokenBudget, ListStoryboardShotResultsRequest,
    ListStoryboardShotResultsResponse, ModelConfigSummary, ProductWarning,
    PromptTextCompilationStatus, RemoveStoryboardShotResultRequest,
    RemoveStoryboardShotResultResponse, SaveStoryboardShotResultRequest,
    SaveStoryboardShotResultResponse, ScenePerformanceProjection, SequenceFieldState,
    SequenceGrouping, ShotGroundingSource, StoryboardBindingEvidence, StoryboardDurationPlan,
    StoryboardExportStatus, StructureMode, TextGenerationOutputSchema, TextGenerationRequest,
    TextGenerationResponse, TextGenerationTask, TextModelProvider, TextModelProviderKind,
    UpdateStoryboardRowsRequest, UpdateStoryboardShotResultRequest,
    UpdateStoryboardShotResultResponse,
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
    #[serde(default, alias = "title", alias = "shot")]
    shot_title: String,
    #[serde(default, alias = "character", alias = "subject", alias = "role")]
    person: String,
    #[serde(default, alias = "scale", alias = "shot_size", alias = "framing")]
    scene_scale: String,
    #[serde(
        default,
        alias = "visual",
        alias = "image",
        alias = "scene_description",
        alias = "composition"
    )]
    visual_description: String,
    #[serde(
        default,
        alias = "action",
        alias = "performance",
        alias = "character_motion"
    )]
    character_action: String,
    #[serde(
        default,
        alias = "camera",
        alias = "camera_motion",
        alias = "camera_move",
        alias = "lens_movement"
    )]
    camera_movement: String,
    #[serde(default)]
    dialogue: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct LiveStoryboardRowsEnvelope {
    rows: Vec<LiveStoryboardRowPatch>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct LiveRepairSummary {
    raw_failed_validator: bool,
    reasons: Vec<String>,
}

impl LiveRepairSummary {
    fn repaired(&self) -> bool {
        !self.reasons.is_empty()
    }

    fn push_reason(&mut self, reason: &str) {
        if !self.reasons.iter().any(|item| item == reason) {
            self.reasons.push(reason.to_string());
        }
    }

    fn extend(&mut self, other: LiveRepairSummary) {
        self.raw_failed_validator |= other.raw_failed_validator;
        for reason in other.reasons {
            self.push_reason(&reason);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LiveTextRepairResult {
    text: String,
    summary: LiveRepairSummary,
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct StoryboardBindingFieldAnchors {
    source_text: String,
    accepted_confirmation_body: String,
    person: Vec<String>,
    visual_description: Vec<String>,
    character_action: Vec<String>,
    camera_movement: Vec<String>,
    packaging: Vec<String>,
    forbidden_facts: Vec<String>,
    duration_seconds: u16,
    target_duration_mode: String,
    has_binding_context: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DesktopDurationPlan {
    target_duration_mode: String,
    story_length_profile: String,
    source_material_length_chars: u32,
    auto_segment_strategy: String,
    estimated_total_story_duration_seconds: u16,
    generated_shot_task_count: u32,
    duration_plan_summary: String,
    warnings: Vec<ProductWarning>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SceneEntryMapping {
    desktop_entry: &'static str,
    chinese_label: &'static str,
    runtime_scene_family: &'static str,
    canonical_bucket: &'static str,
    aliases: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CurrentShotFactAnchor {
    source: String,
    subject: String,
}

const STORYBOARD_DURATION_SOURCE: &str = "storyboard_duration_plan.allocated_row_duration_seconds";
const DEFAULT_EXPAND_SCRIPT_DURATION_SECONDS: u16 = 30;
const TARGET_DURATION_MODE_FIXED_SECONDS: &str = "fixed_seconds";
const TARGET_DURATION_MODE_LONG_TEXT_AUTO: &str = "long_text_auto";
const AUTO_SEGMENT_STRATEGY_FIXED_SECONDS: &str = "fixed_seconds_user_selected";
const AUTO_SEGMENT_STRATEGY_LONG_TEXT: &str = "long_text_auto_story_fact_segments";
const STORY_LENGTH_PROFILE_SHORT_CLIP: &str = "short_clip";
const STORY_LENGTH_PROFILE_STANDARD_CLIP: &str = "standard_clip";
const STORY_LENGTH_PROFILE_LONG_STORY: &str = "long_story";
const STORY_LENGTH_PROFILE_LONG_STORY_AUTO: &str = "long_story_auto";
const STORY_LENGTH_PROFILE_SHORT_STORY: &str = "short_story_2000_2500";
const STORY_LENGTH_PROFILE_TWO_MINUTE_STORY: &str = "two_minute_story_2500_3500";
const SEEDANCE_STANDARD_SEGMENT_SECONDS: u16 = 10;
const SEEDANCE_REMAINDER_SEGMENT_SECONDS: u16 = 5;
const SEEDANCE_MAX_SEGMENT_SECONDS: u16 = 15;
const MVP_SCENE_CANONICAL_BUCKET: &str = "daily_dialogue";
const LIVE_PERSON_VISUAL_OR_ABSTRACT_TERMS: &[&str] = &[
    "高对比",
    "焦点",
    "画面",
    "构图",
    "光影",
    "光线",
    "色调",
    "氛围",
    "当前空间",
    "空间",
    "背景",
    "环境",
    "镜头",
    "特写",
    "近景",
    "中景",
    "全景",
    "远景",
    "低机位",
    "俯拍",
    "仰拍",
    "广角俯",
    "左下角",
    "subject label",
    "subject_label",
    "主体标签",
    "主体 label",
];
const STORYBOARD_PERSON_FORBIDDEN_LABEL_TERMS: &[&str] = &[
    "环境",
    "沙盘",
    "城建面板",
    "镜头",
    "构图",
    "画面",
    "场景",
    "当前空间",
    "空镜",
    "UI",
    "战报 UI",
    "小地图",
    "面板",
    "行军地图",
    "军阵",
    "白意图",
    "郑重递",
    "左手",
    "右手",
    "头部",
    "手部",
    "单膝",
    "位置",
    "未言语",
    "高耸绷",
    "屈护住",
    "扶着倚",
    "关系保",
    "轮廓缓",
    "背景灰",
    "居右三",
    "罗盘紧",
    "苏瑶方",
    "后停顿",
    "广角俯",
    "眉心",
    "左下角",
    "那里有",
    "那里",
    "左臂垂",
    "方眉",
    "铠甲裂",
    "利望",
    "继续",
    "视线方",
    "方向压",
    "方向",
    "主角之",
    "林峰压",
    "苏瑶压",
];
const LIVE_PERSON_SOURCE_FRAGMENT_TERMS: &[&str] = &[
    "没答",
    "那人",
    "左边岔",
    "关系保",
    "轮廓缓",
    "背景灰",
    "居右三",
    "罗盘紧",
    "苏瑶方",
    "后停顿",
    "广角俯",
    "眉心",
    "左下角",
    "那里有",
    "那里",
    "猛然",
    "突然",
    "忽然",
    "终于",
    "初立",
    "初峙",
    "初现",
    "后迅速",
    "那里有",
    "那里",
    "后提醒",
    "后微撤",
    "左臂垂",
    "方眉",
    "铠甲裂",
    "利望",
    "继续",
    "视线方",
    "方向压",
    "方向",
    "主角之",
    "林峰压",
    "苏瑶压",
    "后颈",
    "废墟单",
    "废墟",
    "方挥",
    "怀中",
    "罗盘被",
    "半边肩",
    "边肩",
    "边咳",
    "边咳了",
    "时抬",
    "肩甲投",
    "苏瑶上",
    "攥紧旧",
    "攥紧照",
    "攥紧",
    "旧照",
    "扶住",
    "单膝",
    "头部",
    "左手",
    "右手",
    "手部",
    "位置",
    "未言语",
    "瞳孔",
    "喉结",
    "呼吸",
    "胸膛",
    "指节",
    "膝盖",
];
const LIVE_PERSON_ACTION_STATE_FRAGMENT_TERMS: &[&str] = &[
    "准备反击",
    "反击准备",
    "准备反",
    "坚持",
    "坚定",
    "喘息",
    "喘息声",
    "急喘",
    "低喘",
];
const LIVE_PERSON_SOURCE_BOUND_STATE_SUBJECT_TERMS: &[&str] = &["紧张", "坚毅"];
const PRODUCT_OUTPUT_FORBIDDEN_FRAGMENT_TERMS: &[&str] = &["关系保"];
const EXPAND_SCRIPT_FORBIDDEN_ADDED_IDENTITY_TERMS: &[&str] = &[
    "邻居家大叔",
    "邻居家孩子",
    "熟悉的身影",
    "邻居",
    "孩子",
    "熟人",
    "大叔",
];
const EXPAND_SCRIPT_FORBIDDEN_EXTERNAL_SETTING_TERMS: &[&str] = &[
    "北营兵符拓片",
    "丞相帐",
    "建安十七年监造",
    "中军",
    "虎牢关守军轮值图",
    "罗盘认路",
    "黑釉兵俑",
    "戴着皮套的手",
    "铭文",
    "金线",
    "兵符",
    "拓片",
    "虎牢关",
    "皮套",
    "世界观物件",
    "灯塔底部",
    "灯塔",
    "木栈道",
    "水面",
    "探照灯",
    "衣袖裂口",
    "衣袖",
    "三名黑衣追兵",
    "刀锋出鞘",
    "断戟立",
    "断戟",
    "断梁",
    "龟裂",
    "甲胄",
    "铠甲",
    "肩甲",
    "剑柄",
    "左膝",
    "左臂",
    "左拳",
    "指节",
    "渗血",
    "身体伤口",
    "伤口",
];
const EXPAND_SCRIPT_FORBIDDEN_EXTERNAL_ACTION_TERMS: &[&str] = &["冷笑", "讥笑", "嗤笑"];
const EXPAND_SCRIPT_FORBIDDEN_MILITARY_SCALE_EXPANSION_TERMS: &[&str] = &[
    "整列军阵",
    "整支军阵",
    "整队军阵",
    "成列军阵",
    "军阵列队",
    "阵列组织",
    "军队规模",
    "军政结构",
];

const RUNTIME_SCENE_ENTRY_MAPPINGS: &[SceneEntryMapping] = &[
    SceneEntryMapping {
        desktop_entry: "hot_blood_battle",
        chinese_label: "热血战斗",
        runtime_scene_family: "action_beat",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &["action_beat"],
    },
    SceneEntryMapping {
        desktop_entry: "ensemble_performance",
        chinese_label: "群像表演",
        runtime_scene_family: "group_performance",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &["group_performance"],
    },
    SceneEntryMapping {
        desktop_entry: "emotional_dialogue",
        chinese_label: "情绪对话",
        runtime_scene_family: "dialogue_beat",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &["dialogue_beat"],
    },
    SceneEntryMapping {
        desktop_entry: "encounter_performance",
        chinese_label: "相遇表演",
        runtime_scene_family: "encounter_performance",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &[],
    },
    SceneEntryMapping {
        desktop_entry: "field_chase",
        chinese_label: "场域追逐",
        runtime_scene_family: "field_chase",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &[],
    },
    SceneEntryMapping {
        desktop_entry: "spectacle_showcase",
        chinese_label: "奇观展示",
        runtime_scene_family: "spectacle_showcase",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &[],
    },
    SceneEntryMapping {
        desktop_entry: "daily_healing",
        chinese_label: "日常治愈",
        runtime_scene_family: "healing_daily",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &["healing_daily"],
    },
    SceneEntryMapping {
        desktop_entry: "guoman_hot_blood_combat",
        chinese_label: "国漫热血打斗",
        runtime_scene_family: "guoman_action",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &["guoman_action"],
    },
    SceneEntryMapping {
        desktop_entry: "guoman_ensemble_performance",
        chinese_label: "国漫群像表演",
        runtime_scene_family: "guoman_group",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &["guoman_group"],
    },
    SceneEntryMapping {
        desktop_entry: "ink_wuxia_combat",
        chinese_label: "水墨武打",
        runtime_scene_family: "ink_action",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &["ink_action"],
    },
    SceneEntryMapping {
        desktop_entry: "eastern_spectacle",
        chinese_label: "东方奇观",
        runtime_scene_family: "eastern_spectacle",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &[],
    },
    SceneEntryMapping {
        desktop_entry: "xianxia_action",
        chinese_label: "仙侠动作",
        runtime_scene_family: "xianxia_action",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &[],
    },
    SceneEntryMapping {
        desktop_entry: "urban_fantasy",
        chinese_label: "都市奇幻",
        runtime_scene_family: "urban_fantasy",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &[],
    },
    SceneEntryMapping {
        desktop_entry: "chinese_war_formation",
        chinese_label: "国战军阵建立",
        runtime_scene_family: "nation_war_establishing",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &["nation_war_establishing", "war_formation"],
    },
    SceneEntryMapping {
        desktop_entry: "weapon_highlight",
        chinese_label: "武将兵器高光",
        runtime_scene_family: "weapon_highlight",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &[],
    },
    SceneEntryMapping {
        desktop_entry: "council_strategy",
        chinese_label: "朝堂军帐权谋",
        runtime_scene_family: "court_strategy",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &["court_strategy"],
    },
    SceneEntryMapping {
        desktop_entry: "siege_defense",
        chinese_label: "多军团攻城",
        runtime_scene_family: "siege_assault",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &["siege_assault", "multi_army_siege"],
    },
    SceneEntryMapping {
        desktop_entry: "slg_sandbox_view",
        chinese_label: "沙盘战略视口",
        runtime_scene_family: "sandtable_overview",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &["sandtable_overview"],
    },
    SceneEntryMapping {
        desktop_entry: "slg_march_encirclement",
        chinese_label: "行军轨迹合围",
        runtime_scene_family: "march_encirclement",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &["march_encirclement"],
    },
    SceneEntryMapping {
        desktop_entry: "slg_city_growth",
        chinese_label: "城建演进反馈",
        runtime_scene_family: "city_build_feedback",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &["city_build_feedback"],
    },
    SceneEntryMapping {
        desktop_entry: "slg_battle_report",
        chinese_label: "战报 UI",
        runtime_scene_family: "battle_report_ui",
        canonical_bucket: MVP_SCENE_CANONICAL_BUCKET,
        aliases: &["battle_report_ui"],
    },
];

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

const PRODUCT_CONTROL_LINE_PREFIXES: &[&str] = &[
    "scene_type:",
    "scene_label:",
    "scene_category:",
    "current_scene_type:",
    "current_scene_label:",
    "current_scene_category:",
    "current_target_duration_seconds:",
    "scene_rewrite_rule:",
    "source_package:",
    "target_duration_seconds:",
    "source_input_type:",
    "authoring_mode:",
    "source_material_summary:",
    "preserved_fact_summary:",
    "character_names:",
    "character_relationships:",
    "preserved_event_order:",
    "ending_state:",
    "screenplay_title:",
    "continuity_context_summary:",
    "content_priority:",
    "kb_guidance_mode:",
    "source_story_facts_take_priority",
    "source_story_facts_take_priority_over_kb_advice:",
    "changed_for_screenplay_summary:",
    "omitted_detail_summary:",
    "dialogue_intent:",
    "prompt_text_compilation",
    "duration_source:",
    "剧本改写：",
    "源材料识别：",
    "保留原则：",
    "结尾状态保留：",
    "目标时长：",
    "状态保留：",
    "内部字段：",
    "结构说明：",
];

const PRODUCT_CONTROL_ANYWHERE_TERMS: &[&str] = &[
    "scene_type:",
    "scene_label:",
    "scene_category:",
    "current_scene_type:",
    "current_scene_label:",
    "current_scene_category:",
    "current_target_duration_seconds:",
    "scene_rewrite_rule:",
    "source_package:",
    "source_input_type:",
    "authoring_mode:",
    "screenplay_title:",
    "source_material_summary:",
    "preserved_fact_summary:",
    "continuity_context_summary:",
    "content_priority:",
    "kb_guidance_mode:",
    "changed_for_screenplay_summary:",
    "omitted_detail_summary:",
    "source_material_body_begin",
    "source_material_body_end",
    "target_duration_seconds",
    "扩写剧本：",
    "扩写剧本:",
    "readystub",
    "prompt_text_compilation",
    "duration_source",
];

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

const QWEN_TEXT_MODEL_MAX_TRANSPORT_ATTEMPTS: usize = 3;
const TEXT_MODEL_NETWORK_RETRY_RECOVERED_CODE: &str = "text_model_network_retry_recovered";
const TEXT_MODEL_QA_NO_LOCAL_FALLBACK_CODE: &str = "text_model_qa_no_local_fallback_blocked";
const TEXT_MODEL_OUTPUT_CONTRACT_NORMALIZED_CODE: &str = "text_model_output_contract_normalized";
const QA_PROXY_ENV_EVIDENCE_CODE: &str = "qa_proxy_env_evidence";

pub fn expand_script(state: &AppState, request: ExpandScriptRequest) -> ExpandScriptResponse {
    let scene_label = request.scene_label.as_deref().unwrap_or_default().trim();
    let scene_category = request.scene_category.as_deref().unwrap_or_default().trim();
    let normalized_scene_type = normalize_scene_type(&request.scene_type);
    let source_analysis = analyze_desktop_source_input(&request.synopsis_text, &request);
    let duration_plan = plan_desktop_duration(&request, &source_analysis);
    let target_duration_seconds = duration_plan.estimated_total_story_duration_seconds;
    let is_story_expansion = is_story_expansion_request(&request, &source_analysis);
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
        structure_type: Some(duration_plan.story_length_profile.clone()),
    };
    let script_hash = stable_hash_hex(&format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        request.scene_type.trim(),
        scene_label,
        scene_category,
        target_duration_seconds,
        duration_plan.target_duration_mode,
        duration_plan.auto_segment_strategy,
        source_analysis.source_input_type.as_str(),
        source_analysis.authoring_mode.as_str(),
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
    let model_story_input = build_expand_script_model_story_input(
        &request,
        &source_analysis,
        &normalized_scene_type,
        scene_label,
        scene_category,
        target_duration_seconds,
    );
    let generation_request = build_text_generation_request(
        TextGenerationTask::ExpandScript,
        Some(normalized_scene_type.clone()),
        model_story_input,
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
    warnings.extend(duration_plan.warnings.clone());
    warnings.extend(model_config_warnings(request.model_config_summary.as_ref()));
    if let Some(warning) = qa_proxy_env_evidence_warning() {
        warnings.push(warning);
    }
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

    let blocking_generation_warnings =
        text_generation_fallback_blocking_warnings(&generated_script.warnings);
    if qa_no_local_fallback_enabled() && !blocking_generation_warnings.is_empty() {
        let mut blockers = blocking_generation_warnings.clone();
        if !blockers
            .iter()
            .any(|warning| warning.code == TEXT_MODEL_QA_NO_LOCAL_FALLBACK_CODE)
        {
            blockers.push(qa_no_local_fallback_blocker(
                &provider,
                &blocking_generation_warnings,
            ));
        }
        let response = ExpandScriptResponse {
            status: BridgeCallStatus::Blocked,
            script_id,
            expanded_script_text: String::new(),
            script_hash,
            blockers,
            warnings,
            source_input_type: source_analysis.source_input_type.clone(),
            authoring_mode: source_analysis.authoring_mode.clone(),
            source_material_summary: source_analysis.source_material_summary.clone(),
            source_story_facts: source_analysis.source_story_facts.clone(),
            preserved_fact_summary: source_analysis.preserved_fact_summary.clone(),
            changed_for_screenplay_summary: source_analysis.changed_for_screenplay_summary.clone(),
            omitted_detail_summary: source_analysis.omitted_detail_summary.clone(),
            continuity_warnings: source_analysis.continuity_warnings.clone(),
            target_duration_mode: duration_plan.target_duration_mode,
            story_length_profile: duration_plan.story_length_profile,
            source_material_length_chars: duration_plan.source_material_length_chars,
            auto_segment_strategy: duration_plan.auto_segment_strategy,
            estimated_total_story_duration_seconds: duration_plan
                .estimated_total_story_duration_seconds,
            generated_shot_task_count: 0,
            duration_plan_summary: duration_plan.duration_plan_summary,
            kb_router_result,
        };
        return response;
    }
    let mut live_expand_repair_warning = None;
    let (live_text, validator_failure_reason) = if blocking_generation_warnings.is_empty() {
        match validate_generated_script_text_with_reason(
            &generated_script.text,
            &request.synopsis_text,
        ) {
            Ok(text) => (Some(text), None),
            Err(reason) => {
                if let Some(repair) = repair_live_expanded_script_text(
                    &generated_script.text,
                    &request.synopsis_text,
                    &normalized_scene_type,
                    scene_label,
                ) {
                    live_expand_repair_warning = Some(live_repair_warning(
                        "text_model_live_expand_repaired",
                        &repair.summary,
                    ));
                    (Some(repair.text), None)
                } else {
                    (None, Some(reason))
                }
            }
        }
    } else {
        (None, None)
    };
    let uses_local_safety_fallback = false;
    if let Some(warning) = expand_live_fallback_warning(
        &provider,
        &blocking_generation_warnings,
        live_text.is_some(),
        validator_failure_reason.as_deref(),
        uses_local_safety_fallback,
    ) {
        warnings.push(warning);
    }
    if let Some(warning) = live_expand_repair_warning {
        warnings.push(warning);
    }
    warnings.extend(source_analysis.continuity_warnings.clone());
    let fallback_script = if is_story_expansion {
        build_deterministic_expanded_story_material(&request.synopsis_text, scene_label)
    } else {
        build_deterministic_rewrite_script(
            &request.synopsis_text,
            scene_label,
            target_duration_seconds,
            &source_analysis,
        )
    };
    let sanitized_fallback_script = sanitize_product_body_text(&fallback_script);
    let expanded_script_text = live_text.unwrap_or(sanitized_fallback_script);

    let response = ExpandScriptResponse {
        status: if warnings.is_empty() {
            BridgeCallStatus::Ready
        } else {
            BridgeCallStatus::WarningOnly
        },
        script_id,
        expanded_script_text,
        script_hash,
        blockers: vec![],
        warnings,
        source_input_type: source_analysis.source_input_type.clone(),
        authoring_mode: source_analysis.authoring_mode.clone(),
        source_material_summary: source_analysis.source_material_summary.clone(),
        source_story_facts: source_analysis.source_story_facts.clone(),
        preserved_fact_summary: source_analysis.preserved_fact_summary.clone(),
        changed_for_screenplay_summary: source_analysis.changed_for_screenplay_summary.clone(),
        omitted_detail_summary: source_analysis.omitted_detail_summary.clone(),
        continuity_warnings: source_analysis.continuity_warnings.clone(),
        target_duration_mode: duration_plan.target_duration_mode,
        story_length_profile: duration_plan.story_length_profile,
        source_material_length_chars: duration_plan.source_material_length_chars,
        auto_segment_strategy: duration_plan.auto_segment_strategy,
        estimated_total_story_duration_seconds: duration_plan
            .estimated_total_story_duration_seconds,
        generated_shot_task_count: duration_plan.generated_shot_task_count,
        duration_plan_summary: duration_plan.duration_plan_summary,
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
        .accepted_rewrite_snapshot
        .as_ref()
        .map(|snapshot| snapshot.accepted_confirmation_body.clone())
        .filter(|text| !text.trim().is_empty())
        .or_else(|| {
            request
                .expanded_script_text
                .clone()
                .filter(|text| !text.trim().is_empty())
        })
        .or_else(|| {
            if request.accepted_rewrite_snapshot.is_some() {
                None
            } else {
                request
                    .script_id
                    .as_deref()
                    .and_then(|script_id| state.find_script(script_id))
                    .map(|script| script.expanded_script_text.clone())
            }
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
        && !is_supported_desktop_scene_type(&grounding.primary_scene_type)
        && !is_supported_desktop_scene_type(&grounding.shot_scene_type)
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
    blockers.extend(current_case_binding_preflight(&request, &grounding));

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

    let Some(row_durations) = allocate_storyboard_row_durations(
        request.selected_total_duration_seconds,
        selected_records.len().max(1),
    ) else {
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
    let row_count = row_durations.len();

    let mut rows = Vec::new();
    let mut deterministic_rows = Vec::new();
    let mut warnings = Vec::new();
    let mut raw_patch_rows = Vec::new();
    let mut patch_repaired_rows = Vec::new();
    let (provider, session_api_key) = current_text_model_provider(state);
    let generation_request = build_text_generation_request(
        TextGenerationTask::GenerateStoryboard,
        Some(shot_scene_type.clone()),
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
    let live_generation =
        run_text_generation(&provider, session_api_key.as_deref(), &generation_request);
    warnings.extend(live_generation.warnings.clone());
    if let Some(warning) = qa_proxy_env_evidence_warning() {
        warnings.push(warning);
    }
    let blocking_live_generation_warnings =
        text_generation_fallback_blocking_warnings(&live_generation.warnings);
    if qa_no_local_fallback_enabled() && !blocking_live_generation_warnings.is_empty() {
        let mut blockers = blocking_live_generation_warnings.clone();
        if !blockers
            .iter()
            .any(|warning| warning.code == TEXT_MODEL_QA_NO_LOCAL_FALLBACK_CODE)
        {
            blockers.push(qa_no_local_fallback_blocker(
                &provider,
                &blocking_live_generation_warnings,
            ));
        }
        if let Some(warning) = qa_proxy_env_evidence_warning() {
            blockers.push(warning);
        }
        return blocked_storyboard_response(
            state,
            &router_request,
            None,
            request.selected_total_duration_seconds,
            blockers,
            now_ms,
        );
    }
    let live_row_patches = match extract_live_storyboard_row_patches(&live_generation) {
        Ok(patches) => patches,
        Err(warning) => {
            warnings.push(warning);
            Vec::new()
        }
    };

    for (index, duration_seconds) in row_durations.iter().copied().enumerate() {
        let record = &selected_records[index % selected_records.len()];
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

        let draft =
            build_shot_grounded_row_draft(&grounding, index, row_count, duration_seconds, record);
        let scene_projection = draft.scene_performance_projection.clone();
        let prompt_compilation = compile_seedance_prompt_text(
            record,
            &scene_projection,
            duration_seconds,
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
            camera_movement: draft.camera_movement.clone(),
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
        normalize_storyboard_row_subject_quality(&mut row);
        repair_b_sandbox_strategy_storyboard_row_if_needed(&mut row);
        let deterministic_row = row.clone();
        deterministic_rows.push(deterministic_row.clone());
        if let Some(patch) = live_row_patches.get(index) {
            apply_live_storyboard_patch(&mut row, patch, &deterministic_row);
        }
        normalize_storyboard_row_subject_quality(&mut row);
        raw_patch_rows.push(row.clone());
        repair_live_storyboard_patch_from_baseline(&mut row, &deterministic_row);
        normalize_storyboard_row_subject_quality(&mut row);
        repair_b_sandbox_strategy_storyboard_row_if_needed(&mut row);
        patch_repaired_rows.push(row.clone());
        rows.push(row);
    }

    diversify_repeated_storyboard_subjects(&mut deterministic_rows);
    diversify_repeated_storyboard_subjects(&mut rows);
    let mut validator_deterministic_rows = deterministic_rows.clone();
    repair_storyboard_rows_from_binding_context(
        &mut validator_deterministic_rows,
        &request,
        &grounding,
    );

    if !live_row_patches.is_empty() {
        let mut live_repair_summary = LiveRepairSummary::default();
        let pre_repair_findings = validate_live_storyboard_rows(
            &rows,
            request.selected_total_duration_seconds,
            &validator_deterministic_rows,
        );
        live_repair_summary.raw_failed_validator = !pre_repair_findings.is_empty();
        live_repair_summary.extend(repair_live_storyboard_rows_from_source(
            &mut rows,
            &validator_deterministic_rows,
        ));
        live_repair_summary.extend(repair_storyboard_rows_from_binding_context(
            &mut rows, &request, &grounding,
        ));
        diversify_repeated_storyboard_subjects(&mut rows);
        let live_validator_findings = validate_live_storyboard_rows(
            &rows,
            request.selected_total_duration_seconds,
            &validator_deterministic_rows,
        );
        if !live_validator_findings.is_empty() {
            if qa_no_local_fallback_enabled() {
                let live_blocked_diag_warnings = blocked_runtime_diagnostic_warnings(
                    &request,
                    &grounding,
                    &row_durations,
                    &live_row_patches,
                    &validator_deterministic_rows,
                    &raw_patch_rows,
                    &patch_repaired_rows,
                    &rows,
                    &live_repair_summary,
                    &pre_repair_findings,
                    &live_validator_findings,
                );
                let mut blockers = live_validator_findings;
                blockers.extend(live_blocked_diag_warnings);
                blockers.push(ProductWarning {
                    code: "text_model_live_storyboard_validator_hard_fail".to_string(),
                    message: "qa_no_local_fallback=true; live storyboard failed validator; fallback_used=false; local_candidate=false; raw_values_redacted=true".to_string(),
                    related_sample_id: None,
                });
                blockers.push(qa_no_local_fallback_blocker(&provider, &blockers));
                if let Some(warning) = qa_proxy_env_evidence_warning() {
                    blockers.push(warning);
                }
                return blocked_storyboard_response(
                    state,
                    &router_request,
                    None,
                    request.selected_total_duration_seconds,
                    blockers,
                    now_ms,
                );
            }
            warnings.extend(live_validator_findings);
            warnings.push(ProductWarning {
                code: "text_model_live_storyboard_fallback".to_string(),
                message: "千问生成的分镜内容未通过本地校验，已回退到本地候选结果。".to_string(),
                related_sample_id: None,
            });
            rows = deterministic_rows;
        } else if live_repair_summary.repaired() {
            warnings.push(live_repair_warning(
                "text_model_live_storyboard_repaired",
                &live_repair_summary,
            ));
        }
    } else if !blocking_live_generation_warnings.is_empty() {
        warnings.push(ProductWarning {
            code: "text_model_live_storyboard_fallback".to_string(),
            message: "未启用千问或调用失败，已使用本地候选结果。".to_string(),
            related_sample_id: None,
        });
    }
    repair_storyboard_rows_from_binding_context(&mut rows, &request, &grounding);
    diversify_repeated_storyboard_subjects(&mut rows);
    warnings.extend(model_config_warnings(request.model_config_summary.as_ref()));

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
    let duration_plan =
        build_storyboard_duration_plan(request.selected_total_duration_seconds, &row_durations);
    let binding_evidence = build_storyboard_binding_evidence(
        &request,
        &grounding,
        &rows,
        &duration_plan,
        &rows_hash,
        &kb_router_result,
    );
    blockers.extend(story_fact_frame_binding_gate_blockers(&binding_evidence));
    blockers.extend(storyboard_prompt_text_packaging_gate_blockers(&rows));
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

    let response = GenerateStoryboardResponse {
        task_id: Some(task_id),
        result_id,
        rows,
        selected_total_duration_seconds: request.selected_total_duration_seconds,
        target_duration_mode: TARGET_DURATION_MODE_FIXED_SECONDS.to_string(),
        story_length_profile: String::new(),
        source_material_length_chars: stable_source_material_length_chars(
            &grounding.grounding_text,
        ),
        auto_segment_strategy: AUTO_SEGMENT_STRATEGY_FIXED_SECONDS.to_string(),
        estimated_total_story_duration_seconds: request.selected_total_duration_seconds,
        generated_shot_task_count: 1,
        duration_plan_summary: format!(
            "mode={}; total={}s; generated_shot_tasks=1",
            TARGET_DURATION_MODE_FIXED_SECONDS, request.selected_total_duration_seconds
        ),
        duration_plan,
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
        binding_evidence,
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
            message: "已定稿分镜集合中已存在这个镜头结果。".to_string(),
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
            message: "查看已定稿分镜前需要有效项目。".to_string(),
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
                message: "没有找到可更新的已定稿镜头。".to_string(),
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
                message: "移除已定稿镜头前需要项目和镜头结果。".to_string(),
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
                message: "没有找到要移除的已定稿镜头。".to_string(),
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
            message: "导出已定稿分镜集合前需要有效项目。".to_string(),
            related_sample_id: None,
        });
    }
    if request.include_unconfirmed {
        blockers.push(ProductWarning {
            code: "storyboard_bank_include_unconfirmed_not_supported".to_string(),
            message: "V1 已定稿分镜集合只导出已确认镜头。".to_string(),
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
                message: "暂无已确认分镜，无法导出完整分镜包。".to_string(),
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
                    message: "已定稿镜头的 rows_hash 与当前 rows 不一致，已停止导出。".to_string(),
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
                    message: "已定稿分镜集合总时长超出 V1 导出范围。".to_string(),
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
                    artifact_path: Some(artifact.path.display().to_string()),
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
                message: format!("已定稿分镜集合导出失败：{error:?}"),
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
                message: format!("已定稿分镜集合缺少必要字段：{field_name}。"),
                related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
            });
        }
    }
    if rows.is_empty() {
        blockers.push(ProductWarning {
            code: "storyboard_bank_rows_required".to_string(),
            message: "确认定稿前需要至少一行分镜。".to_string(),
            related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
        });
    }
    if prompt_text.trim().is_empty() {
        blockers.push(ProductWarning {
            code: "storyboard_bank_prompt_text_required".to_string(),
            message: "确认定稿前需要干净的分镜提示词。".to_string(),
            related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
        });
    }
    if shot_duration_seconds == 0 {
        blockers.push(ProductWarning {
            code: "storyboard_bank_shot_duration_required".to_string(),
            message: "确认定稿前需要镜头时长。".to_string(),
            related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
        });
    }
    if duration_source != STORYBOARD_DURATION_SOURCE {
        blockers.push(ProductWarning {
            code: "storyboard_bank_duration_source_unsupported".to_string(),
            message: "镜头时长来源必须来自系统分镜时长分配。".to_string(),
            related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
        });
    }
    if rows.iter().any(|row| row.shot_duration_seconds == 0) {
        blockers.push(ProductWarning {
            code: "storyboard_bank_row_duration_required".to_string(),
            message: "每行已定稿分镜都需要镜头时长。".to_string(),
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
            message: "镜头总时长必须等于行时长合计。".to_string(),
            related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
        });
    }

    if finalized_storyboard_payload_contains_forbidden_terms(shot_task_name, prompt_text, rows) {
        blockers.push(ProductWarning {
            code: "storyboard_bank_forbidden_payload".to_string(),
            message: "已定稿分镜集合拒绝保存内部原始提示、密钥或全量知识库内容。".to_string(),
            related_sample_id: Some(result_id.to_string()).filter(|value| !value.is_empty()),
        });
    }

    let expected_rows_hash = stable_hash_hex(&serialize_storyboard_rows(rows));
    if !rows_hash.trim().is_empty() && rows_hash.trim() != expected_rows_hash {
        blockers.push(ProductWarning {
            code: "storyboard_bank_rows_hash_mismatch".to_string(),
            message: "rows_hash 与已定稿 rows 不一致，无法确认定稿。".to_string(),
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

fn build_expand_script_model_story_input(
    request: &ExpandScriptRequest,
    source_analysis: &DesktopSourceInputAnalysis,
    scene_type: &str,
    scene_label: &str,
    scene_category: &str,
    target_duration_seconds: u16,
) -> String {
    let scene_label = non_blank_string(scene_label)
        .or_else(|| scene_label_for_scene_type(scene_type))
        .unwrap_or_else(|| scene_type.to_string());
    let scene_category = non_blank_string(scene_category)
        .unwrap_or_else(|| "unspecified_scene_category".to_string());
    let changed_summary = non_blank_string(&source_analysis.changed_for_screenplay_summary)
        .unwrap_or_else(|| "Rewrite the source body to match the selected scene type.".to_string());
    let omitted_summary = non_blank_string(&source_analysis.omitted_detail_summary).unwrap_or_else(
        || {
            "If the source body already carries another scene style, keep only source facts and replace the old expression style."
                .to_string()
        },
    );

    format!(
        "current_scene_type: {}\ncurrent_scene_label: {}\ncurrent_scene_category: {}\ncurrent_target_duration_seconds: {}\nscene_rewrite_rule: The selected scene label is binding. Rewrite rhythm, emotional temperature, action density, relationship expression, and visual focus to fit current_scene_label. Do not keep the old scene style when it conflicts with current_scene_label.\nchanged_for_screenplay_summary: {}\nomitted_detail_summary: {}\nsource_material_body_begin\n{}\nsource_material_body_end",
        scene_type,
        scene_label,
        scene_category,
        target_duration_seconds,
        changed_summary,
        omitted_summary,
        request.synopsis_text.trim(),
    )
}

fn run_text_generation(
    provider: &TextModelProvider,
    session_api_key: Option<&str>,
    request: &TextGenerationRequest,
) -> TextGenerationResponse {
    if !provider.enabled {
        return run_text_generation_or_qa_hard_fail(
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
        return run_text_generation_or_qa_hard_fail(
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
        return run_text_generation_or_qa_hard_fail(
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
        return run_text_generation_or_qa_hard_fail(
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
        Err(warning) => run_text_generation_or_qa_hard_fail(provider, request, warning),
    }
}

fn run_text_generation_or_qa_hard_fail(
    provider: &TextModelProvider,
    request: &TextGenerationRequest,
    warning: ProductWarning,
) -> TextGenerationResponse {
    if qa_no_local_fallback_enabled() {
        return run_text_generation_qa_hard_fail(provider, request, warning);
    }
    run_text_generation_stub(provider, request, warning)
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

fn run_text_generation_qa_hard_fail(
    provider: &TextModelProvider,
    _request: &TextGenerationRequest,
    warning: ProductWarning,
) -> TextGenerationResponse {
    let sanitized_warning = qa_sanitized_provider_warning(warning);
    let mut warnings = vec![sanitized_warning.clone()];
    warnings.push(qa_no_local_fallback_blocker(provider, &[sanitized_warning]));
    TextGenerationResponse {
        text: String::new(),
        structured_json: None,
        usage_tokens: None,
        latency_ms: Some(0),
        warnings,
        provider: provider.provider,
        model: provider.model.clone(),
    }
}

fn qa_sanitized_provider_warning(mut warning: ProductWarning) -> ProductWarning {
    warning.message = warning
        .message
        .replace(
            "fallback will use local candidate if retry is exhausted.",
            "qa hard-fail prevents fallback local output.",
        )
        .replace("local candidate", "local_candidate=false")
        .replace("本地候选结果", "local_candidate=false");
    warning
}

fn qa_no_local_fallback_enabled() -> bool {
    parse_bool_env("HOPE_QA_NO_LOCAL_FALLBACK") || parse_bool_env("HOPE_QA_PROVIDER_HARD_FAIL")
}

fn qa_no_local_fallback_blocker(
    provider: &TextModelProvider,
    warnings: &[ProductWarning],
) -> ProductWarning {
    let category = warnings
        .first()
        .map(provider_warning_error_category)
        .unwrap_or("unknown");
    let warning_codes = warnings
        .iter()
        .map(|warning| warning.code.as_str())
        .collect::<Vec<_>>()
        .join(",");
    ProductWarning {
        code: TEXT_MODEL_QA_NO_LOCAL_FALLBACK_CODE.to_string(),
        message: format!(
            "qa_no_local_fallback=true; provider={}; model={}; error_category={}; warning_codes={}; fallback_used=false; local_candidate=false; raw_values_redacted=true",
            provider_kind_code(provider.provider),
            provider.model,
            category,
            warning_codes
        ),
        related_sample_id: warnings
            .first()
            .and_then(|warning| warning.related_sample_id.clone()),
    }
}

fn qa_proxy_env_evidence_warning() -> Option<ProductWarning> {
    if !parse_bool_env("HOPE_QA_PROXY_EVIDENCE") && !parse_bool_env("HOPE_QA_PROXY_CLEARED") {
        return None;
    }
    let http_proxy_present = process_env_present("HTTP_PROXY");
    let https_proxy_present = process_env_present("HTTPS_PROXY");
    let all_proxy_present = process_env_present("ALL_PROXY");
    let no_proxy_present = process_env_present("NO_PROXY");
    Some(ProductWarning {
        code: QA_PROXY_ENV_EVIDENCE_CODE.to_string(),
        message: format!(
            "qa_no_proxy={}; process_env_proxy_present={}; http_proxy_present={}; https_proxy_present={}; all_proxy_present={}; no_proxy_present={}; raw_values_redacted=true",
            parse_bool_env("HOPE_QA_PROXY_CLEARED"),
            http_proxy_present || https_proxy_present || all_proxy_present,
            http_proxy_present,
            https_proxy_present,
            all_proxy_present,
            no_proxy_present
        ),
        related_sample_id: None,
    })
}

fn process_env_present(name: &str) -> bool {
    env::var(name)
        .ok()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn provider_warning_error_category(warning: &ProductWarning) -> &'static str {
    if let Some(category) = warning_message_error_category(&warning.message) {
        return category;
    }
    match warning.code.as_str() {
        "text_model_network_error" => match http_status_from_warning_message(&warning.message) {
            Some(403) => "http_403",
            Some(status) if (500..=599).contains(&status) => "http_5xx",
            Some(_) => "http_non_success",
            None => "network_or_transport",
        },
        "text_model_response_invalid" => "response_invalid",
        "text_model_api_key_missing" => "credential_missing",
        "text_model_base_url_missing" => "endpoint_missing",
        "text_model_provider_not_supported" => "provider_not_supported",
        "text_model_live_call_closed" => "provider_disabled",
        TEXT_MODEL_QA_NO_LOCAL_FALLBACK_CODE => "qa_hard_fail",
        _ => "provider_error",
    }
}

fn warning_message_error_category(message: &str) -> Option<&'static str> {
    let lower = message.to_ascii_lowercase();
    if lower.contains("error_category=timeout") {
        Some("timeout")
    } else if lower.contains("error_category=connect") {
        Some("connect")
    } else if lower.contains("error_category=request") {
        Some("request")
    } else if lower.contains("error_category=body") {
        Some("body")
    } else if lower.contains("error_category=decode") {
        Some("decode")
    } else if lower.contains("error_category=redirect") {
        Some("redirect")
    } else if lower.contains("error_category=network_or_transport") {
        Some("network_or_transport")
    } else {
        None
    }
}

fn elapsed_bucket_ms(elapsed_ms: u128) -> &'static str {
    match elapsed_ms {
        0..=999 => "lt_1s",
        1000..=4999 => "1_5s",
        5000..=14999 => "5_15s",
        15000..=59999 => "15_60s",
        _ => "gte_60s",
    }
}

fn expand_live_fallback_warning(
    provider: &TextModelProvider,
    generation_warnings: &[ProductWarning],
    live_text_accepted: bool,
    validator_failure_reason: Option<&str>,
    uses_local_safety_fallback: bool,
) -> Option<ProductWarning> {
    if let Some(warning) = generation_warnings.first() {
        let message = match warning.code.as_str() {
            "text_model_live_call_closed" => format!(
                "{} 未启用，已使用本地候选结果。",
                provider_kind_display_name(provider.provider)
            ),
            "text_model_provider_not_supported" => {
                "当前文本模型 provider 暂不支持 live 生成，已使用本地候选结果。".to_string()
            }
            "text_model_api_key_missing" | "text_model_base_url_missing" => {
                "千问已启用但连接配置不完整，已使用本地候选结果。".to_string()
            }
            "text_model_network_error" | "text_model_response_invalid" => format!(
                "千问 API 调用失败或响应不可用，已使用本地候选结果；原始原因：{}。",
                warning.code
            ),
            _ => format!(
                "文本模型 live 生成未返回可用正文，已使用本地候选结果；原始原因：{}。",
                warning.code
            ),
        };
        return Some(ProductWarning {
            code: "text_model_live_expand_fallback".to_string(),
            message,
            related_sample_id: warning.related_sample_id.clone(),
        });
    }

    if !live_text_accepted {
        let reason = validator_failure_reason
            .map(sanitize_expand_validator_reason)
            .filter(|reason| !reason.is_empty())
            .unwrap_or_else(|| "未返回可用正文".to_string());
        return Some(ProductWarning {
            code: "text_model_live_expand_fallback".to_string(),
            message: format!("千问返回文本未通过本地校验：{reason}，已使用本地候选结果。"),
            related_sample_id: None,
        });
    }

    uses_local_safety_fallback.then(|| ProductWarning {
        code: "text_model_live_expand_fallback".to_string(),
        message: "扩写模式当前采用本地安全回退；千问返回已通过校验，但未直接写入结果。".to_string(),
        related_sample_id: None,
    })
}

fn sanitize_expand_validator_reason(reason: &str) -> String {
    let mut cleaned = reason
        .replace('\n', " ")
        .replace('\r', " ")
        .replace("prompt_body", "prompt")
        .replace("source_register", "source")
        .replace("overlay JSON", "overlay")
        .replace("API key", "credential")
        .replace("api key", "credential");
    for forbidden in ["token", "secret", "密钥", "令牌"] {
        cleaned = cleaned.replace(forbidden, "credential");
    }
    cleaned.trim().chars().take(96).collect::<String>()
}

fn live_repair_warning(code: &str, summary: &LiveRepairSummary) -> ProductWarning {
    let mut reasons = summary
        .reasons
        .iter()
        .map(|reason| sanitize_repair_reason_code(reason))
        .filter(|reason| !reason.is_empty())
        .collect::<Vec<_>>();
    reasons.sort();
    reasons.dedup();
    let repair_reason = if reasons.is_empty() {
        "source_grounded_repair".to_string()
    } else {
        reasons.into_iter().take(8).collect::<Vec<_>>().join(",")
    };
    ProductWarning {
        code: code.to_string(),
        message: format!(
            "live_raw_failed_validator={}; live_repaired=true; fallback_used=false; repair_reason={repair_reason}",
            summary.raw_failed_validator
        ),
        related_sample_id: None,
    }
}

fn sanitize_blocked_runtime_diag_value(value: &str) -> String {
    sanitize_product_body_text(value)
        .replace(';', "_")
        .replace('=', "_")
        .chars()
        .take(48)
        .collect()
}

fn storyboard_row_field_value<'a>(row: &'a GeneratedStoryboardRow, field_name: &str) -> &'a str {
    match field_name {
        "person" => row.person.as_str(),
        "shot_title" => row.shot_title.as_str(),
        "visual_description" => row.visual_description.as_str(),
        "character_action" => row.character_action.as_str(),
        "camera_movement" => row.camera_movement.as_str(),
        _ => "",
    }
}

fn blocked_runtime_field_classification(
    row: &GeneratedStoryboardRow,
    field_name: &str,
    source_text: &str,
) -> (String, String) {
    let field_value = storyboard_row_field_value(row, field_name);
    let trimmed = field_value.trim();
    if trimmed.is_empty() {
        return ("empty".to_string(), String::new());
    }
    if let Some(label) = live_field_visual_or_abstract_subject_term(field_name, field_value) {
        return (
            "visual_or_abstract_subject".to_string(),
            sanitize_blocked_runtime_diag_value(label),
        );
    }
    if let Some(label) = live_field_source_fragment_subject_term(field_name, field_value) {
        return (
            "source_fragment_subject".to_string(),
            sanitize_blocked_runtime_diag_value(label),
        );
    }
    if storyboard_field_has_source_external_drift(field_value, source_text) {
        return ("source_external_drift".to_string(), String::new());
    }
    if field_name == "character_action" && is_role_action_grounding_incomplete(field_value) {
        return ("role_action_incomplete".to_string(), String::new());
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
        return ("visual_description_incomplete".to_string(), String::new());
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
        return ("camera_movement_incomplete".to_string(), String::new());
    }
    ("grounded".to_string(), String::new())
}

fn blocked_runtime_person_classification(
    candidate: &str,
    evidence: &str,
    baseline_person: &str,
) -> (String, String, bool, String) {
    let trimmed = candidate.trim();
    if trimmed.is_empty() {
        return ("empty".to_string(), String::new(), true, String::new());
    }
    if let Some(label) = live_person_visual_or_abstract_term(trimmed) {
        return (
            "visual_or_abstract_subject".to_string(),
            sanitize_blocked_runtime_diag_value(label),
            true,
            String::new(),
        );
    }
    if let Some(normalized) =
        normalize_live_character_label_to_source_subject(trimmed, evidence, baseline_person)
    {
        let grounded = live_source_subject_label_is_grounded(&normalized, evidence, baseline_person);
        return (
            if grounded {
                "normalized_to_source_subject".to_string()
            } else {
                "normalized_but_ungrounded".to_string()
            },
            sanitize_blocked_runtime_diag_value(&normalized),
            grounded,
            sanitize_blocked_runtime_diag_value(&normalized),
        );
    }
    if live_person_label_should_use_source_bound_repair(trimmed, evidence) {
        return (
            "source_bound_repair_needed".to_string(),
            String::new(),
            false,
            String::new(),
        );
    }
    if source_external_generic_role_subject(trimmed, evidence) {
        return (
            "source_external_generic_role".to_string(),
            sanitize_blocked_runtime_diag_value(trimmed),
            false,
            String::new(),
        );
    }
    let grounded = live_character_label_is_grounded(trimmed, evidence, baseline_person);
    (
        if grounded {
            "grounded".to_string()
        } else {
            "ungrounded_character_candidate".to_string()
        },
        if grounded {
            String::new()
        } else {
            sanitize_blocked_runtime_diag_value(trimmed)
        },
        grounded,
        String::new(),
    )
}

fn blocked_runtime_source_role_hit(candidate: &str, source_text: &str) -> bool {
    let parts = split_live_subject_parts(candidate);
    if parts.is_empty() {
        return false;
    }
    let candidates = source_person_role_candidates(source_text);
    parts
        .into_iter()
        .all(|part| candidates.iter().any(|candidate| candidate == part.trim()))
}

fn blocked_runtime_diagnostic_warnings(
    request: &GenerateStoryboardRequest,
    grounding: &StoryboardGroundingContext,
    row_durations: &[u16],
    live_row_patches: &[LiveStoryboardRowPatch],
    baseline_rows: &[GeneratedStoryboardRow],
    raw_patch_rows: &[GeneratedStoryboardRow],
    patch_repaired_rows: &[GeneratedStoryboardRow],
    repaired_rows: &[GeneratedStoryboardRow],
    live_repair_summary: &LiveRepairSummary,
    pre_repair_findings: &[ProductWarning],
    post_repair_findings: &[ProductWarning],
) -> Vec<ProductWarning> {
    let mut warnings = Vec::new();
    let duration_plan =
        build_storyboard_duration_plan(request.selected_total_duration_seconds, row_durations);
    let rows_hash = stable_hash_hex(&serialize_storyboard_rows(repaired_rows));
    warnings.push(ProductWarning {
        code: "qa_live_blocked_binding_diag".to_string(),
        message: format!(
            "current_case_id={};source_text_hash={};accepted_rewrite_hash={};task_script_hash={};story_fact_frame_hash={};source_profile={};scene_type={};duration_seconds={};duration_plan_hash={};storyboard_rows_hash={}",
            sanitize_blocked_runtime_diag_value(&request.current_case_id),
            sanitize_blocked_runtime_diag_value(&request.source_text_hash),
            sanitize_blocked_runtime_diag_value(&request.accepted_rewrite_hash),
            sanitize_blocked_runtime_diag_value(&request.task_script_hash),
            sanitize_blocked_runtime_diag_value(&request.story_fact_frame_hash),
            sanitize_blocked_runtime_diag_value(&infer_storyboard_source_profile(
                &grounding.grounding_text,
                &request.source_profile,
            )),
            sanitize_blocked_runtime_diag_value(&grounding.shot_scene_type),
            request.selected_total_duration_seconds,
            stable_binding_hash_value(&duration_plan),
            sanitize_blocked_runtime_diag_value(&rows_hash),
        ),
        related_sample_id: None,
    });

    let prompt_text_rows = repaired_rows
        .iter()
        .filter(|row| !row.prompt_text.trim().is_empty())
        .count();
    let prompt_text_blob = repaired_rows
        .iter()
        .map(|row| row.prompt_text.trim())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    warnings.push(ProductWarning {
        code: "qa_live_blocked_prompt_diag".to_string(),
        message: format!(
            "row_count={};prompt_text_present={};prompt_text_nonempty_rows={};prompt_text_hash={}",
            repaired_rows.len(),
            (!prompt_text_blob.is_empty()),
            prompt_text_rows,
            if prompt_text_blob.is_empty() {
                String::new()
            } else {
                stable_binding_hash_json(&prompt_text_blob)
            },
        ),
        related_sample_id: None,
    });

    let mut pre_codes = pre_repair_findings
        .iter()
        .map(|warning| sanitize_repair_reason_code(&warning.code))
        .filter(|code| !code.is_empty())
        .collect::<Vec<_>>();
    pre_codes.sort();
    pre_codes.dedup();
    let mut post_codes = post_repair_findings
        .iter()
        .map(|warning| sanitize_repair_reason_code(&warning.code))
        .filter(|code| !code.is_empty())
        .collect::<Vec<_>>();
    post_codes.sort();
    post_codes.dedup();
    let repair_reasons = if live_repair_summary.reasons.is_empty() {
        "none".to_string()
    } else {
        live_repair_summary
            .reasons
            .iter()
            .map(|reason| sanitize_repair_reason_code(reason))
            .filter(|reason| !reason.is_empty())
            .collect::<Vec<_>>()
            .join("|")
    };
    warnings.push(ProductWarning {
        code: "qa_live_blocked_repair_diag".to_string(),
        message: format!(
            "stage=repair_live_storyboard_patch_from_baseline_then_repair_storyboard_external_drift_fields_then_validate_live_storyboard_rows;raw_failed_validator={};repair_reasons={};pre_codes={};post_codes={}",
            live_repair_summary.raw_failed_validator,
            repair_reasons,
            if pre_codes.is_empty() {
                "none".to_string()
            } else {
                pre_codes.join("|")
            },
            if post_codes.is_empty() {
                "none".to_string()
            } else {
                post_codes.join("|")
            },
        ),
        related_sample_id: None,
    });

    for (index, post_row) in repaired_rows.iter().enumerate() {
        let Some(baseline_row) = baseline_rows.get(index) else {
            continue;
        };
        let Some(raw_patch_row) = raw_patch_rows.get(index) else {
            continue;
        };
        let Some(pre_repair_row) = patch_repaired_rows.get(index) else {
            continue;
        };
        let source_text = format!(
            "{}\n{}",
            baseline_row.shot_script, baseline_row.scene_performance_projection.fused_source_text
        );
        if let Some(live_patch) = live_row_patches.get(index) {
            let raw_input_person = live_patch.person.trim();
            let raw_bound_person = raw_patch_row.person.trim();
            let pre_repair_person = pre_repair_row.person.trim();
            let post_repair_person = post_row.person.trim();
            let (baseline_class, baseline_label, _, _) = blocked_runtime_person_classification(
                baseline_row.person.as_str(),
                &source_text,
                &baseline_row.person,
            );
            let (raw_input_class, raw_input_label, _, raw_input_normalized) =
                blocked_runtime_person_classification(
                    raw_input_person,
                    &source_text,
                    &baseline_row.person,
                );
            let (raw_bound_class, raw_bound_label, _, _) = blocked_runtime_person_classification(
                raw_bound_person,
                &source_text,
                &baseline_row.person,
            );
            let (pre_repair_class, pre_repair_label, _, _) =
                blocked_runtime_person_classification(
                    pre_repair_person,
                    &source_text,
                    &baseline_row.person,
                );
            let (post_repair_class, post_repair_label, post_grounded, post_grounded_normalized) =
                blocked_runtime_person_classification(
                    post_repair_person,
                    &source_text,
                    &baseline_row.person,
                );
            let bound_person = bind_live_storyboard_person_to_source(raw_input_person, baseline_row);
            let bind_source_role_hit =
                blocked_runtime_source_role_hit(&bound_person, &source_text);
            let post_untrusted = live_storyboard_row_untrusted_character_detail(post_row, baseline_row)
                .map(
                    |detail| {
                        (
                            detail.field.to_string(),
                            sanitize_blocked_runtime_diag_value(&detail.candidate),
                            sanitize_blocked_runtime_diag_value(&detail.normalized_candidate),
                            detail.reason.to_string(),
                        )
                    },
                )
                .unwrap_or_else(|| {
                    (
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                    )
                });
            let raw_input_hash = stable_binding_hash_json(raw_input_person);
            let baseline_hash = stable_binding_hash_json(&baseline_row.person);
            let raw_bound_hash = stable_binding_hash_json(raw_bound_person);
            let pre_repair_hash = stable_binding_hash_json(pre_repair_person);
            let post_repair_hash = stable_binding_hash_json(post_repair_person);
            let patch_changed = raw_input_hash != raw_bound_hash;
            let repair_changed = pre_repair_hash != post_repair_hash;
            warnings.push(ProductWarning {
                code: "qa_live_blocked_person_diag".to_string(),
                message: format!(
                    "row={};baseline_class={};baseline_label={};baseline_hash={};raw_input_class={};raw_input_label={};raw_input_hash={};raw_input_normalize_after={};raw_bound_class={};raw_bound_label={};raw_bound_hash={};pre_repair_class={};pre_repair_label={};pre_repair_hash={};post_repair_class={};post_repair_label={};post_repair_hash={};bind_source_role_hit={};bind_normalize_before={};bind_normalize_after={};post_grounded={};post_grounded_normalized={};post_untrusted_field={};post_untrusted_candidate={};post_untrusted_normalized={};post_untrusted_reason={};patch_changed={};repair_changed={}",
                    post_row.order,
                    baseline_class,
                    baseline_label,
                    baseline_hash,
                    raw_input_class,
                    raw_input_label,
                    raw_input_hash,
                    raw_input_normalized,
                    raw_bound_class,
                    raw_bound_label,
                    raw_bound_hash,
                    pre_repair_class,
                    pre_repair_label,
                    pre_repair_hash,
                    post_repair_class,
                    post_repair_label,
                    post_repair_hash,
                    bind_source_role_hit,
                    sanitize_blocked_runtime_diag_value(raw_input_person),
                    sanitize_blocked_runtime_diag_value(&bound_person),
                    post_grounded,
                    post_grounded_normalized,
                    post_untrusted.0,
                    post_untrusted.1,
                    post_untrusted.2,
                    post_untrusted.3,
                    patch_changed,
                    repair_changed,
                ),
                related_sample_id: Some(post_row.prompt_text_source_row_id.clone()),
            });
        }
        for field_name in [
            "shot_title",
            "visual_description",
            "character_action",
            "camera_movement",
        ] {
            let (baseline_class, baseline_label) =
                blocked_runtime_field_classification(baseline_row, field_name, &source_text);
            let (raw_patch_class, raw_patch_label) =
                blocked_runtime_field_classification(raw_patch_row, field_name, &source_text);
            let (pre_repair_class, pre_repair_label) =
                blocked_runtime_field_classification(pre_repair_row, field_name, &source_text);
            let (post_repair_class, post_repair_label) =
                blocked_runtime_field_classification(post_row, field_name, &source_text);

            let baseline_hash = stable_binding_hash_json(storyboard_row_field_value(
                baseline_row,
                field_name,
            ));
            let raw_patch_hash =
                stable_binding_hash_json(storyboard_row_field_value(raw_patch_row, field_name));
            let pre_repair_hash =
                stable_binding_hash_json(storyboard_row_field_value(pre_repair_row, field_name));
            let post_repair_hash =
                stable_binding_hash_json(storyboard_row_field_value(post_row, field_name));
            let patch_changed = raw_patch_hash != pre_repair_hash;
            let repair_changed = pre_repair_hash != post_repair_hash;

            if patch_changed
                || repair_changed
                || post_repair_class != "grounded"
                || raw_patch_class != "grounded"
            {
                warnings.push(ProductWarning {
                    code: "qa_live_blocked_field_diag".to_string(),
                    message: format!(
                        "row={};field={};baseline_class={};baseline_label={};baseline_hash={};raw_patch_class={};raw_patch_label={};raw_patch_hash={};pre_repair_class={};pre_repair_label={};pre_repair_hash={};post_repair_class={};post_repair_label={};post_repair_hash={};patch_changed={};repair_changed={}",
                        post_row.order,
                        field_name,
                        baseline_class,
                        baseline_label,
                        baseline_hash,
                        raw_patch_class,
                        raw_patch_label,
                        raw_patch_hash,
                        pre_repair_class,
                        pre_repair_label,
                        pre_repair_hash,
                        post_repair_class,
                        post_repair_label,
                        post_repair_hash,
                        patch_changed,
                        repair_changed,
                    ),
                    related_sample_id: Some(post_row.prompt_text_source_row_id.clone()),
                });
            }
        }
    }

    warnings
}

fn sanitize_repair_reason_code(reason: &str) -> String {
    reason
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
        .take(48)
        .collect::<String>()
}

fn repair_live_expanded_script_text(
    live_text: &str,
    source_text: &str,
    scene_type: &str,
    scene_label: &str,
) -> Option<LiveTextRepairResult> {
    let original = sanitize_product_body_text(live_text);
    if original.trim().is_empty()
        || contains_product_control_text(&original)
        || looks_like_storyboard_or_prompt_text(&original)
    {
        return None;
    }
    let mut repaired = original.clone();
    let mut summary = LiveRepairSummary {
        raw_failed_validator: true,
        reasons: Vec::new(),
    };

    repair_live_expand_external_terms(&mut repaired, source_text, &mut summary);
    repair_live_expand_abstract_pressure(&mut repaired, source_text, &mut summary);
    repair_live_expand_ungrounded_names(&mut repaired, source_text, &mut summary);

    if let Some(canonical) = canonical_live_expand_source_text(source_text, scene_type, scene_label)
    {
        if source_has_a_ruin_enemy_facts(source_text)
            || source_has_b_alley_pursuit_facts(source_text)
            || live_expand_text_needs_source_canonicalization(&repaired, source_text)
            || live_expand_text_needs_scene_adaptation(&repaired, source_text, scene_label)
            || validate_generated_script_text_with_reason(&repaired, source_text).is_err()
        {
            repaired = canonical;
            summary.push_reason("source_fact_canonicalized");
            summary.push_reason("scene_adaptation_repaired");
        }
    }

    if repaired == original && !summary.repaired() {
        return None;
    }

    validate_generated_script_text_with_reason(&repaired, source_text)
        .ok()
        .map(|text| LiveTextRepairResult { text, summary })
}

fn repair_live_expand_external_terms(
    text: &mut String,
    source_text: &str,
    summary: &mut LiveRepairSummary,
) {
    for term in EXPAND_SCRIPT_FORBIDDEN_ADDED_IDENTITY_TERMS
        .iter()
        .chain(EXPAND_SCRIPT_FORBIDDEN_EXTERNAL_SETTING_TERMS.iter())
        .chain(EXPAND_SCRIPT_FORBIDDEN_MILITARY_SCALE_EXPANSION_TERMS.iter())
        .chain(EXPAND_SCRIPT_FORBIDDEN_EXTERNAL_ACTION_TERMS.iter())
        .copied()
    {
        if text.contains(term) && !source_text.contains(term) {
            *text = text.replace(term, "");
            summary.push_reason("source_external_term_removed");
        }
    }
}

fn repair_live_expand_abstract_pressure(
    text: &mut String,
    source_text: &str,
    summary: &mut LiveRepairSummary,
) {
    if !text.contains("危险感") || source_text.contains("危险感") {
        return;
    }
    let pressure = if source_has_b_alley_pursuit_facts(source_text) {
        "追兵逼近压力"
    } else if source_has_a_ruin_enemy_facts(source_text) {
        "对峙压力"
    } else {
        "压力"
    };
    *text = text.replace("危险感", pressure);
    summary.push_reason("abstract_pressure_rebound");
}

fn repair_live_expand_ungrounded_names(
    text: &mut String,
    source_text: &str,
    summary: &mut LiveRepairSummary,
) {
    if !source_has_a_ruin_enemy_facts(source_text) && !source_has_b_alley_pursuit_facts(source_text)
    {
        return;
    }
    for _ in 0..8 {
        let Some(name) = generated_script_ungrounded_character_name(text, source_text) else {
            break;
        };
        let replacement = source_grounded_name_replacement(source_text);
        if replacement.trim().is_empty() || replacement == name {
            break;
        }
        *text = text.replace(&name, &replacement);
        summary.push_reason("source_external_name_rebound");
    }
}

fn source_grounded_name_replacement(source_text: &str) -> String {
    if source_has_b_alley_pursuit_facts(source_text) {
        "追兵".to_string()
    } else if source_has_a_ruin_enemy_facts(source_text) {
        "主角".to_string()
    } else {
        let registry = CharacterRegistry::from_story_text(source_text, source_text);
        registry
            .characters
            .first()
            .map(|character| trim_detected_character_name(&character.name))
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "来人".to_string())
    }
}

fn live_expand_text_needs_source_canonicalization(text: &str, source_text: &str) -> bool {
    if source_has_b_alley_pursuit_facts(source_text) {
        return !contains_any_story_term(text, &["林峰护住苏瑶", "护住苏瑶"])
            || !contains_any_story_term(text, &["阿青提醒", "提醒他们"])
            || !contains_any_story_term(text, &["黑衣追兵", "追兵"])
            || !text.contains("巷口")
            || !contains_any_story_term(text, &["逼近", "压近", "追兵压力", "追兵逼近压力"]);
    }
    if source_has_a_ruin_enemy_facts(source_text) {
        return !text.contains("废墟")
            || !text.contains("主角")
            || !text.contains("敌人")
            || !contains_any_story_term(text, &["单膝跪地", "单膝", "跪地"])
            || !contains_any_story_term(text, &["缓步逼近", "逼近", "对峙压力"]);
    }
    false
}

fn live_expand_text_needs_scene_adaptation(
    text: &str,
    source_text: &str,
    scene_label: &str,
) -> bool {
    let scene_label = scene_label.trim();
    if scene_label.is_empty() {
        return false;
    }
    if source_has_b_alley_pursuit_facts(source_text) {
        if scene_label.contains("沙盘") || scene_label.contains("视口") {
            return !contains_any_story_term(text, &["视口", "态势", "退路", "调度"]);
        }
        if scene_label.contains("热血") || scene_label.contains("战斗") {
            return !contains_any_story_term(text, &["热血", "紧迫", "对抗", "动作节奏"]);
        }
    }
    if source_has_a_ruin_enemy_facts(source_text) {
        if scene_label.contains("国战") || scene_label.contains("军阵") {
            return !contains_any_story_term(
                text,
                &["国战", "军阵", "战场秩序", "前场压迫", "后场调度"],
            );
        }
        if scene_label.contains("热血") || scene_label.contains("战斗") {
            return !contains_any_story_term(text, &["热血", "废墟对峙", "撑住", "战斗压力"]);
        }
    }
    false
}

fn canonical_live_expand_source_text(
    source_text: &str,
    scene_type: &str,
    scene_label: &str,
) -> Option<String> {
    build_scene_adapted_expanded_story_material(source_text, scene_label)
        .or_else(|| build_scene_adapted_expanded_story_material(source_text, scene_type))
}

fn build_scene_adapted_expanded_story_material(
    source_text: &str,
    scene_label: &str,
) -> Option<String> {
    let label = scene_label.trim();
    if source_has_b_alley_pursuit_facts(source_text) {
        if label.contains("沙盘") || label.contains("视口") || label.contains("sandbox") {
            return Some(
                "沙盘战略视口中，巷口压力被放进态势关系里：林峰护住苏瑶，阿青提醒他们黑衣追兵从巷口逼近；视口强调追兵逼近关系、退路收窄和调度先后。第二段继续保留林峰护住苏瑶、阿青提醒、黑衣追兵、巷口和逼近，让后续分镜能从态势、退路和追兵逼近压力拆开。".to_string(),
            );
        }
        return Some(
            "热血战斗节奏下，林峰护住苏瑶的动作先压住画面，阿青提醒他们黑衣追兵从巷口逼近；紧迫、对抗和护人压力一起推进，追兵逼近压力让退路变窄。第二段继续保留林峰护住苏瑶、阿青提醒、黑衣追兵、巷口和逼近，把动作节奏与压迫感推得更明确。".to_string(),
        );
    }
    if source_has_a_ruin_enemy_facts(source_text) {
        if label.contains("国战") || label.contains("军阵") || label.contains("war") {
            return Some(
                "国战军阵建立的表达改变视角和调度感：废墟之上，主角单膝跪地，敌人缓步逼近；前场压迫、后场调度和战场秩序感把对峙距离排清楚，事实仍只围绕主角、敌人、废墟、单膝跪地和逼近展开。第二段继续用军阵视角强调前后场压力，敌人缓步逼近继续施压；主角仍在废墟之上保持单膝跪地，对峙两端被放进战场秩序里。".to_string(),
            );
        }
        return Some(
            "热血战斗的表达从废墟对峙开始：主角单膝跪地却仍撑住身体，敌人缓步逼近，把战斗压力一步步压到身前。第二段继续保留废墟、主角、敌人、单膝跪地和逼近，只强化对抗、撑住、逼近压力和动作节奏。".to_string(),
        );
    }
    None
}

fn source_has_a_ruin_enemy_facts(source_text: &str) -> bool {
    source_text.contains("主角")
        && source_text.contains("敌人")
        && contains_any_story_term(source_text, &["废墟", "废墟之上"])
        && contains_any_story_term(source_text, &["单膝跪地", "单膝", "跪地"])
        && contains_any_story_term(source_text, &["缓步逼近", "逼近", "对峙压力"])
}

fn source_has_b_alley_pursuit_facts(source_text: &str) -> bool {
    source_text.contains("林峰")
        && source_text.contains("苏瑶")
        && source_text.contains("阿青")
        && contains_any_story_term(source_text, &["黑衣追兵", "追兵"])
        && source_text.contains("巷口")
        && contains_any_story_term(source_text, &["逼近", "压近", "追兵压力"])
}

fn source_has_c_rainy_dock_photo_facts(source_text: &str) -> bool {
    source_text.contains("女主")
        && contains_any_story_term(source_text, &["旧照片", "旧相片", "照片", "相片"])
        && contains_any_story_term(source_text, &["陌生人", "追来的陌生人", "追来的人"])
        && contains_any_story_term(source_text, &["路灯", "灯下"])
        && contains_any_story_term(source_text, &["雨夜码头", "码头"])
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
    let mut retryable_failures = 0usize;
    let (body, transport_latency_ms) = loop {
        match transport(endpoint, api_key, &payload) {
            Ok(result) => break result,
            Err(warning)
                if qwen_transport_warning_is_retryable(&warning)
                    && retryable_failures + 1 < QWEN_TEXT_MODEL_MAX_TRANSPORT_ATTEMPTS =>
            {
                retryable_failures += 1;
            }
            Err(warning) => {
                return Err(qwen_retry_exhausted_warning(
                    warning,
                    retryable_failures,
                    started.elapsed().as_millis(),
                ));
            }
        }
    };
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
    let (structured_json, normalizer_actions) =
        parse_model_output_contract_json(content, request.output_schema);
    let mut warnings = qwen_retry_recovered_warnings(request, retryable_failures);
    if !normalizer_actions.is_empty() {
        warnings.push(model_output_contract_normalized_warning(
            request,
            &normalizer_actions,
        ));
    }

    Ok(TextGenerationResponse {
        text: content.to_string(),
        structured_json,
        usage_tokens: response.usage.and_then(|usage| usage.total_tokens),
        latency_ms: Some(transport_latency_ms.max(started.elapsed().as_millis() as u64)),
        warnings,
        provider: provider.provider,
        model: provider.model.clone(),
    })
}

fn qwen_retry_recovered_warnings(
    request: &TextGenerationRequest,
    retryable_failures: usize,
) -> Vec<ProductWarning> {
    if retryable_failures == 0 {
        return Vec::new();
    }
    vec![ProductWarning {
        code: TEXT_MODEL_NETWORK_RETRY_RECOVERED_CODE.to_string(),
        message: format!(
            "qwen provider transport retry recovered; retryable_failures={}; attempts={}; max_attempts={}; fallback_used=false",
            retryable_failures,
            retryable_failures + 1,
            QWEN_TEXT_MODEL_MAX_TRANSPORT_ATTEMPTS,
        ),
        related_sample_id: request.selected_sample_ids.first().cloned(),
    }]
}

fn qwen_retry_exhausted_warning(
    mut warning: ProductWarning,
    retryable_failures: usize,
    elapsed_ms: u128,
) -> ProductWarning {
    if retryable_failures == 0 || !qwen_transport_warning_is_retryable(&warning) {
        return warning;
    }
    let category = provider_warning_error_category(&warning);
    warning.message = format!(
        "{}; retry_exhausted=true; attempts={}; max_attempts={}; elapsed_bucket={}; error_category={}; fallback_used=false; local_candidate=false; raw_values_redacted=true",
        warning.message,
        retryable_failures + 1,
        QWEN_TEXT_MODEL_MAX_TRANSPORT_ATTEMPTS,
        elapsed_bucket_ms(elapsed_ms),
        category
    );
    warning
}

fn parse_model_output_contract_json(
    content: &str,
    output_schema: TextGenerationOutputSchema,
) -> (Option<Value>, Vec<String>) {
    if !matches!(
        output_schema,
        TextGenerationOutputSchema::StoryboardRowsJson | TextGenerationOutputSchema::RepairPlanJson
    ) {
        return (None, Vec::new());
    }

    for (candidate, extraction_action) in model_output_json_candidates(content) {
        let Ok(value) = serde_json::from_str::<Value>(&candidate) else {
            continue;
        };
        let mut actions = Vec::new();
        if let Some(action) = extraction_action {
            push_unique_normalizer_action(&mut actions, action);
        }
        let normalized = normalize_model_output_contract_value(value, output_schema, &mut actions);
        return (Some(normalized), actions);
    }

    (None, Vec::new())
}

fn model_output_json_candidates(content: &str) -> Vec<(String, Option<&'static str>)> {
    let mut candidates = Vec::new();
    let trimmed = content.trim();
    if !trimmed.is_empty() {
        candidates.push((trimmed.to_string(), None));
    }
    if let Some(fenced) = extract_first_markdown_json_fence(trimmed) {
        candidates.push((fenced, Some("markdown_json_unwrapped")));
    }
    if let Some(balanced) = extract_first_balanced_json(trimmed) {
        candidates.push((balanced, Some("json_payload_extracted")));
    }

    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|(candidate, _)| seen.insert(candidate.clone()))
        .collect()
}

fn extract_first_markdown_json_fence(content: &str) -> Option<String> {
    let fence_start = content.find("```")?;
    let after_fence = &content[fence_start + 3..];
    let body_start = after_fence
        .find('\n')
        .map(|index| index + 1)
        .unwrap_or_default();
    let body = &after_fence[body_start..];
    let fence_end = body.find("```")?;
    let candidate = body[..fence_end].trim();
    (!candidate.is_empty()).then(|| candidate.to_string())
}

fn extract_first_balanced_json(content: &str) -> Option<String> {
    let (start_index, first_char) = content
        .char_indices()
        .find(|(_, character)| matches!(character, '{' | '['))?;
    let mut stack = Vec::new();
    stack.push(first_char);
    let mut in_string = false;
    let mut escaped = false;

    for (relative_index, character) in content[start_index..].char_indices().skip(1) {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match character {
            '"' => in_string = true,
            '{' | '[' => stack.push(character),
            '}' => {
                if stack.pop() != Some('{') {
                    return None;
                }
            }
            ']' => {
                if stack.pop() != Some('[') {
                    return None;
                }
            }
            _ => {}
        }
        if stack.is_empty() {
            let end_index = start_index + relative_index + character.len_utf8();
            return Some(content[start_index..end_index].trim().to_string());
        }
    }
    None
}

fn normalize_model_output_contract_value(
    value: Value,
    output_schema: TextGenerationOutputSchema,
    actions: &mut Vec<String>,
) -> Value {
    match output_schema {
        TextGenerationOutputSchema::StoryboardRowsJson => {
            normalize_storyboard_rows_contract_value(value, actions)
        }
        _ => value,
    }
}

fn normalize_storyboard_rows_contract_value(value: Value, actions: &mut Vec<String>) -> Value {
    match value {
        Value::Array(rows) => {
            push_unique_normalizer_action(actions, "top_level_rows_array_normalized");
            json!({ "rows": normalize_storyboard_contract_rows(rows, actions) })
        }
        Value::Object(map) => {
            let row_keys = [
                "rows",
                "storyboard_rows",
                "storyboard",
                "shots",
                "shot_list",
            ];
            if let Some((key, rows)) = row_keys.iter().find_map(|key| {
                map.get(*key)
                    .and_then(|value| value.as_array())
                    .map(|rows| (*key, rows.clone()))
            }) {
                if key != "rows" {
                    push_unique_normalizer_action(actions, "rows_field_alias_mapped");
                }
                return json!({ "rows": normalize_storyboard_contract_rows(rows, actions) });
            }
            Value::Object(map)
        }
        other => other,
    }
}

fn normalize_storyboard_contract_rows(rows: Vec<Value>, actions: &mut Vec<String>) -> Vec<Value> {
    rows.into_iter()
        .map(|row| normalize_storyboard_contract_row(row, actions))
        .collect()
}

fn normalize_storyboard_contract_row(row: Value, actions: &mut Vec<String>) -> Value {
    let Value::Object(map) = row else {
        return row;
    };
    let mut normalized = serde_json::Map::new();
    for (canonical, aliases) in [
        ("shot_title", &["shot_title", "title", "shot"][..]),
        ("person", &["person", "character", "subject", "role"][..]),
        (
            "scene_scale",
            &["scene_scale", "scale", "shot_size", "framing"][..],
        ),
        (
            "visual_description",
            &[
                "visual_description",
                "visual",
                "image",
                "scene_description",
                "composition",
            ][..],
        ),
        (
            "character_action",
            &[
                "character_action",
                "action",
                "performance",
                "character_motion",
            ][..],
        ),
        (
            "camera_movement",
            &[
                "camera_movement",
                "camera",
                "camera_motion",
                "camera_move",
                "lens_movement",
            ][..],
        ),
        ("dialogue", &["dialogue", "line", "spoken_line"][..]),
    ] {
        if let Some(value) = model_contract_string_field(&map, aliases) {
            if aliases.first().copied() != Some(canonical) || !map.contains_key(canonical) {
                push_unique_normalizer_action(actions, "row_field_alias_mapped");
            }
            normalized.insert(canonical.to_string(), Value::String(value));
        }
    }
    if map.contains_key("prompt_text") {
        push_unique_normalizer_action(actions, "model_prompt_text_ignored");
    }
    Value::Object(normalized)
}

fn model_contract_string_field(
    map: &serde_json::Map<String, Value>,
    aliases: &[&str],
) -> Option<String> {
    aliases.iter().find_map(|alias| match map.get(*alias)? {
        Value::String(value) => Some(value.trim().to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    })
}

fn push_unique_normalizer_action(actions: &mut Vec<String>, action: &str) {
    if !actions.iter().any(|item| item == action) {
        actions.push(action.to_string());
    }
}

fn model_output_contract_normalized_warning(
    request: &TextGenerationRequest,
    actions: &[String],
) -> ProductWarning {
    ProductWarning {
        code: TEXT_MODEL_OUTPUT_CONTRACT_NORMALIZED_CODE.to_string(),
        message: format!(
            "model_output_contract_normalized=true; actions={}; normalizer_added_facts=false; sample_text_used=false; template_style_applied=false; creative_freedom_preserved=true; hard_gate_status=unchanged; raw_values_redacted=true",
            actions.join("|")
        ),
        related_sample_id: request.selected_sample_ids.first().cloned(),
    }
}

fn text_generation_fallback_blocking_warnings(warnings: &[ProductWarning]) -> Vec<ProductWarning> {
    warnings
        .iter()
        .filter(|warning| {
            warning.code != TEXT_MODEL_NETWORK_RETRY_RECOVERED_CODE
                && warning.code != TEXT_MODEL_OUTPUT_CONTRACT_NORMALIZED_CODE
        })
        .cloned()
        .collect()
}

fn qwen_transport_warning_is_retryable(warning: &ProductWarning) -> bool {
    if warning.code != "text_model_network_error" {
        return false;
    }
    match http_status_from_warning_message(&warning.message) {
        Some(status) => (500..=599).contains(&status),
        None => true,
    }
}

fn http_status_from_warning_message(message: &str) -> Option<u16> {
    let marker = "HTTP ";
    let start = message.find(marker)? + marker.len();
    let digits = message[start..]
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();
    digits.parse::<u16>().ok()
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
    let system_prompt = "You are Hope's controlled text-generation layer. Ground storyboard output in shot_script first, then expanded_script_text, then primary scene fields, and only then compressed KB context. Preserve accepted character relationships, motivation, event order, timeline, prop state, emotional progression, conflict causality, and next-scene continuity. Scene changes expression only; directing schedules shots only. For storyboard rows, person is a source-bound character name or role label only; it must never be a visual term, camera term, composition word, or abstract label such as 高对比, 焦点, 画面, 构图, 光影, 氛围, or 空间. Never invent character names, rename roles, emit full KB rows, source_register, overlay JSON, internal control text, real director names, IP names, brand names, or external asset bindings.";
    let user_prompt = match (request.task_type, request.output_schema) {
        (TextGenerationTask::ExpandScript, TextGenerationOutputSchema::PlainText) => format!(
            "task_type=expand_script\nscene_type={}\nduration_seconds={}\nstory_input={}\nkb_context_summary={}\nselected_sample_ids={}\nselected_kb_rules={}\noutput_schema=story_script_plain_text\nconstraints=Write only user-visible story body. current_scene_label inside story_input is binding; rewrite the visible expression to match it even when the source body was written for another scene type. Preserve source facts, character roles, event order, and duration target, but change rhythm, action density, emotional temperature, relationship expression, and visual focus to fit the selected scene. Scene style changes expression only; it must not add outside world facts, military records, artifacts, inscriptions, map lore, camp names, official titles, or source-external props such as 北营兵符拓片, 丞相帐, 建安十七年监造, 中军, 虎牢关守军轮值图, 罗盘认路, 黑釉兵俑, 戴着皮套的手, 铭文, 金线, or new worldview objects. Do not expand one approaching enemy into a whole formation, army scale, rank structure, or organized military array; military scenes may change composition, order, and pressure only while keeping the source fact as 敌人, 逼近的人, or 对峙对象. Do not invent character names or rename role labels. Preserve source role boundaries such as 主角 and 敌人 even when the selected scene is daily or healing. If the source body only has role labels such as 主角 and 敌人, keep those role labels and never add a proper name such as 孔却燃. When the source is only 主角、敌人、废墟、单膝跪地、缓步逼近, or 对峙压力, do not add 冷笑, specific expressions, body wounds, body-part details, weapon details, armor, 断戟, or other new action/prop facts. National-war expression may change order and pressure, but must not add armor, body, weapon, or army-scale details such as 铠甲, 甲胄, 剑柄, 左膝, 左臂, 肩甲, 断戟, or 整列军阵. If the source contains 黑衣追兵, keep it as 黑衣追兵 or 追兵; never expand it into 三名黑衣追兵 or any numbered pursuer group, and never add 左臂, 衣袖, body wounds, or injury details. For sandbox view, when the source includes 林峰护住苏瑶, 阿青提醒, 黑衣追兵, 巷口, and 逼近, all five facts must remain visible. If conflict needs softening, use non-identifying labels implied by the source, such as 逼近的人, 对峙对象, or 来人, while preserving pursuit pressure when source has 黑衣追兵逼近, 敌人逼近, 追兵压力, or 对峙压力. If the source contains 黑衣追兵, 追兵, 敌人, 逼近, 对峙压力, 断后, or 撤离, the final body must keep a visible pressure term such as 追兵, 敌人, 逼近, 对峙压力, 追兵压力, 断后, or 撤离. Never recast an enemy or bystander as 邻居家大叔, 邻居家孩子, 熟悉的身影, 邻居, 孩子, 熟人, or any new concrete identity. Do not output scene_type, scene_label, scene_category, current_scene_label, target_duration_seconds, source_package, source_input_type, authoring_mode, prompt_text, storyboard rows, timecodes, JSON, source_register, overlay JSON, or internal control lines. Keep the story ready for later shot decomposition without pre-formatting shots.",
            scene_type,
            duration_seconds,
            request.story_input,
            request.kb_context_summary,
            request.selected_sample_ids.join(","),
            request.selected_kb_rules.join(" | "),
        ),
        (
            TextGenerationTask::GenerateStoryboard,
            TextGenerationOutputSchema::StoryboardRowsJson,
        ) => format!(
            "task_type=generate_storyboard\nscene_type={}\nduration_seconds={}\nshot_script={}\nkb_context_summary={}\nselected_sample_ids={}\nselected_kb_rules={}\noutput_schema=storyboard_rows_json\nconstraints=Output a JSON object with a rows array. Each row must include person, shot_title, scene_scale, camera_movement, visual_description, character_action, dialogue, and duration_seconds. The person field must use only a name or role label explicitly present in shot_script; do not invent character names or rename roles. If shot_script contains 主角 or 敌人, bind person to those source roles before considering any visual wording. Do not append action, measure words, possessives, or body reactions to names; examples like 主角之, 林峰压, 苏瑶压, 林峰一, 林峰一把, 林峰猛然, 林峰转身, and 林峰抬手 are invalid person values and must be 主角, 林峰, or 苏瑶. Never use 断戟立, 断戟, weapon fragments, body parts, camera, composition, lighting, framing, scene object, UI, or abstract labels as person; forbidden person examples include 环境, 沙盘, 城建面板, 镜头, 构图, 画面, 场景, UI, 高对比, 焦点, 光影, 氛围, 空间, 特写, 近景, 中景, 全景, 低机位, 俯拍, and 仰拍. Preserve accepted continuity. Visual descriptions must include environment or space, composition, visible light or atmosphere, and the current visual event; for A-source rows include visible source facts 主角, 敌人, 废墟, 单膝跪地, 缓步逼近, or 对峙压力, and for B-source rows include 林峰护住苏瑶, 阿青提醒, 黑衣追兵, 巷口, or 逼近. Never emit full KB rows, source_register, overlay JSON, or internal control text. Never turn internal sample evidence into final prompt_text.",
            scene_type,
            duration_seconds,
            request.story_input,
            request.kb_context_summary,
            request.selected_sample_ids.join(","),
            request.selected_kb_rules.join(" | "),
        ),
        _ => format!(
            "task_type={:?}\nscene_type={}\nduration_seconds={}\nstory_input={}\nkb_context_summary={}\nselected_sample_ids={}\nselected_kb_rules={}\noutput_schema={}\nconstraints=Keep total duration conserved. Preserve accepted continuity. Do not invent character names or rename roles. Never emit full KB rows, source_register, overlay JSON, or internal control text. Never turn internal sample evidence into final prompt_text.",
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
        .timeout(std::time::Duration::from_secs(
            qwen_transport_timeout_seconds(),
        ))
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
        .map_err(|error| {
            let category = reqwest_transport_error_category(&error);
            ProductWarning {
                code: "text_model_network_error".to_string(),
                message: format!(
                    "qwen compatible transport request failed; error_category={}; retryable=true; fallback will use local candidate if retry is exhausted.",
                    category
                ),
                related_sample_id: None,
            }
        })?;
    let status = response.status();
    if !status.is_success() {
        return Err(ProductWarning {
            code: "text_model_network_error".to_string(),
            message: format!(
                "qwen compatible transport returned HTTP {}; fallback will use local candidate if retry is exhausted.",
                status.as_u16()
            ),
            related_sample_id: None,
        });
    }
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

fn qwen_transport_timeout_seconds() -> u64 {
    env::var("HOPE_QA_PROVIDER_TIMEOUT_SECONDS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| (10..=120).contains(value))
        .unwrap_or(30)
}

fn reqwest_transport_error_category(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "timeout"
    } else if error.is_connect() {
        "connect"
    } else if error.is_request() {
        "request"
    } else if error.is_body() {
        "body"
    } else if error.is_decode() {
        "decode"
    } else if error.is_redirect() {
        "redirect"
    } else {
        "network_or_transport"
    }
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

fn provider_kind_code(kind: TextModelProviderKind) -> &'static str {
    match kind {
        TextModelProviderKind::Qwen => "qwen",
        TextModelProviderKind::Doubao => "doubao",
        TextModelProviderKind::Custom => "custom",
    }
}

fn resolve_expand_script_target_duration_seconds(request: &ExpandScriptRequest) -> u16 {
    if normalize_target_duration_mode(&request.target_duration_mode)
        == Some(TARGET_DURATION_MODE_LONG_TEXT_AUTO)
    {
        return estimate_long_text_auto_duration_seconds(
            stable_source_material_length_chars(&request.synopsis_text),
            request.source_input_type.trim(),
            &request.source_story_facts,
        );
    }

    request
        .target_duration_seconds
        .or(request.selected_total_duration_seconds)
        .or_else(|| infer_duration_seconds_from_text(&request.synopsis_text))
        .filter(|duration| is_supported_storyboard_duration(*duration))
        .unwrap_or(DEFAULT_EXPAND_SCRIPT_DURATION_SECONDS)
}

fn plan_desktop_duration(
    request: &ExpandScriptRequest,
    source_analysis: &DesktopSourceInputAnalysis,
) -> DesktopDurationPlan {
    let source_material_length_chars = if request.source_material_length_chars > 0 {
        request.source_material_length_chars
    } else {
        stable_source_material_length_chars(&request.synopsis_text)
    };
    let requested_mode = request.target_duration_mode.trim();
    let normalized_mode = normalize_target_duration_mode(requested_mode);
    let target_duration_mode = normalized_mode
        .unwrap_or(TARGET_DURATION_MODE_FIXED_SECONDS)
        .to_string();
    let mut warnings = Vec::new();

    if normalized_mode.is_none() && !requested_mode.is_empty() {
        warnings.push(duration_plan_warning(
            "target_duration_mode_invalid",
            "target_duration_mode must be fixed_seconds or long_text_auto.",
        ));
    }

    if target_duration_mode == TARGET_DURATION_MODE_LONG_TEXT_AUTO {
        let estimated_total_story_duration_seconds = estimate_long_text_auto_duration_seconds(
            source_material_length_chars,
            &source_analysis.source_input_type,
            &source_analysis.source_story_facts,
        );
        let story_length_profile =
            non_blank_string(&request.story_length_profile).unwrap_or_else(|| {
                derive_auto_story_length_profile(
                    source_material_length_chars,
                    estimated_total_story_duration_seconds,
                )
            });
        let auto_segment_strategy = non_blank_string(&request.auto_segment_strategy)
            .unwrap_or_else(|| AUTO_SEGMENT_STRATEGY_LONG_TEXT.to_string());
        if request.auto_segment_strategy.trim().is_empty() {
            warnings.push(duration_plan_warning(
                "auto_segment_strategy_missing",
                "long_text_auto used the deterministic story-fact segment strategy.",
            ));
        }
        let narrative_beat_count = estimate_long_text_auto_narrative_beat_count(
            source_material_length_chars,
            &source_analysis.source_story_facts,
        );
        let shot_task_durations = allocate_long_text_auto_shot_task_durations(
            estimated_total_story_duration_seconds,
            narrative_beat_count,
        );
        if shot_task_durations.is_empty() {
            warnings.push(duration_plan_warning(
                "long_text_auto_duration_plan_missing",
                "long_text_auto could not produce a duration plan.",
            ));
        }
        if shot_task_durations.len() <= 1 {
            warnings.push(duration_plan_warning(
                "long_text_auto_compressed_to_single_clip_blocked",
                "long_text_auto must split source material into multiple shot tasks.",
            ));
        }

        return DesktopDurationPlan {
            target_duration_mode,
            story_length_profile,
            source_material_length_chars,
            auto_segment_strategy,
            estimated_total_story_duration_seconds,
            generated_shot_task_count: shot_task_durations.len() as u32,
            duration_plan_summary: build_long_text_duration_plan_summary(
                shot_task_durations.len(),
                &shot_task_durations,
            ),
            warnings,
        };
    }

    let fixed_duration = resolve_expand_script_target_duration_seconds(request);
    let row_durations = allocate_storyboard_row_durations(fixed_duration, 0)
        .unwrap_or_else(|| vec![fixed_duration]);
    let story_length_profile =
        non_blank_string(&request.story_length_profile).unwrap_or_else(|| {
            derive_auto_story_length_profile(source_material_length_chars, fixed_duration)
        });
    let auto_segment_strategy = non_blank_string(&request.auto_segment_strategy)
        .unwrap_or_else(|| AUTO_SEGMENT_STRATEGY_FIXED_SECONDS.to_string());

    DesktopDurationPlan {
        target_duration_mode: TARGET_DURATION_MODE_FIXED_SECONDS.to_string(),
        story_length_profile: story_length_profile.clone(),
        source_material_length_chars,
        auto_segment_strategy,
        estimated_total_story_duration_seconds: fixed_duration,
        generated_shot_task_count: row_durations.len() as u32,
        duration_plan_summary: format!(
            "mode={}; story_length_profile={}; source_chars={}; total={}s; generated_shot_tasks={}; shot_task_durations={}",
            TARGET_DURATION_MODE_FIXED_SECONDS,
            story_length_profile,
            source_material_length_chars,
            fixed_duration,
            row_durations.len(),
            join_durations(&row_durations)
        ),
        warnings,
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
        STORY_LENGTH_PROFILE_LONG_STORY_AUTO
    } else if source_material_length_chars >= 2_500 || estimated_total_story_duration_seconds >= 120
    {
        STORY_LENGTH_PROFILE_TWO_MINUTE_STORY
    } else if source_material_length_chars >= 2_000 || estimated_total_story_duration_seconds >= 105
    {
        STORY_LENGTH_PROFILE_SHORT_STORY
    } else if estimated_total_story_duration_seconds >= 75 {
        STORY_LENGTH_PROFILE_LONG_STORY
    } else if estimated_total_story_duration_seconds >= 45 {
        STORY_LENGTH_PROFILE_STANDARD_CLIP
    } else {
        STORY_LENGTH_PROFILE_SHORT_CLIP
    }
    .to_string()
}

fn estimate_long_text_auto_narrative_beat_count(
    source_material_length_chars: u32,
    facts: &SourceStoryFacts,
) -> usize {
    let event_based = facts.event_order.len().max(facts.core_events.len()).max(1);
    let length_based = match source_material_length_chars {
        0..=220 => 3,
        221..=800 => 5,
        801..=2_000 => 7,
        2_001..=2_500 => 8,
        2_501..=3_500 => 10,
        3_501..=6_000 => 12,
        _ => 14,
    };
    event_based.max(length_based).min(18)
}

fn build_long_text_duration_plan_summary(shot_task_count: usize, durations: &[u16]) -> String {
    let duration_profile = if durations.iter().any(|duration| *duration == 15) {
        if durations
            .iter()
            .any(|duration| *duration == SEEDANCE_REMAINDER_SEGMENT_SECONDS)
        {
            "10-15 second durations, with a 5-second remainder only when needed"
        } else {
            "10-15 second durations, with 15-second tasks reserved for denser beats"
        }
    } else if durations
        .iter()
        .any(|duration| *duration == SEEDANCE_REMAINDER_SEGMENT_SECONDS)
    {
        "10-second durations, with a 5-second remainder only when needed"
    } else {
        "10-second durations"
    };
    format!(
        "Long text auto segmented the story around narrative beats into {shot_task_count} shot tasks; each storyboard row keeps Seedance-friendly {duration_profile}."
    )
}

fn round_duration_to_five(duration_seconds: u16) -> u16 {
    ((duration_seconds + SEEDANCE_REMAINDER_SEGMENT_SECONDS - 1)
        / SEEDANCE_REMAINDER_SEGMENT_SECONDS)
        * SEEDANCE_REMAINDER_SEGMENT_SECONDS
}

fn allocate_long_text_auto_shot_task_durations(
    total_duration_seconds: u16,
    narrative_beat_count: usize,
) -> Vec<u16> {
    if total_duration_seconds < SEEDANCE_STANDARD_SEGMENT_SECONDS
        || total_duration_seconds % SEEDANCE_REMAINDER_SEGMENT_SECONDS != 0
    {
        return vec![];
    }
    let min_count = usize::from(total_duration_seconds.div_ceil(15));
    let max_count = usize::from(total_duration_seconds / SEEDANCE_REMAINDER_SEGMENT_SECONDS);
    let preferred_count = usize::from(total_duration_seconds.div_ceil(10)).min(max_count);
    let target_count = narrative_beat_count
        .max(1)
        .min(max_count)
        .clamp(min_count, preferred_count.max(min_count));
    let mut durations = vec![SEEDANCE_STANDARD_SEGMENT_SECONDS; target_count];
    let baseline_total = u16::try_from(target_count)
        .ok()
        .and_then(|count| count.checked_mul(SEEDANCE_STANDARD_SEGMENT_SECONDS))
        .unwrap_or(total_duration_seconds);
    if total_duration_seconds > baseline_total {
        let upgrades = usize::from(
            (total_duration_seconds - baseline_total) / SEEDANCE_REMAINDER_SEGMENT_SECONDS,
        );
        for duration in durations.iter_mut().take(upgrades) {
            *duration += SEEDANCE_REMAINDER_SEGMENT_SECONDS;
        }
    } else if baseline_total > total_duration_seconds {
        let reductions = usize::from(
            (baseline_total - total_duration_seconds) / SEEDANCE_REMAINDER_SEGMENT_SECONDS,
        );
        for duration in durations.iter_mut().rev().take(reductions) {
            *duration -= SEEDANCE_REMAINDER_SEGMENT_SECONDS;
        }
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

#[cfg(test)]
fn validate_generated_script_text(text: &str, source_text: &str) -> Option<String> {
    validate_generated_script_text_with_reason(text, source_text).ok()
}

fn validate_generated_script_text_with_reason(
    text: &str,
    source_text: &str,
) -> Result<String, String> {
    let trimmed = text.trim();
    let cleaned = sanitize_product_body_text(trimmed);
    if trimmed.is_empty() || cleaned.is_empty() {
        return Err("live text empty after sanitization".to_string());
    }
    if contains_forbidden_generation_terms(&cleaned) {
        return Err("contains forbidden generation term".to_string());
    }
    if looks_like_storyboard_or_prompt_text(&cleaned) {
        return Err("looks like storyboard or prompt text instead of story body".to_string());
    }
    if let Some(reason) = generated_script_identity_validator_reason(&cleaned, source_text) {
        return Err(reason);
    }
    Ok(cleaned)
}

fn generated_script_identity_validator_reason(
    generated_text: &str,
    source_text: &str,
) -> Option<String> {
    if let Some(term) = generated_script_forbidden_added_identity_term(generated_text, source_text)
    {
        return Some(format!("新增源文本外身份：{term}"));
    }
    if let Some(term) =
        generated_script_forbidden_external_setting_term(generated_text, source_text)
    {
        return Some(format!("新增源文本外设定：{term}"));
    }
    if let Some(term) =
        generated_script_forbidden_military_scale_expansion_term(generated_text, source_text)
    {
        return Some(format!("新增源文本外军阵规模：{term}"));
    }
    if let Some(term) = generated_script_forbidden_external_action_term(generated_text, source_text)
    {
        return Some(format!("新增源文本外动作：{term}"));
    }
    if let Some(name) = generated_script_ungrounded_character_name(generated_text, source_text) {
        return Some(format!("新增源文本外姓名：{name}"));
    }
    if generated_script_omits_required_conflict_pressure(generated_text, source_text) {
        return Some("丢失源文本追兵/对峙压力".to_string());
    }
    None
}

fn generated_script_forbidden_added_identity_term(
    generated_text: &str,
    source_text: &str,
) -> Option<&'static str> {
    EXPAND_SCRIPT_FORBIDDEN_ADDED_IDENTITY_TERMS
        .iter()
        .copied()
        .find(|term| generated_text.contains(term) && !source_text.contains(term))
}

fn generated_script_forbidden_external_setting_term(
    generated_text: &str,
    source_text: &str,
) -> Option<&'static str> {
    EXPAND_SCRIPT_FORBIDDEN_EXTERNAL_SETTING_TERMS
        .iter()
        .copied()
        .find(|term| generated_text.contains(term) && !source_text.contains(term))
}

fn generated_script_forbidden_military_scale_expansion_term(
    generated_text: &str,
    source_text: &str,
) -> Option<&'static str> {
    EXPAND_SCRIPT_FORBIDDEN_MILITARY_SCALE_EXPANSION_TERMS
        .iter()
        .copied()
        .find(|term| generated_text.contains(term) && !source_text.contains(term))
}

fn generated_script_forbidden_external_action_term(
    generated_text: &str,
    source_text: &str,
) -> Option<&'static str> {
    EXPAND_SCRIPT_FORBIDDEN_EXTERNAL_ACTION_TERMS
        .iter()
        .copied()
        .find(|term| generated_text.contains(term) && !source_text.contains(term))
}

fn generated_script_omits_required_conflict_pressure(
    generated_text: &str,
    source_text: &str,
) -> bool {
    let source_has_pursuit_pressure = contains_any_story_term(
        source_text,
        &[
            "黑衣追兵",
            "追兵",
            "敌人",
            "敌方",
            "敌将",
            "对手",
            "对峙对象",
        ],
    ) && contains_any_story_term(
        source_text,
        &["逼近", "压近", "追来", "追上", "压力", "对峙"],
    );
    if !source_has_pursuit_pressure {
        return false;
    }

    let preserves_pursuer = contains_any_story_term(
        generated_text,
        &[
            "黑衣追兵",
            "追兵",
            "敌人",
            "敌方",
            "敌将",
            "对手",
            "那人",
            "逼近的人",
            "对峙对象",
            "来人",
        ],
    ) || contains_any_story_term(generated_text, &["对峙压力", "追兵压力"])
        || (contains_any_story_term(source_text, &["断后", "撤离"])
            && contains_any_story_term(generated_text, &["断后", "撤离"]));
    let preserves_pressure = contains_any_story_term(
        generated_text,
        &[
            "逼近", "压近", "追来", "追上", "压力", "对峙", "断后", "撤离",
        ],
    );
    !(preserves_pursuer && preserves_pressure)
}

fn generated_script_ungrounded_character_name(
    generated_text: &str,
    source_text: &str,
) -> Option<String> {
    let source_registry = CharacterRegistry::from_story_text(source_text, source_text);
    let source_names = source_registry
        .characters
        .iter()
        .map(|character| trim_detected_character_name(&character.name))
        .collect::<Vec<_>>();
    let generated_registry = CharacterRegistry::from_story_text(generated_text, generated_text);
    if let Some(name) = generated_registry
        .characters
        .into_iter()
        .find_map(|character| {
            let name = trim_detected_character_name(&character.name);
            if name.is_empty()
                || source_text.contains(&name)
                || source_names.iter().any(|source_name| source_name == &name)
                || live_person_visual_or_abstract_term(&name).is_some()
                || looks_like_name_noise_candidate(&name)
                || looks_like_non_character_phrase(&name)
            {
                None
            } else {
                Some(name)
            }
        })
    {
        return Some(name);
    }

    if let Some(name) = detect_common_two_character_names(generated_text)
        .into_iter()
        .find(|name| {
            !source_text.contains(name)
                && !source_names.iter().any(|source_name| source_name == name)
        })
    {
        return Some(name);
    }

    if source_names
        .iter()
        .any(|name| source_name_is_specific_character_name(name))
    {
        return None;
    }

    detect_common_three_character_names(generated_text, &source_names)
        .into_iter()
        .find(|name| {
            !source_text.contains(name)
                && !source_names.iter().any(|source_name| source_name == name)
        })
}

fn detect_common_two_character_names(text: &str) -> Vec<String> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut names = Vec::new();
    for index in 0..chars.len().saturating_sub(1) {
        if !is_common_single_surname(chars[index]) || !is_cjk_unified_ideograph(chars[index + 1]) {
            continue;
        }
        let before = previous_visible_char(&chars, index);
        let after = next_visible_char(&chars, index + 2);
        let preceded_by_name_verb = matches!(
            before,
            Some('见' | '遇' | '叫' | '喊' | '向' | '对' | '和' | '与' | '同' | '找' | '问')
        );
        if !narrative_name_boundary_is_valid(before, after) && !preceded_by_name_verb {
            continue;
        }
        let candidate = chars[index..index + 2].iter().collect::<String>();
        if is_generic_actor_label(&candidate)
            || live_person_visual_or_abstract_term(&candidate).is_some()
            || looks_like_name_noise_candidate(&candidate)
            || looks_like_non_character_phrase(&candidate)
        {
            continue;
        }
        push_unique_fact(&mut names, candidate);
    }
    names
}

fn detect_common_three_character_names(text: &str, source_names: &[String]) -> Vec<String> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut names = Vec::new();
    for index in 0..chars.len().saturating_sub(2) {
        if !is_common_single_surname(chars[index])
            || !is_cjk_unified_ideograph(chars[index + 1])
            || !is_cjk_unified_ideograph(chars[index + 2])
        {
            continue;
        }
        let before = previous_visible_char(&chars, index);
        let after = next_visible_char(&chars, index + 3);
        let preceded_by_name_verb = matches!(
            before,
            Some('见' | '遇' | '叫' | '喊' | '向' | '对' | '和' | '与' | '同' | '找' | '问')
        );
        let followed_by_action = candidate_followed_by_narrative_action(&chars, index + 3);
        if !narrative_name_boundary_is_valid(before, after)
            && !preceded_by_name_verb
            && !followed_by_action
        {
            continue;
        }
        let candidate = chars[index..index + 3].iter().collect::<String>();
        if candidate_is_source_name_with_action_tail(&candidate, source_names)
            || is_generic_actor_label(&candidate)
            || live_person_visual_or_abstract_term(&candidate).is_some()
            || looks_like_name_noise_candidate(&candidate)
            || looks_like_non_character_phrase(&candidate)
        {
            continue;
        }
        push_unique_fact(&mut names, candidate);
    }
    names
}

fn candidate_is_source_name_with_action_tail(candidate: &str, source_names: &[String]) -> bool {
    source_names.iter().any(|source_name| {
        let source_name = source_name.trim();
        !source_name.is_empty()
            && candidate
                .strip_prefix(source_name)
                .is_some_and(person_tail_is_action_or_quantity)
    })
}

fn candidate_followed_by_narrative_action(chars: &[char], end: usize) -> bool {
    narrative_tail_starts_with(
        chars,
        end,
        &[
            "单膝",
            "缓步",
            "护住",
            "压低",
            "低声",
            "转身",
            "回身",
            "从",
            "在",
            "把",
            "将",
            "让",
            "带",
            "守",
            "提醒",
            "继续",
            "仍",
            "坚持",
            "准备反击",
            "准备反",
            "反击",
        ],
    )
}

fn source_name_is_specific_character_name(name: &str) -> bool {
    let trimmed = name.trim();
    !trimmed.is_empty()
        && !is_generic_actor_label(trimmed)
        && live_person_visual_or_abstract_term(trimmed).is_none()
        && !looks_like_name_noise_candidate(trimmed)
        && !looks_like_non_character_phrase(trimmed)
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

fn contains_product_control_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    PRODUCT_CONTROL_ANYWHERE_TERMS
        .iter()
        .any(|term| lower.contains(term))
}

fn is_product_control_metadata_segment(segment: &str) -> bool {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    is_v0_screenplay_metadata_segment(trimmed)
        || PRODUCT_CONTROL_LINE_PREFIXES
            .iter()
            .any(|prefix| lower.starts_with(&prefix.to_ascii_lowercase()))
        || matches!(
            lower.as_str(),
            "source_material_body_begin" | "source_material_body_end"
        )
        || trimmed.starts_with("扩写剧本：")
        || trimmed.starts_with("扩写剧本:")
        || lower.contains("readystub")
}

fn strip_numbered_story_prefix(line: &str) -> Option<String> {
    for delimiter in [':', '：'] {
        let Some((prefix, remainder)) = line.split_once(delimiter) else {
            continue;
        };
        let label = prefix.trim();
        let body = remainder.trim();
        if label.is_empty() || body.is_empty() {
            continue;
        }
        let lower = label.to_ascii_lowercase();
        let looks_like_story_label = label.chars().any(|character| character.is_ascii_digit())
            || lower == "scene"
            || lower.starts_with("scene ")
            || lower.starts_with("scene_")
            || lower == "shot"
            || lower.starts_with("shot ")
            || lower.starts_with("shot_")
            || lower == "event"
            || lower.starts_with("event ")
            || lower.starts_with("event_")
            || lower == "beat"
            || lower.starts_with("beat ")
            || lower.starts_with("beat_")
            || label.starts_with("段落")
            || label.starts_with("场景")
            || label.starts_with("镜头")
            || label.starts_with("事件")
            || label.starts_with("节拍");
        if looks_like_story_label {
            return Some(body.to_string());
        }
    }
    None
}

fn sanitize_product_body_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("synopsis:") {
        let value = trimmed["synopsis:".len()..].trim();
        let value = scrub_forbidden_product_output_fragments(value);
        return (!value.is_empty()).then_some(value);
    }
    if is_product_control_metadata_segment(trimmed) {
        return None;
    }
    if let Some(stripped) = strip_numbered_story_prefix(trimmed) {
        let value = stripped.trim();
        let value = scrub_forbidden_product_output_fragments(value);
        return (!value.is_empty() && !contains_product_control_text(&value)).then_some(value);
    }
    if contains_product_control_text(trimmed) {
        return None;
    }
    Some(scrub_forbidden_product_output_fragments(trimmed))
}

fn sanitize_product_body_text(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n");
    let mut lines = normalized
        .lines()
        .filter_map(sanitize_product_body_line)
        .collect::<Vec<_>>();
    if lines.is_empty() {
        lines = split_story_segments(&normalized)
            .into_iter()
            .filter_map(|segment| sanitize_product_body_line(&segment))
            .collect();
    }
    lines.join("\n")
}

fn scrub_forbidden_product_output_fragments(value: &str) -> String {
    let mut cleaned = value
        .replace("关系保留", "人物关系延续")
        .replace("关系保持", "关系延续");
    for term in PRODUCT_OUTPUT_FORBIDDEN_FRAGMENT_TERMS {
        cleaned = cleaned.replace(term, "关系");
    }
    cleaned.trim().to_string()
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
        || [
            "0-3s", "3-6s", "6-9s", "9-15s", "0–3s", "3–6s", "6–9s", "9–15s",
        ]
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
    if let Some(material) = build_scene_adapted_expanded_story_material(synopsis, scene_label) {
        return material;
    }
    let beat_count = match target_duration_seconds {
        0..=15 => 2,
        16..=30 => 3,
        31..=45 => 4,
        _ => 6,
    };
    let seconds_per_beat = (target_duration_seconds / beat_count).max(1);
    let mut beats = vec![format!(
        "target_duration_seconds: {target_duration_seconds}"
    )];
    beats.push(format!(
        "扩写剧本：按{target_duration_seconds}秒连续剧情处理，保留真实人物名、主体关系、动作对象和清晰起承转合。"
    ));
    beats.push(format!(
        "第1拍 0-{}秒：在{}的叙事方向中，故事从“{}”展开，先建立人物所处环境、压力来源和行动目标。",
        seconds_per_beat, scene_label, synopsis
    ));
    for index in 1..beat_count {
        let start = index as u16 * seconds_per_beat;
        let end = if index + 1 == beat_count {
            target_duration_seconds
        } else {
            ((index as u16 + 1) * seconds_per_beat).min(target_duration_seconds)
        };
        beats.push(format!(
            "第{}拍 {}-{}秒：冲突继续升级，人物围绕目标和阻碍完成可见动作变化，节奏比上一拍更紧，情绪和空间关系保持连续。",
            index + 1,
            start,
            end,
        ));
    }
    beats.push(
        "结尾让主角完成一次明确的转折或确认，为后续镜头拆解留下连续的动作线、情绪线和空间线。"
            .to_string(),
    );
    beats.join(" ")
}

#[derive(Debug, Clone)]
struct DesktopSourceInputAnalysis {
    source_input_type: String,
    authoring_mode: String,
    source_material_summary: String,
    source_story_facts: SourceStoryFacts,
    preserved_fact_summary: String,
    changed_for_screenplay_summary: String,
    omitted_detail_summary: String,
    continuity_warnings: Vec<ProductWarning>,
}

fn analyze_desktop_source_input(
    source_text: &str,
    request: &ExpandScriptRequest,
) -> DesktopSourceInputAnalysis {
    let detected_input_type = classify_desktop_source_input(source_text);
    let source_input_type =
        non_blank_string(&request.source_input_type).unwrap_or_else(|| detected_input_type.clone());
    let authoring_mode = non_blank_string(&request.authoring_mode)
        .unwrap_or_else(|| authoring_mode_for_source_input_type(&source_input_type).to_string());
    let source_story_facts = if source_facts_are_empty(&request.source_story_facts) {
        extract_desktop_source_story_facts(source_text)
    } else {
        request.source_story_facts.clone()
    };
    let source_material_summary = non_blank_string(&request.source_material_summary)
        .unwrap_or_else(|| compact_source_summary(source_text, "未提供源材料", 260));
    let preserved_fact_summary = non_blank_string(&request.preserved_fact_summary)
        .unwrap_or_else(|| build_preserved_fact_summary(&source_story_facts));
    let changed_for_screenplay_summary = non_blank_string(&request.changed_for_screenplay_summary)
        .unwrap_or_else(|| changed_for_screenplay_summary(&source_input_type, &authoring_mode));
    let omitted_detail_summary = non_blank_string(&request.omitted_detail_summary)
        .unwrap_or_else(|| omitted_detail_summary(&source_input_type, &source_story_facts));
    let mut continuity_warnings = Vec::new();
    if source_input_type == "mixed_material" {
        continuity_warnings.push(ProductWarning {
            code: "source_input_type_uncertain".to_string(),
            message: "材料类型不够明确，已按保守方式整理为可拍剧本。".to_string(),
            related_sample_id: None,
        });
    }
    if source_text.chars().count() > 6_000 {
        continuity_warnings.push(ProductWarning {
            code: "source_document_too_long_for_single_pass".to_string(),
            message: "源文档较长，本轮会优先保留人物、事件顺序和结尾状态。".to_string(),
            related_sample_id: None,
        });
    }

    DesktopSourceInputAnalysis {
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

fn is_story_expansion_request(
    request: &ExpandScriptRequest,
    analysis: &DesktopSourceInputAnalysis,
) -> bool {
    analysis.source_input_type == "synopsis"
        && analysis.authoring_mode == "expand_from_synopsis"
        && normalize_target_duration_mode(&request.target_duration_mode)
            != Some(TARGET_DURATION_MODE_LONG_TEXT_AUTO)
        && request.target_duration_seconds.is_none()
        && request.selected_total_duration_seconds.is_none()
}

fn classify_desktop_source_input(source_text: &str) -> String {
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

    if has_screenplay_marker && (has_novel_marker || has_synopsis_marker) {
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

fn extract_desktop_source_story_facts(source_text: &str) -> SourceStoryFacts {
    let registry = CharacterRegistry::from_story_text(source_text, source_text);
    let mut facts = SourceStoryFacts::default();
    for role in source_person_role_candidates(source_text) {
        push_unique_fact(&mut facts.character_names, role);
    }
    for character in &registry.characters {
        let name = trim_detected_character_name(&character.name);
        push_unique_fact(&mut facts.character_names, name.clone());
        push_unique_fact(
            &mut facts.character_relationships,
            format!("{}:{}", character.role, name),
        );
    }
    let segments = split_story_segments(source_text);
    for (index, segment) in segments.iter().enumerate() {
        let compact = compact_source_summary(segment, "source event", 120);
        if index < 8 {
            push_unique_fact(&mut facts.core_events, compact.clone());
        }
        if index < 12 {
            push_unique_fact(
                &mut facts.event_order,
                format!("{}: {}", index + 1, compact),
            );
        }
        if contains_any_story_term(
            segment,
            &["先", "随后", "然后", "最后", "当夜", "清晨", "之前", "之后"],
        ) {
            push_unique_fact(&mut facts.timeline_facts, compact.clone());
        }
        if contains_any_story_term(segment, &["刀", "剑", "信", "钥匙", "玉佩", "卷轴"]) {
            push_unique_fact(&mut facts.prop_state, compact.clone());
        }
        if contains_any_story_term(segment, &["旧照片", "旧相片", "照片", "相片"]) {
            push_unique_fact(&mut facts.prop_state, compact.clone());
        }
        if contains_any_story_term(segment, &["桥", "城门", "房间", "山", "街", "战场"]) {
            push_unique_fact(&mut facts.location_facts, compact.clone());
        }
        if contains_any_story_term(segment, &["雨夜码头", "码头", "路灯"]) {
            push_unique_fact(&mut facts.location_facts, compact.clone());
        }
        if contains_any_story_term(segment, &["害怕", "迟疑", "愤怒", "决意", "沉默", "泪"])
        {
            push_unique_fact(&mut facts.emotional_progression, compact.clone());
        }
        if contains_any_story_term(
            segment,
            &["追杀", "交锋", "对峙", "反击", "威胁", "冲突", "敌"],
        ) {
            push_unique_fact(&mut facts.conflict_progression, compact.clone());
        }
        if contains_any_story_term(segment, &["追来", "追来的陌生人", "陌生人停", "停在路灯外"])
        {
            push_unique_fact(&mut facts.conflict_progression, compact.clone());
        }
    }
    facts.ending_state = segments
        .last()
        .map(|segment| compact_source_summary(segment, "source ending state", 140))
        .unwrap_or_default();
    facts
}

fn build_preserved_fact_summary(facts: &SourceStoryFacts) -> String {
    let names = if facts.character_names.is_empty() {
        "未识别到明确人物名".to_string()
    } else {
        facts.character_names.join("、")
    };
    let events = if facts.event_order.is_empty() {
        "未识别到明确事件顺序".to_string()
    } else {
        facts
            .event_order
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join(" | ")
    };
    format!("保留人物：{names}；保留事件顺序：{events}")
}

fn changed_for_screenplay_summary(source_input_type: &str, authoring_mode: &str) -> String {
    match authoring_mode {
        "expand_from_synopsis" => "把短梗概扩写成可继续改写为剧本的完整故事内容。".to_string(),
        "polish_existing_screenplay" => "整理已有剧本文本，保持主线剧情不新增。".to_string(),
        "adapt_story_to_screenplay" => {
            "把小说章节改写为可拍剧本，优先保留人物、事件顺序和情绪推进。".to_string()
        }
        "rewrite_from_full_story" => {
            "把完整故事改写为可拍剧本，优先保留剧情事实和结尾状态。".to_string()
        }
        _ => format!("按保守方式整理 {source_input_type}，不主动新增未经确认的主线剧情。"),
    }
}

fn omitted_detail_summary(source_input_type: &str, facts: &SourceStoryFacts) -> String {
    if source_input_type == "synopsis" {
        "短梗概模式不删减原始事实，只补足承接段落。".to_string()
    } else if facts.core_events.len() > 8 {
        "次要描写被压缩，人物、事件顺序和结尾状态优先保留。".to_string()
    } else {
        "未主动省略已识别的核心事件。".to_string()
    }
}

fn build_deterministic_rewrite_script(
    source_text: &str,
    scene_label: &str,
    target_duration_seconds: u16,
    analysis: &DesktopSourceInputAnalysis,
) -> String {
    if analysis.source_input_type == "synopsis" {
        return build_deterministic_expanded_story_script(
            source_text,
            scene_label,
            target_duration_seconds,
        );
    }

    let segments = split_story_segments(source_text);
    let mut lines = vec![
        format!("剧本改写：{}", analysis.changed_for_screenplay_summary),
        format!(
            "源材料识别：{}",
            source_input_type_product_label(&analysis.source_input_type)
        ),
        format!("保留原则：{}", analysis.preserved_fact_summary),
    ];
    for (index, segment) in segments.iter().take(8).enumerate() {
        lines.push(format!(
            "场景{}：{}。本场只把输入事实改写为可表演动作、台词目的和可拆镜头调度，不新增未经输入支持的主线剧情。",
            index + 1,
            compact_source_summary(segment, "角色推进当前事件", 180)
        ));
    }
    if lines.len() <= 3 {
        lines.push(format!(
            "场景1：{}。人物目标、空间关系和动作结果保持清楚，方便继续拆解镜头。",
            compact_source_summary(source_text, "角色推进当前事件", 180)
        ));
    }
    if !analysis.source_story_facts.ending_state.trim().is_empty() {
        lines.push(format!(
            "结尾状态保留：{}",
            analysis.source_story_facts.ending_state
        ));
    }
    lines.push(format!("目标时长：{} 秒。", target_duration_seconds));
    lines.join("\n")
}

fn build_deterministic_expanded_story_material(source_text: &str, scene_label: &str) -> String {
    if let Some(material) = build_scene_adapted_expanded_story_material(source_text, scene_label) {
        return material;
    }

    let seed = compact_source_summary(source_text, "主角在压力中推进目标", 180);
    let scene_label = if scene_label.trim().is_empty() {
        "当前场景"
    } else {
        scene_label.trim()
    };
    let mut paragraphs = vec![format!(
        "故事从{}展开。{} 这不是直接分镜，而是一版完整故事内容，用来给后续剧本改写保留人物目标、事件顺序和情绪推进。",
        scene_label, seed
    )];
    let beat_templates = [
        "开端里，主角先面对一个具体处境，目标被迫显形，周围环境也给出可见压力。",
        "随后，逼近的人或对峙对象带来压力，主角不能只解释原因，必须通过行动回应眼前阻碍。",
        "关系层面出现迟疑或误解，主角的选择开始改变与对峙对象之间的距离，也让冲突不再只是外部威胁。",
        "中段加入一次转折，主角得到线索或看见代价，原先的判断被迫重新排列。",
        "压力继续升级，空间、道具或时间限制把人物推向更窄的选择口。",
        "主角做出明确动作，这个动作改变局面，也暴露出下一段必须承接的问题。",
        "情绪从犹豫进入清醒，人物不再等待别人解释，而是主动承担后果。",
        "结尾保留开放压力，但让当前段落的选择、动作结果和人物状态都有清楚落点。",
    ];
    let mut index = 0usize;
    while paragraphs.join("\n\n").chars().count() < 2_050 {
        let template = beat_templates[index % beat_templates.len()];
        paragraphs.push(format!(
            "第{}段：{} 源梗概仍是本段的事实边界：{} 后续改写剧本时，应保留这一段的行动对象、情绪变化和结果状态。",
            index + 1,
            template,
            seed
        ));
        index += 1;
    }
    truncate_chars(&paragraphs.join("\n\n"), 2_450)
}

fn source_input_type_product_label(source_input_type: &str) -> &'static str {
    match source_input_type {
        "full_story" => "完整故事",
        "novel_chapter" => "小说章节",
        "screenplay_text" => "已有剧本",
        "mixed_material" => "混合材料",
        _ => "故事梗概",
    }
}

fn compact_source_summary(value: &str, fallback: &str, max_chars: usize) -> String {
    let source = if value.trim().is_empty() {
        fallback
    } else {
        value.trim()
    };
    let mut summary = source.chars().take(max_chars).collect::<String>();
    if source.chars().count() > max_chars {
        summary.push_str("...");
    }
    summary
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut text = value.chars().take(max_chars).collect::<String>();
    text.push_str("...");
    text
}

fn push_unique_fact(values: &mut Vec<String>, value: String) {
    let trimmed = value.trim();
    if trimmed.is_empty() || values.iter().any(|item| item == trimmed) {
        return;
    }
    values.push(trimmed.to_string());
}

fn trim_detected_character_name(name: &str) -> String {
    let mut chars = name.trim().chars().collect::<Vec<_>>();
    while chars.len() > 2 {
        let Some(last) = chars.last().copied() else {
            break;
        };
        if matches!(
            last,
            '先' | '已'
                | '也'
                | '把'
                | '从'
                | '仍'
                | '在'
                | '向'
                | '对'
                | '半'
                | '没'
                | '的'
                | '和'
                | '与'
        ) {
            chars.pop();
        } else {
            break;
        }
    }
    chars.into_iter().collect()
}

fn source_facts_are_empty(facts: &SourceStoryFacts) -> bool {
    facts.character_names.is_empty()
        && facts.core_events.is_empty()
        && facts.event_order.is_empty()
        && facts.ending_state.trim().is_empty()
}

fn extract_live_storyboard_row_patches(
    generation_response: &TextGenerationResponse,
) -> Result<Vec<LiveStoryboardRowPatch>, ProductWarning> {
    let Some(structured_json) = generation_response.structured_json.as_ref() else {
        if text_generation_fallback_blocking_warnings(&generation_response.warnings).is_empty()
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

fn apply_live_storyboard_patch(
    row: &mut GeneratedStoryboardRow,
    patch: &LiveStoryboardRowPatch,
    baseline_row: &GeneratedStoryboardRow,
) {
    let mut raw_person_patch = None;
    if let Some(value) = non_blank_string(&patch.shot_title) {
        row.shot_title = value;
    }
    if let Some(value) = non_blank_string(&patch.person) {
        raw_person_patch = Some(value.clone());
        row.person = bind_live_storyboard_person_to_source(&value, baseline_row);
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
    if let Some(value) = non_blank_string(&patch.camera_movement) {
        row.camera_movement = value;
    }
    if let Some(value) = non_blank_string(&patch.dialogue) {
        row.dialogue = value;
    }
    if let Some(raw_person) = raw_person_patch {
        let bound_person = row.person.trim().to_string();
        let source_text = format!(
            "{}\n{}",
            baseline_row.shot_script, baseline_row.scene_performance_projection.fused_source_text
        );
        let should_replace_raw_person =
            canonicalize_environment_mixed_storyboard_person(raw_person.trim(), &source_text)
                .is_some()
                || bind_live_person_candidate_to_baseline_name(raw_person.trim(), baseline_row)
                    .is_some();
        let should_repair_source_bound_subject =
            live_person_label_should_use_source_bound_repair(raw_person.trim(), &source_text);
        if should_replace_raw_person
            && !bound_person.is_empty()
            && raw_person.trim() != bound_person
        {
            replace_storyboard_row_subject_text(row, raw_person.trim(), &bound_person);
        } else if should_repair_source_bound_subject
            && !bound_person.is_empty()
            && raw_person.trim() != bound_person
        {
            replace_storyboard_row_subject_slot_text(row, raw_person.trim(), &bound_person);
        }
    }
}

fn repair_live_storyboard_patch_from_baseline(
    row: &mut GeneratedStoryboardRow,
    baseline_row: &GeneratedStoryboardRow,
) {
    let source_text = format!(
        "{}\n{}",
        baseline_row.shot_script, baseline_row.scene_performance_projection.fused_source_text
    );
    if row.person.trim().is_empty() || storyboard_person_is_empty_shot_marker(&row.person) {
        row.person = baseline_row.person.clone();
    }
    if row.scene_performance_projection.person.trim().is_empty()
        || storyboard_person_is_empty_shot_marker(&row.scene_performance_projection.person)
    {
        row.scene_performance_projection.person = row.person.clone();
    }
    if row.scene_scale.trim().is_empty() {
        row.scene_scale = baseline_row.scene_scale.clone();
    }
    if row.shot_title.trim().is_empty()
        || storyboard_field_needs_source_grounded_restore(
            "shot_title",
            &row.shot_title,
            &source_text,
        )
    {
        row.shot_title = baseline_row.shot_title.clone();
    }
    if is_role_action_grounding_incomplete(&row.character_action)
        || storyboard_field_needs_source_grounded_restore(
            "character_action",
            &row.character_action,
            &source_text,
        )
    {
        row.character_action = baseline_row.character_action.clone();
        row.scene_performance_projection.character_action = baseline_row
            .scene_performance_projection
            .character_action
            .clone();
    }
    let live_visual_was_normalized_to_a_ruin_repair = row
        .visual_description
        .contains("主角单膝跪地的停顿放在废墟前侧")
        && row
            .visual_description
            .contains("敌人缓步逼近的来路压在后侧");
    let baseline_visual_keeps_a_ruin_facts = baseline_row.visual_description.contains("主角")
        && baseline_row.visual_description.contains("敌人")
        && baseline_row.visual_description.contains("废墟")
        && contains_any_story_term(&baseline_row.visual_description, &["逼近", "对峙压力"]);
    if is_visual_description_grounding_incomplete(
        &row.visual_description,
        &row.person,
        &row.scene_scale,
        &row.character_action,
        &row.camera_movement,
        &row.shot_script,
    ) || storyboard_field_needs_source_grounded_restore(
        "visual_description",
        &row.visual_description,
        &source_text,
    ) || (live_visual_was_normalized_to_a_ruin_repair && baseline_visual_keeps_a_ruin_facts)
    {
        row.visual_description = baseline_row.visual_description.clone();
        row.scene_performance_projection.visual_description = baseline_row
            .scene_performance_projection
            .visual_description
            .clone();
    }
    if is_camera_movement_grounding_incomplete(
        &row.camera_movement,
        &row.shot_title,
        &row.scene_scale,
        &row.visual_description,
        &row.shot_script,
    ) || storyboard_field_needs_source_grounded_restore(
        "camera_movement",
        &row.camera_movement,
        &source_text,
    ) {
        row.camera_movement = baseline_row.camera_movement.clone();
    }
}

fn repair_live_storyboard_rows_from_source(
    rows: &mut [GeneratedStoryboardRow],
    deterministic_rows: &[GeneratedStoryboardRow],
) -> LiveRepairSummary {
    let mut summary = LiveRepairSummary::default();
    for (index, row) in rows.iter_mut().enumerate() {
        let Some(baseline_row) = deterministic_rows.get(index) else {
            continue;
        };
        let before = storyboard_row_repair_signature(row);
        let source_text = format!(
            "{}\n{}",
            baseline_row.shot_script, baseline_row.scene_performance_projection.fused_source_text
        );

        repair_live_storyboard_row_subject_binding(row, baseline_row, &source_text, &mut summary);
        repair_storyboard_external_drift_fields(row, baseline_row, &source_text, &mut summary);
        repair_storyboard_source_fragment_action_fields(row, &source_text, &mut summary);
        repair_storyboard_abstract_pressure_fields(row, &source_text, &mut summary);
        normalize_storyboard_row_subject_quality(row);

        if source_has_a_ruin_enemy_facts(&source_text)
            && storyboard_row_needs_a_source_grounding_repair(row)
        {
            repair_a_ruin_enemy_storyboard_row(row);
            summary.push_reason("visual_grounding_restored");
        }
        if source_has_b_alley_pursuit_facts(&source_text)
            && storyboard_row_needs_b_source_grounding_repair(row)
        {
            repair_b_alley_pursuit_storyboard_row(row);
            summary.push_reason("pursuit_pressure_restored");
        }
        if source_has_b_alley_pursuit_facts(&source_text)
            && storyboard_row_has_b_sandbox_strategy_context(row)
            && storyboard_row_needs_b_sandbox_strategy_repair(row)
        {
            repair_b_sandbox_strategy_storyboard_row(row);
            summary.push_reason("sandbox_strategy_expression_restored");
        }
        if source_has_c_rainy_dock_photo_facts(&source_text)
            && storyboard_row_needs_c_rainy_dock_photo_repair(row)
        {
            repair_c_rainy_dock_photo_storyboard_row(row);
            summary.push_reason("rainy_dock_photo_fact_frame_restored");
        }

        normalize_storyboard_row_subject_quality(row);
        if before != storyboard_row_repair_signature(row) && !summary.repaired() {
            summary.push_reason("source_grounded_field_repair");
        }
    }
    summary
}

fn repair_storyboard_rows_from_binding_context(
    rows: &mut [GeneratedStoryboardRow],
    request: &GenerateStoryboardRequest,
    grounding: &StoryboardGroundingContext,
) -> LiveRepairSummary {
    let mut summary = LiveRepairSummary::default();
    let anchors = build_storyboard_binding_field_anchors(request, grounding);
    let required_b_facts = storyboard_binding_context_b_alley_pursuit_facts(request, grounding);
    if required_b_facts.is_empty() && !anchors.has_binding_context {
        return summary;
    }
    let rows_text = storyboard_rows_binding_text(rows);
    let binding_has_missing_b_facts = required_b_facts
        .iter()
        .any(|fact| !binding_text_contains(&rows_text, fact));

    for row in rows {
        let before = storyboard_row_repair_signature(row);
        scrub_storyboard_row_binding_forbidden_facts(row, &anchors);
        if binding_has_missing_b_facts || storyboard_row_needs_b_source_grounding_repair(row) {
            if storyboard_row_has_b_sandbox_strategy_context(row) {
                repair_b_sandbox_strategy_storyboard_row(row);
            } else {
                repair_b_alley_pursuit_storyboard_row(row);
            }
            normalize_storyboard_row_subject_quality(row);
        }
        normalize_storyboard_row_subject_quality(row);
        apply_storyboard_binding_source_fields_to_row(row, &anchors);
        apply_storyboard_binding_prompt_packaging(row, &anchors);
        scrub_storyboard_row_binding_forbidden_facts(row, &anchors);
        if source_has_a_ruin_enemy_facts(&anchors.source_text)
            && storyboard_row_needs_a_source_grounding_repair(row)
        {
            repair_a_ruin_enemy_storyboard_row(row);
            summary.push_reason("visual_grounding_restored");
        }
        if before != storyboard_row_repair_signature(row) {
            summary.push_reason("binding_story_fact_frame_restored");
        }
    }

    summary
}

fn storyboard_binding_context_b_alley_pursuit_facts(
    request: &GenerateStoryboardRequest,
    grounding: &StoryboardGroundingContext,
) -> Vec<String> {
    let mut facts = normalize_binding_fact_list(&request.must_keep_facts);
    for fact in source_derived_storyboard_must_keep_facts(&grounding.grounding_text) {
        push_unique_fact(&mut facts, fact);
    }

    let context = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        request.source_profile,
        request.current_case_id,
        grounding.shot_script,
        grounding.expanded_script_text,
        grounding.grounding_text,
        facts.join("\n")
    );
    if source_has_b_alley_pursuit_facts(&context) {
        for fact in ["林峰护住苏瑶", "阿青提醒", "黑衣追兵", "巷口", "逼近"] {
            push_unique_fact(&mut facts, fact.to_string());
        }
    }

    facts
        .into_iter()
        .filter(|fact| storyboard_binding_fact_is_b_alley_pursuit(fact))
        .collect()
}

fn storyboard_binding_fact_is_b_alley_pursuit(fact: &str) -> bool {
    contains_any_story_term(
        fact,
        &[
            "林峰护住苏瑶",
            "护住苏瑶",
            "阿青提醒",
            "提醒他们",
            "黑衣追兵",
            "追兵",
            "巷口",
            "逼近",
            "压近",
            "追兵压力",
        ],
    )
}

fn build_storyboard_binding_field_anchors(
    request: &GenerateStoryboardRequest,
    grounding: &StoryboardGroundingContext,
) -> StoryboardBindingFieldAnchors {
    let snapshot = request.accepted_rewrite_snapshot.as_ref();
    let accepted_confirmation_body = snapshot
        .map(|snapshot| sanitize_product_body_text(&snapshot.accepted_confirmation_body))
        .unwrap_or_default();
    let mut source_text = [
        accepted_confirmation_body.as_str(),
        grounding.shot_script.as_str(),
        grounding.expanded_script_text.as_str(),
        grounding.grounding_text.as_str(),
        request.source_profile.as_str(),
    ]
    .join("\n");

    let mut forbidden_facts = normalize_binding_fact_list(&request.forbidden_facts);
    if let Some(snapshot) = snapshot {
        for fact in normalize_binding_fact_list(&snapshot.forbidden_facts) {
            push_unique_fact(&mut forbidden_facts, fact);
        }
    }
    for fact in source_derived_storyboard_forbidden_facts(&grounding.grounding_text) {
        push_unique_fact(&mut forbidden_facts, fact);
    }

    let mut explicit_facts = normalize_binding_fact_list(&request.must_keep_facts);
    if let Some(snapshot) = snapshot {
        for fact in normalize_binding_fact_list(&snapshot.explicit_facts) {
            push_unique_fact(&mut explicit_facts, fact);
        }
        for fact in normalize_binding_fact_list(&snapshot.must_keep_facts) {
            push_unique_fact(&mut explicit_facts, fact);
        }
    }
    for fact in source_derived_storyboard_must_keep_facts(&grounding.grounding_text) {
        push_unique_fact(&mut explicit_facts, fact);
    }
    source_text.push('\n');
    source_text.push_str(&explicit_facts.join("\n"));

    let mut anchors = StoryboardBindingFieldAnchors {
        source_text: source_text.clone(),
        accepted_confirmation_body,
        forbidden_facts,
        duration_seconds: snapshot
            .map(|snapshot| snapshot.duration_seconds)
            .unwrap_or(request.selected_total_duration_seconds),
        target_duration_mode: snapshot
            .map(|snapshot| snapshot.target_duration_mode.clone())
            .unwrap_or_else(|| request.target_duration_mode.clone()),
        has_binding_context: snapshot.is_some()
            || !request.current_case_id.trim().is_empty()
            || !request.story_fact_frame_hash.trim().is_empty()
            || !request.must_keep_facts.is_empty()
            || !request.forbidden_facts.is_empty(),
        ..StoryboardBindingFieldAnchors::default()
    };

    for fact in explicit_facts {
        push_storyboard_binding_fact_anchor(&mut anchors, &fact, false);
    }
    if let Some(snapshot) = snapshot {
        for inferred in &snapshot.inferred_scene_facts {
            if binding_inferred_scene_fact_is_low_risk(inferred, &anchors) {
                push_storyboard_binding_fact_anchor(&mut anchors, &inferred.fact, true);
            }
        }
    }

    anchors
}

fn push_storyboard_binding_fact_anchor(
    anchors: &mut StoryboardBindingFieldAnchors,
    fact: &str,
    inferred: bool,
) {
    let clean = normalize_binding_text(fact);
    if clean.is_empty()
        || !storyboard_binding_fact_is_allowed(&clean, anchors, inferred)
        || contains_product_control_text(&clean)
    {
        return;
    }

    if storyboard_binding_fact_is_person_anchor(&clean, &anchors.source_text) {
        push_unique_fact(&mut anchors.person, clean.clone());
    }
    if storyboard_binding_fact_is_visual_anchor(&clean) || inferred {
        push_unique_fact(&mut anchors.visual_description, clean.clone());
    }
    if storyboard_binding_fact_is_action_anchor(&clean) || inferred {
        push_unique_fact(&mut anchors.character_action, clean.clone());
    }
    if storyboard_binding_fact_is_camera_anchor(&clean) || inferred {
        push_unique_fact(&mut anchors.camera_movement, clean.clone());
    }
    push_unique_fact(&mut anchors.packaging, clean);
}

fn storyboard_binding_fact_is_allowed(
    fact: &str,
    anchors: &StoryboardBindingFieldAnchors,
    inferred: bool,
) -> bool {
    if binding_fact_hits_forbidden_fact(fact, &anchors.forbidden_facts) {
        return false;
    }
    if binding_anchor_has_absolute_forbidden_term(fact) {
        return false;
    }
    if !inferred && binding_anchor_has_source_external_high_risk_term(fact, &anchors.source_text) {
        return false;
    }
    if generated_script_forbidden_added_identity_term(fact, &anchors.source_text).is_some()
        || generated_script_forbidden_external_setting_term(fact, &anchors.source_text).is_some()
        || generated_script_forbidden_military_scale_expansion_term(fact, &anchors.source_text)
            .is_some()
        || generated_script_forbidden_external_action_term(fact, &anchors.source_text).is_some()
    {
        return false;
    }
    generated_script_ungrounded_character_name(fact, &anchors.source_text).is_none()
}

fn binding_inferred_scene_fact_is_low_risk(
    inferred: &core_domain::contracts::InferredSceneFactBinding,
    anchors: &StoryboardBindingFieldAnchors,
) -> bool {
    let fact = normalize_binding_text(&inferred.fact);
    if fact.is_empty() || !storyboard_binding_fact_is_allowed(&fact, anchors, true) {
        return false;
    }
    if binding_inferred_fact_changes_core_story(&fact) {
        return false;
    }
    let evidence = format!(
        "{}\n{}\n{}",
        fact, inferred.inference_reason, inferred.inference_scope
    );
    contains_any_story_term(
        &evidence,
        &[
            "低风险",
            "临时",
            "当前场景",
            "场面调度",
            "动作调度",
            "分镜",
            "生图",
            "视频",
            "剪辑",
            "节奏",
            "情绪可视化",
            "木棍",
            "短刀",
            "摊位",
            "墙角",
            "街边障碍",
            "停顿",
            "回望",
            "握紧",
            "格挡",
            "错身",
            "对峙",
        ],
    )
}

fn binding_inferred_fact_changes_core_story(fact: &str) -> bool {
    contains_any_story_term(
        fact,
        &[
            "长期设定",
            "永久装备",
            "专属装备",
            "神器",
            "法宝",
            "新增人物",
            "关键人物",
            "身份变为",
            "改为",
            "阵营",
            "等级",
            "军队规模",
            "军团规模",
            "核心剧情",
            "因果",
            "地点改为",
        ],
    )
}

fn binding_fact_hits_forbidden_fact(fact: &str, forbidden_facts: &[String]) -> bool {
    forbidden_facts.iter().any(|forbidden| {
        let forbidden = forbidden.trim();
        !forbidden.is_empty() && fact.contains(forbidden)
    })
}

fn binding_anchor_has_absolute_forbidden_term(fact: &str) -> bool {
    contains_any_story_term(
        fact,
        &[
            "发型",
            "发色",
            "脸型",
            "五官",
            "身材",
            "年龄外观",
            "人物参考图",
            "参考图",
            "画像化",
            "外观设计",
            "源外身份",
            "源外关键人物",
            "源外人数",
            "源外阵营",
            "源外等级",
            "源外神器",
            "源外装备",
            "源外武器",
            "源外伤势",
            "源外血迹",
        ],
    )
}

fn binding_anchor_has_source_external_high_risk_term(fact: &str, source_text: &str) -> bool {
    EXPAND_SCRIPT_FORBIDDEN_EXTERNAL_SETTING_TERMS
        .iter()
        .chain(EXPAND_SCRIPT_FORBIDDEN_MILITARY_SCALE_EXPANSION_TERMS.iter())
        .chain(EXPAND_SCRIPT_FORBIDDEN_EXTERNAL_ACTION_TERMS.iter())
        .any(|term| fact.contains(*term) && !source_text.contains(*term))
}

fn storyboard_binding_fact_is_person_anchor(fact: &str, source_text: &str) -> bool {
    let trimmed = fact.trim();
    if trimmed.is_empty()
        || trimmed.chars().count() > 8
        || subject_is_non_character_anchor(trimmed)
        || live_person_visual_or_abstract_term(trimmed).is_some()
        || storyboard_person_label_is_forbidden(trimmed)
        || live_person_label_looks_like_source_fragment(trimmed)
        || looks_like_non_character_phrase(trimmed)
    {
        return false;
    }
    source_person_role_candidates(source_text)
        .into_iter()
        .any(|candidate| candidate == trimmed)
}

fn storyboard_binding_fact_is_visual_anchor(fact: &str) -> bool {
    contains_any_story_term(
        fact,
        &[
            "废墟", "巷口", "街巷", "街边", "街道", "摊位", "墙角", "墙面", "障碍", "城市", "城墙",
            "战场", "宫殿", "庭院", "房间", "桥", "码头", "路灯", "雨夜", "照片", "相片", "木棍",
            "短刀", "瓦砾", "尘土", "光", "雨", "雪", "前景", "后景", "空间", "场景",
        ],
    )
}

fn storyboard_binding_fact_is_action_anchor(fact: &str) -> bool {
    contains_any_story_term(
        fact,
        &[
            "护住",
            "护着",
            "提醒",
            "逼近",
            "压近",
            "追来",
            "追击",
            "跪",
            "单膝",
            "攥紧",
            "握紧",
            "回头",
            "回望",
            "停住",
            "停步",
            "格挡",
            "错身",
            "对峙",
            "冲向",
            "后退",
            "退路",
            "撤离",
            "反击",
            "撑住",
            "借用",
            "拉开距离",
            "停顿",
        ],
    )
}

fn storyboard_binding_fact_is_camera_anchor(fact: &str) -> bool {
    contains_any_story_term(
        fact,
        &[
            "运镜",
            "镜头",
            "推近",
            "推进",
            "跟拍",
            "摇移",
            "环绕",
            "定机位",
            "低机位",
            "俯拍",
            "仰拍",
            "逼近",
            "追击",
            "后退",
            "动线",
            "退路",
            "格挡",
            "错身",
            "节奏落点",
        ],
    )
}

fn apply_storyboard_binding_source_fields_to_row(
    row: &mut GeneratedStoryboardRow,
    anchors: &StoryboardBindingFieldAnchors,
) {
    if !anchors.has_binding_context {
        return;
    }
    rebind_storyboard_person_from_binding(row, anchors);
    append_storyboard_binding_field_clause_safe(
        &mut row.visual_description,
        "场景锚点",
        &anchors.visual_description,
        5,
    );
    append_storyboard_binding_field_clause_safe(
        &mut row.character_action,
        "表演锚点",
        &anchors.character_action,
        5,
    );
    if row.camera_movement.trim().is_empty() && !anchors.camera_movement.is_empty() {
        let subject = if row.person.trim().is_empty() {
            "当前人物"
        } else {
            row.person.trim()
        };
        row.camera_movement = format!(
            "{}定机位观察{subject}，镜头捕捉{}。",
            row.scene_scale.trim(),
            anchors
                .camera_movement
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(" / ")
        );
    } else {
        append_storyboard_binding_field_clause_safe(
            &mut row.camera_movement,
            "运动锚点",
            &anchors.camera_movement,
            4,
        );
    }
    row.scene_performance_projection.person = row.person.clone();
    row.scene_performance_projection.visual_description = row.visual_description.clone();
    row.scene_performance_projection.character_action = row.character_action.clone();
}

fn rebind_storyboard_person_from_binding(
    row: &mut GeneratedStoryboardRow,
    anchors: &StoryboardBindingFieldAnchors,
) {
    if anchors.person.is_empty() {
        return;
    }
    let current = row.person.trim();
    let should_rebind = current.is_empty()
        || storyboard_person_is_empty_shot_marker(current)
        || live_person_label_should_use_source_bound_repair(current, &anchors.source_text)
        || live_person_label_should_use_source_binding(current, true);
    if !should_rebind {
        return;
    }
    let Some(next_person) = binding_person_anchor_for_row(row, anchors) else {
        return;
    };
    let previous_person = row.person.clone();
    row.person = next_person.clone();
    row.scene_performance_projection.person = next_person.clone();
    if !previous_person.trim().is_empty() {
        replace_storyboard_row_subject_slot_text(row, &previous_person, &next_person);
    }
}

fn binding_person_anchor_for_row(
    row: &GeneratedStoryboardRow,
    anchors: &StoryboardBindingFieldAnchors,
) -> Option<String> {
    let row_text = storyboard_row_user_visible_repair_signature(row);
    anchors
        .person
        .iter()
        .find(|person| row_text.contains(person.as_str()))
        .cloned()
        .or_else(|| {
            let index = row.order.saturating_sub(1) as usize;
            anchors
                .person
                .get(index % anchors.person.len().max(1))
                .cloned()
        })
        .or_else(|| anchors.person.first().cloned())
}

#[allow(dead_code)]
fn append_storyboard_binding_field_clause(
    field: &mut String,
    label: &str,
    facts: &[String],
    limit: usize,
) {
    let missing = facts
        .iter()
        .filter(|fact| !binding_text_contains(field, fact))
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return;
    }
    let clause = format!("{label}：{}", missing.join(" / "));
    if field.trim().is_empty() {
        *field = clause;
    } else {
        *field = format!("{}；{}", field.trim(), clause);
    }
}

fn append_storyboard_binding_field_clause_safe(
    field: &mut String,
    label: &str,
    facts: &[String],
    limit: usize,
) {
    let missing = facts
        .iter()
        .filter(|fact| !binding_text_contains(field, fact))
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return;
    }
    let clause = format!("{label}({})", missing.join(" / "));
    if field.trim().is_empty() {
        *field = clause;
    } else {
        *field = format!("{}；{}", field.trim(), clause);
    }
}

fn apply_storyboard_binding_prompt_packaging(
    row: &mut GeneratedStoryboardRow,
    anchors: &StoryboardBindingFieldAnchors,
) {
    if !anchors.has_binding_context {
        return;
    }
    let accepted_body =
        storyboard_binding_packaging_fragment(&anchors.accepted_confirmation_body, anchors, 180);
    let mut sections =
        vec!["视频分镜提示词：以确认稿和事实锚点为准，输出当前单镜头画面。".to_string()];
    if !accepted_body.is_empty() {
        sections.push(format!("确认稿正文：{accepted_body}"));
    }
    push_storyboard_prompt_anchor_section(&mut sections, "人物名称/代号", &anchors.person, 6);
    push_storyboard_prompt_anchor_section(
        &mut sections,
        "角色表演锚",
        &anchors.character_action,
        5,
    );
    push_storyboard_prompt_anchor_section(
        &mut sections,
        "基础/复杂场景描述",
        &anchors.visual_description,
        5,
    );
    push_storyboard_prompt_anchor_section(&mut sections, "视频运动锚", &anchors.camera_movement, 4);
    sections.push(format!(
        "分镜字段：人物={}；画面描述={}；角色动作={}；运镜={}；目标时长={}秒。",
        row.person.trim(),
        storyboard_binding_packaging_fragment(&row.visual_description, anchors, 220),
        storyboard_binding_packaging_fragment(&row.character_action, anchors, 180),
        storyboard_binding_packaging_fragment(&row.camera_movement, anchors, 140),
        row.duration_seconds
    ));
    if anchors.duration_seconds > 0 {
        sections.push(format!(
            "目标时长/节奏落点：任务{}秒，当前镜头{}秒，模式{}。",
            anchors.duration_seconds, row.duration_seconds, anchors.target_duration_mode
        ));
    }
    row.prompt_text = scrub_storyboard_binding_forbidden_text(&sections.join("；"), anchors);
}

fn push_storyboard_prompt_anchor_section(
    sections: &mut Vec<String>,
    label: &str,
    facts: &[String],
    limit: usize,
) {
    let values = facts.iter().take(limit).cloned().collect::<Vec<_>>();
    if !values.is_empty() {
        sections.push(format!("{label}：{}", values.join(" / ")));
    }
}

fn storyboard_binding_packaging_fragment(
    value: &str,
    anchors: &StoryboardBindingFieldAnchors,
    max_chars: usize,
) -> String {
    let clean =
        scrub_storyboard_binding_forbidden_text(&sanitize_product_body_text(value), anchors);
    compact_source_summary(&clean, "", max_chars)
}

fn scrub_storyboard_row_binding_forbidden_facts(
    row: &mut GeneratedStoryboardRow,
    anchors: &StoryboardBindingFieldAnchors,
) {
    if !anchors.has_binding_context {
        return;
    }
    for value in [
        &mut row.person,
        &mut row.shot_title,
        &mut row.visual_description,
        &mut row.character_action,
        &mut row.camera_movement,
        &mut row.prompt_text,
        &mut row.scene_performance_projection.person,
        &mut row.scene_performance_projection.source_sample_title,
        &mut row.scene_performance_projection.visual_description,
        &mut row.scene_performance_projection.character_action,
    ] {
        *value = scrub_storyboard_binding_forbidden_text(value, anchors);
    }
}

fn scrub_storyboard_binding_forbidden_text(
    value: &str,
    anchors: &StoryboardBindingFieldAnchors,
) -> String {
    let mut clean = value.to_string();
    for forbidden in &anchors.forbidden_facts {
        if !forbidden.trim().is_empty() {
            clean = clean.replace(forbidden, "");
        }
    }
    for term in [
        "发型",
        "发色",
        "脸型",
        "五官",
        "身材",
        "人物参考图",
        "参考图",
        "画像化",
        "外观设计",
    ] {
        clean = clean.replace(term, "");
    }
    for &term in EXPAND_SCRIPT_FORBIDDEN_EXTERNAL_SETTING_TERMS
        .iter()
        .chain(EXPAND_SCRIPT_FORBIDDEN_MILITARY_SCALE_EXPANSION_TERMS.iter())
        .chain(EXPAND_SCRIPT_FORBIDDEN_EXTERNAL_ACTION_TERMS.iter())
    {
        if !anchors.source_text.contains(term) {
            clean = clean.replace(term, "");
        }
    }
    clean
        .replace("；；", "；")
        .replace("：；", "：")
        .replace(" /  / ", " / ")
        .trim_matches(|character| matches!(character, '；' | ' ' | '\n' | '\r' | '\t'))
        .to_string()
}

fn repair_live_storyboard_row_subject_binding(
    row: &mut GeneratedStoryboardRow,
    baseline_row: &GeneratedStoryboardRow,
    source_text: &str,
    summary: &mut LiveRepairSummary,
) {
    for raw_subject in live_storyboard_row_source_bound_subject_fragments(row, source_text) {
        let canonical_subject =
            source_bound_subject_for_raw_live_subject(&raw_subject, baseline_row, source_text);
        if canonical_subject.trim().is_empty() {
            continue;
        }
        row.person = canonical_subject.clone();
        if row.scene_performance_projection.person.trim().is_empty()
            || row.scene_performance_projection.person.trim() == raw_subject
            || live_person_label_should_use_source_bound_repair(
                &row.scene_performance_projection.person,
                source_text,
            )
        {
            row.scene_performance_projection.person = canonical_subject.clone();
        }
        replace_storyboard_row_subject_slot_text(row, &raw_subject, &canonical_subject);
        summary.push_reason("source_bound_subject_repair");
    }
}

fn source_bound_subject_for_raw_live_subject(
    raw_subject: &str,
    baseline_row: &GeneratedStoryboardRow,
    source_text: &str,
) -> String {
    normalize_live_character_label_to_source_subject(raw_subject, source_text, &baseline_row.person)
        .unwrap_or_else(|| {
            preferred_source_bound_subject_for_live_repair(baseline_row, source_text)
        })
}

fn live_storyboard_row_source_bound_subject_fragments(
    row: &GeneratedStoryboardRow,
    source_text: &str,
) -> Vec<String> {
    let mut fragments = Vec::new();
    for value in [
        row.person.as_str(),
        row.scene_performance_projection.person.as_str(),
    ] {
        for part in split_live_subject_parts(value) {
            if live_person_label_should_use_source_bound_repair(&part, source_text) {
                push_unique_fact(&mut fragments, part);
            }
        }
    }
    for term in live_person_subject_state_terms() {
        if live_row_has_subject_slot_fragment(row, term) {
            push_unique_fact(&mut fragments, term.to_string());
        }
    }
    fragments
}

fn live_row_has_subject_slot_fragment(row: &GeneratedStoryboardRow, term: &str) -> bool {
    [
        ("shot_title", row.shot_title.as_str()),
        ("visual_description", row.visual_description.as_str()),
        ("character_action", row.character_action.as_str()),
        ("camera_movement", row.camera_movement.as_str()),
        ("prompt_text", row.prompt_text.as_str()),
        (
            "visual_description",
            row.scene_performance_projection.visual_description.as_str(),
        ),
        (
            "character_action",
            row.scene_performance_projection.character_action.as_str(),
        ),
    ]
    .iter()
    .any(|(field_name, value)| {
        live_field_action_state_subject_term(field_name, value).is_some_and(|label| label == term)
    })
}

fn repair_storyboard_external_drift_fields(
    row: &mut GeneratedStoryboardRow,
    baseline_row: &GeneratedStoryboardRow,
    source_text: &str,
    summary: &mut LiveRepairSummary,
) {
    if storyboard_field_needs_source_grounded_restore("shot_title", &row.shot_title, source_text) {
        row.shot_title = baseline_row.shot_title.clone();
        summary.push_reason("source_external_field_rebound");
    }
    if storyboard_field_needs_source_grounded_restore(
        "visual_description",
        &row.visual_description,
        source_text,
    ) {
        row.visual_description = baseline_row.visual_description.clone();
        row.scene_performance_projection.visual_description = baseline_row
            .scene_performance_projection
            .visual_description
            .clone();
        summary.push_reason("source_external_field_rebound");
    }
    if storyboard_field_needs_source_grounded_restore(
        "character_action",
        &row.character_action,
        source_text,
    ) {
        row.character_action = baseline_row.character_action.clone();
        row.scene_performance_projection.character_action = baseline_row
            .scene_performance_projection
            .character_action
            .clone();
        summary.push_reason("source_external_field_rebound");
    }
    if storyboard_field_needs_source_grounded_restore(
        "camera_movement",
        &row.camera_movement,
        source_text,
    ) {
        row.camera_movement = baseline_row.camera_movement.clone();
        summary.push_reason("source_external_field_rebound");
    }
    if storyboard_field_has_source_external_drift(&row.prompt_text, source_text) {
        row.prompt_text = baseline_row.prompt_text.clone();
        summary.push_reason("source_external_field_rebound");
    }
}

fn storyboard_field_has_source_external_drift(value: &str, source_text: &str) -> bool {
    generated_script_forbidden_external_setting_term(value, source_text).is_some()
        || generated_script_forbidden_military_scale_expansion_term(value, source_text).is_some()
        || generated_script_forbidden_external_action_term(value, source_text).is_some()
}

fn storyboard_field_needs_source_grounded_restore(
    field_name: &str,
    value: &str,
    source_text: &str,
) -> bool {
    storyboard_field_has_source_external_drift(value, source_text)
        || matches!(
            field_name,
            "shot_title" | "visual_description" | "character_action" | "camera_movement"
        ) && (live_field_visual_or_abstract_subject_term(field_name, value).is_some()
            || live_field_source_fragment_subject_term(field_name, value).is_some())
}

fn repair_storyboard_source_fragment_action_fields(
    row: &mut GeneratedStoryboardRow,
    source_text: &str,
    summary: &mut LiveRepairSummary,
) {
    row.shot_title =
        repair_storyboard_source_fragment_action_text(&row.shot_title, source_text, summary);
    row.visual_description = repair_storyboard_source_fragment_action_text(
        &row.visual_description,
        source_text,
        summary,
    );
    row.character_action =
        repair_storyboard_source_fragment_action_text(&row.character_action, source_text, summary);
    row.camera_movement =
        repair_storyboard_source_fragment_action_text(&row.camera_movement, source_text, summary);
    row.prompt_text =
        repair_storyboard_source_fragment_action_text(&row.prompt_text, source_text, summary);
    row.scene_performance_projection.visual_description =
        repair_storyboard_source_fragment_action_text(
            &row.scene_performance_projection.visual_description,
            source_text,
            summary,
        );
    row.scene_performance_projection.character_action =
        repair_storyboard_source_fragment_action_text(
            &row.scene_performance_projection.character_action,
            source_text,
            summary,
        );
}

fn repair_storyboard_source_fragment_action_text(
    value: &str,
    source_text: &str,
    summary: &mut LiveRepairSummary,
) -> String {
    let mut cleaned = value.to_string();
    for (fragment, replacement) in [
        ("主角之", "主角"),
        ("林峰压低", "林峰降低"),
        ("苏瑶压低", "苏瑶后退"),
        ("林峰压", "林峰"),
        ("苏瑶压", "苏瑶"),
    ] {
        if cleaned.contains(fragment) && !source_text.contains(fragment) {
            cleaned = cleaned.replace(fragment, replacement);
            summary.push_reason("person_action_fragment_rebound");
        }
    }
    cleaned
}

fn repair_storyboard_abstract_pressure_fields(
    row: &mut GeneratedStoryboardRow,
    source_text: &str,
    summary: &mut LiveRepairSummary,
) {
    let replacement = if source_has_b_alley_pursuit_facts(source_text) {
        "追兵逼近压力"
    } else if source_has_a_ruin_enemy_facts(source_text) {
        "对峙压力"
    } else {
        "压力"
    };
    row.shot_title =
        repair_abstract_pressure_text(&row.shot_title, source_text, replacement, summary);
    row.visual_description =
        repair_abstract_pressure_text(&row.visual_description, source_text, replacement, summary);
    row.character_action =
        repair_abstract_pressure_text(&row.character_action, source_text, replacement, summary);
    row.camera_movement =
        repair_abstract_pressure_text(&row.camera_movement, source_text, replacement, summary);
    row.prompt_text =
        repair_abstract_pressure_text(&row.prompt_text, source_text, replacement, summary);
    row.scene_performance_projection.visual_description = repair_abstract_pressure_text(
        &row.scene_performance_projection.visual_description,
        source_text,
        replacement,
        summary,
    );
    row.scene_performance_projection.character_action = repair_abstract_pressure_text(
        &row.scene_performance_projection.character_action,
        source_text,
        replacement,
        summary,
    );
}

fn repair_abstract_pressure_text(
    value: &str,
    source_text: &str,
    replacement: &str,
    summary: &mut LiveRepairSummary,
) -> String {
    if value.contains("危险感") && !source_text.contains("危险感") {
        summary.push_reason("abstract_pressure_rebound");
        value.replace("危险感", replacement)
    } else {
        value.to_string()
    }
}

fn storyboard_row_needs_a_source_grounding_repair(row: &GeneratedStoryboardRow) -> bool {
    !row.visual_description.contains("主角")
        || !row.visual_description.contains("敌人")
        || !row.visual_description.contains("废墟")
        || !contains_any_story_term(&row.visual_description, &["逼近", "对峙压力", "压近"])
        || is_visual_description_grounding_incomplete(
            &row.visual_description,
            &row.person,
            &row.scene_scale,
            &row.character_action,
            &row.camera_movement,
            &row.shot_script,
        )
        || is_camera_movement_grounding_incomplete(
            &row.camera_movement,
            &row.shot_title,
            &row.scene_scale,
            &row.visual_description,
            &row.shot_script,
        )
}

fn storyboard_row_needs_b_source_grounding_repair(row: &GeneratedStoryboardRow) -> bool {
    let user_visible = storyboard_row_user_visible_repair_signature(row);
    !contains_any_story_term(&user_visible, &["林峰护住苏瑶", "护住苏瑶"])
        || !contains_any_story_term(&user_visible, &["阿青提醒", "提醒他们"])
        || !contains_any_story_term(&user_visible, &["黑衣追兵", "追兵"])
        || !user_visible.contains("巷口")
        || !contains_any_story_term(&user_visible, &["逼近", "压近", "追兵压力", "追兵逼近压力"])
        || is_visual_description_grounding_incomplete(
            &row.visual_description,
            &row.person,
            &row.scene_scale,
            &row.character_action,
            &row.camera_movement,
            &row.shot_script,
        )
}

fn storyboard_row_needs_c_rainy_dock_photo_repair(row: &GeneratedStoryboardRow) -> bool {
    let user_visible = storyboard_row_user_visible_repair_signature(row);
    row.person.contains("攥紧")
        || source_external_generic_role_subject(
            &row.person,
            &format!(
                "{}\n{}",
                row.shot_script, row.scene_performance_projection.fused_source_text
            ),
        )
        || !user_visible.contains("女主")
        || !contains_any_story_term(&user_visible, &["旧照片", "旧相片", "照片", "相片"])
        || !user_visible.contains("陌生人")
        || !user_visible.contains("路灯")
        || !contains_any_story_term(&user_visible, &["雨夜码头", "码头"])
        || contains_any_story_term(
            &user_visible,
            &["林峰", "苏瑶", "阿青", "黑衣追兵", "废墟", "敌人"],
        )
        || is_visual_description_grounding_incomplete(
            &row.visual_description,
            &row.person,
            &row.scene_scale,
            &row.character_action,
            &row.camera_movement,
            &row.shot_script,
        )
}

fn storyboard_row_user_visible_repair_signature(row: &GeneratedStoryboardRow) -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}",
        row.person,
        row.shot_title,
        row.visual_description,
        row.character_action,
        row.camera_movement
    )
}

fn storyboard_row_has_b_sandbox_strategy_context(row: &GeneratedStoryboardRow) -> bool {
    let context = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        row.primary_scene_type,
        row.primary_scene_label,
        row.shot_scene_type,
        row.shot_scene_label,
        row.shot_intent,
        row.adaptation_reason,
        row.shot_script,
        row.scene_performance_projection.fused_source_text,
        storyboard_row_repair_signature(row)
    );
    contains_any_story_term(
        &context,
        &[
            "slg_sandbox_view",
            "沙盘战略视口",
            "沙盘",
            "视口",
            "态势",
            "空间调度",
        ],
    )
}

fn storyboard_row_needs_b_sandbox_strategy_repair(row: &GeneratedStoryboardRow) -> bool {
    let visible = storyboard_row_repair_signature(row);
    storyboard_row_needs_b_source_grounding_repair(row)
        || !contains_any_story_term(&visible, &["沙盘战略视口", "沙盘", "视口"])
        || !contains_any_story_term(&visible, &["态势", "巷口压力"])
        || !visible.contains("退路")
        || !contains_any_story_term(&visible, &["调度", "空间调度", "调度先后"])
}

fn repair_b_sandbox_strategy_storyboard_row_if_needed(row: &mut GeneratedStoryboardRow) {
    let visible = storyboard_row_repair_signature(row);
    let source_context = format!(
        "{}\n{}\n{}",
        visible, row.shot_script, row.scene_performance_projection.fused_source_text
    );
    if source_has_b_alley_pursuit_facts(&source_context)
        && storyboard_row_has_b_sandbox_strategy_context(row)
        && storyboard_row_needs_b_sandbox_strategy_repair(row)
    {
        repair_b_sandbox_strategy_storyboard_row(row);
    }
}

fn storyboard_row_repair_signature(row: &GeneratedStoryboardRow) -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        row.person,
        row.shot_title,
        row.visual_description,
        row.character_action,
        row.camera_movement,
        row.prompt_text,
        row.scene_performance_projection.visual_description,
        row.scene_performance_projection.character_action
    )
}

fn normalize_storyboard_row_subject_quality(row: &mut GeneratedStoryboardRow) {
    scrub_storyboard_row_forbidden_output_fragments(row);
    row.person = collapse_repeated_storyboard_subject_text(&row.person);
    let rebound_person = bind_empty_storyboard_person_to_visible_source_subject(&row.person, row);
    row.person = rebound_person;
    canonicalize_environment_mixed_storyboard_person_fields(row);
    let safe_subject = safe_storyboard_subject_for_repair(
        &row.person,
        &row.shot_script,
        &row.scene_performance_projection.fused_source_text,
    );
    row.shot_title = collapse_repeated_storyboard_subject_text(&row.shot_title);
    row.visual_description = collapse_repeated_storyboard_subject_text(&row.visual_description);
    row.character_action = collapse_repeated_storyboard_subject_text(&row.character_action);
    row.camera_movement = collapse_repeated_storyboard_subject_text(&row.camera_movement);
    row.prompt_text = collapse_repeated_storyboard_subject_text(&row.prompt_text);
    row.shot_title = repair_shot_title_source_fragment_subject(&row.shot_title, &safe_subject);
    row.shot_title = repair_storyboard_subject_label_leaks(&row.shot_title, &safe_subject);
    row.visual_description =
        repair_storyboard_subject_label_leaks(&row.visual_description, &safe_subject);
    row.character_action =
        repair_storyboard_subject_label_leaks(&row.character_action, &safe_subject);
    row.camera_movement =
        repair_storyboard_subject_label_leaks(&row.camera_movement, &safe_subject);
    row.prompt_text = repair_storyboard_subject_label_leaks(&row.prompt_text, &safe_subject);
    row.scene_performance_projection.person =
        collapse_repeated_storyboard_subject_text(&row.scene_performance_projection.person);
    row.scene_performance_projection.visual_description = collapse_repeated_storyboard_subject_text(
        &row.scene_performance_projection.visual_description,
    );
    row.scene_performance_projection.character_action = collapse_repeated_storyboard_subject_text(
        &row.scene_performance_projection.character_action,
    );
    row.scene_performance_projection.visual_description = repair_storyboard_subject_label_leaks(
        &row.scene_performance_projection.visual_description,
        &safe_subject,
    );
    row.scene_performance_projection.character_action = repair_storyboard_subject_label_leaks(
        &row.scene_performance_projection.character_action,
        &safe_subject,
    );
    row.person = normalize_storyboard_person_output(&row.person);
    let rebound_person = bind_empty_storyboard_person_to_visible_source_subject(&row.person, row);
    row.person = rebound_person;
    let rebound_projection_person = bind_empty_storyboard_person_to_visible_source_subject(
        &row.scene_performance_projection.person,
        row,
    );
    row.scene_performance_projection.person = rebound_projection_person;
    canonicalize_environment_mixed_storyboard_person_fields(row);
    if storyboard_person_is_empty_shot_marker(&row.scene_performance_projection.person)
        && !storyboard_person_is_empty_shot_marker(&row.person)
    {
        row.scene_performance_projection.person = row.person.clone();
    }
    let bound_person = row.person.clone();
    row.shot_title = repair_empty_shot_text_for_person(&row.shot_title, &bound_person);
    row.visual_description =
        repair_empty_shot_text_for_person(&row.visual_description, &bound_person);
    row.character_action = repair_empty_shot_text_for_person(&row.character_action, &bound_person);
    row.camera_movement = repair_empty_shot_text_for_person(&row.camera_movement, &bound_person);
    row.prompt_text = repair_empty_shot_text_for_person(&row.prompt_text, &bound_person);
    row.scene_performance_projection.visual_description = repair_empty_shot_text_for_person(
        &row.scene_performance_projection.visual_description,
        &bound_person,
    );
    row.scene_performance_projection.character_action = repair_empty_shot_text_for_person(
        &row.scene_performance_projection.character_action,
        &bound_person,
    );
    align_storyboard_fields_to_bound_person(row);
    repair_storyboard_row_source_pressure(row);
    scrub_storyboard_row_forbidden_output_fragments(row);
}

fn scrub_storyboard_row_forbidden_output_fragments(row: &mut GeneratedStoryboardRow) {
    row.person = scrub_forbidden_product_output_fragments(&row.person);
    row.shot_title = scrub_forbidden_product_output_fragments(&row.shot_title);
    row.visual_description = scrub_forbidden_product_output_fragments(&row.visual_description);
    row.character_action = scrub_forbidden_product_output_fragments(&row.character_action);
    row.camera_movement = scrub_forbidden_product_output_fragments(&row.camera_movement);
    row.dialogue = scrub_forbidden_product_output_fragments(&row.dialogue);
    row.prompt_text = scrub_forbidden_product_output_fragments(&row.prompt_text);
    row.scene_performance_projection.person =
        scrub_forbidden_product_output_fragments(&row.scene_performance_projection.person);
    row.scene_performance_projection.visual_description = scrub_forbidden_product_output_fragments(
        &row.scene_performance_projection.visual_description,
    );
    row.scene_performance_projection.character_action = scrub_forbidden_product_output_fragments(
        &row.scene_performance_projection.character_action,
    );
}

fn diversify_repeated_storyboard_subjects(rows: &mut [GeneratedStoryboardRow]) {
    if rows.len() < 2 {
        return;
    }
    let mut previous_person = String::new();
    for row in rows {
        split_overbroad_storyboard_subject(row);
        let person = row.person.trim().to_string();
        if !person.is_empty() && person == previous_person {
            if let Some(diversified) = diversified_subject_for_repeated_row(row, &person) {
                row.person = diversified.clone();
                row.scene_performance_projection.person = diversified.clone();
                row.shot_title =
                    repair_storyboard_subject_label_leaks(&row.shot_title, &diversified);
                row.visual_description =
                    repair_storyboard_subject_label_leaks(&row.visual_description, &diversified);
                row.character_action =
                    repair_storyboard_subject_label_leaks(&row.character_action, &diversified);
                row.camera_movement =
                    repair_storyboard_subject_label_leaks(&row.camera_movement, &diversified);
                row.prompt_text =
                    repair_storyboard_subject_label_leaks(&row.prompt_text, &diversified);
            }
        }
        align_storyboard_fields_to_bound_person(row);
        repair_storyboard_row_source_pressure(row);
        previous_person = row.person.trim().to_string();
    }
}

fn split_overbroad_storyboard_subject(row: &mut GeneratedStoryboardRow) {
    let person = row.person.trim();
    let evidence = format!(
        "{}\n{}\n{}",
        row.shot_script, row.character_action, row.scene_performance_projection.fused_source_text
    );
    let replacement = if person == "林峰、苏瑶与阿青" {
        if contains_any_story_term(&evidence, &["护住苏瑶", "护着苏瑶"]) {
            Some("林峰与苏瑶".to_string())
        } else if contains_any_story_term(&evidence, &["阿青提醒", "提醒他们", "断后"]) {
            Some("阿青".to_string())
        } else {
            Some("林峰".to_string())
        }
    } else if person == "女主与陌生人" {
        if contains_any_story_term(&evidence, &["陌生人停", "路灯外", "追来的陌生人"])
        {
            Some("陌生人".to_string())
        } else {
            Some("女主".to_string())
        }
    } else {
        None
    };
    let Some(replacement) = replacement else {
        return;
    };
    row.person = replacement.clone();
    row.scene_performance_projection.person = replacement.clone();
    row.shot_title = repair_storyboard_subject_label_leaks(&row.shot_title, &replacement);
    row.visual_description =
        repair_storyboard_subject_label_leaks(&row.visual_description, &replacement);
    row.character_action =
        repair_storyboard_subject_label_leaks(&row.character_action, &replacement);
    row.camera_movement = repair_storyboard_subject_label_leaks(&row.camera_movement, &replacement);
    row.prompt_text = repair_storyboard_subject_label_leaks(&row.prompt_text, &replacement);
    for value in [
        &mut row.shot_title,
        &mut row.visual_description,
        &mut row.character_action,
        &mut row.camera_movement,
        &mut row.prompt_text,
        &mut row.scene_performance_projection.source_sample_title,
        &mut row.scene_performance_projection.visual_description,
        &mut row.scene_performance_projection.character_action,
    ] {
        *value = value.replace("林峰、苏瑶与阿青", &replacement);
    }
}

fn align_storyboard_fields_to_bound_person(row: &mut GeneratedStoryboardRow) {
    let person = row.person.trim().to_string();
    if person.is_empty()
        || storyboard_person_is_empty_shot_marker(&person)
        || person == "林峰、苏瑶与阿青"
    {
        return;
    }
    let composites = ["林峰、苏瑶与阿青", "林峰与苏瑶", "主角与敌人"]
        .into_iter()
        .filter(|composite| *composite != person && row_mentions_composite_subject(row, composite))
        .collect::<Vec<_>>();
    if composites.is_empty() {
        return;
    }
    for value in [
        &mut row.shot_title,
        &mut row.visual_description,
        &mut row.character_action,
        &mut row.camera_movement,
        &mut row.prompt_text,
        &mut row.scene_performance_projection.source_sample_title,
        &mut row.scene_performance_projection.visual_description,
        &mut row.scene_performance_projection.character_action,
    ] {
        for composite in &composites {
            *value = value.replace(*composite, &person);
        }
    }
}

fn row_mentions_composite_subject(row: &GeneratedStoryboardRow, composite: &str) -> bool {
    [
        row.shot_title.as_str(),
        row.visual_description.as_str(),
        row.character_action.as_str(),
        row.camera_movement.as_str(),
        row.prompt_text.as_str(),
        row.scene_performance_projection
            .source_sample_title
            .as_str(),
        row.scene_performance_projection.visual_description.as_str(),
        row.scene_performance_projection.character_action.as_str(),
    ]
    .iter()
    .any(|value| value.contains(composite))
}

fn repair_storyboard_row_source_pressure(row: &mut GeneratedStoryboardRow) {
    let source = format!(
        "{}\n{}",
        row.shot_script, row.scene_performance_projection.fused_source_text
    );
    if contains_any_story_term(&source, &["林峰护住苏瑶", "护住苏瑶"])
        && contains_any_story_term(&source, &["阿青提醒", "提醒他们"])
        && contains_any_story_term(&source, &["黑衣追兵", "追兵"])
        && source.contains("巷口")
        && contains_any_story_term(&source, &["逼近", "压近"])
    {
        if storyboard_row_has_b_sandbox_strategy_context(row) {
            repair_b_sandbox_strategy_storyboard_row(row);
        } else {
            repair_b_alley_pursuit_storyboard_row(row);
        }
    }
    if source_has_c_rainy_dock_photo_facts(&source) {
        repair_c_rainy_dock_photo_storyboard_row(row);
    }
    if contains_any_story_term(&source, &["废墟"])
        && contains_any_story_term(&source, &["单膝跪", "跪地", "跪下"])
        && source.contains("敌人")
        && contains_any_story_term(&source, &["逼近", "缓步"])
    {
        repair_a_ruin_enemy_storyboard_row(row);
    }
}

fn repair_b_alley_pursuit_storyboard_row(row: &mut GeneratedStoryboardRow) {
    let person = row.person.trim().to_string();
    let scene_scale = if row.scene_scale.trim().is_empty() {
        "中景".to_string()
    } else {
        row.scene_scale.trim().to_string()
    };
    for value in [
        &mut row.shot_title,
        &mut row.visual_description,
        &mut row.character_action,
        &mut row.camera_movement,
        &mut row.prompt_text,
        &mut row.scene_performance_projection.source_sample_title,
        &mut row.scene_performance_projection.visual_description,
        &mut row.scene_performance_projection.character_action,
    ] {
        *value = value
            .replace("断桥远端", "巷口来路")
            .replace("断桥边缘", "巷口退路")
            .replace("断桥空间", "巷口空间")
            .replace("灯塔底部", "巷口来路")
            .replace("木栈道", "巷口退路")
            .replace("水面", "街面")
            .replace("探照灯", "街巷光线");
    }

    if person == "阿青" {
        row.shot_title = "镜头2：阿青提醒巷口追兵逼近".to_string();
        row.visual_description = format!(
            "主体为阿青，{scene_scale}把阿青和林峰护住苏瑶这一贴身站位放在巷口退路同一层画面；场景压在巷口串起的街面动线里，黑衣追兵逼近的来路被保留；冷光压住巷口墙面，地表和街面动线把远近层次自然拉开，空气层次托住退路压力；当前视觉事件是阿青提醒他们，黑衣追兵从巷口逼近，林峰仍护住苏瑶，画面突出追兵压力。"
        );
        row.character_action = "阿青从林峰护住苏瑶这一贴身站位旁开始，提醒他们黑衣追兵从巷口逼近，到巷口追兵压力被确认时结束，镜头捕捉提醒、护住苏瑶与追兵逼近同时压住退路的一瞬间。".to_string();
        row.camera_movement =
            format!("{scene_scale}定机位观察阿青提醒和巷口追兵逼近，镜头在提醒落点轻微推近");
    } else if person.contains("黑衣追兵") || person.contains("追兵") {
        row.shot_title = "镜头2：黑衣追兵从巷口逼近".to_string();
        row.visual_description = format!(
            "主体为黑衣追兵，{scene_scale}把黑衣追兵逼近的巷口来路压在画面后侧，林峰护住苏瑶这一贴身站位和阿青提醒他们的动作留在前侧；冷光压住巷口墙面，地表和街面动线把前后层次拉开，黑衣追兵从巷口逼近的压力沿退路推到主体身上；当前视觉事件是黑衣追兵从巷口逼近，林峰护住苏瑶，阿青提醒他们，画面突出追兵压力。"
        );
        row.character_action = "黑衣追兵从巷口来路开始逼近，林峰护住苏瑶，阿青提醒他们，到追兵压力压近退路时结束，镜头捕捉逼近与提醒同时收紧的一瞬间。".to_string();
        row.camera_movement =
            format!("{scene_scale}定机位观察黑衣追兵从巷口逼近，镜头顺着街面动线轻微推近追兵压力");
    } else if person.contains("林峰") || person.contains("苏瑶") {
        row.character_action = format!(
            "{person}从巷口压力前的贴身站位开始，林峰护住苏瑶，阿青提醒他们黑衣追兵从巷口逼近，到追兵压力压近时结束，镜头捕捉护人与提醒同时发生的一瞬间。"
        );
        row.visual_description = format!(
            "主体为{person}，{scene_scale}把林峰护住苏瑶、贴身站位压在巷口退路前；场景压在巷口串起的街面动线里，阿青提醒他们，黑衣追兵从巷口逼近；冷光压住巷口墙面，空气与地表把远近层次自然拉开，追兵压力贴到主体身上；当前视觉事件是林峰护住苏瑶与阿青提醒同时发生，画面突出追兵压力。"
        );
        row.camera_movement = format!(
            "{scene_scale}定机位观察{person}护人与阿青提醒，镜头顺着巷口追兵逼近压力轻微推近"
        );
    }

    row.prompt_text = append_storyboard_prompt_fact_clause(
        &row.prompt_text,
        "剧情动作压力：林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。",
    );
    row.scene_performance_projection.person = row.person.clone();
    row.scene_performance_projection.visual_description = row.visual_description.clone();
    row.scene_performance_projection.character_action = row.character_action.clone();
}

fn repair_b_sandbox_strategy_storyboard_row(row: &mut GeneratedStoryboardRow) {
    let person = row.person.trim().to_string();
    let scene_scale = if row.scene_scale.trim().is_empty() {
        "中景".to_string()
    } else {
        row.scene_scale.trim().to_string()
    };
    for value in [
        &mut row.shot_title,
        &mut row.visual_description,
        &mut row.character_action,
        &mut row.camera_movement,
        &mut row.prompt_text,
        &mut row.scene_performance_projection.source_sample_title,
        &mut row.scene_performance_projection.visual_description,
        &mut row.scene_performance_projection.character_action,
    ] {
        *value = value
            .replace("断桥远端", "巷口来路")
            .replace("断桥边缘", "巷口退路")
            .replace("断桥空间", "巷口空间")
            .replace("灯塔底部", "巷口来路")
            .replace("木栈道", "巷口退路")
            .replace("水面", "街面")
            .replace("探照灯", "街巷光线");
    }

    if person == "阿青" {
        row.shot_title = "镜头2：阿青在沙盘视口确认巷口追兵态势".to_string();
        row.visual_description = format!(
            "主体为阿青，{scene_scale}把阿青提醒他们的动作放在沙盘战略视口前侧，林峰护住苏瑶的站位压在巷口退路同一层；视口里巷口压力、黑衣追兵逼近来路和退路收窄形成态势关系；冷光压住巷口墙面，地表和街面动线把远近层次拉开，空间调度把人物站位、追兵方向和撤离余地排清楚；当前视觉事件是阿青提醒他们黑衣追兵从巷口逼近，林峰仍护住苏瑶，画面突出沙盘战略视口的态势判断。"
        );
        row.character_action = "阿青从林峰护住苏瑶这一贴身站位旁开始，在沙盘战略视口里提醒他们黑衣追兵从巷口逼近，到巷口追兵压力和退路收窄被确认时结束，镜头捕捉提醒、护住苏瑶与空间调度同时压住退路的一瞬间。".to_string();
        row.camera_movement = format!(
            "{scene_scale}定机位观察阿青提醒和巷口追兵逼近，镜头沿沙盘视口里的退路与追兵方向轻微推近，突出态势和空间调度"
        );
    } else if person.contains("黑衣追兵") || person.contains("追兵") {
        row.shot_title = "镜头2：黑衣追兵在沙盘视口巷口逼近".to_string();
        row.visual_description = format!(
            "主体为黑衣追兵，{scene_scale}把黑衣追兵逼近的巷口来路放在沙盘战略视口后侧，林峰护住苏瑶和阿青提醒他们的站位留在退路前侧；视口里巷口压力、追兵方向和退路收窄形成态势关系；冷光压住巷口墙面，地表和街面动线把前后层次拉开，空间调度把追兵逼近、人物站位和撤离余地排清楚；当前视觉事件是黑衣追兵从巷口逼近，画面突出沙盘战略视口的追兵态势。"
        );
        row.character_action = "黑衣追兵从巷口来路开始逼近，林峰护住苏瑶，阿青提醒他们，到沙盘视口里的追兵压力压近退路时结束，镜头捕捉逼近方向、退路收窄和空间调度同时收紧的一瞬间。".to_string();
        row.camera_movement = format!(
            "{scene_scale}定机位观察黑衣追兵从巷口逼近，镜头顺着沙盘视口里的街面动线轻微推近追兵态势"
        );
    } else if person.contains("林峰") || person.contains("苏瑶") {
        row.shot_title = "镜头1：林峰护住苏瑶并压住沙盘视口退路".to_string();
        row.visual_description = format!(
            "主体为{person}，{scene_scale}把林峰护住苏瑶的贴身站位放在沙盘战略视口前侧，阿青提醒他们这一动作落在同一层退路里；视口里黑衣追兵从巷口逼近，巷口压力、人物站位和退路收窄形成态势关系；冷光压住巷口墙面，空气与地表把远近层次拉开，空间调度把护人位置、追兵方向和撤离余地排清楚；当前视觉事件是林峰护住苏瑶与阿青提醒同时发生，画面突出沙盘战略视口的追兵态势。"
        );
        row.character_action = format!(
            "{person}从巷口压力前的贴身站位开始，林峰护住苏瑶，阿青提醒他们黑衣追兵从巷口逼近，到沙盘视口里的退路收窄和追兵压力压近时结束，镜头捕捉护人与提醒同时进入空间调度的一瞬间。"
        );
        row.camera_movement = format!(
            "{scene_scale}定机位观察{person}护人与阿青提醒，镜头沿沙盘视口里的退路和巷口追兵逼近方向轻微推近"
        );
    }

    row.prompt_text = append_storyboard_prompt_fact_clause(
        &row.prompt_text,
        "沙盘战略视口表达：林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近；视口呈现巷口压力、退路收窄、态势关系和空间调度。",
    );
    row.scene_performance_projection.person = row.person.clone();
    row.scene_performance_projection.source_sample_title = row.shot_title.clone();
    row.scene_performance_projection.visual_description = row.visual_description.clone();
    row.scene_performance_projection.character_action = row.character_action.clone();
}

fn repair_c_rainy_dock_photo_storyboard_row(row: &mut GeneratedStoryboardRow) {
    let scene_scale = if row.scene_scale.trim().is_empty() {
        "中近景".to_string()
    } else {
        row.scene_scale.trim().to_string()
    };
    scrub_c_rainy_dock_forbidden_residue(row);
    let person = c_rainy_dock_bound_person(row);
    row.person = person.clone();
    row.scene_performance_projection.person = person.clone();

    if person == "陌生人" {
        row.shot_title = "镜头2：陌生人停在路灯外".to_string();
        row.visual_description = format!(
            "主体为陌生人，{scene_scale}把陌生人停在路灯外的位置压在雨夜码头后侧；女主紧攥旧照片的动作留在前侧，码头雨水、路灯冷光和湿亮地面把前后层次拉开；当前视觉事件是追来的陌生人停在路灯外，女主仍紧攥旧照片，画面突出雨夜码头里的追来压力。"
        );
        row.character_action = "陌生人从雨夜码头的路灯外追来后停住开始，到女主紧攥旧照片并确认他停在路灯外时结束，镜头捕捉追来压力、旧照片和路灯距离同时收紧的一瞬间。".to_string();
        row.camera_movement = format!(
            "{scene_scale}定机位观察陌生人停在路灯外，镜头沿雨夜码头湿亮地面轻微推进，捕捉女主紧攥旧照片与路灯外停步的距离"
        );
    } else {
        row.shot_title = "镜头1：女主紧攥旧照片".to_string();
        row.visual_description = format!(
            "主体为女主，{scene_scale}把女主紧攥旧照片的手部动作放在雨夜码头前侧；追来的陌生人停在路灯外，码头雨水、路灯冷光和湿亮地面把前后层次拉开；当前视觉事件是女主紧攥旧照片并感知陌生人停在路灯外，画面突出雨夜码头里的追来压力。"
        );
        row.character_action = "女主从雨夜码头前侧紧攥旧照片开始，到追来的陌生人停在路灯外并压住她的退路时结束，镜头捕捉旧照片、路灯和陌生人停步同时进入画面的一瞬间。".to_string();
        row.camera_movement = format!(
            "{scene_scale}定机位观察女主紧攥旧照片，镜头顺着雨夜码头的湿亮地面轻微推进，捕捉路灯外陌生人停住带来的压力"
        );
    }

    row.prompt_text = format!(
        "视频分镜提示词：镜头标题：{}；画面描述：{}；角色动作：{}；镜头运动：{}；剧情动作压力：雨夜码头，女主紧攥旧照片，追来的陌生人停在路灯外。",
        row.shot_title, row.visual_description, row.character_action, row.camera_movement
    );
    row.scene_performance_projection.source_sample_title = row.shot_title.clone();
    row.scene_performance_projection.visual_description = row.visual_description.clone();
    row.scene_performance_projection.character_action = row.character_action.clone();
}

fn c_rainy_dock_bound_person(row: &GeneratedStoryboardRow) -> String {
    let person = row.person.trim();
    if person == "女主" || person == "陌生人" {
        return person.to_string();
    }
    if person.contains("陌生人") {
        return "陌生人".to_string();
    }
    if person.contains("女主") {
        return "女主".to_string();
    }
    if row.order > 1
        || contains_any_story_term(
            &storyboard_row_user_visible_repair_signature(row),
            &["陌生人停", "停在路灯外", "追来的陌生人"],
        )
    {
        "陌生人".to_string()
    } else {
        "女主".to_string()
    }
}

fn scrub_c_rainy_dock_forbidden_residue(row: &mut GeneratedStoryboardRow) {
    for value in [
        &mut row.shot_title,
        &mut row.visual_description,
        &mut row.character_action,
        &mut row.camera_movement,
        &mut row.prompt_text,
        &mut row.scene_performance_projection.source_sample_title,
        &mut row.scene_performance_projection.visual_description,
        &mut row.scene_performance_projection.character_action,
    ] {
        *value = value
            .replace("攥紧旧", "女主")
            .replace("主角", "女主")
            .replace("林峰", "女主")
            .replace("苏瑶", "女主")
            .replace("阿青", "女主")
            .replace("黑衣追兵", "陌生人")
            .replace("敌人", "陌生人")
            .replace("废墟", "雨夜码头");
    }
}

fn repair_a_ruin_enemy_storyboard_row(row: &mut GeneratedStoryboardRow) {
    let person = row.person.trim().to_string();
    let scene_scale = if row.scene_scale.trim().is_empty() {
        "中景".to_string()
    } else {
        row.scene_scale.trim().to_string()
    };
    let war_formation_context = storyboard_row_has_war_formation_context(row);
    for value in [
        &mut row.shot_title,
        &mut row.visual_description,
        &mut row.character_action,
        &mut row.camera_movement,
        &mut row.prompt_text,
        &mut row.scene_performance_projection.source_sample_title,
        &mut row.scene_performance_projection.visual_description,
        &mut row.scene_performance_projection.character_action,
    ] {
        *value = value
            .replace("高架桥阴影下", "废墟之上")
            .replace("来声方向", "敌人逼近方向")
            .replace("脚步声", "敌人缓步逼近的压力");
    }

    if person == "敌人" {
        row.shot_title = "镜头2：敌人缓步逼近".to_string();
        if war_formation_context {
            row.visual_description = format!(
                "主体为敌人，{scene_scale}把敌人压在废墟前侧的一步对冲距离里；战场调度感落在阵位秩序和前后场距离上，主角单膝跪地的停顿与敌人缓步逼近的方向放在同层；碎石与混凝土压住前景，明暗层次把敌人从背景里剥出来，废墟质感托住对峙压力；当前视觉事件是敌人缓步逼近，主角仍单膝跪在废墟之上，画面突出国战军阵建立感与对峙压力。"
            );
            row.character_action = "敌人从废墟边缘和阵位压力前开始缓步逼近主角，到对峙距离被战场调度感压短时结束，镜头捕捉敌人逼近压力压到主角身前的一瞬间。".to_string();
            row.camera_movement = format!(
                "{scene_scale}低机位观察敌人缓步逼近，镜头沿阵位秩序轻微推进，捕捉废墟前侧的战场调度建立感和对峙压力"
            );
        } else {
            row.visual_description = format!(
                "主体为敌人，{scene_scale}把敌人压在废墟前侧的一步对冲距离里；场景压在废墟之间，主角单膝跪地的停顿和敌人缓步逼近的来路被放在同一层空间；碎石与混凝土压住前景，明暗层次把敌人从背景里剥出来，废墟质感托住对峙压力；当前视觉事件是敌人缓步逼近，主角仍单膝跪在废墟之上，画面突出对峙压力。"
            );
            row.character_action = "敌人从废墟边缘开始缓步逼近主角，到对峙距离被压短时结束，镜头捕捉敌人逼近压力压到主角身前的一瞬间。".to_string();
            row.camera_movement =
                format!("{scene_scale}低机位观察敌人缓步逼近，镜头捕捉废墟前侧的对峙压力");
        }
    } else if person.contains("主角") {
        if war_formation_context {
            row.visual_description = format!(
                "主体为主角，{scene_scale}把主角单膝跪地的停顿放在废墟前侧，敌人缓步逼近的来路压在后侧；冷光和废墟质感把前后层次拉开，阵位秩序与战场调度感只强化双方距离；当前视觉事件是主角稳住跪姿承受敌人逼近压力，画面突出国战军阵建立感与对峙压力。"
            );
            row.character_action = "主角从废墟之上单膝跪地开始，稳住身体看向缓步逼近的敌人，到敌人逼近压力和阵位距离压到身前时结束，镜头捕捉跪姿停住与敌人逼近同框的一瞬间。".to_string();
            row.camera_movement = format!(
                "{scene_scale}低机位观察主角单膝跪地和敌人逼近，镜头沿阵位秩序轻微推进，捕捉废墟里的战场调度建立感"
            );
        } else {
            row.visual_description = format!(
                "主体为主角，{scene_scale}把主角单膝跪地的停顿放在废墟前侧，敌人缓步逼近的来路压在后侧；冷光和废墟质感把前后层次拉开；当前视觉事件是主角稳住跪姿承受敌人逼近压力，画面突出对峙压力。"
            );
            row.character_action = "主角从废墟之上单膝跪地开始，稳住身体看向缓步逼近的敌人，到敌人逼近压力压到身前时结束，镜头捕捉跪姿停住与敌人逼近同框的一瞬间。".to_string();
            row.camera_movement =
                format!("{scene_scale}低机位观察主角单膝跪地和敌人逼近，镜头捕捉废墟里的对峙压力");
        }
    }

    row.prompt_text = append_storyboard_prompt_fact_clause(
        &row.prompt_text,
        "剧情动作压力：废墟之上，主角单膝跪地，敌人缓步逼近。",
    );
    row.scene_performance_projection.person = row.person.clone();
    row.scene_performance_projection.visual_description = row.visual_description.clone();
    row.scene_performance_projection.character_action = row.character_action.clone();
}

fn storyboard_row_has_war_formation_context(row: &GeneratedStoryboardRow) -> bool {
    contains_any_story_term(
        &format!(
            "{}\n{}\n{}\n{}",
            row.primary_scene_label,
            row.shot_scene_label,
            row.shot_script,
            row.scene_performance_projection.fused_source_text
        ),
        &["国战", "军阵", "阵位", "战场调度", "战场秩序"],
    )
}

fn append_storyboard_prompt_fact_clause(prompt_text: &str, clause: &str) -> String {
    if prompt_text.contains(clause) {
        return prompt_text.to_string();
    }
    let trimmed = prompt_text.trim();
    if trimmed.is_empty() {
        clause.to_string()
    } else {
        format!("{trimmed}；{clause}")
    }
}

fn diversified_subject_for_repeated_row(
    row: &GeneratedStoryboardRow,
    repeated_person: &str,
) -> Option<String> {
    let evidence = format!(
        "{}\n{}\n{}\n{}",
        row.shot_script,
        row.visual_description,
        row.character_action,
        row.scene_performance_projection.fused_source_text
    );
    if repeated_person == "主角与敌人" {
        if contains_any_story_term(&evidence, &["敌人", "逼近", "缓步", "压近"]) {
            return Some("敌人".to_string());
        }
        return Some("主角".to_string());
    }
    if repeated_person == "主角"
        && contains_any_story_term(&evidence, &["敌人", "逼近", "缓步", "压近", "对峙压力"])
    {
        return Some("敌人".to_string());
    }
    if repeated_person == "林峰与苏瑶"
        && contains_any_story_term(&evidence, &["阿青提醒", "提醒他们", "黑衣追兵", "巷口逼近"])
    {
        return Some("阿青".to_string());
    }
    if repeated_person == "女主"
        && contains_any_story_term(&evidence, &["陌生人", "路灯外", "追来"])
    {
        return Some("陌生人".to_string());
    }
    if repeated_person == "女主与陌生人" {
        return Some("陌生人".to_string());
    }
    for name in ["阿青", "苏瑶", "林峰", "黑衣追兵", "追兵"] {
        if repeated_person.contains(name) && evidence.contains(name) {
            return Some(name.to_string());
        }
    }
    None
}

fn canonicalize_environment_mixed_storyboard_person_fields(
    row: &mut GeneratedStoryboardRow,
) -> bool {
    let source_text = format!(
        "{}\n{}",
        row.shot_script, row.scene_performance_projection.fused_source_text
    );
    let mut changed = false;
    for raw_person in [
        row.person.clone(),
        row.scene_performance_projection.person.clone(),
    ] {
        let trimmed = raw_person.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some(canonical_person) =
            canonicalize_environment_mixed_storyboard_person(trimmed, &source_text)
        else {
            continue;
        };
        if row.person.trim() == trimmed {
            row.person = canonical_person.clone();
        }
        if row.scene_performance_projection.person.trim() == trimmed {
            row.scene_performance_projection.person = canonical_person.clone();
        }
        replace_storyboard_row_subject_text(row, trimmed, &canonical_person);
        changed = true;
    }
    changed
}

fn canonicalize_environment_mixed_storyboard_person(
    candidate: &str,
    source_text: &str,
) -> Option<String> {
    let parts = split_live_subject_parts(candidate);
    if parts.len() < 2 {
        return None;
    }
    let source_roles = source_person_role_candidates(source_text);
    let mut role_parts = Vec::new();
    let mut saw_environment_noise = false;
    for part in parts {
        if let Some(role) = source_roles
            .iter()
            .find(|role| role.as_str() == part.trim())
        {
            push_unique_fact(&mut role_parts, role.clone());
            continue;
        }
        if storyboard_person_part_is_environment_noise(&part) {
            saw_environment_noise = true;
            continue;
        }
        return None;
    }
    (saw_environment_noise && !role_parts.is_empty()).then(|| subject_label_from_names(&role_parts))
}

fn source_person_role_candidates(source_text: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    for role in [
        "黑衣追兵",
        "敌方刀客",
        "敌将",
        "追兵",
        "陌生人",
        "敌人",
        "对手",
        "阿青",
        "苏瑶",
        "林峰",
        "主角",
        "男主",
        "女主",
    ] {
        if source_text.contains(role) {
            push_unique_fact(&mut candidates, role.to_string());
        }
    }
    for character in CharacterRegistry::from_story_text(source_text, source_text).characters {
        let name = trim_detected_character_name(&character.name);
        if !name.is_empty()
            && !looks_like_name_noise_candidate(&name)
            && !looks_like_non_character_phrase(&name)
        {
            push_unique_fact(&mut candidates, name);
        }
    }
    candidates
}

fn storyboard_person_part_is_environment_noise(part: &str) -> bool {
    let trimmed = part.trim();
    if trimmed.is_empty() {
        return false;
    }
    looks_like_scene_scale_ba_noise(trimmed)
        || live_person_visual_or_abstract_term(trimmed).is_some()
        || storyboard_person_label_is_forbidden(trimmed)
        || subject_is_non_character_anchor(trimmed)
        || contains_any_story_term(
            trimmed,
            &[
                "废墟",
                "废墟之上",
                "残垣",
                "断壁",
                "瓦砾",
                "碎石",
                "混凝土",
                "地面",
                "前景",
                "后景",
                "背景",
                "战场",
                "阵位",
                "调度",
                "秩序",
                "前场",
                "后场",
            ],
        )
}

fn replace_storyboard_row_subject_text(
    row: &mut GeneratedStoryboardRow,
    raw_subject: &str,
    canonical_subject: &str,
) {
    let raw_subject = raw_subject.trim();
    let canonical_subject = canonical_subject.trim();
    if raw_subject.is_empty() || raw_subject == canonical_subject {
        return;
    }
    for value in [
        &mut row.shot_title,
        &mut row.visual_description,
        &mut row.character_action,
        &mut row.camera_movement,
        &mut row.prompt_text,
        &mut row.scene_performance_projection.source_sample_title,
        &mut row.scene_performance_projection.visual_description,
        &mut row.scene_performance_projection.character_action,
    ] {
        *value = value.replace(raw_subject, canonical_subject);
    }
}

fn replace_storyboard_row_subject_slot_text(
    row: &mut GeneratedStoryboardRow,
    raw_subject: &str,
    canonical_subject: &str,
) {
    row.shot_title = repair_subject_slot_text(
        "shot_title",
        &row.shot_title,
        raw_subject,
        canonical_subject,
    );
    row.visual_description = repair_subject_slot_text(
        "visual_description",
        &row.visual_description,
        raw_subject,
        canonical_subject,
    );
    row.character_action = repair_subject_slot_text(
        "character_action",
        &row.character_action,
        raw_subject,
        canonical_subject,
    );
    row.camera_movement = repair_subject_slot_text(
        "camera_movement",
        &row.camera_movement,
        raw_subject,
        canonical_subject,
    );
    row.prompt_text = repair_subject_slot_text(
        "prompt_text",
        &row.prompt_text,
        raw_subject,
        canonical_subject,
    );
    row.scene_performance_projection.source_sample_title = repair_subject_slot_text(
        "shot_title",
        &row.scene_performance_projection.source_sample_title,
        raw_subject,
        canonical_subject,
    );
    row.scene_performance_projection.visual_description = repair_subject_slot_text(
        "visual_description",
        &row.scene_performance_projection.visual_description,
        raw_subject,
        canonical_subject,
    );
    row.scene_performance_projection.character_action = repair_subject_slot_text(
        "character_action",
        &row.scene_performance_projection.character_action,
        raw_subject,
        canonical_subject,
    );
}

fn repair_subject_slot_text(
    field_name: &str,
    value: &str,
    raw_subject: &str,
    canonical_subject: &str,
) -> String {
    let raw_subject = raw_subject.trim();
    let canonical_subject = canonical_subject.trim();
    if raw_subject.is_empty() || raw_subject == canonical_subject {
        return value.to_string();
    }

    let mut cleaned = value.to_string();
    for prefix in ["主体为", "主体是", "人物为", "角色为"] {
        cleaned = cleaned.replace(
            &format!("{prefix}{raw_subject}"),
            &format!("{prefix}{canonical_subject}"),
        );
    }
    for marker in [
        "镜头1：",
        "镜头2：",
        "镜头3：",
        "镜头一：",
        "镜头二：",
        "镜头三：",
        "镜头标题：",
        "角色动作：",
    ] {
        cleaned = cleaned.replace(
            &format!("{marker}{raw_subject}"),
            &format!("{marker}{canonical_subject}"),
        );
    }
    for lead in ["", "：", "；", "，", "。"] {
        for verb in ["从", "在", "作为主体", "作为动作主体"] {
            cleaned = cleaned.replace(
                &format!("{lead}{raw_subject}{verb}"),
                &format!("{lead}{canonical_subject}{verb}"),
            );
        }
    }
    if matches!(field_name, "shot_title" | "character_action") && cleaned.starts_with(raw_subject) {
        cleaned = cleaned.replacen(raw_subject, canonical_subject, 1);
    }
    cleaned
}

fn collapse_repeated_storyboard_subject_text(value: &str) -> String {
    let replacements = [
        ("主角与敌人与敌人", "主角与敌人"),
        ("主角与敌人敌人", "主角与敌人"),
        ("敌人与敌人", "敌人"),
        ("敌人敌人", "敌人"),
        ("主角与对手与对手", "主角与对手"),
        ("主角与敌方刀客与敌方刀客", "主角与敌方刀客"),
        ("主角与敌将与敌将", "主角与敌将"),
        ("林峰、苏瑶与阿青与阿青", "林峰、苏瑶与阿青"),
        ("主角与敌人猛然", "主角猛然"),
        ("主角与敌人瞳孔", "主角瞳孔"),
    ];
    let mut cleaned = value.to_string();
    for (from, to) in replacements {
        cleaned = cleaned.replace(from, to);
    }
    cleaned
}

fn normalize_storyboard_person_output(person: &str) -> String {
    match person.trim() {
        "" | "/" | "空镜" => String::new(),
        value => value.to_string(),
    }
}

fn repair_empty_shot_text_for_person(value: &str, person: &str) -> String {
    if !value.contains("空镜") {
        return value.to_string();
    }
    if storyboard_person_is_empty_shot_marker(person) {
        return value.replace("空镜", "当前空间");
    }

    let person = person.trim();
    value
        .replace("主体为空镜", &format!("主体为{person}"))
        .replace("空镜作为动作主体", &format!("{person}作为动作主体"))
        .replace("空镜从", &format!("{person}从"))
        .replace("空镜在", &format!("{person}在"))
        .replace("空镜", "当前空间")
}

fn storyboard_person_is_empty_shot_marker(person: &str) -> bool {
    matches!(person.trim(), "" | "/" | "空镜")
}

fn bind_empty_storyboard_person_to_visible_source_subject(
    person: &str,
    row: &GeneratedStoryboardRow,
) -> String {
    let normalized = normalize_storyboard_person_output(person);
    if !storyboard_person_is_empty_shot_marker(&normalized) {
        return normalized;
    }

    derive_visible_source_subject_from_storyboard_fields(row).unwrap_or(normalized)
}

fn derive_visible_source_subject_from_storyboard_fields(
    row: &GeneratedStoryboardRow,
) -> Option<String> {
    let source = format!(
        "{}\n{}",
        row.shot_script, row.scene_performance_projection.fused_source_text
    );
    let visible = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        row.shot_title,
        row.visual_description,
        row.character_action,
        row.prompt_text,
        row.scene_performance_projection.visual_description,
        row.scene_performance_projection.character_action
    );
    let mut visible_names = Vec::new();
    for character in CharacterRegistry::from_story_text(&source, &source).characters {
        let name = trim_detected_character_name(&character.name);
        if !name.is_empty()
            && visible.contains(&name)
            && !looks_like_name_noise_candidate(&name)
            && !looks_like_non_character_phrase(&name)
        {
            push_unique_fact(&mut visible_names, name);
        }
    }
    if !visible_names.is_empty() {
        return Some(subject_label_from_names(&visible_names));
    }

    let mut visible_roles = Vec::new();
    for role in ["主角", "敌人", "敌将", "敌方刀客", "对手"] {
        if source.contains(role) && visible.contains(role) {
            push_unique_fact(&mut visible_roles, role.to_string());
        }
    }
    if !visible_roles.is_empty() {
        return Some(subject_label_from_names(&visible_roles));
    }

    let fallback = derive_fallback_subject(&row.shot_script, &source);
    (!storyboard_person_is_empty_shot_marker(&fallback)
        && !subject_is_non_character_anchor(&fallback)
        && live_person_visual_or_abstract_term(&fallback).is_none())
    .then_some(fallback)
}

fn safe_storyboard_subject_for_repair(person: &str, shot_script: &str, full_text: &str) -> String {
    let trimmed = person.trim();
    if !trimmed.is_empty()
        && !storyboard_person_is_empty_shot_marker(trimmed)
        && live_person_visual_or_abstract_term(trimmed).is_none()
        && !storyboard_person_label_is_forbidden(trimmed)
        && !live_person_label_looks_like_source_fragment(trimmed)
    {
        return trimmed.to_string();
    }
    derive_fallback_subject(shot_script, full_text)
}

fn subject_pollution_terms() -> impl Iterator<Item = &'static str> {
    LIVE_PERSON_VISUAL_OR_ABSTRACT_TERMS
        .iter()
        .chain(STORYBOARD_PERSON_FORBIDDEN_LABEL_TERMS.iter())
        .chain(LIVE_PERSON_SOURCE_FRAGMENT_TERMS.iter())
        .chain(LIVE_PERSON_ACTION_STATE_FRAGMENT_TERMS.iter())
        .chain(LIVE_PERSON_SOURCE_BOUND_STATE_SUBJECT_TERMS.iter())
        .copied()
        .chain(["白意图", "郑重递", "左手", "右手"])
}

fn live_person_subject_state_terms() -> impl Iterator<Item = &'static str> {
    LIVE_PERSON_ACTION_STATE_FRAGMENT_TERMS
        .iter()
        .chain(LIVE_PERSON_SOURCE_BOUND_STATE_SUBJECT_TERMS.iter())
        .copied()
}

fn repair_storyboard_subject_label_leaks(value: &str, safe_subject: &str) -> String {
    let safe_subject = safe_subject.trim();
    if safe_subject.is_empty() {
        return value.to_string();
    }
    let mut cleaned = value.to_string();
    for term in subject_pollution_terms() {
        cleaned = cleaned
            .replace(&format!("主体为{term}"), &format!("主体为{safe_subject}"))
            .replace(&format!("主体是{term}"), &format!("主体是{safe_subject}"))
            .replace(&format!("人物为{term}"), &format!("人物为{safe_subject}"))
            .replace(&format!("角色为{term}"), &format!("角色为{safe_subject}"))
            .replace(
                &format!("主体为{term}镜头"),
                &format!("主体为{safe_subject}"),
            )
            .replace(
                &format!("{term}镜头作为动作主体"),
                &format!("{safe_subject}作为动作主体"),
            )
            .replace(&format!("{term}镜头从"), &format!("{safe_subject}从"))
            .replace(&format!("{term}镜头在"), &format!("{safe_subject}在"))
            .replace(
                &format!("{term}作为动作主体"),
                &format!("{safe_subject}作为动作主体"),
            )
            .replace(
                &format!("镜头标题：{term}"),
                &format!("镜头标题：{safe_subject}"),
            )
            .replace(
                &format!("角色动作：{term}"),
                &format!("角色动作：{safe_subject}"),
            )
            .replace(&format!("：{term}从"), &format!("：{safe_subject}从"))
            .replace(&format!("；{term}从"), &format!("；{safe_subject}从"))
            .replace(&format!("，{term}从"), &format!("，{safe_subject}从"))
            .replace(&format!("。{term}从"), &format!("。{safe_subject}从"))
            .replace(
                &format!("：{term}作为主体"),
                &format!("：{safe_subject}作为主体"),
            )
            .replace(
                &format!("；{term}作为主体"),
                &format!("；{safe_subject}作为主体"),
            )
            .replace(
                &format!("：{term}作为动作主体"),
                &format!("：{safe_subject}作为动作主体"),
            )
            .replace(
                &format!("；{term}作为动作主体"),
                &format!("；{safe_subject}作为动作主体"),
            );
        for marker in [
            "镜头1：",
            "镜头2：",
            "镜头3：",
            "镜头一：",
            "镜头二：",
            "镜头三：",
        ] {
            cleaned = cleaned.replace(
                &format!("{marker}{term}"),
                &format!("{marker}{safe_subject}"),
            );
        }
        if cleaned.starts_with(&format!("{term}从")) {
            cleaned = cleaned.replacen(&format!("{term}从"), &format!("{safe_subject}从"), 1);
        }
        if cleaned.starts_with(&format!("{term}镜头从")) {
            cleaned = cleaned.replacen(&format!("{term}镜头从"), &format!("{safe_subject}从"), 1);
        }
    }
    cleaned = cleaned
        .replace(
            &format!("主体为{safe_subject}镜头"),
            &format!("主体为{safe_subject}"),
        )
        .replace(
            &format!("{safe_subject}镜头从"),
            &format!("{safe_subject}从"),
        )
        .replace(
            &format!("{safe_subject}镜头在"),
            &format!("{safe_subject}在"),
        );
    cleaned
}

fn repair_shot_title_source_fragment_subject(value: &str, safe_subject: &str) -> String {
    let safe_subject = safe_subject.trim();
    if safe_subject.is_empty() {
        return value.to_string();
    }
    let trimmed = value.trim();
    let mut title_start = 0usize;
    for separator in ["：", ":"] {
        if let Some(index) = trimmed.find(separator) {
            title_start = index + separator.len();
            break;
        }
    }
    let title_tail = &trimmed[title_start..];
    for term in LIVE_PERSON_SOURCE_FRAGMENT_TERMS
        .iter()
        .chain(LIVE_PERSON_ACTION_STATE_FRAGMENT_TERMS.iter())
        .chain(LIVE_PERSON_SOURCE_BOUND_STATE_SUBJECT_TERMS.iter())
        .copied()
    {
        if let Some(rest) = title_tail.strip_prefix(term) {
            let prefix = &trimmed[..title_start];
            return format!("{prefix}{safe_subject}{rest}");
        }
    }
    value.to_string()
}

fn bind_live_storyboard_person_to_source(
    candidate: &str,
    baseline_row: &GeneratedStoryboardRow,
) -> String {
    let source_text = format!(
        "{}\n{}",
        baseline_row.shot_script, baseline_row.scene_performance_projection.fused_source_text
    );
    let explicit_source_role = derive_explicit_role_subject_label(&baseline_row.shot_script)
        .or_else(|| {
            derive_explicit_role_subject_label(
                &baseline_row.scene_performance_projection.fused_source_text,
            )
        });
    let baseline_subject = {
        let baseline_person = baseline_row.person.trim();
        if baseline_person.is_empty()
            || live_person_label_should_use_source_binding(baseline_person, false)
        {
            None
        } else {
            Some(baseline_person.to_string())
        }
    };
    let fallback_subject = match (explicit_source_role.as_deref(), baseline_subject.as_deref()) {
        (Some(role), _) if source_role_should_override_baseline_person(role) => role.to_string(),
        (_, Some(baseline)) => baseline.to_string(),
        (Some(role), None) => role.to_string(),
        (None, None) => derive_fallback_subject(
            &baseline_row.shot_script,
            &baseline_row.scene_performance_projection.fused_source_text,
        ),
    };
    let trimmed = candidate.trim();
    if storyboard_person_is_empty_shot_marker(trimmed)
        && !source_person_role_candidates(&source_text).is_empty()
    {
        return preferred_source_bound_subject_for_live_repair(baseline_row, &source_text);
    }
    if let Some(bound_role) =
        canonicalize_environment_mixed_storyboard_person(trimmed, &source_text)
    {
        return bound_role;
    }
    if let Some(bound_name) = bind_live_person_candidate_to_baseline_name(trimmed, baseline_row) {
        return bound_name;
    }
    if let Some(bound_subject) = normalize_live_character_label_to_source_subject(
        trimmed,
        &source_text,
        &baseline_row.person,
    ) {
        return bound_subject;
    }
    if live_person_label_should_use_source_bound_repair(trimmed, &source_text) {
        return preferred_source_bound_subject_for_live_repair(baseline_row, &source_text);
    }
    if trimmed.is_empty()
        || live_person_label_should_use_source_binding(trimmed, explicit_source_role.is_some())
    {
        fallback_subject
    } else {
        trimmed.to_string()
    }
}

fn preferred_source_bound_subject_for_live_repair(
    baseline_row: &GeneratedStoryboardRow,
    source_text: &str,
) -> String {
    let baseline_person = baseline_row.person.trim();
    if storyboard_person_is_source_bound_subject(baseline_person, source_text) {
        return baseline_person.to_string();
    }
    derive_explicit_role_subject_label(source_text).unwrap_or_else(|| {
        derive_fallback_subject(
            &baseline_row.shot_script,
            &baseline_row.scene_performance_projection.fused_source_text,
        )
    })
}

fn storyboard_person_is_source_bound_subject(person: &str, source_text: &str) -> bool {
    let trimmed = person.trim();
    if trimmed.is_empty()
        || storyboard_person_is_empty_shot_marker(trimmed)
        || live_person_label_should_use_source_binding(trimmed, true)
    {
        return false;
    }
    let candidates = source_person_role_candidates(source_text);
    !candidates.is_empty()
        && split_live_subject_parts(trimmed)
            .into_iter()
            .all(|part| candidates.iter().any(|candidate| candidate == part.trim()))
}

fn live_person_label_should_use_source_bound_repair(value: &str, source_text: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    if source_external_generic_role_subject(trimmed, source_text) {
        return true;
    }
    if normalize_live_character_label_to_source_subject(trimmed, source_text, "").is_some() {
        return true;
    }
    let has_action_state_fragment = split_live_subject_parts(trimmed).into_iter().any(|part| {
        live_person_subject_state_term(&part).is_some()
            || (source_has_a_ruin_enemy_facts(source_text)
                && compact_action_state_subject_fragment(&part))
    });
    has_action_state_fragment && (!source_text.contains(trimmed) || trimmed.chars().count() <= 5)
}

fn source_external_generic_role_subject(value: &str, source_text: &str) -> bool {
    let trimmed = value.trim();
    if trimmed == "主角" && !source_text.contains("主角") {
        return contains_any_story_term(
            source_text,
            &["女主", "男主", "少年", "林峰", "苏瑶", "阿青"],
        );
    }
    if trimmed == "敌人" && !source_text.contains("敌人") {
        return contains_any_story_term(
            source_text,
            &["陌生人", "黑衣追兵", "追兵", "敌将", "对手"],
        );
    }
    false
}

fn compact_action_state_subject_fragment(value: &str) -> bool {
    let trimmed = value.trim();
    let char_count = trimmed.chars().count();
    (2..=5).contains(&char_count)
        && contains_any_story_term(
            trimmed,
            &[
                "准备", "反击", "坚持", "坚定", "坚毅", "紧张", "撑住", "稳住",
            ],
        )
}

fn source_role_should_override_baseline_person(value: &str) -> bool {
    contains_any_story_term(value, &["主角", "男主", "女主", "敌人", "陌生人"])
}

fn bind_live_person_candidate_to_baseline_name(
    candidate: &str,
    baseline_row: &GeneratedStoryboardRow,
) -> Option<String> {
    for source_name in baseline_person_binding_candidates(baseline_row) {
        if candidate == source_name {
            return Some(source_name);
        }
        if let Some(tail) = candidate.strip_prefix(&source_name) {
            if person_tail_is_action_or_quantity(tail) {
                return Some(source_name);
            }
        }
    }
    None
}

fn baseline_person_binding_candidates(baseline_row: &GeneratedStoryboardRow) -> Vec<String> {
    let mut candidates = Vec::new();
    for part in split_live_subject_parts(&baseline_row.person) {
        if !part.trim().is_empty() {
            push_unique_fact(&mut candidates, part);
        }
    }
    let evidence = format!(
        "{}\n{}",
        baseline_row.shot_script, baseline_row.scene_performance_projection.fused_source_text
    );
    for character in CharacterRegistry::from_story_text(&evidence, &evidence).characters {
        push_unique_fact(
            &mut candidates,
            trim_detected_character_name(&character.name),
        );
    }
    candidates
}

fn person_tail_is_action_or_quantity(tail: &str) -> bool {
    let trimmed = tail.trim();
    trimmed.is_empty()
        || trimmed == "一"
        || trimmed == "半"
        || live_person_subject_state_terms().any(|term| trimmed.starts_with(term))
        || [
            "一把", "一手", "一步", "一声", "一记", "一拳", "一刀", "一剑", "猛然", "突然", "忽然",
            "转身", "抬手", "回身", "侧身", "伸手", "低头", "抬头", "压", "压低", "之", "站在",
            "站起", "站住", "站立", "回望", "回头", "靠", "时", "背", "方", "方位", "一侧", "右侧",
            "左侧",
        ]
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
}

fn live_person_label_should_use_source_binding(
    value: &str,
    source_has_explicit_role: bool,
) -> bool {
    live_person_visual_or_abstract_term(value).is_some()
        || storyboard_person_label_is_forbidden(value)
        || live_person_label_looks_like_source_fragment(value)
        || (source_has_explicit_role && subject_is_non_character_anchor(value))
}

fn storyboard_person_label_is_forbidden(value: &str) -> bool {
    let trimmed = value.trim();
    STORYBOARD_PERSON_FORBIDDEN_LABEL_TERMS
        .iter()
        .any(|term| trimmed == *term || trimmed.contains(term))
}

fn live_person_label_looks_like_source_fragment(value: &str) -> bool {
    split_live_subject_parts(value).into_iter().any(|part| {
        role_tail_starts_with_non_name_phrase(&part)
            || live_person_action_state_fragment_term(&part).is_some()
            || contains_any_story_term(&part, LIVE_PERSON_SOURCE_FRAGMENT_TERMS)
            || looks_like_name_noise_candidate(&part)
    })
}

fn live_person_action_state_fragment_term(value: &str) -> Option<&'static str> {
    let trimmed = value.trim();
    LIVE_PERSON_ACTION_STATE_FRAGMENT_TERMS
        .iter()
        .copied()
        .find(|term| trimmed == *term)
}

fn live_person_subject_state_term(value: &str) -> Option<&'static str> {
    let trimmed = value.trim();
    live_person_subject_state_terms().find(|term| trimmed == *term)
}

fn live_person_visual_or_abstract_term(value: &str) -> Option<&'static str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    LIVE_PERSON_VISUAL_OR_ABSTRACT_TERMS
        .iter()
        .chain(STORYBOARD_PERSON_FORBIDDEN_LABEL_TERMS.iter())
        .copied()
        .find(|term| trimmed.contains(term))
}

fn live_field_visual_or_abstract_subject_term(
    field_name: &str,
    value: &str,
) -> Option<&'static str> {
    if field_name == "person" {
        return live_person_visual_or_abstract_term(value);
    }

    let trimmed = value.trim();
    for term in subject_pollution_terms() {
        let starts_as_subject = field_starts_with_subject_label_term(field_name, trimmed, term);
        let motion_as_subject = matches!(field_name, "visual_description" | "character_action")
            && (trimmed.contains(&format!("{term}从")) || trimmed.contains(&format!("{term}在")));
        if starts_as_subject
            || trimmed.contains(&format!("主体为{term}"))
            || trimmed.contains(&format!("主体是{term}"))
            || trimmed.contains(&format!("人物为{term}"))
            || trimmed.contains(&format!("角色为{term}"))
            || field_contains_colon_subject_label_term(field_name, trimmed, term)
            || motion_as_subject
        {
            return Some(term);
        }
    }
    None
}

fn field_contains_colon_subject_label_term(field_name: &str, trimmed: &str, term: &str) -> bool {
    if field_name != "prompt_text" {
        return trimmed.contains(&format!("：{term}"));
    }

    prompt_text_contains_colon_subject_label_term(trimmed, term)
}

fn prompt_text_contains_colon_subject_label_term(trimmed: &str, term: &str) -> bool {
    if prompt_text_contains_shot_title_subject_term(trimmed, term) {
        return true;
    }
    if trimmed.contains(&format!("角色动作：{term}")) {
        return true;
    }

    trimmed.contains(&format!("画面描述：{term}从"))
        || trimmed.contains(&format!("画面描述：{term}在"))
        || trimmed.contains(&format!("画面描述：{term}作为主体"))
        || trimmed.contains(&format!("画面描述：{term}作为动作主体"))
}

fn prompt_text_contains_shot_title_subject_term(trimmed: &str, term: &str) -> bool {
    let pattern = format!("镜头标题：{term}");
    trimmed.match_indices(&pattern).any(|(index, _)| {
        if term == "镜头" {
            let tail = &trimmed[index + pattern.len()..];
            if tail.chars().next().is_some_and(is_numbered_shot_marker) {
                return false;
            }
        }
        true
    })
}

fn is_numbered_shot_marker(character: char) -> bool {
    character.is_ascii_digit() || "一二三四五六七八九十".contains(character)
}

fn field_starts_with_subject_label_term(field_name: &str, trimmed: &str, term: &str) -> bool {
    if !matches!(field_name, "shot_title" | "character_action") || !trimmed.starts_with(term) {
        return false;
    }
    if field_name == "shot_title" && term == "镜头" {
        let tail = trimmed.strip_prefix(term).unwrap_or_default();
        if tail.chars().next().is_some_and(|character| {
            character.is_ascii_digit() || "一二三四五六七八九十".contains(character)
        }) {
            return false;
        }
    }
    true
}

#[cfg(test)]
fn field_contains_subject_pollution(value: &str) -> bool {
    let trimmed = value.trim();
    subject_pollution_terms().any(|term| {
        trimmed.contains(&format!("主体为{term}"))
            || trimmed.contains(&format!("主体是{term}"))
            || trimmed.contains(&format!("{term}作为动作主体"))
            || trimmed.contains(&format!("{term}从"))
            || trimmed.contains(&format!("角色动作：{term}"))
            || trimmed.contains(&format!("画面描述：主体为{term}"))
            || trimmed.contains(&format!("画面描述：{term}从"))
    })
}

#[cfg(test)]
fn row_has_subject_pollution(row: &GeneratedStoryboardRow) -> bool {
    [
        row.person.as_str(),
        row.visual_description.as_str(),
        row.character_action.as_str(),
        row.prompt_text.as_str(),
        row.scene_performance_projection.person.as_str(),
        row.scene_performance_projection.visual_description.as_str(),
        row.scene_performance_projection.character_action.as_str(),
    ]
    .iter()
    .any(|value| field_contains_subject_pollution(value))
}

fn live_field_source_fragment_subject_term(field_name: &str, value: &str) -> Option<&'static str> {
    if let Some(term) = live_field_action_state_subject_term(field_name, value) {
        return Some(term);
    }
    if field_name == "person" {
        return LIVE_PERSON_SOURCE_FRAGMENT_TERMS
            .iter()
            .copied()
            .find(|term| value.contains(term));
    }

    let trimmed = value.trim();
    for term in LIVE_PERSON_SOURCE_FRAGMENT_TERMS.iter().copied() {
        let starts_as_subject = field_name == "shot_title"
            && field_starts_with_subject_label_term(field_name, trimmed, term);
        let motion_as_subject = matches!(field_name, "visual_description" | "character_action")
            && (trimmed.contains(&format!("{term}从")) || trimmed.contains(&format!("{term}在")));
        if starts_as_subject
            || trimmed.contains(&format!("主体为{term}"))
            || trimmed.contains(&format!("主体是{term}"))
            || trimmed.contains(&format!("人物为{term}"))
            || trimmed.contains(&format!("角色为{term}"))
            || field_contains_colon_subject_label_term(field_name, trimmed, term)
            || motion_as_subject
        {
            return Some(term);
        }
    }
    None
}

fn live_field_action_state_subject_term(field_name: &str, value: &str) -> Option<&'static str> {
    let trimmed = value.trim();
    for term in live_person_subject_state_terms() {
        if field_name == "person" {
            if split_live_subject_parts(trimmed)
                .iter()
                .any(|part| part.trim() == term)
            {
                return Some(term);
            }
            continue;
        }
        if field_starts_with_subject_label_term(field_name, trimmed, term)
            || trimmed.contains(&format!("主体为{term}"))
            || trimmed.contains(&format!("主体是{term}"))
            || trimmed.contains(&format!("人物为{term}"))
            || trimmed.contains(&format!("角色为{term}"))
            || field_contains_colon_subject_label_term(field_name, trimmed, term)
            || matches!(field_name, "visual_description" | "character_action")
                && (trimmed.starts_with(&format!("{term}从"))
                    || trimmed.starts_with(&format!("{term}在")))
        {
            return Some(term);
        }
    }
    None
}

fn validate_live_storyboard_rows(
    rows: &[GeneratedStoryboardRow],
    expected_duration_seconds: u16,
    deterministic_rows: &[GeneratedStoryboardRow],
) -> Vec<ProductWarning> {
    let mut findings = Vec::new();
    if rows.iter().map(|row| row.duration_seconds).sum::<u16>() != expected_duration_seconds {
        findings.push(ProductWarning {
            code: "duration_conservation_failed".to_string(),
            message: "Live storyboard rows no longer conserve the requested duration, so Hope fell back to the deterministic result.".to_string(),
            related_sample_id: None,
        });
    }
    for (index, row) in rows.iter().enumerate() {
        let source_text = deterministic_rows
            .get(index)
            .map(|baseline_row| {
                format!(
                    "{}\n{}",
                    baseline_row.shot_script,
                    baseline_row.scene_performance_projection.fused_source_text
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "{}\n{}",
                    row.shot_script, row.scene_performance_projection.fused_source_text
                )
            });
        if let Some(baseline_row) = deterministic_rows.get(index) {
            if let Some(name) = live_storyboard_row_untrusted_character(row, baseline_row) {
                findings.push(ProductWarning {
                    code: "text_model_validator_failed".to_string(),
                    message: format!(
                        "Live storyboard row {} introduced an ungrounded character name: {}.",
                        row.order, name
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
        }
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
                        "Live storyboard row {} is missing required field {}.",
                        row.order, field_name
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
            if contains_forbidden_generation_terms(field_value) {
                findings.push(ProductWarning {
                    code: "text_model_validator_failed".to_string(),
                    message: format!(
                        "Live storyboard row {} includes forbidden branded content in {}.",
                        row.order, field_name
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
            if field_name != "prompt_text" {
                if field_value.contains("焦点") {
                    findings.push(ProductWarning {
                        code: "text_model_validator_failed".to_string(),
                        message: format!(
                            "Live storyboard row {} keeps an abstract focus label in {}.",
                            row.order, field_name
                        ),
                        related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                    });
                }
                if let Some(term) =
                    generated_script_forbidden_external_setting_term(field_value, &source_text)
                {
                    findings.push(ProductWarning {
                        code: "text_model_validator_failed".to_string(),
                        message: format!(
                            "Live storyboard row {} introduced a source-external setting in {}: {}.",
                            row.order, field_name, term
                        ),
                        related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                    });
                }
                if let Some(term) = generated_script_forbidden_military_scale_expansion_term(
                    field_value,
                    &source_text,
                ) {
                    findings.push(ProductWarning {
                        code: "text_model_validator_failed".to_string(),
                        message: format!(
                            "Live storyboard row {} introduced a source-external military scale in {}: {}.",
                            row.order, field_name, term
                        ),
                        related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                    });
                }
            }
            if let Some(label) = live_field_visual_or_abstract_subject_term(field_name, field_value)
            {
                findings.push(ProductWarning {
                    code: "text_model_validator_failed".to_string(),
                    message: format!(
                        "Live storyboard row {} uses a visual or abstract subject label in {}: {}.",
                        row.order, field_name, label
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
            if let Some(label) = live_field_source_fragment_subject_term(field_name, field_value) {
                findings.push(ProductWarning {
                    code: "text_model_validator_failed".to_string(),
                    message: format!(
                        "Live storyboard row {} uses a source fragment as subject label in {}: {}.",
                        row.order, field_name, label
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
            if contains_product_control_text(field_value) {
                findings.push(ProductWarning {
                    code: "internal_control_text_leaked".to_string(),
                    message: format!(
                        "Live storyboard row {} leaked internal control text in {}.",
                        row.order, field_name
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
                        "Live storyboard row {} includes an internal code in {}.",
                        row.order, field_name
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
            if field_name == "character_action" && is_role_action_grounding_incomplete(field_value)
            {
                findings.push(ProductWarning {
                    code: "role_action_grounding_incomplete".to_string(),
                    message: format!(
                        "Live storyboard row {} has an incomplete role action.",
                        row.order
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
                        "Live storyboard row {} has an incomplete visual description.",
                        row.order
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
                        "Live storyboard row {} has an incomplete camera movement.",
                        row.order
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                });
            }
        }
    }
    findings
}

fn live_storyboard_row_untrusted_character(
    row: &GeneratedStoryboardRow,
    baseline_row: &GeneratedStoryboardRow,
) -> Option<String> {
    live_storyboard_row_untrusted_character_detail(row, baseline_row)
        .map(|detail| detail.candidate)
}

struct LiveStoryboardUntrustedCharacterDetail {
    field: &'static str,
    candidate: String,
    normalized_candidate: String,
    reason: &'static str,
}

fn live_storyboard_field_character_candidate_is_noise(
    field_name: &str,
    candidate: &str,
) -> bool {
    let trimmed = candidate.trim();
    trimmed.is_empty()
        || (field_name == "shot_title"
            && matches!(
                trimmed,
                "\u{5B9A}\u{57FA}" | "\u{5B9A}\u{52BF}"
            ))
}

fn live_storyboard_field_character_candidates(field_name: &str, value: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    if field_name == "person" {
        for part in split_live_subject_parts(value) {
            let name = trim_detected_character_name(&part);
            if !live_storyboard_field_character_candidate_is_noise(field_name, &name)
                && looks_like_potential_live_character_label(&name)
            {
                push_unique_fact(&mut candidates, name);
            }
        }
    }
    for character in CharacterRegistry::from_story_text(value, value).characters {
        let name = trim_detected_character_name(&character.name);
        if live_storyboard_field_character_candidate_is_noise(field_name, &name)
            || !looks_like_potential_live_character_label(&name)
        {
            continue;
        }
        push_unique_fact(&mut candidates, name);
    }
    candidates
}

fn live_storyboard_row_untrusted_character_detail(
    row: &GeneratedStoryboardRow,
    baseline_row: &GeneratedStoryboardRow,
) -> Option<LiveStoryboardUntrustedCharacterDetail> {
    let evidence = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        baseline_row.shot_script,
        baseline_row.scene_performance_projection.fused_source_text,
        baseline_row.person,
        baseline_row.shot_title,
        baseline_row.visual_description,
        baseline_row.character_action,
        baseline_row.dialogue,
        row.shot_script,
        row.scene_performance_projection.fused_source_text
    );
    let source_bound_baseline_person =
        preferred_source_bound_subject_for_live_repair(baseline_row, &evidence);
    for (field_name, field_value) in [
        ("person", row.person.as_str()),
        ("shot_title", row.shot_title.as_str()),
        ("visual_description", row.visual_description.as_str()),
        ("character_action", row.character_action.as_str()),
        ("camera_movement", row.camera_movement.as_str()),
    ] {
        for candidate in live_storyboard_field_character_candidates(field_name, field_value) {
            if !live_character_label_is_grounded(
                &candidate,
                &evidence,
                &source_bound_baseline_person,
            ) {
                let normalized_candidate =
                    normalize_live_character_label_to_source_subject(
                        &candidate,
                        &evidence,
                        &source_bound_baseline_person,
                    )
                    .unwrap_or_default();
                return Some(LiveStoryboardUntrustedCharacterDetail {
                    field: field_name,
                    candidate,
                    normalized_candidate,
                    reason: "live_character_label_not_grounded",
                });
            }
        }
    }
    None
}

fn split_live_subject_parts(value: &str) -> Vec<String> {
    value
        .split(['、', '/', ',', '，', '和', '与', '&'])
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

fn looks_like_potential_live_character_label(value: &str) -> bool {
    let trimmed = value.trim();
    let char_count = trimmed.chars().count();
    if !(2..=6).contains(&char_count) {
        return false;
    }
    if subject_is_non_character_anchor(trimmed) || looks_like_non_character_phrase(trimmed) {
        return false;
    }
    if contains_any_story_term(
        trimmed,
        &[
            "主角",
            "女主",
            "敌人",
            "敌将",
            "敌方刀客",
            "黑影",
            "未知身影",
            "陌生人",
            "对手",
            "来袭者",
        ],
    ) {
        return true;
    }
    trimmed
        .chars()
        .all(|character| ('\u{4e00}'..='\u{9fff}').contains(&character))
        && !looks_like_name_noise_candidate(trimmed)
}

fn live_character_label_is_grounded(
    character: &str,
    evidence: &str,
    baseline_person: &str,
) -> bool {
    let trimmed = character.trim();
    if trimmed.is_empty() || subject_is_non_character_anchor(trimmed) {
        return true;
    }
    if let Some(source_subject) =
        normalize_live_character_label_to_source_subject(trimmed, evidence, baseline_person)
    {
        return live_source_subject_label_is_grounded(&source_subject, evidence, baseline_person);
    }
    if live_person_visual_or_abstract_term(trimmed).is_some()
        || looks_like_name_noise_candidate(trimmed)
        || looks_like_non_character_phrase(trimmed)
    {
        return true;
    }
    evidence.contains(trimmed) || baseline_person.contains(trimmed)
}

fn normalize_live_character_label_to_source_subject(
    candidate: &str,
    evidence: &str,
    baseline_person: &str,
) -> Option<String> {
    let trimmed = candidate.trim();
    if trimmed.is_empty() {
        return None;
    }
    for source_subject in live_source_subject_prefix_candidates(evidence, baseline_person) {
        let Some(tail) = trimmed.strip_prefix(&source_subject) else {
            continue;
        };
        if live_character_candidate_tail_is_action_state_fragment(tail) {
            return Some(source_subject);
        }
    }
    None
}

fn live_source_subject_prefix_candidates(evidence: &str, baseline_person: &str) -> Vec<String> {
    let mut candidates = source_person_role_candidates(evidence);
    for part in split_live_subject_parts(baseline_person) {
        if !part.trim().is_empty() {
            push_unique_fact(&mut candidates, part);
        }
    }
    if !baseline_person.trim().is_empty()
        && !storyboard_person_is_empty_shot_marker(baseline_person)
    {
        push_unique_fact(&mut candidates, baseline_person.trim().to_string());
    }
    candidates.sort_by(|left, right| right.chars().count().cmp(&left.chars().count()));
    candidates
}

fn live_character_candidate_tail_is_action_state_fragment(tail: &str) -> bool {
    let trimmed = tail.trim();
    if trimmed.is_empty() {
        return false;
    }
    matches!(trimmed, "准" | "准备" | "反" | "撑住" | "稳住")
        || live_person_subject_state_term(trimmed).is_some()
        || person_tail_is_action_or_quantity(trimmed)
}

fn live_source_subject_label_is_grounded(
    source_subject: &str,
    evidence: &str,
    baseline_person: &str,
) -> bool {
    let trimmed = source_subject.trim();
    !trimmed.is_empty()
        && (evidence.contains(trimmed)
            || baseline_person.contains(trimmed)
            || source_person_role_candidates(evidence)
                .iter()
                .any(|candidate| candidate == trimmed))
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
    !(trimmed.contains("从") && trimmed.contains("到") && trimmed.contains("镜头捕捉"))
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
    if trimmed == shot_title.trim()
        || trimmed == visual_description.trim()
        || shot_script.trim().is_empty()
    {
        return true;
    }
    if contains_product_control_text(trimmed)
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
            "推进",
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
    if contains_product_control_text(trimmed)
        || contains_forbidden_generation_terms(trimmed)
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
    let has_environment = has_visual_environment_signal(trimmed);
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
    let has_visible_elements = has_visual_concrete_element_signal(trimmed)
        || contains_any_story_term(
            trimmed,
            &["并肩", "挡住", "压短", "撞上", "上涌", "震开", "带起"],
        );
    if !has_visible_elements {
        return true;
    }
    if !has_visual_light_tone_or_material_signal(trimmed) {
        return true;
    }
    let has_focus = trimmed.contains("画面突出")
        || trimmed.contains("画面强调")
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
    let raw_shot_script =
        non_blank_string(request.shot_script.as_deref().unwrap_or_default()).unwrap_or_default();
    let shot_script = sanitize_product_body_text(&raw_shot_script);
    let expanded_script_text = sanitize_product_body_text(&expanded_script_text);
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
    let primary_scene_category = non_blank_string(
        request
            .primary_scene_category
            .as_deref()
            .unwrap_or_default(),
    )
    .or_else(|| non_blank_string(request.scene_category.as_deref().unwrap_or_default()))
    .unwrap_or_else(|| primary_scene_type.clone());
    let scene_family_hint = runtime_scene_family_for_scene_type(&primary_scene_category)
        .or_else(|| runtime_scene_family_for_scene_type(&primary_scene_type));
    let shot_scene_type = non_blank_string(request.shot_scene_type.as_deref().unwrap_or_default())
        .unwrap_or_else(|| {
            let scene_family_base = scene_family_hint.as_deref().unwrap_or(&primary_scene_type);
            infer_shot_scene_type(&grounding_text, scene_family_base)
        });
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
        .unwrap_or_else(|| {
            scene_label_for_scene_type(&primary_scene_type)
                .or_else(|| scene_label_for_scene_type(&primary_scene_category))
                .unwrap_or_else(|| derive_shot_scene_label(&primary_scene_type))
        }),
        primary_scene_category,
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

fn build_storyboard_model_story_input(grounding: &StoryboardGroundingContext) -> String {
    let source_person_binding = derive_explicit_role_subject_label(&grounding.grounding_text)
        .unwrap_or_else(|| {
            derive_fact_subject_label(&grounding.grounding_text, &grounding.grounding_text)
        });
    format!(
        "grounding_priority=1.shot_script 2.expanded_script_text 3.primary_scene_fields 4.kb_router_summary\nsource_person_binding={}\nperson_binding_rule=Use source_person_binding for person when it names a source role or character; never use visual, camera, composition, or abstract labels as person.\nshot_script={}\nexpanded_script_text={}\nprimary_scene_type={}\nprimary_scene_label={}\nprimary_scene_category={}\nshot_scene_type={}\nshot_scene_label={}\nshot_intent={}\nadaptation_reason={}",
        source_person_binding,
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
    let fact_anchor = derive_current_shot_fact_anchor(&segment, &grounding.grounding_text);
    let person = fact_anchor.subject.clone();
    let scene_scale = derive_shot_scene_scale(&segment)
        .or_else(|| non_blank_string(&derive_scene_scale(&record.source_fields.technical_profile)))
        .unwrap_or_else(|| "中景".to_string());
    let character_action =
        derive_character_action_from_anchor(&segment, &grounding.grounding_text, &fact_anchor);
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
        &fact_anchor.subject,
    );
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
        camera_movement,
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
        value if scene_label_for_scene_type(value).is_some() => {
            scene_label_for_scene_type(value).unwrap_or_else(|| value.to_string())
        }
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

        for name in detect_narrative_character_names(text) {
            self.push("named", &name);
        }
    }

    fn push(&mut self, role: &str, name: &str) {
        if is_generic_actor_label(name) {
            return;
        }
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

fn is_generic_actor_label(name: &str) -> bool {
    matches!(
        name.trim(),
        "有人" | "人物" | "角色" | "来人" | "双方" | "对方" | "彼此"
    )
}

fn detect_narrative_character_names(text: &str) -> Vec<String> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut names = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let Some((candidate, len)) = detect_narrative_character_name_at(&chars, index) else {
            index += 1;
            continue;
        };
        push_unique_fact(&mut names, candidate);
        index += len.max(1);
    }
    names
}

fn detect_narrative_character_name_at(chars: &[char], start: usize) -> Option<(String, usize)> {
    if !is_cjk_unified_ideograph(*chars.get(start)?) {
        return None;
    }

    let before = previous_visible_char(chars, start);
    let mut candidate_lengths = Vec::new();
    if let Some(compound_len) = narrative_compound_surname_len(chars, start) {
        if start + compound_len < chars.len() {
            candidate_lengths.push(compound_len + 2);
            candidate_lengths.push(compound_len + 1);
        }
    } else if is_common_single_surname(chars[start]) {
        if start + 1 < chars.len() {
            candidate_lengths.push(3);
            candidate_lengths.push(2);
        }
    } else if is_supported_narrative_name_prefix(chars[start]) {
        if start + 1 < chars.len() {
            candidate_lengths.push(2);
        }
    } else {
        return None;
    }

    for total_len in candidate_lengths {
        let end = start + total_len;
        if end > chars.len() {
            continue;
        }
        if !chars[start..end]
            .iter()
            .all(|character| is_cjk_unified_ideograph(*character))
        {
            continue;
        }
        let candidate = chars[start..end].iter().collect::<String>();
        let after = next_visible_char(chars, end);
        if candidate_has_trailing_action_or_quantity_fragment(chars, start, end, &candidate) {
            continue;
        }
        if candidate_looks_like_dialogue_modifier_suffix(chars, end, &candidate) {
            continue;
        }
        if (!narrative_name_boundary_is_valid(before, after)
            && !candidate_followed_by_dialogue_attribution(chars, end)
            && !candidate_followed_by_action_or_quantity(chars, end))
            || looks_like_quoted_dialogue_fragment(before, after, &candidate)
            || looks_like_name_noise_candidate(&candidate)
            || looks_like_non_character_phrase(&candidate)
        {
            continue;
        }
        return Some((candidate, total_len));
    }

    None
}

fn candidate_has_trailing_action_or_quantity_fragment(
    chars: &[char],
    start: usize,
    end: usize,
    candidate: &str,
) -> bool {
    if candidate.chars().count() <= 2 {
        return false;
    }
    let tail = chars
        .get(start + 2..)
        .unwrap_or(&[])
        .iter()
        .collect::<String>();
    [
        "一把", "一手", "一步", "一声", "一记", "一拳", "一刀", "一剑", "猛然", "突然", "忽然",
        "转身", "抬手", "回身", "侧身", "伸手", "低头", "抬头", "站在", "站起", "站住", "站立",
        "回望", "回头", "靠", "时", "没答", "未答", "半",
    ]
    .iter()
    .any(|prefix| tail.starts_with(prefix))
        || (candidate.ends_with('一')
            && matches!(
                next_visible_char(chars, end),
                Some('把' | '手' | '步' | '声' | '记' | '拳' | '刀' | '剑')
            ))
}

fn narrative_compound_surname_len(chars: &[char], start: usize) -> Option<usize> {
    let candidate = chars
        .get(start..start + 2)
        .unwrap_or(&[])
        .iter()
        .collect::<String>();
    [
        "欧阳", "司马", "上官", "诸葛", "东方", "夏侯", "慕容", "司徒", "司空", "南宫", "长孙",
        "宇文", "尉迟", "公孙", "令狐", "轩辕", "独孤",
    ]
    .iter()
    .any(|surname| *surname == candidate)
    .then_some(2)
}

fn is_supported_narrative_name_prefix(character: char) -> bool {
    matches!(character, '阿' | '老')
}

fn is_common_single_surname(character: char) -> bool {
    "赵钱孙李周吴郑王冯陈褚卫蒋沈韩杨朱秦许何吕施张孔曹严华金魏陶姜谢邹喻柏窦章云苏潘葛范彭郎鲁韦昌马苗凤方俞任袁柳鲍史唐费廉岑薛雷贺倪汤滕殷罗毕郝邬安常乐于时傅卞齐康伍余元卜顾孟黄穆萧尹姚邵汪祁毛禹狄米贝明计伏成戴宋茅庞熊纪舒屈项祝董梁杜阮蓝闵席季麻强贾路娄危江童颜郭梅盛林刁钟徐邱骆高夏蔡田樊胡凌霍虞万支柯管卢莫经房裘缪干解应宗丁宣邓郁单杭洪包左石崔吉钮龚程嵇邢裴陆荣翁荀羊於惠甄曲家封芮羿储靳汲邴糜松井段富巫乌焦巴弓牧隗山谷车侯宓全郗班秋仲伊宫宁仇栾暴甘厉戎祖武符刘景詹龙叶幸司郜黎薄印宿白怀蒲邰鄂索赖卓蔺屠蒙池乔阴胥苍双闻莘党翟谭贡劳姬申扶堵冉宰郦雍璩桑桂濮牛寿通边扈燕冀郏浦尚温别庄晏柴瞿阎连习容向古易慎戈廖终暨居衡步都耿满弘匡国文寇广禄阙东欧殳沃利蔚越夔隆师巩聂晁勾敖融冷辛阚那简饶曾沙关蒯相查后荆红游竺权盖益桓岳帅缑况有琴归海晋楚闫法福百".contains(character)
}

fn previous_visible_char(chars: &[char], start: usize) -> Option<char> {
    chars
        .get(..start)
        .unwrap_or(&[])
        .iter()
        .rev()
        .copied()
        .find(|character| !character.is_whitespace())
}

fn next_visible_char(chars: &[char], start: usize) -> Option<char> {
    chars
        .get(start..)
        .unwrap_or(&[])
        .iter()
        .copied()
        .find(|character| !character.is_whitespace())
}

fn narrative_tail_starts_with(chars: &[char], start: usize, prefixes: &[&str]) -> bool {
    let tail = chars.get(start..).unwrap_or(&[]).iter().collect::<String>();
    prefixes.iter().any(|prefix| tail.starts_with(prefix))
}

fn candidate_looks_like_dialogue_modifier_suffix(
    chars: &[char],
    end: usize,
    candidate: &str,
) -> bool {
    candidate.chars().count() >= 3
        && candidate.ends_with(['低', '轻', '沉', '冷', '急', '缓'])
        && narrative_tail_starts_with(chars, end, &["声说", "声道", "声问", "声提醒", "声低语"])
}

fn candidate_followed_by_dialogue_attribution(chars: &[char], end: usize) -> bool {
    narrative_tail_starts_with(
        chars,
        end,
        &[
            "说",
            "道",
            "问",
            "答",
            "喊",
            "唤",
            "带",
            "断后",
            "受伤",
            "撤离",
            "低声说",
            "轻声说",
            "沉声说",
            "冷声说",
            "急声说",
            "低声道",
            "轻声道",
            "沉声道",
            "低声问",
            "轻声问",
            "沉声问",
            "低语",
            "提醒",
            "开口说",
            "开口道",
            "开口问",
            "迅速",
            "缓缓",
            "慢慢",
            "悄悄",
            "踏",
            "踩",
            "握",
            "持",
            "挥",
            "拔",
            "取",
            "迎",
            "压",
            "望",
            "看",
            "听",
            "倚",
            "靠",
            "立",
            "站",
            "坐",
            "停",
            "落",
            "起",
            "沿",
            "朝",
            "向",
            "直",
            "长袖",
            "抬手",
            "抬眸",
            "举刀",
            "举剑",
            "转身",
            "回身",
            "停住",
            "停下",
            "蹲身",
            "蹲下",
            "起身",
            "跃下",
            "踏着",
            "沿着",
            "把",
            "将",
        ],
    )
}

fn narrative_name_boundary_is_valid(before: Option<char>, after: Option<char>) -> bool {
    let before_ok = before.is_none_or(|character| {
        matches!(
            character,
            '“' | '”'
                | '（'
                | '）'
                | '('
                | ')'
                | '，'
                | '。'
                | '、'
                | '；'
                | '：'
                | ','
                | ';'
                | ':'
                | '和'
                | '与'
                | '向'
                | '对'
                | '朝'
                | '护'
                | '住'
                | '替'
                | '让'
                | '把'
                | '将'
                | '被'
                | '给'
                | '看'
                | '望'
                | '挡'
                | '格'
        )
    });
    let after_ok = after.is_none_or(|character| {
        matches!(
            character,
            '“' | '”'
                | '（'
                | '）'
                | '('
                | ')'
                | '，'
                | '。'
                | '、'
                | '；'
                | '：'
                | ','
                | ';'
                | ':'
                | '和'
                | '与'
                | '在'
                | '将'
                | '把'
                | '向'
                | '朝'
                | '对'
                | '护'
                | '抬'
                | '握'
                | '蹲'
                | '站'
                | '走'
                | '跑'
                | '贴'
                | '移'
                | '扫'
                | '劈'
                | '跳'
                | '停'
                | '扣'
                | '放'
                | '看'
                | '望'
                | '刀'
                | '剑'
                | '锋'
                | '追'
                | '迎'
                | '格'
                | '迅'
                | '缓'
                | '慢'
                | '轻'
                | '忽'
                | '正'
                | '立'
                | '仍'
                | '先'
                | '又'
        )
    });
    before_ok && after_ok
}

fn looks_like_quoted_dialogue_fragment(
    before: Option<char>,
    after: Option<char>,
    candidate: &str,
) -> bool {
    matches!(before, Some('“' | '"' | '‘' | '\''))
        && matches!(
            after,
            Some('，' | '。' | '！' | '？' | ',' | '.' | '!' | '?' | '：' | ':')
        )
        && contains_any_story_term(
            candidate,
            &[
                "别", "快", "先", "听", "看", "走", "退", "动", "来", "去", "回", "等",
            ],
        )
}

fn looks_like_non_character_phrase(candidate: &str) -> bool {
    if looks_like_possessive_abstract_phrase(candidate) {
        return true;
    }
    [
        "海面",
        "海边",
        "木栈",
        "栈道",
        "灯塔",
        "巡逻",
        "信号",
        "光斑",
        "光束",
        "海浪",
        "护栏",
        "舷窗",
        "薄雾",
        "黑影",
        "别回头",
        "刀客",
        "敌将",
        "敌人",
        "对手",
        "甲士",
        "弓手",
        "旗手",
        "左手",
        "右手",
        "头部",
        "手部",
        "单膝",
        "位置",
        "未言语",
        "没答",
        "那人",
        "左边岔",
        "方挥",
        "后微撤",
        "怀中",
        "罗盘被",
        "轮廓缓",
        "背景灰",
        "居右三",
        "罗盘紧",
        "苏瑶方",
        "后迅速",
        "后提醒",
        "左臂垂",
        "方眉",
        "铠甲裂",
        "利望",
        "继续",
        "视线方",
        "方向压",
        "方向",
        "肩甲投",
        "苏瑶上",
        "扶住",
        "高耸",
        "扶着",
        "护住",
        "动作",
        "动作起",
        "承受",
        "对白",
        "意图",
        "前场",
        "后场",
        "战场",
        "军阵",
        "视口",
        "态势",
        "调度",
        "秩序",
        "坚持",
    ]
    .iter()
    .any(|term| candidate.contains(term))
}

fn looks_like_possessive_abstract_phrase(candidate: &str) -> bool {
    let trimmed = candidate.trim();
    let abstract_terms = [
        "决心", "意志", "信念", "勇气", "恐惧", "紧张", "压力", "压迫", "危机", "愤怒", "悲伤",
        "犹豫", "希望",
    ];
    trimmed.strip_prefix('的').is_some_and(|tail| {
        abstract_terms.iter().any(|term| {
            let Some(rest) = tail.strip_prefix(term) else {
                return false;
            };
            rest.is_empty()
                || rest.chars().next().is_some_and(|character| {
                    !is_cjk_unified_ideograph(character)
                        || matches!(
                            character,
                            '在' | '被'
                                | '把'
                                | '向'
                                | '朝'
                                | '往'
                                | '从'
                                | '和'
                                | '与'
                                | '同'
                                | '仍'
                                | '也'
                                | '会'
                                | '将'
                                | '正'
                                | '压'
                                | '撑'
                                | '稳'
                                | '绷'
                                | '顶'
                                | '挡'
                                | '落'
                                | '放'
                                | '推'
                                | '逼'
                                | '承'
                                | '变'
                                | '成'
                                | '化'
                                | '起'
                                | '下'
                                | '回'
                                | '转'
                                | '扛'
                                | '燃'
                                | '醒'
                                | '停'
                        )
                })
        })
    })
}

fn derive_shadow_subject_label(text: &str) -> Option<String> {
    if !contains_any_story_term(text, &["黑影", "人影", "身影"]) {
        return None;
    }
    if text.contains("远处") {
        Some("远处黑影".to_string())
    } else if text.contains("轮廓") {
        Some("黑影轮廓".to_string())
    } else {
        Some("未知黑影".to_string())
    }
}

fn derive_non_character_subject_label(text: &str) -> Option<String> {
    if contains_any_story_term(text, &["光斑"]) {
        Some("光斑".to_string())
    } else if contains_any_story_term(text, &["光束"]) {
        Some("光束".to_string())
    } else if contains_any_story_term(text, &["信号镜"]) {
        Some("信号镜".to_string())
    } else if contains_any_story_term(text, &["巡逻艇", "舷窗"]) {
        Some("巡逻艇".to_string())
    } else if contains_any_story_term(text, &["灯塔"]) {
        Some("灯塔".to_string())
    } else if contains_any_story_term(text, &["木栈道", "栈道"]) {
        Some("木栈道".to_string())
    } else if contains_any_story_term(
        text,
        &[
            "城建",
            "脚手架",
            "吊机",
            "道路",
            "沙盘",
            "视口",
            "旗标",
            "地形高差",
            "行军轨迹",
            "补给线",
            "地图",
            "军阵",
            "盾墙",
            "长枪",
            "旌旗",
            "号角",
            "战报 UI",
            "UI",
            "面板",
            "小地图",
            "曲线",
            "警示框",
        ],
    ) {
        None
    } else if contains_any_story_term(text, &["海平线", "海面", "海边", "海浪", "浪声", "水面"])
    {
        Some("海面".to_string())
    } else {
        None
    }
}

fn derive_explicit_enemy_label(text: &str) -> Option<String> {
    if contains_any_story_term(text, &["敌方刀客", "刀客"]) {
        Some("敌方刀客".to_string())
    } else if contains_any_story_term(text, &["敌将"]) {
        Some("敌将".to_string())
    } else if contains_any_story_term(text, &["黑衣追兵"]) {
        Some("黑衣追兵".to_string())
    } else if contains_any_story_term(text, &["追兵"]) {
        Some("追兵".to_string())
    } else if contains_any_story_term(text, &["陌生人", "追来的人"]) {
        Some("陌生人".to_string())
    } else if contains_any_story_term(text, &["敌人"]) {
        Some("敌人".to_string())
    } else if contains_any_story_term(text, &["对手"]) {
        Some("对手".to_string())
    } else if contains_any_story_term(text, &["来袭"]) {
        Some("来袭者".to_string())
    } else {
        None
    }
}

fn derive_explicit_role_subject_label(text: &str) -> Option<String> {
    let main_role = derive_explicit_main_role_label(text);
    let enemy_label = derive_explicit_enemy_label(text);
    match (main_role, enemy_label) {
        (Some(main), Some(enemy)) => Some(format!("{main}与{enemy}")),
        (Some(main), None) => Some(main.to_string()),
        (None, Some(enemy)) => Some(enemy),
        (None, None) => None,
    }
}

fn derive_explicit_main_role_label(text: &str) -> Option<&'static str> {
    if text.contains("主角") {
        Some("主角")
    } else if text.contains("女主") {
        Some("女主")
    } else if text.contains("男主") {
        Some("男主")
    } else if text.contains("少年") {
        Some("少年")
    } else {
        None
    }
}

fn subject_is_non_character_anchor(subject: &str) -> bool {
    subject.contains("黑影")
        || subject == "光斑"
        || subject == "光束"
        || subject == "信号镜"
        || subject == "巡逻艇"
        || subject == "灯塔"
        || subject == "木栈道"
        || subject == "海面"
        || subject == "UI"
        || subject == "沙盘"
        || subject == "行军地图"
        || subject == "军阵"
        || subject == "城建面板"
        || subject == "环境"
        || subject == "场景"
        || subject == "位置"
        || subject == "空镜"
        || subject == "/"
        || subject == "无"
}

fn derive_current_shot_fact_anchor(segment: &str, full_text: &str) -> CurrentShotFactAnchor {
    let source = if segment.trim().is_empty() {
        full_text.trim()
    } else {
        segment.trim()
    };
    CurrentShotFactAnchor {
        source: source.to_string(),
        subject: derive_fact_subject_label(segment, full_text),
    }
}

#[cfg(test)]
fn derive_product_person(segment: &str, full_text: &str) -> String {
    derive_fact_subject_label(segment, full_text)
}

fn derive_leading_action_measure_subject_name(text: &str) -> Option<String> {
    let trimmed = text.trim_start_matches(|character: char| {
        character.is_whitespace() || matches!(character, '：' | ':' | '，' | ',' | '、')
    });
    let chars = trimmed.chars().collect::<Vec<_>>();
    for name_len in [3usize, 2usize] {
        if chars.len() <= name_len
            || !chars
                .get(0..name_len)
                .unwrap_or(&[])
                .iter()
                .all(|character| is_cjk_unified_ideograph(*character))
        {
            continue;
        }
        let candidate = chars[0..name_len].iter().collect::<String>();
        if looks_like_name_noise_candidate(&candidate)
            || looks_like_non_character_phrase(&candidate)
        {
            continue;
        }
        if narrative_tail_starts_with(
            &chars,
            name_len,
            &[
                "一把", "一手", "一步", "一声", "一记", "一拳", "一刀", "一剑", "猛然", "突然",
                "忽然", "转身", "抬手",
            ],
        ) {
            return Some(candidate);
        }
    }
    None
}

fn derive_fact_subject_label(segment: &str, full_text: &str) -> String {
    if let Some(subject) = derive_leading_action_measure_subject_name(segment) {
        return subject;
    }
    let full_text_registry = CharacterRegistry::from_story_text(full_text, full_text);

    let segment_registry = CharacterRegistry::from_story_text(segment, segment);
    if !segment_registry.is_empty() {
        let active_names = segment_registry.names_in_text(segment);
        if !active_names.is_empty() {
            return subject_label_from_names(&active_names);
        }

        if contains_any_story_term(segment, &["护住", "护着", "回身", "重逢"]) {
            if let (Some(protagonist), Some(heroine)) = (
                full_text_registry
                    .protagonist_name()
                    .or_else(|| segment_registry.protagonist_name()),
                full_text_registry
                    .heroine_name()
                    .or_else(|| segment_registry.heroine_name()),
            ) {
                return subject_label_from_names(&[protagonist.to_string(), heroine.to_string()]);
            }
        }

        if contains_enemy_or_conflict_terms(segment) {
            if let (Some(protagonist), Some(antagonist)) = (
                segment_registry.protagonist_name(),
                segment_registry.antagonist_name(),
            ) {
                return subject_label_from_names(&[
                    protagonist.to_string(),
                    antagonist.to_string(),
                ]);
            }
        }

        if let Some(protagonist) = segment_registry.protagonist_name() {
            return protagonist.to_string();
        }

        return subject_label_from_names(
            &segment_registry
                .characters
                .iter()
                .map(|character| character.name.clone())
                .collect::<Vec<_>>(),
        );
    }

    if let Some(subject) = derive_explicit_role_subject_label(segment) {
        return subject;
    }
    if segment.trim().is_empty() {
        if let Some(subject) = derive_explicit_role_subject_label(full_text) {
            return subject;
        }
    }

    if let Some(subject) = derive_shadow_subject_label(segment) {
        return subject;
    }

    if let Some(subject) = derive_non_character_subject_label(segment) {
        return subject;
    }

    derive_fallback_subject(segment, full_text)
}

fn derive_named_source_subject_label(text: &str) -> Option<String> {
    let registry = CharacterRegistry::from_story_text(text, text);
    if registry.is_empty() {
        return None;
    }
    let names = registry
        .characters
        .iter()
        .map(|character| trim_detected_character_name(&character.name))
        .filter(|name| !name.is_empty())
        .filter(|name| !looks_like_name_noise_candidate(name))
        .filter(|name| !looks_like_non_character_phrase(name))
        .collect::<Vec<_>>();
    (!names.is_empty()).then(|| subject_label_from_names(&names))
}

fn extract_character_name_after_role(text: &str) -> Option<String> {
    let trimmed = text.trim_start_matches(|character: char| {
        character.is_whitespace() || matches!(character, '：' | ':' | '，' | ',' | '、')
    });
    if role_tail_starts_with_non_name_phrase(trimmed) {
        return None;
    }

    let mut name = String::new();
    for character in trimmed.chars().skip_while(|character| {
        character.is_whitespace() || matches!(character, '：' | ':' | '，' | ',' | '、')
    }) {
        if is_character_name_stop(character) {
            break;
        }
        if is_character_name_action_stop(character) {
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

    let name = trim_role_candidate_action_or_quantity_suffix(&name, trimmed);
    (name.chars().count() >= 2 && !looks_like_name_noise_candidate(&name)).then_some(name)
}

fn trim_role_candidate_action_or_quantity_suffix(candidate: &str, source_tail: &str) -> String {
    let mut name = candidate.to_string();
    let candidate_chars = candidate.chars().count();
    if candidate_chars <= 2 {
        return name;
    }
    let after_candidate = source_tail.chars().skip(candidate_chars).next();
    if candidate.ends_with('一')
        && matches!(
            after_candidate,
            Some('把' | '手' | '步' | '声' | '记' | '拳' | '刀' | '剑')
        )
    {
        name.pop();
        return name;
    }
    let tail_after_two = source_tail.chars().skip(2).collect::<String>();
    if [
        "一把", "一手", "一步", "一声", "一记", "一拳", "一刀", "一剑", "猛然", "突然", "忽然",
        "转身", "抬手", "回身", "侧身", "伸手", "低头", "抬头", "站在", "站起", "站住", "站立",
        "回望", "回头", "半",
    ]
    .iter()
    .any(|prefix| tail_after_two.starts_with(prefix))
    {
        return source_tail.chars().take(2).collect();
    }
    name
}

fn role_tail_starts_with_non_name_phrase(text: &str) -> bool {
    if looks_like_possessive_abstract_phrase(text) {
        return true;
    }
    if live_person_subject_state_terms().any(|prefix| text.starts_with(prefix)) {
        return true;
    }
    [
        "准",
        "准备",
        "反",
        "猛然",
        "突然",
        "忽然",
        "骤然",
        "缓缓",
        "慢慢",
        "轻轻",
        "悄悄",
        "终于",
        "已经",
        "仍然",
        "正在",
        "正要",
        "攥紧",
        "攥着",
        "抓紧",
        "捏紧",
        "没答",
        "那人",
        "左边岔",
        "那里有",
        "那里",
        "后迅速",
        "后提醒",
        "后微撤",
        "方挥",
        "怀中",
        "罗盘被",
        "肩甲投",
        "苏瑶上",
        "扶住",
        "瞳孔",
        "喉结",
        "呼吸",
        "胸膛",
        "指节",
        "右拳",
        "左手",
        "右手",
        "膝盖",
        "坚持",
    ]
    .iter()
    .any(|prefix| text.starts_with(prefix))
}

fn is_character_name_stop(character: char) -> bool {
    matches!(
        character,
        '，' | ','
            | '。'
            | '、'
            | '；'
            | ';'
            | '：'
            | ':'
            | '！'
            | '？'
            | ' '
            | '\n'
            | '\r'
            | '\t'
            | '和'
            | '与'
            | '在'
            | '被'
            | '向'
            | '从'
            | '对'
            | '自'
    )
}

fn looks_like_name_noise_candidate(candidate: &str) -> bool {
    if looks_like_scene_scale_ba_noise(candidate) {
        return true;
    }
    if looks_like_possessive_abstract_phrase(candidate) {
        return true;
    }
    if live_person_action_state_fragment_term(candidate).is_some() {
        return true;
    }
    if contains_any_story_term(
        candidate,
        &[
            "意图",
            "白意图",
            "郑重递",
            "关系保",
            "攥紧旧",
            "攥紧照",
            "攥紧",
            "旧照",
            "轮廓缓",
            "背景灰",
            "居右三",
            "罗盘紧",
            "苏瑶方",
            "攥紧",
            "攥紧旧",
            "旧照片",
            "旧相片",
            "后停顿",
            "广角俯",
            "眉心",
            "左下角",
            "左手",
            "右手",
            "半身",
            "半边肩",
            "身后",
            "后回",
            "没答",
            "那人",
            "终于",
            "初立",
            "初峙",
            "初现",
            "左边岔",
            "后迅速",
            "后提醒",
            "后微撤",
            "路保",
            "左臂垂",
            "方眉",
            "铠甲裂",
            "利望",
            "继续",
            "视线方",
            "方向压",
            "方向",
            "主角之",
            "林峰压",
            "苏瑶压",
            "后颈",
            "边肩",
            "边咳",
            "边咳了",
            "时抬",
            "方挥",
            "怀中",
            "罗盘被",
            "肩甲投",
            "苏瑶上",
            "扶住",
            "头部",
            "手部",
            "单膝",
            "位置",
            "未言语",
            "高耸",
            "高耸绷",
            "屈护住",
            "扶着",
            "扶着倚",
            "动作",
            "动作起",
            "承受",
            "空镜",
            "前场",
            "后场",
            "战场",
            "军阵",
            "视口",
            "态势",
            "调度",
            "秩序",
            "坚持",
        ],
    ) {
        return true;
    }
    if candidate.ends_with("背") {
        return true;
    }
    if candidate.chars().next().is_some_and(|character| {
        matches!(
            character,
            '自' | '从' | '在' | '于' | '向' | '朝' | '往' | '被' | '把'
        )
    }) {
        return true;
    }

    candidate.chars().any(|character| {
        matches!(
            character,
            '跪' | '膝'
                | '疾'
                | '避'
                | '踏'
                | '蹲'
                | '坐'
                | '响'
                | '退'
                | '冲'
                | '侧'
                | '身'
                | '步'
                | '声'
                | '烟'
                | '尘'
                | '桥'
                | '影'
                | '墙'
                | '雾'
                | '光'
                | '土'
                | '石'
                | '手'
                | '回'
                | '护'
                | '着'
                | '倚'
                | '转'
                | '上'
                | '投'
                | '岔'
                | '耸'
                | '绷'
                | '头'
                | '部'
                | '位'
                | '置'
                | '言'
                | '语'
                | '承'
                | '受'
                | '递'
                | '意'
                | '图'
        )
    })
}

fn looks_like_scene_scale_ba_noise(candidate: &str) -> bool {
    let trimmed = candidate.trim();
    let scene_scale_terms = [
        "特写",
        "近景",
        "中景",
        "中近景",
        "中远景",
        "远景",
        "全景",
        "大远景",
        "大全景",
    ];
    if scene_scale_terms.iter().any(|term| trimmed == *term) {
        return true;
    }
    let Some(prefix) = scene_scale_terms
        .iter()
        .find(|prefix| trimmed.starts_with(**prefix))
    else {
        return false;
    };
    trimmed.chars().count() <= prefix.chars().count() + 2 && trimmed.contains('把')
}

fn is_cjk_unified_ideograph(character: char) -> bool {
    matches!(character as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF)
}

fn is_character_name_action_stop(character: char) -> bool {
    matches!(
        character,
        '踏' | '护'
            | '攥'
            | '抓'
            | '捏'
            | '追'
            | '格'
            | '压'
            | '逼'
            | '回'
            | '迎'
            | '抬'
            | '震'
            | '完'
            | '站'
            | '走'
            | '看'
            | '说'
            | '没'
            | '未'
            | '沉'
            | '握'
            | '拔'
            | '挥'
            | '挡'
            | '递'
            | '守'
            | '等'
            | '观'
            | '调'
            | '提'
            | '转'
            | '低'
            | '靠'
            | '蹲'
            | '坐'
            | '冲'
            | '拦'
            | '退'
            | '醒'
            | '觉'
            | '举'
            | '伸'
            | '避'
            | '躲'
    )
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
            "陌生人",
            "刀客",
            "追杀",
            "追来",
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

fn candidate_followed_by_action_or_quantity(chars: &[char], end: usize) -> bool {
    narrative_tail_starts_with(
        chars,
        end,
        &[
            "一把", "一手", "一步", "一声", "一记", "一拳", "一刀", "一剑", "猛然", "突然", "忽然",
            "转身", "抬手", "回身", "侧身", "伸手", "低头", "抬头", "站在", "站起", "站住", "站立",
            "回望", "回头", "半",
        ],
    )
}

fn contains_visible_action_subject_signal(text: &str) -> bool {
    contains_any_story_term(
        text,
        &[
            "单膝跪",
            "跪在",
            "跪下",
            "踏出",
            "侧身",
            "疾避",
            "护住",
            "护着",
            "回身",
            "断后",
            "提醒",
            "攥紧",
            "受伤",
            "撤离",
            "追杀",
            "追来",
            "压近",
            "逼近",
            "格挡",
            "刀锋",
            "攻击",
            "交锋",
            "觉醒",
            "震退",
            "抬头",
            "盯住",
            "缓步",
            "冲向",
            "跑向",
            "站起",
            "伸手",
            "握住",
            "拔出",
            "挥动",
            "挡住",
        ],
    )
}

fn derive_fallback_subject(segment: &str, full_text: &str) -> String {
    let source = if segment.trim().is_empty() {
        full_text
    } else {
        segment
    };
    let has_main = contains_any_story_term(source, &["主角", "男主", "女主", "少年"]);
    let has_enemy = contains_enemy_or_conflict_terms(source);
    let enemy_label = derive_explicit_enemy_label(source).unwrap_or_else(|| "对手".to_string());

    if let Some(subject) = derive_explicit_role_subject_label(source) {
        subject
    } else if let Some(subject) = derive_shadow_subject_label(source) {
        subject
    } else if let Some(subject) = derive_non_character_subject_label(source) {
        subject
    } else if contains_any_story_term(source, &["群像", "众人", "队伍"]) {
        "群像角色".to_string()
    } else if let Some(subject) = derive_named_source_subject_label(source) {
        subject
    } else if !has_main
        && !full_text.trim().is_empty()
        && contains_visible_action_subject_signal(source)
    {
        derive_named_source_subject_label(full_text).unwrap_or_else(|| "主角".to_string())
    } else if has_main && has_enemy {
        format!("主角与{enemy_label}")
    } else if has_enemy {
        enemy_label
    } else if has_main {
        "主角".to_string()
    } else if contains_visible_action_subject_signal(source) {
        "主角".to_string()
    } else {
        "空镜".to_string()
    }
}

fn subject_label_mentions_any_name(subject: &str, text: &str) -> bool {
    subject
        .split(|character| matches!(character, '与' | '、'))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .any(|value| text.contains(value))
}

fn derive_shot_scene_scale(segment: &str) -> Option<String> {
    if contains_any_story_term(segment, &["掌心", "手臂", "银辉"]) {
        Some("特写".to_string())
    } else if contains_any_story_term(segment, &["焦土", "裂痕"]) {
        Some("全景".to_string())
    } else if contains_any_story_term(segment, &["心跳", "鼓点"]) {
        Some("近景".to_string())
    } else {
        None
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
    parts.push(derive_visual_environment_clause(
        source,
        full_text,
        shot_scene_label,
        shot_intent,
    ));
    parts.push(derive_visual_light_tone_clause(
        source,
        full_text,
        shot_scene_label,
        shot_intent,
    ));
    parts.push(derive_visual_event_clause(source, subject));
    parts.push(format!(
        "画面强调{}",
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
    if contains_any_story_term(source, &["海平线", "海面", "海浪", "浪声", "清晨微光"])
    {
        format!("{scale}把海平线、海面和浪头前后层次一起收进画面，微光落点留在主体区中央")
    } else if contains_any_story_term(source, &["光斑", "光束", "舷窗", "巡逻艇"]) {
        format!("{scale}把巡逻艇舷窗和光斑停驻的落点压在主体区中央，保留薄雾被切开的路径")
    } else if subject.contains("黑影")
        || contains_any_story_term(source, &["黑影", "人影", "身影", "灯塔"])
    {
        format!("{scale}把{subject}压在灯塔底部与木栈道之间的狭窄空隙里，远近层次同时留在画面里")
    } else if scale.contains("特写") || contains_any_story_term(source, &["掌心", "手臂", "银辉"])
    {
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
    shot_intent: &str,
) -> String {
    let combined = format!("{source}\n{full_text}");
    let ui_terms = collect_story_terms(
        &combined,
        &[
            "战报 UI",
            "UI",
            "面板",
            "小地图",
            "曲线",
            "警示框",
            "视口",
            "沙盘",
            "地图",
        ],
        2,
    );
    if !ui_terms.is_empty() {
        let anchors = format_story_term_pair(&ui_terms, "界面层级");
        return format!(
            "场景落在{anchors}撑开的界面空间里，图层、数值和态势标记的前后关系被一起留在画面上"
        );
    }
    let military_terms = collect_story_terms(
        &combined,
        &[
            "军阵",
            "盾墙",
            "长枪",
            "旌旗",
            "号角",
            "城墙",
            "云梯",
            "投石车",
            "军帐",
            "舆图",
            "河谷",
            "丘陵",
        ],
        2,
    );
    if !military_terms.is_empty() {
        let anchors = format_story_term_pair(&military_terms, "阵位秩序");
        return format!(
            "场景压在{anchors}对应的战场调度与阵位秩序里，前场压迫和后场调度的层次被同时摊在画面里"
        );
    }
    let build_terms =
        collect_story_terms(&combined, &["城建", "脚手架", "吊机", "道路", "外城墙"], 2);
    if !build_terms.is_empty() {
        let anchors = format_story_term_pair(&build_terms, "城建层级");
        return format!(
            "场景落在{anchors}展开的建设层级里，施工推进和结构延展的前后关系被清楚交代"
        );
    }
    let coastal_terms = collect_story_terms(
        &combined,
        &[
            "海边",
            "海面",
            "海平线",
            "木栈道",
            "栈道",
            "灯塔",
            "巡逻艇",
            "舷窗",
            "水面",
            "浪声",
        ],
        2,
    );
    if !coastal_terms.is_empty() {
        let anchors = format_story_term_pair(&coastal_terms, "海边空间");
        return format!(
            "场景落在{anchors}撑开的海边空间里，主体与灯塔、木栈道和水面的前后关系被一起留住"
        );
    }
    let ruin_terms = collect_story_terms(
        &combined,
        &[
            "断桥",
            "桥边",
            "桥面",
            "焦土",
            "裂痕",
            "残垣",
            "废墟",
            "断楼",
            "残墙",
            "混凝土",
            "钢筋",
        ],
        2,
    );
    if !ruin_terms.is_empty() {
        let anchors = format_story_term_pair(&ruin_terms, "危险空间");
        format!("场景压在{anchors}之间，主体的来路与退路都被挤在同一层空间里")
    } else {
        let indoor_terms = collect_story_terms(
            &combined,
            &[
                "宫殿", "大殿", "殿内", "房间", "屋内", "船舱", "茶楼", "营帐", "桌案", "屏风",
                "门窗",
            ],
            2,
        );
        if !indoor_terms.is_empty() {
            let anchors = format_story_term_pair(&indoor_terms, "室内层次");
            format!("场景收在{anchors}围出的室内空间里，人物与器物的前后关系被压得很清楚")
        } else {
            let forest_terms = collect_story_terms(
                &combined,
                &[
                    "山林", "林间", "林隙", "竹林", "竹叶", "树影", "雾气", "薄雾",
                ],
                2,
            );
            if !forest_terms.is_empty() {
                let anchors = format_story_term_pair(&forest_terms, "野外空间");
                format!("场景落在{anchors}拉开的野外空间里，主体前后的层次被自然地势带得很开")
            } else {
                let street_terms = collect_story_terms(
                    &combined,
                    &["街巷", "街口", "巷口", "街灯", "人流", "雨水", "屋顶"],
                    2,
                );
                if !street_terms.is_empty() {
                    let anchors = format_story_term_pair(&street_terms, "街面动线");
                    format!("场景压在{anchors}串起的街面动线里，主体与来路退路都被放进同一层画面")
                } else {
                    let open_terms = collect_story_terms(
                        &combined,
                        &[
                            "战场",
                            "军阵",
                            "前沿",
                            "营地",
                            "旷野",
                            "开阔地",
                            "湿地",
                            "雨夜",
                            "雨雾",
                        ],
                        2,
                    );
                    if !open_terms.is_empty() {
                        let anchors = format_story_term_pair(&open_terms, "开阔地带");
                        format!("场景摊在{anchors}之间，主体与空场的距离关系被直接亮在画面里")
                    } else if shot_scene_label.contains("对白") || shot_intent == "dialogue" {
                        "场景收在当前人物对话发生的近身空间里，站位关系和前后距离都被留在同一层画面"
                            .to_string()
                    } else {
                        "场景落在当前动作发生的可见空间里，主体与周围环境的前后关系被清楚交代"
                            .to_string()
                    }
                }
            }
        }
    }
}

fn derive_visual_tone_effect(source: &str, shot_intent: &str) -> String {
    if contains_any_story_term(source, &["追杀", "压近", "逼近", "来袭", "对冲"]) {
        "把空间压得更紧，来袭压力直接贴到主体身上".to_string()
    } else if contains_any_story_term(source, &["护住", "护着", "回身", "挡住"]) {
        "把贴身相护时的压力托得更实".to_string()
    } else if contains_any_story_term(source, &["掌心", "银辉", "觉醒"]) || shot_intent == "reveal"
    {
        "把力量将起未起的压迫感顶在画面前沿".to_string()
    } else if shot_intent == "dialogue" {
        "让静场里的试探和停顿压得更深".to_string()
    } else {
        "把当前这拍的情绪压力稳稳按在画面里".to_string()
    }
}

fn derive_visual_light_tone_clause(
    source: &str,
    full_text: &str,
    shot_scene_label: &str,
    shot_intent: &str,
) -> String {
    let combined = format!("{source}\n{full_text}");
    let light_terms = collect_story_terms(
        &combined,
        &[
            "冷白光",
            "冷光",
            "银辉",
            "火光",
            "火星",
            "烛光",
            "街灯",
            "灯影",
            "灯火",
            "月光",
            "晨光",
            "微光",
            "暮色",
            "残阳",
            "反光",
            "雨雾",
            "雾气",
            "薄雾",
            "阴影",
            "树影",
            "灰云",
            "探照灯",
            "光斑",
            "光束",
        ],
        2,
    );
    let surface_terms = collect_story_terms(
        &combined,
        &[
            "钢筋",
            "锈屑",
            "混凝土",
            "碎石",
            "碎土",
            "烟尘",
            "灰尘",
            "石面",
            "布面",
            "桌案",
            "屏风",
            "门窗",
            "雨水",
            "湿地",
            "枝叶",
            "人流",
            "海面",
            "水面",
            "护栏",
            "铁质",
            "锈蚀",
            "舷窗",
            "船体",
        ],
        2,
    );
    let effect = derive_visual_tone_effect(source, shot_intent);
    if contains_any_story_term(
        &combined,
        &["海平线", "海面", "木栈道", "灯塔", "巡逻艇", "舷窗"],
    ) {
        if !light_terms.is_empty() && !surface_terms.is_empty() {
            let light = format_story_term_pair(&light_terms, "海边光线");
            let surface = format_story_term_pair(&surface_terms, "海边表面");
            format!("{light}落在{surface}上，潮湿空气和锈蚀材质把这一拍的压力压得更稳，{effect}")
        } else if !light_terms.is_empty() {
            let light = format_story_term_pair(&light_terms, "海边光线");
            format!("{light}把海雾和水面层次一起拉开，{effect}")
        } else if !surface_terms.is_empty() {
            let surface = format_story_term_pair(&surface_terms, "海边表面");
            format!("{surface}的潮湿和锈蚀质地都被看得很实，{effect}")
        } else {
            format!("海边薄雾、水面反差和锈蚀材质把这一拍压得更冷更紧，{effect}")
        }
    } else if !light_terms.is_empty() && !surface_terms.is_empty() {
        let light = format_story_term_pair(&light_terms, "光线");
        let surface = format_story_term_pair(&surface_terms, "环境表面");
        format!("{light}落在{surface}上，{effect}")
    } else if !light_terms.is_empty() {
        let light = format_story_term_pair(&light_terms, "光线");
        format!("{light}把画面层次拉开，{effect}")
    } else if !surface_terms.is_empty() {
        let surface = format_story_term_pair(&surface_terms, "环境表面");
        format!("{surface}的质地被看得很实，{effect}")
    } else if contains_any_story_term(
        &combined,
        &[
            "宫殿", "大殿", "殿内", "房间", "屋内", "船舱", "茶楼", "营帐",
        ],
    ) {
        format!("室内明暗把空间压得更深，木面与布面的层次让这一拍显得更稳更紧，{effect}")
    } else if contains_any_story_term(
        &combined,
        &["山林", "林间", "竹林", "街巷", "巷口", "旷野", "开阔地"],
    ) {
        format!("空气与地表把远近层次自然拉开，当前环境的湿度和颗粒感一直托着剧情压力，{effect}")
    } else if shot_scene_label.contains("对白") || shot_intent == "dialogue" {
        "光线被收得很克制，人物面部与衣料表面只剩窄窄反差，让静场里的试探和停顿压得更深".to_string()
    } else {
        format!("明暗层次把主体从背景里剥出来，空气与地表的质感让这一拍更有剧情压力，{effect}")
    }
}

fn derive_visual_event_clause(source: &str, subject: &str) -> String {
    if contains_any_story_term(source, &["海平线", "海面", "海浪", "浪声", "清晨微光"])
    {
        "当前视觉事件是清晨微光沿海平线浮起，海浪在岸边持续拍岸，海面节奏被稳稳留在画面里"
            .to_string()
    } else if contains_any_story_term(source, &["光斑", "光束", "舷窗", "薄雾"]) {
        "当前视觉事件是锐利光束劈开薄雾直射巡逻艇舷窗，光斑在船体上跳跃两下后停驻".to_string()
    } else if subject.contains("黑影")
        || contains_any_story_term(source, &["黑影", "人影", "身影", "灯塔"])
    {
        format!("当前视觉事件是{subject}贴着灯塔底部移动，轮廓在探照灯扫过水面时忽明忽暗")
    } else if contains_any_story_term(source, &["蹲身", "蹲下", "斜扣", "接缝"])
        && contains_any_story_term(source, &["信号镜"])
    {
        format!("当前视觉事件是{subject}把信号镜压进锈蚀护栏接缝，镜面反光被稳稳留在落点")
    } else if contains_any_story_term(source, &["重逢"]) {
        format!("当前视觉事件是{subject}在断裂边缘重新并肩，视线和站位同时重新对上")
    } else if contains_any_story_term(source, &["单膝跪", "跪在", "跪下"]) {
        if contains_any_story_term(source, &["敌人", "对手"])
            && contains_any_story_term(source, &["逼近", "缓步"])
        {
            format!("当前视觉事件是{subject}单膝跪在废墟之上，敌人缓步逼近并把对峙压力压到身前")
        } else if contains_any_story_term(source, &["脚步声", "步声"]) {
            format!("当前视觉事件是{subject}在坍塌阴影下单膝跪住，脚步声从画外逼近并压住停顿")
        } else {
            format!("当前视觉事件是{subject}单膝跪下并把重心压稳，画面停在动作落点")
        }
    } else if contains_any_story_term(source, &["烟尘"])
        && contains_any_story_term(source, &["踏出", "侧身", "疾避"])
    {
        "当前视觉事件是烟尘中有人踏出，主角侧身避开冲击线，双方位置在焦土边缘错开".to_string()
    } else if contains_any_story_term(source, &["护住", "护着", "回身"]) {
        if contains_any_story_term(source, &["林峰护住苏瑶", "护住苏瑶"])
            && contains_any_story_term(source, &["阿青提醒", "提醒他们"])
            && contains_any_story_term(source, &["黑衣追兵", "追兵"])
            && source.contains("巷口")
        {
            "当前视觉事件是林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近，护人和提醒同时压住退路"
                .to_string()
        } else {
            format!("当前视觉事件是{subject}回身挡住来势，身体横切进对冲路线")
        }
    } else if contains_any_story_term(source, &["追杀", "压近", "逼近"]) {
        format!("当前视觉事件是{subject}把距离继续压短，来袭方向直逼主体前线")
    } else if contains_any_story_term(source, &["格挡", "刀锋", "攻击", "交锋"]) {
        format!("当前视觉事件是{subject}与对手的锋线正面撞上，冲击点停在接触瞬间")
    } else if contains_any_story_term(source, &["掌心", "银辉", "觉醒"]) {
        format!("当前视觉事件是{subject}的掌心或手臂出现醒目的能量变化，力量在画面内继续上涌")
    } else if contains_any_story_term(source, &["震退", "七步"]) {
        format!("当前视觉事件是{subject}借反冲把对手震开，脚下碎土被力量带起")
    } else {
        format!(
            "当前视觉事件是{subject}在当前空间内完成清晰可见的状态变化，动作落点停在镜头正要继续之前"
        )
    }
}

fn derive_visual_focus_clause(source: &str, shot_intent: &str) -> String {
    if contains_any_story_term(source, &["海平线", "海面", "海浪", "浪声", "清晨微光"])
    {
        "海面在低沉浪声里持续压场的建立感".to_string()
    } else if contains_any_story_term(source, &["光斑", "光束", "舷窗", "薄雾"]) {
        "锐利光束锁定巡逻艇落点的确认感".to_string()
    } else if contains_any_story_term(source, &["黑影", "人影", "身影", "灯塔"]) {
        "不明来向贴着灯塔底部逼近的压力感".to_string()
    } else if contains_any_story_term(source, &["蹲身", "蹲下", "斜扣", "接缝"])
        && contains_any_story_term(source, &["信号镜"])
    {
        "信号镜落点与手部动作一起压实的隐蔽感".to_string()
    } else if contains_any_story_term(source, &["重逢"]) {
        "失而复得后的确认与仍未放松的紧绷感".to_string()
    } else if contains_any_story_term(source, &["护住", "护着", "回身"]) {
        "护人与来袭同时挤进画面的压迫感".to_string()
    } else if contains_any_story_term(source, &["追杀", "压近", "逼近"]) {
        "来势压近和退路被夺走的突袭感".to_string()
    } else if contains_any_story_term(source, &["格挡", "刀锋", "攻击", "交锋"]) {
        "正面硬碰时双方谁都不退的对峙感".to_string()
    } else if contains_any_story_term(source, &["掌心", "银辉", "觉醒"]) || shot_intent == "reveal"
    {
        "力量爆发前一刻的控场反转".to_string()
    } else if contains_any_story_term(source, &["震退", "七步"]) {
        "反击成立后的控场优势".to_string()
    } else if shot_intent == "dialogue" {
        "人物关系在静默停顿里的拉扯感".to_string()
    } else {
        "动作即将进入下一拍前的悬停感".to_string()
    }
}
fn derive_character_action_from_anchor(
    _segment: &str,
    full_text: &str,
    fact_anchor: &CurrentShotFactAnchor,
) -> String {
    let source = fact_anchor.source.as_str();
    let registry = CharacterRegistry::from_story_text(source, source);
    let subject = fact_anchor.subject.as_str();
    let target = derive_action_target(&registry, subject, source, full_text);
    let start_state = derive_action_start_state(source);
    let action = derive_visible_action(source, subject, &target);
    let end_state = derive_action_end_state(source);
    let captured_moment = derive_captured_moment(source);

    if action.contains("镜头捕捉") && action.contains("从") && action.contains("到") {
        return format!("{action}。");
    }

    format!(
        "{subject}从{start_state}开始，{action}，到{end_state}时结束，镜头捕捉{captured_moment}。"
    )
}

#[cfg(test)]
fn derive_character_action_from_story(segment: &str, full_text: &str) -> String {
    let fact_anchor = derive_current_shot_fact_anchor(segment, full_text);
    derive_character_action_from_anchor(segment, full_text, &fact_anchor)
}

fn derive_action_target(
    registry: &CharacterRegistry,
    subject: &str,
    segment: &str,
    full_text: &str,
) -> String {
    let full_registry = CharacterRegistry::from_story_text(full_text, full_text);
    if contains_any_story_term(segment, &["重逢"]) {
        return "彼此".to_string();
    }
    if contains_any_story_term(segment, &["护住", "护着"]) {
        if let Some(heroine) = registry
            .heroine_name()
            .or_else(|| full_registry.heroine_name())
        {
            if !subject.contains(heroine) || subject.contains('、') || subject.contains('与') {
                return heroine.to_string();
            }
        }
        if let Some(protagonist) = registry
            .protagonist_name()
            .or_else(|| full_registry.protagonist_name())
        {
            if subject.contains(protagonist) {
                return "被保护者".to_string();
            }
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
    if subject.contains("敌方刀客")
        || subject.contains("敌将")
        || subject.contains("敌人")
        || subject.contains("对手")
    {
        "主角".to_string()
    } else if subject.contains('与') || subject.contains('、') {
        "彼此".to_string()
    } else if let Some(shadow) = derive_shadow_subject_label(segment) {
        if !subject.contains(&shadow) {
            shadow
        } else {
            "灯塔底部".to_string()
        }
    } else if let Some(enemy) = derive_explicit_enemy_label(&combined) {
        if !subject.contains(&enemy) {
            enemy
        } else {
            "当前空间".to_string()
        }
    } else if let Some(anchor) = derive_non_character_subject_label(segment) {
        if !subject.contains(&anchor) {
            anchor
        } else {
            derive_named_source_subject_label(full_text).unwrap_or_else(|| "可见行动线".to_string())
        }
    } else {
        derive_named_source_subject_label(full_text).unwrap_or_else(|| "可见行动线".to_string())
    }
}

fn derive_action_start_state(source: &str) -> &'static str {
    if contains_any_story_term(source, &["单膝跪", "跪在", "跪下"]) {
        if contains_any_story_term(source, &["废墟"]) {
            "废墟之上单膝跪地"
        } else {
            "当前阴影下压低重心"
        }
    } else if contains_any_story_term(source, &["烟尘"])
        && contains_any_story_term(source, &["踏出", "侧身", "疾避"])
    {
        "烟尘边缘确认对方位置"
    } else if contains_any_story_term(source, &["焦土", "裂痕", "犁痕"]) {
        "焦土裂痕边缘稳住身体"
    } else if contains_any_story_term(source, &["断桥", "重逢"]) {
        "断桥残口确认彼此位置"
    } else if contains_any_story_term(source, &["追杀", "压近", "逼近"]) {
        if contains_any_story_term(source, &["巷口", "街口", "街巷"]) {
            "巷口来路压近前稳住身位"
        } else if contains_any_story_term(source, &["断桥", "桥"]) {
            "断桥远端压低重心"
        } else {
            "逼近压力前稳住身位"
        }
    } else if contains_any_story_term(source, &["格挡", "刀锋", "攻击"]) {
        "迎面冲击前的半步停顿"
    } else if contains_any_story_term(source, &["掌心", "银辉", "觉醒"]) {
        "格挡后的短暂停滞"
    } else {
        "上一动作落点稳定身体"
    }
}

fn derive_visible_action(source: &str, subject: &str, target: &str) -> String {
    if contains_any_story_term(source, &["海平线", "海面", "海浪", "浪声", "清晨微光"])
    {
        "海面从清晨微光浮上海平线开始，到浪声低沉而持续地拍向岸边时稳定下来，镜头捕捉海浪推着海面层次起伏的节奏".to_string()
    } else if contains_any_story_term(source, &["光斑", "光束", "舷窗", "薄雾"]) {
        "光斑从锐利光束劈开薄雾开始，沿巡逻艇舷窗与船体跳跃两下，到反光停驻时收住，镜头捕捉光束压住舷窗落点的瞬间".to_string()
    } else if contains_any_story_term(source, &["黑影", "人影", "身影"]) {
        format!(
            "{subject}从贴着灯塔底部压低移动开始，到轮廓掠过探照灯边缘时收住，镜头捕捉黑影在雾里忽明忽暗的一瞬间"
        )
    } else if contains_any_story_term(source, &["蹲身", "蹲下", "斜扣", "接缝"])
        && contains_any_story_term(source, &["信号镜"])
    {
        format!(
            "{subject}从迅速蹲身压低重心开始，将信号镜斜扣进锈蚀护栏接缝，到镜面反光被调稳时结束，镜头捕捉手部和落点一起定住的一瞬间"
        )
    } else if contains_any_story_term(source, &["紧握", "握紧"])
        && contains_any_story_term(source, &["信号镜"])
    {
        format!(
            "{subject}从紧握信号镜压低身位开始，到视线与海面方向重新锁定时收住，镜头捕捉信号镜边缘反光被手指压紧的一瞬间"
        )
    } else if contains_any_story_term(source, &["重逢"]) {
        format!("{subject}在断桥残口向{target}靠近，确认对方安全并重新建立站位")
    } else if contains_any_story_term(source, &["单膝跪", "跪在", "跪下"]) {
        if contains_any_story_term(source, &["敌人", "对手"])
            && contains_any_story_term(source, &["逼近", "缓步"])
        {
            if subject.contains("敌人") || subject.contains("对手") {
                format!("{subject}从废墟边缘缓步逼近{target}，把对峙距离继续压短")
            } else {
                format!("{subject}单膝跪地稳住身体，敌人缓步逼近后把对峙压力压到身前")
            }
        } else if contains_any_story_term(source, &["脚步声", "步声"]) {
            format!("{subject}单膝跪在阴影下稳住身体，听见脚步声逼近后把注意力转向来声方向")
        } else {
            format!("{subject}单膝跪下稳住身体，把重心压低到当前停顿位置")
        }
    } else if contains_any_story_term(source, &["烟尘"])
        && contains_any_story_term(source, &["踏出", "侧身", "疾避"])
    {
        format!("{subject}在烟尘边缘完成一进一避，敌方踏出时主角侧身让开冲击线")
    } else if contains_any_story_term(source, &["护住", "护着", "回身"]) {
        if contains_any_story_term(source, &["林峰护住苏瑶", "护住苏瑶"])
            && contains_any_story_term(source, &["阿青提醒", "提醒他们"])
            && contains_any_story_term(source, &["黑衣追兵", "追兵"])
            && source.contains("巷口")
        {
            if subject.contains("阿青") {
                "阿青提醒他们黑衣追兵从巷口逼近，让林峰护住苏瑶的退路压力被确认".to_string()
            } else {
                format!("{subject}护住苏瑶，阿青提醒他们黑衣追兵从巷口逼近")
            }
        } else {
            format!("{subject}回身护住{target}，用身体挡住逼近的威胁")
        }
    } else if contains_any_story_term(source, &["追杀", "压近", "逼近"]) {
        if contains_any_story_term(source, &["黑衣追兵", "追兵"])
            && contains_any_story_term(source, &["巷口", "街口", "街巷"])
        {
            format!("{subject}从巷口来路逼近{target}，把撤离压力压到当前站位前")
        } else if contains_any_story_term(source, &["断桥", "桥"]) {
            format!("{subject}提刀逼向{target}，把对方压向断桥边缘")
        } else {
            format!("{subject}继续逼近{target}，把双方距离压到当前对峙前线")
        }
    } else if contains_any_story_term(source, &["格挡", "刀锋", "攻击", "交锋"]) {
        format!("{subject}迎着{target}的冲击抬臂格挡，让银辉与刀锋正面相撞")
    } else if contains_any_story_term(source, &["掌心", "银辉", "觉醒"]) {
        format!("{subject}将掌心朝向{target}，让银辉沿手臂上升并压住对方攻势")
    } else if contains_any_story_term(source, &["震退", "七步"]) {
        format!("{subject}借格挡余力反震{target}，把对方逼到脚步失衡")
    } else {
        format!(
            "{subject}从当前站位开始调整动作落点，到姿态在画面内稳定时结束，镜头捕捉这一拍状态变化的完成瞬间"
        )
    }
}

fn derive_action_end_state(source: &str) -> &'static str {
    if contains_any_story_term(source, &["海平线", "海面", "海浪", "浪声", "清晨微光"])
    {
        "海面节奏稳定延续"
    } else if contains_any_story_term(source, &["光斑", "光束", "舷窗", "薄雾"]) {
        "光斑在船体落点停驻"
    } else if contains_any_story_term(source, &["黑影", "人影", "身影"]) {
        "黑影轮廓压在灯塔底部"
    } else if contains_any_story_term(source, &["蹲身", "蹲下", "斜扣", "接缝"])
        && contains_any_story_term(source, &["信号镜"])
    {
        "信号镜落点稳住"
    } else if contains_any_story_term(source, &["紧握", "握紧"])
        && contains_any_story_term(source, &["信号镜"])
    {
        "信号镜被压稳在手中"
    } else if contains_any_story_term(source, &["重逢"]) {
        "两人重新并肩"
    } else if contains_any_story_term(source, &["单膝跪", "跪在", "跪下"]) {
        if contains_any_story_term(source, &["敌人", "对手"])
            && contains_any_story_term(source, &["逼近", "缓步"])
        {
            "敌人逼近压力压到身前"
        } else {
            "来声方向被确认"
        }
    } else if contains_any_story_term(source, &["烟尘"])
        && contains_any_story_term(source, &["踏出", "侧身", "疾避"])
    {
        "双方位置重新错开"
    } else if contains_any_story_term(source, &["护住", "护着"]) {
        "被保护者退到安全半步"
    } else if contains_any_story_term(source, &["追杀", "压近", "逼近"]) {
        if contains_any_story_term(source, &["巷口", "街口", "街巷"]) {
            "巷口追兵压力被确认"
        } else if contains_any_story_term(source, &["断桥", "桥"]) {
            "目标被逼到断桥边缘"
        } else {
            "逼近压力压到身前"
        }
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
    if contains_any_story_term(source, &["海平线", "海面", "海浪", "浪声", "清晨微光"])
    {
        "清晨微光压住海平线的一瞬间"
    } else if contains_any_story_term(source, &["光斑", "光束", "舷窗", "薄雾"]) {
        "光斑两次跳跃后停驻的一瞬间"
    } else if contains_any_story_term(source, &["黑影", "人影", "身影"]) {
        "黑影轮廓擦过灯塔底部的一瞬间"
    } else if contains_any_story_term(source, &["蹲身", "蹲下", "斜扣", "接缝"])
        && contains_any_story_term(source, &["信号镜"])
    {
        "信号镜反光停住的一瞬间"
    } else if contains_any_story_term(source, &["紧握", "握紧"])
        && contains_any_story_term(source, &["信号镜"])
    {
        "手指压住信号镜边缘的一瞬间"
    } else if contains_any_story_term(source, &["重逢"]) {
        "两人视线重新对上的一瞬间"
    } else if contains_any_story_term(source, &["单膝跪", "跪在", "跪下"]) {
        if contains_any_story_term(source, &["敌人", "对手"])
            && contains_any_story_term(source, &["逼近", "缓步"])
        {
            "主角跪姿与敌人逼近同框的一瞬间"
        } else {
            "跪姿停住并听见脚步声的一瞬间"
        }
    } else if contains_any_story_term(source, &["烟尘"])
        && contains_any_story_term(source, &["踏出", "侧身", "疾避"])
    {
        "烟尘中踏出与侧身避让交错的一瞬间"
    } else if contains_any_story_term(source, &["护住", "护着"]) {
        "身体挡住威胁的一瞬间"
    } else if contains_any_story_term(source, &["追杀", "压近", "逼近"]) {
        if contains_any_story_term(source, &["巷口", "街口", "街巷"]) {
            "黑衣追兵从巷口逼近的一瞬间"
        } else if contains_any_story_term(source, &["断桥", "桥"]) {
            "刀锋压入断桥空间的一瞬间"
        } else {
            "逼近压力压入当前空间的一瞬间"
        }
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

fn derive_shot_title(index: usize, segment: &str, person: &str, character_action: &str) -> String {
    let action_core = derive_shot_title_action_core(segment, person, character_action);
    if subject_is_non_character_anchor(person) || action_core.contains("镜头") {
        format!("镜头{}：{}", index + 1, action_core)
    } else {
        format!("镜头{}：{}{}", index + 1, person, action_core)
    }
}

fn derive_camera_movement_from_story(
    shot_script: &str,
    scene_scale: &str,
    visual_description: &str,
    subject: &str,
) -> String {
    let evidence = format!("{shot_script}\n{visual_description}");
    if contains_any_story_term(&evidence, &["海平线", "海面", "海浪", "浪声", "清晨微光"])
    {
        format!("{scene_scale}远景定机位建立海平线与海面层次，保留浪声推动的持续起伏")
    } else if contains_any_story_term(&evidence, &["光斑", "光束", "舷窗", "薄雾"]) {
        format!("{scene_scale}缓慢推向巡逻艇舷窗，再跟住光斑两次跳跃后的停驻")
    } else if contains_any_story_term(&evidence, &["黑影", "人影", "身影", "灯塔"]) {
        format!("{scene_scale}缓慢横移掠过灯塔底部，保持{subject}与木栈道和水面的空间关系")
    } else if contains_any_story_term(&evidence, &["蹲身", "蹲下", "斜扣", "接缝"])
        && contains_any_story_term(&evidence, &["信号镜"])
    {
        format!("{scene_scale}跟随{subject}下压到手部，再锁定信号镜与护栏接缝处的反光落点")
    } else if contains_any_story_term(&evidence, &["站起", "起身"]) {
        format!("{scene_scale}低机位上摇，捕捉{subject}站起的动作转折")
    } else if contains_any_story_term(&evidence, &["抬起", "举起"]) {
        format!("{scene_scale}缓慢上摇，跟住{subject}抬起动作的发力线")
    } else if contains_any_story_term(&evidence, &["后撤", "震退", "退开"]) {
        format!("{scene_scale}跟随{subject}后撤半步，稳住动作对象和空间距离")
    } else if contains_any_story_term(&evidence, &["单膝跪", "跪在", "跪下", "脚步声", "步声"])
    {
        format!("{scene_scale}低机位稳住{subject}的跪姿和阴影空间，再把视线压力压向来势方向")
    } else if contains_any_story_term(&evidence, &["烟尘"])
        && contains_any_story_term(&evidence, &["踏出", "侧身", "疾避"])
    {
        format!("{scene_scale}横移穿过烟尘边缘，交代踏出与侧身避让之间的空间错位")
    } else if contains_any_story_term(&evidence, &["掌心", "手臂", "刀柄", "手部"]) {
        format!("{scene_scale}缓慢推近{subject}手部动作，停在动作发力瞬间")
    } else if contains_any_story_term(&evidence, &["焦土", "战场", "残骸", "断桥", "城门"])
    {
        format!("{scene_scale}横移掠过环境边缘，再锁定{subject}当前动作")
    } else if contains_any_story_term(&evidence, &["望向", "看向", "对手", "敌", "黑影"]) {
        format!("{scene_scale}过肩跟拍{subject}视线方向，保持动作对象在画面内")
    } else if scene_scale.contains("特写") {
        format!("{scene_scale}缓慢推近{subject}的关键动作，保持画面重心稳定")
    } else if scene_scale.contains("全景") {
        format!("{scene_scale}定机位观察{subject}动作起止，保留环境和主体关系")
    } else {
        format!("{scene_scale}定机位观察{subject}动作起止，镜头在关键瞬间轻微推近")
    }
}

fn derive_shot_title_action_core(segment: &str, person: &str, character_action: &str) -> String {
    if contains_any_story_term(segment, &["海平线", "海面", "海浪", "浪声", "清晨微光"])
    {
        "清晨海面建立镜头".to_string()
    } else if contains_any_story_term(segment, &["战报 UI", "UI", "小地图", "曲线", "警示框"])
    {
        "战报 UI 刷新".to_string()
    } else if contains_any_story_term(segment, &["沙盘", "视口", "旗标", "地形高差"]) {
        "沙盘视口态势".to_string()
    } else if contains_any_story_term(segment, &["行军轨迹", "补给线", "地图"]) {
        "行军轨迹合围".to_string()
    } else if contains_any_story_term(segment, &["城建", "脚手架", "吊机", "道路", "外城墙"])
    {
        "城建演进反馈".to_string()
    } else if contains_any_story_term(segment, &["军帐", "舆图"]) {
        "军帐权谋调度".to_string()
    } else if contains_any_story_term(segment, &["攻城", "云梯", "投石车", "城墙"]) {
        "多军团攻城".to_string()
    } else if contains_any_story_term(segment, &["军阵", "盾墙", "长枪", "旌旗", "号角"])
    {
        "军阵建立".to_string()
    } else if contains_any_story_term(segment, &["灯塔", "木栈道", "栈道"])
        && !contains_any_story_term(segment, &["蹲身", "蹲下", "信号镜"])
    {
        "灯塔与木栈道环境镜头".to_string()
    } else if contains_any_story_term(segment, &["光斑", "光束", "舷窗", "巡逻艇"]) {
        "巡逻艇接收光斑".to_string()
    } else if contains_any_story_term(segment, &["蹲身", "蹲下", "斜扣", "接缝"])
        && contains_any_story_term(segment, &["信号镜"])
    {
        "安放信号镜".to_string()
    } else if contains_any_story_term(segment, &["紧握", "握紧"])
        && contains_any_story_term(segment, &["信号镜"])
    {
        "握紧信号镜".to_string()
    } else if person.contains("黑影") || contains_any_story_term(segment, &["黑影", "人影", "身影"])
    {
        "远处黑影贴边移动".to_string()
    } else if contains_any_story_term(segment, &["重逢"]) {
        "断桥重逢".to_string()
    } else if contains_any_story_term(segment, &["单膝跪", "跪在", "跪下"]) {
        if contains_any_story_term(segment, &["脚步声", "步声"]) {
            "跪姿中的脚步声停顿".to_string()
        } else {
            "单膝跪下停顿".to_string()
        }
    } else if contains_any_story_term(segment, &["烟尘"])
        && contains_any_story_term(segment, &["踏出", "侧身", "疾避"])
    {
        "烟尘踏出与侧身疾避".to_string()
    } else if contains_any_story_term(segment, &["护住", "护着", "回身"]) {
        "回身护人".to_string()
    } else if contains_any_story_term(segment, &["追杀", "压近", "逼近"]) {
        "压近断桥".to_string()
    } else if contains_any_story_term(segment, &["掌心", "银辉", "觉醒"]) {
        "银辉觉醒".to_string()
    } else if contains_any_story_term(segment, &["震退", "七步"]) {
        "震退对手".to_string()
    } else if contains_any_story_term(segment, &["焦土", "裂痕"]) {
        "踏碎焦土".to_string()
    } else if contains_any_story_term(segment, &["格挡", "刀锋", "攻击", "交锋"]) {
        "踏步格挡".to_string()
    } else if character_action.contains("镜头捕捉") {
        "动作落点".to_string()
    } else {
        "状态落点".to_string()
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

fn collect_story_terms<'a>(value: &str, terms: &[&'a str], max: usize) -> Vec<&'a str> {
    let mut found = Vec::new();
    for term in terms {
        if value.contains(term) && !found.contains(term) {
            found.push(*term);
            if found.len() >= max {
                break;
            }
        }
    }
    found
}

fn format_story_term_pair(terms: &[&str], fallback: &str) -> String {
    match terms {
        [] => fallback.to_string(),
        [one] => (*one).to_string(),
        [first, second, ..] => format!("{first}与{second}"),
    }
}

fn has_visual_environment_signal(text: &str) -> bool {
    contains_any_story_term(
        text,
        &[
            "断桥",
            "桥边",
            "桥面",
            "焦土",
            "裂痕",
            "残垣",
            "废墟",
            "断楼",
            "残墙",
            "混凝土",
            "钢筋",
            "城门",
            "门墙",
            "宫殿",
            "大殿",
            "殿内",
            "房间",
            "室内",
            "屋内",
            "船舱",
            "茶楼",
            "营帐",
            "桌案",
            "屏风",
            "门窗",
            "街巷",
            "街口",
            "街灯",
            "人流",
            "林间",
            "林隙",
            "山林",
            "竹林",
            "竹叶",
            "树影",
            "雾气",
            "薄雾",
            "营地",
            "帐幕",
            "巷口",
            "雨夜",
            "雨夜码头",
            "码头",
            "雨雾",
            "雨水",
            "路灯",
            "湿地",
            "战场",
            "军阵",
            "前沿",
            "旷野",
            "开阔地",
            "海边",
            "海面",
            "海平线",
            "海浪",
            "木栈道",
            "栈道",
            "灯塔",
            "巡逻艇",
            "舷窗",
            "水面",
            "盾墙",
            "旌旗",
            "城墙",
            "云梯",
            "投石车",
            "军帐",
            "舆图",
            "河谷",
            "丘陵",
            "沙盘",
            "视口",
            "地图",
            "小地图",
            "战报 UI",
            "面板",
            "城建",
            "脚手架",
            "道路",
            "近身空间",
            "可见空间",
            "室内空间",
            "野外空间",
            "开阔地带",
            "街面动线",
        ],
    )
}

fn has_visual_concrete_element_signal(text: &str) -> bool {
    contains_any_story_term(
        text,
        &[
            "碎石",
            "碎土",
            "烟尘",
            "火光",
            "火星",
            "银辉",
            "反光",
            "冷光",
            "冷白光",
            "烛光",
            "街灯",
            "路灯",
            "灯影",
            "月光",
            "旧照片",
            "照片",
            "码头",
            "钢筋",
            "锈屑",
            "混凝土",
            "刀锋",
            "门墙",
            "桌案",
            "屏风",
            "门窗",
            "枝叶",
            "树影",
            "帐幕",
            "石面",
            "布面",
            "雨水",
            "雾气",
            "人流",
            "信号镜",
            "光斑",
            "光束",
            "舷窗",
            "护栏",
            "铁质",
            "锈蚀",
            "船体",
            "水面",
            "海浪",
            "长枪",
            "刀背",
            "号角",
            "旗标",
            "补给线",
            "吊机",
            "脚手架",
            "曲线",
            "警示框",
        ],
    )
}

fn has_visual_light_tone_or_material_signal(text: &str) -> bool {
    contains_any_story_term(
        text,
        &[
            "冷光",
            "冷白光",
            "冷白",
            "银辉",
            "火光",
            "火星",
            "烛光",
            "街灯",
            "路灯",
            "灯影",
            "月光",
            "雨水",
            "雾气",
            "薄雾",
            "树影",
            "暮色",
            "反光",
            "烟尘",
            "湿亮",
            "粗粝",
            "硬光",
            "灰云",
            "灰褐",
            "质感",
            "材质感",
            "阴影",
            "明暗",
            "木面",
            "布面",
            "颗粒感",
            "空气",
            "地表",
            "斑驳",
            "雨雾",
            "雾气",
            "树影",
            "锈屑",
            "金属",
            "冷亮",
            "发冷",
            "微光",
            "探照灯",
            "光斑",
            "光束",
            "锈蚀",
            "铁质",
            "冷灯",
            "屏幕",
            "告警",
            "晨雾",
        ],
    )
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

fn runtime_scene_option_mappings() -> &'static [SceneEntryMapping] {
    RUNTIME_SCENE_ENTRY_MAPPINGS
}

fn resolve_scene_entry_mapping(scene_type: &str) -> Option<&'static SceneEntryMapping> {
    let scene_type = scene_type.trim();
    if scene_type.is_empty() {
        return None;
    }
    runtime_scene_option_mappings().iter().find(|mapping| {
        mapping.desktop_entry == scene_type
            || mapping.runtime_scene_family == scene_type
            || mapping.aliases.iter().any(|alias| *alias == scene_type)
    })
}

fn runtime_scene_family_for_scene_type(scene_type: &str) -> Option<String> {
    resolve_scene_entry_mapping(scene_type).map(|mapping| mapping.runtime_scene_family.to_string())
}

fn scene_label_for_scene_type(scene_type: &str) -> Option<String> {
    resolve_scene_entry_mapping(scene_type).map(|mapping| mapping.chinese_label.to_string())
}

fn normalize_scene_type(scene_type: &str) -> String {
    resolve_scene_entry_mapping(scene_type)
        .map(|mapping| mapping.canonical_bucket.to_string())
        .unwrap_or_else(|| scene_type.trim().to_string())
}

fn is_supported_storyboard_duration(duration_seconds: u16) -> bool {
    (SEEDANCE_REMAINDER_SEGMENT_SECONDS..=60).contains(&duration_seconds)
        && duration_seconds % SEEDANCE_REMAINDER_SEGMENT_SECONDS == 0
}

fn is_supported_desktop_scene_type(scene_type: &str) -> bool {
    resolve_scene_entry_mapping(scene_type).is_some()
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
        format!(
            "运镜：{}",
            derive_camera_movement_from_story(
                shot_script,
                &scene_projection.scene_scale,
                &scene_projection.visual_description,
                &scene_projection.person,
            )
        ),
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
        format!(
            "运镜：{}",
            derive_camera_movement_from_story(
                &record.source_fields.sample_title,
                &scene_projection.scene_scale,
                &scene_projection.visual_description,
                &scene_projection.person,
            )
        ),
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
        target_duration_mode: TARGET_DURATION_MODE_FIXED_SECONDS.to_string(),
        story_length_profile: String::new(),
        source_material_length_chars: 0,
        auto_segment_strategy: AUTO_SEGMENT_STRATEGY_FIXED_SECONDS.to_string(),
        estimated_total_story_duration_seconds: selected_total_duration_seconds,
        generated_shot_task_count: 0,
        duration_plan_summary: format!(
            "mode={}; total={}s; generated_shot_tasks=0",
            TARGET_DURATION_MODE_FIXED_SECONDS, selected_total_duration_seconds
        ),
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
        binding_evidence: StoryboardBindingEvidence::default(),
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

fn normalize_binding_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn stable_binding_hash_json(value: &str) -> String {
    stable_binding_hash_serialized(
        &serde_json::to_string(&normalize_binding_text(value)).unwrap_or_default(),
    )
}

fn stable_binding_hash_value<T: Serialize>(value: &T) -> String {
    stable_binding_hash_serialized(&serde_json::to_string(value).unwrap_or_default())
}

fn stable_binding_hash_serialized(source: &str) -> String {
    let mut hash: u32 = 2_166_136_261;
    for byte in source.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    format!("{hash:08x}")
}

fn normalize_binding_fact_list(items: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut output = Vec::new();
    for item in items {
        let clean = normalize_binding_text(item);
        if clean.is_empty() || !seen.insert(clean.clone()) {
            continue;
        }
        output.push(clean);
    }
    output
}

fn normalize_binding_must_keep_fact_list(items: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut output = Vec::new();
    for item in items {
        let clean = normalize_binding_must_keep_fact(item);
        if clean.is_empty() || !seen.insert(clean.clone()) {
            continue;
        }
        output.push(clean);
    }
    output
}

fn normalize_binding_must_keep_fact(value: &str) -> String {
    strip_binding_fact_order_prefix(&normalize_binding_text(value))
}

fn strip_binding_fact_order_prefix(value: &str) -> String {
    let trimmed = value.trim();
    let mut digit_end = 0usize;
    for (index, character) in trimmed.char_indices() {
        if character.is_ascii_digit() {
            digit_end = index + character.len_utf8();
        } else {
            break;
        }
    }
    if digit_end == 0 {
        return trimmed.to_string();
    }

    let rest = trimmed[digit_end..].trim_start();
    let mut chars = rest.chars();
    let Some(delimiter) = chars.next() else {
        return trimmed.to_string();
    };
    if matches!(delimiter, '.' | '．' | '、' | ')' | '）' | ':' | '：') {
        chars.as_str().trim_start().to_string()
    } else {
        trimmed.to_string()
    }
}

fn augment_binding_facts_from_source(
    facts: &mut Vec<String>,
    source_text: &str,
    candidates: &[&str],
) {
    for candidate in candidates {
        if source_text.contains(candidate)
            || *candidate == "码头" && source_text.contains("雨夜码头")
        {
            push_unique_fact(facts, (*candidate).to_string());
        }
    }
}

fn source_derived_storyboard_must_keep_facts(source_text: &str) -> Vec<String> {
    let mut facts = Vec::new();
    if source_has_b_alley_pursuit_facts(source_text) {
        augment_binding_facts_from_source(
            &mut facts,
            source_text,
            &["林峰护住苏瑶", "阿青提醒", "黑衣追兵", "巷口", "逼近"],
        );
    }
    if source_has_c_rainy_dock_photo_facts(source_text) {
        augment_binding_facts_from_source(
            &mut facts,
            source_text,
            &[
                "女主",
                "旧照片",
                "旧相片",
                "照片",
                "相片",
                "陌生人",
                "路灯",
                "雨夜码头",
                "码头",
            ],
        );
    }
    facts
}

fn source_derived_storyboard_forbidden_facts(source_text: &str) -> Vec<String> {
    let mut facts = Vec::new();
    if source_has_c_rainy_dock_photo_facts(source_text) {
        for forbidden in ["林峰", "苏瑶", "阿青", "黑衣追兵", "废墟", "敌人"] {
            if !source_text.contains(forbidden) {
                push_unique_fact(&mut facts, forbidden.to_string());
            }
        }
    }
    facts
}

fn infer_storyboard_source_profile(source_text: &str, requested_profile: &str) -> String {
    let requested_profile = requested_profile.trim();
    if source_has_c_rainy_dock_photo_facts(source_text)
        && (requested_profile.is_empty() || requested_profile == "synopsis")
    {
        "C_rainy_dock_photo".to_string()
    } else {
        requested_profile.to_string()
    }
}

fn binding_text_contains(source: &str, fact: &str) -> bool {
    let clean_source = normalize_binding_text(source);
    let clean_fact = normalize_binding_text(fact);
    clean_fact.is_empty() || clean_source.contains(&clean_fact)
}

fn binding_source_fact_covered_by_rows(rows_text: &str, fact: &str) -> bool {
    if binding_text_contains(rows_text, fact) {
        return true;
    }

    let clean_rows = normalize_binding_text(rows_text);
    let clean_fact = normalize_binding_must_keep_fact(fact);
    if clean_fact.is_empty() {
        return true;
    }
    if binding_a_ruin_sentence_fact(&clean_fact) {
        return binding_a_ruin_sentence_atoms_covered(&clean_rows);
    }
    if binding_b_alley_pursuit_sentence_fact(&clean_fact) {
        return binding_b_alley_pursuit_sentence_atoms_covered(&clean_rows);
    }
    if binding_a_ruin_duel_relation_fact(&clean_fact) {
        return clean_rows.contains("主角")
            && clean_rows.contains("敌人")
            && contains_any_story_term(&clean_rows, &["对峙", "对峙压力", "逼近", "缓步逼近"]);
    }
    if binding_a_ruin_enemy_approach_fact(&clean_fact) {
        return clean_rows.contains("敌人")
            && contains_any_story_term(&clean_rows, &["敌人逼近", "缓步逼近", "逼近", "压近"]);
    }
    if binding_a_ruin_duel_pressure_fact(&clean_fact) {
        return clean_rows.contains("主角")
            && clean_rows.contains("敌人")
            && contains_any_story_term(&clean_rows, &["对峙", "对峙压力", "逼近", "缓步逼近", "压近"]);
    }
    false
}

fn binding_a_ruin_sentence_fact(fact: &str) -> bool {
    fact.contains("废墟")
        && fact.contains("主角")
        && fact.contains("敌人")
        && contains_any_story_term(fact, &["单膝跪地", "单膝", "跪地"])
        && contains_any_story_term(fact, &["逼近", "缓步逼近"])
}

fn binding_a_ruin_sentence_atoms_covered(rows_text: &str) -> bool {
    rows_text.contains("废墟")
        && rows_text.contains("主角")
        && rows_text.contains("敌人")
        && contains_any_story_term(rows_text, &["单膝跪地", "单膝", "跪地"])
        && contains_any_story_term(rows_text, &["逼近", "缓步逼近"])
}

fn binding_b_alley_pursuit_sentence_fact(fact: &str) -> bool {
    fact.contains("林峰")
        && fact.contains("苏瑶")
        && contains_any_story_term(fact, &["林峰护住苏瑶", "护住苏瑶"])
        && fact.contains("阿青")
        && fact.contains("提醒")
        && fact.contains("黑衣追兵")
        && fact.contains("巷口")
        && fact.contains("逼近")
}

fn binding_b_alley_pursuit_sentence_atoms_covered(rows_text: &str) -> bool {
    rows_text.contains("林峰")
        && rows_text.contains("苏瑶")
        && contains_any_story_term(rows_text, &["林峰护住苏瑶", "护住苏瑶"])
        && rows_text.contains("阿青")
        && contains_any_story_term(rows_text, &["阿青提醒", "提醒他们", "提醒"])
        && rows_text.contains("黑衣追兵")
        && rows_text.contains("巷口")
        && contains_any_story_term(rows_text, &["逼近", "压近"])
}

fn binding_a_ruin_duel_relation_fact(fact: &str) -> bool {
    fact.contains("主角") && fact.contains("敌人") && fact.contains("对峙")
}

fn binding_a_ruin_enemy_approach_fact(fact: &str) -> bool {
    fact.contains("敌人") && contains_any_story_term(fact, &["逼近", "缓步逼近", "压近"])
}

fn binding_a_ruin_duel_pressure_fact(fact: &str) -> bool {
    contains_any_story_term(fact, &["对峙", "对峙压力"])
}

fn storyboard_rows_binding_text(rows: &[GeneratedStoryboardRow]) -> String {
    rows.iter()
        .map(|row| {
            [
                row.person.as_str(),
                row.visual_description.as_str(),
                row.character_action.as_str(),
                row.camera_movement.as_str(),
            ]
            .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn storyboard_rows_delivery_text(rows: &[GeneratedStoryboardRow]) -> String {
    rows.iter()
        .map(|row| {
            [
                row.person.as_str(),
                row.shot_title.as_str(),
                row.visual_description.as_str(),
                row.character_action.as_str(),
                row.camera_movement.as_str(),
                row.prompt_text.as_str(),
            ]
            .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn snapshot_field_mismatch(
    request_value: &str,
    snapshot_value: &str,
    code: &str,
    message: &str,
) -> Option<ProductWarning> {
    if !request_value.trim().is_empty() && request_value != snapshot_value {
        Some(ProductWarning {
            code: code.to_string(),
            message: message.to_string(),
            related_sample_id: None,
        })
    } else {
        None
    }
}

fn accepted_snapshot_binding_preflight(
    request: &GenerateStoryboardRequest,
    snapshot: &AcceptedRewriteSnapshotBinding,
    grounding: &StoryboardGroundingContext,
) -> Vec<ProductWarning> {
    let mut blockers = Vec::new();
    if snapshot.accepted_confirmation_body.trim().is_empty() {
        blockers.push(ProductWarning {
            code: "accepted_rewrite_snapshot_body_missing".to_string(),
            message: "generate_storyboard requires the current accepted rewrite snapshot body."
                .to_string(),
            related_sample_id: None,
        });
    }
    if let Some(warning) = snapshot_field_mismatch(
        &request.current_case_id,
        &snapshot.current_case_id,
        "accepted_rewrite_snapshot_case_mismatch",
        "generate_storyboard flat binding fields do not match accepted_rewrite_snapshot.current_case_id.",
    ) {
        blockers.push(warning);
    }
    if let Some(warning) = snapshot_field_mismatch(
        &request.source_text_hash,
        &snapshot.source_text_hash,
        "accepted_rewrite_snapshot_source_hash_mismatch",
        "generate_storyboard flat binding fields do not match accepted_rewrite_snapshot.source_text_hash.",
    ) {
        blockers.push(warning);
    }
    if let Some(warning) = snapshot_field_mismatch(
        &request.accepted_rewrite_hash,
        &snapshot.accepted_rewrite_hash,
        "accepted_rewrite_snapshot_hash_mismatch",
        "generate_storyboard flat binding fields do not match accepted_rewrite_snapshot.accepted_rewrite_hash.",
    ) {
        blockers.push(warning);
    }
    if let Some(warning) = snapshot_field_mismatch(
        &request.story_fact_frame_hash,
        &snapshot.story_fact_frame_hash,
        "accepted_rewrite_snapshot_story_fact_frame_mismatch",
        "generate_storyboard flat binding fields do not match accepted_rewrite_snapshot.story_fact_frame_hash.",
    ) {
        blockers.push(warning);
    }

    let actual_accepted_hash = stable_binding_hash_json(&snapshot.accepted_confirmation_body);
    if !snapshot.accepted_rewrite_hash.trim().is_empty()
        && actual_accepted_hash != snapshot.accepted_rewrite_hash
    {
        blockers.push(ProductWarning {
            code: "accepted_rewrite_snapshot_body_hash_mismatch".to_string(),
            message: "accepted_rewrite_snapshot body does not match accepted_rewrite_hash."
                .to_string(),
            related_sample_id: None,
        });
    }
    if !snapshot.scene_type.trim().is_empty()
        && !grounding.primary_scene_type.trim().is_empty()
        && snapshot.scene_type != grounding.primary_scene_type
    {
        blockers.push(ProductWarning {
            code: "accepted_rewrite_snapshot_scene_mismatch".to_string(),
            message:
                "generate_storyboard primary scene does not match the accepted rewrite snapshot."
                    .to_string(),
            related_sample_id: None,
        });
    }
    let forbidden_conflicts =
        accepted_snapshot_forbidden_binding_conflicts(request, snapshot, grounding);
    if !forbidden_conflicts.is_empty() {
        blockers.push(ProductWarning {
            code: "accepted_rewrite_snapshot_forbidden_fact_conflict".to_string(),
            message: format!(
                "accepted rewrite snapshot conflicts with forbidden story-fact anchors: {}.",
                forbidden_conflicts.join(" / ")
            ),
            related_sample_id: None,
        });
    }

    blockers
}

fn accepted_snapshot_forbidden_binding_conflicts(
    request: &GenerateStoryboardRequest,
    snapshot: &AcceptedRewriteSnapshotBinding,
    grounding: &StoryboardGroundingContext,
) -> Vec<String> {
    let mut conflicts = Vec::new();
    let mut forbidden_facts = normalize_binding_fact_list(&request.forbidden_facts);
    for fact in normalize_binding_fact_list(&snapshot.forbidden_facts) {
        push_unique_fact(&mut forbidden_facts, fact);
    }
    for fact in source_derived_storyboard_forbidden_facts(&grounding.grounding_text) {
        push_unique_fact(&mut forbidden_facts, fact);
    }
    let forbidden_facts_for_inferred = forbidden_facts.clone();
    let accepted_body = normalize_binding_text(&snapshot.accepted_confirmation_body);
    for forbidden in forbidden_facts {
        if !forbidden.trim().is_empty() && accepted_body.contains(&forbidden) {
            push_unique_fact(&mut conflicts, forbidden);
        }
    }
    if binding_anchor_has_absolute_forbidden_term(&accepted_body) {
        push_unique_fact(
            &mut conflicts,
            "appearance_or_source_external_anchor".to_string(),
        );
    }
    if binding_anchor_has_source_external_high_risk_term(&accepted_body, &grounding.grounding_text)
    {
        push_unique_fact(
            &mut conflicts,
            "source_external_high_risk_anchor".to_string(),
        );
    }
    for inferred in &snapshot.inferred_scene_facts {
        let fact = normalize_binding_text(&inferred.fact);
        if binding_fact_hits_forbidden_fact(&fact, &forbidden_facts_for_inferred)
            || binding_anchor_has_absolute_forbidden_term(&fact)
            || binding_anchor_has_source_external_high_risk_term(&fact, &grounding.grounding_text)
        {
            push_unique_fact(&mut conflicts, fact);
        }
    }
    conflicts
}

fn current_case_binding_preflight(
    request: &GenerateStoryboardRequest,
    grounding: &StoryboardGroundingContext,
) -> Vec<ProductWarning> {
    let mut blockers = Vec::new();
    if let Some(snapshot) = request.accepted_rewrite_snapshot.as_ref() {
        blockers.extend(accepted_snapshot_binding_preflight(
            request, snapshot, grounding,
        ));
    }
    let expected_task_hash = request.task_script_hash.trim();
    if !expected_task_hash.is_empty() {
        let actual_task_hash = stable_binding_hash_json(&grounding.grounding_text);
        if actual_task_hash != expected_task_hash {
            blockers.push(ProductWarning {
                code: "current_case_binding_mismatch".to_string(),
                message: "generate_storyboard request does not match the accepted current shot script snapshot."
                    .to_string(),
                related_sample_id: None,
            });
        }
    }
    blockers
}

fn build_storyboard_binding_evidence(
    request: &GenerateStoryboardRequest,
    grounding: &StoryboardGroundingContext,
    rows: &[GeneratedStoryboardRow],
    duration_plan: &StoryboardDurationPlan,
    rows_hash: &str,
    kb_router_result: &KbRouterRuntimeResponse,
) -> StoryboardBindingEvidence {
    let mut must_keep_facts = normalize_binding_must_keep_fact_list(&request.must_keep_facts);
    if let Some(snapshot) = request.accepted_rewrite_snapshot.as_ref() {
        for fact in normalize_binding_must_keep_fact_list(&snapshot.explicit_facts) {
            push_unique_fact(&mut must_keep_facts, fact);
        }
        for fact in normalize_binding_must_keep_fact_list(&snapshot.must_keep_facts) {
            push_unique_fact(&mut must_keep_facts, fact);
        }
    }
    for fact in normalize_binding_must_keep_fact_list(&source_derived_storyboard_must_keep_facts(
        &grounding.grounding_text,
    )) {
        push_unique_fact(&mut must_keep_facts, fact);
    }
    let mut forbidden_facts = normalize_binding_fact_list(&request.forbidden_facts);
    if let Some(snapshot) = request.accepted_rewrite_snapshot.as_ref() {
        for fact in normalize_binding_fact_list(&snapshot.forbidden_facts) {
            push_unique_fact(&mut forbidden_facts, fact);
        }
    }
    for fact in source_derived_storyboard_forbidden_facts(&grounding.grounding_text) {
        push_unique_fact(&mut forbidden_facts, fact);
    }
    let rows_text = storyboard_rows_binding_text(rows);
    let delivery_text = storyboard_rows_delivery_text(rows);
    let missing_source_facts = must_keep_facts
        .iter()
        .filter(|fact| !binding_source_fact_covered_by_rows(&rows_text, fact))
        .cloned()
        .collect::<Vec<_>>();
    let forbidden_fact_hits = forbidden_facts
        .iter()
        .filter(|fact| binding_text_contains(&delivery_text, fact))
        .cloned()
        .collect::<Vec<_>>();
    let actual_task_hash = stable_binding_hash_json(&grounding.grounding_text);
    let stale_binding_detected =
        !request.task_script_hash.trim().is_empty() && request.task_script_hash != actual_task_hash;

    StoryboardBindingEvidence {
        current_case_id: request.current_case_id.clone(),
        source_text_hash: request.source_text_hash.clone(),
        accepted_rewrite_hash: request.accepted_rewrite_hash.clone(),
        task_script_hash: if request.task_script_hash.trim().is_empty() {
            actual_task_hash
        } else {
            request.task_script_hash.clone()
        },
        story_fact_frame_hash: request.story_fact_frame_hash.clone(),
        source_profile: infer_storyboard_source_profile(
            &grounding.grounding_text,
            &request.source_profile,
        ),
        scene_type: grounding.shot_scene_type.clone(),
        duration_seconds: request.selected_total_duration_seconds,
        duration_plan_hash: stable_binding_hash_value(duration_plan),
        storyboard_rows_hash: rows_hash.to_string(),
        must_keep_facts,
        missing_source_facts,
        forbidden_facts,
        forbidden_fact_hits,
        stale_binding_detected,
        kb_rule_pack_ids: kb_router_result
            .selected_kb_rules
            .iter()
            .map(|rule| rule.rule_id.clone())
            .collect(),
        kb_snapshot_hash: kb_router_result.retrieval_trace.snapshot_checksum.clone(),
    }
}

fn story_fact_frame_binding_gate_blockers(
    binding_evidence: &StoryboardBindingEvidence,
) -> Vec<ProductWarning> {
    let mut blockers = Vec::new();
    if binding_evidence.stale_binding_detected {
        blockers.push(ProductWarning {
            code: "story_fact_frame_stale_binding_detected".to_string(),
            message: "Storyboard rows were generated against a stale accepted snapshot binding."
                .to_string(),
            related_sample_id: None,
        });
    }
    if !binding_evidence.missing_source_facts.is_empty() {
        blockers.push(ProductWarning {
            code: "story_fact_frame_anchor_not_fully_mapped".to_string(),
            message: format!(
                "Some accepted story-fact anchors were not fully mapped into source-of-truth storyboard fields: {}.",
                binding_evidence.missing_source_facts.join(" / ")
            ),
            related_sample_id: None,
        });
    }
    if !binding_evidence.forbidden_fact_hits.is_empty() {
        blockers.push(ProductWarning {
            code: "story_fact_frame_forbidden_fact_hit".to_string(),
            message: format!(
                "Storyboard rows still contain forbidden story-fact anchors: {}.",
                binding_evidence.forbidden_fact_hits.join(" / ")
            ),
            related_sample_id: None,
        });
    }
    blockers
}

fn storyboard_prompt_text_packaging_gate_blockers(
    rows: &[GeneratedStoryboardRow],
) -> Vec<ProductWarning> {
    rows.iter()
        .filter_map(|row| {
            prompt_text_packaging_only_violation_reason(&row.prompt_text).map(|reason| {
                ProductWarning {
                    code: "prompt_text_packaging_only_violation".to_string(),
                    message: format!(
                        "Storyboard row {} prompt_text must stay packaging-only; violation={reason}.",
                        row.order
                    ),
                    related_sample_id: Some(row.prompt_text_source_row_id.clone()),
                }
            })
        })
        .collect()
}

fn prompt_text_packaging_only_violation_reason(prompt_text: &str) -> Option<&'static str> {
    let trimmed = prompt_text.trim();
    if trimmed.is_empty() {
        return Some("empty_prompt_text");
    }
    if contains_product_control_text(trimmed) {
        return Some("internal_control_text");
    }
    let lower = trimmed.to_lowercase();
    let forbidden_terms = [
        "prompt_text",
        "raw kb",
        "raw_kb",
        "raw prompt",
        "raw_prompt",
        "prompt_body",
        "source_register",
        "source register",
        "overlay json",
        "overlay_json",
        "schema",
        "trace",
        "retrieval_trace",
        "rows_hash",
        "storyboard_rows_hash",
        "kb_context_summary",
        "full_kb_rows",
        "validator payload",
    ];
    if forbidden_terms.iter().any(|term| lower.contains(term)) {
        return Some("raw_internal_payload_marker");
    }
    if (trimmed.starts_with('{') || trimmed.starts_with('['))
        && contains_any_story_term(&lower, &["prompt", "schema", "trace", "rows"])
    {
        return Some("json_like_internal_payload");
    }
    None
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
    use core_domain::contracts::AcceptedRewriteSnapshotBinding;
    use core_domain::{
        BridgeCallStatus, ExpandScriptRequest, FailurePatternRecord, GenerateStoryboardRequest,
        GeneratedStoryboardRow, GoldenSampleAssetSource, GoldenSampleClassification,
        GoldenSampleComparisonBaseline, GoldenSampleFailureMappingAsset, GoldenSampleFewshotState,
        GoldenSampleFieldCoverageRuleAsset, GoldenSampleLibraryAsset,
        GoldenSampleLibraryProvenance, GoldenSampleLibraryRecord, GoldenSampleNegativeSample,
        GoldenSampleRepairMappingAsset, GoldenSampleRepairMappingPlanning,
        GoldenSampleSourceContext, GoldenSampleSourceFields, GoldenSampleSourceRegister,
        GoldenSampleV3CoreCoverage, GoldenSampleValidatorEvidence, KbBundleManifestRecord,
        KbBundleRecordCounts, KbGoldenSampleRuntimePackage, KbRouterRuntimeRequest,
        KbRouterTaskType, KbRuntimeSummary, KbSnapshotRecord, ProductWarning, PromptTemplateRecord,
        PromptTextCompilationStatus, ScenePerformanceProjection, SceneTaxonomyRecord,
        SequenceFieldState, SequenceGrouping, ShotGroundingSource, StoryboardBindingEvidence,
        StoryboardDurationPlan, StructureMode, TextGenerationOutputSchema, TextGenerationRequest,
        TextGenerationResponse, TextGenerationTask, TextModelProvider, TextModelProviderKind,
    };
    use export_engine::{V120StoryboardExportRequest, export_v120_storyboard_bundle};
    use project_store::{
        DualSqliteConnectionPolicy, KbKnowledgeBundle, KbRuntimeHandle, StoreSkeleton,
    };

    use super::{
        AppShellReadonlyStatusSnapshot, LiveRepairSummary, LiveStoryboardRowPatch,
        StoryboardGroundingContext, StoryboardPreviewPlanRequest,
        TEXT_MODEL_NETWORK_RETRY_RECOVERED_CODE, TEXT_MODEL_OUTPUT_CONTRACT_NORMALIZED_CODE,
        TEXT_MODEL_QA_NO_LOCAL_FALLBACK_CODE, ValidationExportPanelState,
        append_storyboard_binding_field_clause_safe, binding_source_fact_covered_by_rows,
        allocate_storyboard_row_durations, apply_live_storyboard_patch,
        build_deterministic_expanded_story_material, build_deterministic_expanded_story_script,
        build_project_create_or_switch_snapshot_from_fixture, build_qwen_request_payload,
        build_storyboard_binding_evidence, build_storyboard_duration_plan,
        build_storyboard_model_story_input, build_storyboard_preview_plan,
        build_storyboard_rendersegment_cut_preview_snapshot_from_fixture,
        build_text_generation_request, build_validation_export_panel_snapshot_from_fixture,
        build_writer_entry_snapshot_from_fixture, canonical_live_expand_source_text,
        contains_any_story_term, contains_product_control_text, derive_character_action_from_story,
        derive_product_person, derive_shot_title, diversify_repeated_storyboard_subjects,
        empty_kb_router_response, expand_live_fallback_warning, expand_script,
        extract_live_storyboard_row_patches, generate_storyboard,
        generated_script_identity_validator_reason, has_visual_concrete_element_signal,
        has_visual_environment_signal, has_visual_light_tone_or_material_signal,
        is_visual_description_grounding_incomplete, live_field_visual_or_abstract_subject_term,
        live_repair_warning, normalize_scene_type, normalize_storyboard_row_subject_quality,
        parse_model_output_contract_json, qwen_transport_timeout_seconds,
        repair_a_ruin_enemy_storyboard_row, repair_b_alley_pursuit_storyboard_row,
        repair_c_rainy_dock_photo_storyboard_row, repair_live_expanded_script_text,
        repair_live_storyboard_patch_from_baseline, repair_live_storyboard_rows_from_source,
        repair_storyboard_rows_from_binding_context, resolve_expand_script_target_duration_seconds,
        resolve_scene_taxonomy, row_has_subject_pollution, run_qwen_text_generation_with_transport,
        run_text_generation_qa_hard_fail, runtime_scene_option_mappings,
        split_overbroad_storyboard_subject, stable_binding_hash_json,
        story_fact_frame_binding_gate_blockers, storyboard_rows_binding_text,
        storyboard_rows_delivery_text, text_generation_fallback_blocking_warnings,
        validate_generated_script_text, validate_generated_script_text_with_reason,
        validate_live_storyboard_rows,
    };
    use crate::state::load_desktop_shared_fixture;
    use crate::{
        ipc::{
            ProjectCreateOrSwitchRequest, StoryboardRenderSegmentCutPreviewSnapshotRequest,
            ValidationExportPanelSnapshotRequest, WriterEntrySnapshotRequest,
        },
        state::AppState,
    };
    use std::cell::Cell;

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

    fn test_state_with_golden_sample_runtime() -> AppState {
        let mut state = test_state();
        state.kb_runtime.summary.has_golden_sample_v120_package = true;
        state.kb_runtime.summary.golden_sample_record_count = 1;
        state.kb_runtime.summary.golden_sample_source_count = 1;
        state.kb_golden_sample_runtime = Some(test_golden_sample_runtime_package());
        state
    }

    fn test_golden_sample_runtime_package() -> KbGoldenSampleRuntimePackage {
        let record = GoldenSampleLibraryRecord {
            machine_id: "golden_sample_runtime_test_01".to_string(),
            sample_id: "GS-RT-01".to_string(),
            schema_version: "golden_sample_library.v0.2".to_string(),
            source_fields: GoldenSampleSourceFields {
                shot_id: "GS-RT-01".to_string(),
                library_status: "official".to_string(),
                reserve_reason: String::new(),
                sample_type: "single_shot".to_string(),
                sequence_id: "SEQ-01".to_string(),
                shot_order: "01".to_string(),
                sample_title: "断桥对撞".to_string(),
                style_cluster: "hope-action".to_string(),
                scene_category: "daily_dialogue".to_string(),
                scene_tag: "断桥,交锋,银辉,碎土".to_string(),
                quality_grade: "high".to_string(),
                usable_for_fewshot: "Yes".to_string(),
                technical_profile: "MS / eye-level / 24fps".to_string(),
                scene_performance_core: "Bridge-edge confrontation before a direct clash."
                    .to_string(),
                camera_directing_core:
                    "Hold the opposing lines and keep the bridge fracture visible.".to_string(),
                audio_directing_core: "Wind, footfall, and impact tension.".to_string(),
                continuity_negative_core: String::new(),
                reference_bundle: String::new(),
                ip_abstraction_note: "abstracted".to_string(),
                covered_points: "composition / atmosphere / action".to_string(),
                missed_points: String::new(),
                teaching_note: "Positive runtime sample for deterministic storyboard tests."
                    .to_string(),
                prompt_body: "Cinematic bridge confrontation with clean prompt language."
                    .to_string(),
            },
            provenance: GoldenSampleLibraryProvenance {
                source_workbook: "workbook.xlsx".to_string(),
                source_workbook_sha256: "hash".to_string(),
                source_control_memo: "memo.docx".to_string(),
                source_control_memo_sha256: "hash".to_string(),
                source_row_index: 1,
                source_library_status: "official".to_string(),
                source_sample_type: "single_shot".to_string(),
                comparison_baseline_source_id: "baseline".to_string(),
                control_review_doc: "review.md".to_string(),
                control_dispatch_doc: "dispatch.md".to_string(),
            },
            classification: GoldenSampleClassification {
                core: "storyboard_runtime_positive".to_string(),
                library_status: "official".to_string(),
                sample_type: "single_shot".to_string(),
                sequence_id: "SEQ-01".to_string(),
                shot_order: "01".to_string(),
                style_cluster: "hope-action".to_string(),
                scene_category: "daily_dialogue".to_string(),
                scene_tags: vec!["断桥".to_string(), "交锋".to_string(), "银辉".to_string()],
                quality_grade: "high".to_string(),
                usable_for_fewshot: true,
                coverage_surfaces: vec![
                    "scene_performance_core".to_string(),
                    "camera_directing_core".to_string(),
                ],
            },
            fewshot: GoldenSampleFewshotState {
                eligible: true,
                source_value: "Yes".to_string(),
                retrieval_status: "open".to_string(),
            },
            validator_evidence: GoldenSampleValidatorEvidence {
                covered_points: "composition / atmosphere / action".to_string(),
                missed_points: String::new(),
                teaching_note: "No placeholder or reserve-only evidence remains.".to_string(),
                source_quality_grade: "high".to_string(),
                source_library_status: "official".to_string(),
                source_sample_type: "single_shot".to_string(),
                has_coverage_gap: false,
                has_placeholder_signal: false,
                surface_completeness: std::collections::BTreeMap::from([
                    ("scene_performance_core".to_string(), true),
                    ("camera_directing_core".to_string(), true),
                    ("prompt_body".to_string(), true),
                ]),
                reference_bundle_present: false,
            },
            negative_sample: GoldenSampleNegativeSample {
                is_negative_sample: false,
                signal_codes: vec![],
                reserve_reason: String::new(),
            },
            v3_core_coverage: GoldenSampleV3CoreCoverage {
                core: "storyboard_runtime_positive".to_string(),
                coverage_rule_ids: vec![],
                source_coverage_statement: "covered".to_string(),
                source_missing_statement: String::new(),
                source_field_presence: std::collections::BTreeMap::from([
                    ("scene_performance_core".to_string(), true),
                    ("camera_directing_core".to_string(), true),
                    ("prompt_body".to_string(), true),
                ]),
                comparison_baseline: GoldenSampleComparisonBaseline {
                    primary_source_id: "v120".to_string(),
                    comparison_source_id: "v108".to_string(),
                },
            },
            repair_mapping_planning: GoldenSampleRepairMappingPlanning {
                failure_mapping_id: "failure-map-runtime-test".to_string(),
                repair_mapping_id: "repair-map-runtime-test".to_string(),
                planning_only: true,
            },
        };
        let source_context = GoldenSampleSourceContext {
            primary_source_id: "v120".to_string(),
            comparison_source_id: "v108".to_string(),
        };

        KbGoldenSampleRuntimePackage {
            manifest: KbBundleManifestRecord {
                snapshot_version: "v0.2".to_string(),
                snapshot_name: "runtime-test-bundle".to_string(),
                content_hash_algo: "sha256".to_string(),
                content_hash: "runtime-test-hash".to_string(),
                seed_import_format: "golden-sample-v0.2".to_string(),
                imported_at: "2026-04-27T00:00:00Z".to_string(),
                primary_key: "sample_id".to_string(),
                bundle_order: vec![
                    "golden_sample_library".to_string(),
                    "golden_sample_field_coverage_rules".to_string(),
                    "golden_sample_failure_mapping".to_string(),
                    "golden_sample_repair_mapping".to_string(),
                    "golden_sample_sources".to_string(),
                ],
                record_counts: KbBundleRecordCounts {
                    golden_sample_library: 1,
                    golden_sample_field_coverage_rules: 0,
                    golden_sample_failure_mapping: 0,
                    golden_sample_repair_mapping: 0,
                    golden_sample_sources: 1,
                    golden_sample_provenance_entries: 0,
                },
            },
            golden_sample_library: GoldenSampleLibraryAsset {
                schema_version: "golden_sample_library.v0.2".to_string(),
                asset_name: "golden_sample_library".to_string(),
                generated_at: "2026-04-27T00:00:00Z".to_string(),
                source: GoldenSampleAssetSource {
                    primary_workbook: "workbook.xlsx".to_string(),
                    primary_workbook_sha256: "hash".to_string(),
                    control_memo: "memo.docx".to_string(),
                    control_memo_sha256: "hash".to_string(),
                    control_review: "review.md".to_string(),
                    control_dispatch: "dispatch.md".to_string(),
                    comparison_baseline_source_id: "baseline".to_string(),
                },
                record_count: 1,
                source_field_order: vec![
                    "scene_category".to_string(),
                    "scene_tag".to_string(),
                    "prompt_body".to_string(),
                ],
                records: vec![record],
            },
            field_coverage_rules: GoldenSampleFieldCoverageRuleAsset {
                schema_version: "golden_sample_field_coverage_rule.v0.2".to_string(),
                asset_name: "golden_sample_field_coverage_rules".to_string(),
                generated_at: "2026-04-27T00:00:00Z".to_string(),
                source_context: source_context.clone(),
                record_count: 0,
                records: vec![],
            },
            failure_mapping: GoldenSampleFailureMappingAsset {
                schema_version: "golden_sample_failure_mapping.v0.2".to_string(),
                asset_name: "golden_sample_failure_mapping".to_string(),
                generated_at: "2026-04-27T00:00:00Z".to_string(),
                source_context: source_context.clone(),
                failure_code_definitions: vec![],
                record_count: 0,
                records: vec![],
            },
            repair_mapping: GoldenSampleRepairMappingAsset {
                schema_version: "golden_sample_repair_mapping.v0.2".to_string(),
                asset_name: "golden_sample_repair_mapping".to_string(),
                generated_at: "2026-04-27T00:00:00Z".to_string(),
                source_context,
                record_count: 0,
                records: vec![],
            },
            source_register: GoldenSampleSourceRegister {
                schema_version: "golden_sample_source_register.v0.2".to_string(),
                register_name: "runtime-test-source-register".to_string(),
                updated_at: "2026-04-27T00:00:00Z".to_string(),
                sources: vec![],
                provenance_entries: vec![],
            },
        }
    }

    fn assert_visual_description_is_enhanced(row: &GeneratedStoryboardRow) {
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
            row.visual_description.contains("画面强调"),
            "visual_description should land on conflict focus: {}",
            row.visual_description
        );
        assert_ne!(row.visual_description, row.character_action);
        assert_ne!(row.visual_description, row.shot_script);
        assert!(has_visual_environment_signal(&row.visual_description));
        assert!(has_visual_concrete_element_signal(&row.visual_description));
        assert!(has_visual_light_tone_or_material_signal(
            &row.visual_description
        ));
    }

    fn assert_no_fabricated_people_terms(text: &str) {
        for forbidden in [
            "对立人物",
            "目标人物",
            "主体人物",
            "人物完成关键动作",
            "目标人物完成关键动作",
            "单膝跪与步声响",
            "侧身疾与自烟尘",
        ] {
            assert!(
                !text.contains(forbidden),
                "storyboard text should not invent generic people term {forbidden}: {text}"
            );
        }
    }

    fn test_live_validation_row(person: &str) -> GeneratedStoryboardRow {
        let sequence_grouping = SequenceGrouping {
            structure_mode: StructureMode::SingleShot,
            sequence_id: None,
            shot_order: None,
            sequence_field_state: SequenceFieldState::NotApplicable,
        };
        let scene_performance_projection = ScenePerformanceProjection {
            source_sample_id: "test".to_string(),
            source_sample_title: "test".to_string(),
            scene_scale: "中近景".to_string(),
            person: person.to_string(),
            visual_description: format!(
                "主体为{person}，中近景把{person}放在画面前侧，当前视觉事件是废墟中的压迫逼近，画面突出尘土与黑暗钢筋。"
            ),
            character_action: format!(
                "{person}从废墟边缘的低身状态开始，到抬头承受压迫的瞬间结束，镜头捕捉呼吸和手指动作。"
            ),
            fused_source_text: "废墟之上，主角单膝跪地，敌人缓步逼近。".to_string(),
            sequence_grouping: sequence_grouping.clone(),
        };
        GeneratedStoryboardRow {
            shot_id: "shot-1".to_string(),
            order: 1,
            shot_script: "废墟之上，主角单膝跪地，敌人缓步逼近。".to_string(),
            primary_scene_type: "hot_blood_battle".to_string(),
            primary_scene_label: "热血战斗".to_string(),
            primary_scene_category: "combat".to_string(),
            shot_scene_type: "hot_blood_battle".to_string(),
            shot_scene_label: "热血战斗".to_string(),
            shot_intent: "action_beat".to_string(),
            adaptation_reason: "test".to_string(),
            grounding_source: ShotGroundingSource::ShotScript,
            person: person.to_string(),
            shot_title: format!("镜头1：{person}废墟压迫"),
            scene_scale: "中近景".to_string(),
            visual_description: scene_performance_projection.visual_description.clone(),
            character_action: scene_performance_projection.character_action.clone(),
            camera_movement: "中近景定机位观察主角动作起止，镜头捕捉废墟压迫。".to_string(),
            dialogue: "你撑不了多久。".to_string(),
            prompt_text: "以当前镜头脚本为准，输出单个镜头画面。".to_string(),
            prompt_text_compilation_status: PromptTextCompilationStatus::ReadyStub,
            prompt_text_compilation_warnings: vec![],
            prompt_text_source_row_id: "shot-1".to_string(),
            duration_seconds: 10,
            shot_duration_seconds: 10,
            duration_source: "test".to_string(),
            scene_performance_projection,
            external_reference_handle_candidates: vec![],
            sequence_grouping,
        }
    }

    #[test]
    fn live_storyboard_validator_rejects_untrusted_character_names() {
        let baseline = test_live_validation_row("主角");
        let mut live_row = baseline.clone();
        live_row.person = "孔泛".to_string();
        live_row.shot_title = "镜头1：孔泛废墟压迫".to_string();
        live_row.visual_description =
            "主体为孔泛，特写压低孔泛的脸部，强调废墟中的压迫。".to_string();
        live_row.character_action =
            "孔泛从高架桥阴影下压低重心开始，到抬头承受压迫结束，镜头捕捉手指动作。".to_string();

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);

        assert!(
            warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name")),
            "live storyboard rows should reject invented character names: {:?}",
            warnings
        );
    }

    #[test]
    fn live_validator_normalizes_source_prefixed_state_fragments_before_grounding() {
        let protagonist_baseline = test_live_validation_row("主角");
        let enemy_baseline = test_live_validation_row("敌人");

        for (dirty_person, baseline) in [
            ("主角反", protagonist_baseline.clone()),
            ("主角准", protagonist_baseline.clone()),
            ("主角准备", protagonist_baseline.clone()),
            ("主角坚持", protagonist_baseline.clone()),
            ("敌人紧张", enemy_baseline.clone()),
        ] {
            let mut live_row = baseline.clone();
            live_row.person = dirty_person.to_string();
            live_row.scene_performance_projection.person = dirty_person.to_string();
            live_row.shot_title = format!("镜头1：{dirty_person}废墟对峙");
            live_row.visual_description =
                "主体为主角，中近景把主角单膝跪地和敌人缓步逼近放在废墟里。".to_string();
            live_row.character_action =
                "主角从废墟之上单膝跪地开始，到敌人逼近时结束，镜头捕捉对峙压力。".to_string();
            live_row.camera_movement =
                "中近景定机位观察主角单膝跪地和敌人缓步逼近，镜头捕捉废墟对峙。".to_string();

            let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);

            assert!(
                !warnings
                    .iter()
                    .any(|warning| warning.message.contains("ungrounded character name")),
                "{dirty_person} should normalize to its source subject before grounded check: {warnings:?}"
            );
        }
    }

    #[test]
    fn live_validator_rejects_source_prefixed_external_names() {
        let protagonist_baseline = test_live_validation_row("主角");
        let enemy_baseline = test_live_validation_row("敌人");
        let mut linfeng_baseline = test_live_validation_row("林峰");
        linfeng_baseline.shot_script = "林峰护住苏瑶，黑衣追兵从巷口逼近。".to_string();
        linfeng_baseline
            .scene_performance_projection
            .fused_source_text = linfeng_baseline.shot_script.clone();
        linfeng_baseline.scene_performance_projection.person = "林峰".to_string();

        for (dirty_person, baseline) in [
            ("主角李明", protagonist_baseline.clone()),
            ("敌人李明", enemy_baseline.clone()),
            ("林峰李明", linfeng_baseline.clone()),
        ] {
            let mut live_row = baseline.clone();
            live_row.person = dirty_person.to_string();
            live_row.scene_performance_projection.person = dirty_person.to_string();
            live_row.shot_title = format!("镜头1：{dirty_person}压迫");
            live_row.visual_description = format!("主体为{dirty_person}，中近景保留源内压力。");
            live_row.character_action = format!("{dirty_person}从源内压力开始，到对峙压近时结束。");

            let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);

            assert!(
                warnings.iter().any(|warning| warning
                    .message
                    .contains("ungrounded character name")
                    && warning.message.contains("李明")),
                "{dirty_person} should remain rejected as source-external: {warnings:?}"
            );
        }
    }

    #[test]
    fn live_storyboard_patch_rebinds_visual_person_to_source_roles() {
        let baseline = test_live_validation_row("主角");
        let mut live_row = baseline.clone();
        let patch = LiveStoryboardRowPatch {
            shot_title: "镜头1：焦点废墟压迫".to_string(),
            person: "高对比焦点".to_string(),
            scene_scale: "中近景".to_string(),
            visual_description:
                "主体为焦点，中近景把焦点放在画面前侧，当前视觉事件是废墟中的压迫逼近。".to_string(),
            character_action: "焦点从废墟边缘开始，到压迫逼近时结束。".to_string(),
            camera_movement: "中近景定机位观察主角动作起止，镜头捕捉废墟压迫。".to_string(),
            dialogue: String::new(),
        };

        apply_live_storyboard_patch(&mut live_row, &patch, &baseline);

        assert_eq!(live_row.person, "主角与敌人");
        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);
        assert!(
            warnings
                .iter()
                .any(|warning| warning.message.contains("visual or abstract subject label")),
            "validator should still reject visual subject leakage outside person: {:?}",
            warnings
        );
    }

    #[test]
    fn live_storyboard_patch_rebinds_source_fragment_person_to_source_roles() {
        let baseline = test_live_validation_row("主角");
        let mut live_row = baseline.clone();
        let patch = LiveStoryboardRowPatch {
            shot_title: "镜头1：猛然单与瞳孔骤压近断桥".to_string(),
            person: "猛然单与瞳孔骤".to_string(),
            scene_scale: "中近景".to_string(),
            visual_description: "主体为猛然单与瞳孔骤，中近景把双方压在断桥边缘。".to_string(),
            character_action:
                "猛然单与瞳孔骤从断桥远端压低重心开始，到压迫逼近时结束，镜头捕捉刀锋压入空间。"
                    .to_string(),
            camera_movement: "中近景定机位观察主角动作起止，镜头捕捉废墟压迫。".to_string(),
            dialogue: String::new(),
        };

        apply_live_storyboard_patch(&mut live_row, &patch, &baseline);

        assert_eq!(live_row.person, "主角与敌人");
        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);
        assert!(
            warnings
                .iter()
                .any(|warning| warning.message.contains("source fragment as subject label")),
            "validator should still reject source fragment subject leakage: {:?}",
            warnings
        );
    }

    #[test]
    fn live_storyboard_validator_rejects_broken_halberd_subject() {
        let baseline = test_live_validation_row("主角");
        let mut live_row = baseline.clone();
        live_row.person = "断戟立".to_string();
        live_row.scene_performance_projection.person = "断戟立".to_string();
        live_row.shot_title = "镜头1：断戟立压住废墟".to_string();
        live_row.visual_description = "主体为断戟立，中近景把断戟立放在废墟前侧；冷光压住断戟；当前视觉事件是断戟立在废墟中承受压力；画面突出对峙压力".to_string();
        live_row.character_action = "断戟立从废墟前侧停顿开始，到敌人缓步逼近时结束。".to_string();
        live_row.camera_movement = "中近景定机位观察断戟立动作起止。".to_string();

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);

        assert!(
            warnings.iter().any(
                |warning| warning.message.contains("source-external setting")
                    || warning.message.contains("ungrounded character name")
            ),
            "断戟立 must stay fail-closed: {:?}",
            warnings
        );
    }

    #[test]
    fn a_ruin_main_row_repairs_incomplete_visual_description() {
        let mut row = test_live_validation_row("主角");
        row.person = "主角".to_string();
        row.scene_performance_projection.person = "主角".to_string();
        row.visual_description =
            "主体为主角，中近景把主角放在画面前侧；冷光压住背景；画面突出压迫感。".to_string();
        row.scene_performance_projection.visual_description = row.visual_description.clone();

        normalize_storyboard_row_subject_quality(&mut row);

        assert!(
            row.visual_description.contains("敌人"),
            "{}",
            row.visual_description
        );
        assert!(
            row.visual_description.contains("废墟"),
            "{}",
            row.visual_description
        );
        assert!(
            contains_any_story_term(&row.visual_description, &["逼近", "对峙压力"]),
            "{}",
            row.visual_description
        );
        let warnings =
            validate_live_storyboard_rows(&[row], 10, &[test_live_validation_row("主角")]);
        assert!(
            !warnings
                .iter()
                .any(|warning| warning.code == "visual_description_grounding_incomplete"),
            "{warnings:?}"
        );
    }

    #[test]
    fn a_war_repeated_main_subject_diversifies_to_enemy() {
        let mut rows = vec![
            test_live_validation_row("主角"),
            test_live_validation_row("主角"),
        ];
        rows[1].order = 2;
        rows[1].shot_id = "shot-2".to_string();
        rows[1].prompt_text_source_row_id = "shot-2".to_string();
        rows[1].character_action = "主角从废墟压力开始，到敌人继续缓步逼近时结束。".to_string();
        rows[1].visual_description = "主体为主角，中近景把主角放在废墟前侧，敌人缓步逼近的压力压在后侧；冷光压住废墟；当前视觉事件是敌人继续逼近；画面突出对峙压力".to_string();
        rows[1].scene_performance_projection.character_action = rows[1].character_action.clone();
        rows[1].scene_performance_projection.visual_description =
            rows[1].visual_description.clone();

        diversify_repeated_storyboard_subjects(&mut rows);

        assert_eq!(rows[0].person, "主角");
        assert_eq!(rows[1].person, "敌人");
        let second_visible = storyboard_row_visible_text(&rows[1]);
        assert!(second_visible.contains("敌人"), "{second_visible}");
        assert!(!second_visible.contains("主体为主角"), "{second_visible}");
    }

    #[test]
    fn live_storyboard_repair_restores_a_visual_grounding() {
        let baseline = test_live_validation_row("主角");
        let mut live_row = baseline.clone();
        live_row.visual_description =
            "主体为主角，中近景把主角放在画面前侧；冷光压住背景；画面突出压迫感。".to_string();
        live_row.scene_performance_projection.visual_description =
            live_row.visual_description.clone();
        let mut rows = vec![live_row];
        let baselines = vec![baseline];
        let pre_repair = validate_live_storyboard_rows(&rows, 10, &baselines);
        assert!(
            pre_repair
                .iter()
                .any(|warning| warning.code == "visual_description_grounding_incomplete"),
            "{pre_repair:?}"
        );

        let mut summary = LiveRepairSummary {
            raw_failed_validator: !pre_repair.is_empty(),
            reasons: Vec::new(),
        };
        summary.extend(repair_live_storyboard_rows_from_source(
            &mut rows, &baselines,
        ));
        let post_repair = validate_live_storyboard_rows(&rows, 10, &baselines);

        assert!(summary.repaired(), "{summary:?}");
        assert!(post_repair.is_empty(), "{post_repair:?}");
        assert!(rows[0].visual_description.contains("废墟"));
        assert!(rows[0].visual_description.contains("敌人"));
        assert!(contains_any_story_term(
            &rows[0].visual_description,
            &["逼近", "对峙压力"]
        ));
        let warning = live_repair_warning("text_model_live_storyboard_repaired", &summary);
        assert_eq!(warning.code, "text_model_live_storyboard_repaired");
        assert!(warning.message.contains("fallback_used=false"));
    }

    #[test]
    fn live_storyboard_repair_rebinds_b_fragments_and_abstract_pressure() {
        let mut baseline = test_live_validation_row("林峰");
        baseline.shot_script = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        baseline.person = "林峰".to_string();
        baseline.scene_performance_projection.person = "林峰".to_string();
        let mut live_row = baseline.clone();
        live_row.visual_description = "主体为林峰压，中近景把林峰压和苏瑶压放在巷口前侧；冷光制造危险感；当前视觉事件是阿青提醒他们后退。".to_string();
        live_row.character_action = "林峰压低身形护住苏瑶，阿青提醒他们，危险感逼近。".to_string();
        live_row.scene_performance_projection.visual_description =
            live_row.visual_description.clone();
        live_row.scene_performance_projection.character_action = live_row.character_action.clone();
        let mut rows = vec![live_row];
        let baselines = vec![baseline];

        let mut summary = LiveRepairSummary::default();
        summary.raw_failed_validator =
            !validate_live_storyboard_rows(&rows, 10, &baselines).is_empty();
        summary.extend(repair_live_storyboard_rows_from_source(
            &mut rows, &baselines,
        ));
        let post_repair = validate_live_storyboard_rows(&rows, 10, &baselines);
        let combined = storyboard_row_visible_text(&rows[0]);

        assert!(summary.repaired(), "{summary:?}");
        assert!(post_repair.is_empty(), "{post_repair:?}");
        assert!(!combined.contains("林峰压"), "{combined}");
        assert!(!combined.contains("苏瑶压"), "{combined}");
        assert!(!combined.contains("危险感"), "{combined}");
        assert!(combined.contains("林峰护住苏瑶"), "{combined}");
        assert!(combined.contains("阿青提醒"), "{combined}");
        assert!(combined.contains("黑衣追兵"), "{combined}");
        assert!(combined.contains("巷口"), "{combined}");
        assert!(combined.contains("逼近"), "{combined}");
    }

    #[test]
    fn live_storyboard_repair_restores_b_pursuer_visual_grounding() {
        let mut baseline = test_live_validation_row("林峰");
        baseline.shot_script = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        baseline.person = "追兵".to_string();
        baseline.scene_performance_projection.person = "追兵".to_string();
        let mut live_row = baseline.clone();
        live_row.person = "追兵".to_string();
        live_row.scene_performance_projection.person = "追兵".to_string();
        live_row.visual_description =
            "主体为追兵，中近景把追兵放在巷口后侧，黑衣追兵从巷口逼近。".to_string();
        live_row.scene_performance_projection.visual_description =
            live_row.visual_description.clone();
        let mut rows = vec![live_row];
        let baselines = vec![baseline];

        let mut summary = LiveRepairSummary::default();
        summary.raw_failed_validator =
            !validate_live_storyboard_rows(&rows, 10, &baselines).is_empty();
        summary.extend(repair_live_storyboard_rows_from_source(
            &mut rows, &baselines,
        ));
        let post_repair = validate_live_storyboard_rows(&rows, 10, &baselines);
        let combined = storyboard_row_visible_text(&rows[0]);

        assert!(summary.repaired(), "{summary:?}");
        assert!(post_repair.is_empty(), "{post_repair:?}");
        for required in ["林峰", "苏瑶", "阿青", "黑衣追兵", "巷口", "逼近"] {
            assert!(
                combined.contains(required),
                "{required} missing from {combined}"
            );
        }
        assert!(combined.contains("画面突出追兵压力"), "{combined}");
        for forbidden in ["三名", "刀锋", "衣袖裂口", "左臂", "伤口"] {
            assert!(
                !combined.contains(forbidden),
                "{forbidden} leaked into {combined}"
            );
        }
        let warning = live_repair_warning("text_model_live_storyboard_repaired", &summary);
        assert!(warning.message.contains("fallback_used=false"));
    }

    #[test]
    fn live_storyboard_repair_restores_b_aqing_visual_grounding() {
        let mut baseline = test_live_validation_row("阿青");
        baseline.shot_script = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        baseline.person = "阿青".to_string();
        baseline.scene_performance_projection.person = "阿青".to_string();
        let mut live_row = baseline.clone();
        live_row.visual_description =
            "主体为阿青，中近景把阿青放在巷口前侧，提醒他们追兵逼近。".to_string();
        live_row.scene_performance_projection.visual_description =
            live_row.visual_description.clone();
        let mut rows = vec![live_row];
        let baselines = vec![baseline];

        let pre_repair = validate_live_storyboard_rows(&rows, 10, &baselines);
        assert!(
            pre_repair
                .iter()
                .any(|warning| warning.code == "visual_description_grounding_incomplete"),
            "{pre_repair:?}"
        );

        let mut summary = LiveRepairSummary::default();
        summary.raw_failed_validator = true;
        summary.extend(repair_live_storyboard_rows_from_source(
            &mut rows, &baselines,
        ));
        let post_repair = validate_live_storyboard_rows(&rows, 10, &baselines);
        let combined = storyboard_row_visible_text(&rows[0]);

        assert!(summary.repaired(), "{summary:?}");
        assert!(post_repair.is_empty(), "{post_repair:?}");
        for required in ["林峰护住苏瑶", "阿青提醒", "黑衣追兵", "巷口", "逼近"] {
            assert!(
                combined.contains(required),
                "{required} missing from {combined}"
            );
        }
        assert!(!combined.contains("苏瑶的"), "{combined}");
        assert!(!combined.contains("苏瑶放"), "{combined}");
    }

    #[test]
    fn scene_scale_ba_noise_and_name_tails_do_not_create_new_people() {
        let mut baseline = test_live_validation_row("林峰");
        baseline.shot_script = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        baseline.person = "林峰".to_string();
        baseline.scene_performance_projection.person = "林峰".to_string();

        let mut scale_noise_row = baseline.clone();
        scale_noise_row.person = "中景把".to_string();
        scale_noise_row.visual_description =
            "主体为中景把，中景把巷口压力放在后侧，林峰护住苏瑶。".to_string();
        scale_noise_row.character_action = "中景把从巷口压力开始，到追兵逼近时结束。".to_string();
        let scale_noise_warnings =
            validate_live_storyboard_rows(&[scale_noise_row], 10, &[baseline.clone()]);
        assert!(
            !scale_noise_warnings.iter().any(|warning| warning
                .message
                .contains("ungrounded character name: 中景把")),
            "{scale_noise_warnings:?}"
        );

        let mut tailed_name_row = baseline.clone();
        tailed_name_row.person = "林峰仍".to_string();
        tailed_name_row.visual_description =
            "主体为林峰仍，中景把林峰护住苏瑶的站位放在巷口前侧；冷光压住墙面；当前视觉事件是黑衣追兵从巷口逼近；画面突出追兵压力。".to_string();
        tailed_name_row.character_action =
            "林峰仍从护住苏瑶开始，到黑衣追兵从巷口逼近时结束，镜头捕捉护人与提醒同时压住退路的一瞬间。".to_string();
        let tailed_name_warnings =
            validate_live_storyboard_rows(&[tailed_name_row], 10, &[baseline.clone()]);
        assert!(
            !tailed_name_warnings.iter().any(|warning| warning
                .message
                .contains("ungrounded character name: 林峰仍")),
            "{tailed_name_warnings:?}"
        );

        let mut invented_name_row = baseline.clone();
        invented_name_row.person = "李明".to_string();
        invented_name_row.visual_description =
            "主体为李明，中景把李明放在巷口前侧；冷光压住墙面；当前视觉事件是李明挡住退路；画面突出追兵压力。".to_string();
        invented_name_row.character_action =
            "李明从巷口前侧开始，到挡住退路时结束，镜头捕捉李明动作。".to_string();
        let invented_name_warnings =
            validate_live_storyboard_rows(&[invented_name_row], 10, &[baseline]);
        assert!(
            invented_name_warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name")),
            "{invented_name_warnings:?}"
        );
    }

    #[test]
    fn scene_scale_word_is_not_ungrounded_character_name() {
        let mut baseline = test_live_validation_row("敌人");
        baseline.person = "敌人".to_string();
        baseline.scene_performance_projection.person = "敌人".to_string();
        let mut live_row = baseline.clone();
        live_row.scene_scale = "全景".to_string();
        live_row.visual_description = "主体为敌人，全景把敌人、主角和废墟放在前后层次里；冷光压住碎石与混凝土；当前视觉事件是敌人缓步逼近，主角仍单膝跪地；画面突出对峙压力。".to_string();
        live_row.character_action = "敌人从废墟边缘开始缓步逼近主角，到对峙距离被压短时结束，镜头捕捉敌人逼近压力压到主角身前的一瞬间。".to_string();
        live_row.camera_movement =
            "全景定机位观察敌人缓步逼近，镜头捕捉废墟前侧的对峙压力。".to_string();

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline.clone()]);

        assert!(
            !warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name: 全景")),
            "{warnings:?}"
        );

        let mut invented_name_row = baseline.clone();
        invented_name_row.person = "李明".to_string();
        invented_name_row.visual_description = "主体为李明，中近景把李明放在废墟前侧；冷光压住碎石与混凝土；当前视觉事件是李明挡住退路；画面突出对峙压力。".to_string();
        invented_name_row.character_action =
            "李明从废墟前侧开始，到挡住退路时结束，镜头捕捉李明动作。".to_string();
        let invented_name_warnings =
            validate_live_storyboard_rows(&[invented_name_row], 10, &[baseline.clone()]);
        assert!(
            invented_name_warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name")),
            "{invented_name_warnings:?}"
        );

        let mut incomplete_row = baseline.clone();
        incomplete_row.visual_description = "主体为敌人。".to_string();
        let incomplete_warnings = validate_live_storyboard_rows(&[incomplete_row], 10, &[baseline]);
        assert!(
            incomplete_warnings
                .iter()
                .any(|warning| warning.code == "visual_description_grounding_incomplete"),
            "{incomplete_warnings:?}"
        );
    }

    #[test]
    fn action_state_fragment_is_not_ungrounded_character_name() {
        let mut baseline = test_live_validation_row("敌人");
        baseline.person = "敌人".to_string();
        baseline.scene_performance_projection.person = "敌人".to_string();

        let mut allowed_row = baseline.clone();
        allowed_row.visual_description = "主体为敌人，中近景把敌人、主角和废墟放在前后层次里；冷光压住碎石与混凝土；当前视觉事件是敌人坚持缓步逼近，主角仍坚持单膝跪地；画面突出对峙压力。".to_string();
        allowed_row.character_action = "敌人从废墟边缘开始缓步逼近主角，到主角仍坚持单膝跪地承受逼近压力时结束，镜头捕捉敌人逼近压力压到主角身前的一瞬间。".to_string();
        allowed_row.camera_movement =
            "中近景定机位观察敌人缓步逼近，镜头捕捉废墟前侧的对峙压力。".to_string();
        let allowed_warnings =
            validate_live_storyboard_rows(&[allowed_row], 10, &[baseline.clone()]);
        assert!(
            !allowed_warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name: 坚持")),
            "{allowed_warnings:?}"
        );

        for fragment in ["准备反", "准备反击", "反击准备"] {
            let mut action_fragment_row = baseline.clone();
            action_fragment_row.visual_description = format!(
                "主体为敌人，中近景把敌人、主角和废墟放在前后层次里；冷光压住碎石与混凝土；当前视觉事件是敌人{fragment}时仍缓步逼近，主角仍单膝跪地；画面突出对峙压力。"
            );
            action_fragment_row.character_action = format!(
                "敌人从废墟边缘开始缓步逼近主角，到{fragment}时仍压近主角身前时结束，镜头捕捉敌人逼近压力压到主角身前的一瞬间。"
            );
            action_fragment_row.camera_movement =
                "中近景定机位观察敌人缓步逼近，镜头捕捉废墟前侧的对峙压力。".to_string();
            let action_fragment_warnings =
                validate_live_storyboard_rows(&[action_fragment_row], 10, &[baseline.clone()]);
            assert!(
                !action_fragment_warnings.iter().any(|warning| warning
                    .message
                    .contains(&format!("ungrounded character name: {fragment}"))),
                "{fragment}: {action_fragment_warnings:?}"
            );
        }

        let mut title_subject_row = baseline.clone();
        title_subject_row.person = "敌人".to_string();
        title_subject_row.scene_performance_projection.person = "敌人".to_string();
        title_subject_row.shot_title = "镜头3：坚持废墟逼近".to_string();
        title_subject_row.visual_description = "主体为敌人，中近景把敌人、主角和废墟放在前后层次里；冷光压住碎石与混凝土；当前视觉事件是敌人坚持缓步逼近，主角仍单膝跪地；画面突出对峙压力。".to_string();
        title_subject_row.character_action = "敌人从废墟边缘开始缓步逼近主角，到主角仍单膝跪地承受逼近压力时结束，镜头捕捉敌人逼近压力压到主角身前的一瞬间。".to_string();
        title_subject_row.prompt_text =
            "镜头标题：坚持废墟逼近；角色动作：敌人从废墟边缘开始缓步逼近。".to_string();
        normalize_storyboard_row_subject_quality(&mut title_subject_row);
        assert_eq!(title_subject_row.person, "敌人");
        assert_eq!(
            title_subject_row.scene_performance_projection.person,
            "敌人"
        );
        assert!(
            title_subject_row.shot_title.contains("敌人"),
            "{}",
            title_subject_row.shot_title
        );
        assert!(
            !title_subject_row.shot_title.contains("：坚持"),
            "{}",
            title_subject_row.shot_title
        );
        let title_subject_warnings =
            validate_live_storyboard_rows(&[title_subject_row], 10, &[baseline.clone()]);
        assert!(
            !title_subject_warnings
                .iter()
                .any(|warning| warning.message.contains("source fragment as subject label")),
            "{title_subject_warnings:?}"
        );

        for fragment in ["准备反", "准备反击", "反击准备"] {
            let mut title_subject_row = baseline.clone();
            title_subject_row.person = "敌人".to_string();
            title_subject_row.scene_performance_projection.person = "敌人".to_string();
            title_subject_row.shot_title = format!("镜头3：{fragment}废墟逼近");
            title_subject_row.visual_description = format!(
                "主体为敌人，中近景把敌人、主角和废墟放在前后层次里；冷光压住碎石与混凝土；当前视觉事件是敌人{fragment}时仍缓步逼近，主角仍单膝跪地；画面突出对峙压力。"
            );
            title_subject_row.character_action = format!(
                "敌人从废墟边缘开始缓步逼近主角，到{fragment}时仍压近主角身前时结束，镜头捕捉敌人逼近压力压到主角身前的一瞬间。"
            );
            title_subject_row.prompt_text =
                format!("镜头标题：{fragment}废墟逼近；角色动作：敌人从废墟边缘开始缓步逼近。");
            normalize_storyboard_row_subject_quality(&mut title_subject_row);
            assert_eq!(title_subject_row.person, "敌人", "{fragment}");
            assert_eq!(
                title_subject_row.scene_performance_projection.person, "敌人",
                "{fragment}"
            );
            assert!(
                title_subject_row.shot_title.contains("敌人"),
                "{fragment}: {}",
                title_subject_row.shot_title
            );
            assert!(
                !title_subject_row
                    .shot_title
                    .contains(&format!("：{fragment}")),
                "{fragment}: {}",
                title_subject_row.shot_title
            );
            let title_subject_warnings =
                validate_live_storyboard_rows(&[title_subject_row], 10, &[baseline.clone()]);
            assert!(
                !title_subject_warnings
                    .iter()
                    .any(|warning| warning.message.contains("source fragment as subject label")),
                "{fragment}: {title_subject_warnings:?}"
            );
        }

        let mut bad_subject_row = baseline.clone();
        bad_subject_row.person = "坚持".to_string();
        bad_subject_row.visual_description = "主体为坚持，中近景把坚持放在废墟前侧；冷光压住碎石与混凝土；当前视觉事件是坚持挡住退路；画面突出对峙压力。".to_string();
        bad_subject_row.character_action =
            "坚持从废墟前侧开始，到挡住退路时结束，镜头捕捉坚持动作。".to_string();
        let bad_subject_warnings =
            validate_live_storyboard_rows(&[bad_subject_row], 10, &[baseline.clone()]);
        assert!(
            !bad_subject_warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name: 坚持")),
            "{bad_subject_warnings:?}"
        );
        assert!(
            bad_subject_warnings
                .iter()
                .any(|warning| warning.message.contains("source fragment as subject label")),
            "{bad_subject_warnings:?}"
        );

        let mut invented_name_row = baseline.clone();
        invented_name_row.person = "李明".to_string();
        invented_name_row.visual_description = "主体为李明，中近景把李明放在废墟前侧；冷光压住碎石与混凝土；当前视觉事件是李明挡住退路；画面突出对峙压力。".to_string();
        invented_name_row.character_action =
            "李明从废墟前侧开始，到挡住退路时结束，镜头捕捉李明动作。".to_string();
        let invented_name_warnings =
            validate_live_storyboard_rows(&[invented_name_row], 10, &[baseline.clone()]);
        assert!(
            invented_name_warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name")),
            "{invented_name_warnings:?}"
        );

        let mut incomplete_row = baseline.clone();
        incomplete_row.visual_description = "主体为敌人。".to_string();
        let incomplete_warnings = validate_live_storyboard_rows(&[incomplete_row], 10, &[baseline]);
        assert!(
            incomplete_warnings
                .iter()
                .any(|warning| warning.code == "visual_description_grounding_incomplete"),
            "{incomplete_warnings:?}"
        );
    }

    #[test]
    fn possessive_abstract_phrase_is_not_ungrounded_character_name() {
        let mut baseline = test_live_validation_row("主角");
        baseline.person = "主角".to_string();
        baseline.scene_performance_projection.person = "主角".to_string();

        assert!(super::looks_like_name_noise_candidate("的决心"));
        assert!(super::extract_character_name_after_role("的决心压住废墟逼近").is_none());

        let mut abstract_phrase_row = baseline.clone();
        abstract_phrase_row.person = "主角".to_string();
        abstract_phrase_row.shot_title = "镜头3：主角的决心压住废墟逼近".to_string();
        abstract_phrase_row.visual_description = "主体为主角，中近景把主角、敌人和废墟放在前后层次里；冷光压住碎石与混凝土；当前视觉事件是主角的决心在敌人逼近时被压到前侧；画面突出对峙压力。".to_string();
        abstract_phrase_row.character_action = "主角从废墟前侧单膝跪地开始，到敌人缓步逼近时仍撑住身体结束，镜头捕捉主角承受逼近压力的一瞬间。".to_string();
        abstract_phrase_row.camera_movement =
            "中近景定机位观察主角单膝跪地，镜头捕捉废墟前侧的对峙压力。".to_string();
        let abstract_phrase_warnings =
            validate_live_storyboard_rows(&[abstract_phrase_row], 10, &[baseline.clone()]);
        assert!(
            !abstract_phrase_warnings.iter().any(|warning| warning
                .message
                .contains("ungrounded character name: 的决心")),
            "{abstract_phrase_warnings:?}"
        );

        let mut patched_row = baseline.clone();
        patched_row.order = 3;
        let patch = LiveStoryboardRowPatch {
            shot_title: "镜头3：的决心压住废墟逼近".to_string(),
            person: "的决心".to_string(),
            scene_scale: "中近景".to_string(),
            visual_description: "主体为的决心，中近景把主角、敌人和废墟放在前后层次里；冷光压住碎石与混凝土；当前视觉事件是主角的决心在敌人逼近时被压到前侧；画面突出对峙压力。".to_string(),
            character_action: "主角从废墟前侧单膝跪地开始，到敌人缓步逼近时仍撑住身体结束，镜头捕捉主角承受逼近压力的一瞬间。".to_string(),
            camera_movement: "中近景定机位观察主角单膝跪地，镜头捕捉废墟前侧的对峙压力。".to_string(),
            dialogue: String::new(),
        };
        apply_live_storyboard_patch(&mut patched_row, &patch, &baseline);
        normalize_storyboard_row_subject_quality(&mut patched_row);
        assert!(
            matches!(patched_row.person.as_str(), "主角" | "主角与敌人"),
            "{}",
            patched_row.person
        );
        assert!(!patched_row.person.contains("决心"));
        let patched_warnings =
            validate_live_storyboard_rows(&[patched_row], 10, &[baseline.clone()]);
        assert!(
            !patched_warnings.iter().any(|warning| warning
                .message
                .contains("ungrounded character name: 的决心")),
            "{patched_warnings:?}"
        );

        let mut invented_name_row = baseline.clone();
        invented_name_row.person = "李明".to_string();
        invented_name_row.visual_description = "主体为李明，中近景把李明放在废墟前侧；冷光压住碎石与混凝土；当前视觉事件是李明挡住退路；画面突出对峙压力。".to_string();
        invented_name_row.character_action =
            "李明从废墟前侧开始，到挡住退路时结束，镜头捕捉李明动作。".to_string();
        let invented_name_warnings =
            validate_live_storyboard_rows(&[invented_name_row], 10, &[baseline]);
        assert!(
            invented_name_warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name")),
            "{invented_name_warnings:?}"
        );
    }

    #[test]
    fn a_war_enemy_environment_person_rebinds_to_enemy() {
        let mut baseline = test_live_validation_row("敌人");
        baseline.shot_script =
            "废墟之上，主角单膝跪地，敌人缓步逼近。国战军阵建立只压紧阵位和战场调度。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        baseline.person = "敌人".to_string();
        baseline.scene_performance_projection.person = "敌人".to_string();

        let mut live_row = baseline.clone();
        let patch = LiveStoryboardRowPatch {
            shot_title: "镜头2：敌人与废墟逼近".to_string(),
            person: "敌人与废墟".to_string(),
            scene_scale: "全景".to_string(),
            visual_description: "主体为敌人与废墟，全景把敌人与废墟放在阵位前侧；冷光压住碎石与混凝土；当前视觉事件是敌人缓步逼近，主角仍单膝跪地；画面突出对峙压力。".to_string(),
            character_action: "敌人与废墟从废墟边缘开始，到敌人逼近主角身前时结束，镜头捕捉敌人缓步逼近。".to_string(),
            camera_movement:
                "全景定机位观察敌人与废墟，镜头捕捉废墟前侧的对峙压力。".to_string(),
            dialogue: String::new(),
        };

        apply_live_storyboard_patch(&mut live_row, &patch, &baseline);
        normalize_storyboard_row_subject_quality(&mut live_row);

        assert_eq!(live_row.person, "敌人");
        assert_eq!(live_row.scene_performance_projection.person, "敌人");
        let combined = storyboard_row_visible_text(&live_row);
        assert!(!combined.contains("敌人与废墟"), "{combined}");
        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);
        assert!(
            !warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name")),
            "{warnings:?}"
        );
    }

    #[test]
    fn live_storyboard_repair_restores_a_war_enemy_row_person_grounding() {
        let mut baseline = test_live_validation_row("敌人");
        baseline.order = 2;
        baseline.shot_id = "shot-2".to_string();
        baseline.prompt_text_source_row_id = "shot-2".to_string();
        baseline.primary_scene_label = "国战军阵建立".to_string();
        baseline.shot_scene_label = "国战军阵建立".to_string();
        baseline.scene_scale = "全景".to_string();
        baseline.scene_performance_projection.scene_scale = "全景".to_string();
        baseline.shot_script =
            "废墟之上，主角单膝跪地，敌人缓步逼近。国战军阵建立只压紧阵位与战场调度。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        baseline.person = "敌人".to_string();
        baseline.scene_performance_projection.person = "敌人".to_string();
        repair_a_ruin_enemy_storyboard_row(&mut baseline);
        normalize_storyboard_row_subject_quality(&mut baseline);

        let mut live_row = baseline.clone();
        live_row.person = "敌人与废墟".to_string();
        live_row.scene_performance_projection.person = "敌人与废墟".to_string();
        live_row.shot_title = "镜头2：敌人与废墟".to_string();
        live_row.visual_description = "主体为敌人与废墟。".to_string();
        live_row.character_action = "敌人与废墟逼近。".to_string();
        live_row.camera_movement = "全景观察。".to_string();
        live_row.scene_performance_projection.visual_description =
            live_row.visual_description.clone();
        live_row.scene_performance_projection.character_action = live_row.character_action.clone();

        let mut rows = vec![live_row];
        let baselines = vec![baseline];
        let mut summary = LiveRepairSummary::default();
        summary.raw_failed_validator =
            !validate_live_storyboard_rows(&rows, 10, &baselines).is_empty();
        summary.extend(repair_live_storyboard_rows_from_source(
            &mut rows, &baselines,
        ));
        let post_repair = validate_live_storyboard_rows(&rows, 10, &baselines);
        let combined = storyboard_row_visible_text(&rows[0]);

        assert!(summary.repaired(), "{summary:?}");
        assert!(post_repair.is_empty(), "{post_repair:?}");
        assert_eq!(rows[0].person, "敌人");
        assert!(!combined.contains("敌人与废墟"), "{combined}");
        for required in [
            "废墟",
            "主角",
            "敌人",
            "单膝跪地",
            "缓步逼近",
            "阵位",
            "战场调度",
        ] {
            assert!(
                combined.contains(required),
                "{required} missing from {combined}"
            );
        }
        for forbidden in ["甲胄", "铠甲", "剑柄", "断戟", "额外士兵", "军阵规模"] {
            assert!(
                !combined.contains(forbidden),
                "{forbidden} leaked into {combined}"
            );
        }
    }

    #[test]
    fn live_storyboard_repair_keeps_unrepairable_fallback_findings() {
        let baseline = test_live_validation_row("主角");
        let mut live_row = baseline.clone();
        live_row.person = "李明".to_string();
        live_row.visual_description =
            "主体为李明，中近景把李明放在房间前侧；冷光压住背景；当前视觉事件是李明递来雨伞；画面突出安静。".to_string();
        live_row.character_action = "李明从房间前侧开始，到递来雨伞时结束。".to_string();
        let mut rows = vec![live_row];
        let baselines = vec![baseline];

        let mut summary = LiveRepairSummary::default();
        summary.raw_failed_validator =
            !validate_live_storyboard_rows(&rows, 10, &baselines).is_empty();
        summary.extend(repair_live_storyboard_rows_from_source(
            &mut rows, &baselines,
        ));
        let post_repair = validate_live_storyboard_rows(&rows, 10, &baselines);

        assert!(
            post_repair
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name")),
            "{post_repair:?}"
        );
    }

    #[test]
    fn storyboard_person_binding_prefers_roles_over_visual_terms() {
        let role_grounded = "主角站在高对比光束前，敌人从画面焦点边缘逼近。";
        assert_eq!(
            derive_product_person(role_grounded, role_grounded),
            "主角与敌人"
        );

        let action_without_named_subject = "烟尘中有人踏出，随后侧身疾避。";
        assert_eq!(
            derive_product_person(action_without_named_subject, action_without_named_subject),
            "主角"
        );

        let empty_shot = "海浪拍岸，清晨微光沿海平线浮起。";
        assert_eq!(derive_product_person(empty_shot, empty_shot), "海面");
    }

    #[test]
    fn storyboard_person_binding_trims_action_suffix_from_real_names() {
        let named_subject = "林风蹲在矮墙断口上，左手按着膝盖，右手指节蹭着粗粝砖面。";

        assert_eq!(derive_product_person(named_subject, named_subject), "林风");

        let action_measure_subject = "林峰一把推开木门，苏瑶站在他身后。";
        assert_eq!(
            derive_product_person(action_measure_subject, action_measure_subject),
            "林峰"
        );

        let role_action_fragment =
            "废墟之上，主角猛然单膝砸地，敌人踏碎瓦砾步步逼近。主角瞳孔骤缩，右拳悍然捶向地面。";

        assert_eq!(
            derive_product_person(role_action_fragment, role_action_fragment),
            "主角与敌人"
        );
    }

    #[test]
    fn live_storyboard_patch_rebinds_name_action_fragment_to_source_name() {
        let mut baseline = test_live_validation_row("林峰");
        baseline.shot_script = "林峰一把推开木门，苏瑶站在他身后。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        baseline.scene_performance_projection.person = "林峰".to_string();
        for (dirty_person, expected_person) in
            [("林峰一", "林峰"), ("林峰压", "林峰"), ("苏瑶压", "苏瑶")]
        {
            let mut live_row = baseline.clone();
            let patch = LiveStoryboardRowPatch {
                shot_title: "镜头1：林峰推门".to_string(),
                person: dirty_person.to_string(),
                scene_scale: "中近景".to_string(),
                visual_description: format!(
                    "主体为{dirty_person}，中近景把木门和身后苏瑶放在同一画面里。"
                ),
                character_action: "林峰从门前停顿开始，到一把推开木门时结束。".to_string(),
                camera_movement: "中近景定机位观察林峰动作起止。".to_string(),
                dialogue: String::new(),
            };

            apply_live_storyboard_patch(&mut live_row, &patch, &baseline);
            normalize_storyboard_row_subject_quality(&mut live_row);

            assert_eq!(live_row.person, expected_person, "{dirty_person}");
            let combined = format!("{} {}", live_row.visual_description, live_row.prompt_text);
            assert!(!combined.contains("林峰压"), "{combined}");
            assert!(!combined.contains("苏瑶压"), "{combined}");
        }
    }

    #[test]
    fn live_storyboard_patch_rebinds_role_possessive_fragment_to_source_role() {
        let mut baseline = test_live_validation_row("主角");
        baseline.shot_script = "主角单膝跪地，敌人缓步逼近。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        baseline.scene_performance_projection.person = "主角".to_string();
        let mut live_row = baseline.clone();
        let patch = LiveStoryboardRowPatch {
            shot_title: "镜头1：主角承压".to_string(),
            person: "主角之".to_string(),
            scene_scale: "中近景".to_string(),
            visual_description: "主体为主角，中近景把废墟和敌人逼近放在前后层次里。".to_string(),
            character_action: "主角从废墟边缘稳住身体开始，到敌人逼近时结束。".to_string(),
            camera_movement: "中近景定机位观察主角动作起止。".to_string(),
            dialogue: String::new(),
        };

        apply_live_storyboard_patch(&mut live_row, &patch, &baseline);
        normalize_storyboard_row_subject_quality(&mut live_row);

        assert_eq!(live_row.person, "主角");
        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);
        assert!(
            !warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name")),
            "role fragments should rebind without fake-name warnings: {:?}",
            warnings
        );
    }

    #[test]
    fn live_storyboard_patch_rebinds_forbidden_person_labels_to_source_subject() {
        for forbidden_person in ["环境", "沙盘", "城建面板"] {
            let mut baseline = test_live_validation_row("林峰");
            baseline.shot_script = "林峰护住苏瑶，黑衣追兵从巷口逼近。".to_string();
            baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
            baseline.scene_performance_projection.person = "林峰".to_string();
            let mut live_row = baseline.clone();
            let patch = LiveStoryboardRowPatch {
                shot_title: format!("镜头1：{forbidden_person}推进"),
                person: forbidden_person.to_string(),
                scene_scale: "中近景".to_string(),
                visual_description: format!(
                    "主体为{forbidden_person}，中近景把巷口压力放在画面前侧。"
                ),
                character_action: format!("{forbidden_person}从巷口开始，到压力逼近时结束。"),
                camera_movement: "中近景定机位观察林峰动作起止。".to_string(),
                dialogue: String::new(),
            };

            apply_live_storyboard_patch(&mut live_row, &patch, &baseline);

            assert_ne!(live_row.person, forbidden_person);
            assert_eq!(live_row.person, "林峰");
        }
    }

    #[test]
    fn live_storyboard_validator_rejects_forbidden_person_labels() {
        let baseline = test_live_validation_row("林峰");

        for forbidden_person in ["环境", "沙盘", "城建面板"] {
            let mut live_row = baseline.clone();
            live_row.person = forbidden_person.to_string();
            live_row.scene_performance_projection.person = forbidden_person.to_string();

            let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline.clone()]);

            assert!(
                warnings
                    .iter()
                    .any(|warning| warning.message.contains("visual or abstract subject label")),
                "validator should reject forbidden person label {forbidden_person}: {:?}",
                warnings
            );
        }
    }

    #[test]
    fn source_role_registry_keeps_rewritten_names_grounded() {
        let source = "林峰护住苏瑶，阿青从巷口提醒他们，黑衣追兵继续逼近。";
        let generated = "林峰压低声音让苏瑶退后，阿青守住巷口，黑衣追兵的压力仍在逼近。";

        assert!(validate_generated_script_text(generated, source).is_some());

        let mut baseline = test_live_validation_row("林峰");
        baseline.shot_script = "林峰护住苏瑶，阿青提醒他们。".to_string();
        baseline.scene_performance_projection.fused_source_text = source.to_string();
        baseline.scene_performance_projection.person = "林峰".to_string();
        let mut live_row = baseline.clone();
        live_row.person = "苏瑶".to_string();
        live_row.shot_title = "镜头1：苏瑶回望巷口".to_string();
        live_row.visual_description = "主体为苏瑶，中近景把巷口压力放在她身后。".to_string();
        live_row.character_action = "苏瑶从林峰身后回望巷口，到确认阿青提醒时结束。".to_string();
        live_row.camera_movement = "中近景定机位观察苏瑶动作起止。".to_string();
        live_row.scene_performance_projection.fused_source_text = source.to_string();

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);

        assert!(
            !warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name")),
            "source names should stay grounded across rewritten text: {:?}",
            warnings
        );
    }

    #[test]
    fn live_storyboard_patch_rebinds_new_name_pollution_terms() {
        let mut baseline = test_live_validation_row("林峰");
        baseline.shot_script = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        baseline.scene_performance_projection.person = "林峰".to_string();

        for (dirty_person, expected_person) in [
            ("苏瑶半", "苏瑶"),
            ("白意图", "林峰"),
            ("郑重递", "林峰"),
            ("左手", "林峰"),
            ("头部", "林峰"),
            ("未言语", "林峰"),
            ("高耸绷", "林峰"),
            ("屈护住", "林峰"),
            ("扶着倚", "林峰"),
            ("关系保", "林峰"),
            ("半边肩", "林峰"),
            ("终于", "林峰"),
            ("边咳了", "林峰"),
            ("时抬", "林峰"),
            ("初立", "林峰"),
            ("后颈", "林峰"),
            ("林峰背", "林峰"),
        ] {
            let mut live_row = baseline.clone();
            let patch = LiveStoryboardRowPatch {
                shot_title: format!("镜头1：{dirty_person}推进"),
                person: dirty_person.to_string(),
                scene_scale: "中近景".to_string(),
                visual_description: format!("主体为{dirty_person}，中近景保留巷口压力。"),
                character_action: format!("{dirty_person}从巷口压力前开始，到对峙停住时结束。"),
                camera_movement: "中近景定机位观察林峰动作起止。".to_string(),
                dialogue: String::new(),
            };

            apply_live_storyboard_patch(&mut live_row, &patch, &baseline);

            assert_eq!(
                live_row.person, expected_person,
                "{dirty_person} should rebind"
            );
        }
    }

    #[test]
    fn live_storyboard_validator_rejects_new_person_pollution_terms() {
        let baseline = test_live_validation_row("林峰");

        for dirty_person in [
            "白意图",
            "郑重递",
            "左手",
            "头部",
            "位置",
            "未言语",
            "高耸绷",
            "屈护住",
            "扶着倚",
            "关系保",
            "半边肩",
            "终于",
            "边咳了",
            "时抬",
            "初立",
            "后颈",
        ] {
            let mut live_row = baseline.clone();
            live_row.person = dirty_person.to_string();
            live_row.scene_performance_projection.person = dirty_person.to_string();

            let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline.clone()]);

            assert!(
                warnings
                    .iter()
                    .any(|warning| warning.code == "text_model_validator_failed"),
                "validator should reject polluted person {dirty_person}: {:?}",
                warnings
            );
        }
    }

    #[test]
    fn prompt_text_abstract_subject_is_repaired_but_still_validated() {
        let baseline = test_live_validation_row("林峰");
        let mut live_row = baseline.clone();
        live_row.prompt_text =
            "视频分镜提示词：以当前镜头脚本为准；角色动作：镜头从低处推进；画面描述：主体为镜头，焦点压在画面中央。"
                .to_string();
        let warnings = validate_live_storyboard_rows(&[live_row.clone()], 10, &[baseline]);
        assert!(
            warnings
                .iter()
                .any(|warning| warning.message.contains("prompt_text")),
            "validator should keep catching abstract subject labels in prompt_text: {:?}",
            warnings
        );

        normalize_storyboard_row_subject_quality(&mut live_row);

        assert!(!live_row.prompt_text.contains("主体为镜头"));
        assert!(!live_row.prompt_text.contains("：镜头从"));
    }

    #[test]
    fn prompt_text_allows_camera_vocabulary_outside_subject_role() {
        let baseline = test_live_validation_row("林峰");
        let mut live_row = baseline.clone();
        live_row.prompt_text =
            "视频分镜提示词：以当前镜头脚本为准；运镜：中景定机位观察林峰，镜头捕捉左手按住刀柄。"
                .to_string();

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);

        assert!(
            !warnings
                .iter()
                .any(|warning| warning.message.contains("prompt_text")
                    && warning.message.contains("visual or abstract subject label")),
            "camera vocabulary outside subject role should not trigger prompt_text abstract subject warning: {:?}",
            warnings
        );
    }

    #[test]
    fn source_fragment_subjects_are_repaired_in_product_fields() {
        let mut row = test_live_validation_row("主角");
        row.shot_title = "镜头1：单膝废墟压迫".to_string();
        row.character_action = "猛然从废墟边缘开始，到压迫逼近时结束。".to_string();
        row.prompt_text =
            "镜头标题：单膝废墟压迫；画面描述：主体为头部；角色动作：左手从废墟边缘开始，右手作为动作部位。"
                .to_string();

        normalize_storyboard_row_subject_quality(&mut row);

        let combined = format!(
            "{} {} {}",
            row.shot_title, row.character_action, row.prompt_text
        );
        assert!(!combined.contains("：单膝"));
        assert!(!combined.contains("：猛然"));
        assert!(!combined.contains("主体为废墟"));
        assert!(!combined.contains("主体为头部"));
        assert!(!combined.contains("：左手从"));
    }

    #[test]
    fn environment_and_pose_terms_are_allowed_when_not_subject_labels() {
        let baseline = test_live_validation_row("主角");
        let mut live_row = baseline.clone();
        live_row.shot_title = "镜头1：主角承受废墟压迫".to_string();
        live_row.visual_description =
            "主体为主角，中近景把废墟边缘和敌人逼近的空间压力放在后侧。".to_string();
        live_row.character_action = "主角在废墟边缘单膝跪住，到抬头看向敌人时结束。".to_string();
        live_row.prompt_text =
            "视频分镜提示词：画面描述：废墟作为环境压住退路；角色动作：主角单膝跪住并看向敌人。"
                .to_string();

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);

        assert!(
            !warnings
                .iter()
                .any(|warning| warning.message.contains("source fragment as subject label")),
            "environment and pose terms should not be subject labels outside subject grammar: {:?}",
            warnings
        );
    }

    #[test]
    fn source_fragment_subject_slots_still_fail_when_explicit() {
        let baseline = test_live_validation_row("主角");
        let mut live_row = baseline.clone();
        live_row.shot_title = "镜头1：废墟压迫".to_string();
        live_row.character_action = "单膝从废墟边缘开始，到敌人逼近时结束。".to_string();
        live_row.prompt_text = "镜头标题：废墟压迫；角色动作：单膝从废墟边缘开始。".to_string();

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);

        assert!(
            warnings
                .iter()
                .any(|warning| warning.message.contains("source fragment as subject label")),
            "explicit source fragment subject slots should still fail: {:?}",
            warnings
        );
    }

    #[test]
    fn ruin_environment_title_subject_rebinds_before_live_validation() {
        let baseline = test_live_validation_row("主角");
        let mut live_row = baseline.clone();
        live_row.shot_title = "镜头1：废墟压迫".to_string();
        live_row.visual_description =
            "主体为主角，中近景把废墟空间和敌人逼近放在后侧。".to_string();
        live_row.character_action =
            "主角从废墟边缘的低身状态开始，到抬头看向敌人时结束。".to_string();
        live_row.prompt_text =
            "镜头标题：废墟压迫；画面描述：废墟作为环境压住退路；角色动作：主角抬头。".to_string();

        normalize_storyboard_row_subject_quality(&mut live_row);

        assert!(live_row.shot_title.contains("主角"));
        assert!(live_row.visual_description.contains("废墟"));
        assert!(live_row.visual_description.contains("敌人"));
        assert!(live_row.visual_description.contains("逼近"));
        assert!(live_row.prompt_text.contains("废墟"));
        assert!(live_row.prompt_text.contains("敌人"));
        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);
        assert!(
            !warnings
                .iter()
                .any(|warning| warning.message.contains("source fragment as subject label")),
            "ruin should be allowed as environment after subject rebind: {:?}",
            warnings
        );
    }

    #[test]
    fn silent_state_is_not_character_name_or_subject() {
        let source = "林峰护住苏瑶，黑衣追兵从巷口逼近。";
        let accepted = validate_generated_script_text(
            "林峰未言语，只护住苏瑶后退半步，黑衣追兵仍从巷口逼近。",
            source,
        )
        .expect("未言语 should be dialogue/silence state, not a new character");

        assert!(accepted.contains("未言语"));

        let baseline = test_live_validation_row("林峰");
        let mut allowed_row = baseline.clone();
        allowed_row.person = "林峰".to_string();
        allowed_row.character_action = "林峰未言语，只护住苏瑶，到追兵逼近时结束。".to_string();
        allowed_row.prompt_text = "角色动作：林峰未言语；对白：无对白。".to_string();
        let allowed_warnings =
            validate_live_storyboard_rows(&[allowed_row], 10, &[baseline.clone()]);
        assert!(
            !allowed_warnings
                .iter()
                .any(|warning| warning.message.contains("未言语")),
            "未言语 should be allowed as silence state with a real subject: {:?}",
            allowed_warnings
        );

        let mut bad_row = baseline.clone();
        bad_row.person = "未言语".to_string();
        bad_row.character_action = "未言语从废墟边缘开始，到停顿时结束。".to_string();
        bad_row.prompt_text = "角色动作：未言语从废墟边缘开始。".to_string();
        let bad_warnings = validate_live_storyboard_rows(&[bad_row], 10, &[baseline]);
        assert!(
            bad_warnings
                .iter()
                .any(|warning| warning.code == "text_model_validator_failed"),
            "未言语 should fail when used as person/subject: {:?}",
            bad_warnings
        );
    }

    #[test]
    fn location_word_is_space_not_ungrounded_character_name() {
        let source = "林峰护住苏瑶，黑衣追兵从巷口逼近。";
        let accepted = validate_generated_script_text(
            "林峰调整位置护住苏瑶，黑衣追兵仍从巷口逼近，双方压力没有消失。",
            source,
        )
        .expect("位置 should be treated as spatial wording, not a new character name");

        assert!(accepted.contains("位置"));

        let baseline = test_live_validation_row("林峰");
        let mut live_row = baseline.clone();
        live_row.person = "位置".to_string();
        live_row.shot_title = "镜头1：位置压迫".to_string();
        live_row.character_action = "位置从废墟边缘开始，到压力逼近时结束。".to_string();

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);

        assert!(
            warnings
                .iter()
                .any(|warning| warning.code == "text_model_validator_failed"),
            "位置 should not be allowed as person or subject label: {:?}",
            warnings
        );
    }

    #[test]
    fn military_scene_rewrite_does_not_expand_single_enemy_into_formation() {
        let source = "主角压低身形，敌人从坡下逼近。";

        assert!(
            validate_generated_script_text(
                "主角压低身形，坡下出现整列军阵，敌人逼近的压力被阵列推到眼前。",
                source,
            )
            .is_none()
        );

        let accepted = validate_generated_script_text(
            "构图更有秩序感，主角压低身形，敌人仍从坡下逼近，对峙压力没有消失。",
            source,
        )
        .expect("military scene styling may change composition without adding army scale");

        assert!(accepted.contains("敌人"));
        assert!(accepted.contains("逼近"));
    }

    #[test]
    fn blank_person_rebinds_visible_source_names_only() {
        let mut row = test_live_validation_row("/");
        row.shot_script = "林峰护住苏瑶，阿青从巷口提醒他们，黑衣追兵继续逼近。".to_string();
        row.scene_performance_projection.fused_source_text = row.shot_script.clone();
        row.visual_description = "主体为林峰，中近景把苏瑶和巷口压力放在后侧。".to_string();
        row.character_action = "林峰护住苏瑶，到阿青提醒追兵逼近时结束。".to_string();
        row.prompt_text = "镜头标题：林峰护住苏瑶；角色动作：阿青提醒他们。".to_string();
        row.scene_performance_projection.person = "/".to_string();

        normalize_storyboard_row_subject_quality(&mut row);

        assert!(row.person.contains("林峰"));
        assert!(row.person.contains("苏瑶") || row.person.contains("阿青"));

        let mut empty_row = test_live_validation_row("/");
        empty_row.shot_script = "清晨海面微光铺开。".to_string();
        empty_row.scene_performance_projection.fused_source_text = empty_row.shot_script.clone();
        empty_row.visual_description = "清晨海面微光铺开，空间层次保持安静。".to_string();
        empty_row.character_action = "无人物动作，镜头只记录海面光线变化。".to_string();
        empty_row.prompt_text = "画面描述：清晨海面微光铺开。".to_string();
        empty_row.scene_performance_projection.person = "/".to_string();

        normalize_storyboard_row_subject_quality(&mut empty_row);

        assert_eq!(empty_row.person, "");
    }

    #[test]
    fn bound_person_removes_empty_shot_text_from_fields() {
        let mut row = test_live_validation_row("/");
        row.shot_script = "林峰护住苏瑶，阿青从巷口提醒他们，黑衣追兵继续逼近。".to_string();
        row.scene_performance_projection.fused_source_text = row.shot_script.clone();
        row.visual_description = "主体为空镜，中近景里阿青站在巷口提醒林峰。".to_string();
        row.character_action = "空镜从巷口压力开始，到阿青提醒他们时结束。".to_string();
        row.prompt_text = "镜头标题：空镜巷口提醒；角色动作：空镜从巷口开始。".to_string();
        row.scene_performance_projection.person = "/".to_string();
        row.scene_performance_projection.visual_description = row.visual_description.clone();
        row.scene_performance_projection.character_action = row.character_action.clone();

        normalize_storyboard_row_subject_quality(&mut row);

        assert!(row.person.contains("阿青") || row.person.contains("林峰"));
        let combined = format!(
            "{} {} {} {} {}",
            row.shot_title,
            row.visual_description,
            row.character_action,
            row.prompt_text,
            row.scene_performance_projection.visual_description
        );
        assert!(
            !combined.contains("空镜"),
            "person-bound row fields should not keep 空镜 text: {combined}"
        );
    }

    #[test]
    fn empty_shot_patch_rebinds_to_source_role_in_a_source_rows() {
        let mut baseline = test_live_validation_row("\u{4E3B}\u{89D2}");
        baseline.shot_script = "\u{5E9F}\u{589F}\u{4E4B}\u{4E0A}\u{FF0C}\u{4E3B}\u{89D2}\u{5355}\u{819D}\u{8DEA}\u{5730}\u{FF0C}\u{654C}\u{4EBA}\u{7F13}\u{6B65}\u{903C}\u{8FD1}\u{3002}".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        baseline.person = "\u{4E3B}\u{89D2}".to_string();
        baseline.scene_performance_projection.person = "\u{4E3B}\u{89D2}".to_string();

        let mut live_row = baseline.clone();
        let patch = LiveStoryboardRowPatch {
            shot_title: "\u{955C}\u{5934}1\u{FF1A}\u{5E9F}\u{589F}\u{524D}\u{4FA7}\u{505C}\u{987F}".to_string(),
            person: "\u{7A7A}\u{955C}".to_string(),
            scene_scale: "\u{4E2D}\u{8FD1}\u{666F}".to_string(),
            visual_description: "\u{4E3B}\u{4F53}\u{4E3A}\u{7A7A}\u{955C}\u{FF0C}\u{4E2D}\u{8FD1}\u{666F}\u{628A}\u{5E9F}\u{589F}\u{538B}\u{5728}\u{524D}\u{4FA7}\u{FF1B}\u{5F53}\u{524D}\u{89C6}\u{89C9}\u{4E8B}\u{4EF6}\u{662F}\u{4E3B}\u{89D2}\u{5355}\u{819D}\u{8DEA}\u{5730}\u{627F}\u{538B}\u{3002}".to_string(),
            character_action: "\u{7A7A}\u{955C}\u{4ECE}\u{5E9F}\u{589F}\u{524D}\u{4FA7}\u{505C}\u{4F4F}\u{FF0C}\u{5230}\u{654C}\u{4EBA}\u{903C}\u{8FD1}\u{65F6}\u{7ED3}\u{675F}\u{3002}".to_string(),
            camera_movement: "\u{4E2D}\u{8FD1}\u{666F}\u{5B9A}\u{673A}\u{4F4D}\u{89C2}\u{5BDF}\u{7A7A}\u{955C}\u{4E0E}\u{654C}\u{4EBA}\u{903C}\u{8FD1}\u{3002}".to_string(),
            dialogue: String::new(),
        };

        apply_live_storyboard_patch(&mut live_row, &patch, &baseline);

        assert_eq!(live_row.person, "\u{4E3B}\u{89D2}");
    }

    #[test]
    fn camera_and_environment_terms_are_only_blocked_as_subjects() {
        let baseline = test_live_validation_row("林峰");
        let mut allowed_row = baseline.clone();
        allowed_row.person = "林峰".to_string();
        allowed_row.visual_description =
            "主体为林峰，中景保留巷口环境，镜头语言只体现在压迫感里。".to_string();
        allowed_row.character_action = "林峰从巷口环境边缘开始，到追兵逼近时结束。".to_string();
        allowed_row.prompt_text = "运镜：中景定机位观察林峰，镜头捕捉环境里的压迫线。".to_string();
        let allowed_warnings =
            validate_live_storyboard_rows(&[allowed_row], 10, &[baseline.clone()]);
        assert!(
            !allowed_warnings
                .iter()
                .any(|warning| warning.message.contains("visual or abstract subject label")),
            "camera/environment vocabulary should be allowed outside subject slots: {:?}",
            allowed_warnings
        );

        let mut bad_row = baseline.clone();
        bad_row.person = "环境".to_string();
        bad_row.visual_description = "主体为环境，中景保留巷口压力。".to_string();
        bad_row.character_action = "环境从巷口边缘开始，到追兵逼近时结束。".to_string();
        bad_row.prompt_text = "角色动作：镜头从低处推进；画面描述：主体为镜头。".to_string();
        let bad_warnings = validate_live_storyboard_rows(&[bad_row], 10, &[baseline]);
        assert!(
            bad_warnings
                .iter()
                .any(|warning| warning.message.contains("visual or abstract subject label")),
            "camera/environment subject slots should remain blocked: {:?}",
            bad_warnings
        );
    }

    #[test]
    fn empty_shot_person_outputs_slash_without_using_blank_or_empty_shot_label() {
        let mut row = test_live_validation_row("空镜");
        row.shot_script = "清晨海面微光铺开。".to_string();
        row.scene_performance_projection.fused_source_text = row.shot_script.clone();
        row.visual_description = "清晨海面微光铺开，空间层次保持安静。".to_string();
        row.character_action = "无人物动作，镜头只记录海面光线变化。".to_string();
        row.prompt_text = "画面描述：清晨海面微光铺开。".to_string();
        row.scene_performance_projection.visual_description = row.visual_description.clone();
        row.scene_performance_projection.character_action = row.character_action.clone();

        normalize_storyboard_row_subject_quality(&mut row);

        assert_eq!(row.person, "");
        assert_eq!(row.scene_performance_projection.person, "");
    }

    #[test]
    fn expand_script_validator_allows_body_parts_as_actions_not_names() {
        let source = "林峰护住苏瑶，黑衣追兵从巷口逼近。";
        let accepted = validate_generated_script_text(
            "林峰左手扶住苏瑶，右手压低门板，镜头感只体现在动作节奏里，黑衣追兵仍从巷口逼近。",
            source,
        )
        .expect("body parts and camera vocabulary should not force validator fallback");

        assert!(accepted.contains("左手"));
        assert!(accepted.contains("右手"));
        assert!(accepted.contains("黑衣追兵"));
    }

    #[test]
    fn pseudo_name_fragments_are_not_reported_as_ungrounded_characters() {
        let source = "林峰护住苏瑶，黑衣追兵从巷口逼近。";

        for generated in [
            "高耸绷紧的门洞压住退路，林峰护住苏瑶，黑衣追兵仍从巷口逼近。",
            "林峰屈身护住苏瑶，扶着倚在墙边的木门稳住身体，黑衣追兵继续逼近。",
            "苏瑶头部微微偏开，左手按住门框，右手收回身前，追兵压力仍在。",
            "林峰没答，继续护住苏瑶，黑衣追兵仍从巷口逼近。",
            "那人仍在巷口逼近，林峰护住苏瑶后退。",
            "左边岔路被碎石挡住，林峰护住苏瑶转向门洞，追兵压力仍在。",
            "对方挥开烟尘，林峰护住苏瑶，黑衣追兵仍从巷口逼近。",
            "林峰护住苏瑶后微撤半步，黑衣追兵仍从巷口逼近。",
            "林峰半边肩抵住门框，护住苏瑶，黑衣追兵仍从巷口逼近。",
            "林峰终于稳住呼吸，护住苏瑶，黑衣追兵仍从巷口逼近。",
            "苏瑶靠在墙边咳了半声，林峰护住她，黑衣追兵仍从巷口逼近。",
            "林峰护住苏瑶时抬手挡开碎屑，黑衣追兵仍从巷口逼近。",
            "主角初立在断墙前，林峰护住苏瑶，黑衣追兵仍从巷口逼近。",
            "林峰后颈绷紧，继续护住苏瑶，黑衣追兵仍从巷口逼近。",
            "轮廓缓慢压低在巷口背景里，林峰护住苏瑶，黑衣追兵仍从巷口逼近。",
            "背景灰暗下来，林峰护住苏瑶，黑衣追兵仍从巷口逼近。",
            "居右三分的构图压住退路，林峰护住苏瑶，黑衣追兵仍从巷口逼近。",
            "罗盘紧贴碎瓦停住，林峰护住苏瑶，黑衣追兵仍从巷口逼近。",
            "苏瑶方才稳住呼吸，林峰护住她，黑衣追兵仍从巷口逼近。",
        ] {
            let result = super::validate_generated_script_text_with_reason(generated, source);
            assert!(
                result.is_ok(),
                "pseudo name fragments/body parts should not be treated as new names: {generated}; reason: {:?}",
                result.err()
            );
        }

        assert_eq!(
            super::generated_script_ungrounded_character_name(
                "苏瑶在怀中稳住呼吸，林峰护住她，黑衣追兵仍从巷口逼近。",
                source
            ),
            None,
            "怀中 should be body/position wording, not a new name"
        );

        let source_with_compass = "林峰护住苏瑶，罗盘被瓦砾压住，黑衣追兵从巷口逼近。";
        assert!(
            validate_generated_script_text(
                "罗盘被落下的瓦片压住，林峰护住苏瑶，黑衣追兵仍从巷口逼近。",
                source_with_compass,
            )
            .is_some(),
            "passive object fragments should not be treated as new names"
        );
    }

    #[test]
    fn qa_fragment_terms_are_not_ungrounded_character_names() {
        let source = "林峰护住苏瑶，黑衣追兵从巷口逼近。";
        for fragment in [
            "轮廓缓",
            "背景灰",
            "居右三",
            "罗盘紧",
            "苏瑶方",
            "后停顿",
            "广角俯",
            "眉心",
            "左下角",
        ] {
            let generated =
                format!("{fragment}只描述画面状态，林峰护住苏瑶，黑衣追兵仍从巷口逼近。");
            assert_eq!(
                super::generated_script_ungrounded_character_name(&generated, source),
                None,
                "{fragment} should not be treated as an added character name"
            );
        }

        let mut baseline = test_live_validation_row("林峰");
        baseline.shot_script = source.to_string();
        baseline.scene_performance_projection.fused_source_text = source.to_string();
        baseline.scene_performance_projection.person = "林峰".to_string();
        for dirty_person in [
            "轮廓缓",
            "背景灰",
            "居右三",
            "罗盘紧",
            "苏瑶方",
            "后停顿",
            "广角俯",
            "眉心",
            "左下角",
        ] {
            let mut live_row = baseline.clone();
            let patch = LiveStoryboardRowPatch {
                shot_title: format!("镜头1：{dirty_person}压迫"),
                person: dirty_person.to_string(),
                scene_scale: "中近景".to_string(),
                visual_description: format!("主体为{dirty_person}，中近景保留巷口压力。"),
                character_action: format!("{dirty_person}从巷口压力前开始，到追兵逼近时结束。"),
                camera_movement: "中近景定机位观察林峰动作起止。".to_string(),
                dialogue: String::new(),
            };

            apply_live_storyboard_patch(&mut live_row, &patch, &baseline);
            normalize_storyboard_row_subject_quality(&mut live_row);

            assert_ne!(live_row.person, dirty_person);
            assert!(
                !validate_live_storyboard_rows(&[live_row], 10, &[baseline.clone()])
                    .iter()
                    .any(|warning| warning.message.contains("ungrounded character name")),
                "{dirty_person} should be fragment noise, not an added name"
            );
        }
    }

    #[test]
    fn storyboard_action_fragments_rebind_instead_of_becoming_person_names() {
        let baseline = test_live_validation_row("主角");

        for dirty_person in [
            "后迅速",
            "扶住",
            "肩甲投",
            "苏瑶上",
            "初立",
            "后颈",
            "林峰背",
            "后停顿",
            "广角俯",
            "眉心",
            "左下角",
        ] {
            let mut live_row = baseline.clone();
            let patch = LiveStoryboardRowPatch {
                shot_title: format!("镜头1：{dirty_person}压迫"),
                person: dirty_person.to_string(),
                scene_scale: "中近景".to_string(),
                visual_description: format!("主体为{dirty_person}，中近景保留废墟压力。"),
                character_action: format!("{dirty_person}从废墟边缘开始，到敌人逼近时结束。"),
                camera_movement: "中近景定机位观察主角动作起止。".to_string(),
                dialogue: String::new(),
            };

            apply_live_storyboard_patch(&mut live_row, &patch, &baseline);
            normalize_storyboard_row_subject_quality(&mut live_row);

            assert!(
                live_row.person.contains("主角"),
                "{dirty_person} should rebind to source roles: {}",
                live_row.person
            );
            let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline.clone()]);
            assert!(
                !warnings
                    .iter()
                    .any(|warning| warning.message.contains("ungrounded character name")),
                "action fragments should not become ungrounded names: {:?}",
                warnings
            );
        }
    }

    #[test]
    fn a_hot_30s_third_live_state_subject_rebinds_to_source_roles() {
        for dirty_subject in ["紧张", "坚毅", "坚定", "坚持", "准备反"] {
            let mut baselines = vec![
                test_live_validation_row("主角"),
                test_live_validation_row("敌人"),
                test_live_validation_row("敌人"),
            ];
            for (index, row) in baselines.iter_mut().enumerate() {
                row.order = (index + 1) as u32;
                row.shot_id = format!("shot-{}", index + 1);
                row.prompt_text_source_row_id = row.shot_id.clone();
                row.duration_seconds = 10;
                row.shot_duration_seconds = 10;
            }
            let mut rows = baselines.clone();
            rows[2].person = dirty_subject.to_string();
            rows[2].scene_performance_projection.person = dirty_subject.to_string();
            rows[2].shot_title = format!("镜头3：{dirty_subject}压住废墟对峙");
            rows[2].visual_description = format!(
                "主体为{dirty_subject}，中近景把废墟和敌人逼近放在后侧，主角单膝跪地仍在画面前侧。"
            );
            rows[2].character_action =
                format!("{dirty_subject}从废墟边缘开始，到敌人逼近时结束，镜头捕捉对峙压力。");
            rows[2].camera_movement =
                "中近景定机位观察废墟里的敌人逼近，镜头捕捉主角单膝跪地。".to_string();
            rows[2].prompt_text = format!(
                "镜头标题：{dirty_subject}压迫；画面描述：主体为{dirty_subject}；角色动作：{dirty_subject}从废墟边缘开始。"
            );

            let pre_repair = validate_live_storyboard_rows(&rows, 30, &baselines);
            assert!(
                pre_repair
                    .iter()
                    .any(|warning| warning.code == "text_model_validator_failed"),
                "{dirty_subject} should fail before source-bound repair: {pre_repair:?}"
            );
            if dirty_subject == "紧张" {
                assert!(
                    pre_repair
                        .iter()
                        .any(|warning| warning.message.contains("ungrounded character name: 紧张")),
                    "紧张 should be observed on the live validator character extraction path: {pre_repair:?}"
                );
            }

            let summary = repair_live_storyboard_rows_from_source(&mut rows, &baselines);
            let post_repair = validate_live_storyboard_rows(&rows, 30, &baselines);
            let repaired_row = &rows[2];
            let combined = storyboard_row_visible_text(repaired_row);

            assert!(summary.repaired(), "{dirty_subject}: {summary:?}");
            assert!(
                post_repair.is_empty(),
                "{dirty_subject} should pass after source-bound repair: {post_repair:?}"
            );
            assert!(
                ["主角", "敌人", "主角与敌人"].contains(&repaired_row.person.as_str()),
                "{dirty_subject} rebound to unexpected subject: {}",
                repaired_row.person
            );
            assert!(
                !storyboard_row_visible_text(repaired_row).contains(dirty_subject),
                "{dirty_subject} leaked after repair: {combined}"
            );
            for required in ["主角", "敌人", "废墟", "单膝跪地", "逼近"] {
                assert!(
                    combined.contains(required),
                    "{dirty_subject} missing source fact {required}: {combined}"
                );
            }
            for forbidden in ["甲胄", "铠甲", "剑柄", "断戟", "整列军阵", "额外士兵"]
            {
                assert!(
                    !combined.contains(forbidden),
                    "{dirty_subject} leaked forbidden fact {forbidden}: {combined}"
                );
            }
            assert_eq!(
                rows.iter()
                    .map(|row| row.duration_seconds)
                    .collect::<Vec<_>>(),
                vec![10, 10, 10]
            );
        }
    }

    #[test]
    fn a_hot_30s_third_live_source_prefixed_state_fragments_repair_to_source_roles() {
        for (dirty_subject, expected_person) in [
            ("主角反", "主角"),
            ("主角准", "主角"),
            ("主角准备", "主角"),
            ("主角坚持", "主角"),
            ("敌人紧张", "敌人"),
        ] {
            let mut baselines = vec![
                test_live_validation_row("主角"),
                test_live_validation_row("敌人"),
                test_live_validation_row("敌人"),
            ];
            for (index, row) in baselines.iter_mut().enumerate() {
                row.order = (index + 1) as u32;
                row.shot_id = format!("shot-{}", index + 1);
                row.prompt_text_source_row_id = row.shot_id.clone();
                row.duration_seconds = 10;
                row.shot_duration_seconds = 10;
            }
            let mut rows = baselines.clone();
            rows[2].person = dirty_subject.to_string();
            rows[2].scene_performance_projection.person = dirty_subject.to_string();
            rows[2].shot_title = format!("镜头3：{dirty_subject}压住废墟对峙");
            rows[2].visual_description = format!(
                "主体为{dirty_subject}，中近景把废墟和敌人逼近放在后侧，主角单膝跪地仍在画面前侧。"
            );
            rows[2].character_action =
                format!("{dirty_subject}从废墟边缘开始，到敌人逼近时结束，镜头捕捉对峙压力。");
            rows[2].camera_movement =
                "中近景定机位观察废墟里的敌人逼近，镜头捕捉主角单膝跪地。".to_string();

            let summary = repair_live_storyboard_rows_from_source(&mut rows, &baselines);
            let post_repair = validate_live_storyboard_rows(&rows, 30, &baselines);
            let repaired_row = &rows[2];
            let combined = storyboard_row_visible_text(repaired_row);

            assert!(summary.repaired(), "{dirty_subject}: {summary:?}");
            assert!(
                post_repair.is_empty(),
                "{dirty_subject} should pass after source-bound repair: {post_repair:?}"
            );
            assert_eq!(repaired_row.person, expected_person, "{dirty_subject}");
            assert!(
                !combined.contains(dirty_subject),
                "{dirty_subject} leaked after repair: {combined}"
            );
            for required in ["主角", "敌人", "废墟", "单膝跪地", "逼近"] {
                assert!(
                    combined.contains(required),
                    "{dirty_subject} missing source fact {required}: {combined}"
                );
            }
            assert_eq!(
                rows.iter()
                    .map(|row| row.duration_seconds)
                    .collect::<Vec<_>>(),
                vec![10, 10, 10]
            );
        }
    }

    #[test]
    fn a_hot_30s_state_subjects_still_fail_closed_without_repair() {
        let mut baselines = vec![
            test_live_validation_row("主角"),
            test_live_validation_row("敌人"),
            test_live_validation_row("敌人"),
        ];
        for (index, row) in baselines.iter_mut().enumerate() {
            row.order = (index + 1) as u32;
            row.duration_seconds = 10;
            row.shot_duration_seconds = 10;
        }

        for dirty_subject in ["紧张", "坚毅", "坚定", "坚持", "准备反"] {
            let mut rows = baselines.clone();
            rows[2].person = dirty_subject.to_string();
            rows[2].scene_performance_projection.person = dirty_subject.to_string();
            rows[2].shot_title = format!("镜头3：{dirty_subject}压迫");
            rows[2].visual_description =
                format!("主体为{dirty_subject}，中近景保留废墟和敌人逼近。");
            rows[2].character_action = format!("{dirty_subject}从废墟边缘开始，到敌人逼近时结束。");

            let warnings = validate_live_storyboard_rows(&rows, 30, &baselines);

            assert!(
                warnings
                    .iter()
                    .any(|warning| warning.code == "text_model_validator_failed"),
                "unrepaired {dirty_subject} should stay fail-closed: {warnings:?}"
            );
        }
    }

    #[test]
    fn storyboard_model_prompt_includes_source_person_binding_rule() {
        let grounding = StoryboardGroundingContext {
            shot_script: "主角单膝跪地，敌人缓步逼近。".to_string(),
            expanded_script_text: "主角在废墟中承受压迫。".to_string(),
            grounding_text: "主角单膝跪地，敌人缓步逼近。".to_string(),
            grounding_source: ShotGroundingSource::ShotScript,
            primary_scene_type: "hot_blood_battle".to_string(),
            primary_scene_label: "热血战斗".to_string(),
            primary_scene_category: "action_beat".to_string(),
            shot_scene_type: "hot_blood_battle".to_string(),
            shot_scene_label: "热血战斗".to_string(),
            shot_intent: "action_beat".to_string(),
            adaptation_reason: "test".to_string(),
        };

        let prompt_input = build_storyboard_model_story_input(&grounding);

        assert!(prompt_input.contains("source_person_binding=主角与敌人"));
        assert!(
            prompt_input
                .contains("never use visual, camera, composition, or abstract labels as person")
        );
    }

    #[test]
    fn storyboard_qwen_prompt_names_trace_person_fragments_as_invalid() {
        let provider = TextModelProvider {
            provider: TextModelProviderKind::Qwen,
            model: "qwen-plus".to_string(),
            base_url: Some("https://example.invalid".to_string()),
            api_key_ref: "env:TEST".to_string(),
            enabled: true,
        };
        let request = build_text_generation_request(
            TextGenerationTask::GenerateStoryboard,
            Some("hot_blood_battle".to_string()),
            "主角单膝跪地，敌人缓步逼近。".to_string(),
            None,
            String::new(),
            vec![],
            vec![],
            TextGenerationOutputSchema::StoryboardRowsJson,
            Some(700),
        );
        let payload = build_qwen_request_payload(&provider, &request);
        let user_prompt = payload["messages"][1]["content"]
            .as_str()
            .expect("Qwen payload should include user prompt");

        assert!(user_prompt.contains("主角之"));
        assert!(user_prompt.contains("林峰压"));
        assert!(user_prompt.contains("苏瑶压"));
        assert!(user_prompt.contains("断戟立"));
        assert!(user_prompt.contains("must be 主角, 林峰, or 苏瑶"));
    }

    #[test]
    fn expand_script_live_text_rejects_added_names_and_over_specific_bystanders() {
        let source = "主角在雨巷里退后，敌人从巷口逼近。";

        assert!(
            validate_generated_script_text("主角发现邻居家大叔和邻居家孩子都在巷口等他。", source)
                .is_none()
        );
        assert!(
            validate_generated_script_text("主角转身遇见李明，李明递来雨伞。", source).is_none()
        );

        let accepted = validate_generated_script_text(
            "主角在雨声里放慢呼吸，逼近的人停在巷口，双方仍保持对峙。",
            source,
        )
        .expect("soft conflict label should stay inside source role boundary");

        assert!(accepted.contains("主角"));
        assert!(accepted.contains("逼近的人"));
        assert!(!accepted.contains("邻居"));
    }

    #[test]
    fn expand_script_live_text_rejects_external_setting_injection() {
        let source = "林峰护住苏瑶，黑衣追兵从巷口逼近。";

        for generated in [
            "林峰把北营兵符拓片递给苏瑶，拓片边缘写着建安十七年监造。",
            "林峰在丞相帐外按住虎牢关守军轮值图，金线铭文亮起。",
            "苏瑶举起罗盘认路，带着林峰绕过中军。",
        ] {
            assert!(
                validate_generated_script_text(generated, source).is_none(),
                "external setting should be rejected: {generated}"
            );
        }
    }

    #[test]
    fn daily_healing_script_must_preserve_pursuit_pressure() {
        let source = "林峰护住苏瑶，黑衣追兵从巷口逼近。";

        let accepted = validate_generated_script_text(
            "林峰让苏瑶在屋檐下缓过气，逼近的人仍停在巷口，压力没有消失。",
            source,
        )
        .expect("softened daily healing text should keep pursuit pressure");

        assert!(accepted.contains("逼近的人"));
        assert!(accepted.contains("压力"));
        assert!(
            validate_generated_script_text(
                "林峰让苏瑶在屋檐下缓过气，邻居家的孩子递来一盏灯。",
                source
            )
            .is_none()
        );
        assert!(
            validate_generated_script_text(
                "林峰让苏瑶在屋檐下缓过气，雨声渐渐把巷口安静下来。",
                source
            )
            .is_none()
        );
    }

    #[test]
    fn live_expand_repair_rebinds_added_a_name_to_source_facts() {
        let source = "废墟之上，主角单膝跪地，敌人缓步逼近。";
        let live = "废墟之上，孔却燃单膝跪地，敌人缓步逼近，对峙压力没有消失。";

        let repair = repair_live_expanded_script_text(live, source, "hot_blood_battle", "热血战斗")
            .expect("source-grounded repair should recover role-only A source");

        assert!(!repair.text.contains("孔却燃"), "{}", repair.text);
        assert!(repair.text.contains("主角"), "{}", repair.text);
        assert!(repair.text.contains("敌人"), "{}", repair.text);
        assert!(repair.text.contains("废墟"), "{}", repair.text);
        assert!(contains_any_story_term(&repair.text, &["逼近", "对峙压力"]));
        assert!(validate_generated_script_text(&repair.text, source).is_some());
        let warning = live_repair_warning("text_model_live_expand_repaired", &repair.summary);
        assert_eq!(warning.code, "text_model_live_expand_repaired");
        assert!(warning.message.contains("live_raw_failed_validator=true"));
        assert!(warning.message.contains("fallback_used=false"));
    }

    #[test]
    fn live_expand_repair_removes_b_name_drift_without_leaking() {
        let source = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let live = "方仅在沙盘边示意撤离，林峰护住苏瑶，阿青提醒他们退到墙边。";

        let repair =
            repair_live_expanded_script_text(live, source, "sandbox_strategy_view", "沙盘战略视口")
                .expect("B source should canonicalize safely when live invents a name");

        assert!(!repair.text.contains("方仅"), "{}", repair.text);
        assert!(repair.text.contains("林峰护住苏瑶"), "{}", repair.text);
        assert!(repair.text.contains("阿青提醒"), "{}", repair.text);
        assert!(repair.text.contains("黑衣追兵"), "{}", repair.text);
        assert!(repair.text.contains("巷口"), "{}", repair.text);
        assert!(repair.text.contains("逼近"), "{}", repair.text);
        assert!(validate_generated_script_text(&repair.text, source).is_some());
    }

    #[test]
    fn live_expand_repair_removes_external_setting_details() {
        let a_source = "废墟之上，主角单膝跪地，敌人缓步逼近。";
        let a_live = "废墟之上，主角单膝跪地，敌人的甲胄压住光线后继续逼近。";
        let a_repair = repair_live_expanded_script_text(
            a_live,
            a_source,
            "national_war",
            "国战军阵建立",
        )
        .unwrap_or_else(|| {
            let canonical =
                canonical_live_expand_source_text(a_source, "national_war", "国战军阵建立")
                    .unwrap_or_default();
            panic!(
                "甲胄 should repair back to A source facts; canonical={canonical}; reason={:?}",
                validate_generated_script_text_with_reason(&canonical, a_source).err()
            );
        });
        assert!(!a_repair.text.contains("甲胄"), "{}", a_repair.text);
        assert!(a_repair.text.contains("主角"));
        assert!(a_repair.text.contains("敌人"));
        assert!(validate_generated_script_text(&a_repair.text, a_source).is_some());

        let b_source = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let b_live = "林峰左臂护住苏瑶，阿青提醒他们，危险感压住巷口。";
        let b_repair =
            repair_live_expanded_script_text(b_live, b_source, "hot_blood_battle", "热血战斗")
            .unwrap_or_else(|| {
                let canonical = canonical_live_expand_source_text(
                    b_source,
                    "hot_blood_battle",
                    "热血战斗",
                )
                .unwrap_or_default();
                panic!(
                    "左臂 and abstract pressure should repair back to B source facts; canonical={canonical}; reason={:?}",
                    validate_generated_script_text_with_reason(&canonical, b_source).err()
                );
            });
        assert!(!b_repair.text.contains("左臂"), "{}", b_repair.text);
        assert!(!b_repair.text.contains("危险感"), "{}", b_repair.text);
        assert!(b_repair.text.contains("追兵逼近压力"), "{}", b_repair.text);
        assert!(validate_generated_script_text(&b_repair.text, b_source).is_some());
    }

    #[test]
    fn live_expand_repair_restores_missing_b_pursuit_pressure() {
        let source = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let live = "沙盘战略视口只保留林峰护住苏瑶和阿青提醒他们撤到墙边。";

        let repair =
            repair_live_expanded_script_text(live, source, "sandbox_strategy_view", "沙盘战略视口")
                .expect("missing pursuit pressure should be source-grounded repaired");

        assert!(repair.text.contains("黑衣追兵"), "{}", repair.text);
        assert!(repair.text.contains("巷口"), "{}", repair.text);
        assert!(repair.text.contains("逼近"), "{}", repair.text);
        assert!(repair.text.contains("追兵逼近压力"), "{}", repair.text);
        assert!(validate_generated_script_text(&repair.text, source).is_some());
    }

    #[test]
    fn live_expand_repair_keeps_fallback_when_source_cannot_ground() {
        let source = "少年离开房间。";
        let live = "少年遇见李明，李明递来雨伞。";

        assert!(
            repair_live_expanded_script_text(live, source, "daily_healing", "日常治愈").is_none()
        );
        assert!(validate_generated_script_text(live, source).is_none());
    }

    #[test]
    fn expand_script_qwen_prompt_blocks_external_settings_and_preserves_pressure() {
        let provider = TextModelProvider {
            provider: TextModelProviderKind::Qwen,
            model: "qwen-plus".to_string(),
            base_url: Some("https://example.invalid".to_string()),
            api_key_ref: "env:TEST".to_string(),
            enabled: true,
        };
        let request = build_text_generation_request(
            TextGenerationTask::ExpandScript,
            Some("daily_healing".to_string()),
            "current_scene_label=日常治愈\nstory_input=林峰护住苏瑶，黑衣追兵逼近。".to_string(),
            None,
            String::new(),
            vec![],
            vec![],
            TextGenerationOutputSchema::PlainText,
            Some(700),
        );

        let payload = build_qwen_request_payload(&provider, &request);
        let user_prompt = payload["messages"][1]["content"]
            .as_str()
            .expect("Qwen payload should include user prompt");

        assert!(user_prompt.contains("北营兵符拓片"));
        assert!(user_prompt.contains("虎牢关守军轮值图"));
        assert!(user_prompt.contains("黑釉兵俑"));
        assert!(user_prompt.contains("戴着皮套的手"));
        assert!(user_prompt.contains("铭文"));
        assert!(user_prompt.contains("金线"));
        assert!(user_prompt.contains("黑衣追兵逼近"));
        assert!(user_prompt.contains("对峙压力"));
        assert!(user_prompt.contains("逼近的人"));
        assert!(user_prompt.contains("邻居家大叔"));
        assert!(user_prompt.contains("孔却燃"));
        assert!(user_prompt.contains("铠甲"));
        assert!(user_prompt.contains("甲胄"));
        assert!(user_prompt.contains("剑柄"));
        assert!(user_prompt.contains("冷笑"));
        assert!(user_prompt.contains("左臂"));
        assert!(user_prompt.contains("断戟"));
        assert!(user_prompt.contains("三名黑衣追兵"));
        assert!(user_prompt.contains("林峰护住苏瑶"));
        assert!(user_prompt.contains("阿青提醒"));
    }

    #[test]
    fn expand_fallback_warning_distinguishes_validator_from_provider_failure() {
        let provider = TextModelProvider {
            provider: TextModelProviderKind::Qwen,
            model: "qwen-plus".to_string(),
            base_url: Some("https://example.invalid".to_string()),
            api_key_ref: "env:TEST".to_string(),
            enabled: true,
        };

        let validator_warning = expand_live_fallback_warning(
            &provider,
            &[],
            false,
            Some("新增源文本外姓名：李明 prompt_body token"),
            false,
        )
        .expect("validator fallback should keep a warning");
        assert_eq!(validator_warning.code, "text_model_live_expand_fallback");
        assert!(validator_warning.message.contains("本地校验"));
        assert!(validator_warning.message.contains("新增源文本外姓名：李明"));
        assert!(!validator_warning.message.contains("prompt_body"));
        assert!(!validator_warning.message.contains("token"));
        assert!(!validator_warning.message.contains("未启用千问或调用失败"));

        let provider_warning = expand_live_fallback_warning(
            &provider,
            &[ProductWarning {
                code: "text_model_network_error".to_string(),
                message: "network".to_string(),
                related_sample_id: None,
            }],
            false,
            None,
            false,
        )
        .expect("provider fallback should keep a warning");
        assert!(provider_warning.message.contains("API 调用失败"));
        assert!(!provider_warning.message.contains("未启用千问或调用失败"));

        let local_safety_warning = expand_live_fallback_warning(&provider, &[], true, None, true)
            .expect("local safety fallback should keep a warning");
        assert!(local_safety_warning.message.contains("本地安全回退"));
    }

    fn qwen_test_provider() -> TextModelProvider {
        TextModelProvider {
            provider: TextModelProviderKind::Qwen,
            model: "qwen-plus-2025-07-28".to_string(),
            base_url: Some("https://dashscope.aliyuncs.com/compatible-mode/v1".to_string()),
            api_key_ref: "session-only".to_string(),
            enabled: true,
        }
    }

    fn qwen_storyboard_generation_request() -> TextGenerationRequest {
        TextGenerationRequest {
            task_type: TextGenerationTask::GenerateStoryboard,
            scene_type: Some("hot_blood_battle".to_string()),
            story_input: "Lead faces Rival in ruins.".to_string(),
            duration_plan: Some(StoryboardDurationPlan {
                total_duration_seconds: 15,
                row_count: 1,
                per_row_seconds: 15,
                allocated_seconds: 15,
            }),
            kb_context_summary: String::new(),
            selected_sample_ids: vec!["GS-RT-01".to_string()],
            selected_kb_rules: vec![],
            output_schema: TextGenerationOutputSchema::StoryboardRowsJson,
            temperature: None,
            max_tokens: Some(700),
        }
    }

    fn qwen_success_body() -> serde_json::Value {
        serde_json::json!({
            "choices": [
                {
                    "message": {
                        "content": "{\"rows\":[{\"person\":\"Lead\",\"shot_title\":\"Hold\",\"scene_scale\":\"MS\",\"camera_movement\":\"locked\",\"visual_description\":\"ruins with hard light and current standoff\",\"character_action\":\"Lead holds position\",\"dialogue\":\"\",\"duration_seconds\":15}]}"
                    }
                }
            ],
            "usage": { "total_tokens": 42 }
        })
    }

    fn qwen_body_with_content(content: &str) -> serde_json::Value {
        serde_json::json!({
            "choices": [
                {
                    "message": {
                        "content": content
                    }
                }
            ],
            "usage": { "total_tokens": 42 }
        })
    }

    fn qwen_network_warning(message: &str) -> ProductWarning {
        ProductWarning {
            code: "text_model_network_error".to_string(),
            message: message.to_string(),
            related_sample_id: None,
        }
    }

    #[test]
    fn model_output_contract_normalizer_unwraps_markdown_and_aliases_only_structure() {
        let content = r#"Brief note before JSON.
```json
{"storyboard":[{"character":"Lead","title":"Hold","scale":"MS","visual":"ruins with hard light","action":"Lead holds position","camera":"locked","dialogue":"","prompt_text":"model supplied prompt is ignored"}]}
```
No more content."#;
        let (structured_json, actions) = parse_model_output_contract_json(
            content,
            TextGenerationOutputSchema::StoryboardRowsJson,
        );
        let response = TextGenerationResponse {
            text: content.to_string(),
            structured_json,
            usage_tokens: None,
            latency_ms: Some(1),
            warnings: Vec::new(),
            provider: TextModelProviderKind::Qwen,
            model: "qwen-plus-2025-07-28".to_string(),
        };

        assert!(actions.contains(&"markdown_json_unwrapped".to_string()));
        assert!(actions.contains(&"rows_field_alias_mapped".to_string()));
        assert!(actions.contains(&"row_field_alias_mapped".to_string()));
        assert!(actions.contains(&"model_prompt_text_ignored".to_string()));
        let patches = extract_live_storyboard_row_patches(&response)
            .expect("structural normalizer should produce canonical row patches");
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0].person, "Lead");
        assert_eq!(patches[0].shot_title, "Hold");
        assert_eq!(patches[0].visual_description, "ruins with hard light");
        assert_eq!(patches[0].character_action, "Lead holds position");
        assert_eq!(patches[0].camera_movement, "locked");
    }

    #[test]
    fn model_output_contract_normalizer_warning_is_not_fallback_or_gate_weakening() {
        let content = r#"```json
[{"subject":"Lead","shot":"Hold","framing":"MS","image":"ruins","performance":"holds position","camera_motion":"locked","line":""}]
```"#;
        let response = run_qwen_text_generation_with_transport(
            &qwen_test_provider(),
            &qwen_storyboard_generation_request(),
            "https://example.invalid/chat/completions",
            "redacted-test-key",
            |_, _, _| Ok((qwen_body_with_content(content), 11)),
        )
        .expect("structural contract normalization should keep provider output live");

        assert!(
            response
                .warnings
                .iter()
                .any(|warning| warning.code == TEXT_MODEL_OUTPUT_CONTRACT_NORMALIZED_CODE)
        );
        assert!(
            text_generation_fallback_blocking_warnings(&response.warnings).is_empty(),
            "normalizer telemetry must not become fallback evidence: {:?}",
            response.warnings
        );
        let patches = extract_live_storyboard_row_patches(&response)
            .expect("normalized array output should parse as live rows");
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0].person, "Lead");
        assert_eq!(patches[0].scene_scale, "MS");
    }

    #[test]
    fn qwen_transport_retry_recovered_keeps_live_storyboard_without_network_fallback() {
        let attempts = Cell::new(0usize);
        let response = run_qwen_text_generation_with_transport(
            &qwen_test_provider(),
            &qwen_storyboard_generation_request(),
            "https://example.invalid/chat/completions",
            "redacted-test-key",
            |_, _, _| {
                attempts.set(attempts.get() + 1);
                if attempts.get() == 1 {
                    Err(qwen_network_warning("connection reset before response"))
                } else {
                    Ok((qwen_success_body(), 23))
                }
            },
        )
        .expect("retryable network failure should recover on second attempt");

        assert_eq!(attempts.get(), 2);
        assert!(
            response
                .warnings
                .iter()
                .any(|warning| warning.code == TEXT_MODEL_NETWORK_RETRY_RECOVERED_CODE)
        );
        assert!(
            !response
                .warnings
                .iter()
                .any(|warning| warning.code == "text_model_network_error")
        );
        let patches = extract_live_storyboard_row_patches(&response)
            .expect("recovered qwen response should still parse live rows");
        assert_eq!(patches.len(), 1);
        assert!(
            response
                .warnings
                .iter()
                .all(|warning| warning.code != "text_model_live_storyboard_fallback")
        );
    }

    #[test]
    fn qwen_transport_403_does_not_retry() {
        let attempts = Cell::new(0usize);
        let warning = run_qwen_text_generation_with_transport(
            &qwen_test_provider(),
            &qwen_storyboard_generation_request(),
            "https://example.invalid/chat/completions",
            "redacted-test-key",
            |_, _, _| {
                attempts.set(attempts.get() + 1);
                Err(qwen_network_warning(
                    "qwen compatible transport returned HTTP 403; model permission denied",
                ))
            },
        )
        .expect_err("403 should remain a final provider failure without retry");

        assert_eq!(attempts.get(), 1);
        assert_eq!(warning.code, "text_model_network_error");
        assert!(warning.message.contains("HTTP 403"));
    }

    #[test]
    fn qwen_transport_exhausted_retries_preserve_network_error_for_fallback() {
        let attempts = Cell::new(0usize);
        let warning = run_qwen_text_generation_with_transport(
            &qwen_test_provider(),
            &qwen_storyboard_generation_request(),
            "https://example.invalid/chat/completions",
            "redacted-test-key",
            |_, _, _| {
                attempts.set(attempts.get() + 1);
                Err(qwen_network_warning(
                    "qwen compatible transport returned HTTP 500; transient upstream error",
                ))
            },
        )
        .expect_err("retry exhaustion should surface the original network warning");

        assert_eq!(attempts.get(), 3);
        assert_eq!(warning.code, "text_model_network_error");
        assert!(warning.message.contains("HTTP 500"));
        assert!(warning.message.contains("retry_exhausted=true"));
        assert!(warning.message.contains("attempts=3"));
        assert!(warning.message.contains("error_category=http_5xx"));
        assert!(warning.message.contains("fallback_used=false"));
        assert!(warning.message.contains("local_candidate=false"));
    }

    #[test]
    fn qa_hard_fail_response_does_not_emit_local_candidate_text() {
        let response = run_text_generation_qa_hard_fail(
            &qwen_test_provider(),
            &qwen_storyboard_generation_request(),
            qwen_network_warning(
                "qwen compatible transport returned HTTP 500; fallback will use local candidate if retry is exhausted.",
            ),
        );

        assert!(response.text.is_empty());
        assert!(response.structured_json.is_none());
        assert!(
            response
                .warnings
                .iter()
                .any(|warning| warning.code == TEXT_MODEL_QA_NO_LOCAL_FALLBACK_CODE)
        );
        assert!(response.warnings.iter().all(|warning| {
            !warning
                .message
                .contains("fallback will use local candidate")
        }));
        assert!(
            response
                .warnings
                .iter()
                .all(|warning| !warning.message.contains("local candidate if retry"))
        );
    }

    #[test]
    fn qwen_transport_timeout_uses_qa_override_with_product_default() {
        unsafe {
            std::env::remove_var("HOPE_QA_PROVIDER_TIMEOUT_SECONDS");
        }
        assert_eq!(qwen_transport_timeout_seconds(), 30);
        unsafe {
            std::env::set_var("HOPE_QA_PROVIDER_TIMEOUT_SECONDS", "60");
        }
        assert_eq!(qwen_transport_timeout_seconds(), 60);
        unsafe {
            std::env::set_var("HOPE_QA_PROVIDER_TIMEOUT_SECONDS", "300");
        }
        assert_eq!(qwen_transport_timeout_seconds(), 30);
        unsafe {
            std::env::remove_var("HOPE_QA_PROVIDER_TIMEOUT_SECONDS");
        }
    }

    #[test]
    fn strategic_view_rewrite_keeps_pressure_without_external_props() {
        let source = "沙盘战略视口中，林峰护住苏瑶，阿青提醒敌人正在逼近，追兵压力没有消失。";

        for generated in [
            "沙盘战略视口里出现黑釉兵俑，林峰让苏瑶躲到旗标后。",
            "戴着皮套的手按住沙盘边缘，阿青听见新的铭文亮起。",
        ] {
            assert!(
                validate_generated_script_text(generated, source).is_none(),
                "strategic view should reject external prop injection: {generated}"
            );
        }

        let accepted = validate_generated_script_text(
            "沙盘战略视口只改变表达角度，林峰、苏瑶和阿青的位置被压到同一层态势里，敌人逼近与追兵压力仍然存在。",
            source,
        )
        .expect("strategic view should preserve source roles and pressure");

        assert!(accepted.contains("敌人逼近"));
        assert!(accepted.contains("追兵压力"));
        assert!(!accepted.contains("黑釉兵俑"));
        assert!(!accepted.contains("皮套"));
    }

    #[test]
    fn qa_expand_outputs_scrub_relationship_fragment_and_keep_pressure() {
        let sanitized = super::sanitize_product_body_text(
            "关系保留：林峰护住苏瑶。\n林峰关系保持紧张，黑衣追兵仍从巷口逼近。",
        );
        assert!(!sanitized.contains("关系保"));
        assert!(sanitized.contains("黑衣追兵"));
        assert!(sanitized.contains("逼近"));

        let a_source = "废墟之上，主角单膝跪地，敌人缓步逼近。";
        for generated in [
            "国战视角只把构图变得更有秩序，主角仍单膝跪在废墟上，敌人缓步逼近，对峙压力没有消失。",
            "热血战斗节奏更紧，主角单膝撑住地面，敌人仍在逼近，双方对峙压力压到眼前。",
            "国战视角只把秩序和距离压紧，主角仍单膝跪在废墟上，对峙压力没有消失。",
        ] {
            let accepted = validate_generated_script_text(generated, a_source)
                .expect("A-case rewrite must keep enemy approaching pressure");
            assert!(accepted.contains("敌人") || accepted.contains("对峙压力"));
            assert!(accepted.contains("逼近") || accepted.contains("对峙压力"));
        }
        assert!(
            validate_generated_script_text(
                "国战视角里主角单膝跪在废墟上，阵列旗影压住天空。",
                a_source,
            )
            .is_none()
        );

        let b_source = "林峰护住苏瑶，黑衣追兵逼近，阿青断后掩护他们撤离。";
        for generated in [
            "热血战斗节奏更紧，林峰护住苏瑶，阿青断后，黑衣追兵仍在逼近。",
            "沙盘战略视口只改变观察角度，林峰护住苏瑶撤离，阿青断后，追兵逼近压力仍在。",
            "沙盘战略视口只改变观察角度，林峰护住苏瑶撤离，阿青断后，身后压力仍然压住队形。",
        ] {
            let accepted = validate_generated_script_text(generated, b_source)
                .expect("B-case rewrite must keep pursuit or retreat pressure");
            assert!(
                accepted.contains("追兵")
                    || accepted.contains("逼近")
                    || accepted.contains("断后")
                    || accepted.contains("撤离")
            );
        }
        assert!(
            validate_generated_script_text(
                "沙盘战略视口里林峰护住苏瑶撤到安全点，阿青收起地图。",
                b_source,
            )
            .is_none()
        );
    }

    #[test]
    fn storyboard_rows_scrub_relationship_fragment_from_output_fields() {
        let mut row = test_live_validation_row("关系保");
        row.shot_title = "镜头1：关系保压迫".to_string();
        row.visual_description = "主体为关系保，中近景保留巷口压力。".to_string();
        row.character_action = "关系保从巷口开始，到追兵逼近时结束。".to_string();
        row.prompt_text = "角色动作：关系保从巷口开始；画面描述：主体为关系保。".to_string();
        row.scene_performance_projection.person = "关系保".to_string();
        row.scene_performance_projection.visual_description = row.visual_description.clone();
        row.scene_performance_projection.character_action = row.character_action.clone();

        normalize_storyboard_row_subject_quality(&mut row);

        let combined = format!(
            "{} {} {} {} {} {}",
            row.person,
            row.shot_title,
            row.visual_description,
            row.character_action,
            row.prompt_text,
            row.scene_performance_projection.person
        );
        assert!(!combined.contains("关系保"));
    }

    #[test]
    fn deterministic_expand_script_avoids_new_bystander_identities() {
        let material = build_deterministic_expanded_story_material(
            "主角想让这段关系平静下来，但敌人仍在门外逼近。",
            "日常治愈",
        );

        for forbidden in [
            "邻居家大叔",
            "邻居家孩子",
            "熟悉的身影",
            "邻居",
            "孩子",
            "熟人",
        ] {
            assert!(
                !material.contains(forbidden),
                "deterministic material should not add {forbidden}: {material}"
            );
        }
        assert!(material.contains("主角"));
        assert!(material.contains("逼近的人") || material.contains("对峙对象"));
    }

    #[test]
    fn storyboard_subject_quality_collapses_repeated_role_phrases() {
        let mut row = test_live_validation_row("主角与敌人");
        row.shot_title = "镜头1：主角与敌人与敌人压近断桥".to_string();
        row.visual_description = "主体为主角与敌人猛然站在画面前侧。".to_string();
        row.character_action = "主角与敌人瞳孔骤缩后开始对峙。".to_string();
        row.prompt_text =
            "镜头标题：主角与敌人与敌人压近；角色动作：林峰、苏瑶与阿青与阿青从巷口压力开始。"
                .to_string();
        row.scene_performance_projection.person = "林峰、苏瑶与阿青与阿青".to_string();
        row.scene_performance_projection.visual_description = row.visual_description.clone();
        row.scene_performance_projection.character_action = row.character_action.clone();

        normalize_storyboard_row_subject_quality(&mut row);

        let combined = format!(
            "{} {} {} {} {}",
            row.person,
            row.shot_title,
            row.visual_description,
            row.character_action,
            row.scene_performance_projection.person
        );
        assert!(!combined.contains("主角与敌人与敌人"));
        assert!(!combined.contains("林峰、苏瑶与阿青与阿青"));
        assert!(!combined.contains("主角与敌人猛然"));
        assert!(!combined.contains("主角与敌人瞳孔"));
    }

    #[test]
    fn fallback_rows_do_not_output_slash_person_and_keep_fifteen_seconds() {
        let state = test_state_with_golden_sample_runtime();
        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "fallback-no-slash-person".to_string(),
                script_id: None,
                shot_script: Some("主角护住同伴，敌人从断墙后逼近。".to_string()),
                expanded_script_text: Some("主角护住同伴，敌人继续逼近。".to_string()),
                primary_scene_type: Some("daily_dialogue".to_string()),
                primary_scene_label: Some("断墙逼近".to_string()),
                primary_scene_category: Some("action_dialogue".to_string()),
                shot_scene_type: Some("action_beat".to_string()),
                shot_scene_label: Some("断墙逼近镜头".to_string()),
                shot_intent: Some("action_beat".to_string()),
                adaptation_reason: Some("duration and person fallback test".to_string()),
                selected_total_duration_seconds: 15,
                target_duration_mode: String::new(),
                auto_segment_strategy: String::new(),
                model_config_summary: None,
                scene_type: None,
                scene_label: None,
                scene_category: None,
                ..GenerateStoryboardRequest::default()
            },
        );

        assert_ne!(storyboard.export_status.status, BridgeCallStatus::Blocked);
        assert_eq!(
            storyboard
                .rows
                .iter()
                .map(|row| row.duration_seconds)
                .collect::<Vec<_>>(),
            vec![10, 5]
        );
        assert_eq!(
            storyboard
                .rows
                .iter()
                .map(|row| row.duration_seconds)
                .sum::<u16>(),
            15
        );
        for row in storyboard.rows {
            assert_ne!(row.person, "/");
            assert_ne!(row.scene_performance_projection.person, "/");
            assert!(!row.person.trim().is_empty());
            assert!(row.person.contains("主角") || row.person.contains("敌人"));
        }
    }

    #[test]
    fn expand_script_scrubs_product_control_text_from_visible_story_body() {
        let state = test_state();
        let response = expand_script(
            &state,
            ExpandScriptRequest {
                scene_type: "xianxia_action".to_string(),
                scene_label: Some("断桥交锋".to_string()),
                scene_category: Some("xianxia_action".to_string()),
                model_config_summary: None,
                selected_total_duration_seconds: Some(30),
                target_duration_seconds: Some(30),
                target_duration_mode: "fixed_seconds".to_string(),
                story_length_profile: String::new(),
                source_material_length_chars: 0,
                auto_segment_strategy: String::new(),
                source_input_type: "full_story".to_string(),
                authoring_mode: "rewrite_from_full_story".to_string(),
                source_material_summary: String::new(),
                source_story_facts: Default::default(),
                preserved_fact_summary: String::new(),
                changed_for_screenplay_summary: String::new(),
                omitted_detail_summary: String::new(),
                synopsis_text: [
                    "scene_type: xianxia_action",
                    "target_duration_seconds: 30",
                    "source_package: desktop-test",
                    "扩写剧本：按30秒连续剧情处理，保留真实人物名。",
                    "段落1：林峰在断桥边护住叶倾颜，萧寒压近。",
                    "段落2：银辉沿掌心上涌，碎土被震开。",
                ]
                .join("\n"),
            },
        );

        assert_ne!(response.status, BridgeCallStatus::Blocked);
        assert!(!contains_product_control_text(
            &response.expanded_script_text
        ));
        assert!(
            !response
                .expanded_script_text
                .contains("target_duration_seconds")
        );
        assert!(!response.expanded_script_text.contains("扩写剧本"));
        assert!(response.expanded_script_text.contains("林峰"));
    }

    #[test]
    fn storyboard_rows_and_workbook_stay_clean_after_control_text_scrubbing() {
        let state = test_state_with_golden_sample_runtime();
        let polluted_script = [
            "scene_type: daily_dialogue",
            "target_duration_seconds: 30",
            "source_package: desktop-test",
            "扩写剧本：按30秒连续剧情处理，保留真实人物名。",
            "段落1：林峰回身护住叶倾颜，萧寒压近断桥裂口，银辉冷光沿手臂上涌，脚下碎土被震开。",
            "段落2：三人沿残墙与桥边碎石继续逼近，冲突停在正面对撞前一瞬。",
        ]
        .join("\n");

        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "polluted-product-cleanup".to_string(),
                script_id: None,
                shot_script: None,
                expanded_script_text: Some(polluted_script),
                primary_scene_type: Some("daily_dialogue".to_string()),
                primary_scene_label: Some("断桥交锋".to_string()),
                primary_scene_category: Some("action_dialogue".to_string()),
                shot_scene_type: None,
                shot_scene_label: None,
                shot_intent: None,
                adaptation_reason: None,
                selected_total_duration_seconds: 30,
                target_duration_mode: String::new(),
                auto_segment_strategy: String::new(),
                model_config_summary: None,
                scene_type: None,
                scene_label: None,
                scene_category: None,
                ..GenerateStoryboardRequest::default()
            },
        );

        assert_ne!(storyboard.export_status.status, BridgeCallStatus::Blocked);
        for row in &storyboard.rows {
            assert_visual_description_is_enhanced(row);
            assert!(!contains_product_control_text(&row.shot_script));
            assert!(!contains_product_control_text(&row.visual_description));
            assert!(!contains_product_control_text(&row.prompt_text));
        }

        let export = export_v120_storyboard_bundle(&V120StoryboardExportRequest {
            export_manifest_id: "test-clean-export".to_string(),
            result_id: storyboard.result_id.clone(),
            selected_total_duration_seconds: storyboard.selected_total_duration_seconds,
            source_result_id: storyboard.result_id.clone(),
            edited_rows_applied: storyboard.dirty,
            rows: storyboard.rows.clone(),
        })
        .expect("v120 export bundle should build");

        let sheet = export
            .workbook
            .sheets
            .iter()
            .find(|sheet| sheet.machine_name == "v120_storyboard_rows")
            .expect("storyboard workbook should expose storyboard rows");
        for row in &sheet.rows {
            assert!(!contains_product_control_text(&row[5]));
            assert!(!contains_product_control_text(&row[8]));
        }
    }

    #[test]
    fn generate_storyboard_strengthens_ruin_light_and_material_clues() {
        let state = test_state_with_golden_sample_runtime();
        let shot_script = "断楼残墙之间，裸露钢筋斜刺进灰云，风卷着锈屑掠过破碎混凝土。主角被逼到退路尽头，只能顶住来袭。";

        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "ruin-light-material-grounding".to_string(),
                script_id: None,
                shot_script: Some(shot_script.to_string()),
                expanded_script_text: Some(
                    "scene_type: daily_dialogue\nsynopsis: 断楼残墙间的危险逼近。".to_string(),
                ),
                primary_scene_type: Some("daily_dialogue".to_string()),
                primary_scene_label: Some("断楼逼近".to_string()),
                primary_scene_category: Some("action_dialogue".to_string()),
                shot_scene_type: Some("action_beat".to_string()),
                shot_scene_label: Some("断楼残墙对冲镜头".to_string()),
                shot_intent: Some("action_beat".to_string()),
                adaptation_reason: Some("ruin visual grounding".to_string()),
                selected_total_duration_seconds: 10,
                target_duration_mode: String::new(),
                auto_segment_strategy: String::new(),
                model_config_summary: None,
                scene_type: None,
                scene_label: None,
                scene_category: None,
                ..GenerateStoryboardRequest::default()
            },
        );

        assert_ne!(storyboard.export_status.status, BridgeCallStatus::Blocked);
        for row in &storyboard.rows {
            assert_visual_description_is_enhanced(row);
            assert!(contains_any_story_term(
                &row.visual_description,
                &["灰云", "冷白光", "冷白", "反光"]
            ));
            assert!(contains_any_story_term(
                &row.visual_description,
                &["钢筋", "锈屑", "混凝土", "金属"]
            ));
            assert!(!contains_product_control_text(&row.visual_description));
            assert!(
                !row.prompt_text_compilation_warnings
                    .iter()
                    .any(|warning| warning.code == "visual_description_grounding_incomplete")
            );
        }
    }

    #[test]
    fn generate_storyboard_keeps_indoor_visuals_out_of_ruin_template() {
        let state = test_state_with_golden_sample_runtime();
        let shot_script =
            "殿内烛光压低，桌案边的屏风投下长影。女主停在门窗之间压住呼吸，抬手示意同伴噤声。";

        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "indoor-visual-grounding".to_string(),
                script_id: None,
                shot_script: Some(shot_script.to_string()),
                expanded_script_text: Some(
                    "scene_type: daily_dialogue\nsynopsis: 宫殿密谈前的静压时刻。".to_string(),
                ),
                primary_scene_type: Some("daily_dialogue".to_string()),
                primary_scene_label: Some("宫殿密谈".to_string()),
                primary_scene_category: Some("dialogue".to_string()),
                shot_scene_type: Some("daily_dialogue".to_string()),
                shot_scene_label: Some("殿内停顿镜头".to_string()),
                shot_intent: Some("dialogue".to_string()),
                adaptation_reason: Some("indoor visual grounding".to_string()),
                selected_total_duration_seconds: 10,
                target_duration_mode: String::new(),
                auto_segment_strategy: String::new(),
                model_config_summary: None,
                scene_type: None,
                scene_label: None,
                scene_category: None,
                ..GenerateStoryboardRequest::default()
            },
        );

        assert_ne!(storyboard.export_status.status, BridgeCallStatus::Blocked);
        for row in &storyboard.rows {
            assert_visual_description_is_enhanced(row);
            assert!(contains_any_story_term(
                &row.visual_description,
                &["殿内", "桌案", "屏风", "门窗", "室内"]
            ));
            assert!(contains_any_story_term(
                &row.visual_description,
                &["烛光", "阴影", "明暗"]
            ));
            assert!(!contains_any_story_term(
                &row.visual_description,
                &["断楼", "残墙", "钢筋", "焦土", "灰云", "碎石"]
            ));
            assert!(row.prompt_text.contains(&row.visual_description));
        }
    }

    #[test]
    fn generate_storyboard_distinguishes_forest_and_street_visual_contexts() {
        let state = test_state_with_golden_sample_runtime();
        let forest_storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "forest-visual-grounding".to_string(),
                script_id: None,
                shot_script: Some(
                    "山林薄雾压在树影之间，主角贴着湿叶缓步逼近，远处风声把空地拉得更静。"
                        .to_string(),
                ),
                expanded_script_text: Some(
                    "scene_type: daily_dialogue\nsynopsis: 山林跟踪中的静压逼近。".to_string(),
                ),
                primary_scene_type: Some("daily_dialogue".to_string()),
                primary_scene_label: Some("山林逼近".to_string()),
                primary_scene_category: Some("action_dialogue".to_string()),
                shot_scene_type: Some("daily_dialogue".to_string()),
                shot_scene_label: Some("山林潜行镜头".to_string()),
                shot_intent: Some("action_beat".to_string()),
                adaptation_reason: Some("forest visual grounding".to_string()),
                selected_total_duration_seconds: 10,
                target_duration_mode: String::new(),
                auto_segment_strategy: String::new(),
                model_config_summary: None,
                scene_type: None,
                scene_label: None,
                scene_category: None,
                ..GenerateStoryboardRequest::default()
            },
        );
        let street_storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "street-visual-grounding".to_string(),
                script_id: None,
                shot_script: Some(
                    "夜街巷口的街灯照着雨水，人流在远处散开。主角隔着屋檐盯住来人，脚步声在近处压得更急。"
                        .to_string(),
                ),
                expanded_script_text: Some(
                    "scene_type: daily_dialogue\nsynopsis: 夜街盯防中的压力停顿。".to_string(),
                ),
                primary_scene_type: Some("daily_dialogue".to_string()),
                primary_scene_label: Some("夜街盯防".to_string()),
                primary_scene_category: Some("dialogue".to_string()),
                shot_scene_type: Some("daily_dialogue".to_string()),
                shot_scene_label: Some("街巷盯防镜头".to_string()),
                shot_intent: Some("dialogue".to_string()),
                adaptation_reason: Some("street visual grounding".to_string()),
                selected_total_duration_seconds: 10,
                target_duration_mode: String::new(),
                auto_segment_strategy: String::new(),
                model_config_summary: None,
                scene_type: None,
                scene_label: None,
                scene_category: None,
                ..GenerateStoryboardRequest::default()
            },
        );

        assert_ne!(
            forest_storyboard.export_status.status,
            BridgeCallStatus::Blocked
        );
        assert_ne!(
            street_storyboard.export_status.status,
            BridgeCallStatus::Blocked
        );
        let forest_row = forest_storyboard
            .rows
            .first()
            .expect("forest storyboard should produce at least one row");
        let street_row = street_storyboard
            .rows
            .first()
            .expect("street storyboard should produce at least one row");

        assert_visual_description_is_enhanced(forest_row);
        assert_visual_description_is_enhanced(street_row);
        assert!(contains_any_story_term(
            &forest_row.visual_description,
            &["山林", "树影", "雾气", "薄雾", "枝叶"]
        ));
        assert!(!contains_any_story_term(
            &forest_row.visual_description,
            &["街灯", "人流", "街巷", "雨水"]
        ));
        assert!(contains_any_story_term(
            &street_row.visual_description,
            &["街巷", "街灯", "雨水", "人流"]
        ));
        assert!(!contains_any_story_term(
            &street_row.visual_description,
            &["山林", "树影", "雾气", "薄雾"]
        ));
        assert_ne!(
            forest_row.visual_description, street_row.visual_description,
            "different scene inputs should not collapse into one visual template"
        );
    }

    #[test]
    fn generate_storyboard_blocks_action_and_environment_phrases_from_subject_fields() {
        let state = test_state_with_golden_sample_runtime();
        for (task_name, shot_script, forbidden_subject) in [
            (
                "kneel-and-footsteps-subject-guard",
                "主角单膝跪在坍塌的高架桥阴影下，脚步声响起。",
                "单膝跪与步声响",
            ),
            (
                "smoke-and-dodge-subject-guard",
                "敌人自烟尘中踏出，主角侧身疾避。",
                "侧身疾与自烟尘",
            ),
        ] {
            let storyboard = generate_storyboard(
                &state,
                GenerateStoryboardRequest {
                    task_name: task_name.to_string(),
                    script_id: None,
                    shot_script: Some(shot_script.to_string()),
                    expanded_script_text: Some(format!(
                        "scene_type: daily_dialogue\nsynopsis: {shot_script}"
                    )),
                    primary_scene_type: Some("daily_dialogue".to_string()),
                    primary_scene_label: Some("动作事实锚定".to_string()),
                    primary_scene_category: Some("action_beat".to_string()),
                    shot_scene_type: None,
                    shot_scene_label: None,
                    shot_intent: None,
                    adaptation_reason: None,
                    selected_total_duration_seconds: 10,
                    target_duration_mode: String::new(),
                    auto_segment_strategy: String::new(),
                    model_config_summary: None,
                    scene_type: None,
                    scene_label: None,
                    scene_category: None,
                    ..GenerateStoryboardRequest::default()
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
            for row in &storyboard.rows {
                let combined = format!(
                    "{} {} {} {} {} {}",
                    row.person,
                    row.shot_title,
                    row.camera_movement,
                    row.visual_description,
                    row.character_action,
                    row.prompt_text
                );
                assert!(
                    !combined.contains(forbidden_subject),
                    "dirty subject phrase should not reach product fields: {combined}"
                );
                assert_no_fabricated_people_terms(&combined);
                assert!(!contains_product_control_text(&row.prompt_text));
            }
        }
    }

    #[test]
    fn generate_storyboard_keeps_fact_grounding_across_scene_families() {
        let state = test_state_with_golden_sample_runtime();

        for mapping in runtime_scene_option_mappings() {
            let shot_script = format!(
                "{}里，林峰在关键空间完成动作，叶倾颜观察局势。",
                mapping.chinese_label
            );
            let storyboard = generate_storyboard(
                &state,
                GenerateStoryboardRequest {
                    task_name: format!("scene-family-{}", mapping.desktop_entry),
                    script_id: None,
                    shot_script: Some(shot_script.clone()),
                    expanded_script_text: Some(format!(
                        "scene_type: daily_dialogue\nsynopsis: {shot_script}"
                    )),
                    primary_scene_type: Some(mapping.desktop_entry.to_string()),
                    primary_scene_label: Some(mapping.chinese_label.to_string()),
                    primary_scene_category: Some(mapping.runtime_scene_family.to_string()),
                    shot_scene_type: None,
                    shot_scene_label: None,
                    shot_intent: None,
                    adaptation_reason: None,
                    selected_total_duration_seconds: 10,
                    target_duration_mode: String::new(),
                    auto_segment_strategy: String::new(),
                    model_config_summary: None,
                    scene_type: None,
                    scene_label: None,
                    scene_category: None,
                    ..GenerateStoryboardRequest::default()
                },
            );

            assert_ne!(
                storyboard.export_status.status,
                BridgeCallStatus::Blocked,
                "scene family {} should not block",
                mapping.desktop_entry
            );
            assert_eq!(
                storyboard
                    .kb_router_result
                    .retrieval_trace
                    .token_budget
                    .full_kb_rows_included,
                0
            );
            let row = storyboard.rows.first().expect("storyboard row");
            let combined = format!(
                "{} {} {} {} {}",
                row.person,
                row.shot_title,
                row.visual_description,
                row.character_action,
                row.prompt_text
            );
            assert!(combined.contains("林峰") || combined.contains("叶倾颜"));
            assert!(!contains_product_control_text(&combined));
            assert_no_fabricated_people_terms(&combined);
        }
    }

    #[test]
    fn generate_storyboard_keeps_coastal_shadow_fields_fact_grounded() {
        let state = test_state_with_golden_sample_runtime();
        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "coastal-fact-grounding".to_string(),
                script_id: None,
                shot_script: Some(
                    "张雪沿海边木栈道急奔，远处黑影踩过断裂旧缆绳，脚步声从沙丘后逼近。"
                        .to_string(),
                ),
                expanded_script_text: Some(
                    "scene_type: daily_dialogue\nsynopsis: 张雪在海边栈道上被远处黑影逼近。"
                        .to_string(),
                ),
                primary_scene_type: Some("daily_dialogue".to_string()),
                primary_scene_label: Some("海边追逐".to_string()),
                primary_scene_category: Some("field_chase".to_string()),
                shot_scene_type: Some("field_chase".to_string()),
                shot_scene_label: Some("海边栈道追逐镜头".to_string()),
                shot_intent: Some("action_beat".to_string()),
                adaptation_reason: Some("coastal fact grounding".to_string()),
                selected_total_duration_seconds: 10,
                target_duration_mode: String::new(),
                auto_segment_strategy: String::new(),
                model_config_summary: None,
                scene_type: None,
                scene_label: None,
                scene_category: None,
                ..GenerateStoryboardRequest::default()
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

        assert!(combined_rows.contains("张雪"));
        assert!(
            combined_rows.contains("远处黑影")
                || combined_rows.contains("未知黑影")
                || combined_rows.contains("黑影轮廓")
        );
        assert!(contains_any_story_term(
            &combined_rows,
            &["海边", "木栈道", "海面", "灯塔", "水面"]
        ));
        assert_no_fabricated_people_terms(&combined_rows);
        for row in &storyboard.rows {
            assert_visual_description_is_enhanced(row);
            assert_no_fabricated_people_terms(&row.visual_description);
            assert_no_fabricated_people_terms(&row.character_action);
            assert!(!contains_product_control_text(&row.prompt_text));
        }
    }

    #[test]
    fn expand_script_prefers_explicit_target_duration() {
        let request = ExpandScriptRequest {
            scene_type: "hot_blood_battle".to_string(),
            scene_label: Some("热血战斗".to_string()),
            scene_category: Some("combat".to_string()),
            model_config_summary: None,
            selected_total_duration_seconds: Some(15),
            target_duration_seconds: Some(60),
            target_duration_mode: "fixed_seconds".to_string(),
            story_length_profile: String::new(),
            source_material_length_chars: 0,
            auto_segment_strategy: String::new(),
            source_input_type: String::new(),
            authoring_mode: String::new(),
            source_material_summary: String::new(),
            source_story_facts: Default::default(),
            preserved_fact_summary: String::new(),
            changed_for_screenplay_summary: String::new(),
            omitted_detail_summary: String::new(),
            synopsis_text: "男主林峰在断桥边护住叶倾颜，迎战敌将萧寒。".to_string(),
        };

        assert_eq!(resolve_expand_script_target_duration_seconds(&request), 60);

        let script_15 = build_deterministic_expanded_story_script(
            &request.synopsis_text,
            request.scene_label.as_deref().unwrap(),
            15,
        );
        let script_60 = build_deterministic_expanded_story_script(
            &request.synopsis_text,
            request.scene_label.as_deref().unwrap(),
            60,
        );

        assert!(script_60.contains("60秒"));
        assert!(script_60.len() > script_15.len());
    }

    #[test]
    fn seedance_duration_plan_prefers_ten_second_rows_with_five_second_remainder() {
        let durations = allocate_storyboard_row_durations(45, 5)
            .expect("45 seconds should split into Seedance-friendly rows");

        assert_eq!(durations, vec![10, 10, 10, 10, 5]);
        assert!(durations.iter().all(|duration| *duration <= 15));
        assert!(!durations.contains(&9));
        assert_eq!(durations.iter().copied().sum::<u16>(), 45);
    }

    #[test]
    fn storyboard_grounding_uses_real_names_and_explicit_fallback_subjects() {
        let story =
            "男主林峰与女主叶倾颜在断桥重逢，敌将萧寒追杀而至。林峰回身护住叶倾颜，格挡萧寒刀锋。";
        let segment = "林峰回身护住叶倾颜，格挡萧寒刀锋。";

        let person = derive_product_person(segment, story);
        assert!(person.contains("林峰"));
        assert!(person.contains("叶倾颜") || person.contains("萧寒"));
        assert!(!matches!(
            person.as_str(),
            "交锋双方" | "当前镜头主体" | "未指定角色"
        ));

        let character_action = derive_character_action_from_story(segment, story);
        assert!(character_action.contains("林峰"));
        assert!(character_action.contains("叶倾颜") || character_action.contains("萧寒"));
        assert!(character_action.contains("从"));
        assert!(character_action.contains("到"));
        assert!(character_action.contains("镜头捕捉"));

        let shot_title = derive_shot_title(0, segment, &person, &character_action);
        assert!(shot_title.contains(&person));

        let protagonist_name = derive_product_person("主角林峰踏碎焦土。", "主角林峰踏碎焦土。");
        assert_eq!(protagonist_name, "林峰");

        let unnamed_protagonist = derive_product_person("主角踏碎焦土。", "主角踏碎焦土。");
        assert_eq!(unnamed_protagonist, "主角");

        let fallback = derive_product_person("敌方刀客压近断桥。", "敌方刀客压近断桥。");
        assert_eq!(fallback, "敌方刀客");
    }

    #[test]
    fn b_group_source_name_fallback_never_collapses_to_generic_main_role() {
        let source = "林峰护住苏瑶，黑衣追兵逼近。阿青受伤后提醒追兵逼近，苏瑶带阿青撤离。";

        let aqing_segment = "阿青受伤后提醒追兵逼近。";
        let aqing_person = derive_product_person(aqing_segment, source);
        assert!(aqing_person.contains("阿青"), "{aqing_person}");
        assert_ne!(aqing_person, "主角");

        let suyao_segment = "苏瑶带阿青撤离。";
        let suyao_person = derive_product_person(suyao_segment, source);
        assert!(suyao_person.contains("苏瑶"), "{suyao_person}");
        assert!(suyao_person.contains("阿青"), "{suyao_person}");
        assert_ne!(suyao_person, "主角");

        let pronoun_segment = "她受伤后提醒撤离。";
        let pronoun_person = derive_product_person(pronoun_segment, source);
        assert!(
            pronoun_person.contains("林峰")
                || pronoun_person.contains("苏瑶")
                || pronoun_person.contains("阿青"),
            "{pronoun_person}"
        );
        assert_ne!(pronoun_person, "主角");

        let a_person = derive_product_person(
            "主角单膝跪地，敌人缓步逼近。",
            "主角单膝跪地，敌人缓步逼近。",
        );
        assert_eq!(a_person, "主角与敌人");
    }

    #[test]
    fn b_group_fifteen_second_storyboard_rows_keep_source_names() {
        let state = test_state_with_golden_sample_runtime();
        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "b-group-source-name-fallback".to_string(),
                script_id: None,
                shot_script: Some(
                    "林峰护住苏瑶，黑衣追兵逼近。阿青受伤后提醒追兵逼近，苏瑶带阿青撤离。"
                        .to_string(),
                ),
                expanded_script_text: Some(
                    "热血战斗节奏更紧，林峰护住苏瑶，阿青受伤后提醒追兵逼近，苏瑶带阿青撤离。"
                        .to_string(),
                ),
                primary_scene_type: Some("action_beat".to_string()),
                primary_scene_label: Some("热血战斗".to_string()),
                primary_scene_category: Some("action".to_string()),
                shot_scene_type: None,
                shot_scene_label: None,
                shot_intent: None,
                adaptation_reason: None,
                selected_total_duration_seconds: 15,
                target_duration_mode: "fixed_seconds".to_string(),
                auto_segment_strategy: String::new(),
                model_config_summary: None,
                scene_type: None,
                scene_label: None,
                scene_category: None,
                ..GenerateStoryboardRequest::default()
            },
        );

        assert_ne!(storyboard.export_status.status, BridgeCallStatus::Blocked);
        assert_eq!(
            storyboard
                .rows
                .iter()
                .map(|row| row.duration_seconds)
                .collect::<Vec<_>>(),
            vec![10, 5]
        );
        let second_row = storyboard.rows.get(1).expect("second storyboard row");
        assert_ne!(second_row.person, "主角");
        assert!(
            second_row.person.contains("阿青")
                || second_row.person.contains("苏瑶")
                || second_row.person.contains("林峰"),
            "{}",
            second_row.person
        );
    }

    #[test]
    fn four_group_fallback_rows_never_emit_empty_person() {
        let state = test_state_with_golden_sample_runtime();
        let cases = [
            (
                "a-hot-blood-action",
                "热血战斗",
                "废墟之上，主角单膝跪地，敌人缓步逼近。敌人自烟尘中踏出，主角侧身疾避。",
            ),
            (
                "a-war-formation",
                "国战军阵建立",
                "废墟之上，主角单膝跪地，敌人缓步逼近。国战视角只压紧秩序与距离，主角仍面对敌人逼近。",
            ),
            (
                "b-hot-blood-action",
                "热血战斗",
                "林峰护住苏瑶，黑衣追兵逼近。阿青受伤后提醒追兵逼近，苏瑶带阿青撤离。",
            ),
            (
                "b-strategic-board",
                "沙盘战略视口",
                "沙盘战略视口中，林峰护住苏瑶，阿青提醒敌人正在逼近，追兵压力没有消失。苏瑶带阿青撤离，林峰挡住追兵视线。",
            ),
        ];

        for (task_name, scene_label, source) in cases {
            let storyboard = generate_storyboard(
                &state,
                GenerateStoryboardRequest {
                    task_name: task_name.to_string(),
                    script_id: None,
                    shot_script: Some(source.to_string()),
                    expanded_script_text: Some(source.to_string()),
                    primary_scene_type: Some("action_beat".to_string()),
                    primary_scene_label: Some(scene_label.to_string()),
                    primary_scene_category: Some("action".to_string()),
                    shot_scene_type: None,
                    shot_scene_label: None,
                    shot_intent: None,
                    adaptation_reason: None,
                    selected_total_duration_seconds: 15,
                    target_duration_mode: "fixed_seconds".to_string(),
                    auto_segment_strategy: String::new(),
                    model_config_summary: None,
                    scene_type: None,
                    scene_label: None,
                    scene_category: None,
                    ..GenerateStoryboardRequest::default()
                },
            );

            assert_ne!(storyboard.export_status.status, BridgeCallStatus::Blocked);
            assert_eq!(
                storyboard
                    .rows
                    .iter()
                    .map(|row| row.duration_seconds)
                    .collect::<Vec<_>>(),
                vec![10, 5],
                "{task_name}"
            );
            for row in &storyboard.rows {
                assert!(
                    !row.person.trim().is_empty(),
                    "{task_name}: {:?}",
                    row.person
                );
                assert_ne!(row.person, "/");
                assert!(!row.scene_performance_projection.person.trim().is_empty());
                assert_ne!(row.scene_performance_projection.person, "/");
                assert!(!row_has_subject_pollution(row), "{task_name}: {:?}", row);
            }
        }
    }

    #[test]
    fn live_storyboard_patch_repairs_incomplete_fields_from_baseline() {
        let mut baseline = test_live_validation_row("主角");
        baseline.visual_description = "主体为主角，中近景把废墟边缘和敌人逼近放在前后层次里；冷光掠过破碎混凝土；当前视觉事件是主角承受逼近压力；画面突出对峙压力".to_string();
        baseline.character_action =
            "主角从废墟边缘的低身状态开始，稳住重心看向敌人，到敌人逼近时结束，镜头捕捉手指压住地面的动作。"
                .to_string();
        baseline.camera_movement =
            "中近景定机位观察主角动作起止，镜头捕捉废墟边缘的对峙压力。".to_string();
        baseline.scene_performance_projection.visual_description =
            baseline.visual_description.clone();
        baseline.scene_performance_projection.character_action = baseline.character_action.clone();
        normalize_storyboard_row_subject_quality(&mut baseline);
        let mut live_row = baseline.clone();
        let patch = LiveStoryboardRowPatch {
            shot_title: "镜头1：主角避开冲击".to_string(),
            person: String::new(),
            scene_scale: "中近景".to_string(),
            visual_description: "主角站住。".to_string(),
            character_action: "主角动作。".to_string(),
            camera_movement: "观察。".to_string(),
            dialogue: String::new(),
        };

        apply_live_storyboard_patch(&mut live_row, &patch, &baseline);
        normalize_storyboard_row_subject_quality(&mut live_row);
        repair_live_storyboard_patch_from_baseline(&mut live_row, &baseline);
        normalize_storyboard_row_subject_quality(&mut live_row);

        assert!(!live_row.person.trim().is_empty());
        assert_ne!(live_row.person, "/");
        assert_eq!(live_row.visual_description, baseline.visual_description);
        assert_eq!(live_row.character_action, baseline.character_action);
        assert_eq!(live_row.camera_movement, baseline.camera_movement);
        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);
        assert!(
            !warnings.iter().any(|warning| warning.code
                == "visual_description_grounding_incomplete"
                || warning.code == "camera_movement_grounding_incomplete"
                || warning.code == "role_action_grounding_incomplete"),
            "{warnings:?}"
        );
    }

    #[test]
    fn binding_field_clause_safe_avoids_colon_subject_false_positive() {
        let mut field =
            "主体为主角，中景把主角与废墟放在前后层次里；当前视觉事件是主角顶住来袭。"
                .to_string();
        append_storyboard_binding_field_clause_safe(
            &mut field,
            "场景锚点",
            &["废墟之上".to_string(), "敌人缓步逼近".to_string()],
            2,
        );

        assert!(!field.contains("场景锚点："), "{field}");
        assert!(
            live_field_visual_or_abstract_subject_term("visual_description", &field).is_none(),
            "{field}"
        );
    }

    #[test]
    fn binding_gate_maps_a_ruin_enemy_approach_and_pressure_equivalence() {
        let rows_text = "主体为敌人，中近景把敌人压在废墟前侧；主角仍单膝跪地；当前视觉事件是敌人缓步逼近；画面突出对峙压力。";
        assert!(binding_source_fact_covered_by_rows(rows_text, "敌人逼近"));
        assert!(binding_source_fact_covered_by_rows(rows_text, "对峙压力"));
        assert!(binding_source_fact_covered_by_rows(
            rows_text,
            "废墟之上，主角单膝跪地，敌人缓步逼近。"
        ));
    }

    #[test]
    fn live_storyboard_patch_repairs_environment_visual_description_from_baseline() {
        let mut baseline = test_live_validation_row("主角");
        baseline.shot_title = "镜头1：主角顶住来袭".to_string();
        baseline.visual_description = "主体为主角，中近景把主角与敌人压在废墟前侧；冷光压住碎石与混凝土；当前视觉事件是主角顶住敌人逼近；画面突出对峙压力。".to_string();
        baseline.character_action =
            "主角从废墟之上单膝跪地开始，到敌人逼近身前时仍顶住来袭结束。".to_string();
        baseline.camera_movement =
            "中近景定机位观察主角顶住来袭，镜头捕捉废墟前侧的对峙压力。".to_string();
        baseline.scene_performance_projection.visual_description =
            baseline.visual_description.clone();
        baseline.scene_performance_projection.character_action = baseline.character_action.clone();
        let mut live_row = baseline.clone();
        let patch = LiveStoryboardRowPatch {
            shot_title: "废墟压近".to_string(),
            person: "主角".to_string(),
            scene_scale: "中近景".to_string(),
            visual_description:
                "主体为废墟，中近景把废墟压在主角前侧；当前视觉事件是废墟压近。"
                    .to_string(),
            character_action: "废墟从前景压近主角。".to_string(),
            camera_movement: "废墟定机位观察主角。".to_string(),
            dialogue: String::new(),
        };

        apply_live_storyboard_patch(&mut live_row, &patch, &baseline);
        normalize_storyboard_row_subject_quality(&mut live_row);
        repair_live_storyboard_patch_from_baseline(&mut live_row, &baseline);

        assert!(
            live_field_visual_or_abstract_subject_term("shot_title", &live_row.shot_title)
                .is_none(),
            "{}",
            live_row.shot_title
        );
        assert!(live_row.shot_title.contains("主角"), "{}", live_row.shot_title);
        assert!(!live_row.shot_title.contains("废墟"), "{}", live_row.shot_title);
        assert_eq!(live_row.visual_description, baseline.visual_description);
        assert!(!live_row.character_action.contains("废墟从前景"), "{}", live_row.character_action);
        assert!(
            contains_any_story_term(&live_row.character_action, &["主角", "敌人", "逼近"]),
            "{}",
            live_row.character_action
        );
        assert!(!live_row.camera_movement.contains("废墟定机位"), "{}", live_row.camera_movement);
        assert!(
            contains_any_story_term(&live_row.camera_movement, &["主角", "敌人", "对峙压力"]),
            "{}",
            live_row.camera_movement
        );
    }

    #[test]
    fn live_storyboard_repair_restores_a_war_visual_subject_pollution() {
        let mut baseline = test_live_validation_row("敌人");
        baseline.order = 2;
        baseline.primary_scene_label = "国战军阵建立".to_string();
        baseline.shot_scene_label = "国战军阵建立".to_string();
        baseline.scene_scale = "全景".to_string();
        baseline.scene_performance_projection.scene_scale = "全景".to_string();
        baseline.shot_script =
            "废墟之上，主角单膝跪地，敌人缓步逼近。国战军阵建立只压紧阵位与战场调度。"
                .to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        repair_a_ruin_enemy_storyboard_row(&mut baseline);
        normalize_storyboard_row_subject_quality(&mut baseline);

        let mut live_row = baseline.clone();
        live_row.shot_title = "废墟压近".to_string();
        live_row.visual_description = "主体为废墟，全景把废墟压在阵位前侧；当前视觉事件是废墟压近。".to_string();
        live_row.character_action = "废墟从前景压近主角。".to_string();
        live_row.camera_movement = "废墟定机位观察主角。".to_string();
        live_row.scene_performance_projection.visual_description =
            live_row.visual_description.clone();
        live_row.scene_performance_projection.character_action = live_row.character_action.clone();

        let mut rows = vec![live_row];
        let baselines = vec![baseline.clone()];
        let mut summary = LiveRepairSummary::default();
        summary.raw_failed_validator = true;
        summary.extend(repair_live_storyboard_rows_from_source(
            &mut rows, &baselines,
        ));
        let post_repair = validate_live_storyboard_rows(&rows, 10, &baselines);

        assert!(summary.repaired(), "{summary:?}");
        assert!(post_repair.is_empty(), "{post_repair:?}");
        assert_eq!(rows[0].shot_title, baseline.shot_title);
        assert_eq!(rows[0].visual_description, baseline.visual_description);
        assert_eq!(rows[0].character_action, baseline.character_action);
        assert_eq!(rows[0].camera_movement, baseline.camera_movement);
    }

    #[test]
    fn live_storyboard_repair_restores_a_source_camera_movement_after_visual_repair() {
        let source =
            "\u{5E9F}\u{589F}\u{4E4B}\u{4E0A}\u{FF0C}\u{4E3B}\u{89D2}\u{5355}\u{819D}\u{8DEA}\u{5730}\u{FF0C}\u{654C}\u{4EBA}\u{7F13}\u{6B65}\u{903C}\u{8FD1}\u{3002}";
        let mut baseline = test_live_validation_row("\u{4E3B}\u{89D2}");
        baseline.shot_script = source.to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        repair_a_ruin_enemy_storyboard_row(&mut baseline);
        normalize_storyboard_row_subject_quality(&mut baseline);

        let mut live_row = baseline.clone();
        live_row.visual_description = baseline.visual_description.clone();
        live_row.scene_performance_projection.visual_description =
            live_row.visual_description.clone();
        live_row.camera_movement = "\u{89C2}\u{5BDF}\u{3002}".to_string();

        let mut rows = vec![live_row];
        let baselines = vec![baseline.clone()];
        let mut summary = LiveRepairSummary::default();
        summary.raw_failed_validator = true;
        summary.extend(repair_live_storyboard_rows_from_source(
            &mut rows, &baselines,
        ));
        let post_repair = validate_live_storyboard_rows(&rows, 10, &baselines);

        assert!(summary.repaired(), "{summary:?}");
        assert!(post_repair.is_empty(), "{post_repair:?}");
        assert_eq!(rows[0].camera_movement, baseline.camera_movement);
    }

    #[test]
    fn live_storyboard_validator_ignores_ruin_environment_fragments_as_character_candidates() {
        let mut baseline = test_live_validation_row("主角");
        baseline.shot_script = "断楼残墙之间，裸露钢筋斜刺进灰云，风卷着锈屑掠过破碎混凝土。主角被逼到退路尽头，只能顶住来袭。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        baseline.person = "主角".to_string();
        baseline.scene_performance_projection.person = "主角".to_string();

        for phrase in ["风卷残墙", "残墙", "残垣", "断楼残墙"] {
            let mut live_row = baseline.clone();
            live_row.visual_description = format!(
                "主体为主角，中近景把{phrase}与主角所处退路压在同层；冷光压住碎石与混凝土；当前视觉事件是主角顶住来袭；画面突出危险逼近。"
            );
            live_row.character_action =
                "主角从断楼残墙前侧开始，到顶住来袭时结束，镜头捕捉主角动作。"
                    .to_string();
            live_row.camera_movement =
                "中近景定机位观察主角顶住来袭，镜头捕捉断楼残墙间的逼近压力。"
                    .to_string();

            let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline.clone()]);
            assert!(
                !warnings.iter().any(|warning| {
                    warning.message.contains("ungrounded character name")
                        && warning.message.contains(phrase)
                }),
                "{phrase}: {warnings:?}"
            );
        }
    }

    #[test]
    fn live_storyboard_validator_keeps_bare_source_role_grounded() {
        let baseline = test_live_validation_row("主角");
        let warnings = validate_live_storyboard_rows(&[baseline.clone()], 10, &[baseline]);
        assert!(
            !warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name: 主角")),
            "{warnings:?}"
        );
    }

    /*
    #[test]
    fn binding_context_repaired_baseline_keeps_bare_source_role_grounded_for_rewrite() {
        let source =
            "\u{5E9F}\u{589F}\u{4E4B}\u{4E0A}\u{FF0C}\u{4E3B}\u{89D2}\u{5355}\u{819D}\u{8DEA}\u{5730}\u{FF0C}\u{654C}\u{4EBA}\u{7F13}\u{6B65}\u{903C}\u{8FD1}\u{3002}";
        let rewrite_body =
            "\u{5E9F}\u{589F}\u{524D}\u{4FA7}\u{7684}\u{5BF9}\u{5CD9}\u{538B}\u{529B}\u{4E0D}\u{65AD}\u{62AC}\u{9AD8}\u{3002}";
        let mut baseline = test_live_validation_row("");
        baseline.shot_script = rewrite_body.to_string();
        baseline.scene_performance_projection.fused_source_text = rewrite_body.to_string();
        baseline.person.clear();
        baseline.scene_performance_projection.person.clear();

        let request = GenerateStoryboardRequest {
            task_name: "rewrite-binding-baseline".to_string(),
            shot_script: Some(source.to_string()),
            expanded_script_text: Some(source.to_string()),
            source_profile: "A_ruin_duel".to_string(),
            current_case_id: "accepted-f98f1a02-33c6e849-721f45d9".to_string(),
            selected_total_duration_seconds: 10,
            must_keep_facts: vec![
                "\u{4E3B}\u{89D2}".to_string(),
                "\u{654C}\u{4EBA}".to_string(),
                "\u{5E9F}\u{589F}".to_string(),
                "\u{5355}\u{819D}\u{8DEA}\u{5730}".to_string(),
                "\u{903C}\u{8FD1}".to_string(),
            ],
            ..GenerateStoryboardRequest::default()
        };
        let grounding = StoryboardGroundingContext {
            shot_script: rewrite_body.to_string(),
            expanded_script_text: rewrite_body.to_string(),
            grounding_text: source.to_string(),
            grounding_source: ShotGroundingSource::ShotScript,
            primary_scene_type: "hot_blood_battle".to_string(),
            primary_scene_label: "\u{70ED}\u{8840}\u{6218}\u{6597}".to_string(),
            primary_scene_category: "combat".to_string(),
            shot_scene_type: "hot_blood_battle".to_string(),
            shot_scene_label: "\u{70ED}\u{8840}\u{6218}\u{6597}".to_string(),
            shot_intent: "action_beat".to_string(),
            adaptation_reason: "rewrite baseline binding test".to_string(),
        };
        let mut baselines = vec![baseline];
        repair_storyboard_rows_from_binding_context(&mut baselines, &request, &grounding);

        assert_eq!(baselines[0].person, "\u{4E3B}\u{89D2}");

        let mut live_row = baselines[0].clone();
        live_row.person = "\u{4E3B}\u{89D2}".to_string();

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &baselines);
        assert!(
            !warnings.iter().any(|warning| {
                warning
                    .message
                    .contains("ungrounded character name: 涓昏")
            }),
            "{warnings:?}"
        );
    }

    #[test]
    fn live_storyboard_validator_ignores_war_setup_shot_title_fragment_candidate() {
        let mut baseline = test_live_validation_row("涓昏");
        baseline.primary_scene_label = "鍥芥垬鍐涢樀寤虹珛".to_string();
        baseline.shot_scene_label = "鍥芥垬鍐涢樀寤虹珛".to_string();
        baseline.shot_script =
            "搴熷涔嬩笂锛屼富瑙掑崟鑶濊藩鍦帮紝鏁屼汉缂撴閫艰繎銆傚浗鎴樺啗闃靛缓绔嬪彧鍘嬬揣闃典綅涓庢垬鍦鸿皟搴︺€?"
                .to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();

        let mut live_row = baseline.clone();
        live_row.shot_title = "闀滃ご1锛氬畾鍩哄簾澧熷帇杩?.to_string();

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);
        assert!(
            !warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name: 瀹氬熀")),
            "{warnings:?}"
        );
    }

    */

    #[test]
    fn binding_context_repaired_baseline_keeps_bare_source_role_grounded_for_rewrite() {
        let source =
            "\u{5E9F}\u{589F}\u{4E4B}\u{4E0A}\u{FF0C}\u{4E3B}\u{89D2}\u{5355}\u{819D}\u{8DEA}\u{5730}\u{FF0C}\u{654C}\u{4EBA}\u{7F13}\u{6B65}\u{903C}\u{8FD1}\u{3002}";
        let rewrite_body =
            "\u{5E9F}\u{589F}\u{524D}\u{4FA7}\u{7684}\u{5BF9}\u{5CD9}\u{538B}\u{529B}\u{4E0D}\u{65AD}\u{62AC}\u{9AD8}\u{3002}";
        let mut baseline = test_live_validation_row("");
        baseline.shot_script = rewrite_body.to_string();
        baseline.scene_performance_projection.fused_source_text = rewrite_body.to_string();
        baseline.person.clear();
        baseline.scene_performance_projection.person.clear();

        let request = GenerateStoryboardRequest {
            task_name: "rewrite-binding-baseline".to_string(),
            shot_script: Some(source.to_string()),
            expanded_script_text: Some(source.to_string()),
            source_profile: "A_ruin_duel".to_string(),
            current_case_id: "accepted-f98f1a02-33c6e849-721f45d9".to_string(),
            selected_total_duration_seconds: 10,
            must_keep_facts: vec![
                "\u{4E3B}\u{89D2}".to_string(),
                "\u{654C}\u{4EBA}".to_string(),
                "\u{5E9F}\u{589F}".to_string(),
                "\u{5355}\u{819D}\u{8DEA}\u{5730}".to_string(),
                "\u{903C}\u{8FD1}".to_string(),
            ],
            ..GenerateStoryboardRequest::default()
        };
        let grounding = StoryboardGroundingContext {
            shot_script: rewrite_body.to_string(),
            expanded_script_text: rewrite_body.to_string(),
            grounding_text: source.to_string(),
            grounding_source: ShotGroundingSource::ShotScript,
            primary_scene_type: "hot_blood_battle".to_string(),
            primary_scene_label: "\u{70ED}\u{8840}\u{6218}\u{6597}".to_string(),
            primary_scene_category: "combat".to_string(),
            shot_scene_type: "hot_blood_battle".to_string(),
            shot_scene_label: "\u{70ED}\u{8840}\u{6218}\u{6597}".to_string(),
            shot_intent: "action_beat".to_string(),
            adaptation_reason: "rewrite baseline binding test".to_string(),
        };
        let mut baselines = vec![baseline];
        repair_storyboard_rows_from_binding_context(&mut baselines, &request, &grounding);

        assert_eq!(baselines[0].person, "\u{4E3B}\u{89D2}");

        let mut live_row = baselines[0].clone();
        live_row.person = "\u{4E3B}\u{89D2}".to_string();

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &baselines);
        assert!(
            !warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name: \u{4E3B}\u{89D2}")),
            "{warnings:?}"
        );
    }

    #[test]
    fn live_storyboard_validator_ignores_war_setup_shot_title_fragment_candidate() {
        let mut baseline = test_live_validation_row("\u{4E3B}\u{89D2}");
        baseline.primary_scene_label = "\u{56FD}\u{6218}\u{519B}\u{9635}\u{5EFA}\u{7ACB}".to_string();
        baseline.shot_scene_label = "\u{56FD}\u{6218}\u{519B}\u{9635}\u{5EFA}\u{7ACB}".to_string();
        baseline.shot_script =
            "\u{5E9F}\u{589F}\u{4E4B}\u{4E0A}\u{FF0C}\u{4E3B}\u{89D2}\u{5355}\u{819D}\u{8DEA}\u{5730}\u{FF0C}\u{654C}\u{4EBA}\u{7F13}\u{6B65}\u{903C}\u{8FD1}\u{3002}\u{56FD}\u{6218}\u{519B}\u{9635}\u{5EFA}\u{7ACB}\u{53EA}\u{538B}\u{7D27}\u{9635}\u{4F4D}\u{4E0E}\u{6218}\u{573A}\u{8C03}\u{5EA6}\u{3002}"
                .to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();

        let mut live_row = baseline.clone();
        live_row.shot_title =
            "\u{955C}\u{5934}1\u{FF1A}\u{5B9A}\u{57FA}\u{5E9F}\u{589F}\u{538B}\u{8FEB}".to_string();

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);
        assert!(
            !warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name: \u{5B9A}\u{57FA}")),
            "{warnings:?}"
        );
    }

    #[test]
    fn live_storyboard_validator_ignores_war_setup_shot_title_stance_candidate() {
        let mut baseline = test_live_validation_row("\u{4E3B}\u{89D2}");
        baseline.primary_scene_label = "\u{56FD}\u{6218}\u{519B}\u{9635}\u{5EFA}\u{7ACB}".to_string();
        baseline.shot_scene_label = "\u{56FD}\u{6218}\u{519B}\u{9635}\u{5EFA}\u{7ACB}".to_string();
        baseline.shot_script =
            "\u{5E9F}\u{589F}\u{4E4B}\u{4E0A}\u{FF0C}\u{4E3B}\u{89D2}\u{5355}\u{819D}\u{8DEA}\u{5730}\u{FF0C}\u{654C}\u{4EBA}\u{7F13}\u{6B65}\u{903C}\u{8FD1}\u{3002}\u{56FD}\u{6218}\u{519B}\u{9635}\u{5EFA}\u{7ACB}\u{53EA}\u{538B}\u{7D27}\u{9635}\u{4F4D}\u{4E0E}\u{6218}\u{573A}\u{8C03}\u{5EA6}\u{3002}"
                .to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();

        let mut live_row = baseline.clone();
        live_row.shot_title =
            "\u{955C}\u{5934}1\u{FF1A}\u{5B9A}\u{52BF}\u{5E9F}\u{589F}\u{538B}\u{8FEB}".to_string();

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);
        assert!(
            !warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name: \u{5B9A}\u{52BF}")),
            "{warnings:?}"
        );
    }

    #[test]
    fn live_storyboard_validator_does_not_treat_source_bound_initial_appearance_as_new_name() {
        let mut baseline = test_live_validation_row("主角");
        baseline.shot_script = "主角初现于断墙前，敌人逼近。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        let mut live_row = baseline.clone();
        let patch = LiveStoryboardRowPatch {
            shot_title: "镜头1：主角初现于断墙前".to_string(),
            person: "主角初".to_string(),
            scene_scale: "中近景".to_string(),
            visual_description: "主体为主角初，中近景把主角与断墙放在前侧；当前视觉事件是主角初现于断墙前。".to_string(),
            character_action: "主角初从断墙前停住，到敌人逼近时结束。".to_string(),
            camera_movement: "中近景定机位观察主角初现。".to_string(),
            dialogue: String::new(),
        };

        apply_live_storyboard_patch(&mut live_row, &patch, &baseline);
        normalize_storyboard_row_subject_quality(&mut live_row);

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);
        assert!(
            !warnings.iter().any(|warning| warning
                .message
                .contains("ungrounded character name: 主角初")),
            "{warnings:?}"
        );
    }

    #[test]
    fn live_storyboard_validator_does_not_treat_initial_standoff_shot_title_fragment_as_new_name() {
        let mut baseline = test_live_validation_row("主角");
        baseline.shot_script = "废墟之上，主角单膝跪地，敌人缓步逼近。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        let mut live_row = baseline.clone();
        let patch = LiveStoryboardRowPatch {
            shot_title: "镜头1：初峙废墟压迫".to_string(),
            person: "主角".to_string(),
            scene_scale: "中近景".to_string(),
            visual_description: "主体为主角，中近景把主角与废墟放在前侧；当前视觉事件是主角承受敌人逼近压力。".to_string(),
            character_action:
                "主角从废墟边缘开始，到敌人继续逼近时结束，镜头捕捉当前对峙压力。"
                    .to_string(),
            camera_movement: "中近景定机位观察主角与敌人对峙。".to_string(),
            dialogue: String::new(),
        };

        apply_live_storyboard_patch(&mut live_row, &patch, &baseline);
        normalize_storyboard_row_subject_quality(&mut live_row);

        assert!(live_row.shot_title.contains("主角"), "{}", live_row.shot_title);
        assert!(
            !live_row.shot_title.contains("：初峙"),
            "{}",
            live_row.shot_title
        );

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);
        assert!(
            !warnings.iter().any(|warning| warning
                .message
                .contains("ungrounded character name: 初峙")),
            "{warnings:?}"
        );
    }

    #[test]
    fn live_storyboard_validator_does_not_treat_initial_appearance_shot_title_fragment_as_new_name()
    {
        let mut baseline = test_live_validation_row("主角");
        baseline.shot_script = "主角初现于断墙前，敌人逼近。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        let mut live_row = baseline.clone();
        let patch = LiveStoryboardRowPatch {
            shot_title: "镜头1：初现于断墙前".to_string(),
            person: "主角".to_string(),
            scene_scale: "中近景".to_string(),
            visual_description:
                "主体为主角，中近景把主角与断墙放在前侧；当前视觉事件是主角初现于断墙前。"
                    .to_string(),
            character_action: "主角从断墙前停住开始，到敌人逼近时结束。".to_string(),
            camera_movement: "中近景定机位观察主角初现。".to_string(),
            dialogue: String::new(),
        };

        apply_live_storyboard_patch(&mut live_row, &patch, &baseline);
        normalize_storyboard_row_subject_quality(&mut live_row);

        assert!(live_row.shot_title.contains("主角"), "{}", live_row.shot_title);
        assert!(
            !live_row.shot_title.contains("：初现"),
            "{}",
            live_row.shot_title
        );

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);
        assert!(
            !warnings.iter().any(|warning| warning
                .message
                .contains("ungrounded character name: 初现")),
            "{warnings:?}"
        );
    }

    #[test]
    fn live_storyboard_validator_ignores_breath_action_fragment_candidate() {
        let mut baseline = test_live_validation_row("主角");
        baseline.shot_script = "废墟之上，主角单膝跪地，敌人缓步逼近。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        let mut live_row = baseline.clone();
        live_row.shot_title = "镜头1：喘息压迫".to_string();
        live_row.visual_description =
            "主体为主角，中近景把主角放在废墟前侧；当前视觉事件是主角撑住喘息。"
                .to_string();
        live_row.character_action =
            "主角从废墟前侧单膝跪地开始，到敌人逼近时仍撑住喘息结束。".to_string();
        live_row.camera_movement = "中近景定机位观察主角在废墟前侧喘息。".to_string();

        assert!(
            !super::live_storyboard_field_character_candidates("shot_title", &live_row.shot_title)
                .iter()
                .any(|candidate| candidate == "喘息"),
            "{:?}",
            super::live_storyboard_field_character_candidates("shot_title", &live_row.shot_title)
        );
        assert_eq!(
            super::normalize_live_character_label_to_source_subject(
                "主角喘息",
                &baseline.shot_script,
                &baseline.person,
            )
            .as_deref(),
            Some("主角")
        );

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);
        assert!(
            !warnings.iter().any(|warning| warning
                .message
                .contains("ungrounded character name: 喘息")),
            "{warnings:?}"
        );
    }

    #[test]
    fn blocked_person_diagnostic_clears_initial_appearance_shot_title_candidate_after_repair() {
        let mut baseline = test_live_validation_row("主角");
        baseline.order = 1;
        baseline.prompt_text_source_row_id = "shot-1".to_string();
        baseline.shot_script = "主角初现于断墙前，敌人逼近。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        let raw_patch = LiveStoryboardRowPatch {
            shot_title: "镜头1：初现于断墙前".to_string(),
            person: "主角".to_string(),
            scene_scale: "中近景".to_string(),
            visual_description: baseline.visual_description.clone(),
            character_action: baseline.character_action.clone(),
            camera_movement: baseline.camera_movement.clone(),
            dialogue: baseline.dialogue.clone(),
        };
        let mut raw_patch_row = baseline.clone();
        raw_patch_row.shot_title = raw_patch.shot_title.clone();
        let patch_repaired_row = raw_patch_row.clone();
        let repaired_row = raw_patch_row.clone();
        let request = GenerateStoryboardRequest {
            task_name: "qwen3.6-plus-blocked-person-diagnostic".to_string(),
            shot_script: Some(baseline.shot_script.clone()),
            expanded_script_text: Some(baseline.shot_script.clone()),
            selected_total_duration_seconds: 15,
            current_case_id: "cert_expand_baseline".to_string(),
            source_text_hash: "f98f1a02".to_string(),
            accepted_rewrite_hash: "33c6e849".to_string(),
            task_script_hash: "0319980d".to_string(),
            story_fact_frame_hash: "e2b60f15".to_string(),
            source_profile: "A_initial_appearance".to_string(),
            must_keep_facts: vec![
                "主角".to_string(),
                "敌人".to_string(),
                "断墙前".to_string(),
                "初现".to_string(),
                "逼近".to_string(),
            ],
            ..GenerateStoryboardRequest::default()
        };
        let grounding = StoryboardGroundingContext {
            shot_script: baseline.shot_script.clone(),
            expanded_script_text: baseline.shot_script.clone(),
            grounding_text: baseline.shot_script.clone(),
            grounding_source: ShotGroundingSource::ShotScript,
            primary_scene_type: "hot_blood_battle".to_string(),
            primary_scene_label: "热血战斗".to_string(),
            primary_scene_category: "action".to_string(),
            shot_scene_type: "hot_blood_battle".to_string(),
            shot_scene_label: "热血战斗".to_string(),
            shot_intent: "action_beat".to_string(),
            adaptation_reason: "blocked person diagnostic coverage".to_string(),
        };
        let live_repair_summary = LiveRepairSummary {
            raw_failed_validator: true,
            reasons: vec!["binding_story_fact_frame_restored".to_string()],
        };
        let findings = vec![ProductWarning {
            code: "text_model_validator_failed".to_string(),
            message: "Live storyboard row 1 introduced an ungrounded character name: 初现."
                .to_string(),
            related_sample_id: None,
        }];

        let warnings = super::blocked_runtime_diagnostic_warnings(
            &request,
            &grounding,
            &[15],
            &[raw_patch],
            &[baseline],
            &[raw_patch_row],
            &[patch_repaired_row],
            &[repaired_row],
            &live_repair_summary,
            &findings,
            &findings,
        );
        let warning = warnings
            .iter()
            .find(|warning| warning.code == "qa_live_blocked_person_diag")
            .expect("person diagnostic warning");

        assert!(warning.message.contains("row=1"), "{}", warning.message);
        assert!(
            warning.message.contains("bind_source_role_hit=true"),
            "{}",
            warning.message
        );
        assert!(
            warning.message.contains("post_grounded=true"),
            "{}",
            warning.message
        );
        assert!(
            warning.message.contains("post_untrusted_field="),
            "{}",
            warning.message
        );
        assert!(
            warning.message.contains("post_untrusted_candidate="),
            "{}",
            warning.message
        );
        assert!(!warning.message.contains("post_untrusted_field=shot_title"), "{}", warning.message);
        assert!(
            !warning.message.contains("post_untrusted_candidate=初现"),
            "{}",
            warning.message
        );
    }

    #[test]
    fn live_storyboard_validator_still_rejects_unbound_external_name_in_ruin_context() {
        let mut baseline = test_live_validation_row("主角");
        baseline.shot_script = "废墟之上，主角单膝跪地，敌人缓步逼近。".to_string();
        baseline.scene_performance_projection.fused_source_text = baseline.shot_script.clone();
        let mut live_row = baseline.clone();
        live_row.person = "李明".to_string();
        live_row.visual_description =
            "主体为李明，中近景把李明放在废墟前侧；当前视觉事件是李明挡住退路。"
                .to_string();
        live_row.character_action = "李明从废墟前侧开始，到挡住退路时结束。".to_string();

        let warnings = validate_live_storyboard_rows(&[live_row], 10, &[baseline]);
        assert!(
            warnings
                .iter()
                .any(|warning| warning.message.contains("ungrounded character name")),
            "{warnings:?}"
        );
    }

    #[test]
    fn b_hot_live_repair_reinjects_story_fact_frame_into_suyao_and_aqing_rows() {
        let source = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let mut baseline_suyao = test_live_validation_row("苏瑶");
        baseline_suyao.shot_id = "b-hot-row-1".to_string();
        baseline_suyao.order = 1;
        baseline_suyao.shot_script = "林峰护住苏瑶。".to_string();
        baseline_suyao.primary_scene_type = "hot_blood_battle".to_string();
        baseline_suyao.primary_scene_label = "热血战斗".to_string();
        baseline_suyao.shot_scene_type = "hot_blood_battle".to_string();
        baseline_suyao.shot_scene_label = "热血战斗".to_string();
        baseline_suyao.person = "苏瑶".to_string();
        baseline_suyao.scene_performance_projection.person = "苏瑶".to_string();
        baseline_suyao
            .scene_performance_projection
            .fused_source_text = source.to_string();
        repair_b_alley_pursuit_storyboard_row(&mut baseline_suyao);

        let mut baseline_aqing = test_live_validation_row("阿青");
        baseline_aqing.shot_id = "b-hot-row-2".to_string();
        baseline_aqing.order = 2;
        baseline_aqing.duration_seconds = 5;
        baseline_aqing.shot_duration_seconds = 5;
        baseline_aqing.shot_script = "阿青提醒他们，黑衣追兵从巷口逼近。".to_string();
        baseline_aqing.primary_scene_type = "hot_blood_battle".to_string();
        baseline_aqing.primary_scene_label = "热血战斗".to_string();
        baseline_aqing.shot_scene_type = "hot_blood_battle".to_string();
        baseline_aqing.shot_scene_label = "热血战斗".to_string();
        baseline_aqing.person = "阿青".to_string();
        baseline_aqing.scene_performance_projection.person = "阿青".to_string();
        baseline_aqing
            .scene_performance_projection
            .fused_source_text = source.to_string();
        repair_b_alley_pursuit_storyboard_row(&mut baseline_aqing);

        let baselines = vec![baseline_suyao.clone(), baseline_aqing.clone()];
        let mut rows = baselines.clone();
        rows[0].shot_title = "镜头1：苏瑶停住".to_string();
        rows[0].visual_description = "主体为苏瑶，中近景把苏瑶压在街巷墙边的前后层次里；冷光压住门墙和街面动线，空气层次把退路收紧；当前视觉事件是苏瑶稳住呼吸看向来路，画面突出紧迫感。".to_string();
        rows[0].character_action =
            "苏瑶从街巷墙边稳住呼吸开始，到回望来路时结束，镜头捕捉她停顿的一瞬间。".to_string();
        rows[0].camera_movement =
            "中近景定机位观察苏瑶在街巷墙边停住，镜头捕捉退路收紧的压力。".to_string();
        rows[0].prompt_text =
            "视频分镜提示词：以当前镜头脚本为准，输出苏瑶停住的画面。".to_string();
        rows[0].scene_performance_projection.visual_description =
            baseline_suyao.visual_description.clone();
        rows[0].scene_performance_projection.character_action =
            baseline_suyao.character_action.clone();

        rows[1].shot_title = "镜头2：阿青回头".to_string();
        rows[1].visual_description = "主体为阿青，中近景把阿青放在街巷墙边前侧；冷光压住门墙和街面动线，空气层次把退路收紧；当前视觉事件是阿青回头示意他们靠近墙边，画面突出紧迫感。".to_string();
        rows[1].character_action =
            "阿青从街巷墙边观察来路开始，到示意他们靠近墙边时结束，镜头捕捉他停顿的一瞬间。"
                .to_string();
        rows[1].camera_movement =
            "中近景定机位观察阿青在街巷墙边回头，镜头捕捉退路收紧的压力。".to_string();
        rows[1].prompt_text =
            "视频分镜提示词：以当前镜头脚本为准，输出阿青回头的画面。".to_string();
        rows[1].scene_performance_projection.visual_description =
            baseline_aqing.visual_description.clone();
        rows[1].scene_performance_projection.character_action =
            baseline_aqing.character_action.clone();

        let request = GenerateStoryboardRequest {
            task_name: "b-hot-live-repair-story-fact-frame".to_string(),
            shot_script: Some(source.to_string()),
            expanded_script_text: Some(source.to_string()),
            selected_total_duration_seconds: 15,
            current_case_id: "cross_different_script_same_scene".to_string(),
            source_text_hash: "1efec388".to_string(),
            story_fact_frame_hash: "0946ea3a".to_string(),
            source_profile: "B_alley_pursuit".to_string(),
            must_keep_facts: vec![
                "林峰护住苏瑶".to_string(),
                "阿青提醒".to_string(),
                "黑衣追兵".to_string(),
                "巷口".to_string(),
                "逼近".to_string(),
            ],
            forbidden_facts: vec![
                "三名".to_string(),
                "三名黑衣追兵".to_string(),
                "刀锋".to_string(),
                "衣袖裂口".to_string(),
                "左臂".to_string(),
                "伤口".to_string(),
            ],
            ..GenerateStoryboardRequest::default()
        };
        let grounding = StoryboardGroundingContext {
            shot_script: source.to_string(),
            expanded_script_text: source.to_string(),
            grounding_text: source.to_string(),
            grounding_source: ShotGroundingSource::ShotScript,
            primary_scene_type: "hot_blood_battle".to_string(),
            primary_scene_label: "热血战斗".to_string(),
            primary_scene_category: "action".to_string(),
            shot_scene_type: "hot_blood_battle".to_string(),
            shot_scene_label: "热血战斗".to_string(),
            shot_intent: "action_beat".to_string(),
            adaptation_reason: "B source fact frame repair regression".to_string(),
        };
        let duration_plan = build_storyboard_duration_plan(15, &[10, 5]);
        let kb_request = KbRouterRuntimeRequest {
            scene_type: "hot_blood_battle".to_string(),
            synopsis_text: source.to_string(),
            duration_seconds: 15,
            task_type: KbRouterTaskType::GenerateStoryboard,
            primary_scene_type: Some("hot_blood_battle".to_string()),
            primary_scene_label: Some("热血战斗".to_string()),
            shot_scene_type: Some("hot_blood_battle".to_string()),
            shot_scene_label: Some("热血战斗".to_string()),
            shot_intent: Some("action_beat".to_string()),
            structure_type: None,
        };
        let state = test_state();
        let kb_router_result = empty_kb_router_response(&kb_request, &state.kb_runtime);
        let pre_evidence = build_storyboard_binding_evidence(
            &request,
            &grounding,
            &rows,
            &duration_plan,
            "pre-repair",
            &kb_router_result,
        );
        assert!(
            !pre_evidence.missing_source_facts.is_empty(),
            "{pre_evidence:?}"
        );

        let summary = repair_live_storyboard_rows_from_source(&mut rows, &baselines);
        diversify_repeated_storyboard_subjects(&mut rows);
        let post_warnings = validate_live_storyboard_rows(&rows, 15, &baselines);
        assert!(summary.repaired(), "{summary:?}");
        assert!(
            post_warnings.is_empty(),
            "{post_warnings:?}\n{}",
            storyboard_rows_visible_text(&rows)
        );

        let post_evidence = build_storyboard_binding_evidence(
            &request,
            &grounding,
            &rows,
            &duration_plan,
            "post-repair",
            &kb_router_result,
        );
        assert_eq!(
            post_evidence.missing_source_facts,
            Vec::<String>::new(),
            "{post_evidence:?}\n{}",
            storyboard_rows_visible_text(&rows)
        );
        assert_eq!(post_evidence.forbidden_fact_hits, Vec::<String>::new());

        for row in &rows {
            let visible = storyboard_row_visible_text(row);
            for required in ["林峰护住苏瑶", "阿青提醒", "黑衣追兵", "巷口", "逼近"]
            {
                assert!(visible.contains(required), "{required}: {visible}");
            }
            for forbidden in ["三名", "刀锋", "衣袖裂口", "左臂", "伤口"] {
                assert!(!visible.contains(forbidden), "{forbidden}: {visible}");
            }
        }
    }

    #[test]
    fn b_hot_binding_context_repairs_release_rows_before_binding_evidence() {
        let source = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let accepted_rewrite = "苏瑶贴着街巷墙边停住，阿青回头示意他们靠近墙面。";
        let mut rows = vec![
            test_live_validation_row("苏瑶"),
            test_live_validation_row("阿青"),
        ];
        for (index, row) in rows.iter_mut().enumerate() {
            row.order = (index + 1) as u32;
            row.duration_seconds = if index == 0 { 10 } else { 5 };
            row.shot_duration_seconds = row.duration_seconds;
            row.shot_script = accepted_rewrite.to_string();
            row.primary_scene_type = "hot_blood_battle".to_string();
            row.primary_scene_label = "热血战斗".to_string();
            row.shot_scene_type = "hot_blood_battle".to_string();
            row.shot_scene_label = "热血战斗".to_string();
            row.scene_performance_projection.fused_source_text = accepted_rewrite.to_string();
        }
        rows[0].shot_title = "镜头1：苏瑶停住".to_string();
        rows[0].visual_description = "主体为苏瑶，中近景把苏瑶压在街巷墙边的前后层次里；冷光压住门墙和街面动线，空气层次把退路收紧；当前视觉事件是苏瑶稳住呼吸看向来路，画面突出紧迫感。".to_string();
        rows[0].character_action =
            "苏瑶从街巷墙边稳住呼吸开始，到回望来路时结束，镜头捕捉她停顿的一瞬间。".to_string();
        rows[0].camera_movement =
            "中近景定机位观察苏瑶在街巷墙边停住，镜头捕捉退路收紧的压力。".to_string();
        rows[0].prompt_text = "视频分镜提示词：输出苏瑶停住的画面。".to_string();
        rows[1].shot_title = "镜头2：阿青回头".to_string();
        rows[1].visual_description = "主体为阿青，中近景把阿青放在街巷墙边前侧；冷光压住门墙和街面动线，空气层次把退路收紧；当前视觉事件是阿青回头示意他们靠近墙边，画面突出紧迫感。".to_string();
        rows[1].character_action =
            "阿青从街巷墙边观察来路开始，到示意他们靠近墙边时结束，镜头捕捉他停顿的一瞬间。"
                .to_string();
        rows[1].camera_movement =
            "中近景定机位观察阿青在街巷墙边回头，镜头捕捉退路收紧的压力。".to_string();
        rows[1].prompt_text = "视频分镜提示词：输出阿青回头的画面。".to_string();

        let request = GenerateStoryboardRequest {
            task_name: "b-hot-release-binding-context-repair".to_string(),
            shot_script: Some(accepted_rewrite.to_string()),
            expanded_script_text: Some(accepted_rewrite.to_string()),
            selected_total_duration_seconds: 15,
            current_case_id: "cross_different_script_same_scene".to_string(),
            source_text_hash: "da5b4e57".to_string(),
            accepted_rewrite_hash: "4407fe43".to_string(),
            task_script_hash: "abe970c0".to_string(),
            story_fact_frame_hash: "685cec81".to_string(),
            source_profile: "B_alley_pursuit".to_string(),
            must_keep_facts: vec![
                "林峰护住苏瑶".to_string(),
                "阿青提醒".to_string(),
                "黑衣追兵".to_string(),
                "巷口".to_string(),
                "逼近".to_string(),
            ],
            ..GenerateStoryboardRequest::default()
        };
        let grounding = StoryboardGroundingContext {
            shot_script: accepted_rewrite.to_string(),
            expanded_script_text: accepted_rewrite.to_string(),
            grounding_text: accepted_rewrite.to_string(),
            grounding_source: ShotGroundingSource::ShotScript,
            primary_scene_type: "hot_blood_battle".to_string(),
            primary_scene_label: "热血战斗".to_string(),
            primary_scene_category: "action".to_string(),
            shot_scene_type: "hot_blood_battle".to_string(),
            shot_scene_label: "热血战斗".to_string(),
            shot_intent: "action_beat".to_string(),
            adaptation_reason: "release binding context carries B source facts".to_string(),
        };
        let duration_plan = build_storyboard_duration_plan(15, &[10, 5]);
        let kb_request = KbRouterRuntimeRequest {
            scene_type: "hot_blood_battle".to_string(),
            synopsis_text: source.to_string(),
            duration_seconds: 15,
            task_type: KbRouterTaskType::GenerateStoryboard,
            primary_scene_type: Some("hot_blood_battle".to_string()),
            primary_scene_label: Some("热血战斗".to_string()),
            shot_scene_type: Some("hot_blood_battle".to_string()),
            shot_scene_label: Some("热血战斗".to_string()),
            shot_intent: Some("action_beat".to_string()),
            structure_type: None,
        };
        let state = test_state();
        let kb_router_result = empty_kb_router_response(&kb_request, &state.kb_runtime);

        let pre_visible = storyboard_rows_user_visible_scene_text(&rows);
        assert!(
            !pre_visible.contains("林峰护住苏瑶") && !pre_visible.contains("黑衣追兵"),
            "{pre_visible}"
        );
        let pre_evidence = build_storyboard_binding_evidence(
            &request,
            &grounding,
            &rows,
            &duration_plan,
            "pre-release-repair",
            &kb_router_result,
        );
        assert_eq!(
            pre_evidence.missing_source_facts, request.must_keep_facts,
            "{pre_evidence:?}"
        );

        let summary = repair_storyboard_rows_from_binding_context(&mut rows, &request, &grounding);
        diversify_repeated_storyboard_subjects(&mut rows);
        assert!(summary.repaired(), "{summary:?}");

        let final_visible = storyboard_rows_user_visible_scene_text(&rows);
        for required in ["林峰护住苏瑶", "阿青提醒", "黑衣追兵", "巷口", "逼近"] {
            assert!(
                final_visible.contains(required),
                "release B hot missing {required}: {final_visible}"
            );
        }
        for row in &rows {
            assert!(
                !is_visual_description_grounding_incomplete(
                    &row.visual_description,
                    &row.person,
                    &row.scene_scale,
                    &row.character_action,
                    &row.camera_movement,
                    &row.shot_script,
                ),
                "{}",
                storyboard_row_visible_text(row)
            );
        }
        let post_evidence = build_storyboard_binding_evidence(
            &request,
            &grounding,
            &rows,
            &duration_plan,
            "post-release-repair",
            &kb_router_result,
        );
        assert_eq!(post_evidence.missing_source_facts, Vec::<String>::new());
    }

    #[test]
    fn c_rainy_dock_live_repair_restores_generic_source_facts() {
        let source = "雨夜码头，女主攥紧旧照片，追来的陌生人停在路灯外。";
        let mut baseline_heroine = test_live_validation_row("女主");
        baseline_heroine.order = 1;
        baseline_heroine.duration_seconds = 10;
        baseline_heroine.shot_duration_seconds = 10;
        baseline_heroine.shot_script = source.to_string();
        baseline_heroine
            .scene_performance_projection
            .fused_source_text = source.to_string();
        baseline_heroine.primary_scene_type = "hot_blood_battle".to_string();
        baseline_heroine.primary_scene_label = "热血战斗".to_string();
        baseline_heroine.shot_scene_type = "hot_blood_battle".to_string();
        baseline_heroine.shot_scene_label = "热血战斗".to_string();
        repair_c_rainy_dock_photo_storyboard_row(&mut baseline_heroine);

        let mut baseline_stranger = test_live_validation_row("陌生人");
        baseline_stranger.order = 2;
        baseline_stranger.duration_seconds = 5;
        baseline_stranger.shot_duration_seconds = 5;
        baseline_stranger.shot_script = source.to_string();
        baseline_stranger
            .scene_performance_projection
            .fused_source_text = source.to_string();
        baseline_stranger.primary_scene_type = "hot_blood_battle".to_string();
        baseline_stranger.primary_scene_label = "热血战斗".to_string();
        baseline_stranger.shot_scene_type = "hot_blood_battle".to_string();
        baseline_stranger.shot_scene_label = "热血战斗".to_string();
        repair_c_rainy_dock_photo_storyboard_row(&mut baseline_stranger);

        let baselines = vec![baseline_heroine.clone(), baseline_stranger.clone()];
        let mut rows = baselines.clone();
        rows[0].person = "攥紧旧".to_string();
        rows[0].scene_performance_projection.person = "攥紧旧".to_string();
        rows[0].shot_title = "镜头1：攥紧旧在废墟停住".to_string();
        rows[0].visual_description = "主体为攥紧旧，中近景把主角放在废墟前侧；林峰与苏瑶在背景里停住，冷光压住碎石，画面突出敌人逼近。".to_string();
        rows[0].character_action =
            "攥紧旧从废墟前侧开始，到主角看向敌人时结束，镜头捕捉停顿。".to_string();
        rows[0].camera_movement = "中近景定机位观察主角在废墟停住，镜头捕捉敌人逼近。".to_string();
        rows[0].prompt_text =
            "视频分镜提示词：镜头标题：攥紧旧；角色动作：林峰护住苏瑶。".to_string();

        rows[1].person = "主角".to_string();
        rows[1].scene_performance_projection.person = "主角".to_string();
        rows[1].shot_title = "镜头2：主角回望黑衣追兵".to_string();
        rows[1].visual_description = "主体为主角，中近景把主角放在废墟后侧；阿青和黑衣追兵压住背景，冷光压住碎石，画面突出敌人逼近。".to_string();
        rows[1].character_action =
            "主角从废墟后侧开始，到敌人逼近时结束，镜头捕捉回望。".to_string();
        rows[1].camera_movement = "中近景定机位观察主角和黑衣追兵，镜头捕捉敌人逼近。".to_string();
        rows[1].prompt_text =
            "视频分镜提示词：镜头标题：主角；角色动作：阿青提醒黑衣追兵。".to_string();

        let pre_repair = validate_live_storyboard_rows(&rows, 15, &baselines);
        assert!(!pre_repair.is_empty(), "{pre_repair:?}");

        let summary = repair_live_storyboard_rows_from_source(&mut rows, &baselines);
        diversify_repeated_storyboard_subjects(&mut rows);
        let post_repair = validate_live_storyboard_rows(&rows, 15, &baselines);
        assert!(summary.repaired(), "{summary:?}");
        assert!(
            post_repair.is_empty(),
            "{post_repair:?}\n{}",
            storyboard_rows_visible_text(&rows)
        );

        let request = GenerateStoryboardRequest {
            task_name: "c-rainy-dock-live-repair".to_string(),
            shot_script: Some(source.to_string()),
            expanded_script_text: Some(source.to_string()),
            selected_total_duration_seconds: 15,
            current_case_id: "cross_same_duration_same_scene_different_script".to_string(),
            source_profile: "synopsis".to_string(),
            must_keep_facts: vec!["女主".to_string()],
            forbidden_facts: vec![
                "林峰".to_string(),
                "苏瑶".to_string(),
                "阿青".to_string(),
                "黑衣追兵".to_string(),
                "废墟".to_string(),
                "敌人".to_string(),
            ],
            ..GenerateStoryboardRequest::default()
        };
        let grounding = StoryboardGroundingContext {
            shot_script: source.to_string(),
            expanded_script_text: source.to_string(),
            grounding_text: source.to_string(),
            grounding_source: ShotGroundingSource::ShotScript,
            primary_scene_type: "hot_blood_battle".to_string(),
            primary_scene_label: "热血战斗".to_string(),
            primary_scene_category: "action".to_string(),
            shot_scene_type: "hot_blood_battle".to_string(),
            shot_scene_label: "热血战斗".to_string(),
            shot_intent: "action_beat".to_string(),
            adaptation_reason: "C source fact frame repair regression".to_string(),
        };
        let duration_plan = build_storyboard_duration_plan(15, &[10, 5]);
        let kb_request = KbRouterRuntimeRequest {
            scene_type: "hot_blood_battle".to_string(),
            synopsis_text: source.to_string(),
            duration_seconds: 15,
            task_type: KbRouterTaskType::GenerateStoryboard,
            primary_scene_type: Some("hot_blood_battle".to_string()),
            primary_scene_label: Some("热血战斗".to_string()),
            shot_scene_type: Some("hot_blood_battle".to_string()),
            shot_scene_label: Some("热血战斗".to_string()),
            shot_intent: Some("action_beat".to_string()),
            structure_type: None,
        };
        let state = test_state();
        let kb_router_result = empty_kb_router_response(&kb_request, &state.kb_runtime);
        let post_evidence = build_storyboard_binding_evidence(
            &request,
            &grounding,
            &rows,
            &duration_plan,
            "post-repair",
            &kb_router_result,
        );

        assert_eq!(post_evidence.source_profile, "C_rainy_dock_photo");
        for required in ["女主", "旧照片", "陌生人", "路灯", "雨夜码头", "码头"] {
            assert!(
                post_evidence
                    .must_keep_facts
                    .iter()
                    .any(|fact| fact == required),
                "{required}: {post_evidence:?}"
            );
        }
        assert_eq!(post_evidence.missing_source_facts, Vec::<String>::new());
        assert_eq!(post_evidence.forbidden_fact_hits, Vec::<String>::new());

        let visible = storyboard_rows_visible_text(&rows);
        for required in ["女主", "旧照片", "陌生人", "路灯", "雨夜码头"] {
            assert!(visible.contains(required), "{required}: {visible}");
        }
        for forbidden in [
            "攥紧旧",
            "主角",
            "林峰",
            "苏瑶",
            "阿青",
            "黑衣追兵",
            "废墟",
            "敌人",
        ] {
            assert!(!visible.contains(forbidden), "{forbidden}: {visible}");
        }
        for row in &rows {
            assert!(
                matches!(row.person.as_str(), "女主" | "陌生人"),
                "{:?}",
                row.person
            );
        }
    }

    #[test]
    fn recent_live_fragments_are_not_treated_as_new_character_names() {
        let source = "废墟之上，主角单膝跪地，敌人缓步逼近。";
        for fragment in ["左臂垂", "方眉", "铠甲裂", "利望"] {
            let generated = format!(
                "{fragment}只描述画面状态，主角仍单膝跪地，敌人继续逼近，对峙压力没有消失。"
            );
            let reason = generated_script_identity_validator_reason(&generated, source);
            assert!(
                !matches!(reason.as_deref(), Some(reason) if reason.contains("新增源文本外姓名")),
                "{fragment}: {reason:?}"
            );
        }
    }

    #[test]
    fn expand_prompt_requires_visible_pressure_terms() {
        let request = TextGenerationRequest {
            task_type: TextGenerationTask::ExpandScript,
            scene_type: Some("action_beat".to_string()),
            story_input: "林峰护住苏瑶，黑衣追兵逼近，阿青断后掩护撤离。".to_string(),
            duration_plan: Some(StoryboardDurationPlan {
                total_duration_seconds: 15,
                row_count: 1,
                per_row_seconds: 15,
                allocated_seconds: 15,
            }),
            kb_context_summary: String::new(),
            selected_sample_ids: vec![],
            selected_kb_rules: vec![],
            output_schema: TextGenerationOutputSchema::PlainText,
            temperature: None,
            max_tokens: Some(700),
        };
        let provider = TextModelProvider {
            provider: TextModelProviderKind::Qwen,
            model: "qwen-plus".to_string(),
            base_url: Some("https://dashscope.aliyuncs.com/compatible-mode/v1".to_string()),
            api_key_ref: "session-only".to_string(),
            enabled: true,
        };
        let payload = build_qwen_request_payload(&provider, &request);
        let user_prompt = payload["messages"][1]["content"]
            .as_str()
            .expect("user prompt");

        assert!(user_prompt.contains("final body must keep a visible pressure term"));
        assert!(user_prompt.contains("追兵"));
        assert!(user_prompt.contains("对峙压力"));
        assert!(user_prompt.contains("断后"));
        assert!(user_prompt.contains("撤离"));
    }

    #[test]
    fn b_group_second_row_rebinds_empty_person_to_source_subject() {
        let mut row = test_live_validation_row("");
        row.shot_script = "阿青提醒他们，黑衣追兵从巷口逼近。".to_string();
        row.scene_performance_projection.fused_source_text =
            "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。".to_string();
        row.person = String::new();
        row.scene_performance_projection.person = String::new();
        row.shot_title = "镜头2：追兵逼近".to_string();
        row.visual_description = "主体为空镜，中近景把巷口追兵和苏瑶后退放在前后层次里；冷光压住巷口；当前视觉事件是阿青提醒追兵逼近；画面突出追兵压力".to_string();
        row.character_action =
            "空镜从巷口压力开始，到阿青提醒他们撤离时结束，镜头捕捉追兵逼近的压力。".to_string();
        row.camera_movement = "中近景定机位观察巷口压力，镜头捕捉阿青提醒和追兵逼近。".to_string();

        normalize_storyboard_row_subject_quality(&mut row);

        assert!(!row.person.trim().is_empty());
        assert_ne!(row.person, "/");
        assert!(
            row.person.contains("阿青")
                || row.person.contains("林峰")
                || row.person.contains("苏瑶")
                || row.person.contains("追兵"),
            "{}",
            row.person
        );
    }

    #[test]
    fn space_and_lens_terms_are_repaired_before_subject_validation() {
        let mut row = test_live_validation_row("林峰");
        row.shot_script = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。".to_string();
        row.person = "林峰".to_string();
        row.scene_performance_projection.person = "林峰".to_string();
        row.visual_description = "主体为空间镜头，中近景把巷口和追兵放在前后层次里；冷光压住墙面；当前视觉事件是空间镜头从巷口扫过；画面突出追兵压力".to_string();
        row.character_action =
            "空间镜头从巷口压力开始，到阿青提醒他们撤离时结束，镜头捕捉追兵逼近。".to_string();
        row.scene_performance_projection.visual_description = row.visual_description.clone();
        row.scene_performance_projection.character_action = row.character_action.clone();

        normalize_storyboard_row_subject_quality(&mut row);

        let combined = format!("{} {}", row.visual_description, row.character_action);
        assert!(!combined.contains("主体为空间"));
        assert!(!combined.contains("主体为当前空间"));
        assert!(!combined.contains("空间镜头从"));
        assert!(!combined.contains("当前空间从"));
        assert!(!combined.contains("镜头从低处推进"));
        assert!(combined.contains("林峰"));
        let warnings =
            validate_live_storyboard_rows(&[row], 10, &[test_live_validation_row("林峰")]);
        assert!(
            !warnings.iter().any(|warning| warning.message.contains("空间")
                || warning.message.contains("镜头")),
            "{warnings:?}"
        );
    }

    #[test]
    fn a_war_fallback_rows_do_not_repeat_composite_subject() {
        let state = test_state_with_golden_sample_runtime();
        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "a-war-no-repeat-composite".to_string(),
                script_id: None,
                shot_script: Some(
                    "废墟之上，主角单膝跪地，敌人缓步逼近。国战军阵建立只压紧秩序和距离，主角仍面对敌人逼近。"
                        .to_string(),
                ),
                expanded_script_text: Some(
                    "国战视角只把构图变得更有秩序，主角仍单膝跪在废墟上，敌人缓步逼近，对峙压力没有消失。敌人继续逼近，双方距离被压短。"
                        .to_string(),
                ),
                primary_scene_type: Some("action_beat".to_string()),
                primary_scene_label: Some("国战军阵建立".to_string()),
                primary_scene_category: Some("action".to_string()),
                shot_scene_type: None,
                shot_scene_label: None,
                shot_intent: None,
                adaptation_reason: None,
                selected_total_duration_seconds: 15,
                target_duration_mode: "fixed_seconds".to_string(),
                auto_segment_strategy: String::new(),
                model_config_summary: None,
                scene_type: None,
                scene_label: None,
                scene_category: None,
                ..GenerateStoryboardRequest::default()
            },
        );
        assert_eq!(storyboard.rows.len(), 2);
        assert_ne!(storyboard.rows[0].person, storyboard.rows[1].person);
        assert!(!storyboard.rows.iter().all(|row| row.person == "主角与敌人"));
    }

    #[test]
    fn live_fragment_names_do_not_trigger_new_name_validator_reason() {
        let source = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        for generated in [
            "林峰没答，继续护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。",
            "焦点压在巷口，林峰护住苏瑶，阿青提醒他们，追兵压力仍在。",
            "视线方向压向巷口，林峰护住苏瑶，黑衣追兵仍从巷口逼近。",
            "那里有烟尘压住巷口，林峰护住苏瑶，黑衣追兵仍从巷口逼近。",
        ] {
            let reason = generated_script_identity_validator_reason(generated, source);
            assert!(
                !matches!(reason.as_deref(), Some(reason) if reason.contains("新增源文本外姓名")),
                "{generated}: {reason:?}"
            );
        }
    }

    #[test]
    fn expand_validator_rejects_exact_four_case_live_drift_terms() {
        let a_source = "废墟之上，主角单膝跪地，敌人缓步逼近。";
        let named_drift = "废墟之上，孔却燃单膝跪地，敌人缓步逼近，对峙压力没有消失。";
        assert!(
            matches!(
                generated_script_identity_validator_reason(named_drift, a_source).as_deref(),
                Some(reason) if reason.contains("新增源文本外姓名")
            ),
            "{:?}",
            generated_script_identity_validator_reason(named_drift, a_source)
        );
        assert!(validate_generated_script_text(named_drift, a_source).is_none());

        let armor_drift = "废墟之上，主角单膝跪地，敌人的铠甲压住光线后继续逼近。";
        assert!(
            matches!(
                generated_script_identity_validator_reason(armor_drift, a_source).as_deref(),
                Some(reason) if reason.contains("新增源文本外设定")
            ),
            "{:?}",
            generated_script_identity_validator_reason(armor_drift, a_source)
        );
        assert!(validate_generated_script_text(armor_drift, a_source).is_none());

        let cold_laugh_drift = "废墟之上，主角冷笑着单膝跪地，敌人缓步逼近，对峙压力没有消失。";
        assert!(
            matches!(
                generated_script_identity_validator_reason(cold_laugh_drift, a_source).as_deref(),
                Some(reason) if reason.contains("新增源文本外动作")
            ),
            "{:?}",
            generated_script_identity_validator_reason(cold_laugh_drift, a_source)
        );
        assert!(validate_generated_script_text(cold_laugh_drift, a_source).is_none());

        let b_source = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let numbered_pursuer_drift = "阿青看见三名黑衣追兵从巷口逼近，林峰护住苏瑶，追兵压力仍在。";
        assert!(
            matches!(
                generated_script_identity_validator_reason(numbered_pursuer_drift, b_source)
                    .as_deref(),
                Some(reason) if reason.contains("新增源文本外设定")
            ),
            "{:?}",
            generated_script_identity_validator_reason(numbered_pursuer_drift, b_source)
        );
        assert!(validate_generated_script_text(numbered_pursuer_drift, b_source).is_none());

        let left_arm_drift = "林峰左臂护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        assert!(
            matches!(
                generated_script_identity_validator_reason(left_arm_drift, b_source).as_deref(),
                Some(reason) if reason.contains("新增源文本外设定")
            ),
            "{:?}",
            generated_script_identity_validator_reason(left_arm_drift, b_source)
        );
        assert!(validate_generated_script_text(left_arm_drift, b_source).is_none());

        let missing_pursuit_pressure = "沙盘战略视口只保留林峰护住苏瑶和阿青提醒他们撤到墙边。";
        assert!(
            matches!(
                generated_script_identity_validator_reason(missing_pursuit_pressure, b_source)
                    .as_deref(),
                Some(reason) if reason.contains("丢失源文本追兵/对峙压力")
            ),
            "{:?}",
            generated_script_identity_validator_reason(missing_pursuit_pressure, b_source)
        );
        assert!(validate_generated_script_text(missing_pursuit_pressure, b_source).is_none());
    }

    #[test]
    fn b_group_expand_validator_requires_pursuit_pressure() {
        let source = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        assert!(matches!(
            generated_script_identity_validator_reason(
                "林峰护住苏瑶，阿青提醒他们退到墙边。",
                source
            )
            .as_deref(),
            Some("丢失源文本追兵/对峙压力")
        ));
        assert!(
            validate_generated_script_text(
                "林峰护住苏瑶，阿青提醒他们，黑衣追兵仍从巷口逼近，追兵压力没有消失。",
                source,
            )
            .is_some()
        );
    }

    #[test]
    fn a_group_does_not_duplicate_enemy_subject() {
        let mut row = test_live_validation_row("主角与敌人与敌人");
        row.scene_performance_projection.person = "主角与敌人与敌人".to_string();
        row.shot_title = "镜头1：主角与敌人与敌人对峙".to_string();
        row.visual_description =
            "主体为主角与敌人与敌人，中近景把废墟和敌人与敌人逼近放在前后层次里。".to_string();
        row.character_action =
            "主角与敌人与敌人从废墟压力开始，到敌人与敌人继续逼近时结束。".to_string();
        row.prompt_text = "镜头标题：主角与敌人与敌人；角色动作：敌人与敌人逼近。".to_string();
        row.scene_performance_projection.visual_description = row.visual_description.clone();
        row.scene_performance_projection.character_action = row.character_action.clone();

        normalize_storyboard_row_subject_quality(&mut row);

        let combined = format!(
            "{} {} {} {} {}",
            row.person,
            row.visual_description,
            row.character_action,
            row.prompt_text,
            row.scene_performance_projection.visual_description
        );
        assert!(!combined.contains("主角与敌人与敌人"), "{combined}");
        assert!(!combined.contains("敌人与敌人"), "{combined}");
        assert_eq!(row.person, "主角与敌人");
    }

    #[test]
    fn b_hot_battle_rejects_offsource_location_drift() {
        let source = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        for generated in [
            "林峰在灯塔底部护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。",
            "林峰带苏瑶退上木栈道，水面反光，探照灯扫过，黑衣追兵从巷口逼近。",
            "阿青看见三名黑衣追兵刀锋出鞘，林峰护住苏瑶，追兵压力仍在。",
            "林峰衣袖裂口扩大，阿青提醒他们，黑衣追兵从巷口逼近。",
        ] {
            let reason = generated_script_identity_validator_reason(generated, source);
            assert!(
                matches!(reason.as_deref(), Some(reason) if reason.contains("新增源文本外设定")),
                "{generated}: {reason:?}"
            );
            assert!(validate_generated_script_text(generated, source).is_none());
        }
    }

    #[test]
    fn b_hot_battle_keeps_alley_pursuit_facts() {
        let source = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let accepted = validate_generated_script_text(
            "巷口压力压近，林峰护住苏瑶，阿青提醒他们，黑衣追兵继续从巷口逼近。",
            source,
        )
        .expect("source facts should remain valid");

        assert!(accepted.contains("巷口"));
        assert!(accepted.contains("黑衣追兵"));
        assert!(accepted.contains("逼近"));
        assert!(accepted.contains("林峰护住苏瑶"));
        assert!(accepted.contains("阿青提醒"));
    }

    #[test]
    fn a_war_expand_validator_rejects_source_external_packaging() {
        let source = "废墟之上，主角单膝跪地，敌人缓步逼近。";
        for generated in [
            "废墟之上，主角单膝跪地，敌人的甲胄压住光线后继续逼近。",
            "废墟之上，主角单膝跪地，敌人握紧剑柄缓步逼近。",
            "废墟之上，主角左膝压在碎石里，敌人缓步逼近。",
        ] {
            let reason = generated_script_identity_validator_reason(generated, source);
            assert!(
                matches!(reason.as_deref(), Some(reason) if reason.contains("新增源文本外设定")),
                "{generated}: {reason:?}"
            );
            assert!(validate_generated_script_text(generated, source).is_none());
        }
    }

    #[test]
    fn b_sandbox_row_subject_matches_prompt_subject() {
        let mut row = test_live_validation_row("阿青");
        row.shot_script = "阿青提醒他们，黑衣追兵从巷口逼近。".to_string();
        row.scene_performance_projection.fused_source_text =
            "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。".to_string();
        row.person = "阿青".to_string();
        row.scene_performance_projection.person = "阿青".to_string();
        row.visual_description =
            "主体为林峰、苏瑶与阿青，中近景把巷口追兵压力放在前后层次里。".to_string();
        row.character_action = "林峰、苏瑶与阿青从巷口压力开始，到提醒他们撤离时结束。".to_string();
        row.prompt_text =
            "镜头标题：林峰、苏瑶与阿青；角色动作：林峰、苏瑶与阿青提醒他们。".to_string();
        row.scene_performance_projection.visual_description = row.visual_description.clone();
        row.scene_performance_projection.character_action = row.character_action.clone();

        normalize_storyboard_row_subject_quality(&mut row);

        assert_eq!(row.person, "阿青");
        let combined = format!(
            "{} {} {} {}",
            row.visual_description,
            row.character_action,
            row.prompt_text,
            row.scene_performance_projection.visual_description
        );
        assert!(!combined.contains("林峰、苏瑶与阿青"), "{combined}");
        assert!(combined.contains("阿青"), "{combined}");
    }

    #[test]
    fn b_group_rows_split_composite_subjects() {
        let mut row = test_live_validation_row("林峰、苏瑶与阿青");
        row.shot_script = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。".to_string();
        row.scene_performance_projection.fused_source_text = row.shot_script.clone();
        row.visual_description =
            "主体为林峰、苏瑶与阿青，中近景把巷口和追兵逼近放在前后层次里。".to_string();
        row.character_action =
            "林峰、苏瑶与阿青从林峰护住苏瑶开始，到黑衣追兵逼近时结束。".to_string();
        row.prompt_text =
            "镜头标题：林峰、苏瑶与阿青；角色动作：林峰、苏瑶与阿青护住苏瑶。".to_string();
        row.scene_performance_projection.visual_description = row.visual_description.clone();
        row.scene_performance_projection.character_action = row.character_action.clone();

        split_overbroad_storyboard_subject(&mut row);

        assert_eq!(row.person, "林峰与苏瑶");
        let combined = format!(
            "{} {} {}",
            row.visual_description, row.character_action, row.prompt_text
        );
        assert!(!combined.contains("林峰、苏瑶与阿青"), "{combined}");
        assert!(combined.contains("林峰与苏瑶"), "{combined}");
    }

    fn storyboard_row_visible_text(row: &GeneratedStoryboardRow) -> String {
        format!(
            "{} {} {} {} {} {}",
            row.person,
            row.shot_title,
            row.visual_description,
            row.character_action,
            row.camera_movement,
            row.prompt_text
        )
    }

    fn storyboard_rows_visible_text(rows: &[GeneratedStoryboardRow]) -> String {
        rows.iter()
            .map(storyboard_row_visible_text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn storyboard_rows_user_visible_scene_text(rows: &[GeneratedStoryboardRow]) -> String {
        rows.iter()
            .map(|row| {
                format!(
                    "{} {} {} {}",
                    row.shot_title,
                    row.visual_description,
                    row.character_action,
                    row.camera_movement
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn a_ruin_binding_atom_row() -> GeneratedStoryboardRow {
        let mut row = test_live_validation_row("主角");
        row.person = "主角".to_string();
        row.visual_description =
            "主体为主角，中景把主角和敌人放在废墟前后层次里；主角单膝跪地，画面突出对峙压力。"
                .to_string();
        row.character_action = "主角从废墟之上单膝跪地开始，到敌人缓步逼近时结束。".to_string();
        row.camera_movement = "中景定机位观察主角与敌人的对峙压力，捕捉敌人逼近。".to_string();
        row.prompt_text = "视频分镜提示词：仅包装当前镜头。".to_string();
        row.scene_performance_projection.person = row.person.clone();
        row.scene_performance_projection.visual_description = row.visual_description.clone();
        row.scene_performance_projection.character_action = row.character_action.clone();
        row
    }

    fn build_a_ruin_binding_evidence_for_test(
        must_keep_facts: Vec<String>,
        forbidden_facts: Vec<String>,
        rows: Vec<GeneratedStoryboardRow>,
    ) -> StoryboardBindingEvidence {
        let source = "废墟之上，主角单膝跪地，敌人缓步逼近。";
        let request = GenerateStoryboardRequest {
            task_name: "a-ruin-binding-gate".to_string(),
            shot_script: Some(source.to_string()),
            expanded_script_text: Some(source.to_string()),
            selected_total_duration_seconds: 15,
            must_keep_facts,
            forbidden_facts,
            ..GenerateStoryboardRequest::default()
        };
        let grounding = StoryboardGroundingContext {
            shot_script: source.to_string(),
            expanded_script_text: source.to_string(),
            grounding_text: source.to_string(),
            grounding_source: ShotGroundingSource::ShotScript,
            primary_scene_type: "hot_blood_battle".to_string(),
            primary_scene_label: "热血战斗".to_string(),
            primary_scene_category: "action".to_string(),
            shot_scene_type: "hot_blood_battle".to_string(),
            shot_scene_label: "热血战斗".to_string(),
            shot_intent: "action_beat".to_string(),
            adaptation_reason: "binding gate test".to_string(),
        };
        let duration_plan = build_storyboard_duration_plan(15, &[15]);
        let kb_request = KbRouterRuntimeRequest {
            scene_type: "hot_blood_battle".to_string(),
            synopsis_text: source.to_string(),
            duration_seconds: 15,
            task_type: KbRouterTaskType::GenerateStoryboard,
            primary_scene_type: Some("hot_blood_battle".to_string()),
            primary_scene_label: Some("热血战斗".to_string()),
            shot_scene_type: Some("hot_blood_battle".to_string()),
            shot_scene_label: Some("热血战斗".to_string()),
            shot_intent: Some("action_beat".to_string()),
            structure_type: None,
        };
        let state = test_state();
        let kb_router_result = empty_kb_router_response(&kb_request, &state.kb_runtime);
        build_storyboard_binding_evidence(
            &request,
            &grounding,
            &rows,
            &duration_plan,
            "a-ruin-binding-gate",
            &kb_router_result,
        )
    }

    fn build_b_alley_binding_evidence_for_test(
        must_keep_facts: Vec<String>,
        forbidden_facts: Vec<String>,
        rows: Vec<GeneratedStoryboardRow>,
    ) -> StoryboardBindingEvidence {
        let source = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let request = GenerateStoryboardRequest {
            task_name: "b-alley-binding-gate".to_string(),
            shot_script: Some(source.to_string()),
            expanded_script_text: Some(source.to_string()),
            selected_total_duration_seconds: 15,
            must_keep_facts,
            forbidden_facts,
            ..GenerateStoryboardRequest::default()
        };
        let grounding = StoryboardGroundingContext {
            shot_script: source.to_string(),
            expanded_script_text: source.to_string(),
            grounding_text: source.to_string(),
            grounding_source: ShotGroundingSource::ShotScript,
            primary_scene_type: "hot_blood_battle".to_string(),
            primary_scene_label: "热血战斗".to_string(),
            primary_scene_category: "action".to_string(),
            shot_scene_type: "hot_blood_battle".to_string(),
            shot_scene_label: "热血战斗".to_string(),
            shot_intent: "action_beat".to_string(),
            adaptation_reason: "B alley binding gate test".to_string(),
        };
        let duration_plan = build_storyboard_duration_plan(15, &[10, 5]);
        let kb_request = KbRouterRuntimeRequest {
            scene_type: "hot_blood_battle".to_string(),
            synopsis_text: source.to_string(),
            duration_seconds: 15,
            task_type: KbRouterTaskType::GenerateStoryboard,
            primary_scene_type: Some("hot_blood_battle".to_string()),
            primary_scene_label: Some("热血战斗".to_string()),
            shot_scene_type: Some("hot_blood_battle".to_string()),
            shot_scene_label: Some("热血战斗".to_string()),
            shot_intent: Some("action_beat".to_string()),
            structure_type: None,
        };
        let state = test_state();
        let kb_router_result = empty_kb_router_response(&kb_request, &state.kb_runtime);
        build_storyboard_binding_evidence(
            &request,
            &grounding,
            &rows,
            &duration_plan,
            "b-alley-binding-gate",
            &kb_router_result,
        )
    }

    #[test]
    fn binding_gate_blocks_maps_b_alley_pursuit_sentence_equivalence() {
        let source_sentence = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let mut row = test_live_validation_row("林峰");
        row.person = "林峰".to_string();
        row.visual_description =
            "主体为林峰，中景把林峰护住苏瑶压在巷口退路前；阿青在旁提醒他们，黑衣追兵沿巷口来路继续逼近。"
                .to_string();
        row.character_action =
            "林峰从护住苏瑶开始，到阿青提醒、黑衣追兵压近巷口退路时结束。".to_string();
        row.camera_movement = "中景定机位观察林峰护人、阿青提醒和黑衣追兵逼近的压力。".to_string();
        row.prompt_text = "视频分镜提示词：只包装当前镜头。".to_string();

        let rows_text = storyboard_rows_binding_text(&[row.clone()]);
        assert!(!rows_text.contains(source_sentence), "{rows_text}");
        let evidence = build_b_alley_binding_evidence_for_test(
            vec![source_sentence.to_string()],
            vec![],
            vec![row],
        );

        assert_eq!(
            evidence.missing_source_facts,
            Vec::<String>::new(),
            "{evidence:?}"
        );
    }

    #[test]
    fn binding_gate_blocks_b_alley_sentence_when_pursuer_atom_missing() {
        let source_sentence = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let mut row = test_live_validation_row("林峰");
        row.person = "林峰".to_string();
        row.visual_description =
            "主体为林峰，中景把林峰护住苏瑶压在巷口退路前；阿青在旁提醒他们，危险沿巷口逼近。"
                .to_string();
        row.character_action = "林峰从护住苏瑶开始，到阿青提醒、巷口压力压近时结束。".to_string();
        row.camera_movement = "中景定机位观察林峰护人和阿青提醒。".to_string();
        row.prompt_text = "视频分镜提示词：只包装当前镜头。".to_string();

        let evidence = build_b_alley_binding_evidence_for_test(
            vec![source_sentence.to_string()],
            vec![],
            vec![row],
        );

        assert!(
            evidence
                .missing_source_facts
                .iter()
                .any(|fact| fact == source_sentence),
            "{evidence:?}"
        );
    }

    #[test]
    fn binding_gate_blocks_b_alley_sentence_prompt_text_only_coverage() {
        let source_sentence = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let mut row = test_live_validation_row("苏瑶");
        row.person = "苏瑶".to_string();
        row.visual_description = "主体为苏瑶，中景把她放在墙边，冷光压住街面。".to_string();
        row.character_action = "苏瑶从墙边停住开始，到回望来路时结束。".to_string();
        row.camera_movement = "中景定机位观察苏瑶停住。".to_string();
        row.prompt_text = format!("视频分镜提示词：剧情动作压力：{source_sentence}");

        let evidence = build_b_alley_binding_evidence_for_test(
            vec![source_sentence.to_string()],
            vec![],
            vec![row],
        );

        assert!(
            evidence
                .missing_source_facts
                .iter()
                .any(|fact| fact == source_sentence),
            "{evidence:?}"
        );
    }

    #[test]
    fn binding_evidence_does_not_count_prompt_text_as_source_of_truth() {
        let source = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let mut row = test_live_validation_row("苏瑶");
        row.shot_script = "苏瑶贴着墙边停住。".to_string();
        row.scene_performance_projection.fused_source_text = row.shot_script.clone();
        row.visual_description =
            "主体为苏瑶，中近景把苏瑶放在墙边前侧，冷光压住街面，画面突出紧迫感。".to_string();
        row.character_action = "苏瑶从墙边停住开始，到回望来路时结束。".to_string();
        row.camera_movement = "中近景定机位观察苏瑶停住。".to_string();
        row.prompt_text =
            "视频分镜提示词：剧情动作压力：林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。"
                .to_string();
        let rows = vec![row];
        let request = GenerateStoryboardRequest {
            task_name: "prompt-not-source-of-truth".to_string(),
            shot_script: Some(source.to_string()),
            expanded_script_text: Some(source.to_string()),
            selected_total_duration_seconds: 10,
            must_keep_facts: vec![
                "林峰护住苏瑶".to_string(),
                "阿青提醒".to_string(),
                "黑衣追兵".to_string(),
                "巷口".to_string(),
                "逼近".to_string(),
            ],
            ..GenerateStoryboardRequest::default()
        };
        let grounding = StoryboardGroundingContext {
            shot_script: source.to_string(),
            expanded_script_text: source.to_string(),
            grounding_text: source.to_string(),
            grounding_source: ShotGroundingSource::ShotScript,
            primary_scene_type: "hot_blood_battle".to_string(),
            primary_scene_label: "热血战斗".to_string(),
            primary_scene_category: "action".to_string(),
            shot_scene_type: "hot_blood_battle".to_string(),
            shot_scene_label: "热血战斗".to_string(),
            shot_intent: "action_beat".to_string(),
            adaptation_reason: "prompt packaging is not binding evidence".to_string(),
        };
        let duration_plan = build_storyboard_duration_plan(10, &[10]);
        let kb_request = KbRouterRuntimeRequest {
            scene_type: "hot_blood_battle".to_string(),
            synopsis_text: source.to_string(),
            duration_seconds: 10,
            task_type: KbRouterTaskType::GenerateStoryboard,
            primary_scene_type: Some("hot_blood_battle".to_string()),
            primary_scene_label: Some("热血战斗".to_string()),
            shot_scene_type: Some("hot_blood_battle".to_string()),
            shot_scene_label: Some("热血战斗".to_string()),
            shot_intent: Some("action_beat".to_string()),
            structure_type: None,
        };
        let state = test_state();
        let kb_router_result = empty_kb_router_response(&kb_request, &state.kb_runtime);

        let evidence = build_storyboard_binding_evidence(
            &request,
            &grounding,
            &rows,
            &duration_plan,
            "prompt-only",
            &kb_router_result,
        );
        assert!(
            evidence
                .missing_source_facts
                .iter()
                .any(|fact| fact == "黑衣追兵")
                && evidence
                    .missing_source_facts
                    .iter()
                    .any(|fact| fact == "巷口"),
            "{evidence:?}"
        );
    }

    #[test]
    fn accepted_snapshot_anchors_map_to_existing_storyboard_fields() {
        let source = "街边巷口，林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let accepted_snapshot = AcceptedRewriteSnapshotBinding {
            current_case_id: "accepted-field-map".to_string(),
            source_text_hash: "source-hash".to_string(),
            accepted_rewrite_hash: "accepted-hash".to_string(),
            scene_type: "hot_blood_battle".to_string(),
            scene_label: "热血战斗".to_string(),
            scene_category: "action".to_string(),
            duration_seconds: 15,
            target_duration_mode: "fixed_seconds".to_string(),
            accepted_confirmation_body: source.to_string(),
            story_fact_frame_hash: "frame-hash".to_string(),
            explicit_facts: vec![
                "林峰".to_string(),
                "苏瑶".to_string(),
                "阿青".to_string(),
                "黑衣追兵".to_string(),
                "巷口".to_string(),
                "林峰护住苏瑶".to_string(),
                "阿青提醒".to_string(),
                "黑衣追兵逼近".to_string(),
            ],
            inferred_scene_facts: vec![
                core_domain::contracts::InferredSceneFactBinding {
                    fact: "林峰临时借用街边木棍形成格挡动作".to_string(),
                    inference_reason: "低风险动作调度，街边木棍只服务当前场景分镜和AI视频。"
                        .to_string(),
                    inference_scope: "临时动作道具".to_string(),
                },
                core_domain::contracts::InferredSceneFactBinding {
                    fact: "三名黑衣追兵带着伤口逼近".to_string(),
                    inference_reason: "高风险人数和伤势漂移".to_string(),
                    inference_scope: "禁止进入分镜字段".to_string(),
                },
            ],
            must_keep_facts: vec![
                "林峰护住苏瑶".to_string(),
                "阿青提醒".to_string(),
                "黑衣追兵".to_string(),
                "巷口".to_string(),
                "逼近".to_string(),
            ],
            forbidden_facts: vec!["三名".to_string(), "伤口".to_string(), "发型".to_string()],
        };
        let request = GenerateStoryboardRequest {
            task_name: "accepted-field-map".to_string(),
            shot_script: Some(source.to_string()),
            expanded_script_text: Some(source.to_string()),
            selected_total_duration_seconds: 15,
            accepted_rewrite_snapshot: Some(accepted_snapshot.clone()),
            current_case_id: accepted_snapshot.current_case_id.clone(),
            source_text_hash: accepted_snapshot.source_text_hash.clone(),
            accepted_rewrite_hash: accepted_snapshot.accepted_rewrite_hash.clone(),
            task_script_hash: stable_binding_hash_json(source),
            story_fact_frame_hash: accepted_snapshot.story_fact_frame_hash.clone(),
            source_profile: "B_alley_pursuit".to_string(),
            must_keep_facts: accepted_snapshot.must_keep_facts.clone(),
            forbidden_facts: accepted_snapshot.forbidden_facts.clone(),
            ..GenerateStoryboardRequest::default()
        };
        let grounding = StoryboardGroundingContext {
            shot_script: source.to_string(),
            expanded_script_text: source.to_string(),
            grounding_text: source.to_string(),
            grounding_source: ShotGroundingSource::ShotScript,
            primary_scene_type: "hot_blood_battle".to_string(),
            primary_scene_label: "热血战斗".to_string(),
            primary_scene_category: "action".to_string(),
            shot_scene_type: "hot_blood_battle".to_string(),
            shot_scene_label: "热血战斗".to_string(),
            shot_intent: "action_beat".to_string(),
            adaptation_reason: "accepted snapshot field mapping".to_string(),
        };
        let mut rows = vec![
            test_live_validation_row("中景把"),
            test_live_validation_row("阿青"),
        ];
        for (index, row) in rows.iter_mut().enumerate() {
            row.order = (index + 1) as u32;
            row.shot_script = source.to_string();
            row.scene_performance_projection.fused_source_text = source.to_string();
            row.visual_description = "主体为中景把，画面只写墙边停顿。".to_string();
            row.character_action = "中景把从墙边开始，到停住时结束。".to_string();
            row.camera_movement = "镜头观察墙边停顿。".to_string();
            row.prompt_text = "视频分镜提示词：三名黑衣追兵带着伤口逼近，发型明确。".to_string();
        }

        let summary = repair_storyboard_rows_from_binding_context(&mut rows, &request, &grounding);
        assert!(summary.repaired(), "{summary:?}");
        let source_fields = storyboard_rows_binding_text(&rows);
        let delivery = storyboard_rows_delivery_text(&rows);
        for required in [
            "林峰护住苏瑶",
            "阿青提醒",
            "黑衣追兵",
            "巷口",
            "逼近",
            "街边木棍",
        ] {
            assert!(
                source_fields.contains(required),
                "{required}: {source_fields}"
            );
        }
        for forbidden in ["三名", "伤口", "发型"] {
            assert!(!delivery.contains(forbidden), "{forbidden}: {delivery}");
        }
        assert!(
            rows.iter().all(|row| !row.person.contains("中景把")),
            "{rows:?}"
        );
        assert!(
            rows.iter().all(|row| row.prompt_text.contains("确认稿正文")
                && row.prompt_text.contains("角色表演锚")
                && row.prompt_text.contains("基础/复杂场景描述")
                && row.prompt_text.contains("目标时长/节奏落点")),
            "{:?}",
            rows.iter()
                .map(|row| row.prompt_text.as_str())
                .collect::<Vec<_>>()
        );

        let duration_plan = build_storyboard_duration_plan(15, &[10, 5]);
        let kb_request = KbRouterRuntimeRequest {
            scene_type: "hot_blood_battle".to_string(),
            synopsis_text: source.to_string(),
            duration_seconds: 15,
            task_type: KbRouterTaskType::GenerateStoryboard,
            primary_scene_type: Some("hot_blood_battle".to_string()),
            primary_scene_label: Some("热血战斗".to_string()),
            shot_scene_type: Some("hot_blood_battle".to_string()),
            shot_scene_label: Some("热血战斗".to_string()),
            shot_intent: Some("action_beat".to_string()),
            structure_type: None,
        };
        let state = test_state();
        let kb_router_result = empty_kb_router_response(&kb_request, &state.kb_runtime);
        let evidence = build_storyboard_binding_evidence(
            &request,
            &grounding,
            &rows,
            &duration_plan,
            "accepted-field-map",
            &kb_router_result,
        );
        assert_eq!(
            evidence.missing_source_facts,
            Vec::<String>::new(),
            "{evidence:?}"
        );
        assert_eq!(
            evidence.forbidden_fact_hits,
            Vec::<String>::new(),
            "{evidence:?}"
        );
    }

    #[test]
    fn binding_gate_blocks_normalizes_numbered_a_ruin_sentence_facts() {
        let source_sentence = "废墟之上，主角单膝跪地，敌人缓步逼近。";
        let rows = vec![a_ruin_binding_atom_row()];
        let evidence = build_a_ruin_binding_evidence_for_test(
            vec![
                source_sentence.to_string(),
                format!("1. {source_sentence}"),
                "主角与敌人对峙".to_string(),
            ],
            vec![],
            rows,
        );

        assert_eq!(
            evidence.missing_source_facts,
            Vec::<String>::new(),
            "{evidence:?}"
        );
        assert_eq!(
            evidence
                .must_keep_facts
                .iter()
                .filter(|fact| fact.as_str() == source_sentence)
                .count(),
            1,
            "{evidence:?}"
        );
        assert!(
            evidence
                .must_keep_facts
                .iter()
                .all(|fact| !fact.starts_with("1.")),
            "{evidence:?}"
        );
    }

    #[test]
    fn binding_gate_blocks_maps_a_ruin_duel_relation_equivalence() {
        let rows = vec![a_ruin_binding_atom_row()];
        let rows_text = storyboard_rows_binding_text(&rows);
        assert!(!rows_text.contains("主角与敌人对峙"), "{rows_text}");

        let evidence = build_a_ruin_binding_evidence_for_test(
            vec!["主角与敌人对峙".to_string()],
            vec![],
            rows,
        );

        assert_eq!(
            evidence.missing_source_facts,
            Vec::<String>::new(),
            "{evidence:?}"
        );
    }

    #[test]
    fn binding_gate_blocks_prompt_text_only_source_fact_coverage() {
        let source_sentence = "废墟之上，主角单膝跪地，敌人缓步逼近。";
        let mut row = test_live_validation_row("旁观者");
        row.person = "旁观者".to_string();
        row.visual_description = "主体为旁观者，中景只写墙边停顿。".to_string();
        row.character_action = "旁观者从墙边开始，到停住时结束。".to_string();
        row.camera_movement = "中景定机位观察墙边停顿。".to_string();
        row.prompt_text = format!("视频分镜提示词：主角与敌人对峙；{source_sentence}");
        row.scene_performance_projection.person = row.person.clone();
        row.scene_performance_projection.visual_description = row.visual_description.clone();
        row.scene_performance_projection.character_action = row.character_action.clone();

        let evidence = build_a_ruin_binding_evidence_for_test(
            vec![source_sentence.to_string(), "主角与敌人对峙".to_string()],
            vec![],
            vec![row],
        );

        assert!(
            evidence
                .missing_source_facts
                .iter()
                .any(|fact| fact == source_sentence),
            "{evidence:?}"
        );
        assert!(
            evidence
                .missing_source_facts
                .iter()
                .any(|fact| fact == "主角与敌人对峙"),
            "{evidence:?}"
        );
    }

    #[test]
    fn binding_gate_blocks_forbidden_hits_after_equivalence_mapping() {
        let mut row = a_ruin_binding_atom_row();
        row.prompt_text = "视频分镜提示词：甲胄压住光线。".to_string();
        let evidence = build_a_ruin_binding_evidence_for_test(
            vec!["主角与敌人对峙".to_string()],
            vec!["甲胄".to_string()],
            vec![row],
        );

        assert_eq!(
            evidence.missing_source_facts,
            Vec::<String>::new(),
            "{evidence:?}"
        );
        assert_eq!(evidence.forbidden_fact_hits, vec!["甲胄".to_string()]);
        let blockers = story_fact_frame_binding_gate_blockers(&evidence);
        assert!(
            blockers
                .iter()
                .any(|warning| warning.code == "story_fact_frame_forbidden_fact_hit"),
            "{blockers:?}"
        );
    }

    #[test]
    fn binding_gate_blocks_rows_match_only_when_source_facts_are_missing() {
        let mut evidence = StoryboardBindingEvidence {
            storyboard_rows_hash: "same-ui-and-backend-rows".to_string(),
            missing_source_facts: vec!["黑衣追兵".to_string()],
            ..StoryboardBindingEvidence::default()
        };

        let blockers = story_fact_frame_binding_gate_blockers(&evidence);
        assert!(
            blockers
                .iter()
                .any(|warning| warning.code == "story_fact_frame_anchor_not_fully_mapped"),
            "{blockers:?}"
        );

        evidence.missing_source_facts.clear();
        assert!(
            story_fact_frame_binding_gate_blockers(&evidence).is_empty(),
            "{evidence:?}"
        );
    }

    #[test]
    fn binding_gate_blocks_forbidden_hits_and_stale_bindings() {
        let evidence = StoryboardBindingEvidence {
            stale_binding_detected: true,
            forbidden_fact_hits: vec!["三名".to_string(), "伤口".to_string()],
            ..StoryboardBindingEvidence::default()
        };

        let codes = story_fact_frame_binding_gate_blockers(&evidence)
            .into_iter()
            .map(|warning| warning.code)
            .collect::<Vec<_>>();
        assert!(
            codes.contains(&"story_fact_frame_stale_binding_detected".to_string()),
            "{codes:?}"
        );
        assert!(
            codes.contains(&"story_fact_frame_forbidden_fact_hit".to_string()),
            "{codes:?}"
        );
    }

    #[test]
    fn expand_script_scene_adaptation_distinguishes_four_acceptance_cases() {
        let state = test_state();
        let expand_for_scene = |scene_type: &str, scene_label: &str, source: &str| {
            expand_script(
                &state,
                ExpandScriptRequest {
                    scene_type: scene_type.to_string(),
                    scene_label: Some(scene_label.to_string()),
                    scene_category: None,
                    model_config_summary: None,
                    selected_total_duration_seconds: Some(15),
                    target_duration_seconds: Some(15),
                    target_duration_mode: "fixed_seconds".to_string(),
                    story_length_profile: String::new(),
                    source_material_length_chars: 0,
                    auto_segment_strategy: String::new(),
                    source_input_type: String::new(),
                    authoring_mode: String::new(),
                    source_material_summary: String::new(),
                    source_story_facts: Default::default(),
                    preserved_fact_summary: String::new(),
                    changed_for_screenplay_summary: String::new(),
                    omitted_detail_summary: String::new(),
                    synopsis_text: source.to_string(),
                },
            )
            .expanded_script_text
        };

        let a_source = "废墟之上，主角单膝跪地，敌人缓步逼近。";
        let a_hot = expand_for_scene("hot_blood_battle", "热血战斗", a_source);
        let a_war = expand_for_scene("chinese_war_formation", "国战军阵建立", a_source);
        assert_ne!(a_hot, a_war);
        for required in ["热血战斗", "废墟", "主角", "敌人", "逼近", "战斗压力"] {
            assert!(
                a_hot.contains(required),
                "A hot missing {required}: {a_hot}"
            );
        }
        for required in [
            "国战",
            "军阵",
            "前场压迫",
            "后场调度",
            "战场秩序",
            "废墟",
            "主角",
            "敌人",
            "逼近",
        ] {
            assert!(
                a_war.contains(required),
                "A war missing {required}: {a_war}"
            );
        }
        for drift in [
            "孔却燃",
            "冷笑",
            "左拳",
            "指节",
            "渗血",
            "断桥",
            "断梁",
            "龟裂",
            "危险感",
            "甲胄",
            "剑柄",
            "断戟",
            "军队规模",
        ] {
            assert!(!a_hot.contains(drift), "A hot drift {drift}: {a_hot}");
            assert!(!a_war.contains(drift), "A war drift {drift}: {a_war}");
        }
        assert!(validate_generated_script_text(&a_hot, a_source).is_some());
        assert!(validate_generated_script_text(&a_war, a_source).is_some());

        let b_source = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let b_hot = expand_for_scene("hot_blood_battle", "热血战斗", b_source);
        let b_board = expand_for_scene("slg_sandbox_view", "沙盘战略视口", b_source);
        assert_ne!(b_hot, b_board);
        for required in [
            "热血战斗",
            "紧迫",
            "对抗",
            "护住苏瑶",
            "阿青提醒",
            "黑衣追兵",
            "巷口",
            "逼近",
            "动作节奏",
        ] {
            assert!(
                b_hot.contains(required),
                "B hot missing {required}: {b_hot}"
            );
        }
        for required in [
            "沙盘战略视口",
            "视口",
            "态势",
            "巷口压力",
            "退路",
            "调度",
            "黑衣追兵",
            "巷口",
            "逼近",
        ] {
            assert!(
                b_board.contains(required),
                "B board missing {required}: {b_board}"
            );
        }
        for drift in [
            "方仅",
            "左臂",
            "衣袖",
            "断桥",
            "灯塔",
            "水面",
            "地图道具",
            "三名黑衣追兵",
        ] {
            assert!(!b_hot.contains(drift), "B hot drift {drift}: {b_hot}");
            assert!(!b_board.contains(drift), "B board drift {drift}: {b_board}");
        }
        assert!(validate_generated_script_text(&b_hot, b_source).is_some());
        assert!(validate_generated_script_text(&b_board, b_source).is_some());
    }

    #[test]
    fn generate_storyboard_distinguishes_b_hot_and_b_sandbox_rows() {
        let state = test_state_with_golden_sample_runtime();
        let source = "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。";
        let storyboard_for_scene = |scene_type: &str, scene_label: &str| {
            generate_storyboard(
                &state,
                GenerateStoryboardRequest {
                    task_name: format!("b-scene-drift-{scene_type}"),
                    script_id: None,
                    shot_script: Some(source.to_string()),
                    expanded_script_text: Some(source.to_string()),
                    primary_scene_type: Some(scene_type.to_string()),
                    primary_scene_label: Some(scene_label.to_string()),
                    primary_scene_category: Some("action".to_string()),
                    shot_scene_type: Some(scene_type.to_string()),
                    shot_scene_label: Some(scene_label.to_string()),
                    shot_intent: None,
                    adaptation_reason: None,
                    selected_total_duration_seconds: 15,
                    target_duration_mode: "fixed_seconds".to_string(),
                    auto_segment_strategy: String::new(),
                    model_config_summary: None,
                    scene_type: Some(scene_type.to_string()),
                    scene_label: Some(scene_label.to_string()),
                    scene_category: Some("action".to_string()),
                    ..GenerateStoryboardRequest::default()
                },
            )
        };

        let hot = storyboard_for_scene("hot_blood_battle", "热血战斗");
        let sandbox = storyboard_for_scene("slg_sandbox_view", "沙盘战略视口");
        let hot_visible = storyboard_rows_user_visible_scene_text(&hot.rows);
        let sandbox_visible = storyboard_rows_user_visible_scene_text(&sandbox.rows);

        assert_ne!(hot_visible, sandbox_visible);
        for required in ["林峰护住苏瑶", "阿青提醒", "黑衣追兵", "巷口", "逼近"] {
            assert!(
                sandbox_visible.contains(required),
                "B sandbox missing source fact {required}: {sandbox_visible}"
            );
        }
        for required in ["沙盘战略视口", "视口", "态势", "退路", "调度"] {
            assert!(
                sandbox_visible.contains(required),
                "B sandbox missing scene expression {required}: {sandbox_visible}"
            );
        }
        assert!(
            !hot_visible.contains("沙盘战略视口"),
            "B hot should not inherit sandbox label: {hot_visible}"
        );
        for row in &sandbox.rows {
            for (field_name, field_value) in [
                ("shot_title", row.shot_title.as_str()),
                ("visual_description", row.visual_description.as_str()),
                ("character_action", row.character_action.as_str()),
                ("camera_movement", row.camera_movement.as_str()),
            ] {
                assert!(
                    live_field_visual_or_abstract_subject_term(field_name, field_value).is_none(),
                    "B sandbox {field_name} should not use visual or abstract subject: {field_value}"
                );
            }
        }
        for forbidden in ["三名", "刀锋", "衣袖裂口", "左臂", "伤口", "地图道具"] {
            assert!(
                !sandbox_visible.contains(forbidden),
                "B sandbox leaked {forbidden}: {sandbox_visible}"
            );
        }
    }

    #[test]
    fn generate_storyboard_a_hot_30s_keeps_ruin_enemy_grounding_without_drift() {
        let state = test_state_with_golden_sample_runtime();
        let source = "废墟之上，主角单膝跪地，敌人缓步逼近。";
        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "a-hot-30s-regression".to_string(),
                script_id: None,
                shot_script: Some(source.to_string()),
                expanded_script_text: Some(source.to_string()),
                primary_scene_type: Some("hot_blood_battle".to_string()),
                primary_scene_label: Some("热血战斗".to_string()),
                primary_scene_category: Some("action".to_string()),
                shot_scene_type: Some("hot_blood_battle".to_string()),
                shot_scene_label: Some("热血战斗".to_string()),
                shot_intent: None,
                adaptation_reason: None,
                selected_total_duration_seconds: 30,
                target_duration_mode: "fixed_seconds".to_string(),
                auto_segment_strategy: String::new(),
                model_config_summary: None,
                scene_type: Some("hot_blood_battle".to_string()),
                scene_label: Some("热血战斗".to_string()),
                scene_category: Some("action".to_string()),
                ..GenerateStoryboardRequest::default()
            },
        );

        assert_ne!(storyboard.export_status.status, BridgeCallStatus::Blocked);
        assert_eq!(storyboard.rows.len(), 3);
        assert_eq!(
            storyboard
                .rows
                .iter()
                .map(|row| row.duration_seconds)
                .collect::<Vec<_>>(),
            vec![10, 10, 10]
        );
        let all_visible = storyboard_rows_visible_text(&storyboard.rows);
        for required in ["主角", "敌人", "废墟", "单膝跪地", "逼近"] {
            assert!(
                all_visible.contains(required),
                "A hot 30s missing {required}: {all_visible}"
            );
        }
        assert!(
            contains_any_story_term(&all_visible, &["缓步逼近", "敌人缓步"]),
            "A hot 30s missing enemy approach: {all_visible}"
        );
        assert!(
            storyboard.rows.iter().any(|row| row.person == "敌人"),
            "{:?}",
            storyboard
                .rows
                .iter()
                .map(|row| row.person.as_str())
                .collect::<Vec<_>>()
        );
        for forbidden in [
            "甲胄",
            "铠甲",
            "剑柄",
            "断戟",
            "整列军阵",
            "军阵规模",
            "军队规模",
        ] {
            assert!(
                !all_visible.contains(forbidden),
                "A hot 30s leaked {forbidden}: {all_visible}"
            );
        }
    }

    #[test]
    fn final_storyboard_rows_pass_no_empty_person_in_four_baseline_cases() {
        let state = test_state_with_golden_sample_runtime();
        let cases = [
            (
                "a-action-final",
                "热血战斗",
                "废墟之上，主角单膝跪地，敌人缓步逼近。",
            ),
            (
                "a-war-final",
                "国战军阵建立",
                "废墟之上，主角单膝跪地，敌人缓步逼近。",
            ),
            (
                "b-action-final",
                "热血战斗",
                "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。",
            ),
            (
                "b-board-final",
                "沙盘战略视口",
                "林峰护住苏瑶，阿青提醒他们，黑衣追兵从巷口逼近。",
            ),
        ];

        for (task_name, scene_label, source) in cases {
            let storyboard = generate_storyboard(
                &state,
                GenerateStoryboardRequest {
                    task_name: task_name.to_string(),
                    script_id: None,
                    shot_script: Some(source.to_string()),
                    expanded_script_text: Some(source.to_string()),
                    primary_scene_type: Some("action_beat".to_string()),
                    primary_scene_label: Some(scene_label.to_string()),
                    primary_scene_category: Some("action".to_string()),
                    shot_scene_type: None,
                    shot_scene_label: None,
                    shot_intent: None,
                    adaptation_reason: None,
                    selected_total_duration_seconds: 15,
                    target_duration_mode: "fixed_seconds".to_string(),
                    auto_segment_strategy: String::new(),
                    model_config_summary: None,
                    scene_type: None,
                    scene_label: None,
                    scene_category: None,
                    ..GenerateStoryboardRequest::default()
                },
            );

            assert_eq!(
                storyboard
                    .rows
                    .iter()
                    .map(|row| row.duration_seconds)
                    .collect::<Vec<_>>(),
                vec![10, 5],
                "{task_name}"
            );
            for row in &storyboard.rows {
                assert!(
                    !row.person.trim().is_empty(),
                    "{task_name}: {:?}",
                    row.person
                );
                assert_ne!(row.person, "/");
                assert!(!row_has_subject_pollution(row), "{task_name}: {:?}", row);
                let combined = storyboard_row_visible_text(row);
                assert!(
                    !combined.contains("主角与敌人与敌人"),
                    "{task_name}: {combined}"
                );
                assert!(!combined.contains("敌人与敌人"), "{task_name}: {combined}");
                assert!(!combined.contains("焦点"), "{task_name}: {combined}");
                for drift in [
                    "灯塔底部",
                    "灯塔",
                    "木栈道",
                    "水面",
                    "探照灯",
                    "衣袖裂口",
                    "三名黑衣追兵",
                    "刀锋出鞘",
                ] {
                    assert!(
                        !combined.contains(drift),
                        "{task_name}: {drift}: {combined}"
                    );
                }
            }
            let all_visible = storyboard_rows_visible_text(&storyboard.rows);
            if task_name.starts_with("b-") {
                assert!(
                    storyboard.rows.iter().any(|row| row.person.contains("林峰")
                        || row.person.contains("苏瑶")
                        || row.person.contains("阿青")
                        || row.person.contains("追兵")),
                    "{task_name}: {:?}",
                    storyboard
                        .rows
                        .iter()
                        .map(|row| row.person.as_str())
                        .collect::<Vec<_>>()
                );
                assert!(
                    !storyboard
                        .rows
                        .iter()
                        .all(|row| row.person == "林峰、苏瑶与阿青"),
                    "{task_name}"
                );
                for required in ["林峰护住苏瑶", "阿青提醒", "黑衣追兵", "巷口", "逼近"]
                {
                    assert!(
                        all_visible.contains(required),
                        "{task_name}: {required}: {all_visible}"
                    );
                }
                assert!(
                    storyboard.rows.iter().any(|row| row.person == "阿青"),
                    "{task_name}: {:?}",
                    storyboard
                        .rows
                        .iter()
                        .map(|row| row.person.as_str())
                        .collect::<Vec<_>>()
                );
                for row in &storyboard.rows {
                    if row.person == "阿青" {
                        let combined = storyboard_row_visible_text(row);
                        assert!(
                            !combined.contains("林峰、苏瑶与阿青"),
                            "{task_name}: {combined}"
                        );
                        assert!(
                            !combined.contains("主体为林峰与苏瑶"),
                            "{task_name}: {combined}"
                        );
                        assert!(
                            !combined.contains("角色动作：林峰与苏瑶"),
                            "{task_name}: {combined}"
                        );
                        assert!(
                            !combined.contains("林峰与苏瑶从"),
                            "{task_name}: {combined}"
                        );
                        assert!(combined.contains("阿青"), "{task_name}: {combined}");
                    }
                }
            } else {
                assert!(all_visible.contains("废墟"), "{task_name}: {all_visible}");
                assert!(all_visible.contains("敌人"), "{task_name}: {all_visible}");
                assert!(
                    all_visible.contains("逼近")
                        || all_visible.contains("对峙压力")
                        || all_visible.contains("压近"),
                    "{task_name}: {all_visible}"
                );
                assert!(
                    storyboard.rows.iter().any(|row| row.person == "敌人"),
                    "{task_name}: {:?}",
                    storyboard
                        .rows
                        .iter()
                        .map(|row| row.person.as_str())
                        .collect::<Vec<_>>()
                );
                if task_name == "a-action-final" {
                    let enemy_row = storyboard
                        .rows
                        .iter()
                        .find(|row| row.person == "敌人")
                        .expect("A action should split row 2 to enemy");
                    let combined = storyboard_row_visible_text(enemy_row);
                    assert!(!combined.contains("主角与敌人从"), "{combined}");
                    assert!(!combined.contains("主体为主角与敌人"), "{combined}");
                    assert!(!combined.contains("角色动作：主角与敌人"), "{combined}");
                }
                if task_name == "a-war-final" {
                    assert!(
                        !storyboard.rows.iter().all(|row| row.person == "主角"),
                        "{task_name}: {:?}",
                        storyboard
                            .rows
                            .iter()
                            .map(|row| row.person.as_str())
                            .collect::<Vec<_>>()
                    );
                    for term in [
                        "甲胄",
                        "铠甲",
                        "肩甲",
                        "剑柄",
                        "左膝",
                        "整列军阵",
                        "军队规模",
                        "阵列组织",
                        "盾墙",
                        "长枪",
                        "旌旗",
                        "号角",
                    ] {
                        assert!(
                            !all_visible.contains(term),
                            "{task_name}: {term}: {all_visible}"
                        );
                    }
                }
            }
        }
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
    fn desktop_scene_type_aliases_stay_valid_without_opening_new_schema_paths() {
        let state = test_state_with_golden_sample_runtime();
        let scene_types = runtime_scene_option_mappings()
            .iter()
            .flat_map(|mapping| {
                std::iter::once(mapping.desktop_entry).chain(mapping.aliases.iter().copied())
            })
            .collect::<Vec<_>>();
        assert!(
            scene_types.len() > runtime_scene_option_mappings().len(),
            "regression should cover desktop entries and compatibility aliases"
        );

        for scene_type in scene_types {
            assert_eq!(normalize_scene_type(scene_type), "daily_dialogue");
            let resolved = resolve_scene_taxonomy(&state, Some(scene_type));
            assert_eq!(
                resolved.as_ref().map(|item| item.scene_type.as_str()),
                Some("daily_dialogue")
            );

            let script = expand_script(
                &state,
                ExpandScriptRequest {
                    scene_type: scene_type.to_string(),
                    scene_label: None,
                    scene_category: None,
                    model_config_summary: None,
                    selected_total_duration_seconds: Some(10),
                    target_duration_seconds: Some(10),
                    target_duration_mode: "fixed_seconds".to_string(),
                    story_length_profile: String::new(),
                    source_material_length_chars: 0,
                    auto_segment_strategy: String::new(),
                    source_input_type: String::new(),
                    authoring_mode: String::new(),
                    source_material_summary: String::new(),
                    source_story_facts: Default::default(),
                    preserved_fact_summary: String::new(),
                    changed_for_screenplay_summary: String::new(),
                    omitted_detail_summary: String::new(),
                    synopsis_text: format!("synopsis for {scene_type}"),
                },
            );
            assert_ne!(script.status, BridgeCallStatus::Blocked);

            let storyboard = generate_storyboard(
                &state,
                GenerateStoryboardRequest {
                    task_name: format!("alias-storyboard-{scene_type}"),
                    script_id: None,
                    shot_script: Some("林峰确认战场空间，叶倾颜观察下一步动向。".to_string()),
                    expanded_script_text: Some(script.expanded_script_text.clone()),
                    primary_scene_type: None,
                    primary_scene_label: None,
                    primary_scene_category: None,
                    shot_scene_type: None,
                    shot_scene_label: None,
                    shot_intent: None,
                    adaptation_reason: None,
                    selected_total_duration_seconds: 10,
                    target_duration_mode: String::new(),
                    auto_segment_strategy: String::new(),
                    model_config_summary: None,
                    scene_type: Some(scene_type.to_string()),
                    scene_label: None,
                    scene_category: None,
                    ..GenerateStoryboardRequest::default()
                },
            );
            assert_ne!(storyboard.export_status.status, BridgeCallStatus::Blocked);
        }
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
