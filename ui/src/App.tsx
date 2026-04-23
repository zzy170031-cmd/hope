
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
    detail: "Short concept, ask, or creative direction from the editor.",
  },
  {
    value: "synopsis",
    label: "Synopsis",
    detail: "A compact story arc or scene-by-scene summary.",
  },
  {
    value: "script",
    label: "Script",
    detail: "Existing screenplay or scene text to adapt into boards.",
  },
];

const QWEN_CONTRACT_READINESS = [
  "Qwen storyboard generation contract boundary accepted",
  "Writer can capture the minimum local input package",
  "Model execution remains disabled in this internal trial build",
  "API key is treated as local session input only",
];

const STORYBOARD_REQUIREMENT_PLACEHOLDER =
  "Example: A young engineer notices an impossible signal during the first metro ride of the morning. Keep the pace restrained, let the reveal build gradually, and end on a clear handoff into the next beat.";

const STORYBOARD_READINESS_FLOW: StoryboardReadinessStatus[] = [
  "Draft",
  "Needs Detail",
  "Needs Polish",
  "Validated",
  "Ready to Export",
  "Blocked",
];

const STORYBOARD_SEMANTIC_GROUPS: Array<{
  group: StoryboardSemanticGroup;
  status: StoryboardReadinessStatus;
  detail: string;
}> = [
  {
    group: "Visual Intent",
    status: "Draft",
    detail: "Lock the emotional goal of each shot before generation is connected.",
  },
  {
    group: "Camera and Motion",
    status: "Needs Detail",
    detail: "Camera movement and shot rhythm stay as editorial guidance for now.",
  },
  {
    group: "Sound and Dialogue",
    status: "Needs Polish",
    detail: "Dialogue and sound cues remain writing notes, not generated audio.",
  },
  {
    group: "Continuity and Handoff",
    status: "Validated",
    detail: "The shell keeps identity, motion, and scene continuity visible to the team.",
  },
  {
    group: "Reference Locks",
    status: "Needs Detail",
    detail: "Reference guidance stays as constraints only, without raw asset management.",
  },
  {
    group: "Delivery Prep",
    status: "Ready to Export",
    detail: "Excel remains the primary delivery target once validation is clear.",
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
    shot: "Shot 01",
    intent: "Reserve the opening beat for the first visual reveal.",
    camera: "Camera direction will be attached by the future generation pass.",
    sound: "Use ambient sound notes only in this build.",
    handoff: "Carry the same character and space identity into the next shot.",
    status: "Draft",
  },
  {
    shot: "Shot 02",
    intent: "Mark the first shift in attention or tension.",
    camera: "Movement stays as an editor note rather than a generated output.",
    sound: "Dialogue remains optional and can stay empty.",
    handoff: "Continuity review should stay visible before preview and export.",
    status: "Needs Polish",
  },
  {
    shot: "Shot 03",
    intent: "Keep room for the next scene handoff without hardcoding final content.",
    camera: "Cut rhythm will be confirmed during preview review.",
    sound: "Hold for later validation rather than inventing final lines.",
    handoff: "Final export waits for validation and repair guidance.",
    status: "Needs Detail",
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
  const activeRoute = ROUTES.find((route) => route.id === activeView);

  return (
    <Shell activeView={activeView} onNavigate={navigate}>
      <div className="workspace__header">
        <div className="workspace__title">
          <p className="workspace__eyebrow">Desktop MVP</p>
          <h2>{activeRoute?.label ?? "Hope Desktop"}</h2>
          <p className="workspace__lede">
            {activeRoute?.description ??
              "Internal trial shell for the packaged Hope desktop workflow."}
          </p>
        </div>
        <div className="workspace__status">
          <span className="workspace__chip">Internal Trial</span>
          <span className="workspace__chip workspace__chip--soft">{bridgeStatus.label}</span>
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
  const statusTone = isError ? "error" : isChecking ? "checking" : isLocalPresent ? "connected" : "missing";
  const statusLabel =
    runtimeStatus === "local_present"
      ? "Local key draft saved"
      : runtimeStatus === "local_checking"
        ? "Checking local input"
        : runtimeStatus === "local_invalid"
          ? "Local key looks incomplete"
          : "No local Qwen key draft";
  const statusDetail =
    isLocalPresent && lastFive
      ? `Stored for this session only. Ending in *****${lastFive}.`
      : isError
        ? "The key looks too short. No Qwen call was attempted."
        : isChecking
          ? "Reviewing the local field only. No external request is sent."
          : "Optional local note for later Qwen wiring. This build does not call the model.";

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
    <form className="qwen-secret" onSubmit={handleSubmit} aria-label="Local Qwen key draft">
      <div className="qwen-secret__status" aria-live="polite">
        <span className={`qwen-secret__dot qwen-secret__dot--${statusTone}`} />
        <span className="qwen-secret__state">{statusLabel}</span>
        <span className="qwen-secret__detail">{statusDetail}</span>
      </div>
      <div className="qwen-secret__controls">
        <input
          aria-label="Enter a local Qwen API key draft"
          autoComplete="new-password"
          className="qwen-secret__input"
          disabled={isChecking}
          onChange={(event) => setSecretInput(event.target.value)}
          placeholder="Optional local Qwen key draft"
          spellCheck={false}
          type="password"
          value={secretInput}
        />
        <button className="panel__button" disabled={!canSubmit} type="submit">
          Save local draft
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
      <section className="readonly-landing" aria-label="Workspace readiness">
        <div className="readonly-landing__intro">
          <p className="workspace__eyebrow">Readonly Status</p>
          <h3>Workspace Readiness</h3>
          <p>Loading packaged readiness signals for the desktop shell.</p>
        </div>
        <LoadingCard lines={3} />
      </section>
    );
  }

  if (status.status === "error" || !status.data) {
    return (
      <section className="readonly-landing" aria-label="Workspace readiness">
        <div className="readonly-landing__intro">
          <p className="workspace__eyebrow">Readonly Status</p>
          <h3>Workspace Readiness</h3>
          <p>
            The packaged desktop shell is running, but readonly status is not available right now.
          </p>
        </div>
        <div className="readonly-landing__error">
          Readonly status is temporarily unavailable.
        </div>
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
    <section className="readonly-landing" aria-label="Workspace readiness">
      <div className="readonly-landing__intro">
        <p className="workspace__eyebrow">Readonly Status</p>
        <h3>Workspace Readiness</h3>
        <p>
          The shell reads packaged knowledge and validation signals without exposing raw KB content or widening runtime scope.
        </p>
      </div>

      <div className="readonly-landing__grid">
        <StatusTile
          label="Source package"
          value={snapshot.snapshotIdentity.sourceName || "Packaged snapshot"}
        />
        <StatusTile
          label="Seed format"
          value={snapshot.snapshotIdentity.seedFormat || "Unknown"}
        />
        <StatusTile
          label="Snapshot time"
          value={formatTimestamp(snapshot.snapshotIdentity.createdAtTimestamp)}
        />
        <StatusTile
          label="Knowledge guardrails"
          value={knowledgeReady ? "Ready for desktop use" : "Needs follow-up"}
        />
        <StatusTile
          label="Validation coverage"
          value={validationReady ? "Loaded into the shell" : "Still incomplete"}
        />
        <StatusTile
          label="Repair mapping"
          value={repairReady ? "Available to validation review" : "Still incomplete"}
        />
        <StatusTile
          wide
          label="Package summary"
          value={`Snapshot ${snapshot.snapshotIdentity.snapshotId || "unknown"} · Taxonomies ${snapshot.knowledgeBundle.sceneTaxonomyCount} · Failure patterns ${snapshot.knowledgeBundle.failurePatternCount} · Prompt templates ${snapshot.knowledgeBundle.promptTemplateCount}`}
        />
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
      <strong>{value || "Unavailable"}</strong>
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
          <div>
            <h3>Projects</h3>
            <p className="panel__hint">Choose the active project before moving into writing, review, or export.</p>
          </div>
          <button
            type="button"
            className="panel__button panel__button--ghost"
            onClick={() => {
              void invokeHopeCommand(HOPE_TAURI_COMMANDS.projectCreateOrSwitch);
            }}
            title="Desktop IPC action"
          >
            New or Switch
          </button>
        </div>

        {projects.status === "loading" ? (
          <LoadingCard lines={4} />
        ) : projects.status === "error" ? (
          <EmptyState
            title="Project list unavailable"
            description={projects.error ?? "Try again after the packaged desktop bridge is ready."}
          />
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
                <p>Updated {project.updatedAt}</p>
                <p>{project.episodeCount} episodes in scope</p>
              </button>
            ))}
          </div>
        )}
      </div>

      <div className="panel">
        <div className="panel__header">
          <div>
            <h3>Current Project</h3>
            <p className="panel__hint">The desktop shell keeps this selection as the working context for every surface.</p>
          </div>
          <span className="panel__hint">Desktop bridge only</span>
        </div>

        {selectedProject ? (
          <div className="detail-card">
            <strong>{selectedProject.name}</strong>
            <p>Status: {selectedProject.status}</p>
            <p>Updated: {selectedProject.updatedAt}</p>
            <p>Episodes: {selectedProject.episodeCount}</p>
          </div>
        ) : (
          <EmptyState
            title="No project selected yet"
            description="Pick a project first so the rest of the workbench has a clear context."
          />
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
        <div>
          <h3>Storyboard Workbench</h3>
          <p className="panel__hint">
            Shape the brief, review writing layers, and prepare the storyboard handoff for preview and Excel export.
          </p>
        </div>
        <span className="panel__hint">Input / Draft / Review / Export</span>
      </div>
      {!selectedProjectId ? (
        <EmptyState
          title="Choose a project first"
          description="The storyboard workbench opens only after a project is in context."
        />
      ) : writer.status === "loading" ? (
        <LoadingCard lines={4} />
      ) : writer.status === "error" ? (
        <EmptyState
          title="Writer snapshot unavailable"
          description={writer.error ?? "Try again after the packaged desktop bridge responds."}
        />
      ) : (
        <div className="storyboard-workbench">
          <aside className="storyboard-workbench__rail" aria-label="Storyboard guidance">
            <div>
              <p className="workspace__eyebrow">Workbench Focus</p>
              <h4>Creative Direction</h4>
              <p>Keep the tone, timing, and production intent readable without inventing final model output.</p>
            </div>
            <div className="storyboard-lock-card">
              <strong>Consistency Guardrails</strong>
              <p>Character, scene, and continuity notes stay visible as constraints, not as a raw asset library.</p>
            </div>
            <div className="storyboard-lock-card">
              <strong>Generation Boundary</strong>
              <p>This build captures input and review context only. Live Qwen and Seedance remain disabled.</p>
            </div>
            <div className="storyboard-lock-card">
              <strong>Delivery Target</strong>
              <p>The workbench still converges on Excel delivery after preview and validation review.</p>
            </div>
          </aside>

          <div className="storyboard-workbench__main">
            <div className="storyboard-input-card">
              <div>
                <p className="workspace__eyebrow">Story Brief</p>
                <h4>Capture the editorial ask</h4>
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
                <span>This package records the minimum Writer input boundary only. No model call is made.</span>
                <button className="panel__button" disabled type="button">
                  Storyboard draft generation comes later
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
                <p className="workspace__eyebrow">Revision Notes</p>
                <h4>Global editorial changes</h4>
              </div>
              <textarea
                className="storyboard-input-card__field"
                placeholder="Example: Raise the tension in the second beat, but keep the same character identity, location continuity, and export structure."
              />
            </div>

            <div className="stack">
              <LayerCard title="Synopsis" text={writer.data?.synopsis ?? ""} />
              <LayerCard title="Story" text={writer.data?.story ?? ""} />
              <LayerCard title="Screenplay" text={writer.data?.screenplay ?? ""} />
              <LayerCard title="Storyboard Notes" text={writer.data?.storyboard ?? ""} />
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
    <section className="writer-boundary-panel" aria-label="Writer input package">
      <div className="writer-boundary-panel__header">
        <div>
          <p className="workspace__eyebrow">Input Package</p>
          <h4>Minimum contract-ready capture</h4>
        </div>
        <span className="readiness-pill readiness-pill--pending">Contract-only</span>
      </div>

      <div className="writer-boundary-panel__grid">
        <label className="writer-boundary-field">
          <span>Input Kind</span>
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
          <span>Duration Target</span>
          <input
            value={inputBoundary.duration_target}
            onChange={(event) => onChange("duration_target", event.target.value)}
            placeholder="Example: 60 seconds / 3 minutes / single episode opener"
          />
        </label>

        <label className="writer-boundary-field">
          <span>Scene Count Hint</span>
          <input
            value={inputBoundary.scene_count_hint}
            onChange={(event) => onChange("scene_count_hint", event.target.value)}
            placeholder="Example: 3-5 scenes"
          />
        </label>

        <label className="writer-boundary-field writer-boundary-field--wide">
          <span>Story Constraints</span>
          <textarea
            value={inputBoundary.story_constraints}
            onChange={(event) => onChange("story_constraints", event.target.value)}
            placeholder="Capture story limits and beats without writing final storyboard content."
          />
        </label>

        <label className="writer-boundary-field writer-boundary-field--wide">
          <span>Character Constraints</span>
          <textarea
            value={inputBoundary.character_constraints}
            onChange={(event) => onChange("character_constraints", event.target.value)}
            placeholder="Keep identity, continuity, and role notes without building an entity library."
          />
        </label>

        <label className="writer-boundary-field writer-boundary-field--wide">
          <span>Style Constraints</span>
          <textarea
            value={inputBoundary.style_constraints}
            onChange={(event) => onChange("style_constraints", event.target.value)}
            placeholder="Record tone and style direction only. No Qwen or Seedance call is made."
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
        <p className="workspace__eyebrow">Readiness</p>
        <h4>Generation boundary status</h4>
        <p>
          Writer can collect the contract-shaped input package, but the shell still avoids live model execution and export writes.
        </p>
      </div>

      <div className="qwen-readiness-card__status">
        <StatusTile
          label="Input completeness"
          value={`${completedCount} / ${readinessFields.length} fields`}
        />
        <StatusTile label="Generation path" value="Disabled in this build" />
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
            {field.label}: {field.ready ? "captured" : "waiting"}
          </span>
        ))}
      </div>
    </section>
  );
}

function ReadinessStrip() {
  return (
    <div className="readiness-strip" aria-label="Storyboard readiness flow">
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
    <div className="storyboard-semantic-grid" aria-label="Storyboard semantic groups">
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
    <div className="storyboard-table-wrap" aria-label="Storyboard structure preview">
      <div className="storyboard-table-wrap__header">
        <div>
          <p className="workspace__eyebrow">Storyboard Structure</p>
          <h4>Slot-based draft preview</h4>
        </div>
        <span>These rows show workbench structure only. They are not generated storyboards.</span>
      </div>
      <table className="storyboard-table">
        <thead>
          <tr>
            <th>Shot</th>
            <th>Visual Intent</th>
            <th>Camera and Motion</th>
            <th>Sound and Dialogue</th>
            <th>Continuity and Handoff</th>
            <th>Status</th>
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
  const requirement = storyboardNeed.trim() || "Waiting for an editorial brief.";

  return (
    <article className="prompt-preview" aria-label="Generation brief preview">
      <div>
        <p className="workspace__eyebrow">Generation Brief</p>
        <h4>What the future generation pass should cover</h4>
      </div>
      <p>
        Center the future storyboard generation on "{requirement}" and make sure the output can cover visual intent, camera and motion, sound and dialogue, continuity, reference locks, and delivery preparation. This internal trial build does not call Qwen and does not fabricate storyboard content.
      </p>
    </article>
  );
}

function readinessClassName(status: StoryboardReadinessStatus) {
  switch (status) {
    case "Validated":
    case "Ready to Export":
      return "readiness-pill--ok";
    case "Needs Detail":
    case "Needs Polish":
      return "readiness-pill--pending";
    case "Blocked":
      return "readiness-pill--blocked";
    case "Draft":
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
          <div>
            <h3>Storyboard Frames</h3>
            <p className="panel__hint">Review the visual frame notes that feed the storyboard pass.</p>
          </div>
        </div>
        {renderPreviewSection(selectedProjectId, preview, "storyboard")}
      </div>
      <div className="panel">
        <div className="panel__header">
          <div>
            <h3>Camera Movement</h3>
            <p className="panel__hint">Check motion and render-segment notes before delivery.</p>
          </div>
        </div>
        {renderPreviewSection(selectedProjectId, preview, "renderSegment")}
      </div>
      <div className="panel">
        <div className="panel__header">
          <div>
            <h3>Cut Continuity</h3>
            <p className="panel__hint">Keep cut rhythm and continuity visible before export.</p>
          </div>
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
    return (
      <EmptyState
        title="Choose a project first"
        description="Preview surfaces follow the active desktop project context."
      />
    );
  }

  if (preview.status === "loading") {
    return <LoadingCard lines={3} />;
  }

  if (preview.status === "error") {
    return (
      <EmptyState
        title="Preview unavailable"
        description={preview.error ?? "Try again once the packaged desktop bridge responds."}
      />
    );
  }

  const items = preview.data?.[key] ?? [];
  if (items.length === 0) {
    return (
      <EmptyState
        title="Nothing to review yet"
        description="Preview content will appear after the storyboard handoff is filled in."
      />
    );
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
          <div>
            <h3>Excel Delivery</h3>
            <p className="panel__hint">The MVP still ships through Excel once review is complete.</p>
          </div>
        </div>

        {!selectedProjectId ? (
          <EmptyState
            title="Choose a project first"
            description="Export review needs the current desktop project context."
          />
        ) : (
          <div className="detail-card">
            <strong>Storyboard workbook delivery</strong>
            <p>Preview and validation still feed one primary delivery route: export to Excel.</p>
            <button className="panel__button" disabled type="button">
              Export to Excel when validation is clear
            </button>
          </div>
        )}
      </div>

      <div className="panel">
        <div className="panel__header">
          <div>
            <h3>Validation Summary</h3>
            <p className="panel__hint">Track the minimum readiness checks before export.</p>
          </div>
        </div>

        {!selectedProjectId ? (
          <EmptyState
            title="Choose a project first"
            description="Validation follows the currently selected desktop project."
          />
        ) : validation.status === "loading" ? (
          <LoadingCard lines={3} />
        ) : validation.status === "error" ? (
          <EmptyState
            title="Validation snapshot unavailable"
            description={validation.error ?? "Try again after the desktop bridge responds."}
          />
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
          <div>
            <h3>Repair Guidance</h3>
            <p className="panel__hint">Readonly repair guidance stays visible without widening export scope.</p>
          </div>
        </div>

        {!selectedProjectId ? (
          <EmptyState
            title="Choose a project first"
            description="Repair guidance is attached to the active project context."
          />
        ) : validation.status === "loading" ? (
          <LoadingCard lines={4} />
        ) : validation.status === "error" ? (
          <EmptyState
            title="Repair guidance unavailable"
            description={validation.error ?? "Try again after the desktop bridge responds."}
          />
        ) : validation.data?.repairRecommendations.length ? (
          <div className="stack">
            {validation.data.repairRecommendations.map((item) => (
              <RepairRecommendationCard key={item.failureCode} item={item} />
            ))}
          </div>
        ) : (
          <EmptyState
            title="No repair guidance yet"
            description="The current project does not return repair guidance for display right now."
          />
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
      <p>Failure code: {item.failureCode}</p>
      <p>Strategy: {item.repairStrategy}</p>
      <p>Priority: {item.repairPriority}</p>
      <p>Scope: {item.repairScope}</p>
      <p>Validator hint: {item.validatorHint}</p>
      <p className="detail-card__note">
        Prompt templates: {item.promptTemplateNames.length ? item.promptTemplateNames.join(" / ") : "Waiting for template names"}
      </p>
    </article>
  );
}

function formatTimestamp(timestamp: number) {
  if (!timestamp) {
    return "No timestamp";
  }

  const normalized = timestamp > 1_000_000_000_000 ? timestamp : timestamp * 1000;
  const date = new Date(normalized);

  if (Number.isNaN(date.getTime())) {
    return "Unknown";
  }

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}
