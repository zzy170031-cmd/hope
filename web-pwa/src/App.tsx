import {
  type ChangeEvent,
  type ReactNode,
  startTransition,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { buildEvidence, createStoryboardTask } from "./lib/evidence";
import { createStoryboardExcelExport, createScriptExcelExport, auditBundle, downloadBundle } from "./lib/export";
import { extractSourceFacts } from "./lib/facts";
import { sha256Hex } from "./lib/hash";
import { buildSanitizedKbSummary, getDurationProfile, KB_SNAPSHOT } from "./lib/kb";
import { runChatCompletion, testProviderConnection } from "./lib/model";
import { normalizeStoryboardRows, parseJsonResponse } from "./lib/parsing";
import {
  applyProviderChoice,
  createDefaultProviderState,
  deriveConnectionStatus,
  deriveProviderModelRecord,
  getProviderDefinition,
  PROVIDER_DEFINITIONS,
} from "./lib/providers";
import { buildDirectorMessages, buildWritingMessages, compilePromptText } from "./lib/prompts";
import { getSceneTypeLabel, SCENE_TYPE_OPTIONS } from "./lib/sceneTypes";
import { loadProviderState, saveProviderState } from "./lib/storage";
import type {
  ExportAudit,
  ExportBundle,
  GenerationContext,
  GenerationEvidence,
  OperationMode,
  ProviderFormState,
  QaTrace,
  SanitizedKbSummary,
  StoryboardRow,
  StoryboardTask,
  ValidationResult,
} from "./lib/types";
import { normalizeAndValidateStoryboardRows, validateNarrativeBody } from "./lib/validator";

type AsyncStatus = "idle" | "loading" | "ready" | "failed";
type ModalKind = "docs" | "provider" | "kb" | "evidence" | "editor" | "finalized" | "taskCreate" | "taskImport" | null;
type RowField =
  | "person"
  | "shot_size"
  | "camera"
  | "visual_description"
  | "character_action"
  | "dialogue_or_narration"
  | "prompt_text"
  | "duration_seconds"
  | "status"
  | "note";

interface TaskCreationDraft {
  taskName: string;
  taskIndex: number;
  currentBodySource: "accepted_body";
  acceptedBodyHash: string;
  sceneTypeId: string;
  sceneTypeLabel: string;
  targetDurationSeconds: number;
  outputIntent: "storyboard_rows";
  writingGroupRulePackIds: string[];
  directorGroupRulePackIds: string[];
  kbSnapshotHash: string;
  generationGroup: string;
  estimatedShotCount: number;
  durationAllocationStrategy: string;
  continuityStatus: string;
  createFromConfirmedBody: boolean;
  preservePreviousTask: boolean;
  storyboardTaskHash: string;
  selectedTaskId: string;
  selectedTaskStatus: "pending" | "queued" | "generated";
  selectedTaskSourceKind: "system_candidate" | "full_narrative" | "confirmed_body" | "custom_fragment";
  selectedTaskSourceLabel: string;
  peopleSummary: string;
  scriptFragment: string;
  originalFragment: string;
  candidates: TaskCandidate[];
}

interface TaskArchive {
  task: StoryboardTask;
  sourceText: string;
  acceptedBody: string;
  bodyEvidence: GenerationEvidence | null;
  rows: StoryboardRow[];
  confirmedRows: StoryboardRow[];
  rowValidation: ValidationResult | null;
  storyboardEvidence: GenerationEvidence | null;
}

interface TaskCandidate {
  id: string;
  title: string;
  summary: string;
  fragment: string;
  durationSeconds: number;
  sourceKind: "system_candidate" | "full_narrative" | "confirmed_body" | "custom_fragment";
  sourceLabel: string;
  peopleSummary: string;
  queueStatus: "current" | "queued" | "available";
}

const DEFAULT_SOURCE_TEXT =
  "暴雨夜里，林峰护着苏瑶穿过旧城戏台后的狭窄通道。阿青在后方提醒追兵已经逼近巷口，林峰必须在极短时间内把证物交到苏瑶手中，同时决定是继续撤离还是反身断后。";

const DOC_SECTIONS: Array<{ title: string; body: string[] }> = [
  {
    title: "产品链路",
    body: [
      "先输入原始文本与 API 配置，再让 KB 写作组参与扩写或改写。",
      "点击“确定使用”后，accepted body 成为唯一事实源。",
      "新建镜头任务会绑定 accepted_body_hash、scene_type、target_duration、KB 摘要与 source_lineage。",
      "开始生成只能消费当前 fresh storyboard task，并在结果表格里生成 rows、visual_description 和 prompt_text。",
    ],
  },
  {
    title: "安全边界",
    body: [
      "API Key 默认只保存在当前会话，勾选后才会写入本机浏览器。",
      "导出中只保留 sanitized KB summary 和 sanitized provenance summary。",
      "禁止导出 raw KB rows、source_register、overlay JSON、prompt_body、secret 或本机绝对路径。",
    ],
  },
  {
    title: "QA 口径",
    body: [
      "targeted / full16 / formal403 都必须使用 fresh artifact root。",
      "plan-only 只用于结构核对，不能当 formal403 完成证据。",
      "rows=0、rows_match=true、旧 .codex-run partial 或旧通过记录都不能算成功。",
    ],
  },
];

function firstCharacterFromBody(body: string): string {
  const matches = body.match(/[\u4e00-\u9fa5]{2,4}/g) ?? [];
  return matches.find((item) => !/(场景|镜头|画面|分镜|提示词)/.test(item)) ?? "主角";
}

function defaultShotSize(index: number): string {
  return ["中景", "近景", "全景", "特写"][index % 4];
}

function defaultCamera(index: number): string {
  return ["推进", "跟拍", "平移", "定镜"][index % 4];
}

function allocateDurations(targetDuration: number, count: number): number[] {
  const safeCount = Math.max(1, count);
  const base = Math.floor(targetDuration / safeCount);
  const remainder = targetDuration % safeCount;
  return Array.from({ length: safeCount }, (_, index) => Math.max(1, base + (index < remainder ? 1 : 0)));
}

function compactText(value: string, maxLength = 90): string {
  const normalized = value.replace(/\s+/g, " ").trim();
  return normalized.length <= maxLength ? normalized : `${normalized.slice(0, maxLength)}…`;
}

function splitCandidateFragments(body: string): string[] {
  const paragraphs = body
    .split(/\n+/)
    .map((item) => item.trim())
    .filter(Boolean);
  if (paragraphs.length >= 3) {
    return paragraphs.slice(0, 3);
  }

  const sentences = body
    .split(/(?<=[。！？!?])/)
    .map((item) => item.trim())
    .filter(Boolean);
  if (sentences.length <= 3) {
    return [body.trim(), body.trim(), body.trim()].filter(Boolean).slice(0, 3);
  }

  const chunkSize = Math.ceil(sentences.length / 3);
  return Array.from({ length: 3 }, (_, index) => sentences.slice(index * chunkSize, (index + 1) * chunkSize).join(" ").trim())
    .filter(Boolean);
}

function summarizePeople(fragment: string): string {
  const facts = extractSourceFacts(fragment);
  return facts.characters.slice(0, 4).join("、") || "待识别人物";
}

function sourceKindLabel(kind: TaskCandidate["sourceKind"]): string {
  switch (kind) {
    case "custom_fragment":
      return "自定义镜头片段";
    case "full_narrative":
      return "完整扩写剧本";
    case "confirmed_body":
      return "确认后的正文";
    default:
      return "系统候选";
  }
}

function taskDraftStatusLabel(status: TaskCreationDraft["selectedTaskStatus"]): string {
  switch (status) {
    case "generated":
      return "已生成";
    case "queued":
      return "已加入队列";
    default:
      return "待生成";
  }
}

function displayCellText(value: string | number | null | undefined): string {
  if (typeof value === "number") {
    return String(value);
  }
  const normalized = (value ?? "").replace(/\s+/g, " ").trim();
  return normalized || "—";
}

void taskDraftStatusLabel;

function retimeTaskFragment(fragment: string, targetDurationSeconds: number): string {
  const normalized = fragment.replace(/\s+/g, " ").trim();
  const durationProfile = getDurationProfile(targetDurationSeconds);
  const sentences = normalized.split(/(?<=[。！？!?])/).map((item) => item.trim()).filter(Boolean);
  if (sentences.length === 0) {
    return normalized;
  }
  const targetSentenceCount = Math.max(2, Math.min(sentences.length, durationProfile.targetShotCount));
  return sentences.slice(0, targetSentenceCount).join(" ");
}

function buildFallbackVisualDescription(row: StoryboardRow, sceneLabel: string, acceptedBody: string): string {
  return `${row.person}处在${sceneLabel}的当前冲突节点中，镜头里能看到动作、空间和情绪变化。事实源参考：${acceptedBody.slice(0, 50)}。`;
}

function enrichRows(
  rawRows: StoryboardRow[],
  acceptedBody: string,
  kbSummary: SanitizedKbSummary,
  targetDuration: number,
): StoryboardRow[] {
  const durations = allocateDurations(targetDuration, Math.max(rawRows.length, 3));
  const rowSource = rawRows.length >= 3 ? rawRows : Array.from({ length: 3 }, (_, index) => rawRows[index] ?? ({
    shot_index: index + 1,
    person: "",
    shot_size: "",
    camera: "",
    visual_description: "",
    character_action: "",
    dialogue_or_narration: "",
    prompt_text: "",
    duration_seconds: 0,
    status: "generated",
    note: "",
  } as StoryboardRow));

  return rowSource.map((row, index) => {
    const nextRow: StoryboardRow = {
      shot_index: row.shot_index || index + 1,
      person: row.person || firstCharacterFromBody(acceptedBody),
      shot_size: row.shot_size || defaultShotSize(index),
      camera: row.camera || defaultCamera(index),
      visual_description:
        row.visual_description || buildFallbackVisualDescription(row, kbSummary.scene_type_label, acceptedBody),
      character_action: row.character_action || "延续 accepted body 中的关键动作与反应。",
      dialogue_or_narration: row.dialogue_or_narration || "",
      prompt_text: row.prompt_text || "",
      duration_seconds: durations[index],
      status: row.status || "generated",
      note: row.note || "由当前 accepted body 与导演组规则生成",
      is_user_edited: row.is_user_edited ?? false,
    };

    nextRow.prompt_text = nextRow.prompt_text || compilePromptText(
      {
        shot_index: nextRow.shot_index,
        person: nextRow.person,
        shot_size: nextRow.shot_size,
        camera: nextRow.camera,
        visual_description: nextRow.visual_description,
        character_action: nextRow.character_action,
        dialogue_or_narration: nextRow.dialogue_or_narration,
        duration_seconds: nextRow.duration_seconds,
        status: nextRow.status,
        note: nextRow.note,
      },
      {
        acceptedBody,
        sceneLabel: kbSummary.scene_type_label,
        sceneProfile: kbSummary.scene_profile,
        durationPacing: getDurationProfile(targetDuration).pacingDirective,
        directorRuleLabels: kbSummary.director_group_rule_pack_ids,
        negativeConstraints: kbSummary.negative_constraints,
      },
    );

    return nextRow;
  });
}

async function buildTaskCreationDraft(params: {
  acceptedBody: string;
  bodyEvidence: GenerationEvidence;
  sceneTypeId: string;
  sceneTypeLabel: string;
  targetDurationSeconds: number;
  kbSummary: SanitizedKbSummary;
  existingTask: StoryboardTask | null;
}): Promise<TaskCreationDraft> {
  const durationProfile = getDurationProfile(params.targetDurationSeconds);
  const systemFragments = splitCandidateFragments(params.acceptedBody);
  const taskIndex = (params.existingTask?.task_index ?? 0) + 1;
  const candidates: TaskCandidate[] = systemFragments.slice(0, 3).map((fragment, index) => ({
    id: `candidate-${index + 1}`,
    title: `场景候选 ${index + 1}`,
    summary: compactText(fragment, 86),
    fragment,
    durationSeconds: params.targetDurationSeconds,
    sourceKind: "system_candidate",
    sourceLabel: "系统候选",
    peopleSummary: summarizePeople(fragment),
    queueStatus: index === 0 ? "current" : "available",
  }));
  candidates.push({
    id: "full-narrative",
    title: "完整扩写剧本",
    summary: compactText(params.acceptedBody, 86),
    fragment: params.acceptedBody,
    durationSeconds: params.targetDurationSeconds,
    sourceKind: "full_narrative",
    sourceLabel: "完整扩写剧本",
    peopleSummary: summarizePeople(params.acceptedBody),
    queueStatus: "available",
  });
  const selectedCandidate = candidates[0];
  const nextTaskName = `第 ${taskIndex} 个镜头任务`;
  const taskName = `${params.sceneTypeLabel}-${params.targetDurationSeconds}秒-${new Date().toLocaleTimeString("zh-CN", { hour12: false })}`;
  void taskName;
  const identitySeed = stableTaskSeed({
    acceptedBodyHash: params.bodyEvidence.accepted_body_hash || (await sha256Hex(params.acceptedBody)),
    sceneTypeId: params.sceneTypeId,
    targetDurationSeconds: params.targetDurationSeconds,
    kbSnapshotHash: params.kbSummary.kb_snapshot_hash,
    writingGroupRulePackIds: params.kbSummary.writing_group_rule_pack_ids,
    directorGroupRulePackIds: params.kbSummary.director_group_rule_pack_ids,
    outputIntent: "storyboard_rows",
  });
  return {
    taskName: nextTaskName,
    taskIndex,
    currentBodySource: "accepted_body",
    acceptedBodyHash: params.bodyEvidence.accepted_body_hash || (await sha256Hex(params.acceptedBody)),
    sceneTypeId: params.sceneTypeId,
    sceneTypeLabel: params.sceneTypeLabel,
    targetDurationSeconds: params.targetDurationSeconds,
    outputIntent: "storyboard_rows",
    writingGroupRulePackIds: [...params.kbSummary.writing_group_rule_pack_ids],
    directorGroupRulePackIds: [...params.kbSummary.director_group_rule_pack_ids],
    kbSnapshotHash: params.kbSummary.kb_snapshot_hash,
    generationGroup: `第一组分镜生成 / ${params.sceneTypeLabel}`,
    estimatedShotCount: durationProfile.targetShotCount,
    durationAllocationStrategy: `${params.targetDurationSeconds} 秒固定时长 -> ${durationProfile.targetShotCount} 镜 / ${durationProfile.pacingDirective}`,
    continuityStatus: params.existingTask?.stale_state
      ? `存在 stale 任务：${params.existingTask.stale_reasons.join(" / ")}`
      : "从当前 confirmed body 创建，连续性以 accepted body 为事实源。",
    createFromConfirmedBody: true,
    preservePreviousTask: true,
    storyboardTaskHash: await sha256Hex(identitySeed),
    selectedTaskId: selectedCandidate.id,
    selectedTaskStatus: "pending",
    selectedTaskSourceKind: selectedCandidate.sourceKind,
    selectedTaskSourceLabel: selectedCandidate.sourceLabel,
    peopleSummary: selectedCandidate.peopleSummary,
    scriptFragment: selectedCandidate.fragment,
    originalFragment: selectedCandidate.fragment,
    candidates,
  };
}

void buildTaskCreationDraft;

function stableTaskSeed(params: {
  acceptedBodyHash: string;
  sceneTypeId: string;
  targetDurationSeconds: number;
  kbSnapshotHash: string;
  writingGroupRulePackIds: string[];
  directorGroupRulePackIds: string[];
  outputIntent: "storyboard_rows";
}): string {
  return JSON.stringify(params);
}

function createStaleTask(task: StoryboardTask | null, reason: string): StoryboardTask | null {
  if (!task) {
    return null;
  }
  const staleReasons = Array.from(new Set([...task.stale_reasons, reason]));
  return {
    ...task,
    stale_state: true,
    sync_status: "stale",
    stale_reasons: staleReasons,
  };
}

function buildTrace(params: {
  provider: ProviderFormState;
  sceneTypeId: string;
  targetDuration: number;
  acceptedBody: string;
  bodyEvidence: GenerationEvidence | null;
  storyboardTask: StoryboardTask | null;
  kbSummary: SanitizedKbSummary | null;
  rows: StoryboardRow[];
  storyboardEvidence: GenerationEvidence | null;
  exportAudits: ExportAudit[];
}): QaTrace {
  return {
    provider_id: params.provider.providerId,
    model_id: params.provider.directorModel,
    base_url_host: deriveProviderModelRecord(params.provider).base_url_host,
    connection_status: params.provider.connectionStatus,
    connection_message: params.provider.connectionMessage,
    scene_type_id: params.sceneTypeId,
    scene_type_label: getSceneTypeLabel(params.sceneTypeId),
    target_duration_seconds: params.targetDuration,
    accepted_body_ready: Boolean(params.acceptedBody.trim()),
    accepted_body_hash: params.bodyEvidence?.accepted_body_hash ?? "",
    storyboard_task_hash: params.storyboardTask?.task_hash ?? "",
    storyboard_task_sync_status: params.storyboardTask?.sync_status ?? "missing",
    source_lineage: params.storyboardEvidence?.source_lineage ?? params.bodyEvidence?.source_lineage ?? "user_source_text",
    kb_ready: Boolean(params.kbSummary),
    kb_snapshot_hash: params.kbSummary?.kb_snapshot_hash ?? "",
    selected_sample_ids: params.kbSummary?.selected_sample_ids ?? [],
    selected_kb_rules: params.kbSummary?.selected_kb_rules ?? [],
    writing_group_rule_pack_ids: params.kbSummary?.writing_group_rule_pack_ids ?? [],
    director_group_rule_pack_ids: params.kbSummary?.director_group_rule_pack_ids ?? [],
    kb_context_summary: params.kbSummary?.kb_context_summary ?? "",
    applied_to: params.kbSummary?.applied_to ?? [],
    influence_axes: params.kbSummary?.influence_axes ?? [],
    kb_oracle_affects_structure: true,
    raw_kb_rows_included: params.kbSummary?.raw_kb_rows_included ?? 0,
    raw_sample_text_absent: params.kbSummary?.raw_sample_text_absent ?? true,
    source_register_absent: params.kbSummary?.source_register_absent ?? true,
    overlay_json_absent: params.kbSummary?.overlay_json_absent ?? true,
    prompt_body_absent: params.kbSummary?.prompt_body_absent ?? true,
    rows_count: params.rows.length,
    rows_match: params.storyboardEvidence?.rows_match ?? false,
    prompt_text_present: params.storyboardEvidence?.prompt_text_present ?? false,
    prompt_text_boundary_passed: params.storyboardEvidence?.prompt_text_boundary_passed ?? false,
    prompt_text_not_summary_only: params.storyboardEvidence?.prompt_text_not_summary_only ?? false,
    visual_description_visible_frame_passed:
      params.storyboardEvidence?.visual_description_visible_frame_passed ?? false,
    visual_description_no_trace: params.storyboardEvidence?.visual_description_no_trace ?? false,
    prompt_compiled_after_final_row: params.storyboardEvidence?.prompt_compiled_after_final_row ?? false,
    stale_task_detected: params.storyboardEvidence?.stale_task_detected ?? ((params.storyboardTask?.sync_status ?? "missing") === "stale"),
    stale_rows_detected: params.storyboardEvidence?.stale_rows_detected ?? params.rows.some((row) => row.status === "stale"),
    person_field_valid: params.storyboardEvidence?.person_field_valid ?? false,
    location_not_in_person: params.storyboardEvidence?.location_not_in_person ?? false,
    action_fragment_not_in_person: params.storyboardEvidence?.action_fragment_not_in_person ?? false,
    scene_term_not_in_person: params.storyboardEvidence?.scene_term_not_in_person ?? false,
    generic_role_placeholder_absent: params.storyboardEvidence?.generic_role_placeholder_absent ?? false,
    validator_pseudo_success_detected: params.storyboardEvidence?.validator_pseudo_success_detected ?? false,
    warning_taxonomy_classified: params.storyboardEvidence?.warning_taxonomy_classified ?? params.bodyEvidence?.warning_taxonomy_classified ?? false,
    warning_taxonomy: params.storyboardEvidence?.warning_taxonomy ?? params.bodyEvidence?.warning_taxonomy ?? [],
    hardfail_warning_absent: params.storyboardEvidence?.hardfail_warning_absent ?? false,
    fallback_used: params.storyboardEvidence?.fallback_used ?? false,
    provider_failover_used: params.storyboardEvidence?.provider_failover_used ?? false,
    local_candidate: params.storyboardEvidence?.local_candidate ?? false,
    export_audit: params.exportAudits,
  };
}

function cloneRows(rows: StoryboardRow[]): StoryboardRow[] {
  return rows.map((row) => ({ ...row }));
}

function normalizeShotIndexes(rows: StoryboardRow[]): StoryboardRow[] {
  return rows.map((row, index) => ({
    ...row,
    shot_index: index + 1,
  }));
}

async function buildTaskCreationDraftProduct(params: {
  acceptedBody: string;
  bodyEvidence: GenerationEvidence;
  sceneTypeId: string;
  sceneTypeLabel: string;
  targetDurationSeconds: number;
  kbSummary: SanitizedKbSummary;
  existingTask: StoryboardTask | null;
}): Promise<TaskCreationDraft> {
  const durationProfile = getDurationProfile(params.targetDurationSeconds);
  const acceptedBodyHash = params.bodyEvidence.accepted_body_hash || (await sha256Hex(params.acceptedBody));
  const taskIndex = (params.existingTask?.task_index ?? 0) + 1;
  const candidates: TaskCandidate[] = splitCandidateFragments(params.acceptedBody)
    .slice(0, 3)
    .map((fragment, index) => ({
      id: `candidate-${index + 1}`,
      title: `场景候选 ${index + 1}`,
      summary: compactText(fragment, 86),
      fragment,
      durationSeconds: params.targetDurationSeconds,
      sourceKind: "system_candidate",
      sourceLabel: "系统候选",
      peopleSummary: summarizePeople(fragment),
      queueStatus: index === 0 ? "current" : "available",
    }));
  candidates.push({
    id: "full-narrative",
    title: "完整扩写剧本",
    summary: compactText(params.acceptedBody, 86),
    fragment: params.acceptedBody,
    durationSeconds: params.targetDurationSeconds,
    sourceKind: "full_narrative",
    sourceLabel: "完整扩写剧本",
    peopleSummary: summarizePeople(params.acceptedBody),
    queueStatus: "available",
  });

  const selectedCandidate = candidates[0];
  const identitySeed = stableTaskSeed({
    acceptedBodyHash,
    sceneTypeId: params.sceneTypeId,
    targetDurationSeconds: params.targetDurationSeconds,
    kbSnapshotHash: params.kbSummary.kb_snapshot_hash,
    writingGroupRulePackIds: params.kbSummary.writing_group_rule_pack_ids,
    directorGroupRulePackIds: params.kbSummary.director_group_rule_pack_ids,
    outputIntent: "storyboard_rows",
  });

  return {
    taskName: `第 ${taskIndex} 个镜头任务`,
    taskIndex,
    currentBodySource: "accepted_body",
    acceptedBodyHash,
    sceneTypeId: params.sceneTypeId,
    sceneTypeLabel: params.sceneTypeLabel,
    targetDurationSeconds: params.targetDurationSeconds,
    outputIntent: "storyboard_rows",
    writingGroupRulePackIds: [...params.kbSummary.writing_group_rule_pack_ids],
    directorGroupRulePackIds: [...params.kbSummary.director_group_rule_pack_ids],
    kbSnapshotHash: params.kbSummary.kb_snapshot_hash,
    generationGroup: `第一组分镜生成 / ${params.sceneTypeLabel}`,
    estimatedShotCount: durationProfile.targetShotCount,
    durationAllocationStrategy: `${params.targetDurationSeconds} 秒固定时长 -> ${durationProfile.targetShotCount} 镜 / ${durationProfile.pacingDirective}`,
    continuityStatus: params.existingTask?.stale_state
      ? `存在旧任务待同步：${params.existingTask.stale_reasons.join(" / ")}`
      : "从当前确认后的正文创建，连续性以 accepted body 为事实源。",
    createFromConfirmedBody: true,
    preservePreviousTask: true,
    storyboardTaskHash: await sha256Hex(identitySeed),
    selectedTaskId: selectedCandidate.id,
    selectedTaskStatus: "pending",
    selectedTaskSourceKind: selectedCandidate.sourceKind,
    selectedTaskSourceLabel: selectedCandidate.sourceLabel,
    peopleSummary: selectedCandidate.peopleSummary,
    scriptFragment: selectedCandidate.fragment,
    originalFragment: selectedCandidate.fragment,
    candidates,
  };
}

function Modal({
  title,
  children,
  onClose,
  panelClassName = "",
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
  panelClassName?: string;
}) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className={`modal-panel ${panelClassName}`.trim()} onClick={(event) => event.stopPropagation()}>
        <div className="panel-title-row">
          <h2>{title}</h2>
          <button className="ghost-button" onClick={onClose}>
            关闭
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}

export default function App() {
  const [provider, setProvider] = useState<ProviderFormState>(() => createDefaultProviderState());
  const [sourceText, setSourceText] = useState(DEFAULT_SOURCE_TEXT);
  const [sourceFileName, setSourceFileName] = useState("");
  const [sceneTypeId, setSceneTypeId] = useState("hot_blood_battle");
  const [targetDuration, setTargetDuration] = useState(15);
  const [operationMode, setOperationMode] = useState<OperationMode>("expand_story");
  const [kbSummary, setKbSummary] = useState<SanitizedKbSummary | null>(null);
  const [kbStatus, setKbStatus] = useState<AsyncStatus>("idle");
  const [bodyDraft, setBodyDraft] = useState("");
  const [, setNarrativeTitle] = useState("");
  const [acceptedBody, setAcceptedBody] = useState("");
  const [bodyStatus, setBodyStatus] = useState<AsyncStatus>("idle");
  const [bodyValidation, setBodyValidation] = useState<ValidationResult | null>(null);
  const [bodyEvidence, setBodyEvidence] = useState<GenerationEvidence | null>(null);
  const [storyboardTask, setStoryboardTask] = useState<StoryboardTask | null>(null);
  const [taskStatus, setTaskStatus] = useState("尚未创建镜头任务");
  const [storyboardStatus, setStoryboardStatus] = useState<AsyncStatus>("idle");
  const [rows, setRows] = useState<StoryboardRow[]>([]);
  const [confirmedRows, setConfirmedRows] = useState<StoryboardRow[]>([]);
  const [rowValidation, setRowValidation] = useState<ValidationResult | null>(null);
  const [storyboardEvidence, setStoryboardEvidence] = useState<GenerationEvidence | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<AsyncStatus>("idle");
  const [workspaceNotice, setWorkspaceNotice] = useState("先完成 API 配置和连接测试，再走正文、镜头任务与导出链路。");
  const [exportNotice, setExportNotice] = useState("导出区待命");
  const [, setSavedAt] = useState("");
  const [exportAudits, setExportAudits] = useState<ExportAudit[]>([]);
  const [activeModal, setActiveModal] = useState<ModalKind>(null);
  const [taskCreationDraft, setTaskCreationDraft] = useState<TaskCreationDraft | null>(null);
  const [taskLibrary, setTaskLibrary] = useState<TaskArchive[]>([]);
  const [editingRowIndex, setEditingRowIndex] = useState<number | null>(null);
  const [editingRowDraft, setEditingRowDraft] = useState<StoryboardRow | null>(null);
  const [editingRowOriginal, setEditingRowOriginal] = useState<StoryboardRow | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    setProvider(loadProviderState());
  }, []);

  useEffect(() => {
    let active = true;
    setKbStatus("loading");
    buildSanitizedKbSummary(sceneTypeId, targetDuration)
      .then((summary) => {
        if (!active) {
          return;
        }
        setKbSummary(summary);
        setKbStatus("ready");
      })
      .catch(() => {
        if (!active) {
          return;
        }
        setKbSummary(null);
        setKbStatus("failed");
      });
    return () => {
      active = false;
    };
  }, [sceneTypeId, targetDuration]);

  const sceneTypeLabel = getSceneTypeLabel(sceneTypeId);
  const acceptedFacts = useMemo(() => extractSourceFacts(acceptedBody || bodyDraft || sourceText), [acceptedBody, bodyDraft, sourceText]);
  void acceptedFacts;
  const providerDefinition = getProviderDefinition(provider.providerId);
  const currentModelOptions = Array.from(
    new Set([provider.model, ...providerDefinition.defaultModels, "custom-model"].filter(Boolean)),
  );
  const providerMetaLabel = `${providerDefinition.displayName} 路 ${provider.model}`;
  const providerReservedWarning = providerDefinition.transportCapability === "unsupported" ? "预留：已配置未启用" : "";

  const providerReady = provider.connectionStatus === "ready";
  const kbReady = kbStatus === "ready" && kbSummary !== null;
  const bodyReady = Boolean(bodyDraft.trim()) && Boolean(bodyValidation?.passed);
  const acceptedBodyReady = Boolean(acceptedBody.trim());
  const acceptedBodyIsCurrent = acceptedBodyReady && acceptedBody.trim() === bodyDraft.trim();
  const taskReady = Boolean(storyboardTask && storyboardTask.sync_status === "fresh");
  const rowsReady = rows.length > 0 && Boolean(rowValidation?.passed);
  const hasArchivedTasks = taskLibrary.length > 0;
  const confirmedRowsReady =
    confirmedRows.length > 0 &&
    Boolean(storyboardEvidence?.rows_match) &&
    Boolean(storyboardEvidence?.prompt_text_present) &&
    Boolean(storyboardEvidence?.visual_description_visible_frame_passed) &&
    Boolean(storyboardEvidence?.prompt_compiled_after_final_row) &&
    !Boolean(storyboardEvidence?.stale_state);
  const finalizedDuration = confirmedRows.reduce((sum, row) => sum + row.duration_seconds, 0);
  const scriptSurfaceIsDraft = Boolean(bodyDraft.trim()) || bodyStatus !== "idle" || acceptedBodyReady;
  const scriptSurfaceValue = scriptSurfaceIsDraft ? bodyDraft : sourceText;
  const scriptSurfaceLabel = scriptSurfaceIsDraft ? "正文草稿" : "正文输入";
  const scriptSurfaceMeta = sourceFileName ? `材料来源：${sourceFileName}` : "材料来源：手动输入";
  const scriptPreviewStatus =
    bodyStatus === "loading"
      ? "正在生成正文草稿，请等待当前结果返回。"
      : bodyReady
        ? "已识别为故事梗概，将扩写后生成脚本。待确定使用。"
        : acceptedBodyReady
          ? "确认稿已冻结，后续改动会让镜头任务和导出失效。"
        : workspaceNotice;
  const currentShotDurationLabel = storyboardTask ? `${targetDuration} 秒` : "待确认";
  const currentShotStatusLabel = !storyboardTask
    ? "未导入"
    : storyboardTask.sync_status === "fresh"
      ? "已导入"
      : "待重新确认";
  const canTriggerGenerate = providerReady && kbReady && acceptedBodyReady && taskReady && storyboardStatus !== "loading";
  const currentContext: GenerationContext | null = kbSummary
    ? {
        sourceText: operationMode === "rewrite_script" && acceptedBodyReady ? acceptedBody : sourceText,
        sceneTypeId,
        sceneTypeLabel,
        targetDuration,
        mode: operationMode,
        provider,
        kbSummary,
      }
    : null;

  useEffect(() => {
    const trace = buildTrace({
      provider,
      sceneTypeId,
      targetDuration,
      acceptedBody,
      bodyEvidence,
      storyboardTask,
      kbSummary,
      rows,
      storyboardEvidence,
      exportAudits,
    });
    const element = document.getElementById("hope-qa-trace");
    if (element) {
      element.setAttribute("data-hope-qa-trace", JSON.stringify(trace));
    }
    (window as Window & { __hopeQaTrace?: QaTrace }).__hopeQaTrace = trace;
  }, [
    acceptedBody,
    bodyEvidence,
    exportAudits,
    kbSummary,
    provider,
    rows,
    sceneTypeId,
    storyboardEvidence,
    storyboardTask,
      targetDuration,
  ]);

  useEffect(() => {
    if (!storyboardTask) {
      return;
    }
    const snapshot: TaskArchive = {
      task: storyboardTask,
      sourceText,
      acceptedBody,
      bodyEvidence,
      rows: cloneRows(rows),
      confirmedRows: cloneRows(confirmedRows),
      rowValidation,
      storyboardEvidence,
    };
    setTaskLibrary((current) => {
      const next = [snapshot, ...current.filter((item) => item.task.task_id !== storyboardTask.task_id)];
      return next.slice(0, 8);
    });
  }, [acceptedBody, bodyEvidence, confirmedRows, rowValidation, rows, sourceText, storyboardEvidence, storyboardTask]);

  useEffect(() => {
    const buttons = Array.from(document.querySelectorAll<HTMLButtonElement>("button"));
    const importButton = buttons.find((button) => button.textContent?.includes("导入镜头任务"));
    if (!importButton) {
      return;
    }
    importButton.dataset.testid = "import-task-button";
    importButton.disabled = !hasArchivedTasks;
    const handleImport = (event: Event) => {
      if (!hasArchivedTasks) {
        return;
      }
      event.preventDefault();
      event.stopPropagation();
      setActiveModal("taskImport");
    };
    importButton.addEventListener("click", handleImport, true);
    return () => {
      importButton.removeEventListener("click", handleImport, true);
    };
  }, [hasArchivedTasks]);

  function patchProvider(nextPatch: Partial<ProviderFormState>) {
    setProvider((current) => {
      const next = { ...current, ...nextPatch };
      return {
        ...next,
        connectionStatus: deriveConnectionStatus(next),
      };
    });
  }

  function resetDownstream(reason: string, options?: { clearBodyDraft?: boolean; clearAcceptedBody?: boolean }) {
    if (options?.clearBodyDraft) {
      setBodyDraft("");
      setNarrativeTitle("");
      setBodyStatus("idle");
      setBodyValidation(null);
      setBodyEvidence(null);
    }
    if (options?.clearAcceptedBody) {
      setAcceptedBody("");
    }
    setStoryboardTask((current) => createStaleTask(current, reason));
    setRows((current) => current.map((row) => ({ ...row, status: "stale" })));
    setRowValidation(null);
    setEditingRowIndex(null);
    setEditingRowDraft(null);
    setEditingRowIndex(null);
    setEditingRowDraft(null);
    setSavedAt("");
    setTaskStatus(reason);
    setExportNotice(`导出已失效：${reason}`);
  }

  function handleSceneTypeChange(value: string) {
    setSceneTypeId(value);
    if (acceptedBodyReady || storyboardTask || rows.length > 0) {
      resetDownstream("scene_type 已变更，旧任务、旧 rows 与旧导出已全部失效。");
    }
  }

  function handleDurationChange(value: number) {
    setTargetDuration(value);
    if (acceptedBodyReady || storyboardTask || rows.length > 0) {
      resetDownstream("target_duration 已变更，旧任务、旧 rows 与旧导出已全部失效。");
    }
  }

  async function handleTestConnection() {
    setConnectionStatus("loading");
    setWorkspaceNotice("正在测试 API 连接与浏览器直连可用性。");
    const result = await testProviderConnection(provider);
    patchProvider({
      connectionStatus: result.status,
      connectionMessage: result.message,
      corsCheckResult: result.corsCheckResult,
      lastConnectionTest: result.lastConnectionTest,
    });
    setConnectionStatus(result.status === "ready" ? "ready" : "failed");
    setWorkspaceNotice(result.message);
  }

  function handleSaveProviderConfig() {
    saveProviderState(provider);
    setWorkspaceNotice(
      provider.persistApiKey
        ? "配置已保存到本机浏览器；密钥会写入 localStorage，请只在可信设备使用。"
        : "配置已保存；API Key 仅保留在当前会话，不会写入 dist、README、导出或日志。",
    );
  }

  function handleSourceTextChange(value: string) {
    setSourceText(value);
    setWorkspaceNotice("原始文本已更新，正文候选、镜头任务和导出都需要重新生成。");
    resetDownstream("原始文本已变更，accepted body 与下游链路全部失效。", {
      clearBodyDraft: true,
      clearAcceptedBody: true,
    });
  }

  function handleScriptSurfaceChange(value: string) {
    if (scriptSurfaceIsDraft) {
      handleBodyDraftChange(value);
      return;
    }
    handleSourceTextChange(value);
  }

  function handleBodyDraftChange(value: string) {
    const validation = validateNarrativeBody(value);
    setBodyDraft(value);
    setBodyValidation(validation);
    if (acceptedBodyReady || storyboardTask || rows.length > 0) {
      setAcceptedBody("");
      setStoryboardTask((current) => createStaleTask(current, "正文草稿已修改，accepted body 与下游链路已失效。"));
      setRows((current) => current.map((row) => ({ ...row, status: "stale" })));
      setConfirmedRows([]);
      setStoryboardEvidence(null);
      setRowValidation(null);
      setTaskStatus("正文草稿已修改，需要重新“确定使用”并重建镜头任务。");
      setExportNotice("导出已失效：accepted body 已被修改。");
    }
  }

  async function handleGenerateBody(mode: OperationMode) {
    if (!currentContext || !kbSummary || !providerReady) {
      return;
    }

    const sourceForWriting = mode === "rewrite_script" && acceptedBodyReady ? acceptedBody : sourceText;
    const factsForWriting = extractSourceFacts(sourceForWriting);

    setOperationMode(mode);
    setBodyStatus("loading");
    setWorkspaceNotice(mode === "expand_story" ? "写作组正在扩写正文。" : "写作组正在改写正文。");
    setAcceptedBody("");
    setStoryboardTask(null);
    setRows([]);
    setConfirmedRows([]);
    setRowValidation(null);
    setStoryboardEvidence(null);
    setExportAudits([]);
    try {
      const response = await runChatCompletion<string>(
        provider,
        provider.writingModel,
        buildWritingMessages({ ...currentContext, sourceText: sourceForWriting, mode }, factsForWriting),
        { expectJson: true },
      );
      const parsed = parseJsonResponse<{ title?: string; body?: string }>(response.content);
      const nextTitle = String(parsed.title ?? "未命名正文").trim();
      const nextBody = String(parsed.body ?? "").trim();
      const validation = validateNarrativeBody(nextBody);
      const evidence = await buildEvidence({
        sourceText: sourceForWriting,
        acceptedBody: nextBody,
        rows: [],
        kbSummary,
        provider,
        modelId: provider.writingModel,
        sceneTypeId,
        targetDurationSeconds: targetDuration,
        validatorResult: validation,
        outputIntent: "narrative_body",
        task: null,
      });
      setNarrativeTitle(nextTitle);
      setBodyDraft(nextBody);
      setBodyValidation(validation);
      setBodyEvidence(evidence);
      setBodyStatus(validation.passed ? "ready" : "failed");
      setWorkspaceNotice(validation.summary);
    } catch (error) {
      setBodyStatus("failed");
      setBodyValidation({
        passed: false,
        summary: error instanceof Error ? error.message : String(error),
        issues: [
          {
            code: "body_generation_failed",
            message: error instanceof Error ? error.message : String(error),
            severity: "error",
          },
        ],
      });
      setWorkspaceNotice("正文生成失败，请检查 API 配置、网络、CORS 或输入文本。");
    }
  }

  async function handleAcceptBody() {
    if (!kbSummary || !bodyDraft.trim()) {
      return;
    }
    const validation = validateNarrativeBody(bodyDraft);
    if (!validation.passed) {
      setBodyValidation(validation);
      return;
    }
    const evidence = await buildEvidence({
      sourceText,
      acceptedBody: bodyDraft,
      rows: [],
      kbSummary,
      provider,
      modelId: provider.writingModel,
      sceneTypeId,
      targetDurationSeconds: targetDuration,
      validatorResult: validation,
      outputIntent: "narrative_body",
      task: null,
    });
    setAcceptedBody(bodyDraft);
    setBodyEvidence(evidence);
    setRows([]);
    setConfirmedRows([]);
    setStoryboardTask(null);
    setRowValidation(null);
    setStoryboardEvidence(null);
    setEditingRowIndex(null);
    setEditingRowDraft(null);
    setTaskStatus("accepted body 已锁定，可以创建真实镜头任务。");
    setWorkspaceNotice("accepted body 已确认，成为新的唯一事实源。");
    setExportNotice("导出区待命：请先新建镜头任务并生成分镜。");
  }

  async function handleCreateStoryboardTask() {
    if (!kbSummary) {
      setTaskStatus("暂时无法新建镜头任务：KB 摘要还没有就绪。");
      setWorkspaceNotice("请等待 KB 摘要加载完成后，再新建镜头任务。");
      return;
    }
    if (!bodyDraft.trim()) {
      setTaskStatus("暂时无法新建镜头任务：还没有可确认的正文。");
      setWorkspaceNotice("请先输入正文，或点击“扩写故事 / 改写剧本”生成正文草稿。");
      return;
    }
    if (!acceptedBodyReady) {
      setTaskStatus("暂时无法新建镜头任务：当前正文还没有“确定使用”。");
      setWorkspaceNotice("请先点击“确定使用”冻结 accepted body，再新建镜头任务。");
      return;
    }
    if (!bodyEvidence) {
      setTaskStatus("暂时无法新建镜头任务：缺少 accepted body 证据。");
      setWorkspaceNotice("请重新点击“确定使用”，同步正文快照后再新建镜头任务。");
      return;
    }
    const nextDraft = await buildTaskCreationDraftProduct({
      acceptedBody,
      bodyEvidence,
      sceneTypeId,
      sceneTypeLabel,
      targetDurationSeconds: targetDuration,
      kbSummary,
      existingTask: storyboardTask,
    });
    setTaskCreationDraft(nextDraft);
    setActiveModal("taskCreate");
    setTaskStatus("镜头任务待确认：请在弹窗内核对正文候选、时长和任务队列。");
    setWorkspaceNotice("新建镜头任务弹窗已打开，确认后才会创建真实 storyboard task。");
  }

  function handleTaskDraftCandidateSelect(candidateId: string) {
    setTaskCreationDraft((current) => {
      if (!current) {
        return current;
      }
      const selectedCandidate = current.candidates.find((item) => item.id === candidateId);
      if (!selectedCandidate) {
        return current;
      }
      return {
        ...current,
        selectedTaskId: selectedCandidate.id,
        selectedTaskSourceKind: selectedCandidate.sourceKind,
        selectedTaskSourceLabel: selectedCandidate.sourceLabel,
        selectedTaskStatus:
          selectedCandidate.queueStatus === "queued"
            ? "queued"
            : selectedCandidate.queueStatus === "current"
              ? "pending"
              : "pending",
        peopleSummary: selectedCandidate.peopleSummary,
        scriptFragment: selectedCandidate.fragment,
        originalFragment: selectedCandidate.fragment,
      };
    });
  }

  function handleTaskDraftRestoreOriginal() {
    setTaskCreationDraft((current) => (current ? { ...current, scriptFragment: current.originalFragment } : current));
  }

  function handleTaskDraftFragmentChange(value: string) {
    setTaskCreationDraft((current) => {
      if (!current) {
        return current;
      }
      const isOriginal = value.trim() === current.originalFragment.trim();
      return {
        ...current,
        scriptFragment: value,
        selectedTaskSourceKind: isOriginal ? current.selectedTaskSourceKind : "custom_fragment",
        selectedTaskSourceLabel: isOriginal ? current.selectedTaskSourceLabel : "自定义镜头片段",
        peopleSummary: summarizePeople(value),
      };
    });
  }

  function handleTaskDraftDurationChange(value: number) {
    const durationProfile = getDurationProfile(value);
    setTaskCreationDraft((current) =>
      current
        ? {
            ...current,
            targetDurationSeconds: value,
            estimatedShotCount: durationProfile.targetShotCount,
            durationAllocationStrategy: `${value} 秒固定时长 -> ${durationProfile.targetShotCount} 镜 / ${durationProfile.pacingDirective}`,
            candidates: current.candidates.map((candidate) => ({ ...candidate, durationSeconds: value })),
          }
        : current,
    );
  }

  function handleTaskDraftRetiming() {
    setTaskCreationDraft((current) =>
      current
        ? {
            ...current,
            scriptFragment: retimeTaskFragment(current.scriptFragment, current.targetDurationSeconds),
          }
        : current,
    );
  }

  async function handleConfirmStoryboardTaskCreate(options?: { keepOpen?: boolean }) {
    if (!kbSummary || !taskCreationDraft) {
      return;
    }
    const task = await createStoryboardTask({
      acceptedBodyHash: taskCreationDraft.acceptedBodyHash,
      sceneTypeId: taskCreationDraft.sceneTypeId,
      targetDurationSeconds: taskCreationDraft.targetDurationSeconds,
      kbSummary,
      taskName: taskCreationDraft.taskName,
      taskIndex: taskCreationDraft.taskIndex,
      taskSourceKind: taskCreationDraft.selectedTaskSourceKind,
      taskSourceLabel: taskCreationDraft.selectedTaskSourceLabel,
      peopleSummary: taskCreationDraft.peopleSummary,
      scriptFragment: taskCreationDraft.scriptFragment,
      originalFragment: taskCreationDraft.originalFragment,
      currentBodySource: taskCreationDraft.currentBodySource,
      generationGroup: taskCreationDraft.generationGroup,
      estimatedShotCount: taskCreationDraft.estimatedShotCount,
      durationAllocationStrategy: taskCreationDraft.durationAllocationStrategy,
      continuityStatus: taskCreationDraft.continuityStatus,
      createdFromConfirmedBody: taskCreationDraft.createFromConfirmedBody,
      preservePreviousTask: taskCreationDraft.preservePreviousTask,
    });
    if (storyboardTask && !taskCreationDraft.preservePreviousTask) {
      setTaskLibrary((current) => current.filter((item) => item.task.task_id !== storyboardTask.task_id));
    }
    setStoryboardTask(task);
    setRows([]);
    setConfirmedRows([]);
    setRowValidation(null);
    setStoryboardEvidence(null);
    setEditingRowIndex(null);
    setEditingRowDraft(null);
    setExportAudits([]);
    setTaskStatus(`已创建镜头任务“${task.task_name}”，现在可以开始生成。`);
    setWorkspaceNotice("storyboard task 已绑定 accepted body、scene_type、duration、KB 规则、输出意图与 source lineage。");
    setExportNotice("导出区待命：请先开始生成，再保存修改并确认当前镜头。");
    if (options?.keepOpen) {
      setTaskCreationDraft((current) =>
        current
          ? {
              ...current,
              taskIndex: task.task_index + 1,
              taskName: `第 ${task.task_index + 1} 个镜头任务`,
              selectedTaskStatus: "queued",
              candidates: current.candidates.map((candidate) =>
                candidate.id === current.selectedTaskId
                  ? { ...candidate, queueStatus: "queued" }
                  : candidate,
              ),
            }
          : current,
      );
      return;
    }
    setActiveModal(null);
    setTaskCreationDraft(null);
  }

  function handleImportStoryboardTask(taskId: string) {
    const snapshot = taskLibrary.find((item) => item.task.task_id === taskId);
    if (!snapshot) {
      setTaskStatus("导入镜头任务失败：未找到可恢复的任务快照。");
      return;
    }
    setSceneTypeId(snapshot.task.scene_type_id);
    setTargetDuration(snapshot.task.target_duration_seconds);
    setSourceText(snapshot.sourceText);
    setBodyDraft(snapshot.acceptedBody);
    setAcceptedBody(snapshot.acceptedBody);
    setBodyEvidence(snapshot.bodyEvidence);
    setStoryboardTask(snapshot.task);
    setRows(cloneRows(snapshot.rows));
    setConfirmedRows(cloneRows(snapshot.confirmedRows));
    setRowValidation(snapshot.rowValidation);
    setStoryboardEvidence(snapshot.storyboardEvidence);
    setEditingRowIndex(null);
    setEditingRowDraft(null);
    setEditingRowOriginal(null);
    setExportAudits([]);
    setTaskStatus(`已导入镜头任务“${snapshot.task.task_name}”，source lineage 已恢复。`);
    setWorkspaceNotice("已恢复 accepted body、storyboard task、分镜 rows 与 source lineage，可继续开始生成或保存修改。");
    setExportNotice(snapshot.confirmedRows.length > 0 ? "已恢复已确认分镜，请复核后导出。" : "导入的任务尚未带已确认分镜，请先开始生成或保存修改。");
    setActiveModal(null);
  }

  async function handleGenerateStoryboard() {
    if (!currentContext || !kbSummary || !storyboardTask || storyboardTask.sync_status !== "fresh" || !providerReady) {
      return;
    }

    const taskScriptFragment = storyboardTask.script_fragment.trim() || acceptedBody;
    const taskFacts = extractSourceFacts(taskScriptFragment);
    setStoryboardStatus("loading");
    setWorkspaceNotice("导演组正在根据当前 accepted body 与 storyboard task 生成分镜。");
    try {
      const response = await runChatCompletion<string>(
        provider,
        provider.directorModel,
        buildDirectorMessages(currentContext, taskFacts, taskScriptFragment, storyboardTask),
        { expectJson: true, timeoutMs: 180000 },
      );
      const parsed = parseJsonResponse<{ rows?: unknown[] }>(response.content);
      const normalizedRows = normalizeStoryboardRows(parsed.rows ?? []);
      const nextRows = enrichRows(normalizedRows, taskScriptFragment, kbSummary, targetDuration);
      const validation = normalizeAndValidateStoryboardRows(nextRows, kbSummary, targetDuration, taskScriptFragment);
      const provisionalTask: StoryboardTask = {
        ...storyboardTask,
        queue_status: "generated",
        stale_state: !validation.passed,
        sync_status: validation.passed ? "fresh" : "stale",
        stale_reasons: validation.passed ? [] : ["分镜结果未通过校验，当前任务已转为 stale。"],
      };
      const evidence = await buildEvidence({
        sourceText,
        acceptedBody,
        rows: nextRows,
        kbSummary,
        provider,
        modelId: provider.directorModel,
        sceneTypeId,
        targetDurationSeconds: targetDuration,
        validatorResult: validation,
        outputIntent: "storyboard_rows",
        task: provisionalTask,
      });
      const nextTask: StoryboardTask = {
        ...provisionalTask,
        rows_hash: evidence.rows_hash,
        confirmed_row_hash: "",
        stale_state: !validation.passed,
        sync_status: validation.passed ? "fresh" : "stale",
        stale_reasons: validation.passed ? [] : ["分镜结果未通过校验，当前任务已转为 stale。"],
      };

      startTransition(() => {
        setRows(nextRows);
        setConfirmedRows([]);
      });
      setEditingRowIndex(null);
      setEditingRowDraft(null);
      setStoryboardTask(nextTask);
      setRowValidation(validation);
      setStoryboardEvidence(evidence);
      setStoryboardStatus(validation.passed ? "ready" : "failed");
      setTaskStatus(validation.passed ? `已生成 ${nextRows.length} 条分镜，当前结果已可确认。` : "分镜生成完成，但校验未通过。");
      setWorkspaceNotice(validation.summary);
      setExportNotice(validation.passed ? "当前分镜结果已生成，请先点击“保存修改”确认当前镜头后再导出。" : "导出已阻断：分镜未通过校验。");
    } catch (error) {
      setStoryboardStatus("failed");
      setRowValidation({
        passed: false,
        summary: error instanceof Error ? error.message : String(error),
        issues: [
          {
            code: "storyboard_generation_failed",
            message: error instanceof Error ? error.message : String(error),
            severity: "error",
          },
        ],
      });
      setTaskStatus("分镜生成失败，请检查 API、正文质量或模型输出。");
      setWorkspaceNotice("分镜生成失败，请检查连接状态、accepted body 或模型输出。");
    }
  }

  function handleRowFieldChange(field: RowField, value: string) {
    if (editingRowIndex === null || !editingRowDraft || !kbSummary) {
      return;
    }
    let nextRows = rows;
    setEditingRowDraft((current) => {
      if (!current) {
        return current;
      }
      const nextRow: StoryboardRow = {
        ...current,
        [field]: field === "duration_seconds" ? Number(value) || 0 : value,
        is_user_edited: true,
      } as StoryboardRow;

      if (field !== "prompt_text") {
        nextRow.prompt_text = compilePromptText(
          {
            shot_index: nextRow.shot_index,
            person: nextRow.person,
            shot_size: nextRow.shot_size,
            camera: nextRow.camera,
            visual_description: nextRow.visual_description,
            character_action: nextRow.character_action,
            dialogue_or_narration: nextRow.dialogue_or_narration,
            duration_seconds: nextRow.duration_seconds,
            status: nextRow.status,
            note: nextRow.note,
          },
          {
            acceptedBody,
            sceneLabel: kbSummary.scene_type_label,
            sceneProfile: kbSummary.scene_profile,
            durationPacing: getDurationProfile(targetDuration).pacingDirective,
            directorRuleLabels: kbSummary.director_group_rule_pack_ids,
            negativeConstraints: kbSummary.negative_constraints,
          },
        );
      }

      nextRows = rows.map((row, rowIndex) => (rowIndex === editingRowIndex ? nextRow : row));
      return nextRow;
    });

    setRows(nextRows);
    setConfirmedRows([]);
    setStoryboardTask((current) => createStaleTask(current, "分镜表格被人工修改，需要重新确认当前镜头。"));
    setStoryboardEvidence(null);
    setTaskStatus("当前分镜已被人工修改，需要点击“确定使用当前镜头”后才能恢复导出。");
    setExportNotice("导出已阻断：当前分镜有人工修改，尚未重新确认。");
  }

  async function handleConfirmCurrentRows() {
    if (!kbSummary || rows.length === 0 || !storyboardTask) {
      return;
    }
    const validation = normalizeAndValidateStoryboardRows(cloneRows(rows), kbSummary, targetDuration, acceptedBody);
    setRowValidation(validation);
    if (!validation.passed) {
      setWorkspaceNotice(validation.summary);
      return;
    }
    const provisionalTask: StoryboardTask = {
      ...storyboardTask,
      stale_state: false,
      sync_status: "fresh",
      stale_reasons: [],
    };
    const evidence = await buildEvidence({
      sourceText,
      acceptedBody,
      rows,
      kbSummary,
      provider,
      modelId: provider.directorModel,
      sceneTypeId,
      targetDurationSeconds: targetDuration,
      validatorResult: validation,
      outputIntent: "storyboard_rows",
      task: provisionalTask,
    });
    const confirmedRowHash = await sha256Hex(`${evidence.rows_hash}|confirmed`);
    const confirmedEvidence: GenerationEvidence = {
      ...evidence,
      confirmed_row_hash: confirmedRowHash,
      stale_state: false,
      stale_task_detected: false,
      stale_rows_detected: false,
      export_evidence: [
        "export_storyboard_uses_confirmed_rows=true",
        "export_script_uses_confirmed_body_and_rows=true",
      ],
    };
    const refreshedTask: StoryboardTask = {
      ...provisionalTask,
      rows_hash: evidence.rows_hash,
      confirmed_row_hash: confirmedEvidence.confirmed_row_hash,
    };
    setStoryboardTask(refreshedTask);
    setConfirmedRows(cloneRows(rows));
    setStoryboardEvidence(confirmedEvidence);
    setSavedAt(new Date().toLocaleTimeString("zh-CN", { hour12: false }));
    setTaskStatus("当前镜头已确认，导出链路恢复 fresh。");
    setExportNotice("导出区就绪：可以导出分镜词或完整剧本。");
    setWorkspaceNotice(validation.summary);
  }

  function handleEditRow(index: number) {
    const targetRow = rows[index];
    if (!targetRow) {
      return;
    }
    setEditingRowIndex(index);
    setEditingRowDraft({ ...targetRow });
    setEditingRowOriginal({ ...targetRow });
    setTaskStatus("已进入当前分镜行编辑状态，修改后请保存修改。");
  }

  function handleSaveEditedRow(index: number) {
    if (editingRowIndex !== index || !editingRowDraft) {
      return;
    }
    const nextRows = rows.map((row, rowIndex) => (rowIndex === index ? { ...editingRowDraft } : row));
    setRows(nextRows);
    if (kbSummary) {
      setRowValidation(normalizeAndValidateStoryboardRows(cloneRows(nextRows), kbSummary, targetDuration, acceptedBody));
    }
    setEditingRowIndex(null);
    setEditingRowDraft(null);
    setEditingRowOriginal(null);
    setTaskStatus("已保存当前分镜行修改，请继续确认当前镜头。");
  }

  function handleCancelEditedRow(index: number) {
    if (editingRowIndex !== index || !editingRowOriginal || !kbSummary) {
      setEditingRowIndex(null);
      setEditingRowDraft(null);
      setEditingRowOriginal(null);
      return;
    }
    const nextRows = rows.map((row, rowIndex) => (rowIndex === index ? { ...editingRowOriginal } : row));
    setRows(nextRows);
    setConfirmedRows([]);
    setRowValidation(normalizeAndValidateStoryboardRows(cloneRows(nextRows), kbSummary, targetDuration, acceptedBody));
    setEditingRowIndex(null);
    setEditingRowDraft(null);
    setEditingRowOriginal(null);
    setTaskStatus("已取消当前分镜行编辑，恢复为上次保存内容。");
  }

  function handleDuplicateRow(index: number) {
    const baseRow = rows[index];
    if (!baseRow || !kbSummary) {
      return;
    }
    const duplicatedRow: StoryboardRow = {
      ...baseRow,
      status: "edited",
      note: `${baseRow.note} / 已复制`,
      is_user_edited: true,
    };
    const nextRows = normalizeShotIndexes([
      ...rows.slice(0, index + 1),
      duplicatedRow,
      ...rows.slice(index + 1),
    ]);
    setRows(nextRows);
    setConfirmedRows([]);
    setRowValidation(normalizeAndValidateStoryboardRows(cloneRows(nextRows), kbSummary, targetDuration, acceptedBody));
    setStoryboardTask((current) => createStaleTask(current, "分镜行已复制，需要重新确认当前镜头。"));
    setStoryboardEvidence(null);
    setTaskStatus("已复制分镜行，请保存修改后重新确认当前镜头。");
    setExportNotice("导出已阻断：当前分镜行发生变更。");
  }

  function handleConfirmRow(index: number) {
    void handleConfirmCurrentRows();
    setTaskStatus(`已确认当前镜头，当前聚焦第 ${index + 1} 行。`);
  }

  function handleClearStoryboard() {
    setRows([]);
    setConfirmedRows([]);
    setRowValidation(null);
    setStoryboardEvidence(null);
    setStoryboardTask((current) => createStaleTask(current, "已清空分镜结果，需要重新生成。"));
    setTaskStatus("当前分镜结果已清空。");
    setExportNotice("导出区待命：当前没有可导出的内容。");
  }

  async function readTextFile(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onerror = () => reject(new Error("文件读取失败。"));
      reader.onload = () => resolve(String(reader.result ?? ""));
      reader.readAsText(file, "utf-8");
    });
  }

  async function handleImportSource(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) {
      return;
    }
    const text = await readTextFile(file);
    setSourceFileName(file.name);
    handleSourceTextChange(text.trim());
    setWorkspaceNotice(`已导入文本：${file.name}`);
  }

  function recordExport(bundle: ExportBundle) {
    const audit = auditBundle(bundle);
    setExportAudits((current) => [...current.filter((item) => item.file_name !== audit.file_name), audit]);
  }

  function performExport(kind: "storyboard-excel" | "script-excel") {
    if (!kbSummary || !storyboardEvidence || !confirmedRowsReady) {
      return;
    }

    const bundle =
      kind === "storyboard-excel"
        ? createStoryboardExcelExport({
            body: acceptedBody,
            sceneTypeId,
            sceneTypeLabel,
            targetDurationSeconds: targetDuration,
            rows: confirmedRows,
            kbSummary,
            evidence: storyboardEvidence,
          })
        : createScriptExcelExport({
            body: acceptedBody,
            sceneTypeId,
            sceneTypeLabel,
            targetDurationSeconds: targetDuration,
            rows: confirmedRows,
            kbSummary,
            evidence: storyboardEvidence,
          });

    const audit = auditBundle(bundle);
    recordExport(bundle);
    if (storyboardEvidence.stale_state || !storyboardEvidence.rows_match || !storyboardEvidence.prompt_text_present || !storyboardEvidence.visual_description_visible_frame_passed || !storyboardEvidence.prompt_compiled_after_final_row) {
      setExportNotice("导出已阻断：当前确认分镜不满足 fresh/export 条件。");
      return;
    }
    if (audit.contains_secret_leakage || audit.contains_local_path || audit.contains_raw_kb || !audit.usable) {
      setExportNotice("导出已阻断：导出内容存在泄漏或不可用风险。");
      return;
    }
    downloadBundle(bundle);
    setExportNotice(`已导出：${bundle.fileName}`);
  }

  function handleRowEditAction(index: number) {
    handleEditRow(index);
    setTaskStatus("已进入当前分镜行编辑状态，修改后请保存修改。");
  }

  function handleRowDuplicateAction(index: number) {
    const baseRow = rows[index];
    if (!baseRow || !kbSummary) {
      return;
    }
    const duplicatedRow: StoryboardRow = {
      ...baseRow,
      status: "edited",
      note: `${baseRow.note} / 已复制`,
      is_user_edited: true,
    };
    const nextRows = normalizeShotIndexes([
      ...rows.slice(0, index + 1),
      duplicatedRow,
      ...rows.slice(index + 1),
    ]);
    setRows(nextRows);
    setConfirmedRows([]);
    setRowValidation(normalizeAndValidateStoryboardRows(cloneRows(nextRows), kbSummary, targetDuration, acceptedBody));
    setStoryboardTask((current) => createStaleTask(current, "分镜行已复制，需要重新确认当前镜头。"));
    setStoryboardEvidence(null);
    setTaskStatus("已复制分镜行，请保存修改后重新确认当前镜头。");
    setExportNotice("导出已阻断：当前分镜行发生变更。");
  }

  function handleRowDeleteAction(index: number) {
    const nextRows = normalizeShotIndexes(rows.filter((_, rowIndex) => rowIndex !== index));
    setRows(nextRows);
    setConfirmedRows([]);
    if (kbSummary) {
      setRowValidation(normalizeAndValidateStoryboardRows(cloneRows(nextRows), kbSummary, targetDuration, acceptedBody));
    }
    setStoryboardTask((current) => createStaleTask(current, "分镜行已删除，需要重新确认当前镜头。"));
    setStoryboardEvidence(null);
    setTaskStatus("已删除分镜行，需要重新确认当前镜头。");
    setExportNotice("导出已阻断：当前分镜行发生变更。");
  }

  function handleRowConfirmAction(index: number) {
    void handleConfirmCurrentRows();
    setTaskStatus(`已确认当前镜头，当前聚焦第 ${index + 1} 行。`);
  }

  void handleEditRow;
  void handleDuplicateRow;
  void handleConfirmRow;

  const contentBridge = [
    {
      label: "材料",
      value: acceptedBodyReady
        ? `已确认 · ${sourceFileName || "故事梗概"}`
        : sourceFileName
          ? `待确定使用 · ${sourceFileName}`
          : "待确定使用 · 故事梗概",
    },
    { label: "模式", value: `固定时长（${targetDuration} 秒）` },
    {
      label: "计划",
      value: storyboardTask ? `${storyboardTask.task_name} · ${storyboardTask.queue_status === "generated" ? "已生成" : "待生成"}` : "点击“确定使用”后同步到镜头拆解",
    },
    {
      label: "连续性",
      value: storyboardTask ? storyboardTask.continuity_status : "待确认后更新",
    },
  ];
  return (
    <div className="app-shell">
      <div className="workbench">
        <header className="topbar">
          <div className="topbar-brand topbar-brand--compact">
            <div className="brand-mark">
              <img src="/icon.svg" alt="Hope" />
            </div>
            <div className="topbar-title">
              <h1>Hope 动漫分镜脚本生成工作台</h1>
            </div>
          </div>
          <div className="topbar-center">
            <label className="topbar-model-group top-select">
              <span className="topbar-model-group__label">模型选择：</span>
              <div className="topbar-model-controls">
                <select
                  className="topbar-select topbar-select--provider"
                  value={provider.providerId}
                  onChange={(event) => setProvider((current) => applyProviderChoice(current, event.target.value))}
                >
                  {PROVIDER_DEFINITIONS.map((item) => (
                    <option key={item.providerId} value={item.providerId}>
                      {item.displayName}
                    </option>
                  ))}
                </select>
                <select
                  className="topbar-select topbar-select--model"
                  data-testid="top-model-select"
                  value={provider.model}
                  onChange={(event) =>
                    patchProvider({
                      model: event.target.value,
                      writingModel: event.target.value,
                      directorModel: event.target.value,
                    })
                  }
                >
                  {currentModelOptions.map((item) => (
                    <option key={item} value={item}>
                      {item}
                    </option>
                    ))}
                </select>
              </div>
              <small className="topbar-model-meta">{providerMetaLabel}</small>
              {providerReservedWarning ? <small className="topbar-model-warning">{providerReservedWarning}</small> : null}
            </label>
          </div>
          <div className="topbar-actions">
            <div className={`topbar-status topbar-status--${provider.connectionStatus}`}>
              {provider.connectionStatus === "ready"
                ? "API 已连接"
                : connectionStatus === "loading"
                  ? "API 测试中"
                  : provider.connectionStatus === "unsupported"
                    ? "API 未支持"
                    : provider.connectionStatus === "missing_config"
                      ? "API 待配置"
                      : "API 未就绪"}
            </div>
            <button className={activeModal === "docs" ? "toolbar-button toolbar-button--active" : "toolbar-button"} onClick={() => setActiveModal("docs")}>
              API文档
            </button>
            <button className={activeModal === "provider" ? "toolbar-button toolbar-button--active" : "toolbar-button"} onClick={() => setActiveModal("provider")}>
              API接口
            </button>
          </div>
        </header>

        <main className="workbench-main">
          <section className="panel script-panel">
            <div className="panel-title-row">
              <h2>剧本区</h2>
            </div>
            <div className="script-workspace">
              <div className="script-sidebar">
                <label className="scene-select">
                  <span>场景类型：</span>
                  <select data-testid="scene-type-select" value={sceneTypeId} onChange={(event) => handleSceneTypeChange(event.target.value)}>
                    {SCENE_TYPE_OPTIONS.map((item) => (
                      <option key={item.value} value={item.value}>
                        {item.label}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="duration-select duration-select--script">
                  <span>单镜头时长</span>
                  <select data-testid="duration-select" value={targetDuration} onChange={(event) => handleDurationChange(Number(event.target.value))}>
                    {KB_SNAPSHOT.allowedDurations.map((duration) => (
                      <option key={duration} value={duration}>
                        {duration} 秒
                      </option>
                    ))}
                  </select>
                </label>
              </div>
              <div className="script-material">
                <div className="text-control">
                  <div className={scriptSurfaceValue.trim() ? "synopsis-preview" : "synopsis-preview synopsis-preview--empty"}>
                    <span className="synopsis-preview__text">
                      {scriptSurfaceValue.trim() ? scriptSurfaceValue : "请输入或导入故事材料"}
                    </span>
                    <small className="source-input-status">
                      {scriptSurfaceLabel} · {scriptSurfaceMeta} · {scriptPreviewStatus}
                    </small>
                  </div>
                  <div className="text-control__actions">
                    <button className="text-control__button" onClick={() => setActiveModal("editor")} disabled={!scriptSurfaceValue.trim()}>
                      放大编辑
                    </button>
                    <button data-testid="accept-body-button" className="text-control__button" disabled={!bodyReady || acceptedBodyIsCurrent} onClick={() => void handleAcceptBody()}>
                      确定使用
                    </button>
                  </div>
                  <div className="qa-hidden-fields" aria-hidden="true">
                    <textarea data-testid="source-textarea" tabIndex={-1} value={sourceText} onChange={(event) => handleSourceTextChange(event.target.value)} />
                    <textarea data-testid="body-textarea" tabIndex={-1} value={bodyDraft} onChange={(event) => handleBodyDraftChange(event.target.value)} />
                    <div data-testid="task-status-text">{taskStatus}</div>
                  </div>
                </div>
              </div>
              <div className="script-actions">
                <button className="action-button action-button--dark" onClick={() => fileInputRef.current?.click()}>
                  导入文档
                </button>
                <button data-testid="expand-button" className="action-button action-button--dark" disabled={!providerReady || !kbReady} onClick={() => void handleGenerateBody("expand_story")}>
                  扩写故事
                </button>
                <button data-testid="rewrite-button" className="action-button action-button--dark" disabled={!providerReady || !kbReady} onClick={() => void handleGenerateBody("rewrite_script")}>
                  改写剧本
                </button>
              </div>
            </div>
          </section>

          <section className="content-bridge-row">
            {contentBridge.map((item) => (
              <div key={item.label} className="bridge-chip">
                <strong>{item.label}</strong>
                <span>{item.value}</span>
              </div>
            ))}
          </section>

          <section className="panel storyboard-panel">
            <div className="panel-title-row">
              <h2>镜头拆解</h2>
            </div>
            <div className="storyboard-toolbar task-row task-row--compact">
              <label className="task-index-select">
                <strong className="task-index-value task-index-value--live">{storyboardTask ? storyboardTask.task_index : "暂无"}</strong>
                <span>镜头序号</span>
                <strong className="task-index-value">{storyboardTask ? storyboardTask.task_index : "暂无"}</strong>
              </label>
              <label className="task-name-select">
                <select
                  className="task-toolbar-select"
                  value={storyboardTask?.task_id ?? ""}
                  onChange={(event) => event.target.value && handleImportStoryboardTask(event.target.value)}
                >
                  <option value="" disabled>
                    {storyboardTask ? storyboardTask.task_name : "选择镜头任务"}
                  </option>
                  {storyboardTask ? <option value={storyboardTask.task_id}>{storyboardTask.task_name}</option> : null}
                  {taskLibrary
                    .filter((item) => item.task.task_id !== storyboardTask?.task_id)
                    .map((item) => (
                      <option key={item.task.task_id} value={item.task.task_id}>
                        {item.task.task_name}
                      </option>
                    ))}
                </select>
                <button type="button" className="task-current-card task-current-card--pending" onClick={() => setTaskStatus("请选择或导入镜头任务。")} disabled>
                  <strong>第一组分镜生成</strong>
                  <span>{sceneTypeLabel}</span>
                  <em className="task-current-card__status task-current-card__status--pending">待生成</em>
                </button>
              </label>
              <button data-testid="create-task-button" className="action-button action-button--light" onClick={() => void handleCreateStoryboardTask()}>
                新建镜头任务
              </button>
              <button className="action-button action-button--light" disabled={!hasArchivedTasks} onClick={() => setActiveModal("taskImport")}>
                导入镜头任务
              </button>
              <div className="duration-pill">
                <strong>当前镜头时长</strong>
                <span>{currentShotDurationLabel}</span>
              </div>
              <button className="action-button action-button--light" disabled={rows.length === 0} onClick={handleClearStoryboard}>
                清空
              </button>
              <button data-testid="confirm-current-shot-button" className="action-button action-button--light" disabled={!rowsReady} onClick={() => void handleConfirmCurrentRows()}>
                保存修改
              </button>
              <div className="current-shot-status current-shot-status--pending">
                <strong>当前镜头</strong>
                <span>{currentShotStatusLabel}</span>
              </div>
              <button
                data-testid="generate-storyboard-button"
                className="action-button action-button--danger"
                disabled={!canTriggerGenerate}
                onClick={() => void handleGenerateStoryboard()}
              >
                {storyboardStatus === "loading" ? "开始生成中..." : "开始生成"}
              </button>
            </div>
            <div className="panel-title-row panel-title-row--subsection">
              <h2>当前镜头结果</h2>
            </div>
            <div className="table-wrap">
              <table className="result-table" data-testid="storyboard-table">
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
                  {rows.length === 0 ? (
                    <tr>
                      <td colSpan={10} className="empty-table-cell">
                        <div className="empty-state storyboard-empty-state">
                          <strong>当前还没有生成分镜</strong>
                          <span>请先导入镜头任务，再点击“开始生成”。</span>
                          <small>{taskStatus}</small>
                          {storyboardTask ? (
                            <button className="action-button action-button--light" disabled={!canTriggerGenerate} onClick={() => void handleGenerateStoryboard()}>
                              开始生成
                            </button>
                          ) : (
                            <button data-testid="empty-create-task-button" className="action-button action-button--light" onClick={() => void handleCreateStoryboardTask()}>
                              新建镜头任务
                            </button>
                          )}
                        </div>
                      </td>
                    </tr>
                  ) : (
                    rows.map((row, index) => (
                      <tr key={`${row.shot_index}-${index}`} data-row-index={index} className={editingRowIndex === index ? "result-row result-row--editing" : "result-row"}>
                        <td>{row.shot_index}</td>
                        <td title={displayCellText(row.person)}>
                          {editingRowIndex === index ? (
                            <input className="table-field is-editing" value={row.person} onChange={(event) => handleRowFieldChange("person", event.target.value)} />
                          ) : (
                            <div className="table-field is-display">{displayCellText(row.person)}</div>
                          )}
                        </td>
                        <td title={displayCellText(row.camera)}>
                          {editingRowIndex === index ? (
                            <input className="table-field is-editing" value={row.camera} onChange={(event) => handleRowFieldChange("camera", event.target.value)} />
                          ) : (
                            <div className="table-field is-display">{displayCellText(row.camera)}</div>
                          )}
                        </td>
                        <td title={displayCellText(row.shot_size)}>
                          {editingRowIndex === index ? (
                            <input className="table-field is-editing" value={row.shot_size} onChange={(event) => handleRowFieldChange("shot_size", event.target.value)} />
                          ) : (
                            <div className="table-field is-display">{displayCellText(row.shot_size)}</div>
                          )}
                        </td>
                        <td title={displayCellText(row.visual_description)}>
                          {editingRowIndex === index ? (
                            <textarea className="table-field table-field--multiline is-editing" value={row.visual_description} onChange={(event) => handleRowFieldChange("visual_description", event.target.value)} />
                          ) : (
                            <div className="table-field table-field--multiline is-display">{displayCellText(row.visual_description)}</div>
                          )}
                        </td>
                        <td title={displayCellText(row.character_action)}>
                          {editingRowIndex === index ? (
                            <textarea className="table-field table-field--multiline is-editing" value={row.character_action} onChange={(event) => handleRowFieldChange("character_action", event.target.value)} />
                          ) : (
                            <div className="table-field table-field--multiline is-display">{displayCellText(row.character_action)}</div>
                          )}
                        </td>
                        <td title={displayCellText(row.dialogue_or_narration)}>
                          {editingRowIndex === index ? (
                            <textarea className="table-field table-field--multiline is-editing" value={row.dialogue_or_narration} onChange={(event) => handleRowFieldChange("dialogue_or_narration", event.target.value)} />
                          ) : (
                            <div className="table-field table-field--multiline is-display">{displayCellText(row.dialogue_or_narration)}</div>
                          )}
                        </td>
                        <td title={displayCellText(row.prompt_text)}>
                          {editingRowIndex === index ? (
                            <textarea className="table-field table-field--multiline table-field--prompt is-editing" value={row.prompt_text} onChange={(event) => handleRowFieldChange("prompt_text", event.target.value)} />
                          ) : (
                            <div className="table-field table-field--multiline table-field--prompt is-display">{displayCellText(row.prompt_text)}</div>
                          )}
                        </td>
                        <td title={displayCellText(row.duration_seconds)}>
                          {editingRowIndex === index ? (
                            <input className="table-field is-editing" type="number" min={1} value={row.duration_seconds} onChange={(event) => handleRowFieldChange("duration_seconds", event.target.value)} />
                          ) : (
                            <div className="table-field is-display">{displayCellText(row.duration_seconds)}</div>
                          )}
                        </td>
                        <td className="table-action-cell">
                          <div className="table-actions">
                            <button className={editingRowIndex === index ? "table-action table-action--save" : "table-action"} onClick={() => (editingRowIndex === index ? handleSaveEditedRow(index) : handleRowEditAction(index))}>
                              {editingRowIndex === index ? "保存" : "修改"}
                            </button>
                            <button className="table-action" onClick={() => (editingRowIndex === index ? handleCancelEditedRow(index) : handleRowDuplicateAction(index))}>
                              {editingRowIndex === index ? "取消" : "复制"}
                            </button>
                            <button className="table-action table-action--danger" disabled={editingRowIndex === index} onClick={() => handleRowDeleteAction(index)}>
                              删除
                            </button>
                            <button className="table-action table-action--confirm" disabled={editingRowIndex === index} onClick={() => handleRowConfirmAction(index)}>
                              确定使用
                              <span>当前镜头</span>
                            </button>
                          </div>
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </section>
        </main>

        <footer className="workbench-footer">
          <div className="footer-bank__main">
            <strong>已定稿分镜区</strong>
            <span>已确认 {confirmedRows.length}/{rows.length} 个镜头</span>
            <span>总时长 {finalizedDuration} 秒</span>
            <span className="footer-bank__count footer-bank__count--live">已确认 {confirmedRows.length}/{rows.length} 个镜头</span>
            <button className="ghost-button footer-view-button" onClick={() => setActiveModal("finalized")}>
              查看全部
            </button>
          </div>
          <div className="footer-actions">
            <button data-testid="export-storyboard" className="action-button action-button--light footer-export-button" disabled={!confirmedRowsReady || !acceptedBodyReady} onClick={() => performExport("storyboard-excel")}>
              导出分镜词
            </button>
            <button data-testid="export-script" className="action-button action-button--dark footer-export-button" disabled={!confirmedRowsReady || !acceptedBodyReady} onClick={() => performExport("script-excel")}>
              导出完整剧本
            </button>
          </div>
          <div className="export-notice">{exportNotice}</div>
        </footer>

        <input ref={fileInputRef} hidden type="file" accept=".txt,.md,.json" onChange={(event) => void handleImportSource(event)} />
        <div id="hope-qa-trace" hidden />

        {activeModal === "docs" ? (
          <Modal title="API文档" onClose={() => setActiveModal(null)}>
            <div className="modal-stack">
              {DOC_SECTIONS.map((section) => (
                <section key={section.title} className="doc-section">
                  <h3>{section.title}</h3>
                  {section.body.map((item) => (
                    <p key={item}>{item}</p>
                  ))}
                </section>
              ))}
            </div>
          </Modal>
        ) : null}

        {activeModal === "provider" ? (
          <Modal title="API接口" onClose={() => setActiveModal(null)}>
            <div className="config-grid">
              <label>
                <span>Provider</span>
                <select data-testid="provider-select" value={provider.providerId} onChange={(event) => setProvider((current) => applyProviderChoice(current, event.target.value))}>
                  {PROVIDER_DEFINITIONS.map((item) => (
                    <option key={item.providerId} value={item.providerId}>
                      {item.displayName}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                <span>Base URL</span>
                <input data-testid="base-url-input" value={provider.baseUrl} onChange={(event) => patchProvider({ baseUrl: event.target.value })} placeholder="https://example.com/v1" />
              </label>
              <label>
                <span>Endpoint</span>
                <input data-testid="endpoint-input" value={provider.endpoint} onChange={(event) => patchProvider({ endpoint: event.target.value })} placeholder="/chat/completions" />
              </label>
              <label>
                <span>Model</span>
                <input data-testid="model-input" value={provider.model} onChange={(event) => patchProvider({ model: event.target.value })} />
              </label>
              <label>
                <span>writing_model</span>
                <select data-testid="writing-model-select" value={provider.writingModel} onChange={(event) => patchProvider({ writingModel: event.target.value })}>
                  {currentModelOptions.map((item) => (
                    <option key={`writing-${item}`} value={item}>
                      {item}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                <span>director_model</span>
                <select data-testid="director-model-select" value={provider.directorModel} onChange={(event) => patchProvider({ directorModel: event.target.value })}>
                  {currentModelOptions.map((item) => (
                    <option key={`director-${item}`} value={item}>
                      {item}
                    </option>
                  ))}
                </select>
              </label>
              <label className="config-grid__wide">
                <span>API Key</span>
                <input data-testid="api-key-input" type="password" value={provider.apiKey} onChange={(event) => patchProvider({ apiKey: event.target.value })} placeholder="默认只保存在当前会话" />
              </label>
              <label className="persist-toggle">
                <input type="checkbox" checked={provider.persistApiKey} onChange={(event) => patchProvider({ persistApiKey: event.target.checked })} />
                <span>允许写入本机浏览器存储（默认关闭，建议 session-only）</span>
              </label>
              <label>
                <span>validator_model</span>
                <input value={provider.validatorModel} onChange={(event) => patchProvider({ validatorModel: event.target.value })} placeholder="留空表示本地 validator" />
              </label>
            </div>
            <div className="connection-box">
              <div className="connection-box__meta">
                <strong>{providerDefinition.statusHint}</strong>
                <span>transport: {providerDefinition.transportCapability}</span>
                <span>host: {deriveProviderModelRecord(provider).base_url_host || "未填写"}</span>
                <span>last: {provider.lastConnectionTest ?? "未测试"}</span>
              </div>
              <div className="connection-box__actions">
                <button className="ghost-button" onClick={handleSaveProviderConfig}>
                  保存 API 配置
                </button>
                <button data-testid="test-connection-button" className="primary-button" disabled={provider.connectionStatus === "unsupported" || connectionStatus === "loading"} onClick={handleTestConnection}>
                  {connectionStatus === "loading" ? "测试中..." : "测试连接"}
                </button>
              </div>
            </div>
            <div className={`inline-state inline-state--${provider.connectionStatus}`}>{provider.connectionMessage}</div>
          </Modal>
        ) : null}

        {activeModal === "kb" ? (
          <Modal title="KB摘要" onClose={() => setActiveModal(null)}>
            <div className="summary-list">
              <div><strong>scene_type</strong><span>{sceneTypeId} / {sceneTypeLabel}</span></div>
              <div><strong>selected_sample_ids</strong><span>{kbSummary?.selected_sample_ids.join(" / ") ?? "加载中"}</span></div>
              <div><strong>selected_kb_rules</strong><span>{kbSummary?.selected_kb_rules.join(" / ") ?? "加载中"}</span></div>
              <div><strong>scene profile</strong><span>{kbSummary?.scene_profile ?? "加载中"}</span></div>
              <div><strong>kb_context_summary</strong><span>{kbSummary?.kb_context_summary ?? "加载中"}</span></div>
              <div><strong>negative constraints</strong><span>{kbSummary?.negative_constraints.join("；") ?? "加载中"}</span></div>
            </div>
          </Modal>
        ) : null}

        {activeModal === "evidence" ? (
          <Modal title="Evidence" onClose={() => setActiveModal(null)}>
            <div className="summary-list">
              <div><strong>accepted_body_hash</strong><span>{bodyEvidence?.accepted_body_hash.slice(0, 12) ?? "未生成"}</span></div>
              <div><strong>storyboard_task_hash</strong><span>{storyboardTask?.task_hash.slice(0, 12) ?? "未创建"}</span></div>
              <div><strong>rows_hash</strong><span>{storyboardEvidence?.rows_hash.slice(0, 12) ?? "未生成"}</span></div>
              <div><strong>confirmed_row_hash</strong><span>{storyboardEvidence?.confirmed_row_hash?.slice(0, 12) ?? "未确认"}</span></div>
              <div><strong>source_lineage</strong><span>{storyboardEvidence?.source_lineage ?? bodyEvidence?.source_lineage ?? "无"}</span></div>
              <div><strong>warning taxonomy</strong><span>{(storyboardEvidence?.warning_taxonomy ?? bodyEvidence?.warning_taxonomy ?? []).join(" / ") || "无"}</span></div>
              <div><strong>export audit</strong><span>{exportAudits.length ? exportAudits.map((item) => `${item.file_name}:${item.usable ? "usable" : "blocked"}`).join(" / ") : "暂无"}</span></div>
            </div>
          </Modal>
        ) : null}

        {activeModal === "finalized" ? (
          <Modal title="已定稿分镜区" onClose={() => setActiveModal(null)}>
            <div className="summary-list">
              <div><strong>已确认镜头</strong><span>{confirmedRows.length}/{rows.length}</span></div>
              <div><strong>总时长</strong><span>{finalizedDuration} 秒</span></div>
              <div><strong>accepted body</strong><span>{acceptedBodyReady ? "已确认" : "未确认"}</span></div>
            </div>
            <div className="table-wrap finalized-table-wrap">
              <table className="result-table">
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
                  </tr>
                </thead>
                <tbody>
                  {confirmedRows.length === 0 ? (
                    <tr>
                      <td colSpan={9} className="empty-table-cell">
                        <div className="empty-state">
                          <strong>当前还没有已确认分镜</strong>
                          <span>请先生成当前镜头结果，再点击“确定使用当前镜头”恢复导出链路。</span>
                        </div>
                      </td>
                    </tr>
                  ) : (
                    confirmedRows.map((row, index) => (
                      <tr key={`finalized-${row.shot_index}-${index}`}>
                        <td>{row.shot_index}</td>
                        <td>{row.person}</td>
                        <td>{row.camera}</td>
                        <td>{row.shot_size}</td>
                        <td>{row.visual_description}</td>
                        <td>{row.character_action}</td>
                        <td>{row.dialogue_or_narration}</td>
                        <td>{row.prompt_text}</td>
                        <td>{row.duration_seconds}</td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </Modal>
        ) : null}

        {activeModal === "taskCreate" && taskCreationDraft ? (
          <Modal title="新建镜头任务" panelClassName="modal-panel--task-create" onClose={() => setActiveModal(null)}>
            <div className="task-create-modal">
              <p className="task-create-modal__intro">
                系统候选只负责提供草稿；镜头任务队列才会进入下方生成。右侧名称、时长、片段始终来自同一个草稿。
              </p>
              <div className="task-create-modal__layout">
                <aside className="task-create-modal__sidebar">
                  <div className="task-create-stats">
                    <span>候选 {taskCreationDraft.candidates.length} 个</span>
                    <span>已建 {taskLibrary.length + (storyboardTask ? 1 : 0)} 个</span>
                    <span>已生成 {taskLibrary.filter((item) => item.task.queue_status === "generated").length + (storyboardTask?.queue_status === "generated" ? 1 : 0)} 个</span>
                  </div>
                  <div className="task-create-section-title">系统建议（未应用）</div>
                  <div className="task-candidate-list">
                    {taskCreationDraft.candidates.map((candidate) => {
                      const statusLabel =
                        candidate.id === taskCreationDraft.selectedTaskId
                          ? "当前"
                          : candidate.queueStatus === "queued"
                            ? "已加入队列"
                            : "未加入队列";
                      return (
                        <button
                          key={candidate.id}
                          type="button"
                          className={`task-candidate-card${candidate.id === taskCreationDraft.selectedTaskId ? " task-candidate-card--active" : ""}`}
                          onClick={() => handleTaskDraftCandidateSelect(candidate.id)}
                        >
                          <div className="task-candidate-card__header">
                            <strong>{candidate.title}</strong>
                            <span>{statusLabel}</span>
                          </div>
                          <p>{candidate.summary}</p>
                          <small>预计 {candidate.durationSeconds} 秒</small>
                        </button>
                      );
                    })}
                  </div>
                </aside>
                <section className="task-create-modal__main">
                  <div className="task-queue-bar">
                    <div className="task-queue-bar__title">镜头任务队列（将用于生成）</div>
                    <div className="task-queue-bar__controls">
                      <select value={taskCreationDraft.selectedTaskId} onChange={(event) => handleTaskDraftCandidateSelect(event.target.value)}>
                        {taskCreationDraft.candidates.map((candidate) => (
                          <option key={candidate.id} value={candidate.id}>
                            {candidate.title}
                          </option>
                        ))}
                      </select>
                      <span className="task-queue-status">
                        {taskCreationDraft.selectedTaskStatus === "generated"
                          ? "已生成"
                          : taskCreationDraft.selectedTaskStatus === "queued"
                            ? "已加入队列"
                            : "待生成"}
                      </span>
                    </div>
                  </div>
                  <div className="task-source-panel">
                    <strong>当前任务来源</strong>
                    <span>{taskCreationDraft.selectedTaskSourceLabel}</span>
                    <span>预计 {taskCreationDraft.targetDurationSeconds} 秒</span>
                    <span>人物：{taskCreationDraft.peopleSummary}</span>
                    <span>连续性：{taskCreationDraft.continuityStatus}</span>
                    <span>{taskCreationDraft.selectedTaskStatus === "queued" || taskCreationDraft.selectedTaskStatus === "generated" ? "当前已加入任务队列" : "当前尚未加入任务队列"}</span>
                  </div>
                  <label className="task-create-field">
                    <span>镜头任务名称</span>
                    <input
                      data-testid="task-name-input"
                      value={taskCreationDraft.taskName}
                      onChange={(event) => setTaskCreationDraft((current) => (current ? { ...current, taskName: event.target.value } : current))}
                    />
                  </label>
                  <label className="task-create-field">
                    <span>预计镜头时长</span>
                    <select value={taskCreationDraft.targetDurationSeconds} onChange={(event) => handleTaskDraftDurationChange(Number(event.target.value))}>
                      {KB_SNAPSHOT.allowedDurations.map((duration) => (
                        <option key={duration} value={duration}>
                          {duration} 秒
                        </option>
                      ))}
                    </select>
                    <small>当前按系统候选原文使用；调整时长会先更新当前草稿，不会静默改任务队列。</small>
                  </label>
                  <label className="task-create-field">
                    <span>任务队列处理</span>
                    <select
                      value={taskCreationDraft.preservePreviousTask ? "preserve" : "replace"}
                      onChange={(event) =>
                        setTaskCreationDraft((current) =>
                          current
                            ? { ...current, preservePreviousTask: event.target.value === "preserve" }
                            : current,
                        )
                      }
                    >
                      <option value="preserve">保留旧任务并加入当前任务</option>
                      <option value="replace">清空旧任务，仅保留当前任务</option>
                    </select>
                    <small>只影响任务队列，不会清空上方正文；切换 scene_type、duration 或 accepted body 仍会让旧任务 stale。</small>
                  </label>
                  <label className="task-create-field task-create-field--editor">
                    <span>本次镜头任务使用的脚本片段</span>
                    <small>当前编辑：{sourceKindLabel(taskCreationDraft.selectedTaskSourceKind)} / {taskCreationDraft.selectedTaskSourceLabel}</small>
                    <textarea value={taskCreationDraft.scriptFragment} onChange={(event) => handleTaskDraftFragmentChange(event.target.value)} />
                  </label>
                  <div className="task-create-actions">
                    <button className="ghost-button" onClick={handleTaskDraftRetiming} disabled={!acceptedBodyReady}>
                      按时长更新
                    </button>
                    <button className="ghost-button" onClick={handleTaskDraftRestoreOriginal} disabled={!acceptedBodyReady}>
                      恢复原文
                    </button>
                    <button className="ghost-button" onClick={() => void handleConfirmStoryboardTaskCreate({ keepOpen: true })} disabled={!acceptedBodyReady || !taskCreationDraft.taskName.trim()}>
                      加入并继续
                    </button>
                    <button data-testid="task-create-confirm-button" className="primary-button" disabled={!acceptedBodyReady || !taskCreationDraft.taskName.trim()} onClick={() => void handleConfirmStoryboardTaskCreate()}>
                      加入任务队列
                    </button>
                  </div>
                </section>
              </div>
            </div>
          </Modal>
        ) : null}

        {activeModal === "taskImport" ? (
          <Modal title="导入镜头任务" onClose={() => setActiveModal(null)}>
            <div className="summary-list">
              {taskLibrary.length === 0 ? (
                <div><strong>当前没有可导入任务</strong><span>请先创建至少一个 storyboard task。</span></div>
              ) : (
                taskLibrary.map((item) => (
                  <div key={item.task.task_id} className="task-import-item">
                    <div className="task-import-item__meta">
                      <strong>{item.task.task_name}</strong>
                      <span>{item.task.scene_type_label} / {item.task.target_duration_seconds} 秒 / {item.task.sync_status}</span>
                      <span>来源链路与任务队列状态将一并恢复到当前工作台。</span>
                    </div>
                    <button className="ghost-button" onClick={() => handleImportStoryboardTask(item.task.task_id)}>
                      恢复此任务
                    </button>
                  </div>
                ))
              )}
            </div>
          </Modal>
        ) : null}

        {activeModal === "editor" ? (
          <Modal title="放大编辑" onClose={() => setActiveModal(null)}>
            <label className="full-editor">
              <span>{scriptSurfaceIsDraft ? "正文草稿放大编辑" : "正文输入放大编辑"}</span>
              <textarea value={scriptSurfaceValue} onChange={(event) => handleScriptSurfaceChange(event.target.value)} />
            </label>
          </Modal>
        ) : null}
      </div>
    </div>
  );
}

