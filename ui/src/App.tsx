import { useEffect, useMemo, useState } from "react";
import { EmptyState } from "./components/EmptyState";
import { LoadingCard } from "./components/LoadingCard";
import { Shell } from "./components/Shell";
import { ROUTES, resolveRoute } from "./routes";
import {
  HOPE_TAURI_COMMANDS,
  getHopeBridgeStatus,
  invokeHopeCommand,
  loadExportValidationSnapshot,
  loadPreviewSnapshot,
  loadProjectList,
  loadWriterSnapshot,
} from "./bridge/hopeBridge";
import type {
  ExportValidationItem,
  PreviewItem,
  ProjectSummary,
  ValidationExportPanelSnapshot,
  ValidationRepairRecommendation,
  ViewId,
} from "./types";

interface AsyncState<T> {
  status: "loading" | "ready" | "error";
  data: T | null;
  error: string | null;
}

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
        </div>
      </div>

      <ViewRouter
        activeView={activeView}
        selectedProjectId={selectedProjectId}
        setSelectedProjectId={setSelectedProjectId}
      />
    </Shell>
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

  return (
    <section className="panel">
      <div className="panel__header">
        <h3>Synopsis -&gt; Story -&gt; Screenplay -&gt; Storyboard</h3>
        <span className="panel__hint">四层查看入口</span>
      </div>

      {!selectedProjectId ? (
        <EmptyState title="先选择项目" description="Writer 视图只展示当前项目的四层结构。" />
      ) : writer.status === "loading" ? (
        <LoadingCard lines={4} />
      ) : writer.status === "error" ? (
        <EmptyState title="Writer 加载失败" description={writer.error ?? "请稍后重试。"} />
      ) : (
        <div className="stack">
          <LayerCard title="Synopsis" text={writer.data?.synopsis ?? ""} />
          <LayerCard title="Story" text={writer.data?.story ?? ""} />
          <LayerCard title="Screenplay" text={writer.data?.screenplay ?? ""} />
          <LayerCard title="Storyboard" text={writer.data?.storyboard ?? ""} />
        </div>
      )}
    </section>
  );
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
          <span className="panel__hint">共享 fixture 预览</span>
        </div>
        {renderPreviewSection(selectedProjectId, preview, "storyboard")}
      </div>
      <div className="panel">
        <div className="panel__header">
          <h3>RenderSegment</h3>
          <span className="panel__hint">30s - 90s / 不跨 NarrativeScene</span>
        </div>
        {renderPreviewSection(selectedProjectId, preview, "renderSegment")}
      </div>
      <div className="panel">
        <div className="panel__header">
          <h3>Cuts</h3>
          <span className="panel__hint">共享 fixture 中的 cut</span>
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
    return <EmptyState title="暂无预览条目" description="等待共享 fixture 注入更多数据。" />;
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
          <h3>Export 面板</h3>
          <span className="panel__hint">仅展示共享导出链状态</span>
        </div>

        {!selectedProjectId ? (
          <EmptyState title="先选择项目" description="导出面板需要项目上下文。" />
        ) : (
          <div className="detail-card">
            <strong>当前导出链</strong>
            <p>共享 fixture -&gt; validation_report -&gt; Excel / JSON / Markdown</p>
            <p>命令：{HOPE_TAURI_COMMANDS.validationExportPanelSnapshot}</p>
          </div>
        )}
      </div>

      <div className="panel">
        <div className="panel__header">
          <h3>Validation 面板</h3>
          <span className="panel__hint">与 Validation sheet 同步</span>
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
          <span className="panel__hint">KB-backed recommendation</span>
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
          <EmptyState title="暂无修复建议" description="当前项目尚未返回 KB-backed repair 项。" />
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
