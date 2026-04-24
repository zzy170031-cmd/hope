import { useEffect, useMemo, useState } from "react";
import { Shell } from "./components/Shell";
import logoUrl from "./assets/hope-desktop-logo.png";
import {
  expandScript as invokeExpandScript,
  exportBundle as invokeExportBundle,
  generateStoryboard as invokeGenerateStoryboard,
} from "./bridge/hopeBridge";
import { ROUTES, resolveRoute } from "./routes";
import type {
  ExpandScriptResponse,
  ExportArtifactRecord,
  ExportBundleResponse,
  GenerateStoryboardResponse,
  GeneratedStoryboardRow,
  ModelConfigSummary,
  ProductWarning,
  SceneFusionOption,
  StoryboardExportStatus,
  StoryboardWorkbenchRow,
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

const PAGE_SIZE = 5;
const DURATION_OPTIONS = [15, 30, 45, 60];
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
  { group: "三国 / 国战 / SLG", value: "siege_defense", label: "城池攻防" },
  { group: "三国 / 国战 / SLG", value: "slg_sandbox_view", label: "沙盘战略视口" },
  { group: "三国 / 国战 / SLG", value: "slg_march_encirclement", label: "行军轨迹合围" },
  { group: "三国 / 国战 / SLG", value: "slg_city_growth", label: "城建演进反馈" },
  { group: "三国 / 国战 / SLG", value: "slg_battle_report", label: "武将揭示战报" },
];

const API_DOC_ITEMS = [
  "当前页面是桌面工作台的受控交互界面，重点用于脚本整理、分镜生成与本地导出。",
  "模型选择、脚本扩写和分镜生成都服务于当前工作台体验，不代表实时模型已接通。",
  "导出动作仅作用于当前页面中的本地结果，不代表正式导出链路已经上线。",
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
  const [selectedScene, setSelectedScene] = useState<SceneFusionOption>(DEFAULT_SCENE);
  const [synopsis, setSynopsis] = useState(DEFAULT_SYNOPSIS);
  const [expandedScript, setExpandedScript] = useState("");
  const [expandedScriptResult, setExpandedScriptResult] = useState<ExpandScriptResponse | null>(null);
  const [taskName, setTaskName] = useState(DEFAULT_TASK_NAME);
  const [taskSourceScript, setTaskSourceScript] = useState("");
  const [taskScriptId, setTaskScriptId] = useState<string | null>(null);
  const [durationSeconds, setDurationSeconds] = useState(15);
  const [rows, setRows] = useState<StoryboardWorkbenchRow[]>([]);
  const [storyboardResult, setStoryboardResult] = useState<GenerateStoryboardResponse | null>(null);
  const [lastExportResult, setLastExportResult] = useState<ExportBundleResponse | null>(null);
  const [bridgeBusy, setBridgeBusy] = useState<
    "expand" | "generate" | "export_words" | "export_script" | null
  >(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [jumpPage, setJumpPage] = useState("1");
  const [editingRowId, setEditingRowId] = useState<string | null>(null);
  const [taskSerial, setTaskSerial] = useState(1);
  const [exportMessage, setExportMessage] = useState("等待主线 bridge 返回结果。");

  const editingRow = useMemo(
    () => rows.find((row) => row.id === editingRowId) ?? null,
    [rows, editingRowId],
  );

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

  const canImportScript = expandedScript.trim().length > 0;
  const canGenerate = taskName.trim().length > 0 && (taskSourceScript.trim().length > 0 || expandedScript.trim().length > 0 || synopsis.trim().length > 0);
  const pageTokens = useMemo(() => buildPageTokens(pageCount, currentPage), [pageCount, currentPage]);
  const selectedSceneOption = useMemo(() => resolveSceneOption(selectedScene), [selectedScene]);
  const sceneOptionGroups = useMemo(() => groupSceneOptions(SCENE_OPTIONS), []);
  const selectedModelLabel = useMemo(() => resolveModelLabel(modelConfig.provider), [modelConfig.provider]);
  const modelReservedWarning = modelConfig.provider === "qwen"
    ? ""
    : "该模型接口已配置为预留状态，当前仍使用本地文本生成桥接。";
  const modelConfigSummary = useMemo(
    () => buildModelConfigSummary(modelConfig),
    [modelConfig],
  );

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
    setExportMessage(
      provider === "qwen"
        ? "已选择千问 Qwen，当前仍使用本地文本生成桥接。"
        : "该模型接口已配置为预留状态，当前仍使用本地文本生成桥接。",
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

  const handleSaveModelConfig = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const provider = modelConfigDraft.provider;
    const apiKeyRef = normalizeApiKeyRef(modelConfigDraft.apiKeyRef);
    const hadRawValueInRef = modelConfigDraft.apiKeyRef.trim().length > 0 && !apiKeyRef;
    const nextConfig: ModelConfigState = {
      provider,
      model: modelConfigDraft.model.trim() || MODEL_DEFAULTS[provider].model,
      baseUrl: modelConfigDraft.baseUrl.trim(),
      apiKeyRef,
      enabled: modelConfigDraft.enabled,
      apiKeyPresent:
        modelConfigDraft.apiKeyInput.trim().length > 0 ||
        apiKeyRef.length > 0 ||
        hadRawValueInRef ||
        (provider === modelConfig.provider && modelConfig.apiKeyPresent),
    };
    setSelectedModel(provider);
    setModelConfig(nextConfig);
    setModelConfigDraft({ ...nextConfig, apiKeyInput: "" });
    setExportMessage(
      hadRawValueInRef
        ? "检测到密钥引用栏疑似填入了明文 Key，已清空该栏；只记录 api_key_present，不保留明文。"
        : provider === "qwen"
        ? `已保存 ${resolveModelLabel(provider)} 本地配置；本轮不会发起真实模型请求。`
        : "已保存预留模型配置；当前仍使用本地文本生成桥接。",
    );
  };

  const handleExpandScript = async () => {
    setBridgeBusy("expand");
    try {
      const response = await invokeExpandScript({
        scene_type: selectedSceneOption.value,
        scene_label: selectedSceneOption.label,
        scene_category: selectedSceneOption.group,
        model_config_summary: modelConfigSummary,
        synopsis_text: synopsis,
      });
      setExpandedScriptResult(response);
      setExpandedScript(response.expanded_script_text);
      setTaskScriptId(null);
      setLastExportResult(null);
      setExportMessage(`expand_script 完成：${response.script_id}`);
    } catch (error) {
      setExportMessage(`expand_script 失败：${formatError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
  };

  const handleNewTask = () => {
    const nextSerial = taskSerial + 1;
    setTaskSerial(nextSerial);
    setTaskName(`第 ${nextSerial} 个任务`);
    setTaskSourceScript("");
    setTaskScriptId(null);
    setStoryboardResult(null);
    setLastExportResult(null);
    setRows([]);
    setCurrentPage(1);
    setEditingRowId(null);
    setExportMessage("已创建新任务，请先导入扩写脚本。");
  };

  const handleImportScript = () => {
    if (!canImportScript) {
      return;
    }

    setTaskSourceScript(expandedScript.trim());
    setTaskScriptId(expandedScriptResult?.script_id ?? null);
    setLastExportResult(null);
    setExportMessage(
      expandedScriptResult?.script_id
        ? `已导入扩写脚本：${expandedScriptResult.script_id}`
        : "已导入扩写脚本正文。",
    );
  };

  const handleGenerate = async () => {
    if (!canGenerate) {
      return;
    }

    const source = taskSourceScript.trim() || expandedScript.trim() || synopsis.trim();
    setBridgeBusy("generate");
    try {
      const response = await invokeGenerateStoryboard({
        task_name: taskName,
        script_id: taskScriptId,
        expanded_script_text: taskScriptId ? null : source,
        selected_total_duration_seconds: durationSeconds,
        model_config_summary: modelConfigSummary,
      });
      const nextRows = response.rows.map(mapGeneratedStoryboardRow);
      setStoryboardResult(response);
      setLastExportResult(null);
      setRows(nextRows);
      setCurrentPage(1);
      setEditingRowId(null);
      setExportMessage(
        `generate_storyboard 完成：${response.result_id} / ${nextRows.length} rows / ${response.export_status.status}`,
      );
    } catch (error) {
      setExportMessage(`generate_storyboard 失败：${formatError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
  };

  const handleClear = () => {
    setTaskSourceScript("");
    setTaskScriptId(null);
    setStoryboardResult(null);
    setLastExportResult(null);
    setRows([]);
    setCurrentPage(1);
    setEditingRowId(null);
    setExportMessage("当前任务内容已清空。");
  };

  const handleEditField = <K extends keyof StoryboardWorkbenchRow>(field: K, value: StoryboardWorkbenchRow[K]) => {
    if (!editingRow) {
      return;
    }

    setRows((current) =>
      current.map((row) => (row.id === editingRow.id ? { ...row, [field]: value } : row)),
    );
  };

  const handleDuplicate = (rowId: string) => {
    setRows((current) => {
      const index = current.findIndex((row) => row.id === rowId);
      if (index === -1) {
        return current;
      }

      const target = current[index];
      const duplicate: StoryboardWorkbenchRow = {
        ...target,
        id: createRowId(),
        shot: `${target.shot} - 复制`,
      };
      const next = [...current];
      next.splice(index + 1, 0, duplicate);
      return renumberRows(next);
    });
  };

  const handleDelete = (rowId: string) => {
    setRows((current) => renumberRows(current.filter((row) => row.id !== rowId)));
    if (editingRowId === rowId) {
      setEditingRowId(null);
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

  const handleExportWords = async () => {
    if (!storyboardResult || !rows.length) {
      setExportMessage("当前没有可导出的分镜词。");
      return;
    }

    setBridgeBusy("export_words");
    try {
      const response = await invokeExportBundle({
        result_id: storyboardResult.result_id,
        export_format: "storyboard_words",
      });
      setLastExportResult(response);
      setExportMessage(formatExportBundleMessage(response));
    } catch (error) {
      setExportMessage(`export_bundle 失败：${formatError(error)}`);
    } finally {
      setBridgeBusy(null);
    }
  };

  const handleExportScript = async () => {
    if (!storyboardResult || !rows.length) {
      setExportMessage("当前没有可导出的完整脚本结果。");
      return;
    }

    setBridgeBusy("export_script");
    try {
      const response = await invokeExportBundle({
        result_id: storyboardResult.result_id,
        export_format: "full_script",
      });
      setLastExportResult(response);
      setExportMessage(formatExportBundleMessage(response));
    } catch (error) {
      setExportMessage(`export_bundle 失败：${formatError(error)}`);
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
            <section className="top-panel">
              <div className="top-panel__title">{activePanel === "docs" ? "API文档" : "API接口"}</div>
              {activePanel === "docs" ? (
                <ul>
                  {API_DOC_ITEMS.map((item) => (
                    <li key={item}>{item}</li>
                  ))}
                </ul>
              ) : (
                <form className="api-config" onSubmit={handleSaveModelConfig}>
                  <div className="api-config__notice">
                    当前为本地配置预留，本轮不发起真实模型请求；Seedance2.0 仅作为提示词适配目标，不是当前运行时视频接口。
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
                          placeholder={modelConfigDraft.apiKeyPresent ? "已配置，保存时不显示明文" : "仅用于本地 presence 标记"}
                          autoComplete="new-password"
                          data-lpignore="true"
                        />
                        <small>保存后只记录 api_key_present，不显示、不发送明文。</small>
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
                      payload 仅发送 provider / model / enabled / api_key_present，不发送明文 api_key。
                      {modelConfigDraft.provider !== "qwen" ? (
                        <strong>该模型接口已配置为预留状态，当前仍使用本地文本生成桥接。</strong>
                      ) : null}
                    </div>
                  </div>
                </form>
              )}
            </section>
          ) : null}

          <section className="panel-section">
            <div className="section-name">脚本区</div>
            <div className="script-row">
              <label className="scene-select">
                <span>场景类型选择：</span>
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
              <textarea
                className="synopsis-input"
                value={synopsis}
                onChange={(event) => setSynopsis(event.target.value)}
                placeholder="请输入故事梗概"
              />
              <button
                type="button"
                className="action-button action-button--dark"
                onClick={handleExpandScript}
                disabled={bridgeBusy === "expand"}
              >
                扩写脚本
              </button>
            </div>
            {expandedScriptResult || expandedScript ? (
              <div className="bridge-status bridge-status--script">
                <div className="bridge-status__meta">
                  <strong>script_id</strong>
                  <span>{expandedScriptResult?.script_id ?? "expanded_script_text"}</span>
                  <strong>warnings</strong>
                  <span>{formatWarnings(expandedScriptResult?.warnings ?? [])}</span>
                </div>
                <pre>{expandedScript || "等待 expanded_script_text"}</pre>
              </div>
            ) : null}
          </section>

          <section className="panel-section panel-section--task">
            <div className="section-name">任务条</div>
            <div className="task-row">
              <input
                className="task-name-input"
                value={taskName}
                onChange={(event) => setTaskName(event.target.value)}
                placeholder="请输入任务名称（如：第一集分镜生成）"
              />
              <button type="button" className="action-button action-button--light" onClick={handleNewTask}>
                新建任务
              </button>
            </div>
          </section>

          <section className="panel-section panel-section--storyboard">
            <div className="section-name">分镜产出区</div>
            <div className="storyboard-toolbar">
              <button
                type="button"
                className="action-button action-button--light"
                onClick={handleImportScript}
                disabled={!canImportScript || bridgeBusy !== null}
              >
                导入扩写脚本
              </button>

              <label className="duration-select">
                <span>时长选择</span>
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
                className="action-button action-button--dark"
                onClick={handleGenerate}
                disabled={!canGenerate || bridgeBusy === "generate"}
              >
                开始生成
              </button>
            </div>

            {storyboardResult ? (
              <div className="bridge-status bridge-status--compact">
                <div className="bridge-status__meta">
                  <strong>result_id</strong>
                  <span>{storyboardResult.result_id}</span>
                  <strong>export_status</strong>
                  <span>{storyboardResult.export_status.status}</span>
                  <strong>blockers</strong>
                  <span>{formatWarnings(storyboardResult.export_status.blockers)}</span>
                  <strong>warnings</strong>
                  <span>{formatWarnings(storyboardResult.export_status.warnings)}</span>
                </div>
              </div>
            ) : null}

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
                        <td>{row.person}</td>
                        <td>{row.shot}</td>
                        <td>{row.sceneScale}</td>
                        <td>{row.visualDescription}</td>
                        <td>{row.characterAction}</td>
                        <td>{row.dialogue}</td>
                        <td>{row.prompt}</td>
                        <td>{row.durationSeconds}</td>
                        <td>
                          <div className="row-actions">
                            <button type="button" className="link-button" onClick={() => setEditingRowId(row.id)}>
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
                        还没有生成分镜，请先导入扩写脚本并开始生成。
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
                导出分镜词
              </button>
              <button
                type="button"
                className="action-button action-button--dark"
                onClick={handleExportScript}
                disabled={!storyboardResult || bridgeBusy !== null}
              >
                导出完整脚本
              </button>
            </div>
          </section>
        </div>

        {editingRow ? (
          <div className="edit-dialog-backdrop" onClick={() => setEditingRowId(null)}>
            <div className="edit-dialog" onClick={(event) => event.stopPropagation()}>
              <div className="edit-dialog__header">
                <div>
                  <strong>修改分镜</strong>
                  <span>当前行为本地工作台结果，可继续调整。</span>
                </div>
                <button type="button" className="toolbar-button" onClick={() => setEditingRowId(null)}>
                  关闭
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
            </div>
          </div>
        ) : null}
      </div>
    </Shell>
  );
}

function resolveSceneOption(value: SceneFusionOption) {
  return SCENE_OPTIONS.find((option) => option.value === value) ?? SCENE_OPTIONS[0];
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

function buildModelConfigSummary(config: ModelConfigState): ModelConfigSummary {
  return {
    provider: config.provider,
    model: config.model.trim() || MODEL_DEFAULTS[config.provider].model,
    enabled: config.enabled,
    api_key_present: config.apiKeyPresent,
  };
}

function normalizeApiKeyRef(value: string) {
  const cleanValue = value.trim();
  return API_KEY_REF_OPTIONS.some((option) => option.value === cleanValue) ? cleanValue : "";
}

function mapGeneratedStoryboardRow(row: GeneratedStoryboardRow): StoryboardWorkbenchRow {
  return {
    id: row.shot_id || createRowId(),
    order: row.order,
    person: row.person || "not_specified",
    shot: row.shot_title || row.shot_id,
    sceneScale: row.scene_scale || "source",
    visualDescription: row.visual_description,
    characterAction: row.character_action,
    dialogue: row.dialogue || "（无）",
    prompt: row.prompt_text || "prompt_text gated",
    durationSeconds: row.duration_seconds,
  };
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

function formatExportBundleMessage(response: ExportBundleResponse) {
  const readyArtifacts = response.artifacts.filter((artifact) => artifact.ready);
  const readyWithPath = readyArtifacts.filter((artifact) => artifact.artifact_path);
  const prefix = readyWithPath.length ? "导出已生成" : "export_bundle 返回";
  return `${prefix}：${response.export_manifest_id} / ${readyArtifacts.length}/${response.artifacts.length} ready / ${response.export_status.status}`;
}

function formatArtifact(artifact: ExportArtifactRecord) {
  const ready = artifact.ready ? "ready" : `blocked:${artifact.blocked_reason ?? "unknown"}`;
  const path = artifact.artifact_path ? ` path=${artifact.artifact_path}` : "";
  const hash = artifact.content_hash ? ` hash=${artifact.content_hash}` : "";
  const rows = artifact.row_count != null ? ` rows=${artifact.row_count}` : "";
  return `${artifact.artifact_kind} ${ready}${path}${hash}${rows}`;
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
