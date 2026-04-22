import { type FormEvent, useEffect, useMemo, useState } from "react";
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
  ExportValidationItem,
  PreviewItem,
  ProjectSummary,
  QwenRuntimeStatus,
  StoryboardReadinessStatus,
  StoryboardSemanticGroup,
  ValidationExportPanelSnapshot,
  ValidationRepairRecommendation,
  ViewId,
  WriterInputBoundaryDraft,
  WriterInputKind,
} from "./types";

interface AsyncState<T> {
  status: "loading" | "ready" | "error";
  data: T | null;
  error: string | null;
}

const QWEN_LOCAL_CHECK_DELAY_MS = 220;

const EMPTY_WRITER_INPUT_BOUNDARY: WriterInputBoundaryDraft = {
  source_input_text: "",
  input_kind: "brief",
  story_constraints: "",
  character_constraints: "",
  style_constraints: "",
  duration_target: "",
  scene_count_hint: "",
};

const WRITER_INPUT_KIND_OPTIONS: Array<{
  value: WriterInputKind;
  label: string;
  detail: string;
}> = [
  {
    value: "brief",
    label: "Brief",
    detail: "用户短需求或创意概要",
  },
  {
    value: "synopsis",
    label: "Synopsis",
    detail: "已有故事梗概",
  },
  {
    value: "script",
    label: "Script",
    detail: "已有剧本 / 场景文本",
  },
];

const QWEN_CONTRACT_READINESS = [
  "Qwen storyboard generation contract boundary 已通过",
  "contract-only input 可承接",
  "生成链路未启用，等待后续接入",
  "API key 仅做本地输入状态，不表达真实连接",
];

const STORYBOARD_REQUIREMENT_PLACEHOLDER =
  "例：一位年轻工程师在清晨地铁里发现异常信号，节奏克制，结尾留下继续追查的悬念。";

const STORYBOARD_READINESS_FLOW: StoryboardReadinessStatus[] = [
  "草案",
  "待补全",
  "待修正",
  "校验通过",
  "可导出",
  "阻断",
];

const STORYBOARD_SEMANTIC_GROUPS: Array<{
  group: StoryboardSemanticGroup;
  status: StoryboardReadinessStatus;
  detail: string;
}> = [
  {
    group: "画面意图",
    status: "草案",
    detail: "先锁定每格镜头的视觉目标与情绪方向。",
  },
  {
    group: "运动与镜头",
    status: "待补全",
    detail: "镜头运动、景别与切换节奏等待生成链路补全。",
  },
  {
    group: "声音与对白",
    status: "待修正",
    detail: "对白与环境声只做语义占位，后续由校验链确认。",
  },
  {
    group: "连续性与交接",
    status: "校验通过",
    detail: "同一角色不同形态需保持身份、动作与场景交接一致。",
  },
  {
    group: "参考与锁定",
    status: "待补全",
    detail: "角色、场景和参考素材只做锁定提示，不做实体管理。",
  },
  {
    group: "导出就绪",
    status: "可导出",
    detail: "主交付路径固定为导出 Excel。",
  },
];

const STORYBOARD_DRAFT_ROWS: Array<{
  shot: string;
  intent: string;
  camera: string;
  sound: string;
  handoff: string;
  status: StoryboardReadinessStatus;
}> = [
  {
    shot: "镜头 01",
    intent: "建立角色处境和清晨空间气氛。",
    camera: "中景推近，保持人物与环境关系。",
    sound: "低频环境声，保留一句内心提示。",
    handoff: "交接到异常信号的首次出现。",
    status: "草案",
  },
  {
    shot: "镜头 02",
    intent: "突出异常信号带来的认知偏移。",
    camera: "近景切到屏幕反光，避免跳出叙事。",
    sound: "提示音短促，不引入额外实体管理。",
    handoff: "同一角色视线方向保持连续。",
    status: "待修正",
  },
  {
    shot: "镜头 03",
    intent: "留下继续追查的动作出口。",
    camera: "跟拍到车门开启，切点对齐动作。",
    sound: "对白留白，环境声承接下一场。",
    handoff: "导出前等待 validator 确认。",
    status: "待补全",
  },
];

function useHashView() {
  const [activeView, setActiveView] = useState<ViewId>(() => resolveRoute(window.location.hash));

  useEffect(() => {
    const handleHashChange = () => {
      setActiveView(resolveRoute(window.location.hash));
    };

    window.addEventListener("hashchange", handleHashChange);

    if (!window.location.hash) {
      window.location.hash = ROUTES[0].hash;
    }

    return () => {
      window.removeEventListener("hashchange", handleHashChange);
    };
  }, []);

  const navigate = (view: ViewId) => {
    const target = ROUTES.find((route) => route.id === view);
    if (target) {
      window.location.hash = target.hash;
    }
  };

  return { activeView, navigate };
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
  const { activeView, navigate } = useHashView();
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const bridgeStatus = useMemo(() => getHopeBridgeStatus(), []);
  const readonlyStatus = useAsyncCommand(loadAppShellReadonlyStatus, []);

  return (
    <Shell activeView={activeView} onNavigate={navigate}>
      <div className="workspace__header">
        <div>
          <p className="workspace__eyebrow">Fast Gate Repair</p>
          <h2>{ROUTES.find((route) => route.id === activeView)?.label ?? "Hope UI"}</h2>
        </div>
        <div className="workspace__status">
          <span className="workspace__chip">{bridgeStatus.label}</span>
          <span className="workspace__chip workspace__chip--soft">{bridgeStatus.detail}</span>
          <QwenRuntimeSecretControl />
        </div>
      </div>

      <AppShellReadonlyLanding status={readonlyStatus} />

      <ViewRouter
        activeView={activeView}
        selectedProjectId={selectedProjectId}
        setSelectedProjectId={setSelectedProjectId}
      />
    </Shell>
  );
}

function QwenRuntimeSecretControl() {
  const [secretInput, setSecretInput] = useState("");
  const [runtimeStatus, setRuntimeStatus] = useState<QwenRuntimeStatus>("local_missing");
  const [lastFive, setLastFive] = useState<string | null>(null);

  const isChecking = runtimeStatus === "local_checking";
  const isLocalPresent = runtimeStatus === "local_present";
  const isError = runtimeStatus === "local_invalid";
  const canSubmit = secretInput.trim().length > 0 && !isChecking;
  const statusTone =
    isError
      ? "error"
      : isChecking
        ? "checking"
        : isLocalPresent
          ? "connected"
          : "missing";
  const statusLabel =
    runtimeStatus === "local_present"
      ? "本地密钥已填写"
      : runtimeStatus === "local_checking"
        ? "本地检查中"
        : runtimeStatus === "local_invalid"
          ? "本地输入无效"
          : "未配置千问 API";
  const statusDetail =
    isLocalPresent && lastFive
      ? `仅本地保存输入状态 · *****${lastFive}`
      : isError
        ? "密钥长度不足；未调用 Qwen 服务"
        : isChecking
          ? "仅检查本地输入完整度"
          : "等待输入千问 API Key；不会发起真实调用";

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    const candidate = secretInput.trim();
    const safeLastFive = candidate.length > 5 ? candidate.slice(-5) : null;
    setSecretInput("");
    setLastFive(null);

    if (!candidate) {
      setRuntimeStatus("local_missing");
      return;
    }

    setRuntimeStatus("local_checking");
    window.setTimeout(() => {
      if (!safeLastFive) {
        setRuntimeStatus("local_invalid");
        return;
      }

      setLastFive(safeLastFive);
      setRuntimeStatus("local_present");
    }, QWEN_LOCAL_CHECK_DELAY_MS);
  };

  return (
    <form className="qwen-secret" onSubmit={handleSubmit} aria-label="千问本地连接密钥">
      <div className="qwen-secret__status" aria-live="polite">
        <span className={`qwen-secret__dot qwen-secret__dot--${statusTone}`} />
        <span className="qwen-secret__state">{statusLabel}</span>
        <span className="qwen-secret__detail">{statusDetail}</span>
      </div>
      <div className="qwen-secret__controls">
        <input
          aria-label="输入千问 API Key"
          autoComplete="new-password"
          className="qwen-secret__input"
          disabled={isChecking}
          onChange={(event) => setSecretInput(event.target.value)}
          placeholder="输入千问 API Key"
          spellCheck={false}
          type="password"
          value={secretInput}
        />
        <button className="qwen-secret__button" disabled={!canSubmit} type="submit">
          确认连接
        </button>
      </div>
    </form>
  );
}

function AppShellReadonlyLanding({
  status,
}: {
  status: AsyncState<AppShellReadonlyStatus>;
}) {
  if (status.status === "loading") {
    return (
      <section className="readonly-landing" aria-label="桌面后台状态">
        <div className="readonly-landing__intro">
          <p className="workspace__eyebrow">后台约束</p>
          <h3>产品支撑状态</h3>
          <p>正在读取后台约束状态，不展示内部知识库摘要。</p>
        </div>
        <LoadingCard lines={3} />
      </section>
    );
  }

  if (status.status === "error" || !status.data) {
    return (
      <section className="readonly-landing" aria-label="桌面后台状态">
        <div className="readonly-landing__intro">
          <p className="workspace__eyebrow">后台约束</p>
          <h3>产品支撑状态</h3>
          <p>
            后台状态暂不可用。桌面端不会展示内部路径、hash 或知识库摘要。
          </p>
        </div>
        <div className="readonly-landing__error">后台状态暂不可用。</div>
      </section>
    );
  }

  const snapshot = status.data.snapshotBootstrap;
  const validationFeedback = status.data.validationFeedback;
  const knowledgeReady =
    snapshot.summaryCapabilities.hasSceneTaxonomy &&
    snapshot.summaryCapabilities.hasFailurePatterns &&
    snapshot.summaryCapabilities.hasRepairTemplateMapping &&
    snapshot.knowledgeBundle.sceneTaxonomiesReady &&
    snapshot.knowledgeBundle.failurePatternsReady &&
    snapshot.knowledgeBundle.promptTemplatesReady;
  const validationReady =
    validationFeedback.hasFailurePatterns && validationFeedback.repairMappingReady;
  const repairReady =
    validationFeedback.hasRepairTemplateMapping && validationFeedback.repairMappingReady;

  return (
    <section className="readonly-landing" aria-label="桌面后台状态">
      <div className="readonly-landing__intro">
        <p className="workspace__eyebrow">后台约束</p>
        <h3>产品支撑状态</h3>
        <p>知识支撑与校验规则仅作为后台防偏移约束，不展示内部摘要、路径、hash 或计数。</p>
      </div>

      <div className="readonly-landing__grid">
        <StatusTile label="知识支撑" value={knowledgeReady ? "知识支撑已就绪" : "知识支撑待就绪"} />
        <StatusTile label="校验规则" value={validationReady ? "校验规则已加载" : "校验规则待加载"} />
        <StatusTile label="修复建议" value={repairReady ? "修复建议可用" : "修复建议待就绪"} />
      </div>
    </section>
  );
}

function StatusTile({
  label,
  value,
  wide = false,
}: {
  label: string;
  value: string;
  wide?: boolean;
}) {
  return (
    <div className={wide ? "status-tile status-tile--wide" : "status-tile"}>
      <span>{label}</span>
      <strong>{value || "unavailable"}</strong>
    </div>
  );
}

interface ViewRouterProps {
  activeView: ViewId;
  selectedProjectId: string | null;
  setSelectedProjectId: (projectId: string | null) => void;
}

function ViewRouter({ activeView, selectedProjectId, setSelectedProjectId }: ViewRouterProps) {
  if (activeView === "projects") {
    return (
      <ProjectsView
        selectedProjectId={selectedProjectId}
        setSelectedProjectId={setSelectedProjectId}
      />
    );
  }

  if (activeView === "writer") {
    return <WriterView selectedProjectId={selectedProjectId} />;
  }

  if (activeView === "preview") {
    return <PreviewView selectedProjectId={selectedProjectId} />;
  }

  return <ExportView selectedProjectId={selectedProjectId} />;
}

interface ProjectsViewProps {
  selectedProjectId: string | null;
  setSelectedProjectId: (projectId: string | null) => void;
}

function ProjectsView({ selectedProjectId, setSelectedProjectId }: ProjectsViewProps) {
  const projects = useAsyncCommand(loadProjectList, []);
  const selectedProject = useMemo<ProjectSummary | null>(() => {
    return projects.data?.find((project) => project.id === selectedProjectId) ?? null;
  }, [projects.data, selectedProjectId]);

  useEffect(() => {
    if (!selectedProjectId && projects.status === "ready" && projects.data?.length) {
      setSelectedProjectId(projects.data[0].id);
    }
  }, [projects.data, projects.status, selectedProjectId, setSelectedProjectId]);

  return (
    <section className="view-grid">
      <div className="panel">
        <div className="panel__header">
          <h3>项目列表</h3>
          <button
            type="button"
            className="panel__button panel__button--ghost"
            onClick={() => {
              void invokeHopeCommand(HOPE_TAURI_COMMANDS.projectCreateOrSwitch);
            }}
            title="镜像 Rust IPC contract"
          >
            新建 / 切换
          </button>
        </div>

        {projects.status === "loading" ? (
          <LoadingCard lines={4} />
        ) : projects.status === "error" ? (
          <EmptyState title="项目列表加载失败" description={projects.error ?? "请稍后重试。"} />
        ) : (
          <div className="project-list">
            {projects.data?.map((project) => (
              <button
                key={project.id}
                type="button"
                className={
                  project.id === selectedProjectId
                    ? "project-card project-card--active"
                    : "project-card"
                }
                onClick={() => {
                  setSelectedProjectId(project.id);
                  void invokeHopeCommand(HOPE_TAURI_COMMANDS.projectCreateOrSwitch, {
                    project_id: project.id,
                  });
                }}
              >
                <div className="project-card__title">
                  <strong>{project.name}</strong>
                  <span>{project.status}</span>
                </div>
                <p>更新时间：{project.updatedAt}</p>
                <p>{project.episodeCount} 个 Episode</p>
              </button>
            ))}
          </div>
        )}
      </div>

      <div className="panel">
        <div className="panel__header">
          <h3>当前项目</h3>
          <span className="panel__hint">UI 只通过 IPC 调用</span>
        </div>

        {selectedProject ? (
          <div className="detail-card">
            <strong>{selectedProject.name}</strong>
            <p>状态：{selectedProject.status}</p>
            <p>更新时间：{selectedProject.updatedAt}</p>
            <p>Episode 数：{selectedProject.episodeCount}</p>
          </div>
        ) : (
          <EmptyState title="尚未选择项目" description="先选择一个项目，再查看后续视图。" />
        )}
      </div>
    </section>
  );
}

function WriterView({ selectedProjectId }: { selectedProjectId: string | null }) {
  const writer = useAsyncCommand(
    () => loadWriterSnapshot(selectedProjectId ?? undefined),
    [selectedProjectId],
  );
  const [inputBoundary, setInputBoundary] = useState<WriterInputBoundaryDraft>(
    EMPTY_WRITER_INPUT_BOUNDARY,
  );
  const storyboardNeed = inputBoundary.source_input_text;
  const updateInputBoundary = <K extends keyof WriterInputBoundaryDraft>(
    field: K,
    value: WriterInputBoundaryDraft[K],
  ) => {
    setInputBoundary((current) => ({
      ...current,
      [field]: value,
    }));
  };

  return (
    <section className="panel">
      <div className="panel__header">
        <h3>分镜脚本工作台</h3>
        <span className="panel__hint">需求输入 -&gt; 分镜草案 -&gt; 校验 -&gt; 导出 Excel</span>
      </div>

      {!selectedProjectId ? (
        <EmptyState title="先选择项目" description="分镜工作台需要项目上下文。" />
      ) : writer.status === "loading" ? (
        <LoadingCard lines={4} />
      ) : writer.status === "error" ? (
        <EmptyState title="Writer 加载失败" description={writer.error ?? "请稍后重试。"} />
      ) : (
        <div className="storyboard-workbench">
          <aside className="storyboard-workbench__rail" aria-label="分镜参数占位">
            <div>
              <p className="workspace__eyebrow">轻量参数</p>
              <h4>风格预设</h4>
              <p>用户语义的风格组合预设，占位不展示后台规则。</p>
            </div>
            <div className="storyboard-lock-card">
              <strong>参考与锁定</strong>
              <p>角色、场景、形态变化只保留一致性提示，不做实体库管理。</p>
            </div>
            <div className="storyboard-lock-card">
              <strong>生成边界</strong>
              <p>当前仅本地占位，不触发外部生成服务。</p>
            </div>
          </aside>

          <div className="storyboard-workbench__main">
            <div className="storyboard-input-card">
              <div>
                <p className="workspace__eyebrow">用户需求</p>
                <h4>输入分镜目标</h4>
              </div>
              <textarea
                className="storyboard-input-card__field"
                onChange={(event) =>
                  updateInputBoundary("source_input_text", event.target.value)
                }
                placeholder={STORYBOARD_REQUIREMENT_PLACEHOLDER}
                value={storyboardNeed}
              />
              <div className="storyboard-input-card__actions">
                <span>本包只做 Writer 输入边界承接，不触发真实 Qwen 生成。</span>
                <button className="panel__button" disabled type="button">
                  生成分镜草案（待接入）
                </button>
              </div>
            </div>

            <WriterInputBoundaryPanel
              inputBoundary={inputBoundary}
              onChange={updateInputBoundary}
            />
            <QwenContractReadinessCard inputBoundary={inputBoundary} />
            <ReadinessStrip />
            <StoryboardSemanticGrid />
            <StoryboardDraftTable />
            <PromptPreview storyboardNeed={storyboardNeed} />

            <div className="storyboard-revision-card">
              <div>
                <p className="workspace__eyebrow">全局修改</p>
                <h4>底部修改区</h4>
              </div>
              <textarea
                className="storyboard-input-card__field"
                placeholder="例：强化第三个镜头的悬念，但不要改变角色身份和场景连续性。"
              />
            </div>

            <div className="stack">
              <LayerCard title="Synopsis" text={writer.data?.synopsis ?? ""} />
              <LayerCard title="Story" text={writer.data?.story ?? ""} />
              <LayerCard title="Screenplay" text={writer.data?.screenplay ?? ""} />
              <LayerCard title="Storyboard" text={writer.data?.storyboard ?? ""} />
            </div>
          </div>
        </div>
      )}
    </section>
  );
}

interface WriterInputBoundaryPanelProps {
  inputBoundary: WriterInputBoundaryDraft;
  onChange: <K extends keyof WriterInputBoundaryDraft>(
    field: K,
    value: WriterInputBoundaryDraft[K],
  ) => void;
}

function WriterInputBoundaryPanel({
  inputBoundary,
  onChange,
}: WriterInputBoundaryPanelProps) {
  return (
    <section className="writer-boundary-panel" aria-label="Writer input boundary">
      <div className="writer-boundary-panel__header">
        <div>
          <p className="workspace__eyebrow">Input Boundary</p>
          <h4>Qwen contract-only 输入承接</h4>
        </div>
        <span className="readiness-pill readiness-pill--pending">等待后续接入</span>
      </div>

      <div className="writer-boundary-panel__grid">
        <label className="writer-boundary-field">
          <span>input_kind</span>
          <select
            value={inputBoundary.input_kind}
            onChange={(event) =>
              onChange("input_kind", event.target.value as WriterInputKind)
            }
          >
            {WRITER_INPUT_KIND_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label} - {option.detail}
              </option>
            ))}
          </select>
        </label>

        <label className="writer-boundary-field">
          <span>duration_target</span>
          <input
            value={inputBoundary.duration_target}
            onChange={(event) => onChange("duration_target", event.target.value)}
            placeholder="例：60 秒 / 3 分钟 / 单集开场"
          />
        </label>

        <label className="writer-boundary-field">
          <span>scene_count_hint</span>
          <input
            value={inputBoundary.scene_count_hint}
            onChange={(event) => onChange("scene_count_hint", event.target.value)}
            placeholder="例：3-5 个场景"
          />
        </label>

        <label className="writer-boundary-field writer-boundary-field--wide">
          <span>story_constraints</span>
          <textarea
            value={inputBoundary.story_constraints}
            onChange={(event) => onChange("story_constraints", event.target.value)}
            placeholder="只记录用户故事约束，不生成分镜内容。"
          />
        </label>

        <label className="writer-boundary-field writer-boundary-field--wide">
          <span>character_constraints</span>
          <textarea
            value={inputBoundary.character_constraints}
            onChange={(event) => onChange("character_constraints", event.target.value)}
            placeholder="只记录角色一致性要求，不建立实体库。"
          />
        </label>

        <label className="writer-boundary-field writer-boundary-field--wide">
          <span>style_constraints</span>
          <textarea
            value={inputBoundary.style_constraints}
            onChange={(event) => onChange("style_constraints", event.target.value)}
            placeholder="只记录风格方向，不调用 Qwen 或 Seedance。"
          />
        </label>
      </div>
    </section>
  );
}

function QwenContractReadinessCard({
  inputBoundary,
}: {
  inputBoundary: WriterInputBoundaryDraft;
}) {
  const readinessFields = [
    {
      label: "source_input_text",
      ready: inputBoundary.source_input_text.trim().length > 0,
    },
    { label: "input_kind", ready: Boolean(inputBoundary.input_kind) },
    {
      label: "story_constraints",
      ready: inputBoundary.story_constraints.trim().length > 0,
    },
    {
      label: "character_constraints",
      ready: inputBoundary.character_constraints.trim().length > 0,
    },
    {
      label: "style_constraints",
      ready: inputBoundary.style_constraints.trim().length > 0,
    },
    {
      label: "duration_target",
      ready: inputBoundary.duration_target.trim().length > 0,
    },
    {
      label: "scene_count_hint",
      ready: inputBoundary.scene_count_hint.trim().length > 0,
    },
  ];
  const completedCount = readinessFields.filter((field) => field.ready).length;

  return (
    <section className="qwen-readiness-card" aria-label="Qwen contract readiness">
      <div>
        <p className="workspace__eyebrow">Qwen Contract</p>
        <h4>contract-only readiness</h4>
        <p>
          当前仅承接输入边界，不调用模型、不生成分镜、不写入导出链路。
        </p>
      </div>

      <div className="qwen-readiness-card__status">
        <StatusTile
          label="输入完整度"
          value={`${completedCount} / ${readinessFields.length} fields`}
        />
        <StatusTile label="生成链路" value="未启用" />
      </div>

      <div className="qwen-readiness-card__list">
        {QWEN_CONTRACT_READINESS.map((item) => (
          <span key={item} className="readiness-pill readiness-pill--draft">
            {item}
          </span>
        ))}
      </div>

      <div className="qwen-readiness-card__fields">
        {readinessFields.map((field) => (
          <span
            key={field.label}
            className={
              field.ready
                ? "readiness-pill readiness-pill--ok"
                : "readiness-pill readiness-pill--pending"
            }
          >
            {field.label}: {field.ready ? "已填写" : "待填写"}
          </span>
        ))}
      </div>
    </section>
  );
}

function ReadinessStrip() {
  return (
    <div className="readiness-strip" aria-label="分镜 readiness 状态">
      {STORYBOARD_READINESS_FLOW.map((status) => (
        <span key={status} className={`readiness-pill ${readinessClassName(status)}`}>
          {status}
        </span>
      ))}
    </div>
  );
}

function StoryboardSemanticGrid() {
  return (
    <div className="storyboard-semantic-grid" aria-label="分镜语义分组">
      {STORYBOARD_SEMANTIC_GROUPS.map((item) => (
        <article key={item.group} className="storyboard-semantic-card">
          <div className="storyboard-semantic-card__header">
            <strong>{item.group}</strong>
            <span className={`readiness-pill ${readinessClassName(item.status)}`}>
              {item.status}
            </span>
          </div>
          <p>{item.detail}</p>
        </article>
      ))}
    </div>
  );
}

function StoryboardDraftTable() {
  return (
    <div className="storyboard-table-wrap" aria-label="分镜草案展示区">
      <div className="storyboard-table-wrap__header">
        <div>
          <p className="workspace__eyebrow">分镜草案</p>
          <h4>最小脚本表格</h4>
        </div>
        <span>本地占位数据，等待生成链路接入。</span>
      </div>
      <table className="storyboard-table">
        <thead>
          <tr>
            <th>镜头</th>
            <th>画面意图</th>
            <th>运动与镜头</th>
            <th>声音与对白</th>
            <th>连续性与交接</th>
            <th>状态</th>
          </tr>
        </thead>
        <tbody>
          {STORYBOARD_DRAFT_ROWS.map((row) => (
            <tr key={row.shot}>
              <td>{row.shot}</td>
              <td>{row.intent}</td>
              <td>{row.camera}</td>
              <td>{row.sound}</td>
              <td>{row.handoff}</td>
              <td>
                <span className={`readiness-pill ${readinessClassName(row.status)}`}>
                  {row.status}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function PromptPreview({ storyboardNeed }: { storyboardNeed: string }) {
  const requirement = storyboardNeed.trim() || "等待用户输入需求";

  return (
    <article className="prompt-preview" aria-label="分镜提示词预览区">
      <div>
        <p className="workspace__eyebrow">提示词预览</p>
        <h4>最终提示预览</h4>
      </div>
      <p>
        以用户需求“{requirement}”为核心，生成分镜草案时需覆盖画面意图、运动与镜头、声音与对白、
        连续性与交接、参考与锁定、导出就绪六类语义，不展示后台内部内容。
      </p>
    </article>
  );
}

function readinessClassName(status: StoryboardReadinessStatus) {
  switch (status) {
    case "校验通过":
    case "可导出":
      return "readiness-pill--ok";
    case "待补全":
    case "待修正":
      return "readiness-pill--pending";
    case "阻断":
      return "readiness-pill--blocked";
    case "草案":
    default:
      return "readiness-pill--draft";
  }
}

function PreviewView({ selectedProjectId }: { selectedProjectId: string | null }) {
  const preview = useAsyncCommand(
    () => loadPreviewSnapshot(selectedProjectId ?? undefined),
    [selectedProjectId],
  );

  return (
    <section className="view-grid view-grid--preview">
      <div className="panel">
        <div className="panel__header">
          <h3>Storyboard</h3>
          <span className="panel__hint">分镜草案预览</span>
        </div>
        {renderPreviewSection(selectedProjectId, preview, "storyboard")}
      </div>
      <div className="panel">
        <div className="panel__header">
          <h3>RenderSegment</h3>
          <span className="panel__hint">运动与镜头语义</span>
        </div>
        {renderPreviewSection(selectedProjectId, preview, "renderSegment")}
      </div>
      <div className="panel">
        <div className="panel__header">
          <h3>Cuts</h3>
          <span className="panel__hint">连续性与交接</span>
        </div>
        {renderPreviewSection(selectedProjectId, preview, "cuts")}
      </div>
    </section>
  );
}

function renderPreviewSection(
  selectedProjectId: string | null,
  preview: AsyncState<Record<"storyboard" | "renderSegment" | "cuts", PreviewItem[]>>,
  key: "storyboard" | "renderSegment" | "cuts",
) {
  if (!selectedProjectId) {
    return <EmptyState title="先选择项目" description="当前视图需要项目上下文。" />;
  }

  if (preview.status === "loading") {
    return <LoadingCard lines={3} />;
  }

  if (preview.status === "error") {
    return <EmptyState title="预览加载失败" description={preview.error ?? "请稍后重试。"} />;
  }

  const items = preview.data?.[key] ?? [];
  if (items.length === 0) {
    return <EmptyState title="暂无预览条目" description="等待分镜草案补全。" />;
  }

  return (
    <div className="stack">
      {items.map((item) => (
        <LayerCard key={item.id} title={`${item.label} / ${item.duration}`} text={item.note} />
      ))}
    </div>
  );
}

function ExportView({ selectedProjectId }: { selectedProjectId: string | null }) {
  const validation = useAsyncCommand<ValidationExportPanelSnapshot>(
    () => loadExportValidationSnapshot(selectedProjectId ?? undefined),
    [selectedProjectId],
  );

  return (
    <section className="view-grid view-grid--export">
      <div className="panel">
        <div className="panel__header">
          <h3>导出 Excel</h3>
          <span className="panel__hint">主交付路径</span>
        </div>

        {!selectedProjectId ? (
          <EmptyState title="先选择项目" description="导出面板需要项目上下文。" />
        ) : (
          <div className="detail-card">
            <strong>分镜结果表格</strong>
            <p>分镜草案经过校验后，主交付锁定为导出 Excel。</p>
            <button className="panel__button" disabled type="button">
              导出 Excel（待校验通过）
            </button>
          </div>
        )}
      </div>

      <div className="panel">
        <div className="panel__header">
          <h3>Validator 状态</h3>
          <span className="panel__hint">草案 readiness 检查</span>
        </div>

        {!selectedProjectId ? (
          <EmptyState title="先选择项目" description="校验结果将在项目选中后显示。" />
        ) : validation.status === "loading" ? (
          <LoadingCard lines={3} />
        ) : validation.status === "error" ? (
          <EmptyState title="校验加载失败" description={validation.error ?? "请稍后重试。"} />
        ) : (
          <div className="stack">
            {validation.data?.summaryItems.map((item) => (
              <ValidationRow key={item.label} item={item} />
            ))}
          </div>
        )}
      </div>

      <div className="panel">
        <div className="panel__header">
          <h3>Repair 建议</h3>
          <span className="panel__hint">后台修复建议</span>
        </div>

        {!selectedProjectId ? (
          <EmptyState title="先选择项目" description="修复建议会跟随当前项目上下文。" />
        ) : validation.status === "loading" ? (
          <LoadingCard lines={4} />
        ) : validation.status === "error" ? (
          <EmptyState title="修复建议加载失败" description={validation.error ?? "请稍后重试。"} />
        ) : validation.data?.repairRecommendations.length ? (
          <div className="stack">
            {validation.data.repairRecommendations.map((item) => (
              <RepairRecommendationCard key={item.failureCode} item={item} />
            ))}
          </div>
        ) : (
          <EmptyState title="暂无修复建议" description="当前项目尚未返回可展示的修复建议。" />
        )}
      </div>
    </section>
  );
}

function LayerCard({ title, text }: { title: string; text: string }) {
  return (
    <article className="layer-card">
      <div className="layer-card__title">{title}</div>
      <p>{text}</p>
    </article>
  );
}

function ValidationRow({ item }: { item: ExportValidationItem }) {
  const stateClass =
    item.state === "正常"
      ? "validation-row__state--ok"
      : item.state === "待补充"
        ? "validation-row__state--pending"
        : "validation-row__state--blocked";

  return (
    <div className="validation-row">
      <div>
        <strong>{item.label}</strong>
        <p>{item.value}</p>
      </div>
      <span className={`validation-row__state ${stateClass}`}>{item.state}</span>
    </div>
  );
}

function RepairRecommendationCard({ item }: { item: ValidationRepairRecommendation }) {
  return (
    <article className="detail-card">
      <strong>{item.failureName}</strong>
      <p>失败码：{item.failureCode}</p>
      <p>策略：{item.repairStrategy}</p>
      <p>优先级：{item.repairPriority}</p>
      <p>范围：{item.repairScope}</p>
      <p>校验提示：{item.validatorHint}</p>
      <p className="detail-card__note">
        模板：{item.promptTemplateNames.length ? item.promptTemplateNames.join(" / ") : "待补充"}
      </p>
    </article>
  );
}
