import type {
  AppShellReadonlyStatus,
  BridgeMode,
  ConfigureTextModelProviderRequest,
  ExpandScriptRequest,
  ExpandScriptResponse,
  ExportBundleRequest,
  ExportBundleResponse,
  ExportArtifactRecord,
  ExportValidationItem,
  GenerateStoryboardRequest,
  GenerateStoryboardResponse,
  GeneratedStoryboardRow,
  KbRouterRuntimeResponse,
  KbRouterSelectedRule,
  PreviewItem,
  ProductWarning,
  ProjectCreateOrSwitchRequest,
  ProjectSummary,
  StoryboardRenderSegmentCutPreviewSnapshotRequest,
  TextModelProviderStatus,
  UpdateStoryboardRowsRequest,
  ValidationExportPanelSnapshot,
  ValidationExportPanelSnapshotRequest,
  ValidationRepairRecommendation,
  WriterEntrySnapshotRequest,
  WriterLayerSnapshot,
} from "../types";
import {
  MOCK_EXPORT_VALIDATION_SNAPSHOT,
  MOCK_PREVIEW_ITEMS,
  MOCK_PROJECTS,
  MOCK_WRITER_SNAPSHOT,
} from "../mock/mockState";

export const HOPE_TAURI_COMMANDS = {
  projectCreateOrSwitch: "project_create_or_switch",
  writerEntrySnapshot: "writer_entry_snapshot",
  storyboardRenderSegmentCutPreviewSnapshot:
    "storyboard_rendersegment_cut_preview_snapshot",
  validationExportPanelSnapshot: "validation_export_panel_snapshot",
  expandScript: "expand_script",
  generateStoryboard: "generate_storyboard",
  updateStoryboardRows: "update_storyboard_rows",
  exportBundle: "export_bundle",
  configureTextModelProvider: "configure_text_model_provider",
} as const;

type HopeCommandName = (typeof HOPE_TAURI_COMMANDS)[keyof typeof HOPE_TAURI_COMMANDS];

type HopeCommandPayload =
  | ProjectCreateOrSwitchRequest
  | WriterEntrySnapshotRequest
  | StoryboardRenderSegmentCutPreviewSnapshotRequest
  | ValidationExportPanelSnapshotRequest
  | ExpandScriptRequest
  | GenerateStoryboardRequest
  | UpdateStoryboardRowsRequest
  | ExportBundleRequest
  | ConfigureTextModelProviderRequest
  | undefined;

type DesktopInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

interface HopeDesktopWindow extends Window {
  __TAURI__?: {
    invoke?: DesktopInvoke;
    core?: {
      invoke?: DesktopInvoke;
    };
  };
  __HOPE_DESKTOP_BRIDGE__?: {
    invoke?: DesktopInvoke;
  };
}

export interface HopeBridgeStatus {
  mode: BridgeMode;
  label: string;
  detail: string;
}

const DEFAULT_PROJECT_ID = "project-week3-001";
const DEFAULT_PROJECT_REQUEST: ProjectCreateOrSwitchRequest = {};
const PHASE1_REAL_COMMANDS = new Set<HopeCommandName>([
  HOPE_TAURI_COMMANDS.projectCreateOrSwitch,
  HOPE_TAURI_COMMANDS.writerEntrySnapshot,
  HOPE_TAURI_COMMANDS.storyboardRenderSegmentCutPreviewSnapshot,
  HOPE_TAURI_COMMANDS.validationExportPanelSnapshot,
  HOPE_TAURI_COMMANDS.expandScript,
  HOPE_TAURI_COMMANDS.generateStoryboard,
  HOPE_TAURI_COMMANDS.updateStoryboardRows,
  HOPE_TAURI_COMMANDS.exportBundle,
  HOPE_TAURI_COMMANDS.configureTextModelProvider,
]);

function delay(ms: number) {
  return new Promise((resolve) => {
    window.setTimeout(resolve, ms);
  });
}

function resolveDesktopInvoke(): DesktopInvoke | null {
  const desktopWindow = window as HopeDesktopWindow;

  if (typeof desktopWindow.__HOPE_DESKTOP_BRIDGE__?.invoke === "function") {
    return desktopWindow.__HOPE_DESKTOP_BRIDGE__.invoke;
  }

  if (typeof desktopWindow.__TAURI__?.core?.invoke === "function") {
    return desktopWindow.__TAURI__.core.invoke;
  }

  if (typeof desktopWindow.__TAURI__?.invoke === "function") {
    return desktopWindow.__TAURI__.invoke;
  }

  return null;
}

async function invokeDesktopCommand<T>(
  invoke: DesktopInvoke,
  command: HopeCommandName,
  payload?: HopeCommandPayload,
): Promise<T> {
  if (!payload) {
    return invoke<T>(command);
  }

  try {
    return await invoke<T>(command, { request: payload as Record<string, unknown> });
  } catch {
    return invoke<T>(command, payload as Record<string, unknown>);
  }
}

function normalizeProjectSummary(raw: Record<string, unknown>): ProjectSummary {
  return {
    id: String(raw.id ?? raw.project_id ?? ""),
    name: String(raw.name ?? raw.title ?? raw.project_name ?? "未命名项目"),
    status: String(raw.status ?? "待处理"),
    updatedAt: String(raw.updatedAt ?? raw.updated_at ?? "等待桌面壳同步"),
    episodeCount: Number(raw.episodeCount ?? raw.episode_count ?? 0),
  };
}

function normalizeProjectList(raw: unknown): ProjectSummary[] {
  if (Array.isArray(raw)) {
    return raw.map((item) => normalizeProjectSummary(item as Record<string, unknown>));
  }

  if (
    raw &&
    typeof raw === "object" &&
    Array.isArray((raw as { projects?: unknown[] }).projects)
  ) {
    return (raw as { projects: unknown[] }).projects.map((item) =>
      normalizeProjectSummary(item as Record<string, unknown>),
    );
  }

  throw new Error("Unexpected project snapshot shape from desktop bridge.");
}

function readObject(raw: unknown, label: string): Record<string, unknown> {
  if (raw && typeof raw === "object") {
    return raw as Record<string, unknown>;
  }

  throw new Error(`Unexpected ${label} shape from desktop bridge.`);
}

function readNestedObject(
  raw: Record<string, unknown>,
  camelKey: string,
  snakeKey: string,
  label: string,
): Record<string, unknown> {
  return readObject(raw[camelKey] ?? raw[snakeKey], label);
}

function normalizeAppShellReadonlyStatus(raw: unknown): AppShellReadonlyStatus {
  const snapshot = readObject(raw, "app shell readonly status");
  const status = readNestedObject(
    snapshot,
    "readonlyStatus",
    "readonly_status",
    "readonly status",
  );
  const snapshotBootstrap = readNestedObject(
    status,
    "snapshotBootstrap",
    "snapshot_bootstrap",
    "snapshot bootstrap readonly status",
  );
  const snapshotIdentity = readNestedObject(
    snapshotBootstrap,
    "snapshotIdentity",
    "snapshot_identity",
    "snapshot identity",
  );
  const summaryCapabilities = readNestedObject(
    snapshotBootstrap,
    "summaryCapabilities",
    "summary_capabilities",
    "summary capabilities",
  );
  const knowledgeBundle = readNestedObject(
    snapshotBootstrap,
    "knowledgeBundle",
    "knowledge_bundle",
    "knowledge bundle",
  );
  const validationFeedback = readNestedObject(
    status,
    "validationFeedback",
    "validation_feedback",
    "validation feedback readonly status",
  );

  return {
    snapshotBootstrap: {
      snapshotIdentity: {
        snapshotId: String(snapshotIdentity.snapshotId ?? snapshotIdentity.snapshot_id ?? ""),
        snapshotHash: String(
          snapshotIdentity.snapshotHash ?? snapshotIdentity.snapshot_hash ?? "",
        ),
        seedFormat: String(snapshotIdentity.seedFormat ?? snapshotIdentity.seed_format ?? ""),
        sourceName: String(snapshotIdentity.sourceName ?? snapshotIdentity.source_name ?? ""),
        createdAtTimestamp: Number(
          snapshotIdentity.createdAtTimestamp ??
            snapshotIdentity.created_at_timestamp ??
            0,
        ),
        snapshotPath: String(snapshotIdentity.snapshotPath ?? snapshotIdentity.snapshot_path ?? ""),
      },
      summaryCapabilities: {
        hasSceneTaxonomy: Boolean(
          summaryCapabilities.hasSceneTaxonomy ??
            summaryCapabilities.has_scene_taxonomy,
        ),
        hasFailurePatterns: Boolean(
          summaryCapabilities.hasFailurePatterns ??
            summaryCapabilities.has_failure_patterns,
        ),
        hasRepairTemplateMapping: Boolean(
          summaryCapabilities.hasRepairTemplateMapping ??
            summaryCapabilities.has_repair_template_mapping,
        ),
      },
      knowledgeBundle: {
        sceneTaxonomyCount: Number(
          knowledgeBundle.sceneTaxonomyCount ??
            knowledgeBundle.scene_taxonomy_count ??
            0,
        ),
        failurePatternCount: Number(
          knowledgeBundle.failurePatternCount ??
            knowledgeBundle.failure_pattern_count ??
            0,
        ),
        promptTemplateCount: Number(
          knowledgeBundle.promptTemplateCount ??
            knowledgeBundle.prompt_template_count ??
            0,
        ),
        sceneTaxonomiesReady: Boolean(
          knowledgeBundle.sceneTaxonomiesReady ??
            knowledgeBundle.scene_taxonomies_ready,
        ),
        failurePatternsReady: Boolean(
          knowledgeBundle.failurePatternsReady ??
            knowledgeBundle.failure_patterns_ready,
        ),
        promptTemplatesReady: Boolean(
          knowledgeBundle.promptTemplatesReady ??
            knowledgeBundle.prompt_templates_ready,
        ),
        repairMappingsReady: Boolean(
          knowledgeBundle.repairMappingsReady ??
            knowledgeBundle.repair_mappings_ready,
        ),
      },
    },
    validationFeedback: {
      sourceSnapshotId: String(
        validationFeedback.sourceSnapshotId ??
          validationFeedback.source_snapshot_id ??
          "",
      ),
      sourceSnapshotHash: String(
        validationFeedback.sourceSnapshotHash ??
          validationFeedback.source_snapshot_hash ??
          "",
      ),
      sourceSnapshotPath: String(
        validationFeedback.sourceSnapshotPath ??
          validationFeedback.source_snapshot_path ??
          "",
      ),
      hasFailurePatterns: Boolean(
        validationFeedback.hasFailurePatterns ??
          validationFeedback.has_failure_patterns,
      ),
      hasRepairTemplateMapping: Boolean(
        validationFeedback.hasRepairTemplateMapping ??
          validationFeedback.has_repair_template_mapping,
      ),
      failurePatternCount: Number(
        validationFeedback.failurePatternCount ??
          validationFeedback.failure_pattern_count ??
          0,
      ),
      promptTemplateCount: Number(
        validationFeedback.promptTemplateCount ??
          validationFeedback.prompt_template_count ??
          0,
      ),
      repairMappingReady: Boolean(
        validationFeedback.repairMappingReady ??
          validationFeedback.repair_mapping_ready,
      ),
    },
  };
}

function normalizeWriterSnapshot(raw: unknown): WriterLayerSnapshot {
  const snapshot = raw as Record<string, unknown>;

  return {
    synopsis: String(snapshot.synopsis ?? ""),
    story: String(snapshot.story ?? ""),
    screenplay: String(snapshot.screenplay ?? ""),
    storyboard: String(snapshot.storyboard ?? ""),
  };
}

function normalizePreviewItem(raw: Record<string, unknown>): PreviewItem {
  return {
    id: String(raw.id ?? ""),
    label: String(raw.label ?? "未命名条目"),
    duration: String(raw.duration ?? ""),
    note: String(raw.note ?? ""),
  };
}

function normalizePreviewSnapshot(
  raw: unknown,
): Record<"storyboard" | "renderSegment" | "cuts", PreviewItem[]> {
  const snapshot = raw as Record<string, unknown>;
  const storyboard = Array.isArray(snapshot.storyboard) ? snapshot.storyboard : [];
  const renderSegment = Array.isArray(snapshot.renderSegment)
    ? snapshot.renderSegment
    : Array.isArray(snapshot.render_segment)
      ? snapshot.render_segment
      : [];
  const cuts = Array.isArray(snapshot.cuts) ? snapshot.cuts : [];

  return {
    storyboard: storyboard.map((item) => normalizePreviewItem(item as Record<string, unknown>)),
    renderSegment: renderSegment.map((item) =>
      normalizePreviewItem(item as Record<string, unknown>),
    ),
    cuts: cuts.map((item) => normalizePreviewItem(item as Record<string, unknown>)),
  };
}

function normalizeValidationState(state: unknown): ExportValidationItem["state"] {
  switch (String(state)) {
    case "Ready":
    case "正常":
      return "正常";
    case "Blocked":
    case "阻塞":
    case "待对齐":
      return "阻塞";
    case "Pending":
    case "待补充":
    default:
      return "待补充";
  }
}

function normalizeValidationLabel(label: unknown): string {
  switch (String(label)) {
    case "project":
      return "当前项目";
    case "validation_report":
      return "Validation 报告";
    case "repair_recommendations":
      return "修复建议";
    default:
      return String(label ?? "未命名项");
  }
}

function normalizeValidationSummaryItem(raw: Record<string, unknown>): ExportValidationItem {
  return {
    label: normalizeValidationLabel(raw.label),
    value: String(raw.value ?? ""),
    state: normalizeValidationState(raw.state),
  };
}

function normalizeRepairRecommendation(
  raw: Record<string, unknown>,
): ValidationRepairRecommendation {
  return {
    failureCode: String(raw.failureCode ?? raw.failure_code ?? ""),
    failureName: String(raw.failureName ?? raw.failure_name ?? ""),
    repairStrategy: String(raw.repairStrategy ?? raw.repair_strategy ?? ""),
    repairPriority: String(raw.repairPriority ?? raw.repair_priority ?? ""),
    repairScope: String(raw.repairScope ?? raw.repair_scope ?? ""),
    validatorHint: String(raw.validatorHint ?? raw.validator_hint ?? ""),
    promptTemplateNames: Array.isArray(raw.promptTemplateNames)
      ? raw.promptTemplateNames.map((item) => String(item))
      : Array.isArray(raw.prompt_template_names)
        ? raw.prompt_template_names.map((item) => String(item))
        : [],
  };
}

function normalizeValidationExportSnapshot(raw: unknown): ValidationExportPanelSnapshot {
  if (Array.isArray(raw)) {
    return {
      projectId: DEFAULT_PROJECT_ID,
      summaryItems: raw.map((item) =>
        normalizeValidationSummaryItem(item as Record<string, unknown>),
      ),
      repairRecommendations: [],
    };
  }

  const snapshot = raw as Record<string, unknown>;
  const summaryItems = Array.isArray(snapshot.summaryItems)
    ? snapshot.summaryItems
    : Array.isArray(snapshot.summary_items)
      ? snapshot.summary_items
      : [];
  const repairRecommendations = Array.isArray(snapshot.repairRecommendations)
    ? snapshot.repairRecommendations
    : Array.isArray(snapshot.repair_recommendations)
      ? snapshot.repair_recommendations
      : [];

  return {
    projectId: String(snapshot.projectId ?? snapshot.project_id ?? DEFAULT_PROJECT_ID),
    summaryItems: summaryItems.map((item) =>
      normalizeValidationSummaryItem(item as Record<string, unknown>),
    ),
    repairRecommendations: repairRecommendations.map((item) =>
      normalizeRepairRecommendation(item as Record<string, unknown>),
    ),
  };
}

function normalizeProductWarning(raw: unknown): ProductWarning {
  const warning = readObject(raw, "product warning");
  return {
    code: String(warning.code ?? ""),
    message: String(warning.message ?? ""),
    related_sample_id: (warning.related_sample_id ?? warning.relatedSampleId ?? null) as
      | string
      | null,
  };
}

function normalizeWarningList(raw: unknown): ProductWarning[] {
  return Array.isArray(raw) ? raw.map((item) => normalizeProductWarning(item)) : [];
}

function normalizeKbRouterSelectedRule(raw: unknown): KbRouterSelectedRule {
  const rule = readObject(raw, "kb router selected rule");
  return {
    rule_id: String(rule.rule_id ?? rule.ruleId ?? ""),
    family: String(rule.family ?? ""),
    summary: String(rule.summary ?? ""),
    applies_to: Array.isArray(rule.applies_to)
      ? rule.applies_to.map(String)
      : Array.isArray(rule.appliesTo)
        ? rule.appliesTo.map(String)
        : [],
  };
}

function normalizeKbRouterResult(raw: unknown): KbRouterRuntimeResponse | null {
  if (!raw || typeof raw !== "object") {
    return null;
  }

  const router = readObject(raw, "kb router result");
  const retrievalTrace = router.retrieval_trace ?? router.retrievalTrace ?? null;
  const traceObject = retrievalTrace && typeof retrievalTrace === "object"
    ? readObject(retrievalTrace, "kb router retrieval trace")
    : null;
  const tokenBudget = traceObject && (traceObject.token_budget ?? traceObject.tokenBudget)
    ? readObject(traceObject.token_budget ?? traceObject.tokenBudget, "kb router token budget")
    : null;

  return {
    selected_sample_ids: Array.isArray(router.selected_sample_ids)
      ? router.selected_sample_ids.map(String)
      : Array.isArray(router.selectedSampleIds)
        ? router.selectedSampleIds.map(String)
        : [],
    selected_kb_rules: Array.isArray(router.selected_kb_rules)
      ? router.selected_kb_rules.map((item) => normalizeKbRouterSelectedRule(item))
      : Array.isArray(router.selectedKbRules)
        ? router.selectedKbRules.map((item) => normalizeKbRouterSelectedRule(item))
        : [],
    kb_context_summary: String(router.kb_context_summary ?? router.kbContextSummary ?? ""),
    retrieval_trace: retrievalTrace,
    full_kb_rows_included: Number(
      router.full_kb_rows_included ??
        router.fullKbRowsIncluded ??
        tokenBudget?.full_kb_rows_included ??
        tokenBudget?.fullKbRowsIncluded ??
        0,
    ),
  };
}

function normalizeExpandScriptResponse(raw: unknown): ExpandScriptResponse {
  const response = readObject(raw, "expand_script response");
  return {
    script_id: String(response.script_id ?? response.scriptId ?? ""),
    expanded_script_text: String(
      response.expanded_script_text ?? response.expandedScriptText ?? "",
    ),
    script_hash: String(response.script_hash ?? response.scriptHash ?? ""),
    warnings: normalizeWarningList(response.warnings),
    kb_router_result: normalizeKbRouterResult(
      response.kb_router_result ?? response.kbRouterResult ?? response,
    ),
  };
}

function normalizeGeneratedStoryboardRow(raw: unknown): GeneratedStoryboardRow {
  const row = readObject(raw, "generated storyboard row");
  const promptBodyCandidate = readObject(
    row.prompt_body_candidate ?? row.promptBodyCandidate ?? {},
    "prompt body candidate",
  );

  return {
    shot_id: String(row.shot_id ?? row.shotId ?? ""),
    order: Number(row.order ?? 0),
    person: String(row.person ?? ""),
    shot_title: String(row.shot_title ?? row.shotTitle ?? ""),
    scene_scale: String(row.scene_scale ?? row.sceneScale ?? ""),
    visual_description: String(row.visual_description ?? row.visualDescription ?? ""),
    character_action: String(row.character_action ?? row.characterAction ?? ""),
    dialogue: String(row.dialogue ?? ""),
    prompt_text: String(row.prompt_text ?? row.promptText ?? ""),
    prompt_text_compilation_status: String(
      row.prompt_text_compilation_status ?? row.promptTextCompilationStatus ?? "",
    ),
    prompt_text_compilation_warnings: normalizeWarningList(
      row.prompt_text_compilation_warnings ?? row.promptTextCompilationWarnings,
    ),
    prompt_text_source_row_id: String(
      row.prompt_text_source_row_id ?? row.promptTextSourceRowId ?? "",
    ),
    duration_seconds: Number(row.duration_seconds ?? row.durationSeconds ?? 0),
    prompt_body_candidate: {
      source_sample_id: String(
        promptBodyCandidate.source_sample_id ?? promptBodyCandidate.sourceSampleId ?? "",
      ),
      source_prompt_body: String(
        promptBodyCandidate.source_prompt_body ?? promptBodyCandidate.sourcePromptBody ?? "",
      ),
      candidate_text:
        (promptBodyCandidate.candidate_text ?? promptBodyCandidate.candidateText ?? null) as
          | string
          | null,
      blocked: Boolean(promptBodyCandidate.blocked),
      blocker_codes: Array.isArray(promptBodyCandidate.blocker_codes)
        ? promptBodyCandidate.blocker_codes.map(String)
        : Array.isArray(promptBodyCandidate.blockerCodes)
          ? promptBodyCandidate.blockerCodes.map(String)
          : [],
    },
  };
}

function normalizeStoryboardExportStatus(raw: unknown) {
  const status = readObject(raw, "storyboard export status");
  return {
    status: String(status.status ?? "Blocked") as GenerateStoryboardResponse["export_status"]["status"],
    blockers: normalizeWarningList(status.blockers),
    warnings: normalizeWarningList(status.warnings),
    ready_row_count: Number(status.ready_row_count ?? status.readyRowCount ?? 0),
    blocked_row_count: Number(status.blocked_row_count ?? status.blockedRowCount ?? 0),
  };
}

function normalizeGenerateStoryboardResponse(raw: unknown): GenerateStoryboardResponse {
  const response = readObject(raw, "generate_storyboard response");
  const durationPlan = readObject(
    response.duration_plan ?? response.durationPlan ?? {},
    "storyboard duration plan",
  );

  return {
    task_id: (response.task_id ?? response.taskId ?? null) as string | null,
    result_id: String(response.result_id ?? response.resultId ?? ""),
    rows: Array.isArray(response.rows)
      ? response.rows.map((item) => normalizeGeneratedStoryboardRow(item))
      : [],
    selected_total_duration_seconds: Number(
      response.selected_total_duration_seconds ?? response.selectedTotalDurationSeconds ?? 0,
    ),
    duration_plan: {
      total_duration_seconds: Number(
        durationPlan.total_duration_seconds ?? durationPlan.totalDurationSeconds ?? 0,
      ),
      row_count: Number(durationPlan.row_count ?? durationPlan.rowCount ?? 0),
      per_row_seconds: Number(durationPlan.per_row_seconds ?? durationPlan.perRowSeconds ?? 0),
      allocated_seconds: Number(durationPlan.allocated_seconds ?? durationPlan.allocatedSeconds ?? 0),
    },
    export_status: normalizeStoryboardExportStatus(
      response.export_status ?? response.exportStatus ?? {},
    ),
    busy: Boolean(response.busy),
    operation_id: String(response.operation_id ?? response.operationId ?? ""),
    revision: Number(response.revision ?? 0),
    updated_at_ms: Number(response.updated_at_ms ?? response.updatedAtMs ?? 0),
    rows_hash: String(response.rows_hash ?? response.rowsHash ?? ""),
    dirty: Boolean(response.dirty),
    dirty_source_note: (response.dirty_source_note ?? response.dirtySourceNote ?? null) as
      | string
      | null,
    kb_router_result: normalizeKbRouterResult(
      response.kb_router_result ?? response.kbRouterResult ?? response,
    ),
  };
}

function normalizeExportArtifact(raw: unknown): ExportArtifactRecord {
  const artifact = readObject(raw, "export artifact");
  return {
    artifact_id: String(artifact.artifact_id ?? artifact.artifactId ?? ""),
    artifact_kind: String(artifact.artifact_kind ?? artifact.artifactKind ?? ""),
    export_format: String(artifact.export_format ?? artifact.exportFormat ?? ""),
    ready: Boolean(artifact.ready),
    blocked_reason: (artifact.blocked_reason ?? artifact.blockedReason ?? null) as string | null,
    artifact_path: (artifact.artifact_path ?? artifact.artifactPath ?? null) as string | null,
    content_hash: (artifact.content_hash ?? artifact.contentHash ?? null) as string | null,
    byte_size: (artifact.byte_size ?? artifact.byteSize ?? null) as number | null,
    row_count: (artifact.row_count ?? artifact.rowCount ?? null) as number | null,
    selected_total_duration_seconds: (artifact.selected_total_duration_seconds ??
      artifact.selectedTotalDurationSeconds ??
      null) as number | null,
    source_result_id: (artifact.source_result_id ?? artifact.sourceResultId ?? null) as
      | string
      | null,
    edited_rows_applied: Boolean(
      artifact.edited_rows_applied ?? artifact.editedRowsApplied,
    ),
    prompt_text_compilation_statuses: Array.isArray(artifact.prompt_text_compilation_statuses)
      ? artifact.prompt_text_compilation_statuses.map(String)
      : Array.isArray(artifact.promptTextCompilationStatuses)
        ? artifact.promptTextCompilationStatuses.map(String)
        : [],
    prompt_text_compilation_warning_codes: Array.isArray(
      artifact.prompt_text_compilation_warning_codes,
    )
      ? artifact.prompt_text_compilation_warning_codes.map(String)
      : Array.isArray(artifact.promptTextCompilationWarningCodes)
        ? artifact.promptTextCompilationWarningCodes.map(String)
        : [],
    selected_sample_ids: Array.isArray(artifact.selected_sample_ids)
      ? artifact.selected_sample_ids.map(String)
      : Array.isArray(artifact.selectedSampleIds)
        ? artifact.selectedSampleIds.map(String)
        : [],
    selected_kb_rule_ids: Array.isArray(artifact.selected_kb_rule_ids)
      ? artifact.selected_kb_rule_ids.map(String)
      : Array.isArray(artifact.selectedKbRuleIds)
        ? artifact.selectedKbRuleIds.map(String)
        : [],
    kb_context_summary: (artifact.kb_context_summary ?? artifact.kbContextSummary ?? null) as
      | string
      | null,
    retrieval_trace: artifact.retrieval_trace ?? artifact.retrievalTrace ?? null,
    full_kb_rows_included: Number(
      artifact.full_kb_rows_included ?? artifact.fullKbRowsIncluded ?? 0,
    ),
  };
}

function normalizeExportBundleResponse(raw: unknown): ExportBundleResponse {
  const response = readObject(raw, "export_bundle response");
  return {
    export_manifest_id: String(response.export_manifest_id ?? response.exportManifestId ?? ""),
    export_status: normalizeStoryboardExportStatus(
      response.export_status ?? response.exportStatus ?? {},
    ),
    artifacts: Array.isArray(response.artifacts)
      ? response.artifacts.map((item) => normalizeExportArtifact(item))
      : [],
  };
}

function normalizeTextModelProviderStatus(raw: unknown): TextModelProviderStatus {
  const status = readObject(raw, "text model provider status");
  return {
    provider: String(status.provider ?? "qwen") as TextModelProviderStatus["provider"],
    model: String(status.model ?? "qwen-plus"),
    enabled: Boolean(status.enabled),
    base_url_present: Boolean(status.base_url_present ?? status.baseUrlPresent),
    api_key_present: Boolean(status.api_key_present ?? status.apiKeyPresent),
    live_ready: Boolean(status.live_ready ?? status.liveReady),
    status: String(status.status ?? "unconfigured") as TextModelProviderStatus["status"],
    message: String(status.message ?? ""),
    storage: "session-only",
  };
}

function fallbackForCommand<T>(command: HopeCommandName, payload?: HopeCommandPayload): T {
  switch (command) {
    case HOPE_TAURI_COMMANDS.projectCreateOrSwitch:
      return MOCK_PROJECTS as T;
    case HOPE_TAURI_COMMANDS.writerEntrySnapshot:
      return MOCK_WRITER_SNAPSHOT as T;
    case HOPE_TAURI_COMMANDS.storyboardRenderSegmentCutPreviewSnapshot:
      return MOCK_PREVIEW_ITEMS as T;
    case HOPE_TAURI_COMMANDS.validationExportPanelSnapshot:
      return {
        ...MOCK_EXPORT_VALIDATION_SNAPSHOT,
        projectId:
          (payload as ValidationExportPanelSnapshotRequest | undefined)?.project_id ??
          MOCK_EXPORT_VALIDATION_SNAPSHOT.projectId,
      } as T;
    case HOPE_TAURI_COMMANDS.expandScript:
    case HOPE_TAURI_COMMANDS.generateStoryboard:
    case HOPE_TAURI_COMMANDS.updateStoryboardRows:
    case HOPE_TAURI_COMMANDS.exportBundle:
      throw new Error(`${command} requires the desktop bridge.`);
    case HOPE_TAURI_COMMANDS.configureTextModelProvider: {
      const request = payload as ConfigureTextModelProviderRequest | undefined;
      const provider = request?.provider ?? "qwen";
      const qwenEnabled = provider === "qwen" && Boolean(request?.enabled);
      const apiKeyPresent =
        Boolean(request?.api_key?.trim()) || Boolean(request?.api_key_ref?.trim());
      const baseUrlPresent = Boolean(request?.base_url?.trim());
      return {
        provider,
        model: request?.model || "qwen-plus",
        enabled: qwenEnabled,
        base_url_present: baseUrlPresent,
        api_key_present: apiKeyPresent,
        live_ready: qwenEnabled && apiKeyPresent && baseUrlPresent,
        status:
          provider !== "qwen"
            ? "reserved"
            : qwenEnabled && apiKeyPresent && baseUrlPresent
              ? "enabled"
              : "unconfigured",
        message:
          provider !== "qwen"
            ? "该模型接口为预留状态，当前未启用真实调用。"
            : "浏览器预览模式：配置仅在当前页面会话中模拟。",
        storage: "session-only",
      } as T;
    }
    default:
      throw new Error(`Unmapped Hope UI command: ${command}`);
  }
}

export function getHopeBridgeStatus(): HopeBridgeStatus {
  if (resolveDesktopInvoke()) {
    return {
      mode: "desktop",
      label: "Desktop IPC first",
      detail: "Detected desktop bridge and will prefer IPC before any fallback.",
    };
  }

  return {
    mode: "mock",
    label: "Mock fallback active",
    detail: "Running in browser mode, using fixture-backed fallback data.",
  };
}

export async function invokeHopeCommand<T>(
  command: HopeCommandName,
  payload?: HopeCommandPayload,
): Promise<T> {
  const desktopInvoke = PHASE1_REAL_COMMANDS.has(command) ? resolveDesktopInvoke() : null;

  if (desktopInvoke) {
    try {
      return await invokeDesktopCommand<T>(desktopInvoke, command, payload);
    } catch (error) {
      console.warn(
        `[hopeBridge] desktop invoke failed for ${command}, falling back to mock data`,
        error,
      );
    }
  }

  await delay(120);
  return fallbackForCommand<T>(command, payload);
}

export async function loadProjectList() {
  const raw = await invokeHopeCommand<unknown>(
    HOPE_TAURI_COMMANDS.projectCreateOrSwitch,
    DEFAULT_PROJECT_REQUEST,
  );
  return normalizeProjectList(raw);
}

export async function loadAppShellReadonlyStatus() {
  const raw = await invokeHopeCommand<unknown>(
    HOPE_TAURI_COMMANDS.projectCreateOrSwitch,
    DEFAULT_PROJECT_REQUEST,
  );
  return normalizeAppShellReadonlyStatus(raw);
}

export async function loadWriterSnapshot(project_id = DEFAULT_PROJECT_ID) {
  const raw = await invokeHopeCommand<unknown>(
    HOPE_TAURI_COMMANDS.writerEntrySnapshot,
    { project_id },
  );
  return normalizeWriterSnapshot(raw);
}

export async function loadPreviewSnapshot(project_id = DEFAULT_PROJECT_ID) {
  const raw = await invokeHopeCommand<unknown>(
    HOPE_TAURI_COMMANDS.storyboardRenderSegmentCutPreviewSnapshot,
    { project_id },
  );
  return normalizePreviewSnapshot(raw);
}

export async function loadExportValidationSnapshot(project_id = DEFAULT_PROJECT_ID) {
  const raw = await invokeHopeCommand<unknown>(
    HOPE_TAURI_COMMANDS.validationExportPanelSnapshot,
    { project_id },
  );
  return normalizeValidationExportSnapshot(raw);
}

export async function expandScript(request: ExpandScriptRequest) {
  const raw = await invokeHopeCommand<unknown>(HOPE_TAURI_COMMANDS.expandScript, request);
  return normalizeExpandScriptResponse(raw);
}

export async function generateStoryboard(request: GenerateStoryboardRequest) {
  const raw = await invokeHopeCommand<unknown>(
    HOPE_TAURI_COMMANDS.generateStoryboard,
    request,
  );
  return normalizeGenerateStoryboardResponse(raw);
}

export async function updateStoryboardRows(request: UpdateStoryboardRowsRequest) {
  const raw = await invokeHopeCommand<unknown>(
    HOPE_TAURI_COMMANDS.updateStoryboardRows,
    request,
  );
  return normalizeGenerateStoryboardResponse(raw);
}

export async function exportBundle(request: ExportBundleRequest) {
  const raw = await invokeHopeCommand<unknown>(HOPE_TAURI_COMMANDS.exportBundle, request);
  return normalizeExportBundleResponse(raw);
}

export async function configureTextModelProvider(
  request: ConfigureTextModelProviderRequest,
) {
  const raw = await invokeHopeCommand<unknown>(
    HOPE_TAURI_COMMANDS.configureTextModelProvider,
    request,
  );
  return normalizeTextModelProviderStatus(raw);
}
