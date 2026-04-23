
import { useEffect, useMemo, useState } from "react";
import { EmptyState } from "./components/EmptyState";
import { LoadingCard } from "./components/LoadingCard";
import { Shell } from "./components/Shell";
import { ROUTES, resolveRoute } from "./routes";
import {
  HOPE_TAURI_COMMANDS,
  getHopeBridgeStatus,
  invokeHopeCommand,
  loadAppShellReadonlyStatus,
  loadExportValidationSnapshot,
  loadPreviewSnapshot,
  loadProjectList,
  loadWriterSnapshot,
} from "./bridge/hopeBridge";
import type {
  AppShellReadonlyStatus,
  PreviewItem,
  ProjectSummary,
  SceneFusionOption,
  StoryboardWorkbenchRow,
  ValidationExportPanelSnapshot,
  ViewId,
  WorkbenchModelId,
  WriterLayerSnapshot,
} from "./types";

interface AsyncState<T> {
  status: "loading" | "ready" | "error";
  data: T | null;
  error: string | null;
}

interface WorkbenchOption<T extends string> {
  value: T;
  label: string;
  detail: string;
}

const PAGE_SIZE = 5;

const MODEL_OPTIONS: Array<WorkbenchOption<WorkbenchModelId>> = [
  {
    value: "hope_desktop_default",
    label: "Hope 桌面默认",
    detail: "当前包内的受控工作台会话，不代表真实模型已切换。",
  },
  {
    value: "qwen_contract_shell",
    label: "Qwen 合同态",
    detail: "只代表输入与工作流口径，不触发 live Qwen 调用。",
  },
  {
    value: "seedance_overlay_proposal",
    label: "Seedance 导出预案",
    detail: "只代表导出方向选择，不接真实 Seedance。",
  },
];

const SCENE_OPTIONS: Array<WorkbenchOption<SceneFusionOption>> = [
  {
    value: "悬疑开场",
    label: "悬疑开场",
    detail: "适合建立氛围与问题线索。",
  },
  {
    value: "人物对话",
    label: "人物对话",
    detail: "适合承接角色关系和信息对齐。",
  },
  {
    value: "动作推进",
    label: "动作推进",
    detail: "适合推动节奏和镜头调度。",
  },
  {
    value: "情绪转场",
    label: "情绪转场",
    detail: "适合放大氛围变化和镜头呼吸。",
  },
  {
    value: "结尾收束",
    label: "结尾收束",
    detail: "适合收口或抛出下一段交接。",
  },
];

const DURATION_OPTIONS = [15, 30, 45, 60, 90];

const API_DOC_ITEMS = [
  "当前工作台是真实前端交互，不是假装 live Qwen / Seedance。",
  "脚本扩写与分镜产出来自桌面受控工作流，不扩大后端范围。",
  "导出只基于当前工作台前端状态，不宣称接通正式 exporter。",
  "KB / runtime / IPC 边界继续冻结，当前包只做桌面 UI 重构。",
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

function useAsyncCommand<T>(loader: () => Promise<T>, deps: unknown[]): AsyncState<T> {
  const [state, setState] = useState<AsyncState<T>>({
    status: "loading",
    data: null,
    error: null,
  });

  useEffect(() => {
    let alive = true;
    setState({ status: "loading", data: null, error: null });

    loader()
      .then((data) => {
        if (alive) {
          setState({ status: "ready", data, error: null });
        }
      })
      .catch((error: unknown) => {
        if (alive) {
          setState({
            status: "error",
            data: null,
            error: error instanceof Error ? error.message : "Unknown error",
          });
        }
      });

    return () => {
      alive = false;
    };
  }, deps);

  return state;
}

export function App() {
  useWorkbenchRoute();

  const bridgeStatus = useMemo(() => getHopeBridgeStatus(), []);
  const projects = useAsyncCommand(loadProjectList, []);
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const selectedProject = useMemo(
    () => projects.data?.find((project) => project.id === selectedProjectId) ?? null,
    [projects.data, selectedProjectId],
  );

  useEffect(() => {
    if (!selectedProjectId && projects.status === "ready" && projects.data?.length) {
      setSelectedProjectId(projects.data[0].id);
    }
  }, [projects.data, projects.status, selectedProjectId]);

  const readonlyStatus = useAsyncCommand(loadAppShellReadonlyStatus, []);
  const writerSnapshot = useAsyncCommand(
    () => loadWriterSnapshot(selectedProjectId ?? undefined),
    [selectedProjectId],
  );
  const previewSnapshot = useAsyncCommand(
    () => loadPreviewSnapshot(selectedProjectId ?? undefined),
    [selectedProjectId],
  );
  const validationSnapshot = useAsyncCommand(
    () => loadExportValidationSnapshot(selectedProjectId ?? undefined),
    [selectedProjectId],
  );

  const [selectedModel, setSelectedModel] = useState<WorkbenchModelId>("hope_desktop_default");
  const [activePanel, setActivePanel] = useState<"none" | "docs" | "api">("none");
  const [selectedScenes, setSelectedScenes] = useState<SceneFusionOption[]>([
    "悬疑开场",
    "人物对话",
    "动作推进",
  ]);
  const [synopsis, setSynopsis] = useState("");
  const [expandedScript, setExpandedScript] = useState("");
  const [taskName, setTaskName] = useState("任务 01");
  const [taskSourceScript, setTaskSourceScript] = useState("");
  const [durationSeconds, setDurationSeconds] = useState(45);
  const [storyboardRows, setStoryboardRows] = useState<StoryboardWorkbenchRow[]>([]);
  const [currentPage, setCurrentPage] = useState(1);
  const [editingRowId, setEditingRowId] = useState<string | null>(null);
  const [taskCounter, setTaskCounter] = useState(1);
  const [exportMessage, setExportMessage] = useState("尚未导出");

  const editingRow = useMemo(
    () => storyboardRows.find((row) => row.id === editingRowId) ?? null,
    [storyboardRows, editingRowId],
  );

  const pageCount = Math.max(1, Math.ceil(storyboardRows.length / PAGE_SIZE));

  useEffect(() => {
    setCurrentPage((page) => Math.min(page, pageCount));
  }, [pageCount]);

  const pagedRows = useMemo(() => {
    const start = (currentPage - 1) * PAGE_SIZE;
    return storyboardRows.slice(start, start + PAGE_SIZE);
  }, [currentPage, storyboardRows]);

  const modelOption =
    MODEL_OPTIONS.find((option) => option.value === selectedModel) ?? MODEL_OPTIONS[0];

  const canExpandScript = synopsis.trim().length > 0 && selectedScenes.length > 0;
  const canImportScript = expandedScript.trim().length > 0;
  const canGenerate = taskName.trim().length > 0 && taskSourceScript.trim().length > 0;
  const previewCounts = summarizePreview(previewSnapshot.data);
  const writerLengths = summarizeWriterLengths(writerSnapshot.data);
  const validationSummary = summarizeValidation(validationSnapshot.data);

  const handleProjectChange = (projectId: string) => {
    setSelectedProjectId(projectId);
    void invokeHopeCommand(HOPE_TAURI_COMMANDS.projectCreateOrSwitch, {
      project_id: projectId,
    });
  };

  const handleExpandScript = () => {
    if (!canExpandScript) {
      return;
    }

    setExpandedScript(
      buildExpandedScript({
        projectName: displayProjectName(selectedProject, projects.data ?? []),
        synopsis,
        scenes: selectedScenes,
        modelLabel: modelOption.label,
        writerSnapshot: writerSnapshot.data,
      }),
    );
  };

  const handleImportScript = () => {
    if (!canImportScript) {
      return;
    }

    setTaskSourceScript(expandedScript.trim());
    setExportMessage("已将扩写脚本导入当前任务，等待开始生成。");
  };

  const handleNewTask = () => {
    const nextCounter = taskCounter + 1;
    setTaskCounter(nextCounter);
    setTaskName(`任务 ${String(nextCounter).padStart(2, "0")}`);
    setTaskSourceScript("");
    setStoryboardRows([]);
    setCurrentPage(1);
    setEditingRowId(null);
    setExportMessage("已创建新任务，等待导入扩写剧本。");
  };
  const handleClearTask = () => {
    setTaskSourceScript("");
    setStoryboardRows([]);
    setCurrentPage(1);
    setEditingRowId(null);
    setExportMessage("当前任务已清空。");
  };

  const handleGenerateStoryboard = () => {
    if (!canGenerate) {
      return;
    }

    const nextRows = buildStoryboardRows({
      taskName: taskName.trim(),
      sourceScript: taskSourceScript,
      scenes: selectedScenes,
      durationSeconds,
      previewCounts,
      projectName: displayProjectName(selectedProject, projects.data ?? []),
    });

    setStoryboardRows(nextRows);
    setCurrentPage(1);
    setEditingRowId(nextRows[0]?.id ?? null);
    setExportMessage(`已生成 ${nextRows.length} 条桌面受控分镜结果。`);
  };

  const handleEditField = (field: keyof StoryboardWorkbenchRow, value: string | number) => {
    if (!editingRow) {
      return;
    }

    setStoryboardRows((current) =>
      current.map((row) =>
        row.id === editingRow.id
          ? {
              ...row,
              [field]: value,
            }
          : row,
      ),
    );
  };

  const handleDuplicateRow = (rowId: string) => {
    setStoryboardRows((current) => {
      const index = current.findIndex((row) => row.id === rowId);
      if (index === -1) {
        return current;
      }

      const row = current[index];
      const duplicated: StoryboardWorkbenchRow = {
        ...row,
        id: createRowId(),
        prompt: `${row.prompt}；复制版本用于继续调整`,
      };

      const next = [...current];
      next.splice(index + 1, 0, duplicated);
      return renumberRows(next);
    });
  };

  const handleDeleteRow = (rowId: string) => {
    setStoryboardRows((current) => renumberRows(current.filter((row) => row.id !== rowId)));
    if (editingRowId === rowId) {
      setEditingRowId(null);
    }
  };

  const handleExportStoryboard = () => {
    if (!storyboardRows.length) {
      setExportMessage("当前还没有可导出的分镜词。");
      return;
    }

    const csv = buildStoryboardCsv(storyboardRows);
    triggerDownload(`${toSafeFileName(taskName)}-分镜词.csv`, csv, "text/csv;charset=utf-8");
    setExportMessage(`已导出 ${storyboardRows.length} 条分镜词到本地下载。`);
  };

  const handleExportScript = () => {
    const text = buildFullScriptExport({
      projectName: displayProjectName(selectedProject, projects.data ?? []),
      taskName,
      synopsis,
      expandedScript,
      taskSourceScript,
      rows: storyboardRows,
    });
    triggerDownload(`${toSafeFileName(taskName)}-完整脚本.txt`, text, "text/plain;charset=utf-8");
    setExportMessage("已导出当前完整脚本到本地下载。");
  };

  return (
    <Shell>
      <div className="hope-workbench">
        <header className="topbar">
          <div className="topbar__brand">
            <div>
              <p className="topbar__eyebrow">Hope</p>
              <h1>Hope 工作台</h1>
            </div>
            <p className="topbar__subtitle">桌面端单页工作台，围绕脚本整理、分镜产出与本地导出展开。</p>
          </div>

          <div className="topbar__controls">
            <label className="topbar__select">
              <span>模型选择</span>
              <select
                value={selectedModel}
                onChange={(event) => setSelectedModel(event.target.value as WorkbenchModelId)}
              >
                {MODEL_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            <button
              type="button"
              className={activePanel === "docs" ? "topbar__button topbar__button--active" : "topbar__button"}
              onClick={() => setActivePanel((current) => (current === "docs" ? "none" : "docs"))}
            >
              API 文档
            </button>
            <button
              type="button"
              className={activePanel === "api" ? "topbar__button topbar__button--active" : "topbar__button"}
              onClick={() => setActivePanel((current) => (current === "api" ? "none" : "api"))}
            >
              API 接口
            </button>
          </div>
        </header>

        {activePanel !== "none" ? (
          <section className="helper-panel">
            {activePanel === "docs" ? (
              <>
                <div className="helper-panel__header">
                  <h2>API 文档</h2>
                  <p>{modelOption.detail}</p>
                </div>
                <div className="helper-panel__chips">
                  {API_DOC_ITEMS.map((item) => (
                    <span key={item} className="helper-chip">
                      {item}
                    </span>
                  ))}
                </div>
              </>
            ) : (
              <>
                <div className="helper-panel__header">
                  <h2>API 接口</h2>
                  <p>当前只展示已经存在的桌面桥接命令与环境口径，不新增任何 contract。</p>
                </div>
                <div className="helper-panel__grid">
                  <InfoTile label="当前环境" value={bridgeStatus.label} detail={bridgeStatus.detail} />
                  <InfoTile label="项目入口" value={HOPE_TAURI_COMMANDS.projectCreateOrSwitch} detail="项目切换仍走现有 desktop invoke。" />
                  <InfoTile label="Writer 快照" value={HOPE_TAURI_COMMANDS.writerEntrySnapshot} detail="只读取当前已存在的 writer snapshot。" />
                  <InfoTile label="Preview 快照" value={HOPE_TAURI_COMMANDS.storyboardRenderSegmentCutPreviewSnapshot} detail="只读取当前 preview snapshot，不扩新能力。" />
                  <InfoTile label="Export 快照" value={HOPE_TAURI_COMMANDS.validationExportPanelSnapshot} detail="只读取 validation / export snapshot。" />
                </div>
              </>
            )}
          </section>
        ) : null}

        <section className="context-strip">
          <label className="context-strip__project">
            <span>当前项目</span>
            <select
              value={selectedProjectId ?? ""}
              onChange={(event) => handleProjectChange(event.target.value)}
              disabled={projects.status !== "ready" || !(projects.data?.length)}
            >
              {(projects.data ?? []).map((project, index) => (
                <option key={project.id} value={project.id}>
                  {displayProjectName(project, projects.data ?? [], index)}
                </option>
              ))}
            </select>
          </label>
          <div className="context-strip__chips">
            <span className="context-chip">会话模型：{modelOption.label}</span>
            <span className="context-chip">工作任务：{taskName}</span>
            <span className="context-chip">导出状态：{exportMessage}</span>
          </div>
        </section>

        <section className="card script-panel">
          <div className="section-header">
            <div>
              <p className="section-header__eyebrow">脚本区</p>
              <h2>脚本区</h2>
              <p>融合场景选择、故事梗概与扩写脚本结果都在这一块完成。</p>
            </div>
          </div>

          <div className="script-panel__layout">
            <div className="script-panel__main">
              <div className="field-block">
                <div className="field-block__header">
                  <strong>融合场景选择</strong>
                  <span>可多选，用于塑造当前工作台的脚本口径。</span>
                </div>
                <div className="scene-chip-list">
                  {SCENE_OPTIONS.map((option) => {
                    const active = selectedScenes.includes(option.value);
                    return (
                      <button
                        key={option.value}
                        type="button"
                        className={active ? "scene-chip scene-chip--active" : "scene-chip"}
                        onClick={() => toggleSceneOption(option.value, setSelectedScenes)}
                      >
                        <span>{option.label}</span>
                        <small>{option.detail}</small>
                      </button>
                    );
                  })}
                </div>
              </div>

              <div className="field-block">
                <div className="field-block__header">
                  <strong>故事梗概</strong>
                  <span>这里承接当前项目的梗概输入，不伪装成 live generation。</span>
                </div>
                <textarea
                  className="workbench-textarea"
                  value={synopsis}
                  onChange={(event) => setSynopsis(event.target.value)}
                  placeholder="输入故事梗概，例如：深夜雨巷里，一名新记者在追查一段被刻意隐藏的失踪线索。"
                />
              </div>

              <div className="script-panel__actions">
                <button type="button" className="primary-button" disabled={!canExpandScript} onClick={handleExpandScript}>
                  扩写脚本
                </button>
                <span className="field-note">
                  当前扩写结果来自桌面受控工作台整理，不宣称 KB / Qwen 已真实接通。
                </span>
              </div>

              <div className="field-block">
                <div className="field-block__header">
                  <strong>扩写脚本结果</strong>
                  <span>可以继续手动调整，再导入任务条开始生成分镜。</span>
                </div>
                <textarea
                  className="workbench-textarea workbench-textarea--tall"
                  value={expandedScript}
                  onChange={(event) => setExpandedScript(event.target.value)}
                  placeholder="点击“扩写脚本”后，这里会生成当前工作台可继续编辑的扩写结果。"
                />
              </div>
            </div>
            <aside className="script-panel__side">
              <ReadonlyCard status={readonlyStatus} />
              <PackagedSourceCard
                title="当前快照输入"
                description="区分 writer / preview / validation 这三类已打包来源。"
              >
                <SourceMetric label="Writer 快照" value={writerSnapshot.status === "ready" ? "已读取" : writerSnapshot.status === "loading" ? "读取中" : "暂不可用"} detail={`梗概 ${writerLengths.synopsis} 字 · 故事 ${writerLengths.story} 字 · 剧本 ${writerLengths.screenplay} 字`} />
                <SourceMetric label="Preview 快照" value={previewSnapshot.status === "ready" ? "已读取" : previewSnapshot.status === "loading" ? "读取中" : "暂不可用"} detail={`Storyboard ${previewCounts.storyboard} · RenderSegment ${previewCounts.renderSegment} · Cuts ${previewCounts.cuts}`} />
                <SourceMetric label="Validation 快照" value={validationSnapshot.status === "ready" ? "已读取" : validationSnapshot.status === "loading" ? "读取中" : "暂不可用"} detail={`摘要 ${validationSummary.summaryCount} 项 · 修复建议 ${validationSummary.repairCount} 项`} />
              </PackagedSourceCard>
              <PackagedSourceCard
                title="当前工作台结果"
                description="前端本地状态与已读取桌面快照分开维护。"
              >
                <SourceMetric label="扩写脚本" value={expandedScript.trim() ? "已整理" : "待整理"} detail={`${countLines(expandedScript)} 行可继续编辑`} />
                <SourceMetric label="任务脚本" value={taskSourceScript.trim() ? "已导入" : "未导入"} detail={`${countLines(taskSourceScript)} 行参与当前任务`} />
                <SourceMetric label="分镜产出" value={storyboardRows.length ? `${storyboardRows.length} 条` : "尚未生成"} detail={`当前第 ${Math.min(currentPage, pageCount)} / ${pageCount} 页`} />
              </PackagedSourceCard>
            </aside>
          </div>
        </section>

        <section className="card task-strip">
          <div className="section-header section-header--compact">
            <div>
              <p className="section-header__eyebrow">任务条</p>
              <h2>任务条</h2>
            </div>
          </div>

          <div className="task-strip__grid">
            <label className="field-inline">
              <span>任务名</span>
              <input value={taskName} onChange={(event) => setTaskName(event.target.value)} />
            </label>

            <button type="button" className="secondary-button" onClick={handleNewTask}>
              新建任务
            </button>

            <button type="button" className="secondary-button" disabled={!canImportScript} onClick={handleImportScript}>
              导入扩写剧本
            </button>

            <label className="field-inline">
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

            <button type="button" className="primary-button" disabled={!canGenerate} onClick={handleGenerateStoryboard}>
              开始生成
            </button>

            <button type="button" className="ghost-button" onClick={handleClearTask}>
              清空
            </button>
          </div>

          <div className="task-strip__note">
            <span>已导入脚本行数：{countLines(taskSourceScript)}</span>
            <span>当前分页：{Math.min(currentPage, pageCount)} / {pageCount}</span>
            <span>当前模型口径：{modelOption.label}</span>
          </div>
        </section>

        {editingRow ? (
          <section className="card row-editor">
            <div className="section-header section-header--compact">
              <div>
                <p className="section-header__eyebrow">分镜编辑</p>
                <h2>编辑当前行</h2>
              </div>
              <button type="button" className="ghost-button" onClick={() => setEditingRowId(null)}>
                关闭编辑
              </button>
            </div>
            <div className="row-editor__grid">
              <label className="field-inline">
                <span>人物</span>
                <input value={editingRow.person} onChange={(event) => handleEditField("person", event.target.value)} />
              </label>
              <label className="field-inline">
                <span>镜头</span>
                <input value={editingRow.shot} onChange={(event) => handleEditField("shot", event.target.value)} />
              </label>
              <label className="field-inline">
                <span>景别</span>
                <input value={editingRow.sceneScale} onChange={(event) => handleEditField("sceneScale", event.target.value)} />
              </label>
              <label className="field-inline field-inline--wide">
                <span>画面描述</span>
                <textarea value={editingRow.visualDescription} onChange={(event) => handleEditField("visualDescription", event.target.value)} />
              </label>
              <label className="field-inline field-inline--wide">
                <span>角色动作</span>
                <textarea value={editingRow.characterAction} onChange={(event) => handleEditField("characterAction", event.target.value)} />
              </label>
              <label className="field-inline field-inline--wide">
                <span>对白 / 旁白</span>
                <textarea value={editingRow.dialogue} onChange={(event) => handleEditField("dialogue", event.target.value)} />
              </label>
              <label className="field-inline field-inline--wide">
                <span>分镜提示词</span>
                <textarea value={editingRow.prompt} onChange={(event) => handleEditField("prompt", event.target.value)} />
              </label>
              <label className="field-inline">
                <span>时长(秒)</span>
                <input
                  type="number"
                  min={1}
                  value={editingRow.durationSeconds}
                  onChange={(event) => handleEditField("durationSeconds", Number(event.target.value))}
                />
              </label>
            </div>
          </section>
        ) : null}

        <section className="card output-panel">
          <div className="section-header">
            <div>
              <p className="section-header__eyebrow">分镜产出区</p>
              <h2>分镜产出区</h2>
              <p>这里展示桌面工作台生成后的当前结果，支持分页、编辑、复制和删除。</p>
            </div>
            <div className="output-panel__meta">
              <span className="context-chip">每页 5 行</span>
              <span className="context-chip">总计 {storyboardRows.length} 行</span>
            </div>
          </div>

          {storyboardRows.length ? (
            <>
              <div className="storyboard-table-wrap">
                <table className="storyboard-table storyboard-table--workbench">
                  <thead>
                    <tr>
                      <th>序号</th>
                      <th>人物</th>
                      <th>镜头</th>
                      <th>景别</th>
                      <th>画面描述</th>
                      <th>角色动作</th>
                      <th>对白 / 旁白</th>
                      <th>分镜提示词</th>
                      <th>时长(秒)</th>
                      <th>操作</th>
                    </tr>
                  </thead>
                  <tbody>
                    {pagedRows.map((row) => (
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
                          <div className="table-actions">
                            <button type="button" className="table-action" onClick={() => setEditingRowId(row.id)}>
                              编辑
                            </button>
                            <button type="button" className="table-action" onClick={() => handleDuplicateRow(row.id)}>
                              复制
                            </button>
                            <button type="button" className="table-action table-action--danger" onClick={() => handleDeleteRow(row.id)}>
                              删除
                            </button>
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              <div className="pagination">
                <button type="button" className="ghost-button" disabled={currentPage === 1} onClick={() => setCurrentPage((page) => Math.max(1, page - 1))}>
                  上一页
                </button>
                <div className="pagination__pages">
                  {Array.from({ length: pageCount }, (_, index) => index + 1).map((page) => (
                    <button
                      key={page}
                      type="button"
                      className={page === currentPage ? "page-button page-button--active" : "page-button"}
                      onClick={() => setCurrentPage(page)}
                    >
                      {page}
                    </button>
                  ))}
                </div>
                <button type="button" className="ghost-button" disabled={currentPage === pageCount} onClick={() => setCurrentPage((page) => Math.min(pageCount, page + 1))}>
                  下一页
                </button>
              </div>
            </>
          ) : (
            <EmptyState
              title="还没有分镜结果"
              description="先在脚本区扩写脚本，再通过任务条导入并开始生成。"
            />
          )}
        </section>

        <section className="export-bar">
          <div className="export-bar__copy">
            <strong>导出仍然保持桌面受控口径</strong>
            <p>当前只从工作台前端状态导出分镜词和完整脚本，不假装接通正式 exporter。</p>
          </div>
          <div className="export-bar__actions">
            <button type="button" className="secondary-button" onClick={handleExportStoryboard}>
              导出分镜词
            </button>
            <button type="button" className="primary-button" onClick={handleExportScript}>
              导出完整脚本
            </button>
          </div>
        </section>
      </div>
    </Shell>
  );
}

function ReadonlyCard({ status }: { status: AsyncState<AppShellReadonlyStatus> }) {
  if (status.status === "loading") {
    return (
      <div className="subcard">
        <div className="subcard__header">
          <strong>只读状态</strong>
          <span>加载中</span>
        </div>
        <LoadingCard lines={3} />
      </div>
    );
  }

  if (status.status === "error" || !status.data) {
    return (
      <div className="subcard">
        <div className="subcard__header">
          <strong>只读状态</strong>
          <span>暂不可用</span>
        </div>
        <EmptyState title="只读状态暂不可用" description="桌面工作台继续可用，但不会扩大能力声明。" />
      </div>
    );
  }

  const { snapshotBootstrap, validationFeedback } = status.data;
  return (
    <PackagedSourceCard title="只读状态" description="只展示当前打包状态摘要，不暴露 KB 原始内容。">
      <SourceMetric label="来源包" value={snapshotBootstrap.snapshotIdentity.sourceName || "桌面快照"} detail={`种子格式：${snapshotBootstrap.snapshotIdentity.seedFormat || "未知"}`} />
      <SourceMetric label="知识能力" value={`${snapshotBootstrap.knowledgeBundle.sceneTaxonomyCount} / ${snapshotBootstrap.knowledgeBundle.failurePatternCount} / ${snapshotBootstrap.knowledgeBundle.promptTemplateCount}`} detail="分别对应 taxonomy / failure pattern / prompt template 计数。" />
      <SourceMetric label="校验映射" value={validationFeedback.repairMappingReady ? "已就绪" : "待补充"} detail={`failure pattern ${validationFeedback.failurePatternCount} · prompt template ${validationFeedback.promptTemplateCount}`} />
    </PackagedSourceCard>
  );
}

function PackagedSourceCard({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: React.ReactNode;
}) {
  return (
    <div className="subcard">
      <div className="subcard__header">
        <strong>{title}</strong>
        <span>{description}</span>
      </div>
      <div className="source-metric-list">{children}</div>
    </div>
  );
}

function SourceMetric({
  label,
  value,
  detail,
}: {
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <div className="source-metric">
      <div>
        <span>{label}</span>
        <strong>{value}</strong>
      </div>
      <p>{detail}</p>
    </div>
  );
}

function InfoTile({
  label,
  value,
  detail,
}: {
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <div className="info-tile">
      <span>{label}</span>
      <strong>{value}</strong>
      <p>{detail}</p>
    </div>
  );
}

function buildExpandedScript({
  projectName,
  synopsis,
  scenes,
  modelLabel,
  writerSnapshot,
}: {
  projectName: string;
  synopsis: string;
  scenes: SceneFusionOption[];
  modelLabel: string;
  writerSnapshot: WriterLayerSnapshot | null;
}) {
  const baseline = writerSnapshot?.synopsis?.trim()
    ? "当前已读取 writer 快照梗概，可作为桌面整理参考。"
    : "当前未读取到可直接引用的 writer 梗概，脚本完全基于工作台输入整理。";

  return [
    `【项目】${projectName}`,
    `【工作台口径】${modelLabel}`,
    `【场景融合】${scenes.join(" / ")}`,
    "",
    "第一段",
    `${synopsis.trim()} 先从主问题、主人物和主空间入手，确保镜头开场就能建立清晰的注意力中心。`,
    "",
    "第二段",
    `围绕 ${scenes[0] ?? "当前场景"} 与 ${scenes[1] ?? "人物推进"} 展开，补足情绪变化、行为触发和对话动机，让后续分镜可以直接落到镜头层。`,
    "",
    "第三段",
    `在结尾保留 ${scenes[scenes.length - 1] ?? "收束交接"} 的出口，为任务条中的分镜生成提供明确的动作、对白和视觉转场依据。`,
    "",
    "工作台说明",
    baseline,
  ].join("\n");
}
function buildStoryboardRows({
  taskName,
  sourceScript,
  scenes,
  durationSeconds,
  previewCounts,
  projectName,
}: {
  taskName: string;
  sourceScript: string;
  scenes: SceneFusionOption[];
  durationSeconds: number;
  previewCounts: Record<"storyboard" | "renderSegment" | "cuts", number>;
  projectName: string;
}) {
  const lines = sourceScript
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .slice(0, 8);

  const rowCount = Math.max(6, Math.min(8, lines.length || scenes.length + 3));
  const baseDuration = Math.max(1, Math.floor(durationSeconds / rowCount));
  const remainder = Math.max(0, durationSeconds - baseDuration * rowCount);

  return Array.from({ length: rowCount }, (_, index) => {
    const line = lines[index % Math.max(1, lines.length)] ?? sourceScript.trim();
    const scene = scenes[index % Math.max(1, scenes.length)] ?? "人物对话";

    return {
      id: createRowId(),
      order: index + 1,
      person: `主角 ${index + 1}`,
      shot: `镜头 ${String(index + 1).padStart(2, "0")}`,
      sceneScale: pickSceneScale(index),
      visualDescription: `围绕「${scene}」整理当前画面：${line || `${taskName} 的第 ${index + 1} 条工作台结果`}。`,
      characterAction: `角色在 ${projectName} 的当前任务里执行第 ${index + 1} 个动作节点，保持节奏与交接清晰。`,
      dialogue: `第 ${index + 1} 条对白 / 旁白围绕任务「${taskName}」展开，避免伪装成 live 生成内容。`,
      prompt: `希望保持 ${scene} 的镜头语义，结合 storyboard ${previewCounts.storyboard} 条、cut ${previewCounts.cuts} 条的现有快照线索整理为桌面工作台结果。`,
      durationSeconds: baseDuration + (index < remainder ? 1 : 0),
    };
  });
}

function buildStoryboardCsv(rows: StoryboardWorkbenchRow[]) {
  const header = [
    "序号",
    "人物",
    "镜头",
    "景别",
    "画面描述",
    "角色动作",
    "对白 / 旁白",
    "分镜提示词",
    "时长(秒)",
  ];

  const records = rows.map((row) => [
    row.order,
    row.person,
    row.shot,
    row.sceneScale,
    row.visualDescription,
    row.characterAction,
    row.dialogue,
    row.prompt,
    row.durationSeconds,
  ]);

  return [header, ...records]
    .map((record) =>
      record
        .map((value) => `"${String(value).replaceAll('"', '""')}"`)
        .join(","),
    )
    .join("\n");
}

function buildFullScriptExport({
  projectName,
  taskName,
  synopsis,
  expandedScript,
  taskSourceScript,
  rows,
}: {
  projectName: string;
  taskName: string;
  synopsis: string;
  expandedScript: string;
  taskSourceScript: string;
  rows: StoryboardWorkbenchRow[];
}) {
  return [
    `项目：${projectName}`,
    `任务：${taskName}`,
    "",
    "【故事梗概】",
    synopsis.trim() || "暂无",
    "",
    "【扩写脚本】",
    expandedScript.trim() || "暂无",
    "",
    "【导入任务脚本】",
    taskSourceScript.trim() || "暂无",
    "",
    "【当前分镜结果】",
    ...rows.map(
      (row) =>
        `${row.order}. ${row.shot} / ${row.sceneScale} / ${row.person}\n画面：${row.visualDescription}\n动作：${row.characterAction}\n对白：${row.dialogue}\n提示词：${row.prompt}\n时长：${row.durationSeconds} 秒`,
    ),
  ].join("\n");
}

function summarizePreview(
  preview: Record<"storyboard" | "renderSegment" | "cuts", PreviewItem[]> | null,
) {
  return {
    storyboard: preview?.storyboard.length ?? 0,
    renderSegment: preview?.renderSegment.length ?? 0,
    cuts: preview?.cuts.length ?? 0,
  };
}

function summarizeWriterLengths(snapshot: WriterLayerSnapshot | null) {
  return {
    synopsis: snapshot?.synopsis.trim().length ?? 0,
    story: snapshot?.story.trim().length ?? 0,
    screenplay: snapshot?.screenplay.trim().length ?? 0,
  };
}

function summarizeValidation(snapshot: ValidationExportPanelSnapshot | null) {
  return {
    summaryCount: snapshot?.summaryItems.length ?? 0,
    repairCount: snapshot?.repairRecommendations.length ?? 0,
  };
}

function countLines(value: string) {
  if (!value.trim()) {
    return 0;
  }

  return value.split(/\r?\n/).filter((line) => line.trim()).length;
}

function toggleSceneOption(
  option: SceneFusionOption,
  setScenes: (updater: (current: SceneFusionOption[]) => SceneFusionOption[]) => void,
) {
  setScenes((existing) => {
    if (existing.includes(option)) {
      return existing.length === 1 ? existing : existing.filter((item) => item !== option);
    }

    return [...existing, option];
  });
}

function pickSceneScale(index: number) {
  return ["远景", "中景", "近景", "特写", "移动镜头"][index % 5];
}

function createRowId() {
  return `row-${Math.random().toString(36).slice(2, 10)}`;
}

function renumberRows(rows: StoryboardWorkbenchRow[]) {
  return rows.map((row, index) => ({
    ...row,
    order: index + 1,
  }));
}

function toSafeFileName(value: string) {
  return (value.trim() || "hope-workbench").replace(/[\\/:*?"<>|]/g, "-");
}

function triggerDownload(filename: string, content: string, mimeType: string) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
}

function displayProjectName(
  project: ProjectSummary | null,
  projects: ProjectSummary[],
  indexOverride?: number,
) {
  if (!project) {
    return "未选择项目";
  }

  const index = indexOverride ?? projects.findIndex((item) => item.id === project.id);
  return `项目 ${String(index + 1).padStart(2, "0")}`;
}
