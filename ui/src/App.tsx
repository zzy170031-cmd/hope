import { useEffect, useMemo, useState } from "react";
import { Shell } from "./components/Shell";
import logoUrl from "./assets/hope-desktop-logo.png";
import {
  configureTextModelProvider,
  expandScript as invokeExpandScript,
  exportBundle as invokeExportBundle,
  generateStoryboard as invokeGenerateStoryboard,
  updateStoryboardRows as invokeUpdateStoryboardRows,
} from "./bridge/hopeBridge";
import { ROUTES, resolveRoute } from "./routes";
import type {
  ExpandScriptResponse,
  ExportArtifactRecord,
  ExportBundleResponse,
  GenerateStoryboardResponse,
  GeneratedStoryboardRow,
  KbRouterRuntimeResponse,
  ModelConfigSummary,
  ProductWarning,
  SceneFusionOption,
  StoryboardExportStatus,
  StoryboardWorkbenchRow,
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

type LongTextField = "visualDescription" | "characterAction" | "dialogue" | "prompt";

interface TextDialogState {
  kind: "synopsis" | "expandedScript" | "storyboardCell";
  title: string;
  value: string;
  helper: string;
  editable: boolean;
  rowId?: string;
  field?: LongTextField;
}

interface ShotCandidate {
  id: string;
  title: string;
  text: string;
  preview: string;
  durationSeconds: number;
}

interface TaskDraftState {
  mode: "create" | "update";
  taskName: string;
  selectedCandidateId: string;
  scriptText: string;
  durationSeconds: number;
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
  segmentTitle: string;
  scriptText: string;
  scriptId?: string | null;
  sourceSceneType?: SceneFusionOption | string | null;
  sourceSceneLabel?: string | null;
  sourceSceneCategory?: string | null;
  sourceDurationSeconds?: number | null;
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
  exportContentHash?: string | null;
}

const PAGE_SIZE = 5;
const DURATION_OPTIONS = [5, 10, 15, 30, 45, 60];
const DEFAULT_SYNOPSIS = "主角在废墟城市中与敌人激烈战斗，最终觉醒新力量，击败敌人。";
const DEFAULT_SCENE: SceneFusionOption = "hot_blood_battle";
const DEFAULT_TASK_NAME = "第一集分镜生成";

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
  enabled: false,
  base_url_present: true,
  api_key_present: false,
  live_ready: false,
  status: "unconfigured",
  message: "千问：未配置，请在 API 接口中输入 API Key 并保存。本次会话有效。",
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
      "expand_script：读取场景类型、目标时长、故事梗概和 model_config_summary，只返回连续剧情剧本正文。",
      "generate_storyboard：读取镜头剧本、script_id、结构化场景字段、目标时长和模型摘要，返回主线 rows 与 export_status。",
      "export_bundle：导出分镜词或完整剧本，只有主线返回 artifact_path / content_hash / row_count 时才显示导出成功。",
    ],
  },
  {
    title: "模型配置边界",
    items: [
      "千问 Qwen 是当前唯一可启用的文本模型；豆包 Doubao 和自定义模型为预留入口。",
      "千问未配置、未启用或调用失败时会自动回退到本地候选结果；Seedance2.0 仅作为提示词适配目标。",
      "API Key 只在桌面会话内保存，业务 payload、日志和导出 artifact 只记录 api_key_present，不显示明文。",
    ],
  },
  {
    title: "导出与状态",
    items: [
      "WarningOnly 表示已生成但存在候选限制或证据限制，不等同失败。",
      "Excel workbook 未 ready 时不会伪造 Excel ready、下载路径或素材路径。",
      "候选提示词证据不会被声明为最终 prompt_text。",
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
  const [acceptedScriptScene, setAcceptedScriptScene] = useState<SceneOption | null>(null);
  const [taskName, setTaskName] = useState(DEFAULT_TASK_NAME);
  const [taskSourceScript, setTaskSourceScript] = useState("");
  const [taskScriptId, setTaskScriptId] = useState<string | null>(null);
  const [taskSegmentTitle, setTaskSegmentTitle] = useState("");
  const [durationSeconds, setDurationSeconds] = useState(15);
  const [rows, setRows] = useState<StoryboardWorkbenchRow[]>([]);
  const [rowsDirty, setRowsDirty] = useState(false);
  const [storyboardResult, setStoryboardResult] = useState<GenerateStoryboardResponse | null>(null);
  const [lastExportResult, setLastExportResult] = useState<ExportBundleResponse | null>(null);
  const [generatedSceneTasks, setGeneratedSceneTasks] = useState<GeneratedSceneTask[]>([]);
  const [sceneTasks, setSceneTasks] = useState<SceneTaskRecord[]>([]);
  const [currentTaskId, setCurrentTaskId] = useState<string | null>(null);
  const [bridgeBusy, setBridgeBusy] = useState<
    "expand" | "generate" | "save_rows" | "export_words" | "export_script" | null
  >(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [jumpPage, setJumpPage] = useState("1");
  const [editingRowId, setEditingRowId] = useState<string | null>(null);
  const [editingDraft, setEditingDraft] = useState<StoryboardWorkbenchRow | null>(null);
  const [textDialog, setTextDialog] = useState<TextDialogState | null>(null);
  const [taskDraft, setTaskDraft] = useState<TaskDraftState | null>(null);
  const [isTaskPickerOpen, setIsTaskPickerOpen] = useState(false);
  const [taskSerial, setTaskSerial] = useState(0);
  const [isKbSummaryExpanded, setIsKbSummaryExpanded] = useState(false);
  const [exportMessage, setExportMessage] = useState("等待主线 bridge 返回结果。");

  const editingRow = editingDraft;

  const pageCount = Math.max(1, Math.ceil(rows.length / PAGE_SIZE));
  const pagedRows = useMemo(() => {
    const start = (currentPage - 1) * PAGE_SIZE;
    return rows.slice(start, start + PAGE_SIZE);
  }, [currentPage, rows]);

  useEffect(() => {
    setCurrentPage((value) => Math.min(value, pageCount));
  }, [pageCount]);

  useEffect(() => {
    setJumpPage(String(currentPage));
  }, [currentPage]);

  const currentSceneTask = useMemo(
    () => sceneTasks.find((task) => task.id === currentTaskId) ?? null,
    [currentTaskId, sceneTasks],
  );
  const selectedSceneOption = useMemo(() => resolveSceneOption(selectedScene), [selectedScene]);
  const canImportScript = acceptedScript.trim().length > 0;
  const hasTaskScript = taskSourceScript.trim().length > 0;
  const canGenerate = taskName.trim().length > 0 && hasTaskScript;
  const expandedScriptAccepted = Boolean(
    expandedScriptResult?.script_id && acceptedScriptId === expandedScriptResult.script_id,
  );
  const shotCandidateBaseDuration = normalizeDurationOption(acceptedScriptDurationSeconds ?? durationSeconds, durationSeconds);
  const currentTaskDuration = normalizeDurationOption(
    currentSceneTask?.sourceDurationSeconds ?? durationSeconds,
    durationSeconds,
  );
  const currentTaskSceneLabel = currentSceneTask?.sourceSceneLabel ?? acceptedScriptScene?.label ?? selectedSceneOption.label;
  const taskSourceLabel = hasTaskScript
    ? `${taskSegmentTitle || "自定义镜头片段"} · ${
        taskScriptId ? `来源 ${taskScriptId}` : "来源扩写剧本正文"
      }`
    : sceneTasks.length
      ? `已新建 ${sceneTasks.length} 个镜头任务，请在分镜产出区导入镜头任务。`
    : canImportScript
    ? `已确认扩写剧本${acceptedScriptId ? ` ${acceptedScriptId}` : ""}，可拆解镜头。`
    : expandedScript.trim()
    ? "扩写剧本已生成，请先点击“确定使用”再拆解镜头。"
    : "先在剧本区完成扩写，再拆解镜头。";
  const shotCandidates = useMemo(
    () => buildShotCandidates(acceptedScript, shotCandidateBaseDuration),
    [acceptedScript, shotCandidateBaseDuration],
  );
  const usedCandidateIds = useMemo(
    () => new Set(sceneTasks.filter((task) => task.candidateId !== "custom").map((task) => task.candidateId)),
    [sceneTasks],
  );
  const generatedSceneTaskCount = useMemo(
    () => sceneTasks.filter((task) => task.status === "generated").length,
    [sceneTasks],
  );
  const pageTokens = useMemo(() => buildPageTokens(pageCount, currentPage), [pageCount, currentPage]);
  const sceneOptionGroups = useMemo(() => groupSceneOptions(SCENE_OPTIONS), []);
  const selectedModelLabel = useMemo(() => resolveModelLabel(modelConfig.provider), [modelConfig.provider]);
  const modelReservedWarning = modelConfig.provider === "qwen"
    ? ""
    : "该模型接口已配置为预留状态，当前仍使用本地文本生成桥接。";
  const modelConfigSummary = useMemo(
    () => buildModelConfigSummary(modelConfig, modelProviderStatus),
    [modelConfig, modelProviderStatus],
  );
  const modelRuntimeLabel = useMemo(
    () => formatModelProviderStatus(modelConfig, modelProviderStatus),
    [modelConfig, modelProviderStatus],
  );
  const activeKbRouterResult = storyboardResult?.kb_router_result ?? expandedScriptResult?.kb_router_result ?? null;
  const shotGroundingSummary = useMemo(() => buildShotGroundingSummary(rows), [rows]);

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

  const restoreSceneTaskState = (task: SceneTaskRecord) => {
    const restoredDuration = normalizeDurationOption(
      task.sourceDurationSeconds ?? task.selectedTotalDurationSeconds,
      durationSeconds,
    );
    setCurrentTaskId(task.id);
    setTaskName(task.name);
    setTaskSourceScript(task.scriptText);
    setTaskScriptId(task.scriptId ?? acceptedScriptId ?? null);
    setTaskSegmentTitle(task.segmentTitle);
    const restoredScene = findSceneOption(task.sourceSceneType ?? null);
    if (restoredScene) {
      setSelectedScene(restoredScene.value);
    }
    setDurationSeconds(restoredDuration);
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

    const savedRows = response.rows.map(mapGeneratedStoryboardRow);
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
      status: provider === "qwen" ? "unconfigured" : "reserved",
      message:
        provider === "qwen"
          ? "千问：未配置，请在 API 接口中保存本次会话 Key。"
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
      const nextConfig: ModelConfigState = {
        provider,
        model: status.model || requestedModel,
        baseUrl: requestedBaseUrl,
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
      title: expandedScriptAccepted ? "编辑扩写剧本" : "编辑故事梗概",
      value: synopsis,
      helper: expandedScriptAccepted
        ? "这里编辑当前已确认的扩写剧本；保存后会同步给镜头拆解，不会撑开主界面。"
        : "适合输入更长的故事梗概；保存后同步回剧本区，页面布局不会被撑开。",
      editable: true,
    });
  };

  const openExpandedScriptDialog = () => {
    setTextDialog({
      kind: "expandedScript",
      title: "查看扩写剧本",
      value: expandedScript || "等待 expanded_script_text",
      helper: "这里展示主线 bridge 返回的 expanded_script_text；确认使用后才会进入镜头拆解。",
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
      title: `${row.order} · ${label}`,
      value: String(row[field] ?? ""),
      helper: "表格中只显示短预览；这里可以放大查看并编辑完整文本。",
      editable: true,
      rowId: row.id,
      field,
    });
  };

  const openStoryboardEditDialog = (row: StoryboardWorkbenchRow) => {
    setEditingRowId(row.id);
    setEditingDraft({ ...row });
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
      setSynopsis(textDialog.value);
      if (expandedScriptAccepted) {
        setExpandedScript(textDialog.value);
        setAcceptedScript(textDialog.value);
      }
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

  const handleExpandScript = async () => {
    if (bridgeBusy) {
      return;
    }
    if (!confirmDiscardDirty("重新扩写剧本")) {
      return;
    }

    const storyInput = synopsis.trim();
    if (!storyInput) {
      setExportMessage("请先输入故事梗概，再扩写剧本。");
      return;
    }

    setBridgeBusy("expand");
    try {
      const response = await invokeExpandScript({
        scene_type: selectedSceneOption.value,
        scene_label: selectedSceneOption.label,
        scene_category: selectedSceneOption.group,
        model_config_summary: modelConfigSummary,
        selected_total_duration_seconds: durationSeconds,
        synopsis_text: storyInput,
      });
      const scriptBody = extractScriptBody(response.expanded_script_text) || response.expanded_script_text.trim();
      setExpandedScriptResult(response);
      setExpandedScript(scriptBody);
      setSynopsis(scriptBody);
      setExpandedScriptDurationSeconds(durationSeconds);
      setExpandedScriptScene(selectedSceneOption);
      setShowExpandedScriptStatus(false);
      setAcceptedScript("");
      setAcceptedScriptId(null);
      setAcceptedScriptDurationSeconds(null);
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
      setTaskSerial(0);
      setRows([]);
      setRowsDirty(false);
      setCurrentPage(1);
      syncModelProviderStatusFromWarnings(response.warnings);
      setExportMessage(
        `扩写剧本完成：${response.script_id}；目标时长 ${durationSeconds} 秒。剧本正文已进入剧本区，请确认使用后进入镜头拆解。${formatTextModelRunMessage(response.warnings)}`,
      );
    } catch (error) {
      setExportMessage(`扩写剧本失败：${formatError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
  };

  const openTaskDraftDialog = (mode: TaskDraftState["mode"]) => {
    if (bridgeBusy) {
      return;
    }
    if (!canImportScript) {
      setExportMessage("请先扩写剧本并点击“确定使用”，再新建镜头任务。");
      return;
    }

    if (!confirmDiscardDirty(mode === "create" ? "新建镜头任务" : "更新当前镜头")) {
      return;
    }

    const nextSerial = Math.max(taskSerial + 1, sceneTasks.length + 1);
    const currentName = taskName.trim();
    const defaultTaskName = mode === "create"
      ? currentName && currentName !== DEFAULT_TASK_NAME && !hasTaskScript
        ? currentName
        : `第 ${nextSerial} 个镜头任务`
      : currentName || `第 ${Math.max(nextSerial, 1)} 个镜头任务`;
    const nextUnusedCandidate =
      mode === "create"
        ? shotCandidates.find((candidate) => !usedCandidateIds.has(candidate.id) && candidate.id !== "full-script") ??
          shotCandidates.find((candidate) => !usedCandidateIds.has(candidate.id))
        : shotCandidates.find((candidate) => candidate.id === currentSceneTask?.candidateId);
    const firstCandidate = nextUnusedCandidate ?? shotCandidates[0] ?? {
      id: "custom",
      title: "自定义镜头片段",
      text: acceptedScript.trim(),
      preview: acceptedScript.trim(),
      durationSeconds: shotCandidateBaseDuration,
    };
    const existingTaskScript = mode === "update" && taskSourceScript.trim()
      ? taskSourceScript.trim()
      : "";
    const draftDuration = existingTaskScript
      ? normalizeDurationOption(
          currentSceneTask?.sourceDurationSeconds ?? currentSceneTask?.selectedTotalDurationSeconds ?? firstCandidate.durationSeconds,
          firstCandidate.durationSeconds,
        )
      : firstCandidate.durationSeconds;

    setTaskDraft({
      mode,
      taskName: defaultTaskName,
      selectedCandidateId: existingTaskScript ? currentSceneTask?.candidateId ?? "custom" : firstCandidate.id,
      scriptText: existingTaskScript || firstCandidate.text,
      durationSeconds: draftDuration,
    });
  };

  const handleTaskCandidateSelect = (candidateId: string) => {
    const candidate = shotCandidates.find((item) => item.id === candidateId);
    if (!candidate) {
      return;
    }

    setTaskDraft((current) =>
      current
        ? {
            ...current,
            selectedCandidateId: candidate.id,
            scriptText: candidate.text,
            durationSeconds: candidate.durationSeconds,
          }
        : current,
    );
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
    const nextTaskName = taskDraft.taskName.trim() || `第 ${taskSerial + 1} 个镜头任务`;
    const nextTaskId = taskDraft.mode === "create" || !currentTaskId ? createTaskRecordId() : currentTaskId;
    const nextSegmentTitle = candidate?.title ?? "自定义镜头片段";
    const taskScene = acceptedScriptScene ?? selectedSceneOption;
    const taskDurationSeconds = normalizeDurationOption(taskDraft.durationSeconds, shotCandidateBaseDuration);

    if (taskDraft.mode === "create") {
      setTaskSerial((value) => value + 1);
    }

    setTaskName(nextTaskName);
    setTaskSourceScript(selectedText);
    setTaskScriptId(acceptedScriptId);
    setTaskSegmentTitle(nextSegmentTitle);
    setCurrentTaskId(nextTaskId);
    setDurationSeconds(taskDurationSeconds);
    setSceneTasks((current) => {
      const nextRecord: SceneTaskRecord = {
        id: nextTaskId,
        name: nextTaskName,
        candidateId: taskDraft.selectedCandidateId,
        segmentTitle: nextSegmentTitle,
        scriptText: selectedText,
        scriptId: acceptedScriptId,
        sourceSceneType: taskScene.value,
        sourceSceneLabel: taskScene.label,
        sourceSceneCategory: taskScene.group,
        sourceDurationSeconds: taskDurationSeconds,
        status: "draft",
        dirty: false,
        hadRowEdits: false,
      };

      if (taskDraft.mode === "create" || !current.some((task) => task.id === nextTaskId)) {
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
              exportContentHash: undefined,
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

    if (continueCreating && taskDraft.mode === "create") {
      const nextUsedCandidateIds = new Set(usedCandidateIds);
      if (taskDraft.selectedCandidateId !== "custom") {
        nextUsedCandidateIds.add(taskDraft.selectedCandidateId);
      }
      const nextCandidate =
        shotCandidates.find((item) => !nextUsedCandidateIds.has(item.id) && item.id !== "full-script") ??
        shotCandidates.find((item) => !nextUsedCandidateIds.has(item.id));
      const nextQueueIndex = sceneTasks.length + 2;

      setTaskDraft({
        mode: "create",
        taskName: `第 ${nextQueueIndex} 个镜头任务`,
        selectedCandidateId: nextCandidate?.id ?? "custom",
        scriptText: nextCandidate?.text ?? "",
        durationSeconds: nextCandidate?.durationSeconds ?? shotCandidateBaseDuration,
      });
      setExportMessage(
        nextCandidate
          ? `已创建镜头任务“${nextTaskName}”，继续选择“${nextCandidate.title}”创建下一个任务。`
          : `已创建镜头任务“${nextTaskName}”。系统候选已用完，可手动输入下一段镜头任务。`,
      );
      return;
    }

    setTaskDraft(null);
    setExportMessage(
      taskDraft.mode === "create"
        ? `已创建镜头任务“${nextTaskName}”，当前镜头片段为“${nextSegmentTitle}”。生成后可继续新建下一个未使用片段。`
        : `已更新当前镜头任务“${nextTaskName}”，当前镜头片段为“${nextSegmentTitle}”。下一步点击开始生成。`,
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
    if (!canGenerate) {
      setExportMessage("请先用扩写剧本创建镜头任务，再开始生成分镜提示词。");
      return;
    }
    if (!confirmDiscardDirty("重新生成当前镜头任务")) {
      return;
    }

    const source = taskSourceScript.trim();
    const taskDurationSeconds = normalizeDurationOption(
      currentSceneTask?.sourceDurationSeconds ?? durationSeconds,
      durationSeconds,
    );
    const taskSceneOption = findSceneOption(currentSceneTask?.sourceSceneType ?? null) ?? acceptedScriptScene ?? selectedSceneOption;
    setDurationSeconds(taskDurationSeconds);
    setSelectedScene(taskSceneOption.value);
    setBridgeBusy("generate");
    try {
      const response = await invokeGenerateStoryboard({
        task_name: taskName,
        script_id: taskScriptId,
        scene_type: taskSceneOption.value,
        scene_label: currentSceneTask?.sourceSceneLabel ?? taskSceneOption.label,
        scene_category: currentSceneTask?.sourceSceneCategory ?? taskSceneOption.group,
        shot_script: source,
        primary_scene_type: taskSceneOption.value,
        primary_scene_label: currentSceneTask?.sourceSceneLabel ?? taskSceneOption.label,
        primary_scene_category: currentSceneTask?.sourceSceneCategory ?? taskSceneOption.group,
        shot_scene_type: currentSceneTask?.sourceSceneType ?? taskSceneOption.value,
        shot_scene_label: currentSceneTask?.sourceSceneLabel ?? taskSceneOption.label,
        shot_intent: taskSegmentTitle || taskName,
        adaptation_reason:
          currentSceneTask?.sourceSceneType && currentSceneTask.sourceSceneType !== taskSceneOption.value
            ? "镜头任务使用用户选择的局部场景方向。"
            : null,
        expanded_script_text: source,
        selected_total_duration_seconds: taskDurationSeconds,
        model_config_summary: modelConfigSummary,
      });
      const nextRows = response.rows.map(mapGeneratedStoryboardRow);
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
          taskName: taskName.trim() || "未命名镜头任务",
          segmentTitle: taskSegmentTitle || "自定义镜头片段",
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
                  name: taskName.trim() || task.name,
                  segmentTitle: taskSegmentTitle || task.segmentTitle,
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
                  rowsHash: response.rows_hash,
                  updatedAtMs: response.updated_at_ms,
                  baseRevision: response.revision,
                  dirty: false,
                  hadRowEdits: false,
                  exportArtifactPath: undefined,
                  exportStatus: undefined,
                  exportContentHash: undefined,
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
    if (!confirmDiscardDirty("清空当前内容")) {
      return;
    }

    setTaskSourceScript("");
    setTaskScriptId(null);
    setTaskSegmentTitle("");
    setCurrentTaskId(null);
    setStoryboardResult(null);
    setLastExportResult(null);
    setRows([]);
    setRowsDirty(false);
    setCurrentPage(1);
    closeStoryboardEditDialog();
    setExportMessage("当前任务内容已清空。");
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
    if (!confirmDiscardDirty("确认使用新的扩写剧本")) {
      return false;
    }

    const scriptDuration = expandedScriptDurationSeconds ?? durationSeconds;
    const scriptScene = expandedScriptScene ?? selectedSceneOption;
    setAcceptedScript(scriptText);
    setAcceptedScriptId(expandedScriptResult.script_id);
    setAcceptedScriptDurationSeconds(scriptDuration);
    setAcceptedScriptScene(scriptScene);
    setExpandedScript(scriptText);
    setSynopsis(scriptText);
    setDurationSeconds(scriptDuration);
    setTaskSourceScript("");
    setTaskScriptId(null);
    setTaskSegmentTitle("");
    setShowExpandedScriptStatus(false);
    setExportMessage(
      `已确认使用扩写剧本 ${expandedScriptResult.script_id}，场景为${scriptScene.label}，目标时长 ${scriptDuration} 秒。下一步进入镜头拆解。`,
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

  const handleExportWords = async () => {
    if (bridgeBusy) {
      return;
    }
    if (!storyboardResult || !rows.length) {
      setExportMessage("当前没有可导出的分镜词。");
      return;
    }

    setBridgeBusy("export_words");
    try {
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
      const readyArtifact = findPrimaryReadyArtifact(response);
      updateCurrentTaskRecord({
        exportArtifactPath: readyArtifact?.artifact_path ?? undefined,
        exportStatus: response.export_status.status,
        exportContentHash: readyArtifact?.content_hash ?? null,
      });
      setExportMessage(`当前场景分镜词：${formatExportBundleMessage(response)}`);
    } catch (error) {
      setExportMessage(`导出分镜词失败：${formatProductError(error)}`);
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
        `完整剧本应汇总 ${generatedSceneTaskCount || generatedSceneTasks.length} 个已生成场景；当前 bridge 仍只支持单 result_id 导出，多场景完整提示词导出为 gated。`,
      );
      return;
    }

    setBridgeBusy("export_script");
    try {
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
      const readyArtifact = findPrimaryReadyArtifact(response);
      updateCurrentTaskRecord({
        exportArtifactPath: readyArtifact?.artifact_path ?? undefined,
        exportStatus: response.export_status.status,
        exportContentHash: readyArtifact?.content_hash ?? null,
      });
      setExportMessage(
        `完整剧本提示词：${formatExportBundleMessage(response)}；当前仅有 1 个已生成场景，按当前场景导出。`,
      );
    } catch (error) {
      setExportMessage(`导出完整剧本失败：${formatProductError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
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
                <small className={`top-select__status top-select__status--${modelProviderStatus.status}`}>
                  {modelRuntimeLabel}
                </small>
              </label>
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
                  <pre>{`model_config_summary: {
  provider: "qwen" | "doubao" | "custom",
  model: "qwen-plus",
  enabled: true,
  base_url_present: true,
  api_key_present: true
}

qwen_request: {
  task_type,
  scene_type,
  duration_seconds,
  story_input,
  kb_context_summary,
  selected_sample_ids,
  selected_kb_rules,
  output_schema
}`}</pre>
                </div>
              ) : (
                <form className="api-config" onSubmit={handleSaveModelConfig}>
                  <div className="api-config__notice">
                    当前只启用千问文本生成配置；API Key 仅保存在本次桌面会话中。未配置或调用失败时会自动使用本地候选结果；Seedance2.0 仍只是提示词适配目标。
                  </div>
                  <div className="api-config__rows">
                    <div className="api-config__row api-config__row--primary">
                      <label>
                        <span>provider</span>
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
                        <span>model</span>
                        <input
                          name="hope-model-name"
                          autoComplete="off"
                          value={modelConfigDraft.model}
                          onChange={(event) => handleModelConfigDraftChange("model", event.target.value)}
                          placeholder="qwen-plus"
                        />
                      </label>
                      <label>
                        <span>base_url</span>
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
                        <span>API Key（在这里输入）</span>
                        <input
                          name="hope-model-api-key"
                          type="password"
                          value={modelConfigDraft.apiKeyInput}
                          onChange={(event) => handleModelConfigDraftChange("apiKeyInput", event.target.value)}
                          placeholder={modelConfigDraft.apiKeyPresent ? "已配置，本次会话有效；保存时不显示明文" : "在这里输入千问 API Key"}
                          autoComplete="new-password"
                          data-lpignore="true"
                        />
                        <small>保存后清空输入框；业务 payload、日志和导出物只记录 api_key_present，不带明文。</small>
                      </label>
                      <label>
                        <span>api_key_ref（可选引用）</span>
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
                        <small>这里只能选环境变量/钥匙串引用，不填真实 Key。</small>
                      </label>
                      <label className="api-config__toggle">
                        <input
                          type="checkbox"
                          checked={modelConfigDraft.enabled}
                          onChange={(event) => handleModelConfigDraftChange("enabled", event.target.checked)}
                        />
                        <span>enabled</span>
                      </label>
                      <button type="submit" className="action-button action-button--dark">
                        保存配置
                      </button>
                    </div>
                    <div className="api-config__summary">
                      当前状态：{modelRuntimeLabel}。业务 payload 仅发送 provider / model / enabled / base_url_present / api_key_present，不发送明文 api_key。
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
                    onChange={(event) => setSelectedScene(event.target.value as SceneFusionOption)}
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
                  <span>目标时长</span>
                  <select
                    value={durationSeconds}
                    onChange={(event) => setDurationSeconds(Number(event.target.value))}
                  >
                    {DURATION_OPTIONS.map((option) => (
                      <option key={option} value={option}>
                        {option} 秒
                      </option>
                    ))}
                  </select>
                </label>
              </div>
              <div className="text-control">
                <textarea
                  className="synopsis-input"
                  value={synopsis}
                  onChange={(event) => {
                    setSynopsis(event.target.value);
                    if (expandedScriptAccepted) {
                      setExpandedScript(event.target.value);
                      setAcceptedScript(event.target.value);
                    }
                  }}
                  placeholder="请输入故事梗概"
                />
                <button type="button" className="text-control__expand" onClick={openSynopsisDialog}>
                  放大编辑
                </button>
              </div>
              <div className="script-actions">
                <button
                  type="button"
                  className="action-button action-button--dark"
                  onClick={handleExpandScript}
                  disabled={bridgeBusy !== null}
                >
                  {bridgeBusy === "expand" ? "扩写中" : "扩写剧本"}
                </button>
                {expandedScriptResult && !expandedScriptAccepted ? (
                  <button
                    type="button"
                    className="action-button action-button--light"
                    onClick={handleAcceptExpandedScript}
                    disabled={!(synopsis.trim() || expandedScript.trim()) || bridgeBusy !== null}
                  >
                    确定使用
                  </button>
                ) : null}
              </div>
            </div>
            <div className="kb-router-hint">
              剧本导演会按场景类型、目标时长和故事梗概匹配少量 KB 摘要扩写剧情；分镜导演会在镜头生成阶段再次匹配摘要，不会全量调用知识库。
            </div>
          </section>

          <section className="panel-section panel-section--task">
            <div className="section-name">镜头拆解</div>
            <div className="task-row">
              <input
                className="task-name-input"
                value={taskName}
                onChange={(event) => setTaskName(event.target.value)}
                placeholder="请输入任务名称（如：第一集分镜生成）"
              />
              <div className={hasTaskScript ? "task-source task-source--ready" : "task-source"}>
                <strong>{hasTaskScript ? "镜头剧本" : "拆解状态"}</strong>
                <span>{taskSourceLabel}</span>
              </div>
              <button
                type="button"
                className="action-button action-button--light"
                onClick={handleNewTask}
                disabled={bridgeBusy !== null || !canImportScript}
              >
                新建镜头任务
              </button>
            </div>
          </section>

          <section className="panel-section panel-section--storyboard">
            <div className="section-name">分镜产出区</div>
            <div className="storyboard-toolbar">
              <div className={isKbSummaryExpanded ? "storyboard-summary-panel storyboard-summary-panel--expanded" : "storyboard-summary-panel"}>
                {activeKbRouterResult ? (
                  <KbRouterSummary
                    router={activeKbRouterResult}
                    expanded={isKbSummaryExpanded}
                    onToggle={() => setIsKbSummaryExpanded((value) => !value)}
                  />
                ) : (
                  <div className="storyboard-summary-empty">
                    生成后在这里显示知识库匹配与镜头强绑定摘要。
                  </div>
                )}

                {rows.length ? (
                  <div className="grounding-note grounding-note--compact">
                    当前人物/动作若显示为占位文案，表示桌面端仅做显示层产品化；主线仍需补全人物/动作 grounding。
                  </div>
                ) : null}

                {shotGroundingSummary ? (
                  <div className="grounding-note grounding-note--details grounding-note--compact">
                    <strong>镜头强绑定</strong>
                    <span>镜头意图：{shotGroundingSummary.intent}</span>
                    <span>局部场景：{shotGroundingSummary.scene}</span>
                    <span>适配说明：{shotGroundingSummary.reason}</span>
                  </div>
                ) : null}
              </div>
              <div className="storyboard-toolbar__actions">
                <button
                  type="button"
                  className="action-button action-button--light"
                  onClick={handleImportScript}
                  disabled={!sceneTasks.length || bridgeBusy !== null}
                >
                  导入镜头任务
                </button>

                <div className="duration-pill">
                  <strong>镜头时长</strong>
                  <span>{currentTaskDuration} 秒</span>
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
                <button
                  type="button"
                  className="action-button action-button--dark"
                  onClick={handleGenerate}
                  disabled={!canGenerate || bridgeBusy !== null}
                >
                  {bridgeBusy === "generate" ? "生成中" : "开始生成"}
                </button>
              </div>
            </div>

            <div className="table-wrapper">
              <table className="storyboard-table">
                <thead>
                  <tr>
                    <th>序号</th>
                    <th>人物</th>
                    <th>镜头</th>
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
                          <span className="table-compact-text" title={formatInternalPlaceholder(row.person)}>
                            {formatInternalPlaceholder(row.person)}
                          </span>
                        </td>
                        <td>
                          <span className="table-compact-text" title={formatInternalPlaceholder(row.shot)}>
                            {formatInternalPlaceholder(row.shot)}
                          </span>
                        </td>
                        <td>
                          <span className="table-compact-text" title={row.sceneScale}>
                            {row.sceneScale}
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
                          <span className="table-compact-text" title={`${row.durationSeconds}`}>
                            {row.durationSeconds}
                          </span>
                        </td>
                        <td>
                          <div className="row-actions">
                            <button type="button" className="link-button" onClick={() => openStoryboardEditDialog(row)}>
                              修改
                            </button>
                            <button type="button" className="link-button" onClick={() => handleDuplicate(row.id)}>
                              复制
                            </button>
                            <button type="button" className="link-button link-button--danger" onClick={() => handleDelete(row.id)}>
                              删除
                            </button>
                          </div>
                        </td>
                      </tr>
                    ))
                  ) : (
                    <tr>
                      <td colSpan={10} className="empty-table-cell">
                        还没有生成分镜，请先导入镜头任务并开始生成。
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>

            <div className="pagination-row">
              <div className="page-size-control">
                <span>每页显示：</span>
                <select value={PAGE_SIZE} disabled>
                  <option value={PAGE_SIZE}>{PAGE_SIZE}</option>
                </select>
              </div>

              <div className="total-count">共 {rows.length} 条</div>

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
          </section>

          <section className="export-row">
            <div className="export-message">{exportMessage.trim()}</div>
            {lastExportResult ? (
              <div className="export-artifacts">
                {lastExportResult.artifacts.map((artifact) => (
                  <span key={artifact.artifact_id}>{formatArtifact(artifact)}</span>
                ))}
              </div>
            ) : null}
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
                onClick={handleExportScript}
                disabled={!storyboardResult || bridgeBusy !== null}
              >
                {bridgeBusy === "export_script" ? "导出中" : "导出完整剧本"}
              </button>
            </div>
          </section>
        </div>

        {taskDraft ? (
          <div className="edit-dialog-backdrop" onClick={() => setTaskDraft(null)}>
            <div className="task-draft-dialog" onClick={(event) => event.stopPropagation()}>
              <div className="edit-dialog__header">
                <div>
                  <strong>{taskDraft.mode === "create" ? "新建镜头任务" : "更新镜头剧本"}</strong>
                  <span>
                    先把扩写剧本拆成镜头任务队列；下方“导入镜头任务”再选择当前要生成的任务。
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
                    <div className="task-draft-label">系统拆分建议</div>
                    {shotCandidates.length ? shotCandidates.map((candidate) => {
                      const matchedTask = sceneTasks.find((task) => task.candidateId === candidate.id);
                      const candidateStatus = currentTaskId && matchedTask?.id === currentTaskId
                        ? "当前"
                        : matchedTask?.status === "generated"
                        ? "已生成"
                        : matchedTask
                        ? "已建任务"
                        : "未使用";

                      return (
                        <button
                          key={candidate.id}
                          type="button"
                          className={
                            taskDraft.selectedCandidateId === candidate.id
                              ? "shot-candidate shot-candidate--active"
                              : "shot-candidate"
                          }
                          onClick={() => handleTaskCandidateSelect(candidate.id)}
                        >
                          <span className="shot-candidate__head">
                            <strong>{candidate.title}</strong>
                            <em>{candidateStatus}</em>
                          </span>
                          <span>{candidate.preview}</span>
                          <small>预计 {candidate.durationSeconds} 秒</small>
                        </button>
                      );
                    }) : (
                      <div className="shot-candidate shot-candidate--empty">
                        <strong>暂无可拆分内容</strong>
                        <span>请先扩写剧本，或在右侧直接输入自定义片段。</span>
                      </div>
                    )}
                  </div>

                  <div className="task-queue">
                    <div className="task-draft-label">镜头任务队列</div>
                    {sceneTasks.length ? sceneTasks.map((task, index) => (
                      <button
                        key={task.id}
                        type="button"
                        className={task.id === currentTaskId ? "task-queue__item task-queue__item--active" : "task-queue__item"}
                        onClick={() => handleImportSceneTask(task)}
                      >
                        <strong>{index + 1}. {task.name}</strong>
                        <span>
                          {task.segmentTitle} · 预计 {normalizeDurationOption(task.sourceDurationSeconds ?? task.selectedTotalDurationSeconds, durationSeconds)} 秒 · {task.status === "generated" ? `${task.rowCount ?? 0} 行已生成` : "待生成"}
                        </span>
                      </button>
                    )) : (
                      <div className="task-queue__empty">还没有镜头任务，确认创建后会出现在这里。</div>
                    )}
                  </div>
                </div>
                <div className="task-draft-editor">
                  <label>
                    <span>镜头任务名称</span>
                    <input
                      value={taskDraft.taskName}
                      onChange={(event) =>
                        setTaskDraft((current) =>
                          current ? { ...current, taskName: event.target.value } : current,
                        )
                      }
                    />
                  </label>
                  <label className="task-draft-editor__duration">
                    <span>预计镜头时长</span>
                    <select
                      value={taskDraft.durationSeconds}
                      onChange={(event) =>
                        setTaskDraft((current) =>
                          current ? { ...current, durationSeconds: Number(event.target.value) } : current,
                        )
                      }
                    >
                      {DURATION_OPTIONS.map((option) => (
                        <option key={option} value={option}>
                          {option} 秒
                        </option>
                      ))}
                    </select>
                  </label>
                  <label className="task-draft-editor__text">
                    <span>本次镜头任务使用的剧本片段</span>
                    <textarea
                      value={taskDraft.scriptText}
                      onChange={(event) =>
                        setTaskDraft((current) =>
                          current
                            ? {
                                ...current,
                                selectedCandidateId: "custom",
                                scriptText: event.target.value,
                              }
                            : current,
                        )
                      }
                    />
                  </label>
                  <div className="task-draft-actions">
                    <span>确认后，这段文本会进入镜头任务队列；分镜产出区可按任务导入生成。</span>
                    <div className="task-draft-actions__buttons">
                      {taskDraft.mode === "create" ? (
                        <button
                          type="button"
                          className="action-button action-button--light"
                          onClick={() => handleConfirmTaskDraft(true)}
                        >
                          创建并继续新建
                        </button>
                      ) : null}
                      <button
                        type="button"
                        className="action-button action-button--dark"
                        onClick={() => handleConfirmTaskDraft(false)}
                      >
                        {taskDraft.mode === "create" ? "确认创建镜头" : "确认更新镜头"}
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
                {sceneTasks.map((task, index) => (
                  <button
                    key={task.id}
                    type="button"
                    className={task.id === currentTaskId ? "task-picker-card task-picker-card--active" : "task-picker-card"}
                    onClick={() => handleImportSceneTask(task)}
                  >
                    <div>
                      <strong>{index + 1}. {task.name}</strong>
                      <span>
                        {task.segmentTitle} · 预计 {normalizeDurationOption(task.sourceDurationSeconds ?? task.selectedTotalDurationSeconds, durationSeconds)} 秒
                      </span>
                    </div>
                    <em>{task.status === "generated" ? `${task.rowCount ?? 0} 行已生成，可重新导入` : "待生成"}</em>
                    <p>{truncatePreview(task.scriptText, 140)}</p>
                  </button>
                ))}
              </div>
            </div>
          </div>
        ) : null}

        {textDialog ? (
          <div className="edit-dialog-backdrop" onClick={() => setTextDialog(null)}>
            <div className="text-dialog" onClick={(event) => event.stopPropagation()}>
              <div className="edit-dialog__header">
                <div>
                  <strong>{textDialog.title}</strong>
                  <span>{textDialog.helper}</span>
                </div>
                <button type="button" className="toolbar-button" onClick={() => setTextDialog(null)}>
                  关闭
                </button>
              </div>
              <div className="text-dialog__body">
                <textarea
                  value={textDialog.value}
                  readOnly={!textDialog.editable}
                  onChange={(event) =>
                    setTextDialog((current) =>
                      current ? { ...current, value: event.target.value } : current,
                    )
                  }
                />
                <div className="text-dialog__actions">
                  <span>Enter 可换行，右下角可拖拽放大。</span>
                  {textDialog.kind === "expandedScript" ? (
                    <button
                      type="button"
                      className="action-button action-button--light"
                      onClick={() => {
                        if (handleAcceptExpandedScript()) {
                          setTextDialog(null);
                        }
                      }}
                      disabled={!(synopsis.trim() || expandedScript.trim()) || bridgeBusy !== null || expandedScriptAccepted}
                    >
                      {expandedScriptAccepted ? "已确认使用" : "确定使用"}
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
                  <span>镜头</span>
                  <input value={editingRow.shot} onChange={(event) => handleEditField("shot", event.target.value)} />
                </label>
                <label>
                  <span>景别</span>
                  <input value={editingRow.sceneScale} onChange={(event) => handleEditField("sceneScale", event.target.value)} />
                </label>
                <label className="edit-grid__wide">
                  <span>画面描述</span>
                  <textarea value={editingRow.visualDescription} onChange={(event) => handleEditField("visualDescription", event.target.value)} />
                </label>
                <label className="edit-grid__wide">
                  <span>角色动作</span>
                  <textarea value={editingRow.characterAction} onChange={(event) => handleEditField("characterAction", event.target.value)} />
                </label>
                <label className="edit-grid__wide">
                  <span>对白/旁白</span>
                  <textarea value={editingRow.dialogue} onChange={(event) => handleEditField("dialogue", event.target.value)} />
                </label>
                <label className="edit-grid__wide">
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
}: {
  label: string;
  value: string;
  onOpen: () => void;
}) {
  const text = value.trim() || "暂无内容";

  return (
    <button type="button" className="table-text-preview" onClick={onOpen} title={`${label}：${text}`}>
      <span>{text}</span>
      <em>展开</em>
    </button>
  );
}

function KbRouterSummary({
  router,
  expanded,
  onToggle,
}: {
  router: KbRouterRuntimeResponse;
  expanded: boolean;
  onToggle: () => void;
}) {
  const sampleCount = router.selected_sample_ids.length;
  const ruleCount = router.selected_kb_rules.length;
  const summary = formatUserKbSummary(router);

  return (
    <div className="kb-router-summary">
      <div className="kb-router-summary__line">
        <strong>已匹配知识库规则</strong>
        <span>已匹配 {sampleCount} 个参考样本</span>
        <span>已应用 {ruleCount} 条规则</span>
        <span>{(router.full_kb_rows_included ?? 0) === 0 ? "未全量调用知识库" : "知识库调用已被限制"}</span>
        <button type="button" className="link-button" onClick={onToggle}>
          {expanded ? "收起摘要" : "展开摘要"}
        </button>
      </div>
      {expanded ? (
        <p className="kb-router-summary__detail">{summary}</p>
      ) : null}
    </div>
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

function formatModelProviderStatus(
  config: ModelConfigState,
  status: TextModelProviderStatus,
) {
  if (config.provider !== "qwen") {
    return "预留，当前未启用";
  }
  if (status.status === "fallback") {
    return "千问：调用失败已回退";
  }
  if (status.live_ready) {
    return "千问：已启用";
  }
  if (status.enabled && !status.live_ready) {
    return "千问：未配置";
  }
  if (status.api_key_present || status.base_url_present) {
    return "千问：已配置未启用";
  }
  return "千问：未配置";
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
  const totalDuration = normalizeDurationOption(totalDurationSeconds, 15);
  const count = Math.max(1, segmentCount);
  return nearestDurationOption(Math.max(5, totalDuration / count), totalDuration);
}

function buildShotCandidates(expandedScript: string, totalDurationSeconds: number): ShotCandidate[] {
  const scriptBody = extractScriptBody(expandedScript);
  if (!scriptBody) {
    return [];
  }

  const sentences = splitScriptBody(scriptBody);
  const segments = mergeScriptSegments(sentences.length ? sentences : [scriptBody]);
  const segmentDuration = estimateShotDuration(totalDurationSeconds, segments.length);
  const fullDuration = normalizeDurationOption(totalDurationSeconds, 15);
  const candidates = segments.slice(0, 8).map((segment, index) => ({
    id: `candidate-${index + 1}`,
    title: `场景候选 ${index + 1}`,
    text: segment,
    preview: truncatePreview(segment),
    durationSeconds: segmentDuration,
  }));

  if (candidates.length > 1) {
    candidates.push({
      id: "full-script",
      title: "完整扩写剧本",
      text: scriptBody,
      preview: truncatePreview(scriptBody),
      durationSeconds: fullDuration,
    });
  }

  return candidates.length
    ? candidates
    : [
        {
          id: "custom",
          title: "自定义镜头片段",
          text: scriptBody,
          preview: truncatePreview(scriptBody),
          durationSeconds: fullDuration,
        },
      ];
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

function buildShotGroundingSummary(rows: StoryboardWorkbenchRow[]) {
  const row = rows.find(
    (item) => item.shotIntent || item.shotSceneType || item.adaptationReason,
  );
  if (!row) {
    return null;
  }

  const shotScene = [row.shotSceneLabel, row.shotSceneType]
    .map((value) => value?.trim())
    .filter(Boolean)
    .join(" · ");

  return {
    intent: formatInternalPlaceholder(row.shotIntent || "未返回"),
    scene: formatInternalPlaceholder(shotScene || "未返回"),
    reason: row.adaptationReason?.trim() || "沿用主场景方向",
  };
}

function mapGeneratedStoryboardRow(row: GeneratedStoryboardRow): StoryboardWorkbenchRow {
  const promptText = row.prompt_text?.trim();
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
    person: formatInternalPlaceholder(row.person || "not_specified"),
    shot: formatInternalPlaceholder(row.shot_title || row.shot_id),
    sceneScale: row.scene_scale || "source",
    visualDescription: row.visual_description,
    characterAction: formatInternalPlaceholder(row.character_action),
    dialogue: row.dialogue || "（无）",
    prompt: promptText || "prompt_text 未生成，等待主线镜头 grounding",
    durationSeconds: row.duration_seconds,
    backendRow: row,
  };
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
      person: row.person,
      shot_title: row.shot,
      scene_scale: row.sceneScale,
      visual_description: row.visualDescription,
      character_action: row.characterAction,
      dialogue: row.dialogue,
      prompt_text: row.prompt === "prompt_text 未生成，等待主线镜头 grounding" ? "" : row.prompt,
      prompt_text_compilation_status: backend?.prompt_text_compilation_status || "ReadyStub",
      prompt_text_compilation_warnings: backend?.prompt_text_compilation_warnings ?? [],
      prompt_text_source_row_id: backend?.prompt_text_source_row_id || shotId,
      duration_seconds: row.durationSeconds,
      scene_performance_projection: scenePerformanceProjection,
      external_reference_handle_candidates: backend?.external_reference_handle_candidates ?? [],
      sequence_grouping: sequenceGrouping,
    };
  });
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

function formatUserKbSummary(router: KbRouterRuntimeResponse) {
  const ruleText = router.selected_kb_rules
    .slice(0, 3)
    .map((rule) => cleanKbDisplayText(rule.summary))
    .filter(Boolean)
    .join("；");
  const summaryText = cleanKbDisplayText(router.kb_context_summary);
  const baseSummary =
    "本次只抽取与当前场景类型、镜头意图、结构和时长直接相关的少量参考样本与规则，重点用于场景结构、动作连续性、时长守恒和当前文本提示词约束。";
  const detail = [summaryText, ruleText].filter(Boolean).join("；");
  return truncatePreview(`${baseSummary}${detail ? `规则摘要：${detail}` : ""}`, 360);
}

function cleanKbDisplayText(value: string) {
  return value
    .replace(/source[_-]register/gi, "来源内部字段")
    .replace(/provenance/gi, "来源内部字段")
    .replace(/overlay\s+json/gi, "内部配置")
    .replace(/prompt_body/gi, "候选提示词")
    .replace(/teaching_note/gi, "教学说明")
    .replace(/content_cache_key/gi, "缓存标记")
    .replace(/snapshot_checksum/gi, "快照校验")
    .replace(/retrieval_trace/gi, "匹配追溯摘要")
    .replace(/selected_sample_ids=/gi, "参考样本=")
    .replace(/参考样本=[^。]+。?/g, "参考样本已按数量统计。")
    .replace(/样本\s+[A-Za-z0-9_.:-]+：/g, "参考样本：")
    .replace(/full_kb_rows_included/gi, "全量知识库调用")
    .replace(/\s+/g, " ")
    .trim();
}

function hasRetrievalTraceSummary(response: ExportBundleResponse) {
  return response.artifacts.some(
    (artifact) =>
      Boolean(artifact.retrieval_trace) ||
      Boolean(artifact.kb_context_summary) ||
      Boolean(artifact.selected_sample_ids?.length) ||
      Boolean(artifact.selected_kb_rule_ids?.length),
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

function findPrimaryReadyArtifact(response: ExportBundleResponse) {
  return (
    response.artifacts.find((artifact) => artifact.ready && Boolean(artifact.artifact_path)) ??
    response.artifacts.find((artifact) => artifact.ready) ??
    null
  );
}

function formatWarnings(warnings: ProductWarning[]) {
  if (!warnings.length) {
    return "无";
  }

  const codes = warnings.slice(0, 3).map((warning) => warning.code).join(", ");
  return warnings.length > 3 ? `${codes} +${warnings.length - 3}` : codes;
}

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function formatProductError(error: unknown) {
  const raw = formatError(error);
  if (/storyboard_result_not_found/i.test(raw)) {
    return "没有找到可用的分镜结果，请先生成后再保存或导出。";
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

function formatExportBundleMessage(response: ExportBundleResponse) {
  const readyArtifacts = response.artifacts.filter((artifact) => artifact.ready);
  const readyWithPath = readyArtifacts.filter((artifact) => artifact.artifact_path);
  const prefix = readyWithPath.length ? "导出已生成" : "export_bundle 返回";
  const primary = findPrimaryReadyArtifact(response);
  const rows = primary?.row_count != null ? ` / row_count=${primary.row_count}` : "";
  const hash = primary?.content_hash ? ` / content_hash=${primary.content_hash}` : "";
  const edited = primary?.edited_rows_applied != null ? ` / edited_rows_applied=${primary.edited_rows_applied}` : "";
  const statuses = primary?.prompt_text_compilation_statuses?.length
    ? ` / prompt_text=${primary.prompt_text_compilation_statuses.join(",")}`
    : "";
  const warnings = primary?.prompt_text_compilation_warning_codes?.length
    ? ` / warnings=${primary.prompt_text_compilation_warning_codes.join(",")}`
    : "";
  const kbTrace = hasRetrievalTraceSummary(response)
    ? " / 导出已包含知识库匹配摘要和样本 ID 追溯"
    : "";
  return `${prefix}：${response.export_manifest_id} / ${readyArtifacts.length}/${response.artifacts.length} ready / ${response.export_status.status}${rows}${hash}${edited}${statuses}${warnings}${kbTrace}`;
}

function formatArtifact(artifact: ExportArtifactRecord) {
  const ready = artifact.ready ? "ready" : `blocked:${artifact.blocked_reason ?? "unknown"}`;
  const path = artifact.artifact_path ? ` path=${artifact.artifact_path}` : "";
  const hash = artifact.content_hash ? ` hash=${artifact.content_hash}` : "";
  const rows = artifact.row_count != null ? ` rows=${artifact.row_count}` : "";
  const edited = artifact.edited_rows_applied != null ? ` edited_rows_applied=${artifact.edited_rows_applied}` : "";
  const promptStatus = artifact.prompt_text_compilation_statuses?.length
    ? ` prompt_text=${artifact.prompt_text_compilation_statuses.join(",")}`
    : "";
  const warningCodes = artifact.prompt_text_compilation_warning_codes?.length
    ? ` warnings=${artifact.prompt_text_compilation_warning_codes.join(",")}`
    : "";
  return `${artifact.artifact_kind} ${ready}${path}${hash}${rows}${edited}${promptStatus}${warningCodes}`;
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

function renumberRows(rows: StoryboardWorkbenchRow[]) {
  return rows.map((row, index) => ({
    ...row,
    order: index + 1,
  }));
}

function createRowId() {
  return `row-${Math.random().toString(36).slice(2, 10)}`;
}

function createTaskRecordId() {
  return `scene-task-${Math.random().toString(36).slice(2, 10)}`;
}
