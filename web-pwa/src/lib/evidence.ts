import { sha256Hex, stableStringify } from "./hash";
import { getKbSnapshotHash } from "./kb";
import { getSceneTypeLabel, isCanonicalSceneType, SCENE_TYPE_CANONICAL_LIST } from "./sceneTypes";
import type {
  GenerationEvidence,
  ProviderFormState,
  SanitizedKbSummary,
  StoryboardRow,
  StoryboardTask,
  ValidationResult,
} from "./types";

const PERSON_FORBIDDEN_TOKENS = ["环境", "镜头", "构图", "画面", "场景", "空间", "UI", "/"];
const PERSON_LOCATION_HINTS = ["城", "室", "段", "口", "桥", "巷", "台", "楼", "营", "山", "河", "海", "界面"];
const PERSON_ACTION_HINTS = ["奔跑", "冲刺", "追击", "撤离", "断后", "挥", "击", "移动", "突进"];
const PERSON_SCENE_HINTS = ["战报", "军阵", "奇观", "追逐", "表演", "对话", "战斗"];
const GENERIC_ROLE_PLACEHOLDERS = ["主角", "角色A", "角色B", "人物A", "人物B", "某人", "众人", "路人"];

function baseUrlHost(baseUrl: string): string {
  try {
    return new URL(baseUrl).host;
  } catch {
    return "";
  }
}

function hasTraceLikeText(value: string): boolean {
  return /(source_register|overlay[_ -]?json|prompt_body|validator|raw[_ -]?kb|raw[_ -]?sample)/i.test(value);
}

function personFieldFlags(rows: StoryboardRow[]) {
  const people = rows.map((row) => row.person.trim()).filter(Boolean);
  return {
    personFieldValid: people.length > 0 && people.every((value) => !PERSON_FORBIDDEN_TOKENS.includes(value)),
    locationNotInPerson: people.every((value) => !PERSON_LOCATION_HINTS.some((hint) => value.includes(hint))),
    actionFragmentNotInPerson: people.every((value) => !PERSON_ACTION_HINTS.some((hint) => value.includes(hint))),
    sceneTermNotInPerson: people.every((value) => !PERSON_SCENE_HINTS.some((hint) => value.includes(hint))),
    genericRolePlaceholderAbsent: people.every((value) => !GENERIC_ROLE_PLACEHOLDERS.includes(value)),
  };
}

export async function createStoryboardTask(params: {
  acceptedBodyHash: string;
  sceneTypeId: string;
  targetDurationSeconds: number;
  kbSummary: SanitizedKbSummary;
  taskName: string;
  taskIndex: number;
  taskSourceKind: "system_candidate" | "full_narrative" | "confirmed_body" | "custom_fragment";
  taskSourceLabel: string;
  peopleSummary: string;
  scriptFragment: string;
  originalFragment: string;
  currentBodySource: "accepted_body";
  generationGroup: string;
  estimatedShotCount: number;
  durationAllocationStrategy: string;
  continuityStatus: string;
  createdFromConfirmedBody: boolean;
  preservePreviousTask: boolean;
}): Promise<StoryboardTask> {
  const createdAt = new Date().toISOString();
  const baseTask = {
    task_name: params.taskName,
    task_index: params.taskIndex,
    task_source_kind: params.taskSourceKind,
    task_source_label: params.taskSourceLabel,
    people_summary: params.peopleSummary,
    script_fragment: params.scriptFragment,
    original_fragment: params.originalFragment,
    queue_status: "pending" as const,
    current_body_source: params.currentBodySource,
    accepted_body_hash: params.acceptedBodyHash,
    scene_type_id: params.sceneTypeId,
    scene_type_label: getSceneTypeLabel(params.sceneTypeId),
    target_duration_seconds: params.targetDurationSeconds,
    kb_snapshot_hash: params.kbSummary.kb_snapshot_hash,
    selected_sample_ids: [...params.kbSummary.selected_sample_ids],
    selected_kb_rules: [...params.kbSummary.selected_kb_rules],
    writing_group_rule_pack_ids: [...params.kbSummary.writing_group_rule_pack_ids],
    director_group_rule_pack_ids: [...params.kbSummary.director_group_rule_pack_ids],
    output_intent: "storyboard_rows" as const,
    source_lineage: "accepted_body" as const,
    generation_group: params.generationGroup,
    estimated_shot_count: params.estimatedShotCount,
    duration_allocation_strategy: params.durationAllocationStrategy,
    continuity_status: params.continuityStatus,
    created_from_confirmed_body: params.createdFromConfirmedBody,
    preserve_previous_task: params.preservePreviousTask,
  };
  const taskHash = await sha256Hex(stableStringify(baseTask));
  const taskId = await sha256Hex(`${createdAt}|${taskHash}`);

  return {
    task_id: taskId,
    ...baseTask,
    task_hash: taskHash,
    storyboard_task_hash: taskHash,
    rows_hash: "",
    confirmed_row_hash: "",
    stale_state: false,
    sync_status: "fresh",
    stale_reasons: [],
    created_at: createdAt,
  };
}

export async function buildEvidence(params: {
  sourceText: string;
  acceptedBody: string;
  rows: StoryboardRow[];
  kbSummary: SanitizedKbSummary;
  provider: ProviderFormState;
  modelId: string;
  sceneTypeId: string;
  targetDurationSeconds: number;
  validatorResult: ValidationResult;
  outputIntent: "narrative_body" | "storyboard_rows";
  task: StoryboardTask | null;
  fallbackUsed?: boolean;
  localCandidate?: boolean;
}): Promise<GenerationEvidence> {
  const storyboardTask = params.task
    ? stableStringify(params.task)
    : [
        `scene_type_id=${params.sceneTypeId}`,
        `scene_type_label=${getSceneTypeLabel(params.sceneTypeId)}`,
        `target_duration_seconds=${params.targetDurationSeconds}`,
      ].join(" | ");

  const [sourceHash, acceptedBodyHash, rowsHash, kbSnapshotHash, storyboardTaskHash] = await Promise.all([
    sha256Hex(params.sourceText),
    sha256Hex(params.acceptedBody),
    sha256Hex(stableStringify(params.rows)),
    getKbSnapshotHash(),
    sha256Hex(storyboardTask),
  ]);

  const rowsMatch =
    params.rows.length > 0 &&
    params.rows.every(
      (row) =>
        row.person.trim() &&
        row.visual_description.trim() &&
        row.prompt_text.trim() &&
        row.duration_seconds > 0,
    );
  const promptTextBoundaryPassed =
    params.rows.length > 0 &&
    params.rows.every(
      (row) =>
        row.prompt_text.includes("事实源：") &&
        row.prompt_text.includes("主体：") &&
        row.prompt_text.includes("画面：") &&
        row.prompt_text.includes("负面约束："),
    );
  const promptTextNotSummaryOnly =
    params.rows.length > 0 &&
    params.rows.every((row) => row.prompt_text.length > 48 && row.prompt_text.includes("运镜：") && row.prompt_text.includes("时长节奏："));
  const visualDescriptionVisibleFramePassed = !params.validatorResult.issues.some((item) => item.code.includes("visual"));
  const visualDescriptionNoTrace = params.rows.every((row) => !hasTraceLikeText(row.visual_description));
  const promptCompiledAfterFinalRow =
    params.rows.length > 0 && params.rows.every((row) => row.prompt_text.includes("事实源："));
  const warningTaxonomy = params.validatorResult.issues.map((item) => item.code);
  const personFlags = personFieldFlags(params.rows);
  const staleTaskDetected = params.task?.sync_status === "stale";
  const staleRowsDetected = params.rows.some((row) => row.status === "stale");
  const validatorPseudoSuccessDetected =
    params.validatorResult.passed && params.rows.length === 0 && params.outputIntent === "storyboard_rows";
  const taskSyncStatus = params.task?.sync_status ?? "fresh";
  const storyFactFrame =
    params.outputIntent === "narrative_body"
      ? "source_text_candidate -> writing_group"
      : "accepted_body_locked -> storyboard_task -> director_group";

  return {
    artifact_identity: await sha256Hex(`${params.outputIntent}|${acceptedBodyHash}|${storyboardTaskHash}|${rowsHash}`),
    source_lineage: params.outputIntent === "narrative_body" ? "user_source_text" : "accepted_body",
    source_lineage_evidence:
      params.outputIntent === "narrative_body"
        ? "user_source_text -> writing_group -> accepted_body_candidate"
        : `accepted_body(${acceptedBodyHash.slice(0, 12)}) -> storyboard_task(${storyboardTaskHash.slice(0, 12)}) -> director_group`,
    source_hash: sourceHash,
    output_intent: params.outputIntent,
    story_fact_frame: storyFactFrame,
    accepted_body_hash: acceptedBodyHash,
    storyboard_task: storyboardTask,
    storyboard_task_hash: storyboardTaskHash,
    kb_snapshot_hash: kbSnapshotHash,
    selected_sample_ids: [...params.kbSummary.selected_sample_ids],
    selected_kb_rules: [...params.kbSummary.selected_kb_rules],
    writing_group_rule_pack_ids: [...params.kbSummary.writing_group_rule_pack_ids],
    director_group_rule_pack_ids: [...params.kbSummary.director_group_rule_pack_ids],
    kb_context_summary: params.kbSummary.kb_context_summary,
    applied_to: [...params.kbSummary.applied_to],
    influence_axes: [...params.kbSummary.influence_axes],
    kb_oracle_affects_structure: true,
    raw_kb_rows_included: params.kbSummary.raw_kb_rows_included,
    raw_sample_text_absent: params.kbSummary.raw_sample_text_absent,
    source_register_absent: params.kbSummary.source_register_absent,
    overlay_json_absent: params.kbSummary.overlay_json_absent,
    prompt_body_absent: params.kbSummary.prompt_body_absent,
    scene_type_id: params.sceneTypeId,
    scene_type_label: getSceneTypeLabel(params.sceneTypeId),
    scene_type_canonical_coverage: SCENE_TYPE_CANONICAL_LIST.length,
    scene_type_valid: isCanonicalSceneType(params.sceneTypeId),
    target_duration_seconds: params.targetDurationSeconds,
    rows_hash: rowsHash,
    confirmed_row_hash: rowsHash,
    export_source_hash: params.outputIntent === "narrative_body" ? acceptedBodyHash : rowsHash,
    prompt_compilation_version: "hope-web-pwa-v4",
    warning_taxonomy: warningTaxonomy,
    warning_taxonomy_classified: warningTaxonomy.every(Boolean),
    hardfail_warning_absent: params.validatorResult.passed,
    stale_state: staleTaskDetected || staleRowsDetected || !params.validatorResult.passed,
    stale_task_detected: staleTaskDetected,
    stale_rows_detected: staleRowsDetected,
    task_sync_status: taskSyncStatus,
    rows_count: params.rows.length,
    rows_match: rowsMatch,
    prompt_text_present: params.rows.every((row) => row.prompt_text.trim().length > 0),
    prompt_text_boundary_passed: promptTextBoundaryPassed,
    prompt_text_not_summary_only: promptTextNotSummaryOnly,
    visual_description_visible_frame_passed: visualDescriptionVisibleFramePassed,
    visual_description_no_trace: visualDescriptionNoTrace,
    prompt_compiled_after_final_row: promptCompiledAfterFinalRow,
    person_field_valid: personFlags.personFieldValid,
    location_not_in_person: personFlags.locationNotInPerson,
    action_fragment_not_in_person: personFlags.actionFragmentNotInPerson,
    scene_term_not_in_person: personFlags.sceneTermNotInPerson,
    generic_role_placeholder_absent: personFlags.genericRolePlaceholderAbsent,
    validator_pseudo_success_detected: validatorPseudoSuccessDetected,
    fallback_used: Boolean(params.fallbackUsed),
    provider_failover_used: false,
    local_candidate: Boolean(params.localCandidate),
    row_prompt_visual_gate_evidence: `rows=${params.rows.length}; prompt=${params.rows.every((row) => row.prompt_text.trim())}; visual=${params.rows.every((row) => row.visual_description.trim())}`,
    field_aware_entity_gate_evidence: `person=${personFlags.personFieldValid}; location=${personFlags.locationNotInPerson}; action=${personFlags.actionFragmentNotInPerson}; scene=${personFlags.sceneTermNotInPerson}; placeholder=${personFlags.genericRolePlaceholderAbsent}`,
    manual_edit_confirmed_row_evidence:
      params.rows.some((row) => row.is_user_edited)
        ? `manual_edit_detected=true; confirm_required=${params.outputIntent === "storyboard_rows"}`
        : "manual_edit_detected=false",
    blocked_warning_ui_evidence: warningTaxonomy.length ? warningTaxonomy.join(" / ") : "none",
    human_review_spotcheck_evidence:
      params.outputIntent === "storyboard_rows"
        ? `review rows=${params.rows.length}; samples=${params.rows.slice(0, 2).map((row) => row.visual_description.slice(0, 24)).join(" | ")}`
        : `review narrative=${params.acceptedBody.slice(0, 48)}`,
    export_evidence: [
      params.outputIntent === "storyboard_rows" ? "export_storyboard_uses_confirmed_rows=pending_confirm" : "export_storyboard_uses_confirmed_rows=false",
      params.outputIntent === "storyboard_rows" ? "export_script_uses_confirmed_body_and_rows=pending_confirm" : "export_script_uses_confirmed_body_and_rows=false",
    ],
    validator_result: params.validatorResult,
    provider_id: params.provider.providerId,
    model_id: params.modelId,
    base_url_host: baseUrlHost(params.provider.baseUrl),
    cors_check_result: params.provider.corsCheckResult,
    generated_at: new Date().toISOString(),
  };
}
