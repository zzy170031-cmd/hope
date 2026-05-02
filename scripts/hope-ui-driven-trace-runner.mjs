import http from "node:http";

const args = Object.fromEntries(
  process.argv.slice(2).map((item, index, all) => {
    if (!item.startsWith("--")) {
      return [];
    }
    return [item.slice(2), all[index + 1] ?? ""];
  }).filter((item) => item.length === 2),
);

const allowedQwenTextModels = [
  "qwen-plus-2025-07-28",
  "qwen3.6-plus",
  "qwen3.6-plus-2026-04-02",
  "qvq-max-2025-03-25",
  "qwen-plus",
  "qwen-max",
  "qwen-math-turbo",
  "qwen3-max-preview",
  "qwen3-max-2025-09-23",
  "qwen3-max",
  "qwen3-max-2026-01-23",
  "qwen3-max-thinking",
  "qwen3.5-plus",
  "qwen-long",
];

const modelPriority = [
  "qwen-plus-2025-07-28",
  "qwen3.6-plus",
  "qwen3.6-plus-2026-04-02",
  "qvq-max-2025-03-25",
  "qwen-plus",
];

const allowedScriptGoals = ["rewrite", "expand"];
const requestedScriptGoal = String(args["script-goal"] || args.script_goal || args.goal || "expand")
  .trim()
  .toLowerCase();
if (!allowedScriptGoals.includes(requestedScriptGoal)) {
  throw new Error(`Unsupported --script-goal=${requestedScriptGoal}; expected rewrite or expand`);
}

const input = {
  caseId: args.case ?? "case",
  sceneLabel: args.scene ?? "",
  sourceText: args.source ?? "",
  durationSeconds: Number(args.duration ?? 15),
  scriptGoal: requestedScriptGoal,
  expectedProvider: args["expected-provider"] || "qwen",
  expectedModel: args["expected-model"] || "qwen-plus-2025-07-28",
  allowedModels: allowedQwenTextModels,
  modelPriority,
  assertBinding: args["assert-binding"] !== "0",
  assertNoProxyEnv: args["assert-no-proxy-env"] === "1" || args["assert-no-proxy-env"] === "true",
  expectQaHardFail: args["expect-qa-hard-fail"] === "1" || args["expect-qa-hard-fail"] === "true",
};

function processEnvPresent(name) {
  return typeof process.env[name] === "string" && process.env[name].trim().length > 0;
}

function collectRunnerProxyEnvEvidence() {
  const httpProxyPresent = processEnvPresent("HTTP_PROXY");
  const httpsProxyPresent = processEnvPresent("HTTPS_PROXY");
  const allProxyPresent = processEnvPresent("ALL_PROXY");
  return {
    http_proxy_present: httpProxyPresent,
    https_proxy_present: httpsProxyPresent,
    all_proxy_present: allProxyPresent,
    no_proxy_present: processEnvPresent("NO_PROXY"),
    process_env_proxy_present: httpProxyPresent || httpsProxyPresent || allProxyPresent,
    raw_values_redacted: true,
  };
}

const runnerEnvProxyEvidence = collectRunnerProxyEnvEvidence();
if (input.assertNoProxyEnv && runnerEnvProxyEvidence.process_env_proxy_present) {
  console.log(JSON.stringify({
    result: {
      ok: false,
      stage: "runner_proxy_env",
      caseId: input.caseId,
      runner_env_proxy_evidence: runnerEnvProxyEvidence,
    },
  }, null, 2));
  process.exit(1);
}

function getJson(url) {
  return new Promise((resolve, reject) => {
    const req = http.get(url, (res) => {
      let body = "";
      res.setEncoding("utf8");
      res.on("data", (chunk) => {
        body += chunk;
      });
      res.on("end", () => {
        try {
          resolve(JSON.parse(body));
        } catch (error) {
          reject(new Error(`Failed to parse JSON from ${url}: ${error.message}; body=${body.slice(0, 200)}`));
        }
      });
    });
    req.on("error", reject);
    req.setTimeout(10000, () => {
      req.destroy(new Error(`Timed out requesting ${url}`));
    });
  });
}

async function connectCdp() {
  const targets = await getJson("http://127.0.0.1:9224/json/list");
  const target = targets.find((item) => item.url === "http://tauri.localhost/#/workbench") ?? targets[0];
  if (!target?.webSocketDebuggerUrl) {
    throw new Error(`No CDP target with websocket; targets=${JSON.stringify(targets)}`);
  }

  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    ws.addEventListener("open", resolve, { once: true });
    ws.addEventListener("error", reject, { once: true });
  });

  let nextId = 1;
  const pending = new Map();
  ws.addEventListener("message", (event) => {
    const message = JSON.parse(event.data);
    if (!message.id || !pending.has(message.id)) {
      return;
    }
    const { resolve, reject, timer } = pending.get(message.id);
    pending.delete(message.id);
    clearTimeout(timer);
    if (message.error) {
      reject(new Error(`${message.error.message}: ${JSON.stringify(message.error.data ?? {})}`));
    } else {
      resolve(message.result);
    }
  });

  const send = (method, params = {}) => {
    const id = nextId++;
    ws.send(JSON.stringify({ id, method, params }));
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        if (pending.delete(id)) {
          reject(new Error(`CDP timeout: ${method}`));
        }
      }, 240000);
      pending.set(id, { resolve, reject, timer });
    });
  };

  return { target, ws, send };
}

function browserWorkflowExpression(payload) {
  return `
(async () => {
  const input = ${JSON.stringify(payload)};
  const allowedModels = Array.isArray(input.allowedModels) ? input.allowedModels.map(String) : [];
  const modelPriority = Array.isArray(input.modelPriority) ? input.modelPriority.map(String) : [];
  const options = {
    expectedProvider: String(input.expectedProvider ?? "qwen"),
    expectedModel: String(input.expectedModel ?? "qwen-plus-2025-07-28"),
  };
  const scriptGoal = input.scriptGoal === "rewrite" ? "rewrite" : "expand";
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const textOf = (element) => (element?.textContent || "").replace(/\\s+/g, " ").trim();
  const visible = (element) => Boolean(element && (element.offsetWidth || element.offsetHeight || element.getClientRects().length));
  const nativeValue = (element, value) => {
    const proto = element instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : element instanceof HTMLSelectElement ? HTMLSelectElement.prototype : HTMLInputElement.prototype;
    const setter = Object.getOwnPropertyDescriptor(proto, "value")?.set;
    setter?.call(element, value);
    element.dispatchEvent(new Event("input", { bubbles: true }));
    element.dispatchEvent(new Event("change", { bubbles: true }));
  };
  const waitFor = async (fn, label, timeout = 180000) => {
    const started = Date.now();
    let last;
    while (Date.now() - started < timeout) {
      try {
        last = await fn();
        if (last) {
          return last;
        }
      } catch (error) {
        last = error.message;
      }
      await sleep(250);
    }
    throw new Error("waitFor timeout: " + label + "; last=" + String(last ?? ""));
  };
  const buttons = () => Array.from(document.querySelectorAll("button")).filter(visible);
  const findButton = (label, options = {}) => buttons().find((button) => {
    if (options.enabled && button.disabled) {
      return false;
    }
    if (options.within && !button.closest(options.within)) {
      return false;
    }
    return textOf(button).includes(label);
  });
  const clickButton = async (label, options = {}) => {
    const button = findButton(label, { ...options, enabled: options.enabled ?? true });
    if (!button) {
      throw new Error("button not found: " + label + "; buttons=" + buttons().map(textOf).join(" | "));
    }
    button.scrollIntoView({ block: "center", inline: "center" });
    button.click();
    await sleep(options.pause ?? 350);
    return textOf(button);
  };
  const clickFirstButton = async (labels, options = {}) => {
    for (const label of labels) {
      const button = findButton(label, { ...options, enabled: options.enabled ?? true });
      if (button) {
        button.scrollIntoView({ block: "center", inline: "center" });
        button.click();
        await sleep(options.pause ?? 350);
        return textOf(button);
      }
    }
    throw new Error("button not found: " + labels.join(" or ") + "; buttons=" + buttons().map(textOf).join(" | "));
  };
  const commandPlanForScriptGoal = (goal) => goal === "rewrite"
    ? {
        script_goal: "rewrite",
        command_path: "ui.script_actions.handleExpandScript",
        selector: ".script-actions button:nth-of-type(3)",
        trace_command: "expand_script",
      }
    : {
        script_goal: "expand",
        command_path: "ui.script_actions.handleExpandStory",
        selector: ".script-actions button:nth-of-type(2)",
        trace_command: "expand_script",
      };
  const clickScriptGoalCommand = async (goal) => {
    const plan = commandPlanForScriptGoal(goal);
    const button = document.querySelector(plan.selector);
    if (!button || !visible(button)) {
      throw new Error("script_goal command control not found: " + JSON.stringify(plan) + "; buttons=" + buttons().map(textOf).join(" | "));
    }
    if (button.disabled) {
      throw new Error("script_goal command control disabled: " + JSON.stringify({ ...plan, button_text: textOf(button) }));
    }
    button.scrollIntoView({ block: "center", inline: "center" });
    const buttonText = textOf(button);
    button.click();
    await sleep(350);
    return {
      ...plan,
      selected_ui_control: "button",
      selected_ui_text: buttonText,
      selected_by: "script_goal",
    };
  };
  const selectByLabel = (label) => {
    const labels = Array.from(document.querySelectorAll("label"));
    const owner = labels.find((item) => textOf(item).includes(label) && item.querySelector("select"));
    const select = owner?.querySelector("select");
    if (!select) {
      throw new Error("select not found: " + label);
    }
    return select;
  };
  const optionByValueOrText = (select, desired) => {
    return Array.from(select.options).find((item) => item.value === desired || textOf(item).includes(desired));
  };
  const setSelectByLabel = (label, desired) => {
    const select = selectByLabel(label);
    const option = Array.from(select.options).find((item) => item.value === desired || textOf(item).includes(desired));
    if (!option) {
      throw new Error("option not found: " + label + " -> " + desired);
    }
    nativeValue(select, option.value);
    return { label, value: select.value, optionText: textOf(option) };
  };
  const toggleSelectAwayAndBack = async (label, desired) => {
    const select = selectByLabel(label);
    const desiredOption = optionByValueOrText(select, desired);
    if (!desiredOption) {
      throw new Error("option not found: " + label + " -> " + desired);
    }
    const options = Array.from(select.options);
    const alternate =
      options.find((item) => item.value !== desiredOption.value && !textOf(item).includes("长文本")) ||
      options.find((item) => item.value !== desiredOption.value);
    if (!alternate) {
      throw new Error("alternate option not found: " + label);
    }
    nativeValue(select, alternate.value);
    await sleep(350);
    nativeValue(select, desiredOption.value);
    await sleep(350);
    return {
      label,
      desired: desiredOption.value,
      desiredText: textOf(desiredOption),
      alternate: alternate.value,
      alternateText: textOf(alternate),
    };
  };
  const normalizeSourceEditorState = async (sceneLabel, durationValue) => {
    if (findButton("放大编辑", { enabled: true })) {
      return { action: "already_editable" };
    }
    if (!findButton("查看确认稿", { enabled: true })) {
      return { action: "no_confirmation_snapshot_button" };
    }

    const attempts = [
      ["单镜头时长", durationValue],
      ["场景类型", sceneLabel],
    ];
    const errors = [];
    for (const [label, desired] of attempts) {
      try {
        const toggle = await toggleSelectAwayAndBack(label, desired);
        await waitFor(() => findButton("放大编辑", { enabled: true }), "source editor restored to editable", 30000);
        return { action: "ui_select_toggle", toggle };
      } catch (error) {
        errors.push(error.message);
      }
    }
    throw new Error("source editor normalization failed: " + errors.join(" | "));
  };
  const readTraceAttr = () => {
    const raw = document.getElementById("hope-qa-trace")?.getAttribute("data-hope-qa-trace") || "";
    try {
      return raw ? JSON.parse(raw) : null;
    } catch {
      return { parse_error: true, raw: raw.slice(0, 200) };
    }
  };
  const currentTrace = () => window.__hopeQaTrace ? JSON.parse(JSON.stringify(window.__hopeQaTrace)) : null;
  const providerStatus = async () => {
    const invoke = window.__HOPE_DESKTOP_BRIDGE__?.invoke || window.__TAURI__?.core?.invoke || window.__TAURI__?.invoke;
    if (typeof invoke !== "function") {
      throw new Error("desktop invoke missing");
    }
    const raw = await invoke("get_text_model_provider_status");
    return {
      provider: String(raw?.provider ?? "qwen"),
      model: String(raw?.model ?? "unknown"),
      enabled: Boolean(raw?.enabled),
      base_url_present: Boolean(raw?.base_url_present ?? raw?.baseUrlPresent),
      api_key_present: Boolean(raw?.api_key_present ?? raw?.apiKeyPresent),
      live_ready: Boolean(raw?.live_ready ?? raw?.liveReady),
      status: String(raw?.status ?? "unknown"),
      storage: "session-only",
    };
  };
  const normalizeRows = () => {
    const rows = Array.from(document.querySelectorAll(".storyboard-table tbody tr"))
      .filter((row) => !row.querySelector(".empty-table-cell"));
    return rows.map((row) => Array.from(row.querySelectorAll("td")).map((cell) => textOf(cell)));
  };
  const layoutEvidence = () => {
    const table = document.querySelector(".storyboard-table");
    const workbench = document.querySelector(".reference-workbench");
    const rect = (element) => {
      const box = element?.getBoundingClientRect();
      return box ? { x: Math.round(box.x), y: Math.round(box.y), width: Math.round(box.width), height: Math.round(box.height) } : null;
    };
    return {
      inner_width: window.innerWidth,
      inner_height: window.innerHeight,
      device_pixel_ratio: window.devicePixelRatio,
      active_element: document.activeElement?.tagName ?? "",
      workbench_rect: rect(workbench),
      table_rect: rect(table),
      scroll_y: Math.round(window.scrollY),
    };
  };

  if (options.expectedProvider === "qwen" && !allowedModels.includes(options.expectedModel)) {
    return {
      ok: false,
      stage: "expected_model_not_allowed",
      scriptGoal,
      expectedProvider: options.expectedProvider,
      expectedModel: options.expectedModel,
      allowedModels,
      modelPriority,
      layout: layoutEvidence(),
    };
  }

  window.confirm = () => true;
  await waitFor(() => document.readyState === "complete" || document.querySelector(".reference-workbench"), "app shell ready", 30000);
  const status = await providerStatus();
  if (!(status.provider === options.expectedProvider && status.model === options.expectedModel && status.enabled && status.base_url_present && status.api_key_present && status.live_ready && status.status === "enabled")) {
    return {
      ok: false,
      stage: "provider_status",
      providerStatus: status,
      scriptGoal,
      expectedProvider: options.expectedProvider,
      expectedModel: options.expectedModel,
      allowedModels,
      modelPriority,
      layout: layoutEvidence(),
    };
  }

  const sceneSelection = setSelectByLabel("场景类型", input.sceneLabel);
  const durationSelection = setSelectByLabel("单镜头时长", String(input.durationSeconds));
  const sourceEditorNormalization = await normalizeSourceEditorState(input.sceneLabel, String(input.durationSeconds));

  await clickButton("放大编辑");
  const inputTextarea = await waitFor(() => document.querySelector(".text-dialog textarea"), "story material textarea", 30000);
  nativeValue(inputTextarea, input.sourceText);
  await clickButton("保存文本", { within: ".text-dialog" });
  await waitFor(() => !document.querySelector(".text-dialog"), "story material dialog closed", 30000);

  const selectedScriptCommand = await clickScriptGoalCommand(scriptGoal);
  await waitFor(
    () => currentTrace()?.command === selectedScriptCommand.trace_command &&
      !buttons().some((button) => ["扩写中", "处理中"].some((item) => textOf(button).includes(item))),
    scriptGoal + " " + selectedScriptCommand.trace_command + " trace",
    180000,
  );
  const expandTrace = currentTrace();
  const expandTraceAttr = readTraceAttr();

  const expandedDialogEntryText = await clickFirstButton(["查看确认稿", "放大编辑"], { within: ".text-control__actions" });
  const expandedTextarea = await waitFor(() => document.querySelector(".text-dialog textarea"), "expanded story textarea", 30000);
  const expandedText = expandedTextarea.value;
  await clickButton("关闭", { within: ".text-dialog" });
  await waitFor(() => !document.querySelector(".text-dialog"), "expanded text dialog closed", 30000);

  await clickButton("确定使用", { within: ".text-control__actions" });
  await waitFor(() => findButton("新建镜头任务", { enabled: true }), "new task enabled", 30000);
  await clickButton("新建镜头任务");
  await waitFor(() => document.querySelector(".task-draft-dialog"), "task draft dialog", 30000);
  await clickButton("加入任务队列", { within: ".task-draft-actions__buttons" });
  await waitFor(() => !document.querySelector(".task-draft-dialog"), "task draft dialog closed", 30000);
  await waitFor(() => findButton("开始生成", { enabled: true }), "generate enabled", 30000);
  await clickButton("开始生成");
  await waitFor(() => {
    const trace = currentTrace();
    const generateDone = !buttons().some((button) => textOf(button).includes("生成中"));
    if (!trace || trace.command !== "generate_storyboard" || !generateDone) {
      return false;
    }
    return trace.status === "Blocked" || normalizeRows().length > 0;
  }, "generate_storyboard trace completion", 180000);
  const generateTrace = currentTrace();
  const generateTraceAttr = readTraceAttr();
  const tableRows = normalizeRows();
  if (generateTrace?.status === "Blocked") {
    return {
      ok: false,
      stage: "generate_storyboard_blocked",
      warningCodes: generateTrace.warning_codes ?? [],
      caseId: input.caseId,
      scriptGoal,
      selectedScriptCommand,
      expectedProvider: options.expectedProvider,
      expectedModel: options.expectedModel,
      providerStatus: status,
      expandTrace,
      generateTrace,
      generateTraceAttr,
      tableRows,
      tableRowCount: tableRows.length,
      layout: layoutEvidence(),
      visibleTextSample: textOf(document.body).slice(0, 1500),
    };
  }
  const bindingEvidence = generateTrace?.binding_evidence ?? null;
  if (input.assertBinding) {
    const bindingFailures = [];
    const validatorGatePresent = Object.prototype.hasOwnProperty.call(
      generateTrace ?? {},
      "validator_gate_passed",
    );
    if (!validatorGatePresent) {
      bindingFailures.push("validator_gate_missing");
    } else if (generateTrace.validator_gate_passed !== true) {
      const validatorFailures = Array.isArray(generateTrace?.validator_gate_failures) && generateTrace.validator_gate_failures.length
        ? generateTrace.validator_gate_failures
        : ["validator_gate_not_passed"];
      bindingFailures.push(...validatorFailures);
    }
    if (generateTrace?.rows_match !== true) {
      bindingFailures.push("rows_mismatch");
    }
    if (!bindingEvidence) {
      bindingFailures.push("binding_evidence_missing");
    } else {
      if (bindingEvidence.stale_binding_detected !== false) {
        bindingFailures.push("stale_binding_detected");
      }
      if ((bindingEvidence.missing_source_facts ?? []).length) {
        bindingFailures.push("missing_source_facts");
      }
      if ((bindingEvidence.forbidden_fact_hits ?? []).length) {
        bindingFailures.push("forbidden_fact_hits");
      }
    }
    if (bindingFailures.length) {
      return {
        ok: false,
        stage: "binding_evidence",
        bindingFailures: Array.from(new Set(bindingFailures)),
        bindingEvidence,
        caseId: input.caseId,
        scriptGoal,
        selectedScriptCommand,
        expectedProvider: options.expectedProvider,
        expectedModel: options.expectedModel,
        providerStatus: status,
        expandTrace,
        generateTrace,
        tableRows,
        tableRowCount: tableRows.length,
        layout: layoutEvidence(),
      };
    }
  }

  return {
    ok: true,
    caseId: input.caseId,
    scriptGoal,
    selectedScriptCommand,
    expectedProvider: options.expectedProvider,
    expectedModel: options.expectedModel,
    allowedModels,
    modelPriority,
    sceneSelection,
    durationSelection,
    sourceEditorNormalization,
    providerStatus: status,
    expandButtonText: selectedScriptCommand.selected_ui_text,
    expandedDialogEntryText,
    expandedText,
    expandTrace,
    expandTraceAttr,
    generateTrace,
    generateTraceAttr,
    tableRows,
    tableRowCount: tableRows.length,
    layout: layoutEvidence(),
    visibleTextSample: textOf(document.body).slice(0, 1500),
  };
})()
`;
}

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function compactBindingEvidence(value) {
  if (!value) {
    return null;
  }
  return {
    ...value,
    stale_binding_detected: value.stale_binding_detected === false ? false : Boolean(value.stale_binding_detected),
    missing_source_facts: asArray(value.missing_source_facts),
    forbidden_fact_hits: asArray(value.forbidden_fact_hits),
  };
}

function compactValidatorGateEvidence(value) {
  const trace = value?.generateTrace;
  const bindingEvidence = compactBindingEvidence(trace?.binding_evidence ?? value?.bindingEvidence);
  const validatorGatePresent = Object.prototype.hasOwnProperty.call(trace ?? {}, "validator_gate_passed");
  const validatorGateFailures = asArray(trace?.validator_gate_failures);
  const runnerBindingFailures = asArray(value?.bindingFailures);
  return {
    validator_gate_present: validatorGatePresent,
    validator_gate_passed: validatorGatePresent ? trace.validator_gate_passed === true : null,
    validator_gate_failures: validatorGateFailures.length ? validatorGateFailures : runnerBindingFailures,
    rows_match: trace?.rows_match === true,
    row_diffs: asArray(trace?.row_diffs),
    binding_evidence: bindingEvidence
      ? {
          stale_binding_detected: bindingEvidence.stale_binding_detected,
          missing_source_facts: bindingEvidence.missing_source_facts,
          forbidden_fact_hits: bindingEvidence.forbidden_fact_hits,
        }
      : null,
  };
}

function stableEvidenceHash(value) {
  const source = JSON.stringify(value ?? null);
  let hash = 2166136261;
  for (const byte of new TextEncoder().encode(source)) {
    hash ^= byte;
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

function matchAny(source, patterns) {
  return patterns.some((pattern) => pattern.test(source));
}

function compactPromptTextBoundaryEvidence(value) {
  const trace = value?.generateTrace;
  const rows = asArray(trace?.ui_rows).length ? asArray(trace.ui_rows) : asArray(trace?.response_rows);
  const promptTexts = rows.map((row) => String(row?.prompt_text ?? ""));
  const promptTextMissingRows = promptTexts
    .map((promptText, index) => (promptText.trim() ? null : index + 1))
    .filter((index) => index !== null);
  const joinedPromptText = promptTexts.join("\n");
  const checks = [
    {
      field: "prompt_text_raw_body_absent",
      label: "raw_prompt_body",
      patterns: [/raw[_\s-]?prompt[_\s-]?body/i, /task_type\s*=/i, /output_schema\s*=/i, /constraints\s*=/i],
    },
    { field: "sample_text_absent", label: "sample_text", patterns: [/sample_text/i] },
    { field: "smoke_extracts_absent", label: "smoke_extracts", patterns: [/smoke_extracts/i] },
    {
      field: "raw_kb_rows_absent",
      label: "raw_kb_rows",
      patterns: [/raw[_\s-]?kb[_\s-]?rows/i, /kb_context_summary\s*=/i, /selected_sample_ids\s*=/i, /selected_kb_rules\s*=/i],
    },
    { field: "source_register_absent", label: "source_register", patterns: [/source_register/i] },
    { field: "overlay_json_absent", label: "overlay_json", patterns: [/overlay[_\s-]?json/i] },
    { field: "qa_reference_absent", label: "qa_reference", patterns: [/qa[_\s-]?reference/i] },
    { field: "source_sample_id_absent", label: "source_sample_id", patterns: [/source[_\s-]?sample[_\s-]?id/i] },
    { field: "sample_entity_marker_absent", label: "sample_entity_marker", patterns: [/sample[_\s-]?(?:character|prop|place|world|entity)/i] },
  ];
  const promptTextForbiddenSourceHits = checks
    .filter((check) => matchAny(joinedPromptText, check.patterns))
    .map((check) => check.label);
  const absentChecks = Object.fromEntries(checks.map((check) => [check.field, !promptTextForbiddenSourceHits.includes(check.label)]));
  const promptTextMissing = rows.length === 0 || promptTextMissingRows.length > 0;
  return {
    prompt_text_hash: stableEvidenceHash(promptTexts),
    prompt_text_row_count: rows.length,
    prompt_text_nonempty_row_count: promptTexts.length - promptTextMissingRows.length,
    prompt_text_present: !promptTextMissing,
    prompt_text_missing: promptTextMissing,
    prompt_text_missing_rows: promptTextMissingRows,
    prompt_text_source_policy: "accepted_facts_story_fact_frame_scene_duration_rule_tags_only_marker_check",
    prompt_text_boundary_method: "marker_absence_check; runtime source-lineage proof still requires live trace review",
    prompt_text_forbidden_source_hits: promptTextForbiddenSourceHits,
    ...absentChecks,
    prompt_text_boundary_passed: !promptTextMissing && promptTextForbiddenSourceHits.length === 0,
  };
}

function compactPromptTextBoundaryFailures(evidence) {
  const failures = [];
  if (evidence.prompt_text_missing) {
    failures.push("prompt_text_missing");
  }
  for (const hit of asArray(evidence.prompt_text_forbidden_source_hits)) {
    failures.push(`prompt_text_forbidden_source:${hit}`);
  }
  return failures;
}

function compactAcceptedSnapshotEvidence(value) {
  const bindingEvidence = compactBindingEvidence(value?.generateTrace?.binding_evidence ?? value?.bindingEvidence);
  return bindingEvidence
    ? {
        current_case_id: bindingEvidence.current_case_id ?? "",
        accepted_snapshot_hash: bindingEvidence.accepted_rewrite_hash ?? "",
        accepted_rewrite_hash: bindingEvidence.accepted_rewrite_hash ?? "",
        task_script_hash: bindingEvidence.task_script_hash ?? "",
        story_fact_frame_hash: bindingEvidence.story_fact_frame_hash ?? "",
        source_text_hash: bindingEvidence.source_text_hash ?? "",
        duration_plan_hash: bindingEvidence.duration_plan_hash ?? "",
        evidence_source: "generateTrace.binding_evidence",
      }
    : {
        current_case_id: "",
        accepted_snapshot_hash: "",
        accepted_rewrite_hash: "",
        task_script_hash: "",
        story_fact_frame_hash: "",
        source_text_hash: "",
        duration_plan_hash: "",
        evidence_source: "missing",
      };
}

function compactStoryFactFrameEvidence(value) {
  const bindingEvidence = compactBindingEvidence(value?.generateTrace?.binding_evidence ?? value?.bindingEvidence);
  return bindingEvidence
    ? {
        present: true,
        current_case_id: bindingEvidence.current_case_id ?? "",
        source_text_hash: bindingEvidence.source_text_hash ?? "",
        story_fact_frame_hash: bindingEvidence.story_fact_frame_hash ?? "",
        source_profile: bindingEvidence.source_profile ?? "",
        scene_type: bindingEvidence.scene_type ?? "",
        duration_seconds: bindingEvidence.duration_seconds ?? null,
        duration_plan_hash: bindingEvidence.duration_plan_hash ?? "",
        storyboard_rows_hash: bindingEvidence.storyboard_rows_hash ?? "",
        must_keep_facts_present: Array.isArray(bindingEvidence.must_keep_facts),
        must_keep_facts_count: asArray(bindingEvidence.must_keep_facts).length,
        missing_source_facts: asArray(bindingEvidence.missing_source_facts),
        forbidden_facts_present: Array.isArray(bindingEvidence.forbidden_facts),
        forbidden_facts_count: asArray(bindingEvidence.forbidden_facts).length,
        forbidden_fact_hits: asArray(bindingEvidence.forbidden_fact_hits),
        stale_binding_detected: bindingEvidence.stale_binding_detected === false ? false : Boolean(bindingEvidence.stale_binding_detected),
        kb_rule_pack_ids_present: Array.isArray(bindingEvidence.kb_rule_pack_ids),
        kb_rule_pack_ids: asArray(bindingEvidence.kb_rule_pack_ids),
        kb_snapshot_hash: bindingEvidence.kb_snapshot_hash ?? "",
      }
    : { present: false };
}

function compactScriptGoalEvidence(value) {
  const selected = value?.selectedScriptCommand ?? {};
  const goal = value?.scriptGoal ?? selected.script_goal ?? input.scriptGoal;
  const expectedTraceCommand = selected.trace_command ?? "expand_script";
  const observedExpandTraceCommand = value?.expandTrace?.command ?? "";
  const observedGenerateTraceCommand = value?.generateTrace?.command ?? "";
  const observedAppScriptGoal = value?.expandTrace?.script_goal ?? "";
  const observedAppCommandPath = value?.expandTrace?.command_path ?? "";
  return {
    script_goal: goal,
    selected_command_path: selected.command_path ?? "",
    selected_ui_control: selected.selected_ui_control ?? "",
    selected_ui_selector: selected.selector ?? "",
    runner_selected_command_path: selected.command_path ?? "",
    runner_selected_ui_control: selected.selected_ui_control ?? "",
    runner_selected_ui_selector: selected.selector ?? "",
    selected_ui_text: selected.selected_ui_text ?? value?.expandButtonText ?? "",
    evidence_source: "runner_selected_ui_control",
    expected_trace_command: expectedTraceCommand,
    observed_expand_trace_command: observedExpandTraceCommand,
    observed_generate_trace_command: observedGenerateTraceCommand,
    observed_app_script_goal: observedAppScriptGoal,
    observed_app_command_path: observedAppCommandPath,
    observed_app_ui_control_label: value?.expandTrace?.ui_control_label ?? "",
    observed_trace_matches_expected:
      observedExpandTraceCommand === expectedTraceCommand &&
      observedGenerateTraceCommand === "generate_storyboard" &&
      observedAppScriptGoal === goal &&
      observedAppCommandPath === (selected.command_path ?? ""),
    selected_command_path_matches_goal:
      goal === "rewrite"
        ? (selected.command_path ?? "").endsWith("handleExpandScript")
        : (selected.command_path ?? "").endsWith("handleExpandStory"),
  };
}

function compactScriptGoalFailures(evidence) {
  const failures = [];
  if (!evidence.selected_command_path_matches_goal) {
    failures.push("selected_command_path_mismatch");
  }
  if (!evidence.observed_trace_matches_expected) {
    failures.push("observed_script_goal_trace_mismatch");
  }
  if (evidence.observed_app_script_goal !== evidence.script_goal) {
    failures.push("observed_app_script_goal_mismatch");
  }
  if (evidence.observed_app_command_path !== evidence.selected_command_path) {
    failures.push("observed_app_command_path_mismatch");
  }
  return failures;
}

function compactAcceptedSnapshotFailures(evidence) {
  const requiredFields = [
    "current_case_id",
    "accepted_snapshot_hash",
    "accepted_rewrite_hash",
    "task_script_hash",
    "story_fact_frame_hash",
    "source_text_hash",
    "duration_plan_hash",
  ];
  return requiredFields
    .filter((field) => !String(evidence?.[field] ?? "").trim())
    .map((field) => `accepted_snapshot_missing:${field}`);
}

function compactStoryFactFrameFailures(evidence) {
  if (!evidence?.present) {
    return ["story_fact_frame_evidence_missing"];
  }
  const failures = [];
  for (const field of [
    "current_case_id",
    "source_text_hash",
    "story_fact_frame_hash",
    "source_profile",
    "scene_type",
    "duration_plan_hash",
    "storyboard_rows_hash",
    "kb_snapshot_hash",
  ]) {
    if (!String(evidence[field] ?? "").trim()) {
      failures.push(`story_fact_frame_missing:${field}`);
    }
  }
  if (!Number.isFinite(Number(evidence.duration_seconds)) || Number(evidence.duration_seconds) <= 0) {
    failures.push("story_fact_frame_missing:duration_seconds");
  }
  if (!evidence.must_keep_facts_present) {
    failures.push("story_fact_frame_missing:must_keep_facts");
  }
  if (!evidence.forbidden_facts_present) {
    failures.push("story_fact_frame_missing:forbidden_facts");
  }
  if (!evidence.kb_rule_pack_ids_present) {
    failures.push("story_fact_frame_missing:kb_rule_pack_ids");
  }
  if (evidence.stale_binding_detected !== false) {
    failures.push("stale_binding_detected");
  }
  if (asArray(evidence.missing_source_facts).length) {
    failures.push("missing_source_facts");
  }
  if (asArray(evidence.forbidden_fact_hits).length) {
    failures.push("forbidden_fact_hits");
  }
  return failures;
}

function compactFallbackGateFailures(evidence) {
  const failures = [];
  if (!evidence.no_http_403) {
    failures.push("http_403_detected");
  }
  if (!evidence.no_live_fallback) {
    failures.push("live_fallback_detected");
  }
  if (!evidence.qa_hard_fail && !evidence.no_validator_pseudo_success) {
    failures.push("validator_pseudo_success_risk");
  }
  if (input.expectQaHardFail && !evidence.qa_hard_fail) {
    failures.push("qa_hard_fail_evidence_missing");
  }
  return failures;
}

function compactProviderHardFailFailures(evidence) {
  return evidence.qa_hard_fail ? ["provider_retry_exhausted_hard_fail"] : [];
}

function collectLiveFallbackSignals(traces) {
  const signals = [];
  const fallbackUsedFalsePattern = /fallback[_\s-]?used\s*[:=]\s*false/gi;
  const localCandidateFalsePattern = /local[_\s-]?candidate\s*[:=]\s*false/gi;
  const fallbackUsedTruePattern = /fallback[_\s-]?used\s*[:=]\s*true/i;
  const explicitFallbackPattern =
    /(^|\b|_)(fallback[-_\s]?only|live[-_\s]?fallback|fallback[-_\s]?result|fallback[-_\s]?mode|model_config_disabled|text_model_live_call_closed|text_model_live_(?:storyboard|expand)_fallback)\b/i;
  const fallbackStatusPattern = /(^|\b)status\s*[:=]\s*fallback\b/i;
  const localCandidateFallbackPattern = /(已使用本地候选结果|本地候选结果|local[-_\s]?candidate)/i;

  const inspect = (source, text) => {
    const value = String(text ?? "");
    if (!value.trim()) {
      return;
    }
    if (fallbackUsedTruePattern.test(value)) {
      signals.push(`${source}:fallback_used=true`);
      return;
    }
    const withoutFalseMarkers = value.replace(fallbackUsedFalsePattern, "").replace(localCandidateFalsePattern, "");
    if (
      fallbackStatusPattern.test(withoutFalseMarkers) ||
      explicitFallbackPattern.test(withoutFalseMarkers) ||
      localCandidateFallbackPattern.test(withoutFalseMarkers)
    ) {
      signals.push(`${source}:${withoutFalseMarkers.slice(0, 160)}`);
    }
  };

  traces.forEach((trace, index) => {
    const prefix = `trace_${index + 1}`;
    inspect(`${prefix}.status`, trace?.status);
    inspect(`${prefix}.fallback_reason`, trace?.fallback_reason);
    inspect(`${prefix}.validator_reason`, trace?.validator_reason);
    asArray(trace?.warning_codes).forEach((code, warningIndex) => inspect(`${prefix}.warning_code_${warningIndex + 1}`, code));
    asArray(trace?.warnings).forEach((warning, warningIndex) => {
      inspect(`${prefix}.warning_${warningIndex + 1}.code`, warning?.code);
      inspect(`${prefix}.warning_${warningIndex + 1}.message`, warning?.message);
    });
  });

  return Array.from(new Set(signals));
}

function collectTraceWarningRecords(traces) {
  return traces.flatMap((trace, traceIndex) => {
    const prefix = `trace_${traceIndex + 1}`;
    return [
      { source: `${prefix}.fallback_reason`, code: "", message: trace?.fallback_reason ?? "" },
      { source: `${prefix}.validator_reason`, code: "", message: trace?.validator_reason ?? "" },
      ...asArray(trace?.warning_codes).map((code, warningIndex) => ({
        source: `${prefix}.warning_code_${warningIndex + 1}`,
        code: String(code ?? ""),
        message: "",
      })),
      ...asArray(trace?.warnings).map((warning, warningIndex) => ({
        source: `${prefix}.warning_${warningIndex + 1}`,
        code: String(warning?.code ?? ""),
        message: String(warning?.message ?? ""),
      })),
    ];
  }).filter((item) => `${item.code} ${item.message}`.trim());
}

function collectRetryTimelineEvidence(traces) {
  const records = collectTraceWarningRecords(traces);
  const retryRecovered = records.filter((item) =>
    /text_model_network_retry_recovered|retry_recovered=true|retry recovered/i.test(`${item.code} ${item.message}`),
  );
  const retryExhausted = records.filter((item) =>
    /retry_exhausted\s*[:=]\s*true|text_model_qa_no_local_fallback_blocked/i.test(`${item.code} ${item.message}`),
  );
  const qaHardFail = records.filter((item) =>
    /text_model_qa_no_local_fallback_blocked|qa_no_local_fallback\s*[:=]\s*true/i.test(`${item.code} ${item.message}`),
  );
  const fallbackUsedTrue = records.filter((item) =>
    /fallback[_\s-]?used\s*[:=]\s*true/i.test(`${item.code} ${item.message}`),
  );
  const localCandidateTrue = records.filter((item) =>
    /local[_\s-]?candidate\s*[:=]\s*true/i.test(`${item.code} ${item.message}`),
  );
  const attemptValues = records
    .map((item) => `${item.code} ${item.message}`.match(/attempts\s*[:=]\s*(\d+)/i)?.[1])
    .filter(Boolean)
    .map(Number);
  const elapsedBuckets = records
    .map((item) => `${item.code} ${item.message}`.match(/elapsed_bucket\s*[:=]\s*([A-Za-z0-9_]+)/i)?.[1])
    .filter(Boolean);
  const errorCategories = records
    .map((item) => `${item.code} ${item.message}`.match(/error_category\s*[:=]\s*([A-Za-z0-9_]+)/i)?.[1])
    .filter(Boolean);
  return {
    retry_recovered: retryRecovered.length > 0,
    retry_recovered_count: retryRecovered.length,
    retry_exhausted: retryExhausted.length > 0,
    retry_exhausted_count: retryExhausted.length,
    qa_hard_fail: qaHardFail.length > 0,
    fallback_used: fallbackUsedTrue.length > 0,
    local_candidate: localCandidateTrue.length > 0,
    attempts_observed: Array.from(new Set(attemptValues)),
    elapsed_buckets: Array.from(new Set(elapsedBuckets)),
    error_categories: Array.from(new Set(errorCategories)),
    evidence_codes: Array.from(new Set(records.map((item) => item.code).filter(Boolean))),
  };
}

function collectQaProxyEvidence(traces) {
  const records = collectTraceWarningRecords(traces).filter((item) =>
    /qa_proxy_env_evidence/i.test(`${item.code} ${item.message}`),
  );
  const processEnvProxyPresent = records.some((item) =>
    /process_env_proxy_present\s*[:=]\s*true/i.test(item.message),
  );
  return {
    app_proxy_evidence_present: records.length > 0,
    app_process_env_proxy_present: processEnvProxyPresent,
    app_process_env_proxy_cleared: records.length > 0 && !processEnvProxyPresent,
    raw_values_redacted: true,
  };
}

function uniqueStrings(values) {
  return Array.from(new Set(values.map((value) => String(value ?? "")).filter((value) => value.trim())));
}

function collectNormalizerActions(traces) {
  const records = collectTraceWarningRecords(traces).filter((item) =>
    /text_model_output_contract_normalized/i.test(`${item.code} ${item.message}`),
  );
  const actions = records.flatMap((item) => {
    const match = item.message.match(/actions\s*=\s*([^;]+)/i);
    return match ? match[1].split("|").map((value) => value.trim()) : [];
  });
  return uniqueStrings(actions);
}

function compactQualityWarnings(value, creativeFreedomEvidence) {
  const traces = [value?.expandTrace, value?.generateTrace].filter(Boolean);
  const traceWarnings = collectTraceWarningRecords(traces)
    .filter((item) => /quality[_\s-]?review|creative[_\s-]?freedom|template[_\s-]?overconstraint/i.test(`${item.code} ${item.message}`))
    .map((item) => ({
      source: item.source,
      code: item.code || "quality_review",
      category: "quality_review",
      raw_values_redacted: true,
    }));
  const structuralWarnings = [];
  if (creativeFreedomEvidence.template_overconstraint_risk.risk) {
    structuralWarnings.push({
      source: "runner.creative_freedom",
      code: "template_overconstraint_risk",
      category: "quality_review",
      raw_values_redacted: true,
    });
  }
  return [...traceWarnings, ...structuralWarnings];
}

function compactCreativeFreedomEvidence(value, promptTextBoundaryEvidence) {
  const rows = asArray(value?.generateTrace?.ui_rows).length
    ? asArray(value.generateTrace.ui_rows)
    : asArray(value?.generateTrace?.response_rows);
  const rowCount = rows.length;
  const cameraValues = uniqueStrings(rows.map((row) => row?.camera_movement));
  const actionValues = uniqueStrings(rows.map((row) => row?.character_action));
  const repeatedCameraOnly = rowCount > 1 && cameraValues.length === 1;
  const repeatedActionOnly = rowCount > 1 && actionValues.length === 1;
  const sampleLeakageRisk = asArray(promptTextBoundaryEvidence.prompt_text_forbidden_source_hits)
    .some((hit) => /sample|qa_reference|source_sample|raw_kb|source_register|overlay_json/i.test(String(hit)));
  const templateRisk = {
    risk: false,
    fixed_row_count_required: false,
    fixed_three_row_requirement_detected: false,
    fixed_sentence_style_required: false,
    repeated_camera_language_observed: repeatedCameraOnly,
    repeated_action_language_observed: repeatedActionOnly,
    note: "creative style differences are not hard-gated; only fact, boundary, fallback, schema, rows, provider, and cleanup gates are hard.",
  };
  return {
    creative_freedom_preserved: !sampleLeakageRisk,
    row_count: rowCount,
    row_count_is_contract_fixed: false,
    natural_language_variance_allowed: true,
    model_style_variance_allowed: true,
    quality_review_is_not_hard_gate: true,
    template_overconstraint_risk: templateRisk,
    sample_leakage_risk: sampleLeakageRisk,
  };
}

function compactModelOutputContractEvidence({
  value,
  validatorGateEvidence,
  acceptedSnapshotEvidence,
  storyFactFrameEvidence,
  promptTextBoundaryEvidence,
  fallbackGateEvidence,
  hardGateFailures,
  qualityWarnings,
  creativeFreedomEvidence,
}) {
  const traces = [value?.expandTrace, value?.generateTrace].filter(Boolean);
  const rows = asArray(value?.generateTrace?.ui_rows).length
    ? asArray(value.generateTrace.ui_rows)
    : asArray(value?.generateTrace?.response_rows);
  const rowContracts = rows.map((row, index) => ({
    row_index: index + 1,
    person_present: String(row?.person ?? "").trim().length > 0,
    visual_description_present: String(row?.visual_description ?? "").trim().length > 0,
    character_action_present: String(row?.character_action ?? "").trim().length > 0,
    camera_movement_present: String(row?.camera_movement ?? "").trim().length > 0,
    prompt_text_present: String(row?.prompt_text ?? "").trim().length > 0,
    prompt_text_hash: stableEvidenceHash(String(row?.prompt_text ?? "")),
    duration_seconds: Number(row?.duration_seconds ?? 0),
    source_fact_refs: [`story_fact_frame.required#row_${index + 1}`],
    forbidden_fact_refs: [`story_fact_frame.forbidden#row_${index + 1}`],
  }));
  const normalizerActions = collectNormalizerActions(traces);
  const hardGateFailureList = uniqueStrings(hardGateFailures);
  const promptBoundaryPassed = promptTextBoundaryEvidence.prompt_text_boundary_passed === true;
  const sourceBindingPassed = storyFactFrameEvidence?.present === true &&
    storyFactFrameEvidence.stale_binding_detected === false &&
    asArray(storyFactFrameEvidence.missing_source_facts).length === 0 &&
    asArray(storyFactFrameEvidence.forbidden_fact_hits).length === 0;
  const fallbackPassed = fallbackGateEvidence.no_live_fallback === true &&
    fallbackGateEvidence.fallback_used !== true &&
    fallbackGateEvidence.local_candidate !== true;
  return {
    contract_version: "desktop_model_output_contract_v1",
    contract_kind: "structural_safety_contract_not_creative_style_template",
    provider: value?.expectedProvider ?? input.expectedProvider,
    model: value?.expectedModel ?? input.expectedModel,
    script_goal: value?.scriptGoal ?? input.scriptGoal,
    scene_type: storyFactFrameEvidence?.scene_type ?? "",
    duration: storyFactFrameEvidence?.duration_seconds ?? null,
    accepted_snapshot_hash: acceptedSnapshotEvidence.accepted_snapshot_hash,
    story_fact_frame_hash: storyFactFrameEvidence?.story_fact_frame_hash ?? "",
    source_text_hash: storyFactFrameEvidence?.source_text_hash ?? "",
    source_fact_refs: {
      source: "StoryFactFrame.must_keep_facts",
      count: storyFactFrameEvidence?.must_keep_facts_count ?? 0,
      raw_values_redacted: true,
    },
    required_fact_refs: {
      source: "StoryFactFrame.must_keep_facts",
      count: storyFactFrameEvidence?.must_keep_facts_count ?? 0,
      raw_values_redacted: true,
    },
    forbidden_fact_refs: {
      source: "StoryFactFrame.forbidden_facts",
      count: storyFactFrameEvidence?.forbidden_facts_count ?? 0,
      raw_values_redacted: true,
    },
    rows: rowContracts,
    prompt_text_boundary: promptBoundaryPassed,
    no_fallback_evidence: {
      no_live_fallback: fallbackGateEvidence.no_live_fallback,
      fallback_used: fallbackGateEvidence.fallback_used,
      local_candidate: fallbackGateEvidence.local_candidate,
      no_http_403: fallbackGateEvidence.no_http_403,
    },
    validator_status: hardGateFailureList.length === 0 ? "passed" : "hard_fail",
    validator_gate_status: validatorGateEvidence.validator_gate_passed === true ? "passed" : "hard_fail",
    hard_gate_failures: hardGateFailureList,
    quality_warnings: qualityWarnings,
    normalizer_actions: normalizerActions,
    source_fact_binding_status: sourceBindingPassed ? "passed" : "hard_fail",
    prompt_boundary_status: promptBoundaryPassed ? "passed" : "hard_fail",
    fallback_status: fallbackPassed ? "passed" : "hard_fail",
    creative_freedom_preserved: creativeFreedomEvidence.creative_freedom_preserved,
    template_overconstraint_risk: creativeFreedomEvidence.template_overconstraint_risk,
    sample_leakage_risk: creativeFreedomEvidence.sample_leakage_risk,
    raw_prompt_redacted: true,
    raw_provider_response_redacted: true,
  };
}

function compactModelCertificationSummary(contractEvidence) {
  const failures = asArray(contractEvidence.hard_gate_failures);
  const hasFailure = (pattern) => failures.some((failure) => pattern.test(String(failure)));
  const qualityWarningOnly = failures.length === 0 && asArray(contractEvidence.quality_warnings).length > 0;
  return {
    provider: contractEvidence.provider,
    model: contractEvidence.model,
    output_contract_ready: failures.length === 0,
    storyboard_gate_ready: failures.length === 0,
    prompt_text_gate_ready: contractEvidence.prompt_boundary_status === "passed",
    current_gate_candidate: failures.length === 0 && contractEvidence.fallback_status === "passed",
    blocked_by_provider: hasFailure(/provider|http_403|qa_hard_fail/i),
    blocked_by_format: hasFailure(/response_invalid|schema|validator_gate_missing/i),
    blocked_by_fact_binding: hasFailure(/story_fact_frame|missing_source_facts|forbidden_fact_hits|stale_binding/i),
    blocked_by_prompt_boundary: hasFailure(/prompt_text/i),
    blocked_by_output_contract: hasFailure(/rows_mismatch|accepted_snapshot|duration|scene_type/i),
    blocked_by_fallback: hasFailure(/fallback|local_candidate/i),
    blocked_by_quality_warning_only: qualityWarningOnly,
  };
}

function compactFallbackGateEvidence(value, validatorGateEvidence, promptTextBoundaryEvidence) {
  const traces = [value?.expandTrace, value?.generateTrace].filter(Boolean);
  const warningText = traces
    .flatMap((trace) => [
      trace?.fallback_reason,
      trace?.validator_reason,
      ...asArray(trace?.warning_codes),
      ...asArray(trace?.warnings).map((warning) => `${warning?.code ?? ""} ${warning?.message ?? ""}`),
    ])
    .join("\n");
  const rowDiffs = asArray(validatorGateEvidence.row_diffs);
  const liveFallbackSignals = collectLiveFallbackSignals(traces);
  const retryTimeline = collectRetryTimelineEvidence(traces);
  return {
    no_http_403: !/(^|\D)403(\D|$)|http_403|forbidden/i.test(warningText),
    no_live_fallback: liveFallbackSignals.length === 0,
    live_fallback_signals: liveFallbackSignals,
    retry_timeline: retryTimeline,
    qa_proxy_evidence: collectQaProxyEvidence(traces),
    qa_hard_fail: retryTimeline.qa_hard_fail,
    retry_recovered: retryTimeline.retry_recovered,
    retry_exhausted: retryTimeline.retry_exhausted,
    fallback_used: retryTimeline.fallback_used,
    local_candidate: retryTimeline.local_candidate,
    no_validator_pseudo_success:
      validatorGateEvidence.validator_gate_present === true &&
      validatorGateEvidence.validator_gate_passed === true &&
      validatorGateEvidence.rows_match === true &&
      rowDiffs.length === 0 &&
      promptTextBoundaryEvidence.prompt_text_boundary_passed === true,
    rows_match: validatorGateEvidence.rows_match === true,
    row_diffs: rowDiffs,
    validator_gate_present: validatorGateEvidence.validator_gate_present,
    validator_gate_passed: validatorGateEvidence.validator_gate_passed,
    validator_gate_failures: validatorGateEvidence.validator_gate_failures,
  };
}

const cdp = await connectCdp();
try {
  await cdp.send("Runtime.enable");
  await cdp.send("Page.enable");
  const evaluation = await cdp.send("Runtime.evaluate", {
    expression: browserWorkflowExpression(input),
    awaitPromise: true,
    returnByValue: true,
    timeout: 240000,
  });
  if (evaluation.exceptionDetails) {
    throw new Error(JSON.stringify(evaluation.exceptionDetails));
  }
  const value = evaluation.result.value;
  const validatorGateEvidence = compactValidatorGateEvidence(value);
  const scriptGoalEvidence = compactScriptGoalEvidence(value);
  const acceptedSnapshotEvidence = compactAcceptedSnapshotEvidence(value);
  const storyFactFrameEvidence = compactStoryFactFrameEvidence(value);
  const promptTextBoundaryEvidence = compactPromptTextBoundaryEvidence(value);
  const promptTextBoundaryFailures = compactPromptTextBoundaryFailures(promptTextBoundaryEvidence);
  const fallbackGateEvidence = compactFallbackGateEvidence(value, validatorGateEvidence, promptTextBoundaryEvidence);
  const creativeFreedomEvidence = compactCreativeFreedomEvidence(value, promptTextBoundaryEvidence);
  const providerHardFailFailures = compactProviderHardFailFailures(fallbackGateEvidence);
  const downstreamGateFailures = fallbackGateEvidence.qa_hard_fail
    ? []
    : [
        ...compactAcceptedSnapshotFailures(acceptedSnapshotEvidence),
        ...compactStoryFactFrameFailures(storyFactFrameEvidence),
        ...promptTextBoundaryFailures,
      ];
  const assertGateFailures = [
    ...compactScriptGoalFailures(scriptGoalEvidence),
    ...downstreamGateFailures,
    ...compactFallbackGateFailures(fallbackGateEvidence),
    ...providerHardFailFailures,
  ];
  const runnerAssertFailures = input.assertBinding ? Array.from(new Set(assertGateFailures)) : [];
  const qualityWarnings = compactQualityWarnings(value, creativeFreedomEvidence);
  const modelOutputContractEvidence = compactModelOutputContractEvidence({
    value,
    validatorGateEvidence,
    acceptedSnapshotEvidence,
    storyFactFrameEvidence,
    promptTextBoundaryEvidence,
    fallbackGateEvidence,
    hardGateFailures: runnerAssertFailures,
    qualityWarnings,
    creativeFreedomEvidence,
  });
  const modelCertificationSummary = compactModelCertificationSummary(modelOutputContractEvidence);
  const resultOk = runnerAssertFailures.length ? false : value?.ok;
  const resultStage = providerHardFailFailures.length
    ? "provider_hard_fail"
    : runnerAssertFailures.length
      ? "prompt_text_boundary"
      : value?.stage;
  const payload = args.compact
    ? {
        cdp_target: { id: cdp.target.id, url: cdp.target.url, title: cdp.target.title },
        result: {
          ok: resultOk,
          stage: resultStage,
          runner_env_proxy_evidence: runnerEnvProxyEvidence,
          warningCodes: value?.warningCodes,
          bindingFailures: [...asArray(value?.bindingFailures), ...runnerAssertFailures],
          runner_assert_failures: runnerAssertFailures,
          hard_gate_failures: runnerAssertFailures,
          quality_warnings: qualityWarnings,
          bindingEvidence: value?.bindingEvidence,
          caseId: value?.caseId,
          script_goal: scriptGoalEvidence.script_goal,
          selected_command: value?.selectedScriptCommand,
          script_goal_evidence: scriptGoalEvidence,
          expectedProvider: value?.expectedProvider,
          expectedModel: value?.expectedModel,
          allowedModels: value?.allowedModels,
          modelPriority: value?.modelPriority,
          validator_gate_evidence: validatorGateEvidence,
          accepted_snapshot_evidence: acceptedSnapshotEvidence,
          story_fact_frame_binding_evidence: storyFactFrameEvidence,
          prompt_text_boundary_evidence: promptTextBoundaryEvidence,
          prompt_text_boundary_failures: promptTextBoundaryFailures,
          fallback_gate_evidence: fallbackGateEvidence,
          model_output_contract_evidence: modelOutputContractEvidence,
          model_certification_summary: modelCertificationSummary,
          normalizer_actions: modelOutputContractEvidence.normalizer_actions,
          creative_freedom_protection: creativeFreedomEvidence,
          creative_freedom_preserved: creativeFreedomEvidence.creative_freedom_preserved,
          template_overconstraint_risk: creativeFreedomEvidence.template_overconstraint_risk,
          sample_leakage_risk: creativeFreedomEvidence.sample_leakage_risk,
          source_fact_binding_status: modelOutputContractEvidence.source_fact_binding_status,
          prompt_boundary_status: modelOutputContractEvidence.prompt_boundary_status,
          fallback_status: modelOutputContractEvidence.fallback_status,
          retry_timeline_artifact: fallbackGateEvidence.retry_timeline,
          qa_proxy_evidence: fallbackGateEvidence.qa_proxy_evidence,
          no_http_403: fallbackGateEvidence.no_http_403,
          no_live_fallback: fallbackGateEvidence.no_live_fallback,
          no_validator_pseudo_success: fallbackGateEvidence.no_validator_pseudo_success,
          sceneSelection: value?.sceneSelection,
          durationSelection: value?.durationSelection,
          providerStatus: value?.providerStatus,
          expandButtonText: value?.expandButtonText,
          expandedTextPresent: typeof value?.expandedText === "string" && value.expandedText.trim().length > 0,
          expandTrace: value?.expandTrace
            ? {
                status: value.expandTrace.status,
                warning_codes: value.expandTrace.warning_codes,
                fallback_reason: value.expandTrace.fallback_reason,
                validator_reason: value.expandTrace.validator_reason,
                rows_match: value.expandTrace.rows_match,
              }
            : null,
          generateTrace: value?.generateTrace
            ? {
                status: value.generateTrace.status,
                warning_codes: value.generateTrace.warning_codes,
                fallback_reason: value.generateTrace.fallback_reason,
                validator_reason: value.generateTrace.validator_reason,
                backend_rows_hash: value.generateTrace.backend_rows_hash,
                response_rows_hash: value.generateTrace.response_rows_hash,
                ui_rows_hash: value.generateTrace.ui_rows_hash,
                rows_match: validatorGateEvidence.rows_match,
                validator_gate_present: validatorGateEvidence.validator_gate_present,
                validator_gate_passed: validatorGateEvidence.validator_gate_passed,
                validator_gate_failures: validatorGateEvidence.validator_gate_failures,
                row_diffs: validatorGateEvidence.row_diffs,
                binding_evidence: compactBindingEvidence(value.generateTrace.binding_evidence),
                ui_rows: (value.generateTrace.ui_rows ?? []).map((row) => ({
                  order: row.order,
                  person: row.person,
                  visual_description: row.visual_description,
                  character_action: row.character_action,
                  camera_movement: row.camera_movement,
                  duration_seconds: row.duration_seconds,
                })),
              }
            : null,
          tableRowCount: value?.tableRowCount,
          layout: value?.layout,
        },
      }
    : { cdp_target: { id: cdp.target.id, url: cdp.target.url, title: cdp.target.title }, result: value };
  console.log(JSON.stringify(payload, null, 2));
  if (runnerAssertFailures.length) {
    process.exitCode = 1;
  }
} finally {
  cdp.ws.close();
}
