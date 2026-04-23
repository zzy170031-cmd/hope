import { useEffect, useMemo, useState } from "react";
import { Shell } from "./components/Shell";
import logoUrl from "./assets/hope-desktop-logo.png";
import { ROUTES, resolveRoute } from "./routes";
import type { SceneFusionOption, StoryboardWorkbenchRow, ViewId, WorkbenchModelId } from "./types";

type HeaderPanel = "none" | "docs" | "api";

interface Option<T extends string> {
  value: T;
  label: string;
}

const PAGE_SIZE = 5;
const DURATION_OPTIONS = [15, 30, 45, 60];
const DEFAULT_SYNOPSIS = "主角在废墟城市中与敌人激烈战斗，最终觉醒新力量，击败敌人。";
const DEFAULT_SCENE: SceneFusionOption = "热血战斗";
const DEFAULT_TASK_NAME = "第一集分镜生成";

const MODEL_OPTIONS: Array<Option<WorkbenchModelId>> = [
  { value: "gpt_4o", label: "GPT-4o" },
  { value: "gpt_4_1_mini", label: "GPT-4.1 mini" },
  { value: "hope_storyboard_mode", label: "Hope 工作台模式" },
];

const SCENE_OPTIONS: Array<Option<SceneFusionOption>> = [
  { value: "热血战斗", label: "热血战斗" },
  { value: "悬疑追踪", label: "悬疑追踪" },
  { value: "都市奇幻", label: "都市奇幻" },
  { value: "校园日常", label: "校园日常" },
  { value: "治愈成长", label: "治愈成长" },
];

const API_DOC_ITEMS = [
  "当前页面是桌面工作台的受控交互界面，重点用于脚本整理、分镜生成与本地导出。",
  "模型选择、脚本扩写和分镜生成都服务于当前工作台体验，不代表实时模型已接通。",
  "导出动作仅作用于当前页面中的本地结果，不代表正式导出链路已经上线。",
];

const API_INTERFACE_ITEMS = [
  "当前版本保留可点击交互，但不宣称已接入实时 Qwen。",
  "当前版本不宣称已接入实时 Seedance，也不代表外部引用链路已产品化完成。",
  "当前版本不宣称 V120 已直接导入 Hope runtime。",
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
  const [selectedModel, setSelectedModel] = useState<WorkbenchModelId>("gpt_4o");
  const [selectedScene, setSelectedScene] = useState<SceneFusionOption>(DEFAULT_SCENE);
  const [synopsis, setSynopsis] = useState(DEFAULT_SYNOPSIS);
  const [expandedScript, setExpandedScript] = useState(() => buildExpandedScript(DEFAULT_SYNOPSIS, DEFAULT_SCENE));
  const [taskName, setTaskName] = useState(DEFAULT_TASK_NAME);
  const [taskSourceScript, setTaskSourceScript] = useState(() => buildExpandedScript(DEFAULT_SYNOPSIS, DEFAULT_SCENE));
  const [durationSeconds, setDurationSeconds] = useState(15);
  const [rows, setRows] = useState<StoryboardWorkbenchRow[]>(() =>
    buildStoryboardRows({
      taskName: DEFAULT_TASK_NAME,
      scene: DEFAULT_SCENE,
      sourceScript: buildExpandedScript(DEFAULT_SYNOPSIS, DEFAULT_SCENE),
      durationSeconds: 15,
      rowCount: 10,
    }),
  );
  const [currentPage, setCurrentPage] = useState(1);
  const [jumpPage, setJumpPage] = useState("1");
  const [editingRowId, setEditingRowId] = useState<string | null>(null);
  const [taskSerial, setTaskSerial] = useState(1);
  const [exportMessage, setExportMessage] = useState("当前结果来自桌面工作台本地生成。");

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

  const handleExpandScript = () => {
    const nextScript = buildExpandedScript(synopsis, selectedScene);
    setExpandedScript(nextScript);
    setExportMessage("扩写脚本已更新，可导入任务继续生成分镜。");
  };

  const handleNewTask = () => {
    const nextSerial = taskSerial + 1;
    setTaskSerial(nextSerial);
    setTaskName(`第 ${nextSerial} 个任务`);
    setTaskSourceScript("");
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
    setExportMessage("扩写脚本已导入当前任务，可以开始生成。");
  };

  const handleGenerate = () => {
    if (!canGenerate) {
      return;
    }

    const source = taskSourceScript.trim() || expandedScript.trim() || synopsis.trim();
    const nextRows = buildStoryboardRows({
      taskName,
      scene: selectedScene,
      sourceScript: source,
      durationSeconds,
      rowCount: 10,
    });

    setRows(nextRows);
    setCurrentPage(1);
    setEditingRowId(null);
    setExportMessage(`已生成 ${nextRows.length} 条桌面工作台分镜结果。`);
  };

  const handleClear = () => {
    setTaskSourceScript("");
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

  const handleExportWords = () => {
    if (!rows.length) {
      setExportMessage("当前没有可导出的分镜词。");
      return;
    }

    const csv = buildStoryboardCsv(rows);
    triggerDownload(`${toSafeFileName(taskName)}-分镜词.csv`, csv, "text/csv;charset=utf-8");
    setExportMessage(`已导出 ${rows.length} 条分镜词。`);
  };

  const handleExportScript = () => {
    const text = buildFullScriptExport({
      synopsis,
      expandedScript,
      taskName,
      taskSourceScript,
      rows,
    });

    triggerDownload(`${toSafeFileName(taskName)}-完整脚本.txt`, text, "text/plain;charset=utf-8");
    setExportMessage("已导出完整脚本。");
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
            </div>
          </header>

          {activePanel !== "none" ? (
            <section className="top-panel">
              <div className="top-panel__title">{activePanel === "docs" ? "API文档" : "API接口"}</div>
              <ul>
                {(activePanel === "docs" ? API_DOC_ITEMS : API_INTERFACE_ITEMS).map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
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
                  {SCENE_OPTIONS.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </label>
              <textarea
                className="synopsis-input"
                value={synopsis}
                onChange={(event) => setSynopsis(event.target.value)}
                placeholder="请输入故事梗概"
              />
              <button type="button" className="action-button action-button--dark" onClick={handleExpandScript}>
                扩写脚本
              </button>
            </div>
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
                disabled={!canImportScript}
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

              <button type="button" className="action-button action-button--light" onClick={handleClear}>
                清空
              </button>
              <button
                type="button"
                className="action-button action-button--dark"
                onClick={handleGenerate}
                disabled={!canGenerate}
              >
                开始生成
              </button>
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
            <div className="export-actions">
              <button type="button" className="action-button action-button--light" onClick={handleExportWords}>
                导出分镜词
              </button>
              <button type="button" className="action-button action-button--dark" onClick={handleExportScript}>
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

function buildExpandedScript(synopsis: string, scene: SceneFusionOption) {
  const cleanSynopsis = synopsis.trim() || DEFAULT_SYNOPSIS;

  return [
    `${cleanSynopsis}`,
    `镜头重点围绕「${scene}」展开，先建立空间关系，再推进角色冲突。`,
    "中段加入动作变化和情绪升级，让主角与对手的目标更明确。",
    "结尾给出力量反转或节奏收束，为下一页分镜保留明确出口。",
  ].join("\n");
}

function buildStoryboardRows({
  taskName,
  scene,
  sourceScript,
  durationSeconds,
  rowCount,
}: {
  taskName: string;
  scene: SceneFusionOption;
  sourceScript: string;
  durationSeconds: number;
  rowCount: number;
}) {
  const segments = sourceScript
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  const safeSegments = segments.length ? segments : [DEFAULT_SYNOPSIS];
  const sceneScales = ["全景", "中景", "近景", "特写", "中景"];
  const people = ["主角", "敌人", "主角", "主角", "敌人"];
  const baseDuration = Math.max(1, Math.floor(durationSeconds / rowCount));
  const remainder = Math.max(0, durationSeconds - baseDuration * rowCount);
  const promptBase = {
    热血战斗: "dynamic anime fight, ruined city, energy burst",
    悬疑追踪: "mystery alley, tense shadows, cinematic tracking",
    都市奇幻: "urban fantasy, glowing sigils, dramatic skyline",
    校园日常: "school corridor, bright daylight, youth drama",
    治愈成长: "soft light, emotional close-up, warm frame",
  } satisfies Record<SceneFusionOption, string>;

  return Array.from({ length: rowCount }, (_, index) => {
    const segment = safeSegments[index % safeSegments.length];
    const person = people[index % people.length];
    const sceneScale = sceneScales[index % sceneScales.length];
    const shotNumber = index + 1;

    return {
      id: createRowId(),
      order: shotNumber,
      person,
      shot: `镜头${shotNumber}`,
      sceneScale,
      visualDescription: buildVisualDescription(segment, person, sceneScale),
      characterAction: person === "主角"
        ? buildHeroAction(shotNumber, taskName)
        : buildEnemyAction(shotNumber),
      dialogue: person === "主角"
        ? shotNumber % 2 === 0
          ? "主角：这一次，我不会退后。"
          : "（无）"
        : shotNumber % 2 === 0
          ? "敌人：你以为这样就能结束吗？"
          : "敌人：不可能！",
      prompt: `${promptBase[scene]}, shot ${shotNumber}, ${sceneScale.toLowerCase()}`,
      durationSeconds: baseDuration + (index < remainder ? 1 : 0),
    } satisfies StoryboardWorkbenchRow;
  });
}

function buildVisualDescription(segment: string, person: string, sceneScale: string) {
  const clean = segment
    .replace(/。/g, "")
    .replace(/，/g, " ")
    .trim()
    .slice(0, 20);

  return `${clean || "废墟街区对峙"}，${person}处于${sceneScale}构图。`;
}

function buildHeroAction(shotNumber: number, taskName: string) {
  const actions = [
    "主角抬头观察四周。",
    "主角蓄力前冲。",
    "主角稳住呼吸准备反击。",
    "主角释放力量完成压制。",
    `主角继续推进 ${taskName} 的关键动作。`,
  ];

  return actions[(shotNumber - 1) % actions.length];
}

function buildEnemyAction(shotNumber: number) {
  const actions = [
    "敌人冷笑并摆出架势。",
    "敌人挥臂逼近主角。",
    "敌人被压制后短暂后退。",
    "敌人试图重新集结力量。",
    "敌人制造下一次冲击。",
  ];

  return actions[(shotNumber - 1) % actions.length];
}

function buildStoryboardCsv(rows: StoryboardWorkbenchRow[]) {
  const header = ["序号", "人物", "镜头", "景别", "画面描述", "角色动作", "对白/旁白", "分镜提示词", "时长(秒)"];
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
    .map((record) => record.map((value) => `"${String(value).replaceAll('"', '""')}"`).join(","))
    .join("\n");
}

function buildFullScriptExport({
  synopsis,
  expandedScript,
  taskName,
  taskSourceScript,
  rows,
}: {
  synopsis: string;
  expandedScript: string;
  taskName: string;
  taskSourceScript: string;
  rows: StoryboardWorkbenchRow[];
}) {
  return [
    `任务名称：${taskName}`,
    "",
    "【故事梗概】",
    synopsis.trim() || "暂无",
    "",
    "【扩写脚本】",
    expandedScript.trim() || "暂无",
    "",
    "【导入脚本】",
    taskSourceScript.trim() || "暂无",
    "",
    "【分镜结果】",
    ...rows.map(
      (row) => `${row.order}. ${row.person} / ${row.shot} / ${row.sceneScale}\n画面描述：${row.visualDescription}\n角色动作：${row.characterAction}\n对白/旁白：${row.dialogue}\n分镜提示词：${row.prompt}\n时长：${row.durationSeconds} 秒`,
    ),
  ].join("\n");
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
