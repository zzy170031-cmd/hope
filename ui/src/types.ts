export type ViewId = "workbench";

export type LoadStatus = "loading" | "ready" | "empty" | "error";

export type BridgeMode = "desktop" | "mock";

export type QwenRuntimeStatus =
  | "local_missing"
  | "local_checking"
  | "local_present"
  | "local_invalid";

export type WorkbenchModelId =
  | "gpt_4o"
  | "gpt_4_1_mini"
  | "hope_storyboard_mode";

export type SceneFusionOption =
  | "热血战斗"
  | "悬疑追踪"
  | "都市奇幻"
  | "校园日常"
  | "治愈成长";

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
