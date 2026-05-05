import fs from "node:fs";
import path from "node:path";

const requiredTextGateModels = [
  "qwen-plus-2025-07-28",
  "qwen3.6-plus",
  "qwen3.6-plus-2026-04-02",
  "qwen3.6-max-preview",
  "qwen3-max-2026-01-23",
  "qwen3.6-flash",
  "qwen-plus-2025-12-01",
];

const aliasCompatibilityModels = ["qwen-plus"];

const forbiddenRequiredGateModelPatterns = [
  /qvq/i,
  /\bvl\b/i,
  /vision|visual|video/i,
  /math/i,
  /coder/i,
  /happyhorse/i,
];

const requiredRunnerFailureStages = [
  "provider_retry_exhausted_hard_fail",
  "qa_hard_fail_evidence_missing",
  "model_not_found",
  "entitlement",
  "quota",
  "http_403",
  "invalid_parameter",
  "validator_hard_gate_fail",
  "prompt_text_boundary",
];

const requiredModelFields = [
  "gate_role",
  "required_gate_model",
  "not_required_gate",
  "callable_status",
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
  "provider_availability",
  "live3_certification",
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
    availability: [],
    callableRequiredModels: [],
    requireLive3Pool: false,
    compact: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--matrix") {
      args.matrix = argv[++index];
    } else if (arg === "--evidence") {
      args.evidence.push(argv[++index]);
    } else if (arg === "--availability") {
      args.availability.push(argv[++index]);
    } else if (arg === "--callable-required-model") {
      args.callableRequiredModels.push(argv[++index]);
    } else if (arg === "--require-live3-pool") {
      args.requireLive3Pool = true;
    } else if (arg === "--compact") {
      args.compact = true;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  return args;
}

function readJson(filePath) {
  const bytes = fs.readFileSync(filePath);
  let text;
  if (bytes[0] === 0xff && bytes[1] === 0xfe) {
    text = new TextDecoder("utf-16le").decode(bytes);
  } else if (bytes[0] === 0xfe && bytes[1] === 0xff) {
    text = new TextDecoder("utf-16be").decode(bytes);
  } else {
    text = new TextDecoder("utf-8").decode(bytes);
  }
  return JSON.parse(text.replace(/^\uFEFF/, ""));
}

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function assertCondition(condition, message, failures) {
  if (!condition) {
    failures.push(message);
  }
}

function hasOwn(value, field) {
  return Object.prototype.hasOwnProperty.call(value ?? {}, field);
}

function sameStringArray(actual, expected) {
  return Array.isArray(actual) &&
    actual.length === expected.length &&
    actual.every((item, index) => item === expected[index]);
}

function isForbiddenRequiredGateModel(modelName) {
  return forbiddenRequiredGateModelPatterns.some((pattern) => pattern.test(String(modelName ?? "")));
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
  assertCondition(matrix.provider_availability_probe_contract?.raw_values_redacted === true, "provider availability probe contract must be redacted", failures);
  for (const field of ["model", "http_status", "error_category", "error_code", "choices_present", "callable"]) {
    assertCondition(asArray(matrix.provider_availability_probe_contract?.required_result_fields).includes(field), `provider availability probe contract missing ${field}`, failures);
  }
  for (const stage of requiredRunnerFailureStages) {
    assertCondition(asArray(matrix.failure_classification_policy?.runner_stage_order).includes(stage), `failure classification missing ${stage}`, failures);
  }
  assertCondition(
    sameStringArray(matrix.required_text_gate_models, requiredTextGateModels),
    "required_text_gate_models must match the authorized text-only live3 pool",
    failures,
  );
  assertCondition(
    sameStringArray(matrix.model_strategy, requiredTextGateModels),
    "model_strategy must match required_text_gate_models and exclude alias/non-text models",
    failures,
  );
  const aliasReferenceModels = asArray(matrix.alias_compatibility_references).map((item) => item.model);
  for (const alias of aliasCompatibilityModels) {
    assertCondition(aliasReferenceModels.includes(alias), `alias_compatibility_references missing ${alias}`, failures);
  }
  const excludedModelsText = JSON.stringify(matrix.excluded_models ?? []).toLowerCase();
  for (const excludedPattern of ["qvq", "vl", "video", "math", "coder"]) {
    assertCondition(excludedModelsText.includes(excludedPattern), `excluded_models missing ${excludedPattern}`, failures);
  }
  for (const modelName of asArray(matrix.required_text_gate_models)) {
    assertCondition(!isForbiddenRequiredGateModel(modelName), `required_text_gate_models contains non-text or excluded model ${modelName}`, failures);
  }

  const models = asArray(matrix.models);
  const modelNames = models.map((model) => model.model);
  for (const required of requiredTextGateModels) {
    assertCondition(modelNames.includes(required), `missing model ${required}`, failures);
  }
  for (const alias of aliasCompatibilityModels) {
    assertCondition(modelNames.includes(alias), `missing alias compatibility reference ${alias}`, failures);
  }
  for (const model of models) {
    for (const field of requiredModelFields) {
      assertCondition(Object.prototype.hasOwnProperty.call(model, field), `model ${model.model} missing ${field}`, failures);
    }
    const isRequiredTextGateModel = requiredTextGateModels.includes(model.model);
    const isAliasCompatibilityModel = aliasCompatibilityModels.includes(model.model);
    assertCondition(
      !isForbiddenRequiredGateModel(model.model) || model.not_required_gate === true,
      `model ${model.model} is excluded and must be not_required_gate`,
      failures,
    );
    if (isRequiredTextGateModel) {
      assertCondition(model.required_gate_model === true, `model ${model.model} must be required_gate_model`, failures);
      assertCondition(model.not_required_gate === false, `model ${model.model} must not be marked not_required_gate`, failures);
      assertCondition(model.gate_role === "required_text_gate_model", `model ${model.model} gate_role mismatch`, failures);
    }
    if (isAliasCompatibilityModel) {
      assertCondition(model.required_gate_model === false, `alias ${model.model} must not be required_gate_model`, failures);
      assertCondition(model.not_required_gate === true, `alias ${model.model} must be not_required_gate`, failures);
      assertCondition(model.gate_role === "alias_compatibility_reference", `alias ${model.model} gate_role mismatch`, failures);
      assertCondition(model.alias_compatibility_reference === true, `alias ${model.model} must declare alias_compatibility_reference`, failures);
      assertCondition(model.current_gate_candidate === false, `alias ${model.model} cannot be a current gate candidate`, failures);
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
  const prerequisite = matrix.full16_restore_prerequisite;
  assertCondition(Boolean(prerequisite), "full16_restore_prerequisite missing", failures);
  assertCondition(prerequisite?.expected === 16, "full16 expected count must stay 16", failures);
  assertCondition(prerequisite?.not_executed_in_this_node === true, "full16 must not be executed in this node", failures);
  assertCondition(prerequisite?.full16_remains_blocked_until_live_3_case_acceptance === true, "full16 must remain blocked until live3 acceptance", failures);
  assertCondition(prerequisite?.requires_all_callable_required_text_gate_models_live3_3of3 === true, "full16 restore must require all callable required text models live3 3/3", failures);
  assertCondition(prerequisite?.min_passing_text_models >= 5, "full16 restore must require at least five passing text models", failures);
  assertCondition(prerequisite?.requires_one_model_passing_3_case_certification !== true, "full16 restore must not be satisfied by one model", failures);
  assertCondition(prerequisite?.per_model_live3_requirement?.expected === 3, "live3 expected count must be 3", failures);
  assertCondition(prerequisite?.per_model_live3_requirement?.executed === 3, "live3 executed count must be 3", failures);
  assertCondition(prerequisite?.per_model_live3_requirement?.passed === 3, "live3 passed count must be 3", failures);
  assertCondition(prerequisite?.per_model_live3_requirement?.failed === 0, "live3 failed count must be 0", failures);
  assertCondition(prerequisite?.per_model_live3_requirement?.not_run === 0, "live3 not_run count must be 0", failures);
  assertCondition(prerequisite?.no_live_fallback === true, "full16 restore must require no live fallback", failures);
  return failures;
}

function uniqueStrings(values) {
  return Array.from(new Set(values.map((item) => String(item ?? "").trim()).filter(Boolean)));
}

function uniqueNumbers(values) {
  return Array.from(new Set(values.map(Number).filter(Number.isFinite)));
}

function normalizedProviderRetryMissingReason(reason, fieldName, context) {
  const value = String(reason ?? "").trim();
  if (value.startsWith("validator_hard_gate_trace_has_no_provider_")) {
    return `not_applicable_validator_hard_gate_qa_hard_fail_no_provider_transport_retry_${fieldName}_metadata`;
  }
  const legacyCompactMissing = value.startsWith("legacy_compact_artifact_missing_provider_");
  if (value && !legacyCompactMissing) {
    return value;
  }
  if (context.providerRetryObserved) {
    return `provider_transport_retry_${fieldName}_metadata_not_emitted_or_not_captured`;
  }
  if (context.qaHardFail && context.validatorHardFail) {
    return `not_applicable_validator_hard_gate_qa_hard_fail_no_provider_transport_retry_${fieldName}_metadata`;
  }
  if (context.qaHardFail) {
    return `not_applicable_qa_hard_fail_without_provider_transport_retry_${fieldName}_metadata`;
  }
  if (value) {
    return value;
  }
  return null;
}

function providerRetryEvidenceFromArtifact(artifact) {
  const result = artifact?.result ?? artifact ?? {};
  const evidence = evidenceFromArtifact(artifact) ?? {};
  const fallback = result?.fallback_gate_evidence ?? {};
  const modelAvailability = result?.model_availability ?? fallback?.model_availability ?? {};
  const retryEvidence = evidence?.provider_retry_evidence ??
    result?.provider_retry_evidence ??
    fallback?.provider_retry_evidence ??
    fallback?.retry_timeline ??
    result?.retry_timeline_artifact ??
    {};
  const hardGateFailures = uniqueStrings([
    ...asArray(result?.hard_gate_failures),
    ...asArray(evidence?.hard_gate_failures),
  ]);
  const attemptsObserved = uniqueNumbers(
    retryEvidence?.attempts_observed ?? result?.attempts_observed ?? fallback?.attempts_observed ?? [],
  );
  const elapsedBuckets = uniqueStrings(
    retryEvidence?.elapsed_buckets ?? result?.elapsed_buckets ?? fallback?.elapsed_buckets ?? [],
  );
  const maxAttemptsObserved = uniqueNumbers(
    retryEvidence?.max_attempts_observed ?? result?.max_attempts_observed ?? fallback?.max_attempts_observed ?? [],
  );
  const rawAttemptsMissingReason =
    retryEvidence?.attempts_observed_missing_reason ??
    result?.attempts_observed_missing_reason ??
    fallback?.attempts_observed_missing_reason ??
    null;
  const rawElapsedBucketsMissingReason =
    retryEvidence?.elapsed_buckets_missing_reason ??
    result?.elapsed_buckets_missing_reason ??
    fallback?.elapsed_buckets_missing_reason ??
    null;
  const hasProviderRetryHardFail = hardGateFailures.includes("provider_retry_exhausted_hard_fail") ||
    retryEvidence?.provider_retry_exhausted_hard_fail_edge === true ||
    result?.provider_retry_exhausted_hard_fail_edge === true ||
    fallback?.provider_retry_exhausted_hard_fail_edge === true;
  const hasValidatorHardFail = hardGateFailures.includes("validator_hard_gate_fail") ||
    retryEvidence?.validator_hard_gate_fail_edge === true ||
    result?.validator_hard_gate_fail_edge === true ||
    fallback?.validator_hard_gate_fail_edge === true;
  const providerHttpStatusPresent =
    hasOwn(retryEvidence, "provider_http_status") ||
    hasOwn(result, "provider_http_status") ||
    hasOwn(fallback, "provider_http_status") ||
    hasOwn(modelAvailability, "http_status");
  const retryExhausted = Boolean(retryEvidence?.retry_exhausted ?? result?.retry_exhausted ?? fallback?.retry_exhausted);
  const qaHardFail = Boolean(retryEvidence?.qa_hard_fail ?? result?.qa_hard_fail ?? fallback?.qa_hard_fail);
  const validatorOnlyMissingReason = [rawAttemptsMissingReason, rawElapsedBucketsMissingReason]
    .some((reason) => String(reason ?? "").startsWith("validator_hard_gate_trace_has_no_provider_"));
  const notApplicableProviderRetryMissingReason = [rawAttemptsMissingReason, rawElapsedBucketsMissingReason]
    .some((reason) => String(reason ?? "").startsWith("not_applicable_"));
  const rawProviderRetryClassification = String(
    retryEvidence?.provider_retry_classification ??
    result?.provider_retry_classification ??
    fallback?.provider_retry_classification ??
    "",
  ).trim();
  const hasCurrentProviderRetryTelemetry =
    hasOwn(retryEvidence, "provider_retry_observed") ||
    hasOwn(result, "provider_retry_observed") ||
    hasOwn(fallback, "provider_retry_observed") ||
    hasOwn(retryEvidence, "provider_retry_classification") ||
    hasOwn(result, "provider_retry_classification") ||
    hasOwn(fallback, "provider_retry_classification") ||
    hasOwn(retryEvidence, "provider_attempt_metadata_status") ||
    hasOwn(result, "provider_attempt_metadata_status") ||
    hasOwn(fallback, "provider_attempt_metadata_status");
  const rawClassificationIsProviderRetry =
    /^provider_transport_retry_/.test(rawProviderRetryClassification);
  const providerRetryObserved =
    retryEvidence?.provider_retry_observed === true ||
    retryEvidence?.provider_attempt_metadata_status === "observed" ||
    attemptsObserved.length > 0 ||
    maxAttemptsObserved.length > 0 ||
    elapsedBuckets.length > 0 ||
    (retryExhausted && !validatorOnlyMissingReason && !notApplicableProviderRetryMissingReason && rawClassificationIsProviderRetry);
  const missingReasonContext = {
    providerRetryObserved,
    qaHardFail,
    validatorHardFail: hasValidatorHardFail,
  };
  const attemptsMissingReason = normalizedProviderRetryMissingReason(
    rawAttemptsMissingReason ??
      (hasProviderRetryHardFail && attemptsObserved.length === 0
        ? "legacy_compact_artifact_missing_provider_attempts_metadata"
        : null),
    "attempts",
    missingReasonContext,
  );
  const elapsedBucketsMissingReason = normalizedProviderRetryMissingReason(
    rawElapsedBucketsMissingReason ??
      (hasProviderRetryHardFail && elapsedBuckets.length === 0
        ? "legacy_compact_artifact_missing_provider_elapsed_bucket_metadata"
        : null),
    "elapsed_bucket",
    missingReasonContext,
  );
  const providerAttemptMetadataStatus = providerRetryObserved
    ? (attemptsObserved.length || elapsedBuckets.length ? "observed" : "provider_retry_observed_without_attempt_elapsed_metadata")
    : qaHardFail && hasValidatorHardFail
      ? "not_applicable_validator_hard_gate_qa_hard_fail_without_provider_transport_retry"
      : qaHardFail
        ? "not_applicable_qa_hard_fail_without_provider_transport_retry"
        : "not_applicable_no_provider_retry";
  const providerRetryExhaustedHardFailEdge = hasProviderRetryHardFail && providerRetryObserved;
  const legacyProviderRetryExhaustedHardFailEdge =
    hasProviderRetryHardFail && !providerRetryObserved && !hasCurrentProviderRetryTelemetry;
  const providerRetryClassification = rawProviderRetryClassification || (providerRetryObserved
    ? providerRetryExhaustedHardFailEdge
      ? "provider_transport_retry_exhausted"
      : Boolean(retryEvidence?.retry_recovered ?? result?.retry_recovered ?? fallback?.retry_recovered)
        ? "provider_transport_retry_recovered"
        : "provider_transport_retry_metadata_observed"
    : hasProviderRetryHardFail && qaHardFail && hasValidatorHardFail
      ? "legacy_conflated_validator_hard_gate_qa_hard_fail_not_provider_transport_retry"
      : qaHardFail && hasValidatorHardFail
        ? "validator_hard_gate_qa_hard_fail_not_provider_transport_retry"
        : qaHardFail
          ? "qa_hard_fail_without_provider_transport_retry"
          : "not_applicable_no_provider_retry");
  return {
    retry_recovered: Boolean(retryEvidence?.retry_recovered ?? result?.retry_recovered ?? fallback?.retry_recovered),
    retry_recovered_count: Number(retryEvidence?.retry_recovered_count ?? result?.retry_recovered_count ?? fallback?.retry_recovered_count ?? 0),
    retry_exhausted: retryExhausted,
    retry_exhausted_count: Number(retryEvidence?.retry_exhausted_count ?? result?.retry_exhausted_count ?? fallback?.retry_exhausted_count ?? 0),
    qa_hard_fail: qaHardFail,
    attempts_observed: attemptsObserved,
    max_attempts_observed: maxAttemptsObserved,
    elapsed_buckets: elapsedBuckets,
    attempts_observed_missing_reason: attemptsMissingReason,
    elapsed_buckets_missing_reason: elapsedBucketsMissingReason,
    provider_error_category: String(
      retryEvidence?.provider_error_category ??
      result?.provider_error_category ??
      fallback?.provider_error_category ??
      modelAvailability?.error_category ??
      asArray(retryEvidence?.error_categories)[0] ??
      "",
    ),
    provider_http_status: retryEvidence?.provider_http_status ??
      result?.provider_http_status ??
      fallback?.provider_http_status ??
      modelAvailability?.http_status ??
      null,
    provider_http_status_present: providerHttpStatusPresent,
    validator_hard_gate_fail_edge: hasValidatorHardFail,
    qa_hard_fail_edge: qaHardFail,
    provider_retry_exhausted_hard_fail_label_present: hasProviderRetryHardFail,
    legacy_provider_retry_exhausted_hard_fail_edge: legacyProviderRetryExhaustedHardFailEdge,
    provider_retry_observed: providerRetryObserved,
    provider_retry_exhausted_hard_fail_edge: providerRetryExhaustedHardFailEdge,
    provider_retry_classification: providerRetryClassification,
    provider_retry_telemetry_format: hasCurrentProviderRetryTelemetry ? "current" : "legacy_compact",
    provider_attempt_metadata_status: providerAttemptMetadataStatus,
    fallback_used: Boolean(retryEvidence?.fallback_used ?? fallback?.fallback_used ?? false),
    local_candidate: Boolean(retryEvidence?.local_candidate ?? fallback?.local_candidate ?? false),
    hard_gate_failures: hardGateFailures,
    raw_values_redacted: retryEvidence?.raw_values_redacted !== false,
  };
}

function validateProviderRetryTelemetry(artifact, filePath) {
  const failures = [];
  const retryEvidence = providerRetryEvidenceFromArtifact(artifact);
  const needsProviderRetryEvidence =
    retryEvidence.legacy_provider_retry_exhausted_hard_fail_edge ||
    retryEvidence.provider_retry_exhausted_hard_fail_edge ||
    retryEvidence.qa_hard_fail ||
    retryEvidence.retry_exhausted;
  if (!needsProviderRetryEvidence) {
    return failures;
  }
  assertCondition(retryEvidence.raw_values_redacted === true, `${filePath}: provider retry evidence must be redacted`, failures);
  if (retryEvidence.provider_retry_exhausted_hard_fail_edge) {
    assertCondition(retryEvidence.provider_retry_observed === true, `${filePath}: provider retry hard fail edge requires observed provider retry metadata`, failures);
    assertCondition(retryEvidence.provider_retry_classification === "provider_transport_retry_exhausted", `${filePath}: provider retry hard fail edge must be classified as provider_transport_retry_exhausted`, failures);
    assertCondition(retryEvidence.retry_exhausted === true, `${filePath}: retry_exhausted must be true for provider retry hard fail evidence`, failures);
    assertCondition(Number.isInteger(retryEvidence.retry_exhausted_count) && retryEvidence.retry_exhausted_count >= 1, `${filePath}: retry_exhausted_count missing or zero`, failures);
  } else if (retryEvidence.legacy_provider_retry_exhausted_hard_fail_edge) {
    assertCondition(
      retryEvidence.provider_retry_observed === false,
      `${filePath}: legacy provider retry label must not pass as provider retry observed without metadata`,
      failures,
    );
    assertCondition(
      retryEvidence.provider_retry_classification === "legacy_conflated_validator_hard_gate_qa_hard_fail_not_provider_transport_retry",
      `${filePath}: legacy provider retry label must be classified as conflated validator hard-gate QA hard-fail`,
      failures,
    );
    assertCondition(
      String(retryEvidence.provider_attempt_metadata_status ?? "").startsWith("not_applicable_"),
      `${filePath}: legacy provider retry label without metadata must explain provider attempt metadata as not_applicable`,
      failures,
    );
  } else if (retryEvidence.provider_retry_exhausted_hard_fail_label_present) {
    assertCondition(
      false,
      `${filePath}: current provider_retry_exhausted_hard_fail label requires observed provider retry metadata`,
      failures,
    );
  } else if (retryEvidence.qa_hard_fail) {
    assertCondition(
      String(retryEvidence.provider_attempt_metadata_status ?? "").startsWith("not_applicable_"),
      `${filePath}: qa hard-fail without provider retry must explain provider attempt metadata as not_applicable`,
      failures,
    );
  }
  assertCondition(Number.isInteger(retryEvidence.retry_recovered_count) && retryEvidence.retry_recovered_count >= 0, `${filePath}: retry_recovered_count missing`, failures);
  assertCondition(Array.isArray(retryEvidence.attempts_observed), `${filePath}: attempts_observed must be an array`, failures);
  assertCondition(Array.isArray(retryEvidence.elapsed_buckets), `${filePath}: elapsed_buckets must be an array`, failures);
  if (retryEvidence.attempts_observed.length === 0) {
    assertCondition(Boolean(retryEvidence.attempts_observed_missing_reason), `${filePath}: missing attempts_observed requires an explicit missing reason`, failures);
  }
  if (retryEvidence.elapsed_buckets.length === 0) {
    assertCondition(Boolean(retryEvidence.elapsed_buckets_missing_reason), `${filePath}: missing elapsed_buckets requires an explicit missing reason`, failures);
  }
  assertCondition(Boolean(retryEvidence.provider_error_category), `${filePath}: provider_error_category missing`, failures);
  assertCondition(retryEvidence.provider_http_status_present, `${filePath}: provider_http_status field missing`, failures);
  assertCondition(Boolean(retryEvidence.provider_retry_classification), `${filePath}: provider_retry_classification missing`, failures);
  assertCondition(Boolean(retryEvidence.provider_attempt_metadata_status), `${filePath}: provider_attempt_metadata_status missing`, failures);
  if (retryEvidence.provider_retry_exhausted_hard_fail_edge || retryEvidence.legacy_provider_retry_exhausted_hard_fail_edge) {
    assertCondition(retryEvidence.qa_hard_fail === true, `${filePath}: qa_hard_fail must be true for provider retry hard fail evidence`, failures);
  }
  if (retryEvidence.hard_gate_failures.includes("validator_hard_gate_fail")) {
    assertCondition(retryEvidence.validator_hard_gate_fail_edge === true, `${filePath}: validator_hard_gate_fail edge missing`, failures);
  }
  assertCondition(retryEvidence.fallback_used === false, `${filePath}: provider retry hard fail cannot report fallback_used=true`, failures);
  assertCondition(retryEvidence.local_candidate === false, `${filePath}: provider retry hard fail cannot report local_candidate=true`, failures);
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
  failures.push(...validateProviderRetryTelemetry(artifact, filePath));
  return failures;
}

function validateAvailabilityArtifact(artifact, filePath) {
  const failures = [];
  const results = asArray(artifact?.results);
  assertCondition(artifact?.provider === "qwen", `${filePath}: availability provider must be qwen`, failures);
  assertCondition(artifact?.raw_values_redacted === true, `${filePath}: availability artifact must be redacted`, failures);
  assertCondition(sameStringArray(artifact?.required_text_gate_models, requiredTextGateModels), `${filePath}: availability required_text_gate_models mismatch`, failures);
  assertCondition(results.length === requiredTextGateModels.length, `${filePath}: availability results must cover all required text models`, failures);
  for (const requiredModel of requiredTextGateModels) {
    assertCondition(results.some((item) => item.model === requiredModel), `${filePath}: availability missing ${requiredModel}`, failures);
  }
  for (const result of results) {
    assertCondition(requiredTextGateModels.includes(result.model), `${filePath}: availability includes non-required model ${result.model}`, failures);
    for (const field of ["model", "http_status", "error_category", "error_code", "choices_present", "callable"]) {
      assertCondition(Object.prototype.hasOwnProperty.call(result, field), `${filePath}: availability ${result.model} missing ${field}`, failures);
    }
    assertCondition(typeof result.callable === "boolean", `${filePath}: availability ${result.model} callable must be boolean`, failures);
    assertCondition(typeof result.choices_present === "boolean", `${filePath}: availability ${result.model} choices_present must be boolean`, failures);
    assertCondition(result.raw_values_redacted === true, `${filePath}: availability ${result.model} must be redacted`, failures);
    if (result.callable) {
      assertCondition(result.choices_present === true, `${filePath}: callable ${result.model} must have choices_present=true`, failures);
      assertCondition(result.error_category === "none", `${filePath}: callable ${result.model} must have error_category=none`, failures);
    } else {
      assertCondition(["model_not_found", "entitlement", "quota", "http_403", "invalid_parameter", "provider_retry_exhausted", "network_or_transport", "unknown"].includes(result.error_category), `${filePath}: unavailable ${result.model} has unsupported error_category ${result.error_category}`, failures);
    }
  }
  return failures;
}

function evidenceFromArtifact(artifact) {
  return artifact?.result?.model_output_contract_evidence ?? artifact?.model_output_contract_evidence ?? null;
}

function live3ArtifactPassed(artifact) {
  const result = artifact?.result ?? artifact;
  const evidence = evidenceFromArtifact(artifact);
  const fallback = evidence?.no_fallback_evidence ?? result?.fallback_gate_evidence ?? {};
  const storyFactFrame = result?.story_fact_frame_binding_evidence ?? {};
  const acceptedSnapshot = result?.accepted_snapshot_evidence ?? {};
  const promptBoundary = result?.prompt_text_boundary_evidence ?? {};
  const validator = result?.validator_gate_evidence ?? {};
  return result?.ok === true &&
    asArray(result?.runner_assert_failures).length === 0 &&
    asArray(evidence?.hard_gate_failures).length === 0 &&
    fallback.no_http_403 === true &&
    fallback.no_live_fallback === true &&
    fallback.fallback_used === false &&
    fallback.local_candidate === false &&
    storyFactFrame.present === true &&
    String(acceptedSnapshot.accepted_snapshot_hash ?? evidence?.accepted_snapshot_hash ?? "").trim().length > 0 &&
    promptBoundary.prompt_text_boundary_passed === true &&
    validator.rows_match === true &&
    asArray(validator.row_diffs).length === 0;
}

function live3ArtifactCaseId(artifact) {
  return artifact?.result?.caseId ?? artifact?.caseId ?? "";
}

function live3ArtifactScriptGoal(artifact) {
  const evidence = evidenceFromArtifact(artifact);
  return evidence?.script_goal ?? artifact?.result?.script_goal ?? artifact?.script_goal ?? "";
}

function validateLive3Pool(matrix, evidenceArtifacts, callableRequiredModels) {
  const failures = [];
  const requiredCases = asArray(matrix.certification_cases).map((item) => ({
    id: item.id,
    script_goal: item.script_goal,
  }));
  const summary = {
    required_live3_pool_checked: true,
    callable_required_text_models: callableRequiredModels,
    min_passing_text_models: 5,
    required_cases: requiredCases,
    model_results: [],
    passing_models: [],
    unavailable_models_not_counted: [],
  };
  assertCondition(callableRequiredModels.length >= 5, "live3 pool requires at least five callable required text models", failures);
  for (const modelName of callableRequiredModels) {
    assertCondition(requiredTextGateModels.includes(modelName), `callable model ${modelName} is not in required_text_gate_models`, failures);
    assertCondition(!aliasCompatibilityModels.includes(modelName), `alias ${modelName} cannot be callable required live3 evidence`, failures);
  }
  for (const { filePath, artifact } of evidenceArtifacts) {
    const evidence = evidenceFromArtifact(artifact);
    const modelName = evidence?.model ?? artifact?.result?.expectedModel ?? artifact?.expectedModel ?? "";
    assertCondition(requiredTextGateModels.includes(modelName), `${filePath}: evidence model ${modelName} is not a required text gate model`, failures);
    assertCondition(!aliasCompatibilityModels.includes(modelName), `${filePath}: qwen-plus alias cannot count as live3 evidence`, failures);
  }
  for (const modelName of callableRequiredModels) {
    const artifacts = evidenceArtifacts.filter(({ artifact }) => {
      const evidence = evidenceFromArtifact(artifact);
      return (evidence?.model ?? artifact?.result?.expectedModel ?? artifact?.expectedModel) === modelName;
    });
    const caseIds = artifacts.map(({ artifact }) => live3ArtifactCaseId(artifact)).filter(Boolean);
    const duplicateCaseIds = caseIds.filter((caseId, index) => caseIds.indexOf(caseId) !== index);
    for (const duplicateCaseId of Array.from(new Set(duplicateCaseIds))) {
      failures.push(`${modelName}: duplicate live3 certification case ${duplicateCaseId}`);
    }
    for (const requiredCase of requiredCases) {
      const matching = artifacts.filter(({ artifact }) => live3ArtifactCaseId(artifact) === requiredCase.id);
      if (matching.length !== 1) {
        failures.push(`${modelName}: required live3 case ${requiredCase.id} executed ${matching.length} times`);
        continue;
      }
      const observedScriptGoal = live3ArtifactScriptGoal(matching[0].artifact);
      if (observedScriptGoal !== requiredCase.script_goal) {
        failures.push(`${modelName}: case ${requiredCase.id} script_goal ${observedScriptGoal} != ${requiredCase.script_goal}`);
      }
    }
    const passed = artifacts.filter(({ artifact }) => live3ArtifactPassed(artifact)).length;
    const modelResult = {
      model: modelName,
      expected: 3,
      executed: artifacts.length,
      passed,
      failed: artifacts.length - passed,
      not_run: Math.max(0, 3 - artifacts.length),
      cases: artifacts.map(({ filePath, artifact }) => ({
        file: filePath,
        case_id: live3ArtifactCaseId(artifact),
        script_goal: live3ArtifactScriptGoal(artifact),
        passed: live3ArtifactPassed(artifact),
      })),
    };
    summary.model_results.push(modelResult);
    if (modelResult.executed === 3 && modelResult.passed === 3 && modelResult.failed === 0 && modelResult.not_run === 0) {
      summary.passing_models.push(modelName);
    } else {
      failures.push(`${modelName}: live3 requires expected=3 executed=3 passed=3 failed=0 not_run=0`);
    }
  }
  assertCondition(summary.passing_models.length >= 5, "live3 pool requires at least five passing text models", failures);
  return { failures, summary };
}

const args = parseArgs(process.argv.slice(2));
const matrix = readJson(args.matrix);
const matrixFailures = validateMatrix(matrix);
const evidenceArtifacts = args.evidence.map((filePath) => ({ filePath, artifact: readJson(filePath) }));
const evidenceFailures = evidenceArtifacts.flatMap(({ filePath, artifact }) => validateEvidenceArtifact(artifact, filePath));
const availabilityArtifacts = args.availability.map((filePath) => ({ filePath, artifact: readJson(filePath) }));
const availabilityFailures = availabilityArtifacts.flatMap(({ filePath, artifact }) => validateAvailabilityArtifact(artifact, filePath));
const live3PoolResult = args.requireLive3Pool
  ? validateLive3Pool(matrix, evidenceArtifacts, args.callableRequiredModels)
  : { failures: [], summary: null };
const failures = [...matrixFailures, ...evidenceFailures, ...availabilityFailures, ...live3PoolResult.failures];
const result = {
  ok: failures.length === 0,
  matrix: args.matrix,
  evidence_files_checked: args.evidence.length,
  availability_files_checked: args.availability.length,
  required_text_gate_models: requiredTextGateModels,
  alias_compatibility_models: aliasCompatibilityModels,
  callable_required_text_models: args.callableRequiredModels,
  current_gate_candidates: asArray(matrix.models)
    .filter((model) => model.current_gate_candidate === true)
    .map((model) => model.model),
  full16_restore_prerequisite: matrix.full16_restore_prerequisite,
  live3_pool_summary: live3PoolResult.summary,
  provider_retry_evidence_files: evidenceArtifacts.map(({ filePath, artifact }) => ({
    file: filePath,
    ...providerRetryEvidenceFromArtifact(artifact),
  })),
  failures,
  raw_values_redacted: true,
};

console.log(JSON.stringify(result, null, args.compact ? 0 : 2));
if (failures.length) {
  process.exitCode = 1;
}
