import type {
  AppShellReadonlyStatus,
  BridgeMode,
  ExportValidationItem,
  PreviewItem,
  ProjectCreateOrSwitchRequest,
  ProjectSummary,
  StoryboardRenderSegmentCutPreviewSnapshotRequest,
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
} as const;

type HopeCommandName = (typeof HOPE_TAURI_COMMANDS)[keyof typeof HOPE_TAURI_COMMANDS];

type HopeCommandPayload =
  | ProjectCreateOrSwitchRequest
  | WriterEntrySnapshotRequest
  | StoryboardRenderSegmentCutPreviewSnapshotRequest
  | ValidationExportPanelSnapshotRequest
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
const PHASE1_REAL_COMMANDS = new Set<HopeCommandName>([
  HOPE_TAURI_COMMANDS.projectCreateOrSwitch,
  HOPE_TAURI_COMMANDS.writerEntrySnapshot,
  HOPE_TAURI_COMMANDS.storyboardRenderSegmentCutPreviewSnapshot,
  HOPE_TAURI_COMMANDS.validationExportPanelSnapshot,
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
  );
  return normalizeProjectList(raw);
}

export async function loadAppShellReadonlyStatus() {
  const raw = await invokeHopeCommand<unknown>(
    HOPE_TAURI_COMMANDS.projectCreateOrSwitch,
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
