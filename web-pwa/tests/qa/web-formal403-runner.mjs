import matrix from "./web-403-matrix.json" with { type: "json" };
import golden from "./web-403-golden.json" with { type: "json" };
import {
  APP_URL,
  FULL16_CASES,
  prepareArtifactRoot,
  parseCli,
  runWebWorkflowCase,
  SCENE_TYPE_CANONICAL_LIST,
  sceneTypeLabelForValue,
  summarizeResults,
  TARGETED_CASES,
  writeCaseArtifact,
  writeSummaryArtifact,
} from "../../scripts/web-runner-lib.mjs";

function assertCanonicalSceneCoverage() {
  if (SCENE_TYPE_CANONICAL_LIST.length !== 21) {
    throw new Error(`canonical scene coverage mismatch: expected 21, got ${SCENE_TYPE_CANONICAL_LIST.length}`);
  }
}

function assertDurationCoverage() {
  const durations = matrix.fixed.durations.join(",");
  if (durations !== "5,10,15,30,45,60") {
    throw new Error(`duration canonical mismatch: ${durations}`);
  }
}

function assertGoldenCoverage() {
  const goldenSceneIds = Object.keys(golden.scene_types);
  if (goldenSceneIds.length !== 21) {
    throw new Error(`golden scene coverage mismatch: expected 21, got ${goldenSceneIds.length}`);
  }
  for (const sceneType of SCENE_TYPE_CANONICAL_LIST) {
    if (!golden.scene_types[sceneType]) {
      throw new Error(`golden expectation missing for scene type: ${sceneType}`);
    }
  }
}

function expandEntryCases() {
  return matrix.entry.map((entry) => ({
    caseId: entry.case_id,
    kind: "entry",
    description: entry.description,
  }));
}

function expandFixedCases() {
  return SCENE_TYPE_CANONICAL_LIST.flatMap((sceneType) =>
    matrix.fixed.durations.flatMap((duration) =>
      matrix.fixed.modes.map((mode) => ({
        caseId: `fixed-${sceneType}-${duration}-${mode}`,
        kind: "fixed",
        sceneType,
        duration,
        mode,
        sourceText: `${sceneTypeLabelForValue(sceneType)}固定矩阵样例：角色必须在有限时长内完成冲突推进，同时保留环境、关系与动作线，保证输出能落到当前可见镜头。`,
      })),
    ),
  );
}

function expandLongCases() {
  return SCENE_TYPE_CANONICAL_LIST.map((sceneType) => ({
    caseId: `long-${sceneType}`,
    kind: "long",
    sceneType,
    duration: matrix.long.duration,
    mode: matrix.long.mode,
    sourceText: `${sceneTypeLabelForValue(sceneType)}长文本样例：在更完整的叙事里持续推进人物关系、空间变化和冲突升级，确保结果仍能拆成可见分镜，并完成 prompt_text 与 export 的闭环。`,
  }));
}

function buildGoldenExpectation(caseDef) {
  if (caseDef.kind === "entry") {
    return {
      entry_checks: golden.entry[caseDef.caseId] ?? [],
      golden_expectation_matched: true,
    };
  }
  return {
    scene_type_id: caseDef.sceneType,
    scene_type_label: golden.scene_types[caseDef.sceneType]?.label ?? caseDef.sceneType,
    duration: caseDef.duration,
    mode: caseDef.mode,
    duration_capacity: golden.durations[String(caseDef.duration)]?.expected_capacity ?? "unknown",
    required_chain: golden.modes[caseDef.mode]?.required_chain ?? [],
    forbidden_placeholders: golden.scene_types[caseDef.sceneType]?.forbidden_placeholders ?? [],
  };
}

function entryChecks(caseDef, result) {
  const ui = result.ui_evidence ?? {};
  const checks = {
    page_loaded: Boolean(ui.page_loaded),
    scene_selector_ready: Boolean(ui.scene_selector_ready),
    duration_selector_ready: Boolean(ui.duration_selector_ready),
    api_modal_ready: Boolean(ui.provider_modal_ready),
    export_controls_visible: Boolean(ui.export_storyboard_visible && ui.export_script_visible),
    export_default_blocked: Boolean(ui.export_storyboard_disabled && ui.export_script_disabled),
  };
  return {
    ...checks,
    golden_expectation_matched: Object.values(checks).every(Boolean),
  };
}

function workflowChecks(caseDef, result) {
  const trace = result.trace ?? {};
  const sceneExpectation = golden.scene_types[caseDef.sceneType] ?? { label: caseDef.sceneType, forbidden_placeholders: [] };
  const joinedPreview = [result.rows_preview ?? [], result.prompt_text_preview ?? [], result.visual_description_preview ?? []]
    .flat()
    .join(" | ");
  const exportAudit = Array.isArray(result.downloads) ? result.downloads : [];
  return {
    live_ready: trace.connection_status === "ready",
    no_http_403: Boolean(result.no_http_403),
    fallback_used: trace.fallback_used === false,
    local_candidate: trace.local_candidate === false,
    provider_failover_used: trace.provider_failover_used === false,
    source_lineage_consistent: trace.source_lineage === (caseDef.mode === "storyboard_from_body" ? "accepted_body" : trace.source_lineage),
    accepted_body_hash_present: Boolean(trace.accepted_body_hash),
    storyboard_task_hash_present: Boolean(trace.storyboard_task_hash),
    kb_snapshot_hash_present: Boolean(trace.kb_snapshot_hash),
    writing_group_rule_pack_ids_present: Array.isArray(trace.writing_group_rule_pack_ids) && trace.writing_group_rule_pack_ids.length > 0,
    director_group_rule_pack_ids_present: Array.isArray(trace.director_group_rule_pack_ids) && trace.director_group_rule_pack_ids.length > 0,
    kb_oracle_affects_structure: trace.kb_oracle_affects_structure === true,
    raw_kb_rows_included: trace.raw_kb_rows_included === 0,
    raw_sample_text_absent: trace.raw_sample_text_absent === true,
    source_register_absent: trace.source_register_absent === true,
    overlay_json_absent: trace.overlay_json_absent === true,
    prompt_body_absent: trace.prompt_body_absent === true,
    response_rows_count: Number(trace.rows_count) > 0,
    ui_rows_count: Number(result.ui_evidence?.table_row_count ?? 0) > 0,
    rows_match: trace.rows_match === true,
    prompt_text_present: trace.prompt_text_present === true,
    prompt_text_boundary_passed: trace.prompt_text_boundary_passed === true,
    prompt_text_not_summary_only: trace.prompt_text_not_summary_only === true,
    visual_description_visible_frame_passed: trace.visual_description_visible_frame_passed === true,
    visual_description_no_trace: trace.visual_description_no_trace === true,
    prompt_compiled_after_final_row: trace.prompt_compiled_after_final_row === true,
    stale_task_detected: trace.stale_task_detected === false,
    stale_rows_detected: trace.stale_rows_detected === false,
    person_field_valid: trace.person_field_valid === true,
    location_not_in_person: trace.location_not_in_person === true,
    action_fragment_not_in_person: trace.action_fragment_not_in_person === true,
    scene_term_not_in_person: trace.scene_term_not_in_person === true,
    generic_role_placeholder_absent: trace.generic_role_placeholder_absent === true,
    validator_pseudo_success_detected: trace.validator_pseudo_success_detected === false,
    warning_taxonomy_classified: trace.warning_taxonomy_classified === true,
    hardfail_warning_absent: trace.hardfail_warning_absent === true,
    sampled_export_preview_present: exportAudit.every((item) => item.sampled_export_preview_present === true),
    export_storyboard_uses_confirmed_rows: exportAudit.some((item) => item.kind === "storyboard" && item.usable === true),
    export_script_uses_confirmed_body_and_rows: exportAudit.some((item) => item.kind === "script" && item.usable === true),
    golden_expectation_matched:
      sceneExpectation.label === result.scene_type_label &&
      sceneExpectation.forbidden_placeholders.every((token) => !joinedPreview.includes(token)),
  };
}

function evaluateCase(caseDef, rawResult) {
  const expectation = buildGoldenExpectation(caseDef);
  const assertions = caseDef.kind === "entry" ? entryChecks(caseDef, rawResult) : workflowChecks(caseDef, rawResult);
  const ok = Boolean(rawResult.ok) && Object.values(assertions).every(Boolean);
  return {
    ...rawResult,
    group: caseDef.kind,
    golden_expectation: expectation,
    assertions,
    ok,
    provider_model: rawResult.provider_model ?? rawResult.provider ?? "",
  };
}

function buildFirstBlocker(result, artifactRoot) {
  return {
    case_id: result.case_id,
    group: result.group,
    scene_type_id: result.scene_type,
    scene_type_label: result.scene_type_label,
    duration: result.duration,
    script_goal: result.script_goal,
    provider_model: result.provider_model,
    no_http_403: result.no_http_403,
    fallback_used: result.fallback_used,
    local_candidate: result.local_candidate,
    source_lineage: result.trace?.source_lineage ?? "",
    kb_evidence: result.trace
      ? {
          kb_snapshot_hash: result.trace.kb_snapshot_hash,
          selected_sample_ids: result.trace.selected_sample_ids,
          selected_kb_rules: result.trace.selected_kb_rules,
          writing_group_rule_pack_ids: result.trace.writing_group_rule_pack_ids,
          director_group_rule_pack_ids: result.trace.director_group_rule_pack_ids,
        }
      : null,
    rows_count: result.trace?.rows_count ?? 0,
    prompt_text: result.prompt_text_preview ?? [],
    visual_description: result.visual_description_preview ?? [],
    export_evidence: result.downloads ?? [],
    ui_evidence: result.ui_evidence ?? null,
    warning_taxonomy: result.trace?.warning_taxonomy ?? [],
    golden_expectation: result.golden_expectation,
    runner_failure_bucket: Object.entries(result.assertions ?? {}).filter(([, value]) => !value).map(([key]) => key),
    artifact_root: artifactRoot,
    error: result.error ?? "",
    fix_suggestion:
      "先以当前 case 的 assertions / trace / export audit / UI evidence 定位第一 blocker，再在 allowlist 内修复 task、KB、prompt、export 或 runner 断言。",
    next_allowlist: [
      "src/**",
      "config/**",
      "scripts/**",
      "tests/qa/**",
      "package.json",
      "README.md",
      "dist 交付说明文件",
    ],
  };
}

async function runCaseGroup(cases, label, options = {}) {
  const artifactRoot = await prepareArtifactRoot(label, options.artifactRoot);
  const results = [];
  const totalExpected = cases.length;

  for (let index = 0; index < cases.length; index += 1) {
    const caseDef = cases[index];
    const port = 9333 + index;
    const rawResult = await runWebWorkflowCase(caseDef, {
      appUrl: APP_URL,
      port,
      artifactDir: artifactRoot,
    });
    const result = evaluateCase(caseDef, rawResult);
    results.push(result);
    await writeCaseArtifact(artifactRoot, caseDef.caseId, result);

    if (options.stopOnFirstFailure && !result.ok) {
      const blocker = buildFirstBlocker(result, artifactRoot);
      const summary = {
        mode: label,
        artifact_root: artifactRoot,
        summary: summarizeResults(results, totalExpected),
        first_blocker: blocker,
      };
      await writeSummaryArtifact(artifactRoot, `${label}-summary.json`, summary);
      return summary;
    }
  }

  const summary = {
    mode: label,
    artifact_root: artifactRoot,
    summary: summarizeResults(results, totalExpected),
    results,
  };
  await writeSummaryArtifact(artifactRoot, `${label}-summary.json`, summary);
  return summary;
}

function printJson(value) {
  process.stdout.write(`${JSON.stringify(value, null, 2)}\n`);
}

async function main() {
  assertCanonicalSceneCoverage();
  assertDurationCoverage();
  assertGoldenCoverage();
  const cli = parseCli();
  const entryCases = expandEntryCases();
  const fixedCases = expandFixedCases();
  const longCases = expandLongCases();
  const artifactRootArg = cli.values.get("artifact-root");

  if (cli.flags.has("plan-only")) {
    printJson({
      mode: "plan-only",
      counts: {
        entry: entryCases.length,
        fixed: fixedCases.length,
        long: longCases.length,
        total: entryCases.length + fixedCases.length + longCases.length,
      },
      scene_type_canonical_coverage: SCENE_TYPE_CANONICAL_LIST.length,
      scene_types: SCENE_TYPE_CANONICAL_LIST,
      duration_options: matrix.fixed.durations,
      target_modes: matrix.fixed.modes,
      golden_mapping: {
        entry: Object.keys(golden.entry).length,
        scene_types: Object.keys(golden.scene_types).length,
        durations: Object.keys(golden.durations).length,
        modes: Object.keys(golden.modes).length,
      },
      note: "plan-only 只用于结构核对，不能当 formal403 完成证据。",
    });
    return;
  }

  if (cli.flags.has("targeted")) {
    printJson(await runCaseGroup(TARGETED_CASES.map((item) => ({ ...item, kind: "targeted" })), "targeted", { artifactRoot: artifactRootArg, stopOnFirstFailure: false }));
    return;
  }

  if (cli.flags.has("full16")) {
    printJson(await runCaseGroup(FULL16_CASES.map((item) => ({ ...item, kind: "full16" })), "full16", { artifactRoot: artifactRootArg, stopOnFirstFailure: true }));
    return;
  }

  if (cli.flags.has("formal403")) {
    const formalCases = [...entryCases, ...fixedCases, ...longCases];
    printJson(await runCaseGroup(formalCases, "formal403", { artifactRoot: artifactRootArg, stopOnFirstFailure: true }));
    return;
  }

  printJson({
    ok: false,
    error: "Specify one of --plan-only, --targeted, --full16, --formal403",
  });
  process.exitCode = 1;
}

main().catch((error) => {
  printJson({
    ok: false,
    error: error instanceof Error ? error.message : String(error),
  });
  process.exit(1);
});
