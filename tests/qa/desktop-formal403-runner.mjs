import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import zlib from "node:zlib";

const args = parseArgs(process.argv.slice(2));
const repoRoot = path.resolve(args["repo-root"] ?? process.cwd());
const provider = args.provider ?? "qwen";
const model = args.model ?? "qwen3.6-plus";
const timestamp = args.timestamp ?? timestampId();
const artifactRoot = path.resolve(
  args["artifact-root"] ?? path.join(repoRoot, "target", "qa-formal-403-fresh-66d3b75", timestamp),
);
const planOnly = flag(args["plan-only"]);
const postStopSettleMs = Number(args["post-stop-settle-ms"] ?? 3000);
const childMaxBuffer = 32 * 1024 * 1024;
let formal403PlanSummaryHash = "";

function parseArgs(argv) {
  const out = {};
  for (let index = 0; index < argv.length; index += 1) {
    const key = argv[index];
    if (!key.startsWith("--")) {
      throw new Error(`unexpected argument: ${key}`);
    }
    const name = key.slice(2);
    const next = argv[index + 1];
    if (!next || next.startsWith("--")) {
      out[name] = "1";
    } else {
      out[name] = next;
      index += 1;
    }
  }
  return out;
}

function flag(value) {
  return value === "1" || value === "true";
}

function timestampId() {
  const now = new Date();
  const pad = (value) => String(value).padStart(2, "0");
  return `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}-${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`;
}

function readJson(relativePath) {
  const text = fs.readFileSync(path.join(repoRoot, relativePath), "utf8").replace(/^\uFEFF/, "");
  return JSON.parse(text);
}

function hashText(value) {
  return crypto.createHash("sha256").update(String(value), "utf8").digest("hex");
}

function hashFile(relativePath) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(repoRoot, relativePath)))
    .digest("hex");
}

function sanitizeCaseId(value) {
  return String(value)
    .replace(/[^A-Za-z0-9_.-]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .slice(0, 180);
}

function resolveCaseSelector(selector) {
  const match = String(selector).match(/^(.+?)#cases\[(id|case_id)=([^\]]+)\]\.(source_text|source)$/);
  if (!match) {
    throw new Error(`unsupported source_ref selector: ${selector}`);
  }
  const [, file, field, id, sourceField] = match;
  const manifest = readJson(file);
  const item = (manifest.cases ?? []).find((candidate) => String(candidate[field] ?? "") === id);
  if (!item) {
    throw new Error(`source_ref case not found: ${selector}`);
  }
  const source = String(item[sourceField] ?? "");
  if (!source.trim()) {
    throw new Error(`source_ref is empty: ${selector}`);
  }
  return source;
}

function extractDocxText(docxPath) {
  const buf = fs.readFileSync(docxPath);
  const u16 = (offset) => buf.readUInt16LE(offset);
  const u32 = (offset) => buf.readUInt32LE(offset);
  let eocd = -1;
  for (let index = buf.length - 22; index >= Math.max(0, buf.length - 65557); index -= 1) {
    if (u32(index) === 0x06054b50) {
      eocd = index;
      break;
    }
  }
  if (eocd < 0) {
    throw new Error("DOCX central directory not found");
  }
  const cdCount = u16(eocd + 10);
  let offset = u32(eocd + 16);
  let documentEntry = null;
  for (let entry = 0; entry < cdCount; entry += 1) {
    if (u32(offset) !== 0x02014b50) {
      throw new Error("DOCX central directory is invalid");
    }
    const method = u16(offset + 10);
    const compressedSize = u32(offset + 20);
    const nameLength = u16(offset + 28);
    const extraLength = u16(offset + 30);
    const commentLength = u16(offset + 32);
    const localOffset = u32(offset + 42);
    const name = buf.slice(offset + 46, offset + 46 + nameLength).toString("utf8");
    if (name === "word/document.xml") {
      documentEntry = { method, compressedSize, localOffset };
    }
    offset += 46 + nameLength + extraLength + commentLength;
  }
  if (!documentEntry) {
    throw new Error("DOCX word/document.xml not found");
  }
  const localOffset = documentEntry.localOffset;
  if (u32(localOffset) !== 0x04034b50) {
    throw new Error("DOCX local file header is invalid");
  }
  const nameLength = u16(localOffset + 26);
  const extraLength = u16(localOffset + 28);
  const dataStart = localOffset + 30 + nameLength + extraLength;
  const compressed = buf.slice(dataStart, dataStart + documentEntry.compressedSize);
  const xml = (documentEntry.method === 8 ? zlib.inflateRawSync(compressed) : compressed).toString("utf8");
  return xml
    .replace(/<w:tab\/>/g, "\t")
    .replace(/<w:br\/>/g, "\n")
    .replace(/<\/w:p>/g, "\n")
    .replace(/<[^>]+>/g, "")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&amp;/g, "&")
    .replace(/&quot;/g, "\"")
    .replace(/&apos;/g, "'")
    .replace(/\s+\n/g, "\n")
    .trim();
}

function loadCases() {
  const matrix = readJson(path.join("tests", "qa", "desktop-403-matrix.json"));
  const kbGoldenMappingPath = String(matrix.kb_golden_mapping_manifest ?? "");
  if (!kbGoldenMappingPath) {
    throw new Error("desktop-403 matrix missing kb_golden_mapping_manifest");
  }
  const kbGoldenMapping = readJson(kbGoldenMappingPath);
  const kbGoldenMappingHash = hashFile(kbGoldenMappingPath);
  const fourGroups = readJson(path.join("tests", "qa", "desktop-four-groups.json"));
  const fixedSources = matrix.fixed_text_sources.map((item, index) => ({
    id: item.id ?? `fixed_text_${index + 1}`,
    sourceRef: item.source_ref,
    source: resolveCaseSelector(item.source_ref),
  }));
  if (fixedSources.length !== matrix.fixed_text_count) {
    throw new Error(`fixed source count mismatch: ${fixedSources.length} != ${matrix.fixed_text_count}`);
  }
  const docxPath = args["long-text-docx"] ?? matrix.long_text_sample?.original_machine_path;
  if (!docxPath || !fs.existsSync(docxPath)) {
    throw new Error(`long-text docx missing: ${docxPath}`);
  }
  const longText = extractDocxText(docxPath);
  if (!longText.trim()) {
    throw new Error(`long-text docx extracted empty text: ${docxPath}`);
  }
  const cases = [];
  for (const id of matrix.case_count_breakdown.entry_baseline_matrix.case_ids) {
    const item = fourGroups.cases.find((candidate) => candidate.id === id);
    if (!item) {
      throw new Error(`entry baseline case missing: ${id}`);
    }
    cases.push({
      id: `entry_${id}`,
      group: "entry_baseline",
      sourceId: id,
      source: item.source_text,
      sourceHash: hashText(item.source_text),
      scene: item.scene_label,
      sceneType: item.scene_type,
      duration: "15",
      durationMode: "fixed_seconds",
      scriptGoal: "rewrite",
    });
  }
  for (const source of fixedSources) {
    for (const scene of matrix.scene_types) {
      for (const duration of matrix.fixed_durations_seconds) {
        cases.push({
          id: `fixed_${source.id}_${scene.value}_${duration}s`,
          group: "fixed_matrix",
          sourceId: source.id,
          sourceRef: source.sourceRef,
          source: source.source,
          sourceHash: hashText(source.source),
          scene: scene.label,
          sceneType: scene.value,
          duration: String(duration),
          durationMode: "fixed_seconds",
          scriptGoal: "rewrite",
        });
      }
    }
  }
  for (const scene of matrix.scene_types) {
    cases.push({
      id: `long_docx_${scene.value}`,
      group: "long_text_matrix",
      sourceId: "long_text_docx",
      sourceRef: docxPath,
      source: longText,
      sourceHash: hashText(longText),
      scene: scene.label,
      sceneType: scene.value,
      duration: "long_text_auto",
      durationMode: "long_text_auto",
      scriptGoal: "rewrite",
    });
  }
  if (cases.length !== matrix.total_case_count) {
    throw new Error(`formal403 case count mismatch: ${cases.length} != ${matrix.total_case_count}`);
  }
  return {
    matrix,
    kbGoldenMappingPath,
    kbGoldenMappingHash,
    kbGoldenMappingContract: {
      manifest_name: kbGoldenMapping.name ?? kbGoldenMapping.contract_version ?? "desktop-403-kb-golden-mapping",
      reference_only: kbGoldenMapping.reference_only === true,
      human_re_review_required: kbGoldenMapping.human_re_review_required === true ||
        kbGoldenMapping.human_review_required_before_approval_metadata === true,
      approved_samples: Number(kbGoldenMapping.approved_samples ?? 0),
    },
    cases,
    docxPath,
    longTextHash: hashText(longText),
    longTextChars: longText.length,
  };
}

function parseJsonFile(file) {
  return JSON.parse(fs.readFileSync(file, "utf8").replace(/^\uFEFF/, ""));
}

function runCommand(command, commandArgs, options = {}) {
  const result = spawnSync(command, commandArgs, {
    cwd: repoRoot,
    encoding: "utf8",
    env: options.env ?? process.env,
    input: options.input,
    timeout: options.timeoutMs ?? 900000,
    windowsHide: true,
    maxBuffer: options.maxBuffer ?? childMaxBuffer,
  });
  return {
    status: result.status,
    signal: result.signal,
    error: result.error ? String(result.error.message ?? result.error) : null,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

function writeJson(file, value) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

function writeRaw(file, value) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, value, "utf8");
}

function sleepMs(ms) {
  if (!Number.isFinite(ms) || ms <= 0) {
    return;
  }
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, Math.floor(ms));
}

function launcherArgs(extra) {
  return [
    "-NoProfile",
    "-ExecutionPolicy",
    "Bypass",
    "-File",
    path.join(repoRoot, "scripts", "start-hope-release-cdp.ps1"),
    "-RepoRoot",
    repoRoot,
    ...extra,
  ];
}

function runStopOnly(casePaths) {
  const stop = runCommand("powershell.exe", launcherArgs(["-StopOnly"]), { timeoutMs: 120000 });
  writeRaw(casePaths.stoponly, stop.stdout || JSON.stringify({
    ok: false,
    stage: "stoponly_process_no_stdout",
    process_error: stop.error,
    status: stop.status,
    signal: stop.signal,
  }, null, 2));
  if (stop.stderr.trim()) {
    writeRaw(`${casePaths.stoponly}.stderr.log`, stop.stderr);
  }
  let parsed = null;
  try {
    parsed = parseJsonFile(casePaths.stoponly);
  } catch (error) {
    parsed = { ok: false, parse_error: error.message };
  }
  return { command: stop, parsed };
}

function cleanupOk(stop) {
  return stop?.ok === true &&
    Array.isArray(stop.hope_app_remaining_pids) &&
    stop.hope_app_remaining_pids.length === 0 &&
    Array.isArray(stop.webview2_remaining_pids) &&
    stop.webview2_remaining_pids.length === 0 &&
    stop.port_released === true &&
    stop.exe_unlocked === true;
}

function launchOk(launch) {
  return launch?.ok === true &&
    launch.cdp_ready === true &&
    launch.target_url === "http://tauri.localhost/#/workbench" &&
    launch.repo_root === repoRoot;
}

function runnerOk(payload) {
  const r = payload?.result ?? {};
  const prompt = r.prompt_text_boundary_evidence ?? {};
  const story = r.story_fact_frame_binding_evidence ?? {};
  const task = r.task_binding_evidence ?? {};
  const visible = r.visible_body_scene_duration_evidence ?? {};
  const fallback = r.fallback_gate_evidence ?? {};
  const validator = r.validator_gate_evidence ?? {};
  return r.ok === true &&
    Array.isArray(r.runner_assert_failures) &&
    r.runner_assert_failures.length === 0 &&
    Array.isArray(r.hard_gate_failures) &&
    r.hard_gate_failures.length === 0 &&
    fallback.no_http_403 === true &&
    fallback.no_live_fallback === true &&
    fallback.fallback_used === false &&
    fallback.local_candidate === false &&
    validator.rows_match === true &&
    Array.isArray(validator.row_diffs) &&
    validator.row_diffs.length === 0 &&
    story.present === true &&
    Array.isArray(story.kb_rule_pack_ids) &&
    String(story.kb_snapshot_hash ?? "").trim().length > 0 &&
    story.stale_binding_detected === false &&
    task.present === true &&
    String(task.task_scene_type ?? "").trim().length > 0 &&
    Number(task.task_duration_seconds ?? 0) > 0 &&
    visible.body_has_duration === true &&
    (visible.body_has_scene_label === true || visible.body_has_selected_scene_text === true) &&
    visible.rewrite_body_has_scene_label === true &&
    visible.rewrite_body_has_duration === true &&
    visible.ui_rows_have_scene_label === true &&
    visible.ui_rows_have_duration === true &&
    Array.isArray(story.missing_source_facts) &&
    story.missing_source_facts.length === 0 &&
    Array.isArray(story.forbidden_fact_hits) &&
    story.forbidden_fact_hits.length === 0 &&
    prompt.prompt_text_boundary_passed === true &&
    prompt.prompt_text_missing === false &&
    prompt.sample_text_absent === true &&
    prompt.smoke_extracts_absent === true &&
    prompt.raw_kb_rows_absent === true &&
    prompt.qa_reference_absent === true &&
    prompt.source_sample_id_absent === true &&
    prompt.sample_entity_marker_absent === true &&
    prompt.source_register_absent === true &&
    prompt.overlay_json_absent === true;
}

function classifyFailure({ launch, runner, stop }) {
  if (!launchOk(launch)) {
    return "shell/CDP";
  }
  const r = runner?.result ?? {};
  if (r.fallback_gate_evidence?.no_http_403 === false || r.provider_http_status === 403) {
    return "provider/http_403";
  }
  if (r.fallback_gate_evidence?.fallback_used === true || r.fallback_gate_evidence?.no_live_fallback === false) {
    return "fallback";
  }
  if (r.validator_gate_evidence?.rows_match === false || (r.validator_gate_evidence?.row_diffs ?? []).length) {
    return "rows mismatch";
  }
  if (!r.story_fact_frame_binding_evidence?.present ||
      !Array.isArray(r.story_fact_frame_binding_evidence?.kb_rule_pack_ids) ||
      !String(r.story_fact_frame_binding_evidence?.kb_snapshot_hash ?? "").trim()) {
    return "KB evidence missing";
  }
  if (!r.task_binding_evidence?.present ||
      !String(r.task_binding_evidence?.task_scene_type ?? "").trim() ||
      Number(r.task_binding_evidence?.task_duration_seconds ?? 0) <= 0) {
    return "task binding";
  }
  const visible = r.visible_body_scene_duration_evidence ?? {};
  if (visible.body_has_duration !== true ||
      (visible.body_has_scene_label !== true && visible.body_has_selected_scene_text !== true)) {
    return "visible scene/duration";
  }
  if (visible.rewrite_body_has_scene_label !== true ||
      visible.rewrite_body_has_duration !== true ||
      visible.ui_rows_have_scene_label !== true ||
      visible.ui_rows_have_duration !== true) {
    return "visible rewrite/rows semantic chain";
  }
  const prompt = r.prompt_text_boundary_evidence ?? {};
  if (prompt.prompt_text_boundary_passed !== true ||
      prompt.sample_text_absent !== true ||
      prompt.smoke_extracts_absent !== true ||
      prompt.raw_kb_rows_absent !== true ||
      prompt.qa_reference_absent !== true ||
      prompt.source_sample_id_absent !== true ||
      prompt.sample_entity_marker_absent !== true ||
      prompt.source_register_absent !== true ||
      prompt.overlay_json_absent !== true) {
    return "sample leakage";
  }
  if (r.story_fact_frame_binding_evidence?.stale_binding_detected ||
      (r.story_fact_frame_binding_evidence?.missing_source_facts ?? []).length ||
      (r.story_fact_frame_binding_evidence?.forbidden_fact_hits ?? []).length) {
    return "story fact frame";
  }
  if (!cleanupOk(stop)) {
    return "shell/CDP cleanup";
  }
  return "validator";
}

function runCase(testCase, index, total) {
  const safeId = sanitizeCaseId(testCase.id);
  const casePaths = {
    launch: path.join(artifactRoot, `${safeId}.launch.json`),
    runner: path.join(artifactRoot, `${safeId}.json`),
    stoponly: path.join(artifactRoot, `${safeId}.stoponly.json`),
  };
  if (index > 1) {
    sleepMs(postStopSettleMs);
  }
  const launch = runCommand("powershell.exe", launcherArgs([
    "-Provider",
    provider,
    "-Model",
    model,
    "-NoProxy",
    "-QaProviderHardFail",
    "-WebView2ArgumentMode",
    "AppDefault",
    "-StopExisting",
    "-StopOnCdpFailure",
    "-WaitSeconds",
    "20",
  ]), { timeoutMs: 180000 });
  writeRaw(casePaths.launch, launch.stdout || JSON.stringify({
    ok: false,
    stage: "launch_process_no_stdout",
    process_error: launch.error,
    status: launch.status,
    signal: launch.signal,
  }, null, 2));
  if (launch.stderr.trim()) {
    writeRaw(`${casePaths.launch}.stderr.log`, launch.stderr);
  }
  let launchParsed = null;
  try {
    launchParsed = parseJsonFile(casePaths.launch);
  } catch (error) {
    launchParsed = { ok: false, parse_error: error.message };
  }

  let runnerParsed = null;
  let runnerCommand = null;
  if (launchOk(launchParsed)) {
    const childEnv = { ...process.env };
    for (const key of ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy"]) {
      delete childEnv[key];
    }
    childEnv.NO_PROXY = "localhost,127.0.0.1,::1";
    runnerCommand = runCommand("node", [
      path.join(repoRoot, "scripts", "hope-ui-driven-trace-runner.mjs"),
      "--case",
      safeId,
      "--repo-root",
      repoRoot,
      "--gate-level",
      "formal403",
      "--parent-gate-summary-hash",
      formal403PlanSummaryHash,
      "--script-goal",
      testCase.scriptGoal,
      "--scene",
      testCase.scene,
      "--source-stdin",
      "1",
      "--duration",
      testCase.duration,
      "--duration-mode",
      testCase.durationMode,
      "--expected-provider",
      provider,
      "--expected-model",
      model,
      "--assert-binding",
      "1",
      "--assert-no-proxy-env",
      "1",
      "--compact",
      "1",
    ], { env: childEnv, input: testCase.source, timeoutMs: 900000 });
    writeRaw(casePaths.runner, runnerCommand.stdout || "{}");
    if (runnerCommand.stderr.trim()) {
      writeRaw(`${casePaths.runner}.stderr.log`, runnerCommand.stderr);
    }
    try {
      runnerParsed = parseJsonFile(casePaths.runner);
    } catch (error) {
      runnerParsed = { result: { ok: false, stage: "runner_json_parse", parse_error: error.message } };
    }
  } else {
    runnerParsed = { result: { ok: false, stage: "launch_failed_not_run" } };
    writeJson(casePaths.runner, runnerParsed);
  }
  const stop = runStopOnly(casePaths);
  const caseOk = launchOk(launchParsed) && runnerOk(runnerParsed) && cleanupOk(stop.parsed);
  return {
    id: testCase.id,
    safe_id: safeId,
    index,
    total,
    group: testCase.group,
    source_id: testCase.sourceId,
    source_ref: testCase.sourceRef ?? null,
    source_hash: testCase.sourceHash,
    scene_type: testCase.sceneType,
    duration: testCase.duration,
    script_goal: testCase.scriptGoal,
    ok: caseOk,
    failure_bucket: caseOk ? null : classifyFailure({ launch: launchParsed, runner: runnerParsed, stop: stop.parsed }),
    launch_exit: launch.status,
    runner_exit: runnerCommand?.status ?? null,
    stoponly_exit: stop.command.status,
    artifacts: casePaths,
  };
}

const plan = loadCases();
fs.mkdirSync(artifactRoot, { recursive: true });
const evidenceListPath = path.join(artifactRoot, "formal403-evidence-list.json");
const summaryPath = path.join(artifactRoot, "formal403-summary.json");
const planSummary = {
  artifact_root: artifactRoot,
  provider,
  model,
  post_stop_settle_ms: postStopSettleMs,
  repo_root: repoRoot,
  kb_golden_mapping_manifest: plan.kbGoldenMappingPath,
  kb_golden_mapping_hash: plan.kbGoldenMappingHash,
  kb_golden_mapping_contract: plan.kbGoldenMappingContract,
  gate_layer: "formal403",
  targeted: false,
  full16: false,
  formal403: true,
  expected_total: plan.matrix.total_case_count,
  case_counts: {
    entry_baseline: plan.cases.filter((item) => item.group === "entry_baseline").length,
    fixed_matrix: plan.cases.filter((item) => item.group === "fixed_matrix").length,
    long_text_matrix: plan.cases.filter((item) => item.group === "long_text_matrix").length,
  },
  long_text_docx: {
    path: plan.docxPath,
    sha256: crypto.createHash("sha256").update(fs.readFileSync(plan.docxPath)).digest("hex"),
    extracted_chars: plan.longTextChars,
    extracted_text_sha256: plan.longTextHash,
    raw_text_redacted: true,
  },
  case_ids: plan.cases.map((item) => ({
    id: item.id,
    group: item.group,
    source_id: item.sourceId,
    source_ref: item.sourceRef ?? null,
    source_hash: item.sourceHash,
    scene_type: item.sceneType,
    duration: item.duration,
    script_goal: item.scriptGoal,
  })),
};
formal403PlanSummaryHash = hashText(JSON.stringify(planSummary));
planSummary.parent_gate_summary_hash = formal403PlanSummaryHash;
writeJson(path.join(artifactRoot, "formal403-plan.json"), planSummary);
if (planOnly) {
  writeJson(summaryPath, { ...planSummary, plan_only: true, ok: true });
  console.log(JSON.stringify({ ok: true, plan_only: true, artifact_root: artifactRoot, case_counts: planSummary.case_counts }, null, 2));
  process.exit(0);
}

const results = [];
for (let index = 0; index < plan.cases.length; index += 1) {
  const result = runCase(plan.cases[index], index + 1, plan.cases.length);
  results.push(result);
  writeJson(evidenceListPath, { evidence_files: results.map((item) => item.artifacts.runner) });
  const failed = results.filter((item) => !item.ok);
  writeJson(summaryPath, {
    ...planSummary,
    ok: failed.length === 0 && results.length === plan.cases.length,
    executed: results.length,
    passed: results.filter((item) => item.ok).length,
    failed: failed.length,
    not_run: plan.cases.length - results.length,
    first_failure: failed[0] ?? null,
    results,
  });
  if (!result.ok) {
    console.log(JSON.stringify({ ok: false, artifact_root: artifactRoot, executed: results.length, failed: 1, first_failure: result }, null, 2));
    process.exit(2);
  }
}
console.log(JSON.stringify({ ok: true, artifact_root: artifactRoot, executed: results.length, passed: results.length, failed: 0, not_run: 0 }, null, 2));
