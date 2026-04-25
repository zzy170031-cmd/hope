export type ViewId = "workbench";

export type LoadStatus = "loading" | "ready" | "empty" | "error";

export type BridgeMode = "desktop" | "mock";

export type QwenRuntimeStatus =
  | "local_missing"
  | "local_checking"
  | "local_present"
  | "local_invalid";

export type WorkbenchModelId =
  | "qwen"
  | "doubao"
  | "custom";

export type SceneFusionOption =
  | "hot_blood_battle"
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
  | "slg_battle_report";

export interface StoryboardWorkbenchRow {
  id: string;
  order: number;
  person: string;
  shot: string;
  sceneScale: string;
  visualDescription: string;
  characterAction: string;
  dialogue: string;
  prompt: string;
  durationSeconds: number;
  backendRow?: GeneratedStoryboardRow;
}

export interface ProductWarning {
  code: string;
  message: string;
  related_sample_id?: string | null;
  relatedSampleId?: string | null;
}

export interface ExpandScriptRequest {
  scene_type: string;
  scene_label?: string;
  scene_category?: string;
  model_config_summary?: ModelConfigSummary;
  selected_total_duration_seconds?: number;
  synopsis_text: string;
}

export interface ModelConfigSummary {
  provider: WorkbenchModelId;
  model: string;
  enabled: boolean;
  base_url_present: boolean;
  api_key_present: boolean;
}

export type TextModelProviderUiStatus =
  | "unconfigured"
  | "configured_disabled"
  | "enabled"
  | "fallback"
  | "reserved";

export interface ConfigureTextModelProviderRequest {
  provider: WorkbenchModelId;
  model: string;
  base_url: string;
  api_key?: string | null;
  api_key_ref?: string | null;
  enabled: boolean;
}

export interface TextModelProviderStatus {
  provider: WorkbenchModelId;
  model: string;
  enabled: boolean;
  base_url_present: boolean;
  api_key_present: boolean;
  live_ready: boolean;
  status: TextModelProviderUiStatus;
  message: string;
  storage: "session-only";
}

export interface ExpandScriptResponse {
  script_id: string;
  expanded_script_text: string;
  script_hash: string;
  warnings: ProductWarning[];
  kb_router_result?: KbRouterRuntimeResponse | null;
}

export interface KbRouterSelectedRule {
  rule_id: string;
  family: string;
  summary: string;
  applies_to: string[];
}

export interface KbRouterRuntimeResponse {
  selected_sample_ids: string[];
  selected_kb_rules: KbRouterSelectedRule[];
  kb_context_summary: string;
  retrieval_trace?: unknown | null;
  full_kb_rows_included?: number;
}

export interface GenerateStoryboardRequest {
  task_name: string;
  script_id?: string | null;
  scene_type?: string | null;
  scene_label?: string | null;
  scene_category?: string | null;
  expanded_script_text?: string | null;
  selected_total_duration_seconds: number;
  model_config_summary?: ModelConfigSummary;
}

export type BridgeCallStatus = "Ready" | "WarningOnly" | "Blocked" | "Gated";

export interface PromptBodyCandidate {
  source_sample_id: string;
  source_prompt_body: string;
  candidate_text?: string | null;
  blocked: boolean;
  blocker_codes: string[];
}

export interface GeneratedStoryboardRow {
  shot_id: string;
  order: number;
  person: string;
  shot_title: string;
  scene_scale: string;
  visual_description: string;
  character_action: string;
  dialogue: string;
  prompt_text: string;
  prompt_text_compilation_status?: string;
  prompt_text_compilation_warnings?: ProductWarning[];
  prompt_text_source_row_id?: string;
  duration_seconds: number;
  prompt_body_candidate: PromptBodyCandidate;
}

export interface StoryboardDurationPlan {
  total_duration_seconds: number;
  row_count: number;
  per_row_seconds: number;
  allocated_seconds: number;
}

export interface StoryboardExportStatus {
  status: BridgeCallStatus;
  blockers: ProductWarning[];
  warnings: ProductWarning[];
  ready_row_count: number;
  blocked_row_count: number;
}

export interface GenerateStoryboardResponse {
  task_id?: string | null;
  result_id: string;
  rows: GeneratedStoryboardRow[];
  selected_total_duration_seconds?: number;
  duration_plan: StoryboardDurationPlan;
  export_status: StoryboardExportStatus;
  busy?: boolean;
  operation_id?: string;
  revision?: number;
  updated_at_ms?: number;
  rows_hash?: string;
  dirty?: boolean;
  dirty_source_note?: string | null;
  kb_router_result?: KbRouterRuntimeResponse | null;
}

export interface ExportBundleRequest {
  result_id?: string | null;
  task_id?: string | null;
  export_format: string;
}

export interface UpdateStoryboardRowsRequest {
  result_id: string;
  task_id?: string | null;
  rows: GeneratedStoryboardRow[];
  operation_id: string;
  base_revision: number;
  dirty_source_note?: string | null;
}

export interface ExportArtifactRecord {
  artifact_id: string;
  artifact_kind: string;
  export_format: string;
  ready: boolean;
  blocked_reason?: string | null;
  artifact_path?: string | null;
  content_hash?: string | null;
  byte_size?: number | null;
  row_count?: number | null;
  selected_total_duration_seconds?: number | null;
  source_result_id?: string | null;
  edited_rows_applied?: boolean;
  prompt_text_compilation_statuses?: string[];
  prompt_text_compilation_warning_codes?: string[];
  selected_sample_ids?: string[];
  selected_kb_rule_ids?: string[];
  kb_context_summary?: string | null;
  retrieval_trace?: unknown | null;
  full_kb_rows_included?: number;
}

export interface ExportBundleResponse {
  export_manifest_id: string;
  export_status: StoryboardExportStatus;
  artifacts: ExportArtifactRecord[];
}

export type StoryboardReadinessStatus =
  | "Draft"
  | "Needs Detail"
  | "Needs Polish"
  | "Validated"
  | "Ready to Export"
  | "Blocked";

export type StoryboardSemanticGroup =
  | "Visual Intent"
  | "Camera and Motion"
  | "Sound and Dialogue"
  | "Continuity and Handoff"
  | "Reference Locks"
  | "Delivery Prep";

export type WriterInputKind = "synopsis" | "script" | "brief";

export interface WriterInputBoundaryDraft {
  source_input_text: string;
  input_kind: WriterInputKind;
  story_constraints: string;
  character_constraints: string;
  style_constraints: string;
  duration_target: string;
  scene_count_hint: string;
}

export type ExportValidationState = "正常" | "待补充" | "阻塞";

export interface ProjectSummary {
  id: string;
  name: string;
  status: string;
  updatedAt: string;
  episodeCount: number;
}

export interface SnapshotBootstrapReadonlyStatus {
  snapshotIdentity: {
    snapshotId: string;
    snapshotHash: string;
    seedFormat: string;
    sourceName: string;
    createdAtTimestamp: number;
    snapshotPath: string;
  };
  summaryCapabilities: {
    hasSceneTaxonomy: boolean;
    hasFailurePatterns: boolean;
    hasRepairTemplateMapping: boolean;
  };
  knowledgeBundle: {
    sceneTaxonomyCount: number;
    failurePatternCount: number;
    promptTemplateCount: number;
    sceneTaxonomiesReady: boolean;
    failurePatternsReady: boolean;
    promptTemplatesReady: boolean;
    repairMappingsReady: boolean;
  };
}

export interface ValidationFeedbackReadonlyStatus {
  sourceSnapshotId: string;
  sourceSnapshotHash: string;
  sourceSnapshotPath: string;
  hasFailurePatterns: boolean;
  hasRepairTemplateMapping: boolean;
  failurePatternCount: number;
  promptTemplateCount: number;
  repairMappingReady: boolean;
}

export interface AppShellReadonlyStatus {
  snapshotBootstrap: SnapshotBootstrapReadonlyStatus;
  validationFeedback: ValidationFeedbackReadonlyStatus;
}

export interface WriterLayerSnapshot {
  synopsis: string;
  story: string;
  screenplay: string;
  storyboard: string;
}

export interface PreviewItem {
  id: string;
  label: string;
  duration: string;
  note: string;
}

export interface ExportValidationItem {
  label: string;
  value: string;
  state: ExportValidationState;
}

export interface ValidationRepairRecommendation {
  failureCode: string;
  failureName: string;
  repairStrategy: string;
  repairPriority: string;
  repairScope: string;
  validatorHint: string;
  promptTemplateNames: string[];
}

export interface ValidationExportPanelSnapshot {
  projectId: string;
  summaryItems: ExportValidationItem[];
  repairRecommendations: ValidationRepairRecommendation[];
}

export interface ProjectCreateOrSwitchRequest {
  project_id?: string;
  project_name?: string;
}

export interface WriterEntrySnapshotRequest {
  project_id: string;
}

export interface StoryboardRenderSegmentCutPreviewSnapshotRequest {
  project_id: string;
  episode_id?: string;
  narrative_scene_id?: string;
  render_segment_id?: string;
  scene_type?: string;
}

export interface ValidationExportPanelSnapshotRequest {
  project_id: string;
}
