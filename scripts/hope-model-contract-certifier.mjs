import fs from "node:fs";
import path from "node:path";

const requiredModels = [
  "qwen-plus-2025-07-28",
  "qwen3.6-plus",
  "qwen3.6-plus-2026-04-02",
  "qvq-max-2025-03-25",
  "qwen-plus",
];

const requiredModelFields = [
  "provider_probe",
  "output_contract_ready",
  "storyboard_gate_ready",
  "prompt_text_gate_ready",
  "rewrite_ready",
  "expand_ready",
  "blocked_by_format",
  "blocked_by_fact_binding",
  "blocked_by_prompt_boundary",
  "blocked_by_provider",
  "blocked_by_output_contract",
  "blocked_by_fallback",
  "blocked_by_quality_warning_only",
  "current_gate_candidate",
];

const requiredContractFields = [
  "provider",
  "model",
  "script_goal",
  "scene_type",
  "duration",
  "accepted_snapshot_hash",
  "story_fact_frame_hash",
  "source_text_hash",
  "rows",
  "source_fact_refs",
  "required_fact_refs",
  "forbidden_fact_refs",
  "prompt_text_boundary",
  "no_fallback_evidence",
  "validator_status",
  "hard_gate_failures",
  "quality_warnings",
  "normalizer_actions",
  "creative_freedom_preserved",
  "template_overconstraint_risk",
  "sample_leakage_risk",
  "source_fact_binding_status",
  "prompt_boundary_status",
  "fallback_status",
];

function parseArgs(argv) {
  const args = {
    matrix: path.join("tests", "qa", "desktop-model-certification-matrix.json"),
    evidence: [],
    compact: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--matrix") {
      args.matrix = argv[++index];
    } else if (arg === "--evidence") {
      args.evidence.push(argv[++index]);
    } else if (arg === "--compact") {
      args.compact = true;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  return args;
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function assertCondition(condition, message, failures) {
  if (!condition) {
    failures.push(message);
  }
}

function validateMatrix(matrix) {
  const failures = [];
  assertCondition(matrix.contract_version === "desktop_model_output_contract_v1", "matrix.contract_version mismatch", failures);
  assertCondition(matrix.output_contract?.contract_kind === "structural_safety_contract_not_creative_style_template", "output contract kind must protect creative freedom", failures);
  assertCondition(matrix.creative_freedom_protection?.fixed_row_count === false, "contract must not fix row count", failures);
  assertCondition(matrix.creative_freedom_protection?.fixed_sentence_style === false, "contract must not fix sentence style", failures);
  assertCondition(matrix.hard_gate_vs_quality_warning_policy?.hard_gate_failures?.length > 0, "hard gate policy missing", failures);
  assertCondition(matrix.hard_gate_vs_quality_warning_policy?.quality_warnings?.length > 0, "quality warning policy missing", failures);
  assertCondition(matrix.normalizer_boundary?.allowed_actions?.length > 0, "normalizer allowed actions missing", failures);
  assertCondition(matrix.normalizer_boundary?.forbidden_actions?.length > 0, "normalizer forbidden actions missing", failures);

  const models = asArray(matrix.models);
  const modelNames = models.map((model) => model.model);
  for (const required of requiredModels) {
    assertCondition(modelNames.includes(required), `missing model ${required}`, failures);
  }
  for (const model of models) {
    for (const field of requiredModelFields) {
      assertCondition(Object.prototype.hasOwnProperty.call(model, field), `model ${model.model} missing ${field}`, failures);
    }
    const readyAsCandidate = model.current_gate_candidate === true;
    const allReady = model.output_contract_ready === true &&
      model.storyboard_gate_ready === true &&
      model.prompt_text_gate_ready === true &&
      model.rewrite_ready === true &&
      model.expand_ready === true;
    const blocked = requiredModelFields
      .filter((field) => field.startsWith("blocked_by_"))
      .some((field) => model[field] === true);
    assertCondition(!readyAsCandidate || (allReady && !blocked), `model ${model.model} cannot be gate candidate with incomplete or blocked state`, failures);
  }

  assertCondition(asArray(matrix.certification_cases).length === 3, "certification must define exactly three cases", failures);
  assertCondition(matrix.full16_restore_condition?.expected === 16, "full16 expected count must stay 16", failures);
  assertCondition(matrix.full16_restore_condition?.no_live_fallback === true, "full16 restore must require no live fallback", failures);
  return failures;
}

function validateEvidenceArtifact(artifact, filePath) {
  const failures = [];
  const evidence = artifact?.result?.model_output_contract_evidence ?? artifact?.model_output_contract_evidence;
  assertCondition(Boolean(evidence), `${filePath}: model_output_contract_evidence missing`, failures);
  if (!evidence) {
    return failures;
  }
  for (const field of requiredContractFields) {
    assertCondition(Object.prototype.hasOwnProperty.call(evidence, field), `${filePath}: evidence missing ${field}`, failures);
  }
  assertCondition(evidence.contract_kind === "structural_safety_contract_not_creative_style_template", `${filePath}: contract kind mismatch`, failures);
  assertCondition(evidence.raw_prompt_redacted === true, `${filePath}: raw prompt must be redacted`, failures);
  assertCondition(evidence.raw_provider_response_redacted === true, `${filePath}: raw provider response must be redacted`, failures);
  assertCondition(asArray(evidence.hard_gate_failures).every((item) => typeof item === "string"), `${filePath}: hard_gate_failures must be string codes`, failures);
  assertCondition(asArray(evidence.quality_warnings).every((item) => item.raw_values_redacted === true), `${filePath}: quality warnings must be redacted`, failures);
  assertCondition(evidence.template_overconstraint_risk?.fixed_row_count_required === false, `${filePath}: fixed row count is forbidden`, failures);
  return failures;
}

const args = parseArgs(process.argv.slice(2));
const matrix = readJson(args.matrix);
const matrixFailures = validateMatrix(matrix);
const evidenceFailures = args.evidence.flatMap((filePath) => validateEvidenceArtifact(readJson(filePath), filePath));
const failures = [...matrixFailures, ...evidenceFailures];
const result = {
  ok: failures.length === 0,
  matrix: args.matrix,
  evidence_files_checked: args.evidence.length,
  required_models: requiredModels,
  current_gate_candidates: asArray(matrix.models)
    .filter((model) => model.current_gate_candidate === true)
    .map((model) => model.model),
  failures,
  raw_values_redacted: true,
};

console.log(JSON.stringify(result, null, args.compact ? 0 : 2));
if (failures.length) {
  process.exitCode = 1;
}
