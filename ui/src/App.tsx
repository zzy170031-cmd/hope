import { useEffect, useMemo, useState } from "react";
import { Shell } from "./components/Shell";
import logoUrl from "./assets/hope-desktop-logo.png";
import {
  configureTextModelProvider,
  copyExportArtifactToPath,
  expandScript as invokeExpandScript,
  exportBundle as invokeExportBundle,
  exportStoryboardBank as invokeExportStoryboardBank,
  generateStoryboard as invokeGenerateStoryboard,
  getHopeBridgeStatus,
  getTextModelProviderStatus,
  importStoryDocument as invokeImportStoryDocument,
  listStoryboardShotResults as invokeListStoryboardShotResults,
  removeStoryboardShotResult as invokeRemoveStoryboardShotResult,
  saveStoryboardShotResult as invokeSaveStoryboardShotResult,
  selectExportSavePath,
  updateStoryboardShotResult as invokeUpdateStoryboardShotResult,
  updateStoryboardRows as invokeUpdateStoryboardRows,
} from "./bridge/hopeBridge";
import { ROUTES, resolveRoute } from "./routes";
import type {
  ExpandScriptResponse,
  ExportBundleResponse,
  ExportStoryboardBankResponse,
  FinalizedStoryboardShotResult,
  GenerateStoryboardResponse,
  GeneratedStoryboardRow,
  ModelConfigSummary,
  ProductWarning,
  SceneFusionOption,
  SourceStoryFacts,
  StoryboardExportStatus,
  StoryboardWorkbenchRow,
  TargetDurationMode,
  TextModelProviderStatus,
  ViewId,
  WorkbenchModelId,
} from "./types";

type HeaderPanel = "none" | "docs" | "api";

interface Option<T extends string> {
  value: T;
  label: string;
}

interface SceneOption extends Option<SceneFusionOption> {
  group: string;
}

interface ModelConfigState {
  provider: WorkbenchModelId;
  model: string;
  baseUrl: string;
  apiKeyRef: string;
  enabled: boolean;
  apiKeyPresent: boolean;
}

interface ModelConfigDraft extends ModelConfigState {
  apiKeyInput: string;
}

type LongTextField =
  | "person"
  | "shot"
  | "cameraMovement"
  | "visualDescription"
  | "characterAction"
  | "dialogue"
  | "prompt";

interface TextDialogState {
  kind: "synopsis" | "expandedScript" | "storyboardCell" | "summary";
  title: string;
  value: string;
  helper: string;
  editable: boolean;
  rowId?: string;
  field?: LongTextField;
}

interface ScriptDialogBlock {
  label: string;
  body: string;
  synthetic?: boolean;
}

interface ShotCandidate {
  id: string;
  sceneIndex: number;
  title: string;
  text: string;
  preview: string;
  durationSeconds: number;
}

type TaskDraftScriptTextSource = "candidate" | "task" | "manual";

interface TaskDraftManualEditedFields {
  name: boolean;
  duration: boolean;
  scriptText: boolean;
}

interface TaskDraftState {
  mode: "create" | "update";
  sourceKind: "candidate" | "task";
  editingTaskRecordId: string | null;
  taskName: string;
  selectedCandidateId: string;
  sceneIndex: number;
  shotIndexWithinScene: number;
  segmentTitle: string;
  scriptText: string;
  durationSeconds: number;
  baseSegmentTitle: string;
  baseScriptText: string;
  baseDurationSeconds: number;
  scriptTextSource: TaskDraftScriptTextSource;
  manualEditedFields: TaskDraftManualEditedFields;
  durationHint: string;
}

interface GeneratedSceneTask {
  resultId: string;
  taskId?: string | null;
  taskName: string;
  segmentTitle: string;
  rowCount: number;
}

interface SceneTaskRecord {
  id: string;
  name: string;
  candidateId: string;
  sceneIndex: number;
  shotIndexWithinScene: number;
  segmentTitle: string;
  scriptText: string;
  baseSceneText: string;
  scriptId?: string | null;
  sourceSceneType?: SceneFusionOption | string | null;
  sourceSceneLabel?: string | null;
  sourceSceneCategory?: string | null;
  sourceDurationSeconds?: number | null;
  sourceDurationMode?: TargetDurationMode;
  status: "draft" | "generated";
  taskId?: string | null;
  resultId?: string;
  rowCount?: number;
  rowsSnapshot?: StoryboardWorkbenchRow[];
  storyboardResult?: GenerateStoryboardResponse | null;
  promptTextStatus?: string[];
  promptTextWarnings?: string[];
  selectedTotalDurationSeconds?: number;
  rowsHash?: string;
  updatedAtMs?: number;
  baseRevision?: number;
  dirty?: boolean;
  hadRowEdits?: boolean;
  exportArtifactPath?: string;
  exportStatus?: string;
  durationPlanSummary?: string;
  generatedShotTaskCount?: number;
  scriptTextSource?: TaskDraftScriptTextSource;
  userEditedFields?: TaskDraftManualEditedFields;
}

type SourceInputType =
  | "synopsis"
  | "full_story"
  | "novel_chapter"
  | "screenplay_text"
  | "mixed_material";

interface SourceInputAnalysis {
  sourceInputType: SourceInputType;
  authoringMode: string;
  sourceMaterialSummary: string;
  sourceStoryFacts: SourceStoryFacts;
  preservedFactSummary: string;
  changedForScreenplaySummary: string;
  omittedDetailSummary: string;
  statusMessage: string;
  actionLabel: string;
  sourceMaterialLengthChars: number;
  recommendsLongTextMode: boolean;
}

const DEFAULT_STORYBOARD_PAGE_SIZE = 5;
const STORYBOARD_PAGE_SIZE_OPTIONS = [3, 4, 5, 6, 10, 20];
const DURATION_OPTIONS = [5, 10, 15, 30, 45, 60];
const TASK_DRAFT_DURATION_HINT = "时长只用于当前任务的生成与导出标记，不会自动改写正文。";
const FIXED_DURATION_MODE: TargetDurationMode = "fixed_seconds";
const LONG_TEXT_DURATION_MODE: TargetDurationMode = "long_text_auto";
const AUTO_SEGMENT_STRATEGY_LONG_TEXT = "long_text_auto_story_fact_segments";
const AUTO_SEGMENT_STRATEGY_FIXED_SECONDS = "fixed_seconds_user_selected";
const LONG_TEXT_RECOMMENDATION_MESSAGE = "已识别为长文本，将按剧情自动分段；单镜头分镜保持短镜头友好时长。";
const DEFAULT_SYNOPSIS = "主角在废墟城市中与敌人激烈战斗，最终觉醒新力量，击败敌人。";
const DEFAULT_SCENE: SceneFusionOption = "hot_blood_battle";
const DEFAULT_TASK_NAME = "第一集分镜生成";
const DEFAULT_PROJECT_ID = "project-week3-001";
const STORYBOARD_DURATION_SOURCE = "storyboard_duration_plan.allocated_row_duration_seconds";
const EMPTY_PROMPT_TEXT_PLACEHOLDER = "分镜提示词未生成，等待主线镜头校准";
const EMPTY_CAMERA_MOVEMENT_PLACEHOLDER = "运镜未完整生成，等待主线运镜 grounding";
const STORYBOARD_ROWS_HASH_MISMATCH_MESSAGE =
  "当前分镜内容已变化，请先保存修改或重新生成后再确定使用。";
const SCENE_SCALE_LABELS: Record<string, string> = {
  LS: "远景",
  WS: "全景",
  MS: "中景",
  MCU: "中近景",
  CU: "特写",
  ECU: "大特写",
  OTS: "过肩镜头",
  source: "原场景",
};

const STORYBOARD_DIALOG_FIELD_LABELS = [
  "视频分镜提示词",
  "分镜提示词",
  "镜头脚本",
  "镜头标题",
  "景别",
  "运镜",
  "画面描述",
  "角色动作",
  "对白/旁白",
  "对白",
  "旁白",
  "时长",
];

const MODEL_OPTIONS: Array<Option<WorkbenchModelId>> = [
  { value: "qwen", label: "千问 Qwen" },
  { value: "doubao", label: "豆包 Doubao（预留）" },
  { value: "custom", label: "自定义模型（预留）" },
];

const MODEL_DEFAULTS: Record<WorkbenchModelId, Pick<ModelConfigState, "model" | "baseUrl">> = {
  qwen: {
    model: "qwen-plus",
    baseUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1",
  },
  doubao: {
    model: "doubao-seed-reserved",
    baseUrl: "https://ark.cn-beijing.volces.com/api/v3",
  },
  custom: {
    model: "custom-text-model",
    baseUrl: "",
  },
};

const API_KEY_REF_OPTIONS = [
  { value: "", label: "不使用引用" },
  { value: "env:HOPE_TEXT_MODEL_API_KEY", label: "env:HOPE_TEXT_MODEL_API_KEY" },
  { value: "env:QWEN_API_KEY", label: "env:QWEN_API_KEY" },
  { value: "env:DOUBAO_API_KEY", label: "env:DOUBAO_API_KEY" },
  { value: "env:CUSTOM_MODEL_API_KEY", label: "env:CUSTOM_MODEL_API_KEY" },
  { value: "keychain:hope-model-api-key", label: "keychain:hope-model-api-key" },
];

const DEFAULT_MODEL_CONFIG: ModelConfigState = {
  provider: "qwen",
  model: MODEL_DEFAULTS.qwen.model,
  baseUrl: MODEL_DEFAULTS.qwen.baseUrl,
  apiKeyRef: "",
  enabled: true,
  apiKeyPresent: false,
};

const DEFAULT_MODEL_PROVIDER_STATUS: TextModelProviderStatus = {
  provider: "qwen",
  model: MODEL_DEFAULTS.qwen.model,
  base_url: MODEL_DEFAULTS.qwen.baseUrl,
  baseUrl: MODEL_DEFAULTS.qwen.baseUrl,
  enabled: false,
  base_url_present: true,
  api_key_present: false,
  live_ready: false,
  status: "api_config_unsaved",
  message: "API 配置未保存，请在正式应用窗口内保存 session-only API 配置。",
  storage: "session-only",
};

const SCENE_OPTIONS: SceneOption[] = [
  { group: "基础动漫叙事", value: "hot_blood_battle", label: "热血战斗" },
  { group: "基础动漫叙事", value: "ensemble_performance", label: "群像表演" },
  { group: "基础动漫叙事", value: "emotional_dialogue", label: "情绪对话" },
  { group: "基础动漫叙事", value: "encounter_performance", label: "相遇表演" },
  { group: "基础动漫叙事", value: "field_chase", label: "场域追逐" },
  { group: "基础动漫叙事", value: "spectacle_showcase", label: "奇观展示" },
  { group: "基础动漫叙事", value: "daily_healing", label: "日常治愈" },
  { group: "国漫 / 武侠 / 奇幻", value: "guoman_hot_blood_combat", label: "国漫热血打斗" },
  { group: "国漫 / 武侠 / 奇幻", value: "guoman_ensemble_performance", label: "国漫群像表演" },
  { group: "国漫 / 武侠 / 奇幻", value: "ink_wuxia_combat", label: "水墨武打" },
  { group: "国漫 / 武侠 / 奇幻", value: "eastern_spectacle", label: "东方奇观" },
  { group: "国漫 / 武侠 / 奇幻", value: "xianxia_action", label: "仙侠动作" },
  { group: "国漫 / 武侠 / 奇幻", value: "urban_fantasy", label: "都市奇幻" },
  { group: "三国 / 国战 / SLG", value: "chinese_war_formation", label: "国战军阵建立" },
  { group: "三国 / 国战 / SLG", value: "weapon_highlight", label: "武将兵器高光" },
  { group: "三国 / 国战 / SLG", value: "council_strategy", label: "朝堂军帐权谋" },
  { group: "三国 / 国战 / SLG", value: "siege_defense", label: "多军团攻城" },
  { group: "三国 / 国战 / SLG", value: "slg_sandbox_view", label: "沙盘战略视口" },
  { group: "三国 / 国战 / SLG", value: "slg_march_encirclement", label: "行军轨迹合围" },
  { group: "三国 / 国战 / SLG", value: "slg_city_growth", label: "城建演进反馈" },
  { group: "三国 / 国战 / SLG", value: "slg_battle_report", label: "战报 UI" },
];

const API_DOC_SECTIONS = [
  {
    title: "当前已接 Bridge",
    items: [
      "扩写故事 / 改写剧本：使用当前场景类型、目标时长和故事材料，只更新剧本区正文。",
      "开始生成：只使用已确认的镜头任务文本和当前任务时长，返回可编辑分镜表格。",
      "export_bundle：导出分镜词或完整剧本时先选择 Excel 保存路径，完成后只显示用户选择的位置。",
    ],
  },
  {
    title: "模型配置边界",
    items: [
      "千问 Qwen 是当前唯一可启用的文本模型；豆包 Doubao 和自定义模型为预留入口。",
      "千问未配置、未启用或调用失败时会自动使用本地候选结果。",
      "API Key 只在桌面会话内保存，不会显示明文。",
    ],
  },
  {
    title: "导出与状态",
    items: [
      "WarningOnly 表示已生成但存在候选限制或证据限制，不等同失败。",
      "Excel workbook 未 ready 时不会伪造 Excel ready、下载路径或素材路径。",
      "候选提示词证据不会被声明为最终分镜提示词。",
    ],
  },
];

function useWorkbenchRoute(): ViewId {
  const [activeView, setActiveView] = useState<ViewId>(() => resolveRoute(window.location.hash));

  useEffect(() => {
    const syncHash = () => {
      setActiveView(resolveRoute(window.location.hash));
    };

    window.addEventListener("hashchange", syncHash);
    if (!window.location.hash) {
      window.location.hash = ROUTES[0].hash;
    }

    return () => {
      window.removeEventListener("hashchange", syncHash);
    };
  }, []);

  return activeView;
}

export function App() {
  useWorkbenchRoute();

  const [activePanel, setActivePanel] = useState<HeaderPanel>("none");
  const [selectedModel, setSelectedModel] = useState<WorkbenchModelId>(DEFAULT_MODEL_CONFIG.provider);
  const [modelConfig, setModelConfig] = useState<ModelConfigState>(DEFAULT_MODEL_CONFIG);
  const [modelConfigDraft, setModelConfigDraft] = useState<ModelConfigDraft>({
    ...DEFAULT_MODEL_CONFIG,
    apiKeyInput: "",
  });
  const [modelProviderStatus, setModelProviderStatus] = useState<TextModelProviderStatus>(
    DEFAULT_MODEL_PROVIDER_STATUS,
  );
  const [bridgeStatus] = useState(() => getHopeBridgeStatus());
  const [selectedScene, setSelectedScene] = useState<SceneFusionOption>(DEFAULT_SCENE);
  const [synopsis, setSynopsis] = useState(DEFAULT_SYNOPSIS);
  const [expandedScript, setExpandedScript] = useState("");
  const [expandedScriptResult, setExpandedScriptResult] = useState<ExpandScriptResponse | null>(null);
  const [showExpandedScriptStatus, setShowExpandedScriptStatus] = useState(false);
  const [expandedScriptDurationSeconds, setExpandedScriptDurationSeconds] = useState<number | null>(null);
  const [expandedScriptScene, setExpandedScriptScene] = useState<SceneOption | null>(null);
  const [acceptedScript, setAcceptedScript] = useState("");
  const [acceptedScriptId, setAcceptedScriptId] = useState<string | null>(null);
  const [acceptedScriptDurationSeconds, setAcceptedScriptDurationSeconds] = useState<number | null>(null);
  const [acceptedScriptTargetDurationMode, setAcceptedScriptTargetDurationMode] =
    useState<TargetDurationMode | null>(null);
  const [acceptedScriptScene, setAcceptedScriptScene] = useState<SceneOption | null>(null);
  const [taskName, setTaskName] = useState(DEFAULT_TASK_NAME);
  const [taskSourceScript, setTaskSourceScript] = useState("");
  const [taskScriptId, setTaskScriptId] = useState<string | null>(null);
  const [taskSegmentTitle, setTaskSegmentTitle] = useState("");
  const [durationSeconds, setDurationSeconds] = useState(15);
  const [targetDurationMode, setTargetDurationMode] = useState<TargetDurationMode>(FIXED_DURATION_MODE);
  const [durationModeUserSelected, setDurationModeUserSelected] = useState(false);
  const [rows, setRows] = useState<StoryboardWorkbenchRow[]>([]);
  const [rowsDirty, setRowsDirty] = useState(false);
  const [storyboardResult, setStoryboardResult] = useState<GenerateStoryboardResponse | null>(null);
  const [lastExportResult, setLastExportResult] = useState<ExportBundleResponse | null>(null);
  const [generatedSceneTasks, setGeneratedSceneTasks] = useState<GeneratedSceneTask[]>([]);
  const [sceneTasks, setSceneTasks] = useState<SceneTaskRecord[]>([]);
  const [currentTaskId, setCurrentTaskId] = useState<string | null>(null);
  const [bridgeBusy, setBridgeBusy] = useState<
    | "import_document"
    | "expand"
    | "generate"
    | "save_rows"
    | "save_shot"
    | "list_shots"
    | "remove_shot"
    | "export_words"
    | "export_script"
    | "export_bank"
    | null
  >(null);
  const [finalizedShots, setFinalizedShots] = useState<FinalizedStoryboardShotResult[]>([]);
  const [finalizedBankOpen, setFinalizedBankOpen] = useState(false);
  const [lastStoryboardBankExport, setLastStoryboardBankExport] =
    useState<ExportStoryboardBankResponse | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [jumpPage, setJumpPage] = useState("1");
  const [storyboardPageSize, setStoryboardPageSize] = useState(() => resolveStoryboardPageSize());
  const [storyboardPageSizeManual, setStoryboardPageSizeManual] = useState(false);
  const [editingRowId, setEditingRowId] = useState<string | null>(null);
  const [editingDraft, setEditingDraft] = useState<StoryboardWorkbenchRow | null>(null);
  const [textDialog, setTextDialog] = useState<TextDialogState | null>(null);
  const [taskDraft, setTaskDraft] = useState<TaskDraftState | null>(null);
  const [isTaskPickerOpen, setIsTaskPickerOpen] = useState(false);
  const [taskSerial, setTaskSerial] = useState(0);
  const [exportMessage, setExportMessage] = useState("等待导出");

  const editingRow = editingDraft;

  const pageCount = Math.max(1, Math.ceil(rows.length / storyboardPageSize));
  const pageStartRow = rows.length ? (currentPage - 1) * storyboardPageSize + 1 : 0;
  const pageEndRow = rows.length ? Math.min(rows.length, currentPage * storyboardPageSize) : 0;
  const pagedRows = useMemo(() => {
    const start = (currentPage - 1) * storyboardPageSize;
    return rows.slice(start, start + storyboardPageSize);
  }, [currentPage, rows, storyboardPageSize]);

  useEffect(() => {
    setCurrentPage((value) => Math.min(value, pageCount));
  }, [pageCount]);

  useEffect(() => {
    setJumpPage(String(currentPage));
  }, [currentPage]);

  useEffect(() => {
    if (storyboardPageSizeManual) {
      return;
    }

    const handleResize = () => setStoryboardPageSize(resolveStoryboardPageSize());
    handleResize();
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, [storyboardPageSizeManual]);

  const currentSceneTask = useMemo(
    () => sceneTasks.find((task) => task.id === currentTaskId) ?? null,
    [currentTaskId, sceneTasks],
  );
  const currentSceneTaskIndex = useMemo(
    () =>
      currentSceneTask
        ? Math.max(1, sceneTasks.findIndex((task) => task.id === currentSceneTask.id) + 1)
        : 0,
    [currentSceneTask, sceneTasks],
  );
  const selectedSceneOption = useMemo(() => resolveSceneOption(selectedScene), [selectedScene]);
  const sourceInputAnalysis = useMemo(() => analyzeSourceInputForUi(synopsis), [synopsis]);
  const isLongTextDurationMode = targetDurationMode === LONG_TEXT_DURATION_MODE;
  const durationSelectValue = isLongTextDurationMode ? LONG_TEXT_DURATION_MODE : String(durationSeconds);
  const activeScriptDurationSeconds = isLongTextDurationMode
    ? estimateLongTextAutoDurationSeconds(sourceInputAnalysis)
    : durationSeconds;
  const acceptedSourceInputAnalysis = useMemo(
    () => analyzeSourceInputForUi(acceptedScript),
    [acceptedScript],
  );
  const storyPlanIsCurrent = isStoryPlanCurrentForUi({
    draftText: synopsis,
    acceptedText: acceptedScript,
    selectedSceneOption,
    acceptedScene: acceptedScriptScene,
    targetDurationMode,
    acceptedTargetDurationMode: acceptedScriptTargetDurationMode,
    durationSeconds,
    acceptedDurationSeconds: acceptedScriptDurationSeconds,
  });
  const expandedScriptSnapshotIsCurrent = isExpandedScriptSnapshotCurrentForUi({
    response: expandedScriptResult,
    responseScene: expandedScriptScene,
    responseDurationSeconds: expandedScriptDurationSeconds,
    selectedSceneOption,
    targetDurationMode,
    durationSeconds,
  });
  const storyBridgeView = useMemo(
    () =>
      buildStoryBridgeView({
        draftAnalysis: sourceInputAnalysis,
        acceptedAnalysis: acceptedSourceInputAnalysis,
        targetDurationMode,
        durationSeconds,
        response: storyPlanIsCurrent && expandedScriptSnapshotIsCurrent ? expandedScriptResult : null,
        sceneTaskCount: storyPlanIsCurrent ? sceneTasks.length : 0,
        hasAcceptedScript: Boolean(acceptedScript.trim()),
        isCurrent: storyPlanIsCurrent,
      }),
    [
      acceptedScript,
      acceptedSourceInputAnalysis,
      durationSeconds,
      expandedScriptResult,
      expandedScriptSnapshotIsCurrent,
      sceneTasks.length,
      sourceInputAnalysis,
      storyPlanIsCurrent,
      targetDurationMode,
    ],
  );
  const sourceInputStatusText = storyBridgeView.previewStatus;

  const canImportScript = acceptedScript.trim().length > 0 && storyPlanIsCurrent;
  const hasTaskScript = currentSceneTask?.scriptText.trim().length ? true : taskSourceScript.trim().length > 0;
  const canGenerate =
    Boolean(currentSceneTask?.scriptText.trim()) &&
    Boolean(currentSceneTask?.name.trim()) &&
    storyPlanIsCurrent;
  const expandedScriptAccepted = Boolean(
    expandedScriptResult?.script_id &&
      expandedScriptSnapshotIsCurrent &&
      acceptedScriptId === expandedScriptResult.script_id,
  );
  const textDialogScriptBlocks = useMemo(
    () =>
      textDialog && (textDialog.kind === "synopsis" || textDialog.kind === "expandedScript")
        ? parseEditableStoryDialogBlocks(textDialog.value)
        : [],
    [textDialog],
  );
  const acceptedPlanMode = acceptedScriptTargetDurationMode ?? targetDurationMode;
  const acceptedPlanFallbackDuration =
    acceptedPlanMode === LONG_TEXT_DURATION_MODE ? activeScriptDurationSeconds : durationSeconds;
  const shotCandidateBaseDuration = normalizeScriptDurationOption(
    acceptedScriptDurationSeconds ?? acceptedPlanFallbackDuration,
    acceptedPlanFallbackDuration,
  );
  const currentTaskDuration = storyPlanIsCurrent
    ? normalizeDurationOption(
        currentSceneTask?.sourceDurationSeconds ?? shotCandidateBaseDuration,
        shotCandidateBaseDuration,
      )
    : null;
  const currentTaskSceneLabel = currentSceneTask?.sourceSceneLabel ?? acceptedScriptScene?.label ?? selectedSceneOption.label;
  const shotCandidates = useMemo(
    () =>
      storyPlanIsCurrent
        ? buildShotCandidates(acceptedScript, shotCandidateBaseDuration, acceptedPlanMode)
        : [],
    [acceptedPlanMode, acceptedScript, shotCandidateBaseDuration, storyPlanIsCurrent],
  );
  const confirmedSceneTaskIds = useMemo(
    () =>
      new Set(
        finalizedShots
          .filter((shot) => shot.confirmed && shot.shot_task_id)
          .map((shot) => shot.shot_task_id),
      ),
    [finalizedShots],
  );
  const generatedSceneTaskCount = useMemo(
    () => sceneTasks.filter((task) => task.status === "generated").length,
    [sceneTasks],
  );
  const finalizedStoryboardTotalDuration = useMemo(
    () =>
      finalizedShots
        .filter((shot) => shot.confirmed)
        .reduce((total, shot) => total + normalizeDurationOption(shot.shot_duration_seconds, 15), 0),
    [finalizedShots],
  );
  const confirmedShotCount = useMemo(
    () => finalizedShots.filter((shot) => shot.confirmed).length,
    [finalizedShots],
  );
  const currentFinalizedShot = useMemo(
    () =>
      storyboardResult?.result_id
        ? finalizedShots.find((shot) => shot.result_id === storyboardResult.result_id && shot.confirmed) ?? null
        : null,
    [finalizedShots, storyboardResult?.result_id],
  );
  const currentShotConfirmed = Boolean(currentFinalizedShot);
  const currentShotStatusLabel = currentShotConfirmed
    ? "已确认 / 已定稿"
    : rows.length
    ? "待确定使用"
    : hasTaskScript
    ? "待生成"
    : "未导入";
  const currentShotStatusTone = currentShotConfirmed ? "confirmed" : "pending";
  const currentSceneTaskStatus = currentSceneTask
    ? getSceneTaskStatus(currentSceneTask, confirmedSceneTaskIds)
    : null;
  const pageTokens = useMemo(() => buildPageTokens(pageCount, currentPage), [pageCount, currentPage]);
  const sceneOptionGroups = useMemo(() => groupSceneOptions(SCENE_OPTIONS), []);
  const selectedModelLabel = useMemo(() => resolveModelLabel(modelConfig.provider), [modelConfig.provider]);
  const desktopRuntimeAvailable = bridgeStatus.mode === "desktop";
  const desktopRuntimeTone = desktopRuntimeAvailable ? "enabled" : "reserved";
  const desktopRuntimeLabel = desktopRuntimeAvailable ? "真实 IPC 已连接" : "非正式运行环境不可验收";
  const modelReservedWarning = modelConfig.provider === "qwen"
    ? ""
    : "该模型接口已配置为预留状态，当前仍使用本地文本生成桥接。";
  const modelConfigSummary = useMemo(
    () => buildModelConfigSummary(modelConfig, modelProviderStatus),
    [modelConfig, modelProviderStatus],
  );
  const currentTaskDraftQueueTask = useMemo(
    () =>
      taskDraft?.editingTaskRecordId
        ? sceneTasks.find((task) => task.id === taskDraft.editingTaskRecordId) ?? null
        : null,
    [sceneTasks, taskDraft?.editingTaskRecordId],
  );
  const currentTaskDraftQueueStatus = useMemo(
    () => (currentTaskDraftQueueTask ? getSceneTaskStatus(currentTaskDraftQueueTask, confirmedSceneTaskIds) : null),
    [confirmedSceneTaskIds, currentTaskDraftQueueTask],
  );
  const currentTaskDraftQueueTone = currentTaskDraftQueueStatus?.tone ?? "pending";
  const currentTaskDraftQueueStatusLabel = currentTaskDraftQueueStatus?.label ?? "候选草稿";
  const resolveNextShotIndexForCandidate = (candidateId: string, excludedTaskId: string | null = null) =>
    sceneTasks.filter((task) => task.candidateId === candidateId && task.id !== excludedTaskId).length + 1;
  const modelRuntimeLabel = useMemo(
    () => formatModelProviderStatus(modelConfig, modelProviderStatus),
    [modelConfig, modelProviderStatus],
  );
  const modelRuntimeTone = useMemo(
    () => formatModelProviderStatusTone(modelProviderStatus),
    [modelProviderStatus],
  );

  const applyProviderStatusToUi = (status: TextModelProviderStatus) => {
    const provider = normalizeWorkbenchModelId(status.provider);
    const defaultModel = MODEL_DEFAULTS[provider].model;
    const statusBaseUrl = (status.base_url ?? status.baseUrl ?? "").trim();
    const nextConfig: ModelConfigState = {
      provider,
      model: status.model.trim() || defaultModel,
      baseUrl: statusBaseUrl || (status.base_url_present ? MODEL_DEFAULTS[provider].baseUrl : ""),
      apiKeyRef: provider === modelConfig.provider ? modelConfig.apiKeyRef : "",
      enabled: provider === "qwen" && status.enabled,
      apiKeyPresent: status.api_key_present,
    };
    setSelectedModel(provider);
    setModelConfig(nextConfig);
    setModelConfigDraft((current) => ({
      ...nextConfig,
      apiKeyRef: provider === current.provider ? current.apiKeyRef : nextConfig.apiKeyRef,
      apiKeyInput: "",
    }));
    setModelProviderStatus({ ...status, provider });
  };

  const requireDesktopRuntime = (operation: string) => {
    if (desktopRuntimeAvailable) {
      return true;
    }
    setModelProviderStatus({
      ...DEFAULT_MODEL_PROVIDER_STATUS,
      status: "not_desktop",
      live_ready: false,
      message: "未在正式应用窗口内运行，当前环境不可验收。",
    });
    setExportMessage(`${operation} 需要正式应用窗口的真实 IPC；当前环境不可验收。`);
    return false;
  };

  useEffect(() => {
    let cancelled = false;
    if (!desktopRuntimeAvailable) {
      setModelProviderStatus({
        ...DEFAULT_MODEL_PROVIDER_STATUS,
        status: "not_desktop",
        live_ready: false,
        message: "未在正式应用窗口内运行，当前环境不可验收。",
      });
      return () => {
        cancelled = true;
      };
    }

    getTextModelProviderStatus()
      .then((status) => {
        if (!cancelled) {
          applyProviderStatusToUi(status);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setModelProviderStatus({
            ...DEFAULT_MODEL_PROVIDER_STATUS,
            status: "api_config_unsaved",
            live_ready: false,
            message: `API 配置未保存或状态读取失败：${formatProductError(error)}`,
          });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [desktopRuntimeAvailable]);

  const refreshFinalizedShots = async (openDialog = false, scriptIdOverride?: string | null) => {
    setBridgeBusy("list_shots");
    try {
      const response = await invokeListStoryboardShotResults({
        project_id: DEFAULT_PROJECT_ID,
        script_id: scriptIdOverride ?? acceptedScriptId ?? taskScriptId ?? null,
        confirmed: null,
      });
      setFinalizedShots(response.shots);
      if (openDialog) {
        setFinalizedBankOpen(true);
      }
      if (response.warnings.length) {
        setExportMessage(formatWarnings(response.warnings));
      }
      return response.shots;
    } catch (error) {
      setExportMessage(`读取已定稿分镜集合失败：${formatProductError(error)}`);
      return null;
    } finally {
      setBridgeBusy(null);
    }
  };

  const handleOpenFinalizedStoryboardDialog = () => {
    void refreshFinalizedShots(true);
  };

  const confirmDiscardDirty = (actionLabel: string) => {
    if (!rowsDirty) {
      return true;
    }

    return window.confirm(`当前分镜行有未保存修改。${actionLabel} 会丢弃这些本地修改，是否继续？`);
  };

  const updateCurrentTaskRecord = (patch: Partial<SceneTaskRecord>) => {
    if (!currentTaskId) {
      return;
    }

    setSceneTasks((current) =>
      current.map((task) => (task.id === currentTaskId ? { ...task, ...patch } : task)),
    );
  };

  const markRowsAsDirty = (nextRows: StoryboardWorkbenchRow[], message?: string) => {
    const normalizedRows = renumberRows(nextRows);
    setRows(normalizedRows);
    setRowsDirty(true);
    setLastExportResult(null);
    updateCurrentTaskRecord({
      rowsSnapshot: cloneWorkbenchRows(normalizedRows),
      dirty: true,
      hadRowEdits: true,
    });
    if (message) {
      setExportMessage(message);
    }
  };

  const clearExpandedScriptSnapshot = () => {
    setExpandedScriptResult(null);
    setExpandedScriptDurationSeconds(null);
    setExpandedScriptScene(null);
    setShowExpandedScriptStatus(false);
  };

  const invalidateConfirmedPlan = (message?: string) => {
    clearExpandedScriptSnapshot();
    clearConfirmedStoryboardState();
    setTaskDraft(null);
    if (message) {
      setExportMessage(message);
    }
  };

  const handleSceneSelectionChange = (value: SceneFusionOption) => {
    if (value === selectedScene) {
      return;
    }
    setSelectedScene(value);
    invalidateConfirmedPlan("场景类型已变更，请点击“改写剧本”让当前正文重新匹配新场景。");
  };

  const handleDurationSelectionChange = (value: string) => {
    setDurationModeUserSelected(true);
    if (value === LONG_TEXT_DURATION_MODE) {
      if (targetDurationMode !== LONG_TEXT_DURATION_MODE) {
        setTargetDurationMode(LONG_TEXT_DURATION_MODE);
        invalidateConfirmedPlan("目标配置已切换为长文本模式，当前材料待确定使用。");
      }
      return;
    }

    const nextDuration = Number(value);
    if (targetDurationMode !== FIXED_DURATION_MODE || durationSeconds !== nextDuration) {
      setTargetDurationMode(FIXED_DURATION_MODE);
      setDurationSeconds(nextDuration);
      invalidateConfirmedPlan(`单镜头时长已改为 ${nextDuration} 秒，当前材料待确定使用。`);
    }
  };

  const restoreSceneTaskState = (task: SceneTaskRecord) => {
    setCurrentTaskId(task.id);
    setTaskName(task.name);
    setTaskSourceScript(task.scriptText);
    setTaskScriptId(task.scriptId ?? acceptedScriptId ?? null);
    setTaskSegmentTitle(task.segmentTitle);
    setStoryboardResult(task.storyboardResult ?? null);
    setLastExportResult(null);
    setRows(cloneWorkbenchRows(task.rowsSnapshot ?? []));
    setRowsDirty(Boolean(task.dirty));
    setCurrentPage(1);
    closeStoryboardEditDialog();
  };

  const persistCurrentRows = async (
    dirtySourceNote: string,
    rowsOverride?: StoryboardWorkbenchRow[],
  ): Promise<GenerateStoryboardResponse | null> => {
    const sourceResult = storyboardResult ?? currentSceneTask?.storyboardResult ?? null;
    const targetRows = renumberRows(rowsOverride ?? rows);
    if (!sourceResult?.result_id || !targetRows.length) {
      setExportMessage("当前没有可保存的分镜结果，请先生成分镜后再保存修改。");
      return null;
    }

    const response = await invokeUpdateStoryboardRows({
      result_id: sourceResult.result_id,
      task_id: sourceResult.task_id ?? currentSceneTask?.taskId ?? currentTaskId,
      rows: toGeneratedStoryboardRows(targetRows),
      operation_id: `desktop-save-${Date.now()}`,
      base_revision: sourceResult.revision ?? currentSceneTask?.baseRevision ?? 0,
      dirty_source_note: dirtySourceNote,
    });

    if (isBlockedStoryboardResponse(response)) {
      setRows(targetRows);
      setRowsDirty(true);
      updateCurrentTaskRecord({
        rowsSnapshot: cloneWorkbenchRows(targetRows),
        dirty: true,
        hadRowEdits: true,
      });
      setExportMessage(formatBlockedStoryboardResponse(response));
      return null;
    }

    const savedRows = response.rows.map((row) => mapGeneratedStoryboardRow(row));
    const rowWarnings = collectPromptWarningCodes(response.rows);
    setStoryboardResult(response);
    setRows(savedRows);
    setRowsDirty(false);
    setLastExportResult(null);
    updateCurrentTaskRecord({
      taskId: response.task_id ?? currentSceneTask?.taskId ?? null,
      resultId: response.result_id,
      rowCount: savedRows.length,
      rowsSnapshot: cloneWorkbenchRows(savedRows),
      storyboardResult: response,
      promptTextStatus: collectPromptStatuses(response.rows),
      promptTextWarnings: rowWarnings,
      selectedTotalDurationSeconds: response.selected_total_duration_seconds,
      rowsHash: response.rows_hash,
      updatedAtMs: response.updated_at_ms,
      baseRevision: response.revision,
      dirty: false,
      hadRowEdits: true,
    });
    return response;
  };

  const ensureRowsSavedForExport = async (dirtySourceNote: string) => {
    if (!rowsDirty) {
      return storyboardResult;
    }

    setExportMessage("正在保存本地分镜修改，保存成功后继续导出。");
    return persistCurrentRows(dirtySourceNote);
  };

  const syncModelProviderStatusFromWarnings = (warnings: ProductWarning[]) => {
    if (!hasTextModelFallback(warnings)) {
      return;
    }
    setModelProviderStatus((current) => ({
      ...current,
      status: "fallback",
      live_ready: false,
      message: "千问：调用失败已回退，本次已使用本地候选结果。",
    }));
  };

  const handleModelSelect = (provider: WorkbenchModelId) => {
    const defaults = MODEL_DEFAULTS[provider];
    const nextConfig: ModelConfigState = {
      ...modelConfig,
      provider,
      model: provider === modelConfig.provider ? modelConfig.model : defaults.model,
      baseUrl: provider === modelConfig.provider ? modelConfig.baseUrl : defaults.baseUrl,
      apiKeyPresent: provider === modelConfig.provider ? modelConfig.apiKeyPresent : false,
    };
    setSelectedModel(provider);
    setModelConfig(nextConfig);
    setModelConfigDraft({ ...nextConfig, apiKeyInput: "" });
    setModelProviderStatus({
      ...DEFAULT_MODEL_PROVIDER_STATUS,
      provider,
      model: nextConfig.model,
      enabled: false,
      live_ready: false,
      api_key_present: provider === modelConfig.provider && modelProviderStatus.api_key_present,
      status: provider === "qwen" ? "api_config_unsaved" : "reserved",
      message:
        provider === "qwen"
          ? "API 配置未保存，请在正式应用窗口内保存 session-only API 配置。"
          : "该模型接口为预留状态，当前未启用真实调用。",
    });
    setExportMessage(
      provider === "qwen"
        ? "已选择千问 Qwen，请在 API 接口保存配置；未配置时会自动使用本地候选结果。"
        : "该模型接口为预留状态，当前未启用真实调用。",
    );
  };

  const handleModelConfigDraftChange = <K extends keyof ModelConfigDraft>(
    field: K,
    value: ModelConfigDraft[K],
  ) => {
    setModelConfigDraft((current) => {
      const next = { ...current, [field]: value };
      if (field === "provider" && typeof value === "string") {
        const provider = value as WorkbenchModelId;
        const defaults = MODEL_DEFAULTS[provider];
        return {
          ...next,
          provider,
          model: defaults.model,
          baseUrl: defaults.baseUrl,
          apiKeyPresent: false,
          apiKeyInput: "",
        };
      }

      return next;
    });
  };

  const handleSaveModelConfig = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!requireDesktopRuntime("API 配置保存")) {
      return;
    }
    const provider = modelConfigDraft.provider;
    const apiKeyRef = normalizeApiKeyRef(modelConfigDraft.apiKeyRef);
    const hadRawValueInRef = modelConfigDraft.apiKeyRef.trim().length > 0 && !apiKeyRef;
    const requestedModel = modelConfigDraft.model.trim() || MODEL_DEFAULTS[provider].model;
    const requestedBaseUrl = modelConfigDraft.baseUrl.trim();
    const requestedEnabled = provider === "qwen" && modelConfigDraft.enabled;

    try {
      const status = await configureTextModelProvider({
        provider,
        model: requestedModel,
        base_url: requestedBaseUrl,
        api_key:
          provider === "qwen" && !hadRawValueInRef
            ? modelConfigDraft.apiKeyInput.trim() || null
            : null,
        api_key_ref: apiKeyRef || null,
        enabled: requestedEnabled,
      });
      const statusBaseUrl = (status.base_url ?? status.baseUrl ?? "").trim();
      const nextConfig: ModelConfigState = {
        provider,
        model: status.model || requestedModel,
        baseUrl: statusBaseUrl || requestedBaseUrl,
        apiKeyRef,
        enabled: status.enabled,
        apiKeyPresent: status.api_key_present,
      };
      setSelectedModel(provider);
      setModelConfig(nextConfig);
      setModelConfigDraft({ ...nextConfig, apiKeyInput: "" });
      setModelProviderStatus(status);
      setActivePanel("none");
      setExportMessage(
        hadRawValueInRef
          ? "检测到密钥引用栏疑似填入了明文 Key，已清空该栏；请把真实 Key 填在 API Key 输入框。本次未保存明文。"
          : status.status === "enabled"
          ? "已启用千问文本生成；扩写和生成会优先尝试 live Qwen，失败时自动回退本地候选结果。"
          : `${status.message} 配置仅在本次会话有效。`,
      );
    } catch (error) {
      setExportMessage(`API 配置保存失败：${formatProductError(error)}`);
    }
  };

  const openSynopsisDialog = () => {
    setTextDialog({
      kind: "synopsis",
      title: "编辑故事材料",
      value: stripStoryDialogInternalText(synopsis),
      helper: "只编辑正文；保存后需要点击“确定使用”才会同步到镜头拆解。",
      editable: true,
    });
  };

  const openExpandedScriptDialog = () => {
    setTextDialog({
      kind: "expandedScript",
      title: "查看剧本正文",
      value: expandedScript ? formatStoryMaterialDialogText(expandedScript) : "等待剧本正文",
      helper: "确认使用后才会进入镜头拆解。",
      editable: false,
    });
  };

  const openStoryboardCellDialog = (
    row: StoryboardWorkbenchRow,
    field: LongTextField,
    label: string,
  ) => {
    setTextDialog({
      kind: "storyboardCell",
      title: `第 ${row.order} 条 · ${label}`,
      value: formatStoryboardDialogText(String(row[field] ?? "")),
      helper: "表格中只显示短预览；这里查看完整文本，修改请使用操作列“修改”。",
      editable: false,
    });
  };

  const openStoryboardEditDialog = (row: StoryboardWorkbenchRow) => {
    setEditingRowId(row.id);
    setEditingDraft({ ...row, prompt: formatStoryboardDialogText(row.prompt) });
  };

  const closeStoryboardEditDialog = () => {
    setEditingRowId(null);
    setEditingDraft(null);
  };

  const handleSaveTextDialog = async () => {
    if (!textDialog || !textDialog.editable) {
      setTextDialog(null);
      return;
    }

    if (textDialog.kind === "synopsis") {
      const nextText = extractEditableStoryDialogText(textDialog.value);
      setSynopsis(nextText);
      setExpandedScript(nextText);
      clearExpandedScriptSnapshot();
      clearConfirmedStoryboardState();
      setExportMessage("文本已保存，待确定使用。");
    }

    if (textDialog.kind === "storyboardCell" && textDialog.rowId && textDialog.field) {
      const { rowId, field, value } = textDialog;
      const nextRows = rows.map((row) => (row.id === rowId ? { ...row, [field]: value } : row));
      setTextDialog(null);
      setBridgeBusy("save_rows");
      try {
        const saved = await persistCurrentRows("desktop_storyboard_cell_edit", nextRows);
        if (saved) {
          setExportMessage(`已保存表格文本修改：${saved.result_id} / revision ${saved.revision ?? 0}`);
        } else {
          markRowsAsDirty(nextRows, "表格文本已更新到本地，保存未完成；导出前需要重新保存。");
        }
      } catch (error) {
        markRowsAsDirty(nextRows, `保存表格文本失败，本地修改已保留：${formatProductError(error)}`);
      } finally {
        setBridgeBusy(null);
      }
      return;
    }

    setTextDialog(null);
  };

  const handleConfirmTextDialogUse = () => {
    if (!textDialog) {
      return;
    }

    if (textDialog.kind === "synopsis") {
      const nextText = extractEditableStoryDialogText(textDialog.value);
      if (confirmScriptTextForStoryboard(nextText)) {
        setTextDialog(null);
      }
      return;
    }

    if (
      textDialog.kind === "expandedScript" &&
      confirmScriptTextForStoryboard(expandedScript.trim() || extractEditableStoryDialogText(textDialog.value))
    ) {
      setTextDialog(null);
    }
  };

  const handleClearTextDialog = () => {
    setTextDialog((current) => (current && current.editable ? { ...current, value: "" } : current));
  };

  const confirmScriptTextForStoryboard = (sourceText: string) => {
    const nextText = sourceText.trim();
    if (!nextText) {
      setExportMessage("请先填写文案，再确定使用。");
      return false;
    }
    if (!confirmDiscardDirty("确定使用当前文案")) {
      return false;
    }

    const analysis = analyzeSourceInputForUi(nextText);
    const scriptDuration = targetDurationMode === LONG_TEXT_DURATION_MODE
      ? estimateLongTextAutoDurationSeconds(analysis)
      : durationSeconds;
    const scriptScene = selectedSceneOption;
    const shouldCarryExpandedScriptId = Boolean(
      expandedScriptResult?.script_id &&
        expandedScriptSnapshotIsCurrent &&
        normalizeScriptText(expandedScript || synopsis) === normalizeScriptText(nextText),
    );
    const scriptId = shouldCarryExpandedScriptId ? expandedScriptResult?.script_id ?? null : null;
    const nextSceneTasks: SceneTaskRecord[] = [];

    setSynopsis(nextText);
    setExpandedScript(nextText);
    setAcceptedScript(nextText);
    setAcceptedScriptId(scriptId);
    setAcceptedScriptDurationSeconds(scriptDuration);
    setAcceptedScriptTargetDurationMode(targetDurationMode);
    setAcceptedScriptScene(scriptScene);
    setTaskSourceScript("");
    setTaskScriptId(null);
    setTaskSegmentTitle("");
    setTaskName(DEFAULT_TASK_NAME);
    setStoryboardResult(null);
    setLastExportResult(null);
    setGeneratedSceneTasks([]);
    setSceneTasks(nextSceneTasks);
    setCurrentTaskId(null);
    setIsTaskPickerOpen(false);
    setTaskSerial(0);
    setRows([]);
    setRowsDirty(false);
    setCurrentPage(1);
    setExportMessage(
      nextSceneTasks.length
        ? `已确认使用当前文案，已准备 ${nextSceneTasks.length} 个镜头任务。下一步可直接开始生成。`
        : "已确认使用当前文案，系统候选将在新建镜头任务时动态显示；请先保存任务队列后再开始生成。",
    );
    return true;
  };

  const handleConfirmCurrentScriptUse = () => {
    const nextText = synopsis.trim();
    if (!nextText) {
      setExportMessage("请先填写文案，再确定使用。");
      return;
    }

    confirmScriptTextForStoryboard(nextText);
  };

  const clearConfirmedStoryboardState = () => {
    setAcceptedScript("");
    setAcceptedScriptId(null);
    setAcceptedScriptDurationSeconds(null);
    setAcceptedScriptTargetDurationMode(null);
    setAcceptedScriptScene(null);
    setTaskSourceScript("");
    setTaskScriptId(null);
    setTaskSegmentTitle("");
    setStoryboardResult(null);
    setLastExportResult(null);
    setGeneratedSceneTasks([]);
    setSceneTasks([]);
    setCurrentTaskId(null);
    setIsTaskPickerOpen(false);
    setTaskDraft(null);
    setTaskSerial(0);
    setRows([]);
    setRowsDirty(false);
    setCurrentPage(1);
    closeStoryboardEditDialog();
  };

  const resetScriptAndStoryboardState = () => {
    setExpandedScript("");
    clearExpandedScriptSnapshot();
    clearConfirmedStoryboardState();
  };

  const handleImportStoryDocument = async () => {
    if (bridgeBusy) {
      return;
    }
    if (!confirmDiscardDirty("导入故事文档")) {
      return;
    }

    setBridgeBusy("import_document");
    try {
      const document = await invokeImportStoryDocument();
      if (!document) {
        setExportMessage("已取消导入");
        return;
      }
      const text = normalizeScriptText(document.text);
      if (!text) {
        setExportMessage("导入失败：文档正文为空。");
        return;
      }
      const analysis = analyzeSourceInputForUi(text);
      setSynopsis(text);
      resetScriptAndStoryboardState();
      setExportMessage(
        `已导入 ${formatImportedDocumentType(document.file_type)} 文档。${analysis.statusMessage} 当前仍按用户选择的${
          targetDurationMode === LONG_TEXT_DURATION_MODE ? "长文本模式" : `${durationSeconds} 秒`
        }处理。`,
      );
    } catch (error) {
      setExportMessage(`导入失败：${formatProductError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
  };

  const handleExpandStory = async () => {
    if (bridgeBusy) {
      return;
    }
    if (!requireDesktopRuntime("扩写故事")) {
      return;
    }
    if (!confirmDiscardDirty("扩写故事")) {
      return;
    }

    const storyInput = synopsis.trim();
    if (!storyInput) {
      setExportMessage("请先输入故事梗概，再扩写故事。");
      return;
    }

    setBridgeBusy("expand");
    try {
      const requestScene = selectedSceneOption;
      const requestDurationMode = targetDurationMode;
      const requestIsLongText = requestDurationMode === LONG_TEXT_DURATION_MODE;
      const requestDurationSeconds = requestIsLongText
        ? estimateLongTextAutoDurationSeconds(sourceInputAnalysis)
        : durationSeconds;
      const requestAnalysis = buildSceneRewriteRequestAnalysis(
        sourceInputAnalysis,
        requestScene,
        requestDurationMode,
        requestDurationSeconds,
      );
      const response = await invokeExpandScript({
        scene_type: requestScene.value,
        scene_label: requestScene.label,
        scene_category: requestScene.group,
        model_config_summary: modelConfigSummary,
        target_duration_mode: requestDurationMode,
        target_duration_seconds: requestDurationSeconds,
        selected_total_duration_seconds: requestDurationSeconds,
        story_length_profile: requestIsLongText ? "long_story_auto" : storyLengthProfileForUi(requestAnalysis, requestDurationSeconds),
        source_material_length_chars: requestAnalysis.sourceMaterialLengthChars,
        auto_segment_strategy: requestIsLongText ? AUTO_SEGMENT_STRATEGY_LONG_TEXT : AUTO_SEGMENT_STRATEGY_FIXED_SECONDS,
        source_input_type: requestAnalysis.sourceInputType,
        authoring_mode: requestAnalysis.authoringMode,
        source_material_summary: requestAnalysis.sourceMaterialSummary,
        source_story_facts: requestAnalysis.sourceStoryFacts,
        preserved_fact_summary: requestAnalysis.preservedFactSummary,
        changed_for_screenplay_summary: requestAnalysis.changedForScreenplaySummary,
        omitted_detail_summary: requestAnalysis.omittedDetailSummary,
        synopsis_text: storyInput,
      });
      const storyBody = response.expanded_script_text.trim();
      const responseDurationSeconds = requestIsLongText
        ? resolveResponseStoryDurationSeconds(response, requestDurationSeconds)
        : requestDurationSeconds;
      const nextStoryText = storyBody || storyInput;
      const changed = normalizeScriptText(nextStoryText) !== normalizeScriptText(storyInput);
      setSynopsis(nextStoryText);
      resetScriptAndStoryboardState();
      setExpandedScriptResult(response);
      setExpandedScript(nextStoryText);
      setExpandedScriptDurationSeconds(responseDurationSeconds);
      setExpandedScriptScene(requestScene);
      setShowExpandedScriptStatus(false);
      syncModelProviderStatusFromWarnings(response.warnings);
      setExportMessage(
        changed
          ? "故事材料已按当前设置更新，待确定使用。"
          : "内容变化较小，已按当前设置重新处理；可调整材料后重试。",
      );
    } catch (error) {
      setExportMessage(`扩写故事失败：${formatProductError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
  };

  const handleExpandScript = async () => {
    if (bridgeBusy) {
      return;
    }
    if (!requireDesktopRuntime(sourceInputAnalysis.actionLabel)) {
      return;
    }
    if (!confirmDiscardDirty(sourceInputAnalysis.actionLabel)) {
      return;
    }

    const storyInput = synopsis.trim();
    if (!storyInput) {
      setExportMessage("请先输入或导入故事材料，再继续处理剧本。");
      return;
    }

    setBridgeBusy("expand");
    try {
      const requestScene = selectedSceneOption;
      const requestDurationMode = targetDurationMode;
      const requestIsLongText = requestDurationMode === LONG_TEXT_DURATION_MODE;
      const requestDurationSeconds = requestIsLongText
        ? estimateLongTextAutoDurationSeconds(sourceInputAnalysis)
        : durationSeconds;
      const requestAnalysis = buildSceneRewriteRequestAnalysis(
        sourceInputAnalysis,
        requestScene,
        requestDurationMode,
        requestDurationSeconds,
      );
      const response = await invokeExpandScript({
        scene_type: requestScene.value,
        scene_label: requestScene.label,
        scene_category: requestScene.group,
        model_config_summary: modelConfigSummary,
        target_duration_mode: requestDurationMode,
        target_duration_seconds: requestDurationSeconds,
        selected_total_duration_seconds: requestDurationSeconds,
        story_length_profile: requestIsLongText ? "long_story_auto" : storyLengthProfileForUi(requestAnalysis, requestDurationSeconds),
        source_material_length_chars: requestAnalysis.sourceMaterialLengthChars,
        auto_segment_strategy: requestIsLongText ? AUTO_SEGMENT_STRATEGY_LONG_TEXT : AUTO_SEGMENT_STRATEGY_FIXED_SECONDS,
        source_input_type: requestAnalysis.sourceInputType,
        authoring_mode: requestAnalysis.authoringMode,
        source_material_summary: requestAnalysis.sourceMaterialSummary,
        source_story_facts: requestAnalysis.sourceStoryFacts,
        preserved_fact_summary: requestAnalysis.preservedFactSummary,
        changed_for_screenplay_summary: requestAnalysis.changedForScreenplaySummary,
        omitted_detail_summary: requestAnalysis.omittedDetailSummary,
        synopsis_text: storyInput,
      });
      const responseSourceType = normalizeSourceInputType(
        response.source_input_type || requestAnalysis.sourceInputType,
      );
      const responseMessage = statusMessageForSourceInputType(responseSourceType);
      const scriptBody = extractScriptBody(response.expanded_script_text) || response.expanded_script_text.trim();
      const changed = normalizeScriptText(scriptBody) !== normalizeScriptText(storyInput);
      const responseDurationSeconds = requestIsLongText
        ? resolveResponseStoryDurationSeconds(response, requestDurationSeconds)
        : requestDurationSeconds;
      setExpandedScriptResult(response);
      setExpandedScript(scriptBody);
      setSynopsis(scriptBody);
      setExpandedScriptDurationSeconds(responseDurationSeconds);
      setExpandedScriptScene(requestScene);
      setShowExpandedScriptStatus(false);
      clearConfirmedStoryboardState();
      syncModelProviderStatusFromWarnings(response.warnings);
      setExportMessage(
        changed
          ? `${responseMessage} 剧本已更新，请点击“确定使用”后进入镜头拆解。${
              requestIsLongText
                ? formatDurationPlanProductMessage(response) || `目标时长 ${requestDurationSeconds} 秒。`
                : `目标时长 ${requestDurationSeconds} 秒。`
            }${formatTextModelRunMessage(response.warnings)}`
          : `${responseMessage} 内容变化较小，请调整材料或配置后重试。${formatTextModelRunMessage(response.warnings)}`,
      );
    } catch (error) {
      setExportMessage(`${sourceInputAnalysis.actionLabel}失败：${formatProductError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
  };

  const createDraftFromSceneTask = (task: SceneTaskRecord): TaskDraftState => ({
    mode: "update",
    sourceKind: "task",
    editingTaskRecordId: task.id,
    taskName: task.name,
    selectedCandidateId: task.candidateId,
    sceneIndex: task.sceneIndex,
    shotIndexWithinScene: task.shotIndexWithinScene,
    segmentTitle: task.segmentTitle,
    scriptText: task.scriptText,
    durationSeconds: normalizeDurationOption(
      task.sourceDurationSeconds ?? task.selectedTotalDurationSeconds,
      shotCandidateBaseDuration,
    ),
    baseSegmentTitle: task.segmentTitle,
    baseScriptText: task.baseSceneText || task.scriptText,
    baseDurationSeconds: normalizeDurationOption(
      task.sourceDurationSeconds ?? task.selectedTotalDurationSeconds,
      shotCandidateBaseDuration,
    ),
    scriptTextSource: task.scriptTextSource ?? "task",
    manualEditedFields: createTaskDraftEditFlags(),
    durationHint: TASK_DRAFT_DURATION_HINT,
  });

  const confirmTaskDraftSwitch = (targetLabel: string) => {
    if (!taskDraft || !isTaskDraftDirty(taskDraft)) {
      return true;
    }

    return window.confirm(`右侧草稿尚未保存。切换到“${targetLabel}”会丢弃当前草稿改动，是否继续？`);
  };

  const openTaskDraftDialog = (mode: TaskDraftState["mode"]) => {
    if (bridgeBusy) {
      return;
    }
    if (!canImportScript) {
      setExportMessage("请先导入、扩写或编辑文案，并点击“确定使用”后再新建镜头任务。");
      return;
    }

    if (!confirmDiscardDirty(mode === "create" ? "新建镜头任务" : "更新当前镜头")) {
      return;
    }

    if (mode === "update" && currentSceneTask) {
      setTaskDraft(createDraftFromSceneTask(currentSceneTask));
      return;
    }

    const fallbackSceneIndex = shotCandidates.length + 1;
    const nextUnusedCandidate =
      shotCandidates[0];
    const firstCandidate = nextUnusedCandidate ?? {
      id: "custom",
      sceneIndex: fallbackSceneIndex,
      title: "自定义镜头片段",
      text: acceptedScript.trim(),
      preview: acceptedScript.trim(),
      durationSeconds: shotCandidateBaseDuration,
    };
    const nextShotIndex = resolveNextShotIndexForCandidate(firstCandidate.id);
    const defaultTaskName = formatDefaultShotTaskName(firstCandidate.sceneIndex, nextShotIndex);

    setTaskDraft(createTaskDraftFromCandidate(firstCandidate, defaultTaskName, shotCandidateBaseDuration, nextShotIndex));
  };

  const handleTaskCandidateSelect = (candidateId: string) => {
    const candidate = shotCandidates.find((item) => item.id === candidateId);
    if (!candidate) {
      return;
    }

    if (!confirmTaskDraftSwitch(candidate.title)) {
      return;
    }

    const nextShotIndex = resolveNextShotIndexForCandidate(candidate.id);
    setTaskDraft(
      createTaskDraftFromCandidate(
        candidate,
        formatDefaultShotTaskName(candidate.sceneIndex, nextShotIndex),
        shotCandidateBaseDuration,
        nextShotIndex,
      ),
    );
    setExportMessage(`已选择系统候选“${candidate.title}”，右侧正在编辑候选草稿；只有加入任务队列后才会成为可生成任务。`);
  };

  const handleTaskQueueDraftSelect = (task: SceneTaskRecord) => {
    if (!confirmTaskDraftSwitch(task.name)) {
      return;
    }

    setTaskDraft(createDraftFromSceneTask(task));
  };

  const handleConfirmTaskDraft = (continueCreating = false) => {
    if (!taskDraft) {
      return;
    }

    const selectedText = taskDraft.scriptText.trim();
    if (!selectedText) {
      setExportMessage("请先选择或编辑一个镜头片段，再创建镜头任务。");
      return;
    }

    const candidate = shotCandidates.find((item) => item.id === taskDraft.selectedCandidateId);
    const nextTaskName =
      taskDraft.taskName.trim() ||
      formatDefaultShotTaskName(taskDraft.sceneIndex, taskDraft.shotIndexWithinScene);
    const editingTask = taskDraft.editingTaskRecordId
      ? sceneTasks.find((task) => task.id === taskDraft.editingTaskRecordId)
      : null;
    if (taskDraft.mode === "update" && !editingTask) {
      setExportMessage("当前编辑任务已不在队列中，请重新选择右侧任务后再保存。");
      return;
    }
    const nextTaskId = editingTask?.id ?? createTaskRecordId();
    const isCreatingNewRecord = !sceneTasks.some((task) => task.id === nextTaskId);
    const nextSegmentTitle = taskDraft.segmentTitle || candidate?.title || "自定义镜头片段";
    const taskScene = acceptedScriptScene ?? selectedSceneOption;
    const taskDurationSeconds = normalizeDurationOption(taskDraft.durationSeconds, shotCandidateBaseDuration);
    const taskDurationMode = acceptedScriptTargetDurationMode ?? targetDurationMode;

    if (isCreatingNewRecord) {
      setTaskSerial((value) => value + 1);
    }

    setTaskName(nextTaskName);
    setTaskSourceScript(selectedText);
    setTaskScriptId(acceptedScriptId);
    setTaskSegmentTitle(nextSegmentTitle);
    setCurrentTaskId(nextTaskId);
    setSceneTasks((current) => {
      const nextRecord: SceneTaskRecord = {
        id: nextTaskId,
        name: nextTaskName,
        candidateId: taskDraft.selectedCandidateId,
        sceneIndex: taskDraft.sceneIndex,
        shotIndexWithinScene: taskDraft.shotIndexWithinScene,
        segmentTitle: nextSegmentTitle,
        scriptText: selectedText,
        baseSceneText: taskDraft.baseScriptText,
        scriptId: acceptedScriptId,
        sourceSceneType: taskScene.value,
        sourceSceneLabel: taskScene.label,
        sourceSceneCategory: taskScene.group,
        sourceDurationSeconds: taskDurationSeconds,
        sourceDurationMode: taskDurationMode,
        status: "draft",
        dirty: false,
        hadRowEdits: false,
        scriptTextSource: taskDraft.scriptTextSource,
        userEditedFields: taskDraft.manualEditedFields,
      };

      if (!current.some((task) => task.id === nextTaskId)) {
        return [...current, nextRecord];
      }

      return current.map((task) =>
        task.id === nextTaskId
          ? {
              ...task,
              ...nextRecord,
              taskId: undefined,
              resultId: undefined,
              rowCount: undefined,
              rowsSnapshot: undefined,
              storyboardResult: null,
              promptTextStatus: undefined,
              promptTextWarnings: undefined,
              selectedTotalDurationSeconds: undefined,
              rowsHash: undefined,
              updatedAtMs: undefined,
              baseRevision: undefined,
              exportArtifactPath: undefined,
              exportStatus: undefined,
            }
          : task,
      );
    });
    setStoryboardResult(null);
    setLastExportResult(null);
    setRows([]);
    setRowsDirty(false);
    setCurrentPage(1);
    closeStoryboardEditDialog();

    if (continueCreating && isCreatingNewRecord) {
      const currentCandidate = candidate ?? {
        id: taskDraft.selectedCandidateId,
        sceneIndex: taskDraft.sceneIndex,
        title: taskDraft.baseSegmentTitle,
        text: taskDraft.baseScriptText,
        preview: truncatePreview(taskDraft.baseScriptText),
        durationSeconds: taskDraft.baseDurationSeconds,
      };
      const nextShotIndex = resolveNextShotIndexForCandidate(currentCandidate.id) + 1;

      setTaskDraft(
        createTaskDraftFromCandidate(
          currentCandidate,
          formatDefaultShotTaskName(currentCandidate.sceneIndex, nextShotIndex),
          shotCandidateBaseDuration,
          nextShotIndex,
        ),
      );
      setExportMessage(
        `已创建镜头任务“${nextTaskName}”，可继续在“${currentCandidate.title}”下创建第 ${nextShotIndex} 个镜头，或改选左侧其他候选。`,
      );
      return;
    }

    setTaskDraft(null);
    setExportMessage(
      isCreatingNewRecord
        ? `已创建镜头任务“${nextTaskName}”，当前镜头片段为“${nextSegmentTitle}”。生成后可继续新建下一个未使用片段。`
        : `已更新镜头任务“${nextTaskName}”，当前镜头片段为“${nextSegmentTitle}”。下一步点击开始生成。`,
    );
  };

  const handleNewTask = () => {
    openTaskDraftDialog("create");
  };

  const handleImportScript = () => {
    if (bridgeBusy) {
      return;
    }
    if (!sceneTasks.length) {
      setExportMessage("请先在镜头拆解中新建镜头任务，再从这里导入镜头任务。");
      return;
    }

    setIsTaskPickerOpen(true);
  };

  const handleImportSceneTask = (task: SceneTaskRecord) => {
    if (bridgeBusy) {
      return;
    }
    if (task.id !== currentTaskId && !confirmDiscardDirty("切换镜头任务")) {
      return;
    }

    restoreSceneTaskState(task);
    setIsTaskPickerOpen(false);
    setExportMessage(
      task.rowsSnapshot?.length
        ? `已恢复镜头任务“${task.name}”：${task.rowsSnapshot.length} 行分镜快照。`
        : `已导入镜头任务“${task.name}”，当前片段为“${task.segmentTitle}”。下一步点击开始生成。`,
    );
  };

  const handleGenerate = async () => {
    if (bridgeBusy) {
      return;
    }
    if (!requireDesktopRuntime("开始生成")) {
      return;
    }
    if (!canGenerate) {
      setExportMessage("请先点击“确定使用”，并选择一个镜头任务后再开始生成。");
      return;
    }
    if (!currentSceneTask) {
      setExportMessage("生成分镜只能使用已保存的镜头任务，请先从任务队列导入或保存一个任务。");
      return;
    }
    if (!confirmDiscardDirty("重新生成当前镜头任务")) {
      return;
    }

    const source = currentSceneTask.scriptText.trim();
    const sourceTaskName = currentSceneTask.name.trim() || "未命名镜头任务";
    const sourceSegmentTitle = currentSceneTask.segmentTitle || sourceTaskName;
    const sourceScriptId = currentSceneTask.scriptId ?? acceptedScriptId ?? taskScriptId;
    const taskDurationSeconds = normalizeDurationOption(
      currentSceneTask.sourceDurationSeconds ?? durationSeconds,
      durationSeconds,
    );
    const taskSceneOption = findSceneOption(currentSceneTask.sourceSceneType ?? null) ?? acceptedScriptScene ?? selectedSceneOption;
    setTaskName(sourceTaskName);
    setTaskSourceScript(source);
    setTaskScriptId(sourceScriptId);
    setTaskSegmentTitle(sourceSegmentTitle);
    setBridgeBusy("generate");
    try {
      const response = await invokeGenerateStoryboard({
        task_name: sourceTaskName,
        script_id: sourceScriptId,
        scene_type: taskSceneOption.value,
        scene_label: currentSceneTask?.sourceSceneLabel ?? taskSceneOption.label,
        scene_category: currentSceneTask?.sourceSceneCategory ?? taskSceneOption.group,
        shot_script: source,
        primary_scene_type: taskSceneOption.value,
        primary_scene_label: currentSceneTask?.sourceSceneLabel ?? taskSceneOption.label,
        primary_scene_category: currentSceneTask?.sourceSceneCategory ?? taskSceneOption.group,
        shot_scene_type: currentSceneTask?.sourceSceneType ?? taskSceneOption.value,
        shot_scene_label: currentSceneTask?.sourceSceneLabel ?? taskSceneOption.label,
        shot_intent: sourceSegmentTitle || sourceTaskName,
        adaptation_reason:
          currentSceneTask?.sourceSceneType && currentSceneTask.sourceSceneType !== taskSceneOption.value
            ? "镜头任务使用用户选择的局部场景方向。"
            : null,
        expanded_script_text: source,
        selected_total_duration_seconds: taskDurationSeconds,
        target_duration_mode: FIXED_DURATION_MODE,
        auto_segment_strategy: AUTO_SEGMENT_STRATEGY_FIXED_SECONDS,
        model_config_summary: modelConfigSummary,
      });
      const taskCharacters = extractCharacterNamesFromText(source);
      const nextRows = response.rows.map((row) => mapGeneratedStoryboardRow(row, taskCharacters));
      if (isBlockedStoryboardResponse(response)) {
        setStoryboardResult(null);
        setRows([]);
        setRowsDirty(false);
        setLastExportResult(null);
        setExportMessage(formatBlockedStoryboardResponse(response));
        return;
      }

      setStoryboardResult(response);
      setLastExportResult(null);
      setRows(nextRows);
      setRowsDirty(false);
      setGeneratedSceneTasks((current) => {
        const nextTask: GeneratedSceneTask = {
          resultId: response.result_id,
          taskId: response.task_id,
          taskName: sourceTaskName,
          segmentTitle: sourceSegmentTitle || "自定义镜头片段",
          rowCount: nextRows.length,
        };
        return [
          ...current.filter((item) =>
            item.resultId !== response.result_id &&
            item.taskId !== response.task_id &&
            item.taskName !== nextTask.taskName,
          ),
          nextTask,
        ];
      });
      if (currentTaskId) {
        setSceneTasks((current) =>
          current.map((task) =>
            task.id === currentTaskId
              ? {
                  ...task,
                  name: sourceTaskName || task.name,
                  segmentTitle: sourceSegmentTitle || task.segmentTitle,
                  scriptText: source,
                  status: "generated",
                  taskId: response.task_id,
                  resultId: response.result_id,
                  rowCount: nextRows.length,
                  rowsSnapshot: cloneWorkbenchRows(nextRows),
                  storyboardResult: response,
                  promptTextStatus: collectPromptStatuses(response.rows),
                  promptTextWarnings: collectPromptWarningCodes(response.rows),
                  selectedTotalDurationSeconds: response.selected_total_duration_seconds,
                  sourceDurationSeconds: response.selected_total_duration_seconds,
                  sourceDurationMode: task.sourceDurationMode ?? FIXED_DURATION_MODE,
                  durationPlanSummary: response.duration_plan_summary ?? response.durationPlanSummary,
                  generatedShotTaskCount: response.generated_shot_task_count ?? response.generatedShotTaskCount,
                  rowsHash: response.rows_hash,
                  updatedAtMs: response.updated_at_ms,
                  baseRevision: response.revision,
                  dirty: false,
                  hadRowEdits: false,
                  exportArtifactPath: undefined,
                  exportStatus: undefined,
                }
              : task,
          ),
        );
      }
      setCurrentPage(1);
      closeStoryboardEditDialog();
      syncModelProviderStatusFromWarnings(response.export_status.warnings);
      setExportMessage(
        `${formatGenerateStoryboardMessage(response, nextRows.length)}；${formatTextModelRunMessage(
          response.export_status.warnings,
        )}`,
      );
    } catch (error) {
      setExportMessage(`generate_storyboard 失败：${formatError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
  };

  const handleClear = () => {
    if (bridgeBusy) {
      return;
    }
    if (!confirmDiscardDirty("清空当前分镜产出")) {
      return;
    }

    setStoryboardResult(null);
    setLastExportResult(null);
    setRows([]);
    setRowsDirty(false);
    setCurrentPage(1);
    if (currentTaskId) {
      setSceneTasks((current) =>
        current.map((task) =>
          task.id === currentTaskId
            ? {
                ...task,
                status: "draft",
                taskId: undefined,
                resultId: undefined,
                rowCount: undefined,
                rowsSnapshot: undefined,
                storyboardResult: null,
                promptTextStatus: undefined,
                promptTextWarnings: undefined,
                selectedTotalDurationSeconds: undefined,
                rowsHash: undefined,
                updatedAtMs: undefined,
                baseRevision: undefined,
                exportArtifactPath: undefined,
                exportStatus: undefined,
                dirty: false,
                hadRowEdits: false,
              }
            : task,
        ),
      );
    }
    closeStoryboardEditDialog();
    setExportMessage("当前分镜产出已清空，镜头任务和剧本文本已保留。");
  };

  const handleEditField = <K extends keyof StoryboardWorkbenchRow>(field: K, value: StoryboardWorkbenchRow[K]) => {
    setEditingDraft((current) =>
      current ? { ...current, [field]: value } : current,
    );
  };

  const handleSaveStoryboardEdit = async () => {
    if (!editingDraft) {
      return;
    }

    const nextRows = rows.map((row) => (row.id === editingDraft.id ? { ...editingDraft } : row));
    closeStoryboardEditDialog();
    setBridgeBusy("save_rows");
    try {
      const saved = await persistCurrentRows("desktop_storyboard_row_edit", nextRows);
      if (saved) {
        setExportMessage(`已保存分镜修改：${saved.result_id} / revision ${saved.revision ?? 0}`);
      } else {
        markRowsAsDirty(nextRows, "分镜修改已写入本地，保存未完成；导出前需要重新保存。");
      }
    } catch (error) {
      markRowsAsDirty(nextRows, `保存分镜修改失败，本地修改已保留：${formatProductError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
  };

  const handleAcceptExpandedScript = () => {
    const scriptText = (synopsis.trim() || expandedScript.trim());
    if (!scriptText || !expandedScriptResult) {
      setExportMessage("请先完成扩写剧本，再确认使用。");
      return false;
    }
    if (!expandedScriptSnapshotIsCurrent) {
      setExportMessage("场景类型或目标时长已变化，请重新扩写后再确认使用。");
      return false;
    }
    if (!confirmDiscardDirty("确认使用新的扩写剧本")) {
      return false;
    }

    const scriptScene = expandedScriptScene ?? selectedSceneOption;
    const responseMode = normalizeTargetDurationModeForUi(
      expandedScriptResult.target_duration_mode ?? expandedScriptResult.targetDurationMode ?? targetDurationMode,
    );
    const scriptDuration = responseMode === LONG_TEXT_DURATION_MODE
      ? expandedScriptDurationSeconds ?? estimateLongTextAutoDurationSeconds(analyzeSourceInputForUi(scriptText))
      : durationSeconds;
    const nextSceneTasks: SceneTaskRecord[] = [];
    setAcceptedScript(scriptText);
    setAcceptedScriptId(expandedScriptResult.script_id);
    setAcceptedScriptDurationSeconds(scriptDuration);
    setAcceptedScriptTargetDurationMode(responseMode);
    setAcceptedScriptScene(scriptScene);
    setExpandedScript(scriptText);
    setSynopsis(scriptText);
    setTaskSourceScript("");
    setTaskScriptId(null);
    setTaskSegmentTitle("");
    setTaskName(DEFAULT_TASK_NAME);
    setSceneTasks(nextSceneTasks);
    setCurrentTaskId(null);
    setTaskSerial(0);
    setTaskDraft(null);
    setIsTaskPickerOpen(false);
    setStoryboardResult(null);
    setLastExportResult(null);
    setGeneratedSceneTasks([]);
    setRows([]);
    setRowsDirty(false);
    setCurrentPage(1);
    setShowExpandedScriptStatus(false);
    setExportMessage(
      `已确认使用扩写剧本 ${expandedScriptResult.script_id}，系统候选将在新建镜头任务时动态显示；请先保存任务队列后再开始生成。`,
    );
    return true;
  };

  const handleDuplicate = async (rowId: string) => {
    const target = rows.find((row) => row.id === rowId);
    if (!target) {
      return;
    }

    try {
      await navigator.clipboard.writeText(target.prompt);
      setExportMessage("已复制当前行的分镜提示词。");
    } catch (error) {
      setExportMessage(`复制分镜提示词失败：${formatProductError(error)}`);
    }
  };

  const handleDelete = (rowId: string) => {
    const nextRows = renumberRows(rows.filter((row) => row.id !== rowId));
    markRowsAsDirty(nextRows, "已删除分镜行，本地修改待保存。导出前会先保存到主线 snapshot。");
    if (editingRowId === rowId) {
      closeStoryboardEditDialog();
    }
  };

  const handleJump = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const parsed = Number(jumpPage);
    if (!Number.isFinite(parsed)) {
      setJumpPage(String(currentPage));
      return;
    }

    setCurrentPage(clampPage(parsed, pageCount));
  };

  const handleStoryboardPageSizeChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    const nextPageSize = Number(event.target.value);
    if (!Number.isFinite(nextPageSize) || nextPageSize < 1) {
      return;
    }

    setStoryboardPageSizeManual(true);
    setStoryboardPageSize(nextPageSize);
    setCurrentPage(1);
  };

  const handleSaveRows = async () => {
    if (bridgeBusy) {
      return;
    }
    if (!rowsDirty) {
      setExportMessage("当前没有未保存的分镜修改。");
      return;
    }

    setBridgeBusy("save_rows");
    try {
      const saved = await persistCurrentRows("desktop_manual_save");
      if (saved) {
        setExportMessage(`已保存分镜修改：${saved.result_id} / revision ${saved.revision ?? 0}`);
      }
    } catch (error) {
      setExportMessage(`保存分镜修改失败，本地修改已保留：${formatProductError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
  };

  const buildFinalizedShotPayload = (
    sourceResult: GenerateStoryboardResponse,
    sourceRows: GeneratedStoryboardRow[],
  ) => {
    const backendRows = sourceRows;
    const shotDuration = backendRows.reduce(
      (total, row) => total + Number(row.shot_duration_seconds || row.duration_seconds || 0),
      0,
    );
    const scriptId = currentSceneTask?.scriptId ?? taskScriptId ?? acceptedScriptId ?? "";
    const shotTaskId = currentSceneTask?.id ?? sourceResult.task_id ?? currentTaskId ?? "";
    const shotOrder = currentSceneTask
      ? Math.max(1, sceneTasks.findIndex((task) => task.id === currentSceneTask.id) + 1)
      : Math.max(1, finalizedShots.length + 1);
    const promptText = backendRows
      .map((row) => row.prompt_text.trim())
      .filter(Boolean)
      .join("\n\n");

    return {
      project_id: DEFAULT_PROJECT_ID,
      script_id: scriptId,
      shot_task_id: shotTaskId,
      result_id: sourceResult.result_id,
      shot_order: shotOrder,
      shot_task_name: taskName.trim() || currentSceneTask?.name || "未命名镜头任务",
      rows: backendRows,
      prompt_text: promptText,
      shot_duration_seconds: shotDuration,
      duration_source: backendRows[0]?.duration_source || STORYBOARD_DURATION_SOURCE,
      confirmed: true,
      updated_at_ms: Date.now(),
      rows_hash: sourceResult.rows_hash || currentSceneTask?.rowsHash || "",
    };
  };

  const handleConfirmFinalizedShot = async () => {
    if (bridgeBusy) {
      return;
    }
    if (!storyboardResult || !rows.length) {
      setExportMessage("请先生成当前镜头分镜。");
      return;
    }

    setBridgeBusy("save_shot");
    try {
      const savedResult = rowsDirty
        ? await persistCurrentRows("desktop_confirm_finalized_shot_autosave")
        : storyboardResult;
      if (!savedResult) {
        return;
      }

      const payload = buildFinalizedShotPayload(savedResult, savedResult.rows);
      if (!payload.script_id || !payload.shot_task_id || !payload.result_id || !payload.rows.length) {
        setExportMessage("确定使用失败：缺少剧本、镜头任务或分镜结果，请重新导入镜头任务后生成。");
        return;
      }

      const existing = finalizedShots.find((shot) => shot.result_id === payload.result_id);
      const response = existing
        ? await invokeUpdateStoryboardShotResult({
            project_id: payload.project_id,
            result_id: payload.result_id,
            shot_order: payload.shot_order,
            shot_task_name: payload.shot_task_name,
            rows: payload.rows,
            prompt_text: payload.prompt_text,
            shot_duration_seconds: payload.shot_duration_seconds,
            duration_source: payload.duration_source,
            confirmed: true,
            updated_at_ms: payload.updated_at_ms,
            rows_hash: payload.rows_hash,
          })
        : await invokeSaveStoryboardShotResult(payload);

      if (response.status === "Blocked" || response.blockers.length) {
        setExportMessage(
          hasWarningCode(response.blockers, "storyboard_bank_rows_hash_mismatch")
            ? STORYBOARD_ROWS_HASH_MISMATCH_MESSAGE
            : `确定使用未完成：${formatWarnings(response.blockers)}`,
        );
        return;
      }
      if (response.shot) {
        setFinalizedShots((current) => upsertFinalizedShot(current, response.shot!));
      }
      await refreshFinalizedShots(false, payload.script_id);
      setLastStoryboardBankExport(null);
      setExportMessage(
        `已确定使用当前镜头：${response.shot?.shot_task_name ?? payload.shot_task_name} / ${payload.shot_duration_seconds} 秒，已进入已定稿分镜区。`,
      );
    } catch (error) {
      const message = formatProductError(error);
      setExportMessage(
        message === STORYBOARD_ROWS_HASH_MISMATCH_MESSAGE ? message : `确定使用失败：${message}`,
      );
    } finally {
      setBridgeBusy(null);
    }
  };

  const handleEditFinalizedShot = (shot: FinalizedStoryboardShotResult) => {
    if (!confirmDiscardDirty("载入已定稿镜头")) {
      return;
    }
    const nextRows = shot.rows.map((row) => mapGeneratedStoryboardRow(row));
    setRows(nextRows);
    setRowsDirty(false);
    setTaskName(shot.shot_task_name);
    setTaskScriptId(shot.script_id);
    setTaskSourceScript(shot.rows[0]?.shot_script ?? "");
    setStoryboardResult((current) =>
      current && current.result_id === shot.result_id
        ? current
        : {
            task_id: shot.shot_task_id,
            result_id: shot.result_id,
            rows: shot.rows,
            selected_total_duration_seconds: shot.shot_duration_seconds,
            duration_plan: {
              total_duration_seconds: shot.shot_duration_seconds,
              row_count: shot.rows.length,
              per_row_seconds: shot.rows.length
                ? Math.round(shot.shot_duration_seconds / shot.rows.length)
                : shot.shot_duration_seconds,
              allocated_seconds: shot.shot_duration_seconds,
            },
            export_status: {
              status: "Ready",
              blockers: [],
              warnings: [],
              ready_row_count: shot.rows.length,
              blocked_row_count: 0,
            },
            operation_id: "",
            revision: 0,
            updated_at_ms: shot.updated_at_ms,
            rows_hash: shot.rows_hash,
            dirty: false,
          },
    );
    setFinalizedBankOpen(false);
    setExportMessage(`已载入已定稿镜头：${shot.shot_task_name}。可继续查看或修改后保存。`);
  };

  const openFinalizedShotTextDialog = (shot: FinalizedStoryboardShotResult) => {
    const rowLines = shot.rows.map((row) =>
      [
        `${row.order}. ${formatInternalPlaceholder(row.shot_title)}`,
        `人物：${formatInternalPlaceholder(row.person)}`,
        `关键动作：${formatInternalPlaceholder(row.character_action)}`,
        `运镜：${formatCameraMovement(resolveCameraMovement(row))}`,
        `分镜提示词：${row.prompt_text?.trim() || EMPTY_PROMPT_TEXT_PLACEHOLDER}`,
      ].join("\n"),
    );
    setTextDialog({
      kind: "summary",
      title: `已定稿镜头：${shot.shot_task_name}`,
      helper: "这里展示已定稿集合中的产品字段，不展示内部原始提示、来源登记或密钥。",
      value: [
        `镜头序号：${shot.shot_order}`,
        `镜头时长：${shot.shot_duration_seconds} 秒`,
        `确认状态：${shot.confirmed ? "已确认" : "未确认"}`,
        `内容校验：${shot.rows_hash ? "已记录" : "未记录"}`,
        "",
        ...rowLines,
      ].join("\n\n"),
      editable: false,
    });
  };

  const handleRemoveFinalizedShot = async (shot: FinalizedStoryboardShotResult) => {
    if (bridgeBusy) {
      return;
    }
    if (!window.confirm(`确认移除已定稿镜头“${shot.shot_task_name}”？`)) {
      return;
    }

    setBridgeBusy("remove_shot");
    try {
      const response = await invokeRemoveStoryboardShotResult({
        project_id: DEFAULT_PROJECT_ID,
        result_id: shot.result_id,
      });
      if (response.status === "Blocked" || response.blockers.length) {
        setExportMessage(`移除失败：${formatWarnings(response.blockers)}`);
        return;
      }
      if (response.removed) {
        setFinalizedShots((current) =>
          current.filter((item) => item.result_id !== response.result_id),
        );
        setLastStoryboardBankExport(null);
      }
      setExportMessage(response.removed ? "已移除该已定稿镜头。" : "未找到需要移除的已定稿镜头。");
    } catch (error) {
      setExportMessage(`移除已定稿镜头失败：${formatProductError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
  };

  const handleExportStoryboardBank = async () => {
    if (bridgeBusy) {
      return;
    }
    if (!confirmedShotCount) {
      setExportMessage("暂无已确认分镜，无法导出完整剧本。");
      return;
    }

    setBridgeBusy("export_bank");
    setExportMessage("正在导出");
    try {
      const targetPath = await selectExportSavePath(buildDefaultExcelFileName("full_script"));
      if (!targetPath) {
        setLastStoryboardBankExport(null);
        setExportMessage("已取消导出");
        return;
      }
      const response = await invokeExportStoryboardBank({
        project_id: DEFAULT_PROJECT_ID,
        script_id: acceptedScriptId ?? taskScriptId ?? null,
        export_format: "storyboard_bank",
        include_unconfirmed: false,
      });
      setLastStoryboardBankExport(response);
      if (response.no_export) {
        setExportMessage("暂无已确认分镜，无法导出完整剧本。");
        return;
      }
      if (response.status === "Blocked" || response.blockers.length) {
        setExportMessage(`导出失败：${formatWarnings(response.blockers)}`);
        return;
      }
      const excelArtifact = findPrimaryExcelArtifact(response);
      if (!excelArtifact?.artifact_path) {
        setExportMessage("导出失败：未生成可保存的 Excel 文件。");
        return;
      }
      const savedPath = await copyExportArtifactToPath(excelArtifact.artifact_path, targetPath);
      setExportMessage(`已导出：${savedPath}`);
    } catch (error) {
      setExportMessage(`导出失败：${formatProductError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
  };

  const handleExportWords = async () => {
    if (bridgeBusy) {
      return;
    }
    if (!storyboardResult || !rows.length) {
      setExportMessage("当前没有可导出的分镜词。");
      return;
    }

    setBridgeBusy("export_words");
    setExportMessage("正在导出");
    try {
      const targetPath = await selectExportSavePath(buildDefaultExcelFileName("storyboard_words"));
      if (!targetPath) {
        setLastExportResult(null);
        setExportMessage("已取消导出");
        return;
      }
      const exportSource = await ensureRowsSavedForExport("desktop_export_words_autosave");
      if (!exportSource) {
        return;
      }
      const response = await invokeExportBundle({
        result_id: exportSource.result_id,
        task_id: exportSource.task_id ?? currentSceneTask?.taskId ?? currentTaskId,
        export_format: "storyboard_words",
      });
      if (currentSceneTask?.hadRowEdits && hasEditedRowsNotApplied(response)) {
        setExportMessage("导出已阻断：本地编辑未被主线 snapshot 应用，请先保存修改后再导出。");
        return;
      }
      setLastExportResult(response);
      const excelArtifact = findPrimaryExcelArtifact(response);
      if (!excelArtifact?.artifact_path) {
        setExportMessage("导出失败：未生成可保存的 Excel 文件。");
        return;
      }
      const savedPath = await copyExportArtifactToPath(excelArtifact.artifact_path, targetPath);
      updateCurrentTaskRecord({
        exportArtifactPath: savedPath,
        exportStatus: response.export_status.status,
      });
      setExportMessage(`已导出：${savedPath}`);
    } catch (error) {
      setExportMessage(`导出失败：${formatProductError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
  };

  const handleExportScript = async () => {
    if (bridgeBusy) {
      return;
    }
    if (!storyboardResult || !rows.length) {
      setExportMessage("当前没有可导出的完整剧本结果。");
      return;
    }

    if (sceneTasks.length > generatedSceneTaskCount) {
      setExportMessage(
        `完整剧本需要所有镜头任务都完成生成；当前已生成 ${generatedSceneTaskCount}/${sceneTasks.length} 个，未完成任务不会被伪造成导出结果。`,
      );
      return;
    }

    if (generatedSceneTaskCount > 1 || generatedSceneTasks.length > 1) {
      setExportMessage(
        "多镜头完整导出等待已定稿分镜集合接入；当前不会把单镜头结果伪造成全片导出。",
      );
      return;
    }

    setBridgeBusy("export_script");
    setExportMessage("正在导出");
    try {
      const targetPath = await selectExportSavePath(buildDefaultExcelFileName("full_script"));
      if (!targetPath) {
        setLastExportResult(null);
        setExportMessage("已取消导出");
        return;
      }
      const exportSource = await ensureRowsSavedForExport("desktop_export_script_autosave");
      if (!exportSource) {
        return;
      }
      const response = await invokeExportBundle({
        result_id: exportSource.result_id,
        task_id: exportSource.task_id ?? currentSceneTask?.taskId ?? currentTaskId,
        export_format: "full_script",
      });
      if (currentSceneTask?.hadRowEdits && hasEditedRowsNotApplied(response)) {
        setExportMessage("完整剧本导出已阻断：本地编辑未被主线 snapshot 应用，请先保存修改后再导出。");
        return;
      }
      setLastExportResult(response);
      const excelArtifact = findPrimaryExcelArtifact(response);
      if (!excelArtifact?.artifact_path) {
        setExportMessage("导出失败：未生成可保存的 Excel 文件。");
        return;
      }
      const savedPath = await copyExportArtifactToPath(excelArtifact.artifact_path, targetPath);
      updateCurrentTaskRecord({
        exportArtifactPath: savedPath,
        exportStatus: response.export_status.status,
      });
      setExportMessage(`已导出：${savedPath}`);
    } catch (error) {
      setExportMessage(`导出失败：${formatProductError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
  };

  const emptyStoryboardAction = !sceneTasks.length
    ? {
        label: "新建镜头任务",
        onClick: handleNewTask,
        disabled: bridgeBusy !== null || !canImportScript,
      }
    : !hasTaskScript
    ? {
        label: "导入镜头任务",
        onClick: handleImportScript,
        disabled: bridgeBusy !== null,
      }
    : {
        label: "开始生成",
        onClick: handleGenerate,
        disabled: !desktopRuntimeAvailable || bridgeBusy !== null || !canGenerate,
      };

  return (
    <Shell>
      <div className="reference-workbench">
        <div className="reference-shell">
          <header className="reference-topbar">
            <div className="reference-brand">
              <img className="reference-brand__logo" src={logoUrl} alt="Hope Logo" />
              <div className="reference-brand__copy">
                <h1>Hope 动漫分镜脚本生成工作台</h1>
              </div>
            </div>

            <div className="reference-topbar__controls">
              <label className="top-select">
                <span>模型选择：</span>
                <select
                  value={selectedModel}
                  onChange={(event) => handleModelSelect(event.target.value as WorkbenchModelId)}
                >
                  {MODEL_OPTIONS.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
                <small className="top-select__meta">
                  {selectedModelLabel} · {modelConfig.model}
                </small>
                <small className={`top-select__status top-select__status--${modelRuntimeTone}`}>
                  {modelRuntimeLabel}
                </small>
              </label>
              {!desktopRuntimeAvailable ? (
                <span className={`top-select__status top-select__status--${desktopRuntimeTone}`}>
                  {desktopRuntimeLabel}
                </span>
              ) : null}
              <button
                type="button"
                className={activePanel === "docs" ? "toolbar-button toolbar-button--active" : "toolbar-button"}
                onClick={() => setActivePanel((value) => (value === "docs" ? "none" : "docs"))}
              >
                API文档
              </button>
              <button
                type="button"
                className={activePanel === "api" ? "toolbar-button toolbar-button--active" : "toolbar-button"}
                onClick={() => setActivePanel((value) => (value === "api" ? "none" : "api"))}
              >
                API接口
              </button>
              {modelReservedWarning ? (
                <span className="model-reserved-warning">{modelReservedWarning}</span>
              ) : null}
            </div>
          </header>

          {activePanel !== "none" ? (
            <div
              className="top-modal-backdrop"
              onClick={() => setActivePanel("none")}
            >
              <section
                className={activePanel === "docs" ? "top-panel top-panel--modal top-panel--docs-modal" : "top-panel top-panel--modal"}
                onClick={(event) => event.stopPropagation()}
              >
              <div className="top-panel__title top-panel__title--split">
                <span>{activePanel === "docs" ? "API文档" : "API接口"}</span>
                <button type="button" className="toolbar-button toolbar-button--compact" onClick={() => setActivePanel("none")}>
                  关闭
                </button>
              </div>
              {activePanel === "docs" ? (
                <div className="api-docs">
                  {API_DOC_SECTIONS.map((section) => (
                    <section key={section.title}>
                      <h2>{section.title}</h2>
                      <ul>
                        {section.items.map((item) => (
                          <li key={item}>{item}</li>
                        ))}
                      </ul>
                    </section>
                  ))}
                </div>
              ) : (
                <form className="api-config" onSubmit={handleSaveModelConfig}>
                  <div className="api-config__notice">
                    当前只启用千问文本生成配置；API Key 仅保存在本次应用会话中。关键验收必须在正式应用窗口内完成。
                  </div>
                  <div className="api-config__rows">
                    <div className="api-config__row api-config__row--primary">
                      <label>
                        <span>模型服务</span>
                        <select
                          value={modelConfigDraft.provider}
                          onChange={(event) =>
                            handleModelConfigDraftChange("provider", event.target.value as WorkbenchModelId)
                          }
                        >
                          {MODEL_OPTIONS.map((option) => (
                            <option key={option.value} value={option.value}>
                              {option.label}
                            </option>
                          ))}
                        </select>
                      </label>
                      <label>
                        <span>模型名称</span>
                        <input
                          name="hope-model-name"
                          autoComplete="off"
                          value={modelConfigDraft.model}
                          onChange={(event) => handleModelConfigDraftChange("model", event.target.value)}
                          placeholder="qwen-plus"
                        />
                      </label>
                      <label>
                        <span>接口地址</span>
                        <input
                          name="hope-model-base-url"
                          autoComplete="off"
                          value={modelConfigDraft.baseUrl}
                          onChange={(event) => handleModelConfigDraftChange("baseUrl", event.target.value)}
                          placeholder="https://..."
                        />
                      </label>
                    </div>
                    <div className="api-config__row api-config__row--secret">
                      <label>
                        <span>API Key</span>
                        <input
                          name="hope-model-api-key"
                          type="password"
                          value={modelConfigDraft.apiKeyInput}
                          onChange={(event) => handleModelConfigDraftChange("apiKeyInput", event.target.value)}
                          placeholder={modelConfigDraft.apiKeyPresent ? "已配置，本次会话有效；保存时不显示明文" : "在这里输入千问 API Key"}
                          autoComplete="new-password"
                          data-lpignore="true"
                        />
                        <small>保存后会清空输入框；不会显示明文。</small>
                      </label>
                      <label>
                        <span>密钥来源</span>
                        <select
                          name="hope-model-api-key-ref"
                          autoComplete="off"
                          data-lpignore="true"
                          value={normalizeApiKeyRef(modelConfigDraft.apiKeyRef)}
                          onChange={(event) => handleModelConfigDraftChange("apiKeyRef", event.target.value)}
                        >
                          {API_KEY_REF_OPTIONS.map((option) => (
                            <option key={option.value || "none"} value={option.value}>
                              {option.label}
                            </option>
                          ))}
                        </select>
                        <small>可选择环境变量或钥匙串引用，不在这里填写真实 Key。</small>
                      </label>
                      <label className="api-config__toggle">
                        <input
                          type="checkbox"
                          checked={modelConfigDraft.enabled}
                          onChange={(event) => handleModelConfigDraftChange("enabled", event.target.checked)}
                        />
                        <span>启用当前模型</span>
                      </label>
                      <button
                        type="submit"
                        className="action-button action-button--dark"
                        disabled={!desktopRuntimeAvailable}
                      >
                        保存配置
                      </button>
                    </div>
                    <div className="api-config__summary">
                      当前状态：{desktopRuntimeLabel} · {modelRuntimeLabel}。API Key 不会显示明文；调用失败时会使用本地候选结果。
                      {modelConfigDraft.provider !== "qwen" ? (
                        <strong>该模型接口为预留状态，当前未启用真实调用。</strong>
                      ) : null}
                    </div>
                  </div>
                </form>
              )}
              </section>
            </div>
          ) : null}

          <section className="panel-section">
            <div className="section-name">剧本区</div>
            <div className="script-row">
              <div className="script-settings">
                <label className="scene-select">
                  <span>场景类型：</span>
                  <select
                    value={selectedScene}
                    onChange={(event) => handleSceneSelectionChange(event.target.value as SceneFusionOption)}
                  >
                    {sceneOptionGroups.map(([group, options]) => (
                      <optgroup key={group} label={group}>
                        {options.map((option) => (
                          <option key={option.value} value={option.value}>
                            {option.label}
                          </option>
                        ))}
                      </optgroup>
                    ))}
                  </select>
                  <small className="scene-select__meta">
                    {selectedSceneOption.group} · {selectedSceneOption.value}
                  </small>
                </label>
                <label className="duration-select duration-select--script">
                  <span>单镜头时长</span>
                  <select
                    value={durationSelectValue}
                    onChange={(event) => handleDurationSelectionChange(event.target.value)}
                  >
                    {DURATION_OPTIONS.map((option) => (
                      <option key={option} value={option}>
                        {option} 秒
                      </option>
                    ))}
                    <option value={LONG_TEXT_DURATION_MODE}>长文本模式</option>
                  </select>
                </label>
              </div>
              <div className="text-control">
                <div className={synopsis.trim() ? "synopsis-preview" : "synopsis-preview synopsis-preview--empty"}>
                  <span className="synopsis-preview__text">
                    {synopsis.trim() ? truncatePreview(synopsis, 120) : "请输入或导入故事材料"}
                  </span>
                  <small className="source-input-status">{sourceInputStatusText}</small>
                </div>
                <div className="text-control__actions">
                  <button type="button" className="text-control__button" onClick={openSynopsisDialog}>
                    放大编辑
                  </button>
                  <button
                    type="button"
                    className="text-control__button"
                    onClick={handleConfirmCurrentScriptUse}
                    disabled={bridgeBusy !== null || !synopsis.trim()}
                  >
                    确定使用
                  </button>
                </div>
              </div>
              <div className="script-actions">
                <button
                  type="button"
                  className="action-button action-button--dark"
                  onClick={handleImportStoryDocument}
                  disabled={bridgeBusy !== null}
                >
                  {bridgeBusy === "import_document" ? "导入中" : "导入文档"}
                </button>
                <button
                  type="button"
                  className="action-button action-button--dark"
                  onClick={handleExpandStory}
                  disabled={!desktopRuntimeAvailable || bridgeBusy !== null || !synopsis.trim()}
                >
                  {bridgeBusy === "expand" ? "扩写中" : "扩写故事"}
                </button>
                <button
                  type="button"
                  className="action-button action-button--dark"
                  onClick={handleExpandScript}
                  disabled={!desktopRuntimeAvailable || bridgeBusy !== null || !synopsis.trim()}
                >
                  {bridgeBusy === "expand" ? "处理中" : sourceInputAnalysis.actionLabel}
                </button>
              </div>
            </div>
          </section>

          <div
            className={`content-bridge-row content-bridge-row--${storyBridgeView.state}`}
            aria-label="内容承接状态"
          >
            <div className="content-bridge-summary">
              {storyBridgeView.summaryItems.map((item) => (
                <span key={item}>
                  <strong>{formatBridgeItemLabel(item)}</strong>
                  {formatBridgeItemValue(item)}
                </span>
              ))}
            </div>
          </div>

          <section className="panel-section panel-section--task">
            <div className="section-name section-name--inline">
              <span>镜头拆解</span>
            </div>
            <div className="task-row task-row--compact">
              <label className="task-index-select">
                <span>镜头序号</span>
                <strong className="task-index-value">
                  {currentSceneTask ? currentSceneTaskIndex : "暂无"}
                </strong>
              </label>
              <label className="task-name-select">
                <button
                  type="button"
                  className={currentSceneTask ? "task-current-card" : "task-current-card task-current-card--empty"}
                  onClick={handleImportScript}
                  disabled={!sceneTasks.length || bridgeBusy !== null}
                  aria-label="选择镜头任务"
                  title={
                    currentSceneTask
                      ? `第 ${currentSceneTaskIndex} 个 · ${currentSceneTask.name} · ${currentSceneTaskStatus?.label ?? "待生成"}`
                      : taskName
                  }
                >
                  <strong>{currentSceneTask ? currentSceneTask.name : taskName || "暂无镜头任务"}</strong>
                  <span>{currentSceneTask ? currentSceneTask.segmentTitle : "请先创建或导入任务"}</span>
                  <em className={`task-current-card__status task-current-card__status--${currentSceneTaskStatus?.tone ?? "pending"}`}>
                    {currentSceneTaskStatus?.label ?? "待生成"}
                  </em>
                </button>
              </label>
              <button
                type="button"
                className="action-button action-button--light"
                onClick={handleNewTask}
                disabled={bridgeBusy !== null || !canImportScript}
              >
                新建镜头任务
              </button>
              <button
                type="button"
                className="action-button action-button--light"
                onClick={handleImportScript}
                disabled={!sceneTasks.length || bridgeBusy !== null}
              >
                导入镜头任务
              </button>
              <div
                className="duration-pill"
                title={currentTaskDuration ? `当前镜头时长 ${currentTaskDuration} 秒` : "当前镜头时长待确认"}
                aria-label={currentTaskDuration ? `当前镜头时长 ${currentTaskDuration} 秒` : "当前镜头时长待确认"}
              >
                <strong>当前镜头时长</strong>
                <span>{currentTaskDuration ? `${currentTaskDuration} 秒` : "待确认"}</span>
              </div>
              <button
                type="button"
                className="action-button action-button--light"
                onClick={handleClear}
                disabled={bridgeBusy !== null}
              >
                清空
              </button>
              <button
                type="button"
                className="action-button action-button--light"
                onClick={handleSaveRows}
                disabled={!rowsDirty || bridgeBusy !== null}
              >
                {bridgeBusy === "save_rows" ? "保存中" : "保存修改"}
              </button>
              <div
                className={`current-shot-status current-shot-status--${currentShotStatusTone}`}
              >
                <strong>当前镜头</strong>
                <span>{currentShotStatusLabel}</span>
              </div>
              <button
                type="button"
                className="action-button action-button--danger"
                onClick={handleGenerate}
                disabled={!desktopRuntimeAvailable || !canGenerate || bridgeBusy !== null}
              >
                {bridgeBusy === "generate" ? "生成中" : "开始生成"}
              </button>
            </div>
          </section>

          <section className="panel-section panel-section--storyboard">
            <div className="section-name">当前镜头结果</div>

            <div className={rows.length ? "table-wrapper" : "table-wrapper table-wrapper--single-page"}>
              <table className="storyboard-table">
                <thead>
                  <tr>
                    <th>序号</th>
                    <th>人物</th>
                    <th>运镜</th>
                    <th>景别</th>
                    <th>画面描述</th>
                    <th>角色动作</th>
                    <th>对白/旁白</th>
                    <th>分镜提示词</th>
                    <th>时长(秒)</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody>
                  {pagedRows.length ? (
                    pagedRows.map((row) => (
                      <tr key={row.id}>
                        <td>{row.order}</td>
                        <td>
                          <LongTextCell
                            label="人物"
                            value={formatInternalPlaceholder(row.person)}
                            compact
                            onOpen={() => openStoryboardCellDialog(row, "person", "人物")}
                          />
                        </td>
                        <td>
                          <LongTextCell
                            label="运镜"
                            value={row.cameraMovement}
                            compact
                            onOpen={() => openStoryboardCellDialog(row, "cameraMovement", "运镜")}
                          />
                        </td>
                        <td>
                          <span className="table-compact-text" title={row.sceneScale}>
                            {formatSceneScaleLabel(row.sceneScale)}
                          </span>
                        </td>
                        <td>
                          <LongTextCell
                            label="画面描述"
                            value={row.visualDescription}
                            onOpen={() => openStoryboardCellDialog(row, "visualDescription", "画面描述")}
                          />
                        </td>
                        <td>
                          <LongTextCell
                            label="角色动作"
                            value={row.characterAction}
                            onOpen={() => openStoryboardCellDialog(row, "characterAction", "角色动作")}
                          />
                        </td>
                        <td>
                          <LongTextCell
                            label="对白/旁白"
                            value={row.dialogue}
                            onOpen={() => openStoryboardCellDialog(row, "dialogue", "对白/旁白")}
                          />
                        </td>
                        <td>
                          <LongTextCell
                            label="分镜提示词"
                            value={row.prompt}
                            onOpen={() => openStoryboardCellDialog(row, "prompt", "分镜提示词")}
                          />
                          </td>
                          <td>
                            <span
                              className="table-compact-text"
                              title={`权威时长：${row.shotDurationSeconds ?? row.durationSeconds} 秒；${formatDurationSource(row.durationSource)}`}
                            >
                              {row.shotDurationSeconds ?? row.durationSeconds}
                            </span>
                          </td>
                        <td>
                          <div className="row-actions row-actions--stacked">
                            <button type="button" className="link-button" onClick={() => openStoryboardEditDialog(row)}>
                              修改
                            </button>
                            <button type="button" className="link-button" onClick={() => handleDuplicate(row.id)}>
                              复制
                            </button>
                            <button type="button" className="link-button link-button--danger" onClick={() => handleDelete(row.id)}>
                              删除
                            </button>
                            <button
                              type="button"
                              className="link-button link-button--primary link-button--confirm-shot"
                              onClick={() => void handleConfirmFinalizedShot()}
                              disabled={!storyboardResult || !rows.length || bridgeBusy !== null || (currentShotConfirmed && !rowsDirty)}
                              title={rows.length ? "确定使用当前镜头结果，加入已定稿分镜区" : "请先生成当前镜头分镜"}
                              aria-label="确定使用当前镜头结果，加入已定稿分镜区"
                            >
                              <span>确定使用</span>
                              <small>当前镜头</small>
                            </button>
                          </div>
                        </td>
                      </tr>
                    ))
                  ) : (
                    <tr>
                      <td colSpan={10} className="empty-table-cell">
                        <div className="storyboard-empty-state">
                          <strong>当前还没有生成分镜</strong>
                          <span>请先导入镜头任务，再点击“开始生成”。</span>
                          <button
                            type="button"
                            className="action-button action-button--light"
                            onClick={emptyStoryboardAction.onClick}
                            disabled={emptyStoryboardAction.disabled}
                          >
                            {emptyStoryboardAction.label}
                          </button>
                        </div>
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>

            {rows.length ? (
            <div className="pagination-row">
              <div className="page-size-control">
                <label htmlFor="storyboard-page-size">每页显示</label>
                <select id="storyboard-page-size" value={storyboardPageSize} onChange={handleStoryboardPageSizeChange}>
                  {STORYBOARD_PAGE_SIZE_OPTIONS.map((size) => (
                    <option key={size} value={size}>
                      {size} 条
                    </option>
                  ))}
                </select>
              </div>

              <div className="total-count">当前显示 {pageStartRow}-{pageEndRow} / 共 {rows.length} 条 · 第 {currentPage}/{pageCount} 页</div>

              <div className="page-switcher">
                <button
                  type="button"
                  className="page-arrow"
                  onClick={() => setCurrentPage((value) => clampPage(value - 1, pageCount))}
                  disabled={currentPage === 1}
                >
                  &lt;
                </button>
                {pageTokens.map((token, index) =>
                  token === "..." ? (
                    <span key={`ellipsis-${index}`} className="page-ellipsis">
                      ...
                    </span>
                  ) : (
                    <button
                      key={token}
                      type="button"
                      className={token === currentPage ? "page-number page-number--active" : "page-number"}
                      onClick={() => setCurrentPage(token)}
                    >
                      {token}
                    </button>
                  ),
                )}
                <button
                  type="button"
                  className="page-arrow"
                  onClick={() => setCurrentPage((value) => clampPage(value + 1, pageCount))}
                  disabled={currentPage === pageCount}
                >
                  &gt;
                </button>
              </div>

              <form className="jump-form" onSubmit={handleJump}>
                <span>跳至</span>
                <input
                  value={jumpPage}
                  onChange={(event) => setJumpPage(event.target.value.replace(/[^0-9]/g, ""))}
                  inputMode="numeric"
                />
                <span>页</span>
                <button type="submit">跳转</button>
              </form>
            </div>
            ) : null}
          </section>

          <section className="export-row">
            <div className="finalized-bank">
              <strong>已定稿分镜区</strong>
              <span>已确认 {confirmedShotCount}/{sceneTasks.length} 个镜头</span>
              <span>总时长 {finalizedStoryboardTotalDuration} 秒</span>
              <button type="button" className="link-button" onClick={handleOpenFinalizedStoryboardDialog}>
                查看全部
              </button>
            </div>
            <div className="export-actions">
              <button
                type="button"
                className="action-button action-button--light"
                onClick={handleExportWords}
                disabled={!storyboardResult || bridgeBusy !== null}
              >
                {bridgeBusy === "export_words" ? "导出中" : "导出分镜词"}
              </button>
              <button
                type="button"
                className="action-button action-button--dark"
                onClick={handleExportStoryboardBank}
                disabled={bridgeBusy !== null}
              >
                {bridgeBusy === "export_bank" ? "导出中" : "导出完整剧本"}
              </button>
            </div>
          </section>
        </div>

        {finalizedBankOpen ? (
          <div className="edit-dialog-backdrop" onClick={() => setFinalizedBankOpen(false)}>
            <div className="finalized-bank-dialog" onClick={(event) => event.stopPropagation()}>
              <div className="edit-dialog__header">
                <div>
                  <strong>已定稿分镜区</strong>
                  <span>这里展示已确认进入完整分镜包的镜头，不会把当前单镜头结果伪造成全片结果。</span>
                </div>
                <button type="button" className="toolbar-button" onClick={() => setFinalizedBankOpen(false)}>
                  关闭
                </button>
              </div>
              <div className="finalized-bank-dialog__summary">
                <span>已确认 {confirmedShotCount}/{sceneTasks.length} 个镜头</span>
                <span>总时长 {finalizedStoryboardTotalDuration} 秒</span>
                <button
                  type="button"
                  className="link-button"
                  onClick={() => void refreshFinalizedShots(false)}
                  disabled={bridgeBusy !== null}
                >
                  刷新
                </button>
              </div>
              <div className="finalized-bank-table-wrap">
                <table className="finalized-bank-table">
                  <thead>
                    <tr>
                      <th>镜头序号</th>
                      <th>镜头任务名</th>
                      <th>镜头时长</th>
                      <th>人物</th>
                      <th>关键动作</th>
                      <th>分镜提示状态</th>
                      <th>确认</th>
                      <th>内容校验</th>
                      <th>操作</th>
                    </tr>
                  </thead>
                  <tbody>
                    {finalizedShots.length ? (
                      finalizedShots.map((shot) => {
                        const firstRow = shot.rows[0];
                        const promptReady = shot.prompt_text.trim()
                          ? "已生成"
                          : EMPTY_PROMPT_TEXT_PLACEHOLDER;
                        return (
                          <tr key={shot.result_id}>
                            <td>{shot.shot_order}</td>
                            <td>{shot.shot_task_name}</td>
                            <td>{shot.shot_duration_seconds} 秒</td>
                            <td>{formatInternalPlaceholder(firstRow?.person ?? "not_specified")}</td>
                            <td>{formatInternalPlaceholder(firstRow?.character_action ?? "待明确")}</td>
                            <td>{promptReady}</td>
                            <td>{shot.confirmed ? "已确认" : "未确认"}</td>
                            <td>{shot.rows_hash ? "已记录" : "未记录"}</td>
                            <td>
                              <div className="row-actions">
                                <button
                                  type="button"
                                  className="link-button"
                                  onClick={() => openFinalizedShotTextDialog(shot)}
                                >
                                  查看
                                </button>
                                <button
                                  type="button"
                                  className="link-button"
                                  onClick={() => handleEditFinalizedShot(shot)}
                                >
                                  编辑
                                </button>
                                <button
                                  type="button"
                                  className="link-button link-button--danger"
                                  onClick={() => void handleRemoveFinalizedShot(shot)}
                                  disabled={bridgeBusy !== null}
                                >
                                  移除
                                </button>
                              </div>
                            </td>
                          </tr>
                        );
                      })
                    ) : (
                      <tr>
                        <td colSpan={9} className="empty-table-cell">
                          暂无已确认分镜。请先生成当前镜头结果，再点击“确定使用”。
                        </td>
                      </tr>
                    )}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        ) : null}

        {taskDraft ? (
          <div className="edit-dialog-backdrop" onClick={() => setTaskDraft(null)}>
            <div className="task-draft-dialog" onClick={(event) => event.stopPropagation()}>
              <div className="edit-dialog__header">
                <div>
                  <strong>{taskDraft.mode === "create" ? "新建镜头任务" : "更新镜头剧本"}</strong>
                  <span>
                    系统候选来自已确认文本；分镜阶段只拆分和镜头化，不自动扩写剧情。
                  </span>
                </div>
                <button type="button" className="toolbar-button" onClick={() => setTaskDraft(null)}>
                  关闭
                </button>
              </div>
              <div className="task-draft-grid">
                <div className="task-draft-sidebar">
                  <div className="task-draft-summary">
                    候选 {shotCandidates.length} 个 · 已建 {sceneTasks.length} 个 · 已生成 {generatedSceneTaskCount} 个
                  </div>
                  <div className="shot-candidate-list">
                    <div className="task-draft-label">系统建议（未应用）</div>
                    {shotCandidates.length ? shotCandidates.map((candidate) => {
                      const isCandidateInQueue = sceneTasks.some((task) => task.candidateId === candidate.id);
                      const isCurrentCandidateDraft =
                        taskDraft.sourceKind === "candidate" && taskDraft.selectedCandidateId === candidate.id;

                      return (
                        <button
                          key={candidate.id}
                          type="button"
                          className={
                            isCurrentCandidateDraft
                              ? "shot-candidate shot-candidate--active"
                              : "shot-candidate"
                          }
                          onClick={() => handleTaskCandidateSelect(candidate.id)}
                        >
                          <span className="shot-candidate__head">
                            <strong>{candidate.title}</strong>
                            <span className="shot-candidate__badges">
                              {isCurrentCandidateDraft ? (
                                <em className="shot-candidate__status shot-candidate__status--current">当前</em>
                              ) : null}
                              {!isCurrentCandidateDraft && isCandidateInQueue ? (
                                <em className="shot-candidate__status shot-candidate__status--queued">已加入队列</em>
                              ) : null}
                            </span>
                          </span>
                          <span>{candidate.preview}</span>
                          <small>预计任务 {candidate.durationSeconds} 秒</small>
                        </button>
                      );
                    }) : (
                      <div className="shot-candidate shot-candidate--empty">
                        <strong>暂无可拆分内容</strong>
                        <span>请先扩写剧本，或在右侧直接输入自定义片段。</span>
                      </div>
                    )}
                  </div>
                </div>
                <div className="task-draft-editor">
                  <div className="task-queue-select">
                    <div className="task-draft-label">镜头任务队列（将用于生成）</div>
                    {sceneTasks.length ? (
                      <div className="task-queue-select__field">
                        <div className="task-queue-select__current">
                          <span>当前选择</span>
                          <strong>
                            {currentTaskDraftQueueTask
                              ? currentTaskDraftQueueTask.name
                              : taskDraft.segmentTitle || "系统候选草稿"}
                          </strong>
                          <small>
                            {currentTaskDraftQueueTask
                              ? currentTaskDraftQueueTask.segmentTitle
                              : "未选中队列任务"}
                          </small>
                        </div>
                        <div className="task-queue-select__picker">
                          <select
                            aria-label="选择任务队列中的镜头任务"
                            value={taskDraft.editingTaskRecordId ?? ""}
                            onChange={(event) => {
                              const selectedTask = sceneTasks.find((task) => task.id === event.target.value);
                              if (selectedTask) {
                                handleTaskQueueDraftSelect(selectedTask);
                              }
                            }}
                          >
                            <option value="" disabled>
                              候选草稿（未入队）
                            </option>
                            {sceneTasks.map((task, index) => {
                              const queueStatus = getSceneTaskStatus(task, confirmedSceneTaskIds);
                              return (
                                <option key={task.id} value={task.id}>
                                  {formatTaskQueueSelectOption(task, index, queueStatus.label)}
                                </option>
                              );
                            })}
                          </select>
                          <em
                            className={`task-queue-select__status task-queue-select__status--${currentTaskDraftQueueTone}`}
                          >
                            {currentTaskDraftQueueStatusLabel}
                          </em>
                        </div>
                      </div>
                    ) : (
                      <div className="task-queue__empty">还没有镜头任务，确认创建后会出现在这里。</div>
                    )}
                  </div>
                  <div className="task-draft-source-summary">
                    <strong>当前任务来源</strong>
                    <span>
                      {taskDraft.segmentTitle || "自定义片段"} · 预计任务 {taskDraft.durationSeconds} 秒 ·
                      人物：{extractCharacterNamesFromText(taskDraft.scriptText).join("、") || "未识别"}
                    </span>
                    <small>
                      {taskDraft.editingTaskRecordId
                        ? "正在编辑队列中的已创建任务，保存后会覆盖该任务。"
                        : "当前是系统候选草稿，确认后才会加入任务队列。"}
                    </small>
                  </div>
                  <div className="task-draft-editor__fields">
                    <label className="task-draft-editor__name">
                      <span>镜头任务名称</span>
                      <input
                        value={taskDraft.taskName}
                        onChange={(event) =>
                          setTaskDraft((current) =>
                            current
                              ? {
                                  ...current,
                                  taskName: event.target.value,
                                  manualEditedFields: {
                                    ...current.manualEditedFields,
                                    name: true,
                                  },
                                }
                              : current,
                          )
                        }
                      />
                    </label>
                    <label className="task-draft-editor__duration">
                      <span>预计任务时长</span>
                      <select
                        value={taskDraft.durationSeconds}
                        onChange={(event) =>
                          setTaskDraft((current) =>
                            current
                              ? updateTaskDraftDurationFromSystemCandidates(
                                  current,
                                  Number(event.target.value),
                                  shotCandidates,
                                )
                              : current,
                          )
                        }
                      >
                        {DURATION_OPTIONS.map((option) => (
                          <option key={option} value={option}>
                            {option} 秒
                          </option>
                        ))}
                      </select>
                      <small className="task-draft-editor__duration-note">
                        {taskDraft.durationHint}
                      </small>
                    </label>
                  </div>
                  <div className="task-draft-editor__text">
                    <div className="task-draft-editor__text-meta">
                      <span>本次镜头任务使用的剧本片段</span>
                      <small className="task-draft-editor__source">
                        可手动编辑；系统不会自动扩写或续写剧情。当前编辑：{taskDraft.sourceKind === "task" ? "任务队列" : "系统候选"} ·
                        {taskDraft.scriptTextSource === "manual"
                          ? " 手动编辑中"
                          : taskDraft.sourceKind === "task"
                          ? " 队列任务原文"
                          : " 系统候选原文"}
                      </small>
                    </div>
                    <textarea
                      value={taskDraft.scriptText}
                      onChange={(event) =>
                        setTaskDraft((current) =>
                          current
                            ? {
                                ...current,
                                scriptTextSource: "manual",
                                scriptText: event.target.value,
                                manualEditedFields: {
                                  ...current.manualEditedFields,
                                  scriptText: true,
                                },
                                durationHint: TASK_DRAFT_DURATION_HINT,
                              }
                            : current,
                        )
                      }
                    />
                  </div>
                  <div className="task-draft-actions">
                    <span>确认后，这段文本进入任务队列；下方分镜结果只消费已保存任务。</span>
                    <div className="task-draft-actions__buttons">
                      <button
                        type="button"
                        className="action-button action-button--light"
                        onClick={() =>
                          setTaskDraft((current) => (current ? restoreTaskDraftBaseScript(current) : current))
                        }
                      >
                        恢复原文
                      </button>
                      {taskDraft.mode === "create" ? (
                        <button
                          type="button"
                          className="action-button action-button--light"
                          onClick={() => handleConfirmTaskDraft(true)}
                        >
                          加入并继续
                        </button>
                      ) : null}
                      <button
                        type="button"
                        className="action-button action-button--dark"
                        onClick={() => handleConfirmTaskDraft(false)}
                      >
                        {taskDraft.mode === "create" ? "加入任务队列" : "保存到当前任务"}
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        ) : null}

        {isTaskPickerOpen ? (
          <div className="edit-dialog-backdrop" onClick={() => setIsTaskPickerOpen(false)}>
            <div className="task-picker-dialog" onClick={(event) => event.stopPropagation()}>
              <div className="edit-dialog__header">
                <div>
                  <strong>导入镜头任务</strong>
                  <span>从已新建的镜头任务队列中选择一个，作为本次 generate_storyboard 的输入。</span>
                </div>
                <button type="button" className="toolbar-button" onClick={() => setIsTaskPickerOpen(false)}>
                  关闭
                </button>
              </div>
              <div className="task-picker-list">
                {sceneTasks.map((task, index) => {
                  const taskStatus = getSceneTaskStatus(task, confirmedSceneTaskIds);
                  return (
                    <button
                      key={task.id}
                      type="button"
                      className={task.id === currentTaskId ? "task-picker-card task-picker-card--active" : "task-picker-card"}
                      onClick={() => handleImportSceneTask(task)}
                    >
                      <div>
                        <strong>{formatTaskQueueCardTitle(task, index, taskStatus.label)}</strong>
                        <span>
                          {task.segmentTitle} · 预计 {normalizeDurationOption(task.sourceDurationSeconds ?? task.selectedTotalDurationSeconds, durationSeconds)} 秒
                        </span>
                      </div>
                      <em>{task.status === "generated" ? `${task.rowCount ?? 0} 行已生成，可重新导入` : taskStatus.label}</em>
                      <p>{truncatePreview(task.scriptText, 140)}</p>
                    </button>
                  );
                })}
              </div>
            </div>
          </div>
        ) : null}

        {textDialog ? (
          <div className="edit-dialog-backdrop">
            <div className="text-dialog" onClick={(event) => event.stopPropagation()}>
              <div className="edit-dialog__header">
                <div>
                  <strong>{textDialog.title}</strong>
                  <span>{textDialog.helper}</span>
                </div>
                <div className="edit-dialog__header-actions">
                  {textDialog.kind === "synopsis" && textDialog.editable ? (
                    <button
                      type="button"
                      className="toolbar-button toolbar-button--compact"
                      onClick={handleClearTextDialog}
                    >
                      清空文字
                    </button>
                  ) : null}
                  <button type="button" className="toolbar-button" onClick={() => setTextDialog(null)}>
                    关闭
                  </button>
                </div>
              </div>
              <div className="text-dialog__body">
                {textDialog.kind === "synopsis" ? (
                  <label className="story-material-editor">
                    <span>正文</span>
                    <textarea
                      value={textDialog.value}
                      readOnly={!textDialog.editable}
                      placeholder="在这里输入正文"
                      onChange={(event) =>
                        setTextDialog((current) =>
                          current ? { ...current, value: event.target.value } : current,
                        )
                      }
                    />
                  </label>
                ) : textDialog.kind === "expandedScript" ? (
                  textDialogScriptBlocks.length ? (
                    <div className="script-dialog-blocks">
                      {textDialogScriptBlocks.map((block, index) => (
                        <label className="script-dialog-block" key={`${block.label}-${index}`}>
                          <span>{block.label}</span>
                          <textarea
                            value={block.body}
                            rows={estimateScriptBlockRows(block.body)}
                            readOnly={!textDialog.editable}
                            onChange={(event) =>
                              setTextDialog((current) => {
                                if (!current) {
                                  return current;
                                }
                                const blocks = parseEditableStoryDialogBlocks(current.value);
                                const nextBlocks = blocks.map((item, itemIndex) =>
                                  itemIndex === index ? { ...item, body: event.target.value } : item,
                                );
                                return { ...current, value: serializeEditableStoryDialogBlocks(nextBlocks) };
                              })
                            }
                          />
                        </label>
                      ))}
                    </div>
                  ) : (
                    <textarea
                      className="script-dialog-empty-textarea"
                      value=""
                      readOnly={!textDialog.editable}
                      placeholder="在这里输入正文"
                      onChange={(event) =>
                        setTextDialog((current) =>
                          current ? { ...current, value: event.target.value } : current,
                        )
                      }
                    />
                  )
                ) : (
                  <textarea
                    value={textDialog.value}
                    readOnly={!textDialog.editable}
                    onChange={(event) =>
                      setTextDialog((current) =>
                        current ? { ...current, value: event.target.value } : current,
                      )
                    }
                  />
                )}
                <div className="text-dialog__actions">
                  <span>Enter 可换行，正文区域可上下滚动。</span>
                  {textDialog.kind === "expandedScript" || textDialog.kind === "synopsis" ? (
                    <button
                      type="button"
                      className="action-button action-button--light"
                      onClick={handleConfirmTextDialogUse}
                      disabled={
                        bridgeBusy !== null ||
                        (textDialog.kind === "expandedScript"
                          ? !(synopsis.trim() || expandedScript.trim()) || expandedScriptAccepted
                          : !textDialog.value.trim())
                      }
                    >
                      {textDialog.kind === "expandedScript" && expandedScriptAccepted ? "已确认使用" : "确定使用"}
                    </button>
                  ) : null}
                  {textDialog.editable ? (
                    <button
                      type="button"
                      className="action-button action-button--dark"
                      onClick={handleSaveTextDialog}
                      disabled={bridgeBusy !== null}
                    >
                      {bridgeBusy === "save_rows" ? "保存中" : "保存文本"}
                    </button>
                  ) : null}
                </div>
              </div>
            </div>
          </div>
        ) : null}

        {editingRow ? (
          <div className="edit-dialog-backdrop" onClick={closeStoryboardEditDialog}>
            <div className="edit-dialog" onClick={(event) => event.stopPropagation()}>
              <div className="edit-dialog__header">
                <div>
                  <strong>修改分镜</strong>
                  <span>先调整草稿，点击“保存修改”后才会写回表格。</span>
                </div>
                <button type="button" className="toolbar-button" onClick={closeStoryboardEditDialog}>
                  取消
                </button>
              </div>
              <div className="edit-grid">
                <label>
                  <span>人物</span>
                  <input value={editingRow.person} onChange={(event) => handleEditField("person", event.target.value)} />
                </label>
                <label>
                  <span>镜头标题</span>
                  <input value={editingRow.shot} onChange={(event) => handleEditField("shot", event.target.value)} />
                </label>
                <label>
                  <span>景别</span>
                  <input value={editingRow.sceneScale} onChange={(event) => handleEditField("sceneScale", event.target.value)} />
                </label>
                <label className="edit-grid__wide edit-grid__compact">
                  <span>运镜</span>
                  <textarea value={editingRow.cameraMovement} onChange={(event) => handleEditField("cameraMovement", event.target.value)} />
                </label>
                <label className="edit-grid__wide edit-grid__priority">
                  <span>画面描述</span>
                  <textarea value={editingRow.visualDescription} onChange={(event) => handleEditField("visualDescription", event.target.value)} />
                </label>
                <label className="edit-grid__wide edit-grid__compact">
                  <span>角色动作</span>
                  <textarea value={editingRow.characterAction} onChange={(event) => handleEditField("characterAction", event.target.value)} />
                </label>
                <label className="edit-grid__wide edit-grid__compact">
                  <span>对白/旁白</span>
                  <textarea value={editingRow.dialogue} onChange={(event) => handleEditField("dialogue", event.target.value)} />
                </label>
                <label className="edit-grid__wide edit-grid__priority">
                  <span>分镜提示词</span>
                  <textarea value={editingRow.prompt} onChange={(event) => handleEditField("prompt", event.target.value)} />
                </label>
                <label>
                  <span>时长(秒)</span>
                  <input
                    type="number"
                    min={1}
                    value={editingRow.durationSeconds}
                    onChange={(event) => handleEditField("durationSeconds", Math.max(1, Number(event.target.value) || 1))}
                  />
                </label>
              </div>
              <div className="edit-dialog__footer">
                <span>保存后可继续导出，也可以点击开始生成重新生成当前镜头任务。</span>
                <div className="edit-dialog__actions">
                  <button type="button" className="action-button action-button--light" onClick={closeStoryboardEditDialog}>
                    取消
                  </button>
                  <button
                    type="button"
                    className="action-button action-button--dark"
                    onClick={handleSaveStoryboardEdit}
                    disabled={bridgeBusy !== null}
                  >
                    {bridgeBusy === "save_rows" ? "保存中" : "保存修改"}
                  </button>
                </div>
              </div>
            </div>
          </div>
        ) : null}
      </div>
    </Shell>
  );
}

function LongTextCell({
  label,
  value,
  onOpen,
  compact = false,
}: {
  label: string;
  value: string;
  onOpen: () => void;
  compact?: boolean;
}) {
  const text = value.trim() || "暂无内容";

  return (
    <button
      type="button"
      className={compact ? "table-text-preview table-text-preview--compact" : "table-text-preview"}
      onClick={onOpen}
      title={`${label}：${text}`}
    >
      <span>{text}</span>
      <em>展开</em>
    </button>
  );
}

function findSceneOption(value: SceneFusionOption | string | null | undefined) {
  return value ? SCENE_OPTIONS.find((option) => option.value === value) ?? null : null;
}

function resolveSceneOption(value: SceneFusionOption | string | null | undefined) {
  return findSceneOption(value) ?? SCENE_OPTIONS[0];
}

function groupSceneOptions(options: SceneOption[]) {
  const groups: Array<[string, SceneOption[]]> = [];
  for (const option of options) {
    const group = groups.find(([name]) => name === option.group);
    if (group) {
      group[1].push(option);
    } else {
      groups.push([option.group, [option]]);
    }
  }

  return groups;
}

function resolveModelLabel(value: WorkbenchModelId) {
  return MODEL_OPTIONS.find((option) => option.value === value)?.label ?? value;
}

function normalizeWorkbenchModelId(value: string): WorkbenchModelId {
  return value === "doubao" || value === "custom" ? value : "qwen";
}

function buildModelConfigSummary(
  config: ModelConfigState,
  status: TextModelProviderStatus,
): ModelConfigSummary {
  return {
    provider: config.provider,
    model: config.model.trim() || MODEL_DEFAULTS[config.provider].model,
    enabled: config.provider === "qwen" && status.enabled,
    base_url_present: config.baseUrl.trim().length > 0,
    api_key_present: status.api_key_present,
  };
}

function analyzeSourceInputForUi(text: string): SourceInputAnalysis {
  const cleanText = text.trim();
  const sourceInputType = detectSourceInputTypeForUi(cleanText);
  const sourceStoryFacts = buildSourceStoryFactsForUi(cleanText);
  const authoringMode = authoringModeForSourceInputType(sourceInputType);
  const sourceMaterialLengthChars = Array.from(cleanText).length;
  const paragraphs = cleanText.split(/\n+/).map((item) => item.trim()).filter(Boolean);
  const recommendsLongTextMode =
    Boolean(cleanText) &&
    (sourceInputType === "full_story" ||
      sourceInputType === "novel_chapter" ||
      sourceInputType === "mixed_material" ||
      sourceMaterialLengthChars >= 900 ||
      paragraphs.length >= 6);
  const sourceMaterialSummary = cleanText
    ? truncatePreview(cleanText, 260)
    : "等待输入或导入故事材料。";

  return {
    sourceInputType,
    authoringMode,
    sourceMaterialSummary,
    sourceStoryFacts,
    preservedFactSummary: preservedFactSummaryForUi(sourceStoryFacts),
    changedForScreenplaySummary: changedForScreenplaySummaryForUi(sourceInputType),
    omittedDetailSummary: omittedDetailSummaryForUi(sourceInputType, sourceStoryFacts),
    statusMessage: statusMessageForSourceInputType(cleanText ? sourceInputType : "synopsis", cleanText.length === 0),
    actionLabel: actionLabelForSourceInputType(sourceInputType),
    sourceMaterialLengthChars,
    recommendsLongTextMode,
  };
}

function buildSceneRewriteRequestAnalysis(
  analysis: SourceInputAnalysis,
  scene: SceneOption,
  targetDurationMode: TargetDurationMode,
  targetDurationSeconds: number,
): SourceInputAnalysis {
  const modeLabel = targetDurationMode === LONG_TEXT_DURATION_MODE
    ? "长文本模式"
    : `${targetDurationSeconds} 秒`;
  return {
    ...analysis,
    sourceInputType: "synopsis",
    authoringMode: "expand_from_synopsis",
    changedForScreenplaySummary:
      `将当前正文直接转写为“${scene.label}”场景语义，目标按${modeLabel}处理；保留人物与事件事实，但节奏、动作密度、关系表达和画面重心必须贴合当前场景类型。`,
    omittedDetailSummary:
      `如旧正文带有其他场景风格，只保留事实骨架，不保留旧场景的节奏和表现方式。当前场景：${scene.group} / ${scene.label}。`,
  };
}

function detectSourceInputTypeForUi(text: string): SourceInputType {
  const cleanText = text.trim();
  if (!cleanText) {
    return "synopsis";
  }

  const paragraphs = cleanText.split(/\n+/).map((item) => item.trim()).filter(Boolean);
  const sentences = splitScriptBody(cleanText);
  const charCount = Array.from(cleanText).length;
  const hasScreenplayMarks =
    /(^|\n)\s*(场景|镜头|对白|旁白|内景|外景|人物|动作)[：:]/.test(cleanText) ||
    /\b(INT\.|EXT\.|CUT TO|SCENE)\b/i.test(cleanText);
  const hasNovelMarks = /第[一二三四五六七八九十百千万0-9]+[章节回]/.test(cleanText) || /小说章节|本章|章节/.test(cleanText);
  const hasOutlineMarks = /(梗概|故事大纲|概要|一句话|主线)/.test(cleanText);
  const hasMixedMarks =
    hasScreenplayMarks &&
    (hasNovelMarks || /素材|参考|设定|世界观|人物小传|补充/.test(cleanText));

  if (hasMixedMarks) {
    return "mixed_material";
  }
  if (hasScreenplayMarks) {
    return "screenplay_text";
  }
  if (hasNovelMarks) {
    return "novel_chapter";
  }
  if (charCount > 900 || paragraphs.length >= 6 || sentences.length >= 10) {
    return "full_story";
  }
  if (charCount > 420 && /(随后|然后|最终|然而|直到|与此同时|结尾)/.test(cleanText)) {
    return "full_story";
  }
  if (charCount > 280 && !hasOutlineMarks && paragraphs.length >= 3) {
    return "full_story";
  }

  return "synopsis";
}

function normalizeSourceInputType(value: string | null | undefined): SourceInputType {
  if (
    value === "full_story" ||
    value === "novel_chapter" ||
    value === "screenplay_text" ||
    value === "mixed_material" ||
    value === "synopsis"
  ) {
    return value;
  }
  return "mixed_material";
}

function authoringModeForSourceInputType(sourceInputType: SourceInputType) {
  switch (sourceInputType) {
    case "full_story":
      return "rewrite_from_full_story";
    case "novel_chapter":
      return "adapt_story_to_screenplay";
    case "screenplay_text":
      return "polish_existing_screenplay";
    case "mixed_material":
      return "conservative_rewrite_from_mixed_material";
    case "synopsis":
    default:
      return "expand_from_synopsis";
  }
}

function actionLabelForSourceInputType(sourceInputType: SourceInputType) {
  switch (sourceInputType) {
    case "full_story":
    case "novel_chapter":
      return "改写剧本";
    case "screenplay_text":
      return "整理剧本";
    case "mixed_material":
      return "整理材料";
    case "synopsis":
    default:
      return "扩写剧本";
  }
}

function statusMessageForSourceInputType(sourceInputType: SourceInputType, empty = false) {
  if (empty) {
    return "等待输入或导入故事材料。";
  }

  switch (sourceInputType) {
    case "full_story":
      return "已识别为完整故事，将保留剧情事实并改写为剧本。";
    case "novel_chapter":
      return "已识别为小说章节，将保留人物、事件顺序和情绪推进并改写为剧本。";
    case "screenplay_text":
      return "已识别为已有剧本，将整理为可拍分镜剧本。";
    case "mixed_material":
      return "材料类型不够明确，将按保守方式整理为可拍剧本。";
    case "synopsis":
    default:
      return "已识别为故事梗概，将扩写后生成剧本。";
  }
}

function sourceInputTypeProductLabel(sourceInputType: SourceInputType) {
  switch (sourceInputType) {
    case "full_story":
      return "完整故事";
    case "novel_chapter":
      return "小说章节";
    case "screenplay_text":
      return "已有剧本";
    case "mixed_material":
      return "混合材料";
    case "synopsis":
    default:
      return "故事梗概";
  }
}

function isStoryPlanCurrentForUi({
  draftText,
  acceptedText,
  selectedSceneOption,
  acceptedScene,
  targetDurationMode,
  acceptedTargetDurationMode,
  durationSeconds,
  acceptedDurationSeconds,
}: {
  draftText: string;
  acceptedText: string;
  selectedSceneOption: SceneOption;
  acceptedScene: SceneOption | null;
  targetDurationMode: TargetDurationMode;
  acceptedTargetDurationMode: TargetDurationMode | null;
  durationSeconds: number;
  acceptedDurationSeconds: number | null;
}) {
  const cleanDraft = normalizeScriptText(draftText);
  const cleanAccepted = normalizeScriptText(acceptedText);
  if (!cleanDraft || !cleanAccepted || cleanDraft !== cleanAccepted) {
    return false;
  }
  if (!acceptedTargetDurationMode || acceptedTargetDurationMode !== targetDurationMode) {
    return false;
  }
  if (targetDurationMode === FIXED_DURATION_MODE && acceptedDurationSeconds !== durationSeconds) {
    return false;
  }
  return !acceptedScene || acceptedScene.value === selectedSceneOption.value;
}

function isExpandedScriptSnapshotCurrentForUi({
  response,
  responseScene,
  responseDurationSeconds,
  selectedSceneOption,
  targetDurationMode,
  durationSeconds,
}: {
  response: ExpandScriptResponse | null;
  responseScene: SceneOption | null;
  responseDurationSeconds: number | null;
  selectedSceneOption: SceneOption;
  targetDurationMode: TargetDurationMode;
  durationSeconds: number;
}) {
  if (!response || !responseScene || responseScene.value !== selectedSceneOption.value) {
    return false;
  }

  const responseMode = normalizeTargetDurationModeForUi(
    response.target_duration_mode ?? response.targetDurationMode ?? targetDurationMode,
  );
  if (responseMode !== targetDurationMode) {
    return false;
  }
  if (targetDurationMode === FIXED_DURATION_MODE) {
    return normalizeDurationOption(responseDurationSeconds, durationSeconds) === normalizeDurationOption(durationSeconds, durationSeconds);
  }
  return true;
}

function buildStoryBridgeView({
  draftAnalysis,
  acceptedAnalysis,
  targetDurationMode,
  durationSeconds,
  response,
  sceneTaskCount,
  hasAcceptedScript,
  isCurrent,
}: {
  draftAnalysis: SourceInputAnalysis;
  acceptedAnalysis: SourceInputAnalysis;
  targetDurationMode: TargetDurationMode;
  durationSeconds: number;
  response: ExpandScriptResponse | null;
  sceneTaskCount: number;
  hasAcceptedScript: boolean;
  isCurrent: boolean;
}) {
  const modeLabel = targetDurationMode === LONG_TEXT_DURATION_MODE
    ? "长文本模式"
    : `固定时长（${durationSeconds} 秒）`;
  const status = !hasAcceptedScript ? "待确定使用" : isCurrent ? "已确认" : "待重新确认";
  const planAnalysis = isCurrent ? acceptedAnalysis : draftAnalysis;
  const sourceLabel = sourceInputTypeProductLabel(planAnalysis.sourceInputType);
  const responsePlanSummary = isCurrent ? responseDurationPlanSummary(response) : "";
  const plannedTaskCount = isCurrent ? responseShotTaskCount(response) || sceneTaskCount : 0;
  const planHint = !isCurrent
    ? "点击“确定使用”后同步到镜头拆解"
    : plannedTaskCount > 0
    ? `${plannedTaskCount} 个镜头任务`
    : responsePlanSummary || "已同步到镜头拆解";
  const continuityHints = isCurrent && planAnalysis.sourceMaterialLengthChars > 0
    ? buildContinuityHintItems(planAnalysis.sourceStoryFacts)
    : [];
  const continuityHint = isCurrent && continuityHints.length
    ? continuityHints.join(" / ")
    : isCurrent
    ? "保留已确认正文"
    : "待确认后更新";
  const summaryItems = [
    `材料：${status} · ${sourceLabel}`,
    `模式：${modeLabel}${isCurrent ? "" : "（未同步）"}`,
    `计划：${planHint}`,
    `连续性：${continuityHint}`,
  ];
  const detailItems: string[] = [];

  if (!isCurrent) {
    // 承接行只保留产品态摘要；详情留给镜头任务队列承载。
  } else {
    if (targetDurationMode === LONG_TEXT_DURATION_MODE) {
      detailItems.push(
        `分段策略：${responsePlanSummary || "当前按剧情自动分段，单镜头分镜保持短镜头友好时长"}`,
      );
    } else {
      detailItems.push(`分段策略：${responsePlanSummary || `按 ${durationSeconds} 秒目标时长准备镜头任务`}`);
    }
    if (plannedTaskCount > 0) {
      detailItems.push(`预计拆分：${plannedTaskCount} 个镜头任务`);
    }
  }

  const previewStatus = !hasAcceptedScript || !isCurrent
    ? `${draftAnalysis.statusMessage} ${status}。`
    : compactSourceInputStatusText(draftAnalysis, targetDurationMode, durationSeconds, response);

  return {
    summaryItems,
    detailItems,
    previewStatus,
    state: isCurrent ? "confirmed" : hasAcceptedScript ? "stale" : "pending",
  };
}

function compactSourceInputStatusText(
  analysis: SourceInputAnalysis,
  targetDurationMode: TargetDurationMode,
  durationSeconds: number,
  response: ExpandScriptResponse | null,
) {
  if (analysis.sourceMaterialLengthChars <= 0) {
    return "请输入或导入故事材料。";
  }

  const hasCurrentPlan = isResponseDurationPlanCurrent(response, targetDurationMode);
  const shotTaskCount = hasCurrentPlan ? responseShotTaskCount(response) : 0;
  if (targetDurationMode === LONG_TEXT_DURATION_MODE) {
    return shotTaskCount > 1
      ? `长文本已自动分段：${shotTaskCount} 个镜头任务。`
      : "已识别为长文本，将按剧情自动分段。";
  }

  const fixedSuffix = `将按 ${durationSeconds} 秒目标时长处理。`;
  if (shotTaskCount > 1) {
    return `已按固定时长准备 ${shotTaskCount} 个镜头任务。`;
  }

  switch (analysis.sourceInputType) {
    case "full_story":
      return `已识别为完整故事，${fixedSuffix}`;
    case "novel_chapter":
      return `已识别为小说章节，${fixedSuffix}`;
    case "screenplay_text":
      return `已识别为已有剧本，${fixedSuffix}`;
    case "mixed_material":
      return `已识别为混合材料，${fixedSuffix}`;
    case "synopsis":
    default:
      return `已识别为故事梗概，${fixedSuffix}`;
  }
}

function buildContentBridgeItems(
  analysis: SourceInputAnalysis,
  targetDurationMode: TargetDurationMode,
  durationSeconds: number,
  response: ExpandScriptResponse | null,
  sceneTaskCount: number,
) {
  const items = [
    `当前输入：${sourceInputTypeProductLabel(analysis.sourceInputType)}`,
    `当前模式：${targetDurationMode === LONG_TEXT_DURATION_MODE ? "长文本模式" : `固定时长（${durationSeconds} 秒）`}`,
  ];
  const hasCurrentPlan = isResponseDurationPlanCurrent(response, targetDurationMode);
  const responseTaskCount = hasCurrentPlan ? responseShotTaskCount(response) : 0;
  const responsePlanSummary = hasCurrentPlan ? responseDurationPlanSummary(response) : "";
  const plannedTaskCount = responseTaskCount || sceneTaskCount;

  if (targetDurationMode === LONG_TEXT_DURATION_MODE) {
    items.push(`分段策略：${responsePlanSummary || "当前按剧情自动分段，单镜头分镜保持短镜头友好时长"}`);
  } else {
    items.push(`分段策略：${responsePlanSummary || `按 ${durationSeconds} 秒目标时长准备镜头任务`}`);
  }

  if (plannedTaskCount > 1) {
    items.push(`预计拆分为 ${plannedTaskCount} 个镜头任务`);
  }

  const continuityHints = analysis.sourceMaterialLengthChars > 0
    ? buildContinuityHintItems(analysis.sourceStoryFacts)
    : [];
  if (continuityHints.length) {
    items.push(`连续性保留：${continuityHints.join(" / ")}`);
  }

  return items;
}

function formatBridgeItemLabel(item: string) {
  const separatorIndex = item.indexOf("：");
  return separatorIndex >= 0 ? item.slice(0, separatorIndex + 1) : item;
}

function formatBridgeItemValue(item: string) {
  const separatorIndex = item.indexOf("：");
  return separatorIndex >= 0 ? item.slice(separatorIndex + 1) : "";
}

function isResponseDurationPlanCurrent(
  response: ExpandScriptResponse | null,
  targetDurationMode: TargetDurationMode,
) {
  if (!response) {
    return false;
  }
  const responseMode = normalizeTargetDurationModeForUi(
    response.target_duration_mode ?? response.targetDurationMode ?? targetDurationMode,
  );
  return responseMode === targetDurationMode;
}

function responseShotTaskCount(response: ExpandScriptResponse | null) {
  return Number(response?.generated_shot_task_count ?? response?.generatedShotTaskCount ?? 0);
}

function responseDurationPlanSummary(response: ExpandScriptResponse | null) {
  const summary = String(response?.duration_plan_summary ?? response?.durationPlanSummary ?? "").trim();
  if (!summary || /[_{}\[\]=]|hash|trace|manifest|ReadyStub|prompt_body|source_register/i.test(summary)) {
    return "";
  }
  return truncatePreview(summary.replace(/\s+/g, " "), 96).replace(/[。；;]\s*$/, "");
}

function buildContinuityHintItems(facts: SourceStoryFacts) {
  const hints: string[] = [];
  if ((facts.character_relationships?.length ?? facts.characterRelationships?.length ?? 0) > 0) {
    hints.push("保留人物关系");
  }
  if ((facts.event_order?.length ?? facts.eventOrder?.length ?? 0) > 0) {
    hints.push("保留事件顺序");
  }
  if ((facts.prop_state?.length ?? facts.propState?.length ?? 0) > 0) {
    hints.push("保留道具状态");
  }
  if ((facts.emotional_progression?.length ?? facts.emotionalProgression?.length ?? 0) > 0) {
    hints.push("保留情绪推进");
  }
  return hints.length ? hints : ["保留事件顺序"];
}

function buildSourceStoryFactsForUi(text: string): SourceStoryFacts {
  const cleanText = text.trim();
  const segments = splitScriptBody(cleanText);
  const compactSegments = segments.map((item) => truncatePreview(item, 120)).filter(Boolean);
  const coreEvents = compactSegments.slice(0, 8);
  const eventOrder = coreEvents.map((item, index) => `${index + 1}. ${item}`);
  const characterNames = extractCharacterNamesFromText(cleanText);
  const locationFacts = Array.from(
    new Set(
      (cleanText.match(/[在于到][^，。！？；;\n]{2,16}(?:城|镇|村|宫|殿|山|海|街|房|屋|厅|场|城墙|废墟|战场)/g) ?? [])
        .map((item) => item.replace(/^[在于到]/, "").trim())
        .filter(Boolean),
    ),
  ).slice(0, 6);

  return {
    character_names: characterNames,
    characterNames,
    character_relationships: [],
    characterRelationships: [],
    core_events: coreEvents,
    coreEvents,
    event_order: eventOrder,
    eventOrder,
    timeline_facts: coreEvents.slice(0, 4),
    timelineFacts: coreEvents.slice(0, 4),
    prop_state: [],
    propState: [],
    location_facts: locationFacts,
    locationFacts,
    emotional_progression: compactSegments.filter((item) => /(害怕|愤怒|犹豫|坚定|崩溃|觉醒|释然|紧张|绝望|希望)/.test(item)).slice(0, 4),
    emotionalProgression: compactSegments.filter((item) => /(害怕|愤怒|犹豫|坚定|崩溃|觉醒|释然|紧张|绝望|希望)/.test(item)).slice(0, 4),
    conflict_progression: compactSegments.filter((item) => /(冲突|战斗|追击|对抗|阻止|威胁|敌人|危机|争执)/.test(item)).slice(0, 4),
    conflictProgression: compactSegments.filter((item) => /(冲突|战斗|追击|对抗|阻止|威胁|敌人|危机|争执)/.test(item)).slice(0, 4),
    ending_state: compactSegments[compactSegments.length - 1] ?? "",
    endingState: compactSegments[compactSegments.length - 1] ?? "",
  };
}

function extractCharacterNamesFromText(text: string) {
  const matches = text.match(/[\u4e00-\u9fa5]{2,4}/g) ?? [];
  const stopWords = new Set([
    "当前",
    "镜头",
    "任务",
    "系统",
    "候选",
    "场景",
    "目标",
    "人物",
    "主角",
    "敌人",
    "动作",
    "画面",
    "身体",
    "衣袍",
    "石墙",
    "风穿",
    "竹叶",
    "青苔",
    "碎石",
    "刀光",
    "火星",
    "废墟",
    "地面",
    "空气",
    "左手",
    "右手",
    "双掌",
    "长剑",
    "软剑",
    "短刀",
  ]);
  const names: string[] = [];
  for (const match of matches) {
    if (stopWords.has(match) || /^(一个|一声|一道|这一|那个|没有|不是|只是|已经|突然|然后|继续|同时|之间|之中|之上|之下)$/.test(match)) {
      continue;
    }
    if (/(而|的|了|着|在|从|把|被|向|将|与|和|及|或|于|中|上|下|里|外)$/.test(match)) {
      continue;
    }
    if (!names.includes(match)) {
      names.push(match);
    }
    if (names.length >= 4) {
      break;
    }
  }
  return names;
}

function preservedFactSummaryForUi(facts: SourceStoryFacts) {
  const eventCount = facts.core_events?.length ?? facts.coreEvents?.length ?? 0;
  const locationCount = facts.location_facts?.length ?? facts.locationFacts?.length ?? 0;
  return `保留已识别的核心事件 ${eventCount} 条、空间信息 ${locationCount} 条和结尾状态。`;
}

function changedForScreenplaySummaryForUi(sourceInputType: SourceInputType) {
  switch (sourceInputType) {
    case "full_story":
      return "将完整故事压缩为可拍剧本场次，保留人物动机、事件顺序和结尾状态。";
    case "novel_chapter":
      return "将小说章节改写为可表演动作、对白意图和镜头调度。";
    case "screenplay_text":
      return "整理已有剧本结构，补齐可拆分镜的动作与调度表达。";
    case "mixed_material":
      return "按保守方式整理混合材料，不主动新增未经确认的主线剧情。";
    case "synopsis":
    default:
      return "将短梗概扩写为连续故事，再生成可拆分镜的剧本。";
  }
}

function omittedDetailSummaryForUi(sourceInputType: SourceInputType, facts: SourceStoryFacts) {
  if (sourceInputType === "synopsis") {
    return "短梗概阶段不主动省略已输入事实。";
  }
  const eventCount = facts.core_events?.length ?? facts.coreEvents?.length ?? 0;
  return eventCount > 6
    ? "次要描写会压缩，人物、事件顺序和情绪推进优先保留。"
    : "未主动省略已识别的核心事件。";
}

function formatImportedDocumentType(fileType: string) {
  const cleanType = fileType.trim().toLowerCase();
  return cleanType === "docx" ? "docx" : cleanType === "txt" ? "txt" : "故事";
}

function formatModelProviderStatus(
  config: ModelConfigState,
  status: TextModelProviderStatus,
) {
  if (status.status === "not_desktop") {
    return "非正式运行环境不可验收";
  }
  if (config.provider !== "qwen") {
    return "预留，当前未启用";
  }
  if (status.status === "fallback") {
    return "live call 失败进入 fallback";
  }
  if (status.status === "api_config_unsaved" || status.status === "unconfigured") {
    return "API 配置未保存";
  }
  if (status.status === "session_key_missing") {
    return "session-only key 已失效或缺失";
  }
  if (status.status === "provider_disabled" || status.status === "configured_disabled") {
    return "provider 未启用";
  }
  if (status.status === "base_url_missing") {
    return "base_url 缺失";
  }
  if (status.live_ready) {
    return "千问：已启用";
  }
  if (status.api_key_present || status.base_url_present) {
    return "千问：已配置未启用";
  }
  return "千问：未配置";
}

function formatModelProviderStatusTone(status: TextModelProviderStatus) {
  if (status.status === "enabled" && status.live_ready) {
    return "enabled";
  }
  if (status.status === "reserved" || status.status === "not_desktop") {
    return "reserved";
  }
  return "unconfigured";
}

function hasTextModelFallback(warnings: ProductWarning[]) {
  return warnings.some((warning) =>
    /^text_model_/.test(warning.code) &&
    (
      warning.code.includes("fallback") ||
      warning.code.includes("missing") ||
      warning.code.includes("closed") ||
      warning.code.includes("network") ||
      warning.code.includes("invalid") ||
      warning.code.includes("not_supported")
    ),
  );
}

function formatTextModelRunMessage(warnings: ProductWarning[]) {
  return hasTextModelFallback(warnings)
    ? "未启用千问或调用失败，已使用本地候选结果"
    : "千问文本生成已完成";
}

function normalizeApiKeyRef(value: string) {
  const cleanValue = value.trim();
  return API_KEY_REF_OPTIONS.some((option) => option.value === cleanValue) ? cleanValue : "";
}

function normalizeDurationOption(value: number | null | undefined, fallback = 15) {
  const fallbackDuration = DURATION_OPTIONS.includes(fallback) ? fallback : 15;
  const duration = Number(value);
  return DURATION_OPTIONS.includes(duration) ? duration : fallbackDuration;
}

function normalizeScriptDurationOption(value: number | null | undefined, fallback = 15) {
  const fallbackDuration = Number.isFinite(Number(fallback)) && Number(fallback) > 0 ? Number(fallback) : 15;
  const duration = Number(value);
  if (!Number.isFinite(duration) || duration <= 0) {
    return fallbackDuration;
  }
  return Math.min(360, Math.max(5, Math.round(duration / 5) * 5));
}

function normalizeTargetDurationModeForUi(value: unknown): TargetDurationMode {
  return value === LONG_TEXT_DURATION_MODE ? LONG_TEXT_DURATION_MODE : FIXED_DURATION_MODE;
}

function estimateLongTextAutoDurationSeconds(analysis: SourceInputAnalysis) {
  const sourceChars = analysis.sourceMaterialLengthChars;
  const lengthBased =
    sourceChars <= 220 ? 30 :
    sourceChars <= 800 ? 75 :
    sourceChars <= 2000 ? 90 :
    sourceChars <= 2500 ? 105 :
    sourceChars <= 3500 ? 120 :
    sourceChars <= 6000 ? 180 :
    180 + Math.min(6, Math.floor((sourceChars - 6000) / 1500)) * 30;
  const typeFloor =
    analysis.sourceInputType === "full_story" ? 120 :
    analysis.sourceInputType === "novel_chapter" || analysis.sourceInputType === "mixed_material" ? 90 :
    analysis.sourceInputType === "screenplay_text" ? 75 :
    30;
  const factEvents = Math.max(
    analysis.sourceStoryFacts.core_events?.length ?? 0,
    analysis.sourceStoryFacts.coreEvents?.length ?? 0,
    analysis.sourceStoryFacts.event_order?.length ?? 0,
    analysis.sourceStoryFacts.eventOrder?.length ?? 0,
  );
  const factBased = factEvents ? Math.min(18, factEvents) * 10 : 0;
  const total = Math.max(lengthBased, typeFloor, factBased);
  return normalizeScriptDurationOption(total === 60 ? 75 : total, 30);
}

function storyLengthProfileForUi(analysis: SourceInputAnalysis, durationSeconds: number) {
  const sourceChars = analysis.sourceMaterialLengthChars;
  const duration = normalizeScriptDurationOption(durationSeconds, 30);
  if (sourceChars >= 3500 || duration >= 180) {
    return "long_story_auto";
  }
  if (sourceChars >= 2500 || duration >= 120) {
    return "two_minute_story_2500_3500";
  }
  if (sourceChars >= 2000 || duration >= 105) {
    return "short_story_2000_2500";
  }
  if (duration >= 75) {
    return "long_story";
  }
  if (duration >= 45) {
    return "standard_clip";
  }
  return "short_clip";
}

function resolveResponseStoryDurationSeconds(response: ExpandScriptResponse, fallback: number) {
  return normalizeScriptDurationOption(
    response.estimated_total_story_duration_seconds ??
      response.estimatedTotalStoryDurationSeconds ??
      fallback,
    fallback,
  );
}

function formatDurationPlanProductMessage(response: ExpandScriptResponse | GenerateStoryboardResponse | null) {
  if (!response) {
    return "";
  }
  const mode = normalizeTargetDurationModeForUi(response.target_duration_mode ?? response.targetDurationMode);
  const estimatedSeconds = Number(
    response.estimated_total_story_duration_seconds ?? response.estimatedTotalStoryDurationSeconds ?? 0,
  );
  const shotTaskCount = Number(response.generated_shot_task_count ?? response.generatedShotTaskCount ?? 0);
  if (mode === LONG_TEXT_DURATION_MODE) {
    const taskText = shotTaskCount > 1 ? `预计拆分为 ${shotTaskCount} 个镜头任务。` : "";
    const durationText = estimatedSeconds > 0 ? `预计剧情总时长约 ${estimatedSeconds} 秒。` : "";
    return `已识别为长文本，将按剧情自动分段；${taskText}${durationText}单镜头分镜保持短镜头友好时长。`;
  }
  return estimatedSeconds > 0 ? `目标时长 ${estimatedSeconds} 秒。` : "";
}

function nearestDurationOption(value: number, fallback = 15) {
  const fallbackDuration = normalizeDurationOption(fallback, 15);
  const target = Number.isFinite(value) ? value : fallbackDuration;

  return DURATION_OPTIONS.reduce((best, option) => {
    const bestDelta = Math.abs(best - target);
    const optionDelta = Math.abs(option - target);
    return optionDelta < bestDelta || (optionDelta === bestDelta && option > best) ? option : best;
  }, fallbackDuration);
}

function estimateShotDuration(totalDurationSeconds: number, segmentCount: number) {
  const totalDuration = normalizeScriptDurationOption(totalDurationSeconds, 15);
  const count = Math.max(1, segmentCount);
  return nearestDurationOption(Math.max(5, totalDuration / count), 15);
}

function buildShotCandidates(
  expandedScript: string,
  totalDurationSeconds: number,
  targetDurationMode: TargetDurationMode = FIXED_DURATION_MODE,
): ShotCandidate[] {
  const scriptBody = extractScriptBody(expandedScript);
  if (!scriptBody) {
    return [];
  }

  return buildDynamicShotCandidatesFromBody(scriptBody, totalDurationSeconds, targetDurationMode);

  const sentences = splitScriptBody(scriptBody);
  const segments = mergeScriptSegments(sentences.length ? sentences : [scriptBody]);
  const fullDuration = normalizeScriptDurationOption(totalDurationSeconds, 15);
  if (targetDurationMode === LONG_TEXT_DURATION_MODE) {
    const plannedDurations = allocateLongTextAutoShotTaskDurations(fullDuration);
    const durations = plannedDurations.length > 1
      ? plannedDurations
      : Array.from({ length: Math.max(2, segments.length || 2) }, () => 10);
    return durations.map((durationSeconds, index) => {
      const segment = storySegmentForPlannedTask(segments, index, durations.length, scriptBody);
      return {
        id: `auto-segment-${index + 1}`,
        sceneIndex: index + 1,
        title: `自动分段 ${index + 1}`,
        text: segment,
        preview: truncatePreview(segment),
        durationSeconds,
      };
    });
  }

  const fixedFullDuration = normalizeDurationOption(totalDurationSeconds, 15);
  const candidates = segments.slice(0, 8).map((segment, index) => ({
    id: `candidate-${index + 1}`,
    sceneIndex: index + 1,
    title: `场景候选 ${index + 1}`,
    text: segment,
    preview: truncatePreview(segment),
    durationSeconds: fixedFullDuration,
  }));

  return candidates.length
    ? candidates
    : [
        {
          id: "custom",
          sceneIndex: 1,
          title: "自定义镜头片段",
          text: scriptBody,
          preview: truncatePreview(scriptBody),
          durationSeconds: fixedFullDuration,
        },
      ];
}

function buildDynamicShotCandidatesFromBody(
  scriptBody: string,
  totalDurationSeconds: number,
  targetDurationMode: TargetDurationMode,
): ShotCandidate[] {
  const cleanBody = normalizeScriptText(scriptBody);
  if (!cleanBody) {
    return [];
  }

  if (targetDurationMode === LONG_TEXT_DURATION_MODE) {
    const plannedDurations = allocateLongTextAutoShotTaskDurations(totalDurationSeconds);
    const targetCount = plannedDurations.length || Math.max(1, Math.min(8, Math.ceil(cleanBody.length / 320)));
    const segments = rebalanceCandidateSegments(buildCandidateSourceSegments(cleanBody), targetCount);
    const durations = allocateCandidateDurations(totalDurationSeconds, segments);
    return segments.map((segment, index) => ({
      id: `auto-segment-${index + 1}`,
      sceneIndex: index + 1,
      title: `自动分段 ${index + 1}`,
      text: segment,
      preview: truncatePreview(segment),
      durationSeconds: durations[index] ?? 10,
    }));
  }

  const fixedFullDuration = normalizeDurationOption(totalDurationSeconds, 15);
  const sourceSegments = buildCandidateSourceSegments(cleanBody);
  const sentenceCount = splitScriptBody(cleanBody).length || 1;
  const desiredCount = resolveDynamicCandidateCount(cleanBody, sentenceCount, sourceSegments.length, fixedFullDuration);
  const segments = rebalanceCandidateSegments(sourceSegments, desiredCount);
  const durations = allocateCandidateDurations(fixedFullDuration, segments);

  return segments.length
    ? segments.map((segment, index) => ({
        id: `candidate-${index + 1}`,
        sceneIndex: index + 1,
        title: `场景候选 ${index + 1}`,
        text: segment,
        preview: truncatePreview(segment),
        durationSeconds: durations[index] ?? fixedFullDuration,
      }))
    : [
        {
          id: "custom",
          sceneIndex: 1,
          title: "自定义镜头片段",
          text: cleanBody,
          preview: truncatePreview(cleanBody),
          durationSeconds: fixedFullDuration,
        },
      ];
}

function buildCandidateSourceSegments(scriptBody: string) {
  const paragraphSegments = scriptBody
    .split(/\n+/g)
    .map((item) => normalizeScriptText(item))
    .filter(Boolean);
  const source = paragraphSegments.length > 1 ? paragraphSegments : splitScriptBody(scriptBody);
  const sentences = source.length ? source : [scriptBody];
  const grouped: string[] = [];
  let buffer = "";

  sentences.forEach((sentence, index) => {
    const cleanSentence = normalizeScriptText(sentence);
    if (!cleanSentence) {
      return;
    }
    const shouldBreak =
      Boolean(buffer) &&
      (hasSceneTransitionCue(cleanSentence) ||
        hasSceneTransitionCue(sentences[index - 1] ?? "") ||
        buffer.length >= 180 ||
        (buffer.length >= 90 && cleanSentence.length >= 42));

    if (shouldBreak) {
      grouped.push(buffer);
      buffer = cleanSentence;
      return;
    }

    buffer = buffer ? `${buffer}${cleanSentence}` : cleanSentence;
  });

  if (buffer) {
    grouped.push(buffer);
  }

  return uniqueCandidateSegments(grouped.length ? grouped : [scriptBody]);
}

function hasSceneTransitionCue(text: string) {
  return /(随后|接着|与此同时|这时|突然|片刻后|转眼|清晨|黄昏|深夜|室内|室外|街道|巷|楼|桥|公交|残骸|废墟|城|广场|车厢|追逐|打斗|冲进|逃离|来到|离开|出现|转身|回头|最终|最后)/.test(text);
}

function uniqueCandidateSegments(segments: string[]) {
  const seen = new Set<string>();
  const unique: string[] = [];
  for (const segment of segments) {
    const cleanSegment = normalizeScriptText(segment);
    const key = cleanSegment.replace(/\s+/g, "");
    if (!cleanSegment || seen.has(key)) {
      continue;
    }
    seen.add(key);
    unique.push(cleanSegment);
  }
  return unique;
}

function resolveDynamicCandidateCount(
  scriptBody: string,
  sentenceCount: number,
  sourceSegmentCount: number,
  totalDurationSeconds: number,
) {
  const durationUnits = Math.max(1, Math.round(totalDurationSeconds / 5));
  const durationCap =
    totalDurationSeconds <= 15 ? 2 :
    totalDurationSeconds <= 30 ? 3 :
    totalDurationSeconds <= 60 ? 5 :
    8;
  const maxCount = Math.max(1, Math.min(durationUnits, durationCap, sentenceCount || 1));
  const lengthCount =
    scriptBody.length <= 180 ? 1 :
    scriptBody.length <= 420 ? 2 :
    scriptBody.length <= 800 ? 3 :
    Math.min(8, Math.ceil(scriptBody.length / 320));
  const sceneCount = Math.max(1, sourceSegmentCount);
  return Math.max(1, Math.min(maxCount, Math.max(lengthCount, sceneCount)));
}

function rebalanceCandidateSegments(sourceSegments: string[], desiredCount: number) {
  const segments = uniqueCandidateSegments(sourceSegments);
  const targetCount = Math.max(1, Math.min(desiredCount, segments.length));
  if (segments.length <= targetCount) {
    return segments;
  }

  const balanced: string[] = [];
  for (let index = 0; index < targetCount; index += 1) {
    const start = Math.floor((index * segments.length) / targetCount);
    const end = Math.max(start + 1, Math.ceil(((index + 1) * segments.length) / targetCount));
    balanced.push(segments.slice(start, end).join("").trim());
  }
  return uniqueCandidateSegments(balanced);
}

function allocateCandidateDurations(totalDurationSeconds: number, segments: string[]) {
  const count = Math.max(1, segments.length);
  const totalUnits = Math.max(count, Math.round(normalizeScriptDurationOption(totalDurationSeconds, 15) / 5));
  if (count === 1) {
    return [totalUnits * 5];
  }

  const weights = segments.map((segment) => Math.max(1, segment.length));
  const weightTotal = weights.reduce((sum, weight) => sum + weight, 0) || count;
  const rawUnits = weights.map((weight) => (weight / weightTotal) * totalUnits);
  const units = rawUnits.map((value) => Math.max(1, Math.floor(value)));
  let diff = totalUnits - units.reduce((sum, value) => sum + value, 0);
  const rankedIndexes = rawUnits
    .map((value, index) => ({ index, remainder: value - Math.floor(value) }))
    .sort((left, right) => right.remainder - left.remainder);

  while (diff > 0) {
    for (const item of rankedIndexes) {
      if (diff <= 0) {
        break;
      }
      units[item.index] += 1;
      diff -= 1;
    }
  }

  while (diff < 0) {
    const candidateIndex = units.findIndex((value) => value > 1);
    if (candidateIndex < 0) {
      break;
    }
    units[candidateIndex] -= 1;
    diff += 1;
  }

  return units.map((value) => value * 5);
}

function allocateLongTextAutoShotTaskDurations(totalDurationSeconds: number) {
  const total = normalizeScriptDurationOption(totalDurationSeconds, 30);
  const durations: number[] = [];
  let remaining = total;
  while (remaining >= 10) {
    durations.push(10);
    remaining -= 10;
  }
  if (remaining === 5) {
    durations.push(5);
  }
  return durations;
}

function storySegmentForPlannedTask(segments: string[], index: number, count: number, fallback: string) {
  if (!segments.length) {
    return fallback;
  }
  const start = Math.floor((index * segments.length) / count);
  const end = Math.max(start + 1, Math.ceil(((index + 1) * segments.length) / count));
  return segments.slice(start, end).join("").trim() || segments[index % segments.length] || fallback;
}

function createSceneTasksFromCandidates(
  candidates: ShotCandidate[],
  scriptId: string | null | undefined,
  scene: SceneOption,
  sourceDurationMode: TargetDurationMode,
): SceneTaskRecord[] {
  return candidates.map((candidate, index) => ({
    id: createTaskRecordId(),
    name: formatDefaultShotTaskName(candidate.sceneIndex || index + 1, 1),
    candidateId: candidate.id,
    sceneIndex: candidate.sceneIndex || index + 1,
    shotIndexWithinScene: 1,
    segmentTitle: candidate.title,
    scriptText: candidate.text,
    baseSceneText: candidate.text,
    scriptId,
    sourceSceneType: scene.value,
    sourceSceneLabel: scene.label,
    sourceSceneCategory: scene.group,
    sourceDurationSeconds: normalizeDurationOption(candidate.durationSeconds, 10),
    sourceDurationMode,
    status: "draft",
    dirty: false,
    hadRowEdits: false,
    scriptTextSource: "candidate",
    userEditedFields: createTaskDraftEditFlags(),
  }));
}

function formatDefaultShotTaskName(sceneIndex: number, shotIndexWithinScene: number) {
  return `第 ${Math.max(1, sceneIndex)} 个场景 - 第 ${Math.max(1, shotIndexWithinScene)} 个镜头`;
}

function createTaskDraftEditFlags(
  overrides: Partial<TaskDraftManualEditedFields> = {},
): TaskDraftManualEditedFields {
  return {
    name: false,
    duration: false,
    scriptText: false,
    ...overrides,
  };
}

function createTaskDraftFromCandidate(
  candidate: ShotCandidate,
  taskName: string,
  fallbackDurationSeconds: number,
  shotIndexWithinScene = 1,
): TaskDraftState {
  const durationSeconds = normalizeDurationOption(candidate.durationSeconds, fallbackDurationSeconds);
  return {
    mode: "create",
    sourceKind: "candidate",
    editingTaskRecordId: null,
    taskName,
    selectedCandidateId: candidate.id,
    sceneIndex: candidate.sceneIndex,
    shotIndexWithinScene,
    segmentTitle: candidate.title,
    scriptText: candidate.text,
    durationSeconds,
    baseSegmentTitle: candidate.title,
    baseScriptText: candidate.text,
    baseDurationSeconds: durationSeconds,
    scriptTextSource: "candidate",
    manualEditedFields: createTaskDraftEditFlags(),
    durationHint: TASK_DRAFT_DURATION_HINT,
  };
}

function updateTaskDraftDurationFromSystemCandidates(
  draft: TaskDraftState,
  durationSeconds: number,
  candidates: ShotCandidate[],
): TaskDraftState {
  void candidates;
  const nextDurationSeconds = normalizeDurationOption(durationSeconds, draft.durationSeconds);
  return {
    ...draft,
    durationSeconds: nextDurationSeconds,
    manualEditedFields: {
      ...draft.manualEditedFields,
      duration: true,
    },
    durationHint: TASK_DRAFT_DURATION_HINT,
  };
}

function restoreTaskDraftBaseScript(draft: TaskDraftState): TaskDraftState {
  const scriptTextSource: TaskDraftScriptTextSource = draft.sourceKind === "task" ? "task" : "candidate";
  return {
    ...draft,
    segmentTitle: draft.baseSegmentTitle,
    scriptText: draft.baseScriptText,
    scriptTextSource,
    manualEditedFields: {
      ...draft.manualEditedFields,
      scriptText: false,
    },
    durationHint: TASK_DRAFT_DURATION_HINT,
  };
}

function getSceneTaskStatus(task: SceneTaskRecord, confirmedTaskIds: Set<string>) {
  if (task.status === "generated" && !task.dirty && confirmedTaskIds.has(task.id)) {
    return { label: "已确定 / 已定稿", tone: "confirmed" as const };
  }
  if (task.status === "generated" && !task.dirty) {
    return { label: "待确认", tone: "pending" as const };
  }
  return { label: "待生成", tone: "pending" as const };
}

function formatTaskQueueCardTitle(task: SceneTaskRecord, index: number, statusText: string) {
  const sceneIndex = task.sceneIndex || index + 1;
  const shotIndex = task.shotIndexWithinScene || 1;
  return `第 ${sceneIndex} 个场景 - 第 ${shotIndex} 个镜头 - ${statusText}`;
}

function formatTaskQueueSelectOption(task: SceneTaskRecord, index: number, statusText: string) {
  const sceneIndex = task.sceneIndex || index + 1;
  const shotIndex = task.shotIndexWithinScene || 1;
  return `第 ${sceneIndex} 个场景 / 第 ${shotIndex} 个镜头 · ${task.name} · ${statusText}`;
}

function isTaskDraftDirty(draft: TaskDraftState) {
  return draft.manualEditedFields.name || draft.manualEditedFields.duration || draft.manualEditedFields.scriptText;
}

function extractScriptBody(source: string) {
  const text = source.trim();
  if (!text) {
    return "";
  }

  const expandedMatch = text.match(/expanded_script[:：]\s*([\s\S]*)$/i);
  if (expandedMatch?.[1]?.trim()) {
    return normalizeScriptText(expandedMatch[1]);
  }

  const metaKeys =
    "source_package|scene_type|scene_label|scene_category|target_duration_seconds|model_provider|model|model_enabled|api_key_present|text_generation|script_id|warnings|expanded_script";
  const synopsisMatch = text.match(
    new RegExp(`synopsis[:：]\\s*([\\s\\S]*?)(?=\\s+(?:${metaKeys})[:：]|$)`, "i"),
  );
  if (synopsisMatch?.[1]?.trim()) {
    return normalizeScriptText(synopsisMatch[1]);
  }

  const withoutMeta = text.replace(
    new RegExp(`\\b(?:${metaKeys})[:：]\\s*\\S+`, "gi"),
    " ",
  );
  return normalizeScriptText(withoutMeta);
}

function normalizeScriptText(text: string) {
  return text
    .replace(/\r\n/g, "\n")
    .replace(/[ \t]+/g, " ")
    .replace(/\n{2,}/g, "\n")
    .trim();
}

function formatScriptDialogText(text: string) {
  const cleanText = normalizeScriptText(text);
  if (!cleanText) {
    return "";
  }

  return cleanText
    .replace(/^(正文|扩写剧本|镜头脚本|视频分镜提示词|故事材料)[:：]?\s*/u, "$1：\n")
    .replace(/\s*(第\s*\d+\s*段[:：])/gu, "\n\n$1\n")
    .replace(/\s*(第\s*\d+\s*拍\s*\d+\s*-\s*\d+\s*秒[:：])/gu, "\n\n$1\n")
    .replace(/\s*(结尾[:：])/gu, "\n\n$1\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function stripStoryDialogInternalText(text: string) {
  const body = extractScriptBody(text) || text;
  return body
    .replace(
      /^\s*(target_duration_seconds|expanded_script_text|script_id|source_material|脚本改写|源材料识别|保留原则|状态保留|内部字段|结构说明).*$/gim,
      "",
    )
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function splitEditableStorySegments(text: string) {
  const cleanText = stripStoryDialogInternalText(text);
  if (!cleanText) {
    return [""];
  }

  const byBlankLine = cleanText
    .split(/\n\s*\n/g)
    .map((item) => item.trim())
    .filter(Boolean);
  if (byBlankLine.length > 1) {
    return byBlankLine;
  }

  const sentenceSegments = splitScriptBody(cleanText);
  if (sentenceSegments.length <= 2) {
    return [cleanText];
  }
  const chunkSize = Math.ceil(sentenceSegments.length / Math.min(4, sentenceSegments.length));
  const chunks: string[] = [];
  for (let index = 0; index < sentenceSegments.length; index += chunkSize) {
    chunks.push(sentenceSegments.slice(index, index + chunkSize).join("").trim());
  }
  return chunks.filter(Boolean);
}

function storyDialogLabelForIndex(index: number, total: number) {
  if (index === 0) {
    return "正文";
  }
  if (index === total - 1) {
    return "结尾";
  }
  return `第 ${index} 段`;
}

function formatStoryMaterialDialogText(text: string) {
  const segments = splitEditableStorySegments(text);
  return segments
    .map((segment, index) => `${storyDialogLabelForIndex(index, segments.length)}\n${segment}`)
    .join("\n\n")
    .trim();
}

function parseEditableStoryDialogBlocks(value: string): ScriptDialogBlock[] {
  const cleanValue = value.replace(/\r\n/g, "\n").trim();
  if (!cleanValue) {
    return [];
  }

  const headingPattern = /^(正文|第\s*\d+\s*段|结尾)\s*[：:]?\s*$/u;
  const blocks: ScriptDialogBlock[] = [];
  let current: ScriptDialogBlock | null = null;

  for (const rawLine of cleanValue.split("\n")) {
    const line = rawLine.trim();
    if (!line) {
      if (current?.body && !current.body.endsWith("\n")) {
        current.body += "\n";
      }
      continue;
    }

    if (headingPattern.test(line)) {
      if (current) {
        current.body = current.body.trim();
        blocks.push(current);
      }
      current = { label: line.replace(/\s+/g, " "), body: "", synthetic: false };
      continue;
    }

    if (!current) {
      current = { label: "正文", body: "", synthetic: true };
    }
    current.body = current.body ? `${current.body}\n${line}` : line;
  }

  if (current) {
    current.body = current.body.trim();
    blocks.push(current);
  }

  if (blocks.length) {
    return blocks;
  }

  return splitEditableStorySegments(cleanValue).map((segment, index, segments) => ({
    label: storyDialogLabelForIndex(index, segments.length),
    body: segment,
    synthetic: false,
  }));
}

function serializeEditableStoryDialogBlocks(blocks: ScriptDialogBlock[]) {
  return blocks
    .filter((block) => block.body.trim())
    .map((block) => `${block.label}\n${block.body.trim()}`)
    .join("\n\n");
}

function extractEditableStoryDialogText(value: string) {
  return parseEditableStoryDialogBlocks(value)
    .map((block) => block.body.trim())
    .filter(Boolean)
    .join("\n\n")
    .trim();
}

function parseScriptDialogBlocks(value: string): ScriptDialogBlock[] {
  const cleanValue = value.replace(/\r\n/g, "\n").trim();
  if (!cleanValue) {
    return [{ label: "正文", body: "", synthetic: true }];
  }

  const headingPattern =
    /^(正文|扩写剧本|故事材料|镜头脚本|视频分镜提示词|分镜提示词|第\s*\d+\s*段|第\s*\d+\s*拍\s*\d+\s*-\s*\d+\s*秒|结尾|镜头标题|景别|运镜|画面描述|角色动作|对白\/旁白|时长)\s*[：:]?\s*(.*)$/u;
  const blocks: ScriptDialogBlock[] = [];
  let current: ScriptDialogBlock | null = null;

  for (const rawLine of cleanValue.split("\n")) {
    const line = rawLine.trim();
    if (!line) {
      if (current?.body && !current.body.endsWith("\n")) {
        current.body += "\n";
      }
      continue;
    }

    const match = line.match(headingPattern);
    if (match) {
      if (current) {
        current.body = current.body.trim();
        blocks.push(current);
      }
      const label = match[1].replace(/\s+/g, " ");
      current = { label, body: match[2]?.trim() ?? "", synthetic: label === "正文" };
      continue;
    }

    if (!current) {
      current = { label: "正文", body: "", synthetic: true };
    }
    current.body = current.body ? `${current.body}\n${line}` : line;
  }

  if (current) {
    current.body = current.body.trim();
    blocks.push(current);
  }

  return blocks.length ? blocks : [{ label: "正文", body: cleanValue, synthetic: true }];
}

function serializeScriptDialogBlocks(blocks: ScriptDialogBlock[]) {
  return blocks
    .map((block) => {
      const body = block.body.trim();
      return block.synthetic ? body : `${block.label}：\n${body}`;
    })
    .filter(Boolean)
    .join("\n\n");
}

function estimateScriptBlockRows(text: string) {
  const cleanText = text.trim();
  if (!cleanText) {
    return 2;
  }
  const explicitLines = cleanText.split("\n").length;
  const wrappedLines = Math.ceil(Array.from(cleanText).length / 52);
  return Math.min(8, Math.max(2, explicitLines + wrappedLines - 1));
}

function splitScriptBody(text: string) {
  const flattened = text.replace(/\s*\n+\s*/g, " ").trim();
  const matches = flattened.match(/[^。！？!?；;]+[。！？!?；;]?/g) ?? [];
  return matches.map((item) => item.trim()).filter(Boolean);
}

function mergeScriptSegments(sentences: string[]) {
  const segments: string[] = [];
  let buffer = "";

  for (const sentence of sentences) {
    if (!buffer) {
      buffer = sentence;
      continue;
    }

    const next = `${buffer}${sentence}`;
    if (buffer.length < 90 || sentence.length < 26) {
      buffer = next;
      if (buffer.length >= 190) {
        segments.push(buffer);
        buffer = "";
      }
      continue;
    }

    segments.push(buffer);
    buffer = sentence;
  }

  if (buffer) {
    segments.push(buffer);
  }

  return segments.map((segment) => segment.trim()).filter(Boolean);
}

function truncatePreview(text: string, maxLength = 96) {
  const cleanText = text.replace(/\s+/g, " ").trim();
  return cleanText.length > maxLength ? `${cleanText.slice(0, maxLength)}...` : cleanText;
}

function formatSceneScaleLabel(value: string) {
  const cleanValue = value.trim();
  if (!cleanValue) {
    return "未标注";
  }

  return SCENE_SCALE_LABELS[cleanValue.toUpperCase()] ?? SCENE_SCALE_LABELS[cleanValue] ?? cleanValue;
}

function formatStoryboardDialogText(value: string) {
  const cleanValue = value.trim();
  if (!cleanValue) {
    return "（无）";
  }

  const labelPattern = STORYBOARD_DIALOG_FIELD_LABELS.map(escapeRegExp).join("|");
  const formatted = cleanValue.replace(
    new RegExp(`(?:^|[\\s；;，,。]+)(${labelPattern})[：:]\\s*`, "g"),
    (_match, label: string, offset: number) => `${offset === 0 ? "" : "\n\n"}${label}：\n`,
  );

  return formatted.replace(/\n{3,}/g, "\n\n").trim();
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function formatInternalPlaceholder(value: string) {
  const cleanValue = value.trim();
  const dictionary: Record<string, string> = {
    not_specified: "未指定角色",
    not_specified_by_v120_bridge: "未指定角色",
    visual_scene_core: "场景视觉核心",
    fused_scene_performance_core_preserved: "保持表演连续性",
  };

  if (dictionary[cleanValue]) {
    return dictionary[cleanValue];
  }

  return cleanValue
    .replace(/\bnot_specified_by_v120_bridge\b/g, "未指定角色")
    .replace(/\bnot_specified\b/g, "未指定角色")
    .replace(/\bvisual_scene_core\b/g, "场景视觉核心")
    .replace(/\bfused_scene_performance_core_preserved\b/g, "保持表演连续性");
}

function mapGeneratedStoryboardRow(
  row: GeneratedStoryboardRow,
  characterContext: string[] = [],
): StoryboardWorkbenchRow {
  const promptText = row.prompt_text?.trim();
  const primaryCharacter = characterContext[0] ?? "";
  const person = replaceGenericCharacterLabel(
    formatInternalPlaceholder(row.person || "not_specified"),
    primaryCharacter,
  );
  const visualDescription = replaceGenericCharacterLabel(row.visual_description, primaryCharacter);
  const characterAction = replaceGenericCharacterLabel(
    formatInternalPlaceholder(row.character_action),
    primaryCharacter,
  );
  const prompt = replaceGenericCharacterLabel(
    promptText || EMPTY_PROMPT_TEXT_PLACEHOLDER,
    primaryCharacter,
  );
  return {
    id: row.shot_id || createRowId(),
    order: row.order,
    shotScript: row.shot_script,
    primarySceneType: row.primary_scene_type,
    primarySceneLabel: row.primary_scene_label,
    primarySceneCategory: row.primary_scene_category,
    shotSceneType: row.shot_scene_type,
    shotSceneLabel: row.shot_scene_label,
    shotIntent: row.shot_intent,
    adaptationReason: row.adaptation_reason,
    groundingSource: row.grounding_source,
    person,
    shot: formatInternalPlaceholder(row.shot_title || row.shot_id),
    cameraMovement: formatCameraMovement(resolveCameraMovement(row)),
    sceneScale: row.scene_scale || "source",
    visualDescription,
    characterAction,
    dialogue: row.dialogue || "（无）",
    prompt,
    durationSeconds: row.duration_seconds,
    shotDurationSeconds: row.shot_duration_seconds || row.duration_seconds,
    durationSource: row.duration_source,
    backendRow: row,
  };
}

function replaceGenericCharacterLabel(value: string, characterName: string) {
  const cleanName = characterName.trim();
  if (!cleanName) {
    return value;
  }
  return value
    .replace(/目标人物/g, cleanName)
    .replace(/未指定角色/g, cleanName)
    .replace(/主要人物/g, cleanName)
    .replace(/主人公/g, cleanName)
    .replace(/主角/g, cleanName);
}

function cloneWorkbenchRows(rows: StoryboardWorkbenchRow[]) {
  return rows.map((row) => ({
    ...row,
    backendRow: row.backendRow ? { ...row.backendRow } : undefined,
  }));
}

function toGeneratedStoryboardRows(rows: StoryboardWorkbenchRow[]): GeneratedStoryboardRow[] {
  return renumberRows(rows).map((row) => {
    const backend = row.backendRow;
    const shotId = backend?.shot_id || row.id;
    const sequenceGrouping = backend?.sequence_grouping ?? {
      structure_mode: "SingleShot",
      sequence_id: null,
      shot_order: row.order,
      sequence_field_state: "Present",
    };
    const scenePerformanceProjection = backend?.scene_performance_projection ?? {
      source_sample_id: shotId,
      source_sample_title: row.shot,
      scene_scale: row.sceneScale,
      person: row.person,
      visual_description: row.visualDescription,
      character_action: row.characterAction,
      fused_source_text: row.shotScript ?? row.visualDescription,
      sequence_grouping: sequenceGrouping,
    };
    const backendPersonDisplay = formatInternalPlaceholder(backend?.person || "not_specified");
    const backendShotDisplay = formatInternalPlaceholder(backend?.shot_title || backend?.shot_id || "");
    const backendCameraMovement = backend ? resolveCameraMovement(backend) : "";
    const backendCameraMovementDisplay = formatCameraMovement(backendCameraMovement);
    const backendActionDisplay = formatInternalPlaceholder(backend?.character_action ?? "");
    const backendDialogueDisplay = backend?.dialogue || "（无）";
    const backendPromptDisplay = backend?.prompt_text?.trim() || EMPTY_PROMPT_TEXT_PLACEHOLDER;
    return {
      shot_id: shotId,
      order: row.order,
      shot_script: row.shotScript ?? backend?.shot_script ?? "",
      primary_scene_type: row.primarySceneType ?? backend?.primary_scene_type ?? "",
      primary_scene_label: row.primarySceneLabel ?? backend?.primary_scene_label ?? "",
      primary_scene_category: row.primarySceneCategory ?? backend?.primary_scene_category ?? "",
      shot_scene_type: row.shotSceneType ?? backend?.shot_scene_type ?? "",
      shot_scene_label: row.shotSceneLabel ?? backend?.shot_scene_label ?? "",
      shot_intent: row.shotIntent ?? backend?.shot_intent ?? "",
      adaptation_reason: row.adaptationReason ?? backend?.adaptation_reason ?? "",
      grounding_source: row.groundingSource ?? backend?.grounding_source ?? "expanded_script_text",
      person: backend
        ? preserveBackendValueWhenDisplayUnchanged(row.person, backend.person, backendPersonDisplay)
        : row.person,
      shot_title: backend
        ? preserveBackendValueWhenDisplayUnchanged(row.shot, backend.shot_title, backendShotDisplay)
        : row.shot,
      scene_scale: row.sceneScale,
      visual_description: row.visualDescription,
      character_action: backend
        ? preserveBackendValueWhenDisplayUnchanged(row.characterAction, backend.character_action, backendActionDisplay)
        : row.characterAction,
      camera_movement: backend
        ? preserveBackendValueWhenDisplayUnchanged(
            row.cameraMovement,
            backendCameraMovement,
            backendCameraMovementDisplay,
          )
        : row.cameraMovement === EMPTY_CAMERA_MOVEMENT_PLACEHOLDER ? "" : row.cameraMovement,
      dialogue: backend
        ? preserveBackendValueWhenDisplayUnchanged(row.dialogue, backend.dialogue, backendDialogueDisplay)
        : row.dialogue,
      prompt_text: backend
        ? preserveBackendValueWhenDisplayUnchanged(row.prompt, backend.prompt_text, backendPromptDisplay)
        : row.prompt === EMPTY_PROMPT_TEXT_PLACEHOLDER ? "" : row.prompt,
      prompt_text_compilation_status: backend?.prompt_text_compilation_status || "Blocked",
      prompt_text_compilation_warnings: backend?.prompt_text_compilation_warnings ?? [],
      prompt_text_source_row_id: backend?.prompt_text_source_row_id || shotId,
      duration_seconds: row.durationSeconds,
      shot_duration_seconds: row.shotDurationSeconds ?? backend?.shot_duration_seconds ?? row.durationSeconds,
      duration_source: row.durationSource ?? backend?.duration_source ?? STORYBOARD_DURATION_SOURCE,
      scene_performance_projection: scenePerformanceProjection,
      external_reference_handle_candidates: backend?.external_reference_handle_candidates ?? [],
      sequence_grouping: sequenceGrouping,
    };
  });
}

function preserveBackendValueWhenDisplayUnchanged(
  displayValue: string | undefined,
  backendValue: string | undefined,
  backendDisplayValue: string,
) {
  const nextValue = displayValue ?? "";
  return nextValue === backendDisplayValue ? backendValue ?? "" : nextValue;
}

function resolveCameraMovement(row: GeneratedStoryboardRow) {
  return String(row.camera_movement ?? row.cameraMovement ?? "").trim();
}

function formatCameraMovement(value: string | null | undefined) {
  const cleanValue = String(value ?? "").trim();
  return cleanValue ? formatInternalPlaceholder(cleanValue) : EMPTY_CAMERA_MOVEMENT_PLACEHOLDER;
}

function buildDefaultExcelFileName(kind: "storyboard_words" | "full_script") {
  const now = new Date();
  const pad = (value: number) => String(value).padStart(2, "0");
  const stamp = [
    now.getFullYear(),
    pad(now.getMonth() + 1),
    pad(now.getDate()),
    "_",
    pad(now.getHours()),
    pad(now.getMinutes()),
  ].join("");
  const label = kind === "storyboard_words" ? "分镜词" : "完整剧本";
  return `Hope_${label}_${stamp}.xlsx`;
}

function collectPromptStatuses(rows: GeneratedStoryboardRow[]) {
  return Array.from(
    new Set(rows.map((row) => row.prompt_text_compilation_status).filter(Boolean) as string[]),
  );
}

function collectPromptWarningCodes(rows: GeneratedStoryboardRow[]) {
  return Array.from(
    new Set(
      rows
        .flatMap((row) => row.prompt_text_compilation_warnings ?? [])
        .map((warning) => warning.code)
        .filter(Boolean),
    ),
  );
}

function isBlockedStoryboardResponse(response: GenerateStoryboardResponse) {
  return response.export_status.status === "Blocked" || !response.result_id || response.rows.length === 0;
}

function formatBlockedStoryboardResponse(response: GenerateStoryboardResponse) {
  const blockers = response.export_status.blockers.map((warning) => warning.message || warning.code);
  const fallback = blockers.length ? blockers.join("；") : "主线桥接阻断了本次操作，请检查故事梗概、场景类型、镜头剧本和时长。";
  return `本次操作未完成：${fallback}`;
}

function hasEditedRowsNotApplied(response: ExportBundleResponse) {
  return response.artifacts.some((artifact) => artifact.ready && artifact.edited_rows_applied === false);
}

function findPrimaryExcelArtifact(response: ExportBundleResponse | ExportStoryboardBankResponse) {
  return (
    response.artifacts.find(
      (artifact) =>
        artifact.ready &&
        Boolean(artifact.artifact_path) &&
        (artifact.export_format === "xlsx" ||
          artifact.artifact_kind === "excel_workbook" ||
          artifact.artifact_path?.toLowerCase().endsWith(".xlsx")),
    ) ??
    null
  );
}

function formatWarnings(warnings: ProductWarning[]) {
  if (!warnings.length) {
    return "无";
  }

  const codes = warnings.slice(0, 3).map((warning) => formatWarningCode(warning.code)).join(", ");
  return warnings.length > 3 ? `${codes} +${warnings.length - 3}` : codes;
}

function formatWarningCode(code: string) {
  if (code === "storyboard_bank_rows_hash_mismatch") {
    return STORYBOARD_ROWS_HASH_MISMATCH_MESSAGE;
  }
  if (code === "role_action_grounding_incomplete") {
    return "角色动作信息不完整，请重新生成或检查当前镜头脚本";
  }
  if (code === "source_input_type_uncertain") {
    return "材料类型不够明确，已按保守方式整理";
  }
  if (code === "full_story_rewrite_fact_loss_detected") {
    return "完整故事改写可能丢失部分事实，请复核剧情";
  }
  if (code === "rewrite_changed_character_motivation") {
    return "人物动机可能被改写，请复核角色动机";
  }
  if (code === "rewrite_changed_event_order") {
    return "事件顺序可能被改写，请复核剧情顺序";
  }
  if (code === "rewrite_dropped_key_event") {
    return "关键事件可能被省略，请复核源故事";
  }
  if (code === "rewrite_added_unapproved_plot") {
    return "可能新增了未确认剧情，请复核剧本";
  }
  if (code === "source_document_too_long_for_single_pass") {
    return "源文档较长，本轮优先保留人物、事件顺序和结尾状态";
  }
  if (code === "target_duration_mode_invalid") {
    return "目标时长模式无效，已按固定时长保守处理";
  }
  if (code === "long_text_auto_duration_plan_missing") {
    return "长文本自动分段计划未生成，请改用固定时长或缩短材料";
  }
  if (code === "long_text_auto_compressed_to_single_clip_blocked") {
    return "长文本不能压成单个短片，已阻止本次压缩";
  }
  if (code === "auto_segment_strategy_missing") {
    return "已使用系统默认剧情分段策略";
  }
  return code;
}

function hasWarningCode(warnings: ProductWarning[], code: string) {
  return warnings.some((warning) => warning.code === code || warning.message?.includes(code));
}

function formatDurationSource(source: string | null | undefined) {
  if (!source) {
    return "时长来源：当前镜头任务";
  }
  if (source === "storyboard_duration_plan.allocated_row_duration_seconds") {
    return "时长来源：系统分镜时长分配";
  }
  return "时长来源：系统分镜时长分配";
}

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function formatProductError(error: unknown) {
  const raw = formatError(error);
  if (/storyboard_result_not_found/i.test(raw)) {
    return "没有找到可用的分镜结果，请先生成后再保存或导出。";
  }
  if (/storyboard_bank_rows_hash_mismatch/i.test(raw)) {
    return STORYBOARD_ROWS_HASH_MISMATCH_MESSAGE;
  }
  if (/revision/i.test(raw)) {
    return "分镜内容版本已变化，请重新导入当前任务后再保存。";
  }
  if (/duration/i.test(raw)) {
    return "分镜时长不符合当前任务要求，请检查每行时长后再保存。";
  }
  if (/scene_type/i.test(raw)) {
    return "当前场景类型暂未被主线契约接受，请换一个场景类型后重试。";
  }
  if (/script|synopsis/i.test(raw)) {
    return "镜头剧本为空或不可用，请先完成扩写并创建镜头任务。";
  }
  return raw || "操作未完成，请稍后重试。";
}

function formatGenerateStoryboardMessage(response: GenerateStoryboardResponse, rowCount: number) {
  const blockers = response.export_status.blockers.length;
  const warnings = response.export_status.warnings.length;
  const limitationText = response.export_status.status === "WarningOnly"
    ? `，存在候选限制/证据限制：${warnings} 条提示`
    : blockers
    ? `，仍有 ${blockers} 个阻断项`
    : "";

  return `分镜已生成：${rowCount} 条${limitationText}。可继续新建或导入下一个镜头任务。`;
}

function buildPageTokens(pageCount: number, currentPage: number) {
  if (pageCount <= 6) {
    return Array.from({ length: pageCount }, (_, index) => index + 1);
  }

  const tokens: Array<number | "..."> = [1];
  const start = Math.max(2, currentPage - 1);
  const end = Math.min(pageCount - 1, currentPage + 1);

  if (start > 2) {
    tokens.push("...");
  }

  for (let page = start; page <= end; page += 1) {
    tokens.push(page);
  }

  if (end < pageCount - 1) {
    tokens.push("...");
  }

  tokens.push(pageCount);
  return tokens;
}

function clampPage(page: number, pageCount: number) {
  return Math.min(Math.max(1, page), pageCount);
}

function resolveStoryboardPageSize() {
  if (typeof window === "undefined") {
    return DEFAULT_STORYBOARD_PAGE_SIZE;
  }

  const { innerHeight, innerWidth } = window;
  if (innerHeight < 760 || innerWidth < 900) {
    return 3;
  }
  if (innerHeight < 900) {
    return 4;
  }
  if (innerHeight >= 1040 && innerWidth >= 1600) {
    return 6;
  }
  return DEFAULT_STORYBOARD_PAGE_SIZE;
}

function renumberRows(rows: StoryboardWorkbenchRow[]) {
  return rows.map((row, index) => ({
    ...row,
    order: index + 1,
  }));
}

function upsertFinalizedShot(
  shots: FinalizedStoryboardShotResult[],
  nextShot: FinalizedStoryboardShotResult,
) {
  const existingIndex = shots.findIndex((shot) => shot.result_id === nextShot.result_id);
  if (existingIndex === -1) {
    return [...shots, nextShot].sort((left, right) => left.shot_order - right.shot_order);
  }

  const next = [...shots];
  next[existingIndex] = nextShot;
  return next.sort((left, right) => left.shot_order - right.shot_order);
}

function createRowId() {
  return `row-${Math.random().toString(36).slice(2, 10)}`;
}

function createTaskRecordId() {
  return `scene-task-${Math.random().toString(36).slice(2, 10)}`;
}
