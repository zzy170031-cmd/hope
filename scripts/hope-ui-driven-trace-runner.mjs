import http from "node:http";

const args = Object.fromEntries(
  process.argv.slice(2).map((item, index, all) => {
    if (!item.startsWith("--")) {
      return [];
    }
    return [item.slice(2), all[index + 1] ?? ""];
  }).filter((item) => item.length === 2),
);

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
const allowedQwenTextModels = [...requiredTextGateModels, ...aliasCompatibilityModels];
const modelPriority = [...requiredTextGateModels];

function modelGateRole(modelName) {
  if (requiredTextGateModels.includes(modelName)) {
    return "required_text_gate_model";
  }
  if (aliasCompatibilityModels.includes(modelName)) {
    return "alias_compatibility_reference";
  }
  return "not_allowed";
}

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
  requiredTextGateModels,
  aliasCompatibilityModels,
  expectedModelGateRole: modelGateRole(args["expected-model"] || "qwen-plus-2025-07-28"),
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
      }, 480000);
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
  const requiredTextGateModels = Array.isArray(input.requiredTextGateModels) ? input.requiredTextGateModels.map(String) : [];
  const aliasCompatibilityModels = Array.isArray(input.aliasCompatibilityModels) ? input.aliasCompatibilityModels.map(String) : [];
  const modelPriority = Array.isArray(input.modelPriority) ? input.modelPriority.map(String) : [];
  const options = {
    expectedProvider: String(input.expectedProvider ?? "qwen"),
    expectedModel: String(input.expectedModel ?? "qwen-plus-2025-07-28"),
  };
  const expectedModelGateRole = requiredTextGateModels.includes(options.expectedModel)
    ? "required_text_gate_model"
    : aliasCompatibilityModels.includes(options.expectedModel)
      ? "alias_compatibility_reference"
      : "not_allowed";
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
  const waitFor = async (fn, label, timeout = 360000) => {
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
      expectedModelGateRole,
      requiredTextGateModels,
      aliasCompatibilityModels,
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
      expectedModelGateRole,
      requiredTextGateModels,
      aliasCompatibilityModels,
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
    360000,
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
  }, "generate_storyboard trace completion", 360000);
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

function parseBlockedRuntimeWarningMessage(message) {
  const parsed = {};
  for (const part of String(message ?? "").split(/\s*;\s*/)) {
    if (!part) {
      continue;
    }
    const separator = part.indexOf("=");
    if (separator <= 0) {
      continue;
    }
    const key = part.slice(0, separator).trim();
    const value = part.slice(separator + 1).trim();
    if (key) {
      parsed[key] = value;
    }
  }
  return parsed;
}

function parseBlockedRuntimeBool(value) {
  if (value === "true") {
    return true;
  }
  if (value === "false") {
    return false;
  }
  return null;
}

function parseBlockedRuntimeList(value) {
  return String(value ?? "").trim() && String(value ?? "").trim() !== "none"
    ? String(value).split("|").map((item) => item.trim()).filter(Boolean)
    : [];
}

function compactBlockedRuntimeEvidence(trace) {
  const warnings = asArray(trace?.warnings);
  const evidence = {
    binding_context: null,
    prompt_pre_block: null,
    repair_summary: null,
    person_diagnostics: [],
    field_diagnostics: [],
  };
  for (const warning of warnings) {
    const parsed = parseBlockedRuntimeWarningMessage(warning?.message);
    switch (warning?.code) {
      case "qa_live_blocked_binding_diag":
        evidence.binding_context = {
          current_case_id: parsed.current_case_id ?? "",
          source_text_hash: parsed.source_text_hash ?? "",
          accepted_rewrite_hash: parsed.accepted_rewrite_hash ?? "",
          task_script_hash: parsed.task_script_hash ?? "",
          story_fact_frame_hash: parsed.story_fact_frame_hash ?? "",
          source_profile: parsed.source_profile ?? "",
          scene_type: parsed.scene_type ?? "",
          duration_seconds: Number(parsed.duration_seconds ?? 0),
          duration_plan_hash: parsed.duration_plan_hash ?? "",
          storyboard_rows_hash: parsed.storyboard_rows_hash ?? "",
        };
        break;
      case "qa_live_blocked_prompt_diag":
        evidence.prompt_pre_block = {
          row_count: Number(parsed.row_count ?? 0),
          prompt_text_present: parseBlockedRuntimeBool(parsed.prompt_text_present),
          prompt_text_nonempty_rows: Number(parsed.prompt_text_nonempty_rows ?? 0),
          prompt_text_hash: parsed.prompt_text_hash ?? "",
        };
        break;
      case "qa_live_blocked_repair_diag":
        evidence.repair_summary = {
          stage: parsed.stage ?? "",
          raw_failed_validator: parseBlockedRuntimeBool(parsed.raw_failed_validator),
          repair_reasons: parseBlockedRuntimeList(parsed.repair_reasons),
          pre_codes: parseBlockedRuntimeList(parsed.pre_codes),
          post_codes: parseBlockedRuntimeList(parsed.post_codes),
        };
        break;
      case "qa_live_blocked_field_diag":
        evidence.field_diagnostics.push({
          row: Number(parsed.row ?? 0),
          field: parsed.field ?? "",
          baseline_class: parsed.baseline_class ?? "",
          baseline_label: parsed.baseline_label ?? "",
          baseline_hash: parsed.baseline_hash ?? "",
          raw_patch_class: parsed.raw_patch_class ?? "",
          raw_patch_label: parsed.raw_patch_label ?? "",
          raw_patch_hash: parsed.raw_patch_hash ?? "",
          pre_repair_class: parsed.pre_repair_class ?? "",
          pre_repair_label: parsed.pre_repair_label ?? "",
          pre_repair_hash: parsed.pre_repair_hash ?? "",
          post_repair_class: parsed.post_repair_class ?? "",
          post_repair_label: parsed.post_repair_label ?? "",
          post_repair_hash: parsed.post_repair_hash ?? "",
          patch_changed: parseBlockedRuntimeBool(parsed.patch_changed),
          repair_changed: parseBlockedRuntimeBool(parsed.repair_changed),
        });
        break;
      case "qa_live_blocked_person_diag":
        evidence.person_diagnostics.push({
          row: Number(parsed.row ?? 0),
          baseline_class: parsed.baseline_class ?? "",
          baseline_label: parsed.baseline_label ?? "",
          baseline_hash: parsed.baseline_hash ?? "",
          raw_input_class: parsed.raw_input_class ?? "",
          raw_input_label: parsed.raw_input_label ?? "",
          raw_input_hash: parsed.raw_input_hash ?? "",
          raw_input_normalize_after: parsed.raw_input_normalize_after ?? "",
          raw_bound_class: parsed.raw_bound_class ?? "",
          raw_bound_label: parsed.raw_bound_label ?? "",
          raw_bound_hash: parsed.raw_bound_hash ?? "",
          pre_repair_class: parsed.pre_repair_class ?? "",
          pre_repair_label: parsed.pre_repair_label ?? "",
          pre_repair_hash: parsed.pre_repair_hash ?? "",
          post_repair_class: parsed.post_repair_class ?? "",
          post_repair_label: parsed.post_repair_label ?? "",
          post_repair_hash: parsed.post_repair_hash ?? "",
          bind_source_role_hit: parseBlockedRuntimeBool(parsed.bind_source_role_hit),
          bind_normalize_before: parsed.bind_normalize_before ?? "",
          bind_normalize_after: parsed.bind_normalize_after ?? "",
          post_grounded: parseBlockedRuntimeBool(parsed.post_grounded),
          post_grounded_normalized: parsed.post_grounded_normalized ?? "",
          post_untrusted_field: parsed.post_untrusted_field ?? "",
          post_untrusted_candidate: parsed.post_untrusted_candidate ?? "",
          post_untrusted_normalized: parsed.post_untrusted_normalized ?? "",
          post_untrusted_reason: parsed.post_untrusted_reason ?? "",
          patch_changed: parseBlockedRuntimeBool(parsed.patch_changed),
          repair_changed: parseBlockedRuntimeBool(parsed.repair_changed),
        });
        break;
      default:
        break;
    }
  }
  return evidence.binding_context || evidence.prompt_pre_block || evidence.repair_summary || evidence.person_diagnostics.length || evidence.field_diagnostics.length
    ? evidence
    : null;
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
  return evidence.provider_retry_exhausted_hard_fail_edge ? ["provider_retry_exhausted_hard_fail"] : [];
}

function compactProviderAvailabilityFailures(evidence) {
  const availability = evidence.model_availability;
  if (!availability?.unavailable) {
    return [];
  }
  const category = String(availability.error_category ?? "").trim();
  return category ? [category] : ["model_unavailable"];
}

function compactRuntimeValidatorHardGateFailures(traces) {
  const records = collectTraceWarningRecords(traces);
  return records.some((item) =>
    /text_model_live_storyboard_validator_hard_fail|text_model_validator_failed/i.test(`${item.code} ${item.message}`),
  )
    ? ["validator_hard_gate_fail"]
    : [];
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

function warningRecordText(item) {
  return `${item.code} ${item.message}`.trim();
}

function uniqueNumbers(values) {
  return Array.from(new Set(values.filter(Number.isFinite)));
}

function uniqueStrings(values) {
  return Array.from(new Set(values.map((item) => String(item ?? "").trim()).filter(Boolean)));
}

function firstNumberMatch(text, pattern) {
  const match = String(text ?? "").match(pattern);
  return match ? Number(match[1]) : null;
}

function firstStringMatch(text, pattern) {
  return String(text ?? "").match(pattern)?.[1] ?? "";
}

function retryTelemetryMissingReason(fieldName, values, context) {
  if (values.length) {
    return null;
  }
  const hasProviderRetryEdge = context.retry_recovered || context.retry_exhausted;
  if (!hasProviderRetryEdge) {
    if (context.qa_hard_fail && context.validator_hard_gate_fail_edge) {
      return `not_applicable_validator_hard_gate_qa_hard_fail_no_provider_transport_retry_${fieldName}_metadata`;
    }
    if (context.qa_hard_fail) {
      return `not_applicable_qa_hard_fail_without_provider_transport_retry_${fieldName}_metadata`;
    }
    return `not_applicable_no_retry_or_qa_hard_fail_edge:${fieldName}`;
  }
  return `provider_transport_retry_${fieldName}_metadata_not_emitted_or_not_captured`;
}

function collectRetryTimelineEvidence(traces) {
  const records = collectTraceWarningRecords(traces);
  const retryRecovered = records.filter((item) =>
    /text_model_network_retry_recovered|retry_recovered=true|retry recovered/i.test(warningRecordText(item)),
  );
  const retryExhausted = records.filter((item) =>
    /retry_exhausted\s*[:=]\s*true/i.test(warningRecordText(item)),
  );
  const qaHardFail = records.filter((item) =>
    /text_model_qa_no_local_fallback_blocked|qa_no_local_fallback\s*[:=]\s*true/i.test(warningRecordText(item)),
  );
  const fallbackUsedTrue = records.filter((item) =>
    /fallback[_\s-]?used\s*[:=]\s*true/i.test(warningRecordText(item)),
  );
  const localCandidateTrue = records.filter((item) =>
    /local[_\s-]?candidate\s*[:=]\s*true/i.test(warningRecordText(item)),
  );
  const validatorHardGate = records.filter((item) =>
    /text_model_live_storyboard_validator_hard_fail|text_model_validator_failed/i.test(warningRecordText(item)),
  );
  const attemptValues = records
    .map((item) => firstNumberMatch(warningRecordText(item), /(?:attempts|attempt_count)\s*[:=]\s*(\d+)/i))
    .filter((item) => item !== null);
  const maxAttemptValues = records
    .map((item) => firstNumberMatch(warningRecordText(item), /max_attempts\s*[:=]\s*(\d+)/i))
    .filter((item) => item !== null);
  const elapsedBuckets = records
    .map((item) => firstStringMatch(warningRecordText(item), /elapsed_bucket\s*[:=]\s*([A-Za-z0-9_-]+)/i));
  const errorCategories = records
    .map((item) => firstStringMatch(warningRecordText(item), /error_category\s*[:=]\s*([A-Za-z0-9_-]+)/i));
  const evidenceCodes = uniqueStrings(records.map((item) => item.code));
  const attemptsObserved = uniqueNumbers(attemptValues);
  const maxAttemptsObserved = uniqueNumbers(maxAttemptValues);
  const elapsedBucketsObserved = uniqueStrings(elapsedBuckets);
  const providerRetryObserved = retryRecovered.length > 0 ||
    retryExhausted.length > 0 ||
    attemptsObserved.length > 0 ||
    maxAttemptsObserved.length > 0 ||
    elapsedBucketsObserved.length > 0;
  const providerRetryClassification = providerRetryObserved
    ? retryExhausted.length > 0
      ? "provider_transport_retry_exhausted"
      : retryRecovered.length > 0
        ? "provider_transport_retry_recovered"
        : "provider_transport_retry_metadata_observed"
    : qaHardFail.length > 0 && validatorHardGate.length > 0
      ? "validator_hard_gate_qa_hard_fail_not_provider_transport_retry"
      : qaHardFail.length > 0
        ? "qa_hard_fail_without_provider_transport_retry"
        : "not_applicable_no_provider_retry";
  const retryContext = {
    retry_recovered: retryRecovered.length > 0,
    retry_exhausted: retryExhausted.length > 0,
    qa_hard_fail: qaHardFail.length > 0,
    validator_hard_gate_fail_edge: validatorHardGate.length > 0,
    evidence_codes: evidenceCodes,
  };
  const telemetryRecords = records.map((item) => {
    const text = warningRecordText(item);
    return {
      source: item.source,
      code: item.code,
      retry_recovered: /text_model_network_retry_recovered|retry_recovered=true|retry recovered/i.test(text),
      retry_exhausted: /retry_exhausted\s*[:=]\s*true/i.test(text),
      qa_hard_fail: /text_model_qa_no_local_fallback_blocked|qa_no_local_fallback\s*[:=]\s*true/i.test(text),
      attempts: firstNumberMatch(text, /(?:attempts|attempt_count)\s*[:=]\s*(\d+)/i),
      max_attempts: firstNumberMatch(text, /max_attempts\s*[:=]\s*(\d+)/i),
      elapsed_bucket: firstStringMatch(text, /elapsed_bucket\s*[:=]\s*([A-Za-z0-9_-]+)/i) || null,
      error_category: firstStringMatch(text, /error_category\s*[:=]\s*([A-Za-z0-9_-]+)/i) || null,
      raw_values_redacted: true,
    };
  }).filter((item) =>
    item.retry_recovered ||
    item.retry_exhausted ||
    item.qa_hard_fail ||
    item.attempts !== null ||
    item.elapsed_bucket !== null ||
    item.error_category !== null,
  );
  return {
    retry_recovered: retryRecovered.length > 0,
    retry_recovered_count: retryRecovered.length,
    retry_exhausted: retryExhausted.length > 0,
    retry_exhausted_count: retryExhausted.length,
    qa_hard_fail: qaHardFail.length > 0,
    fallback_used: fallbackUsedTrue.length > 0,
    local_candidate: localCandidateTrue.length > 0,
    attempts_observed: attemptsObserved,
    max_attempts_observed: maxAttemptsObserved,
    elapsed_buckets: elapsedBucketsObserved,
    attempts_observed_missing_reason: retryTelemetryMissingReason("attempts", attemptsObserved, retryContext),
    elapsed_buckets_missing_reason: retryTelemetryMissingReason("elapsed_bucket", elapsedBucketsObserved, retryContext),
    error_categories: uniqueStrings(errorCategories),
    evidence_codes: evidenceCodes,
    telemetry_records: telemetryRecords,
    validator_hard_gate_fail_edge: validatorHardGate.length > 0,
    qa_hard_fail_edge: qaHardFail.length > 0,
    provider_retry_exhausted_hard_fail_edge: retryExhausted.length > 0,
    legacy_provider_retry_exhausted_hard_fail_edge: false,
    provider_retry_observed: providerRetryObserved,
    provider_retry_classification: providerRetryClassification,
    provider_retry_telemetry_format: "current",
    provider_attempt_metadata_status: attemptsObserved.length || elapsedBucketsObserved.length
      ? "observed"
      : retryExhausted.length > 0
        ? "missing_from_provider_retry_warning"
        : qaHardFail.length > 0 && validatorHardGate.length > 0
          ? "not_applicable_validator_hard_gate_qa_hard_fail_without_provider_transport_retry"
          : "not_applicable_no_provider_retry",
    raw_values_redacted: true,
  };
}

function classifyProviderAvailability(warningText, retryTimeline) {
  const text = String(warningText ?? "");
  const lower = text.toLowerCase();
  const retryCategories = asArray(retryTimeline?.error_categories);
  const categoryText = retryCategories.join(" ").toLowerCase();
  const statusMatch = lower.match(/(?:http\s*status|status|http_status|http)[^\d]{0,16}(\d{3})/) ??
    lower.match(/\b(4\d\d|5\d\d)\b/);
  const httpStatus = statusMatch ? Number(statusMatch[1]) : null;
  const categoryAndCode = (() => {
    if (/model[_\s-]?not[_\s-]?found|model.*(?:does not exist|not found)|no such model/.test(lower)) {
      return ["model_not_found", "model_not_found", true];
    }
    if (/entitlement|permission denied|access denied|not authorized|unauthorized|no permission/.test(lower)) {
      return ["entitlement", "permission_or_entitlement", true];
    }
    if (/quota|insufficient[_\s-]?quota|billing|balance|rate[_\s-]?limit|too many requests/.test(lower)) {
      return ["quota", "quota_or_rate_limit", true];
    }
    if (httpStatus === 403 || retryCategories.includes("http_403") || /forbidden/.test(lower)) {
      return ["http_403", "http_403", true];
    }
    if (/invalid[_\s-]?(?:parameter|param)|bad request|invalid request|invalid_argument/.test(lower) || retryCategories.includes("http_400")) {
      return ["invalid_parameter", "invalid_parameter", true];
    }
    if (retryCategories.length) {
      return [retryCategories[0], retryCategories[0], false];
    }
    if (retryTimeline?.retry_exhausted) {
      return ["provider_retry_exhausted", "retry_exhausted", false];
    }
    return ["none", "none", false];
  })();
  const [errorCategory, errorCode, unavailable] = categoryAndCode;
  return {
    unavailable,
    http_status: httpStatus,
    error_category: errorCategory,
    error_code: errorCode,
    retry_error_categories: retryCategories,
    raw_values_redacted: true,
  };
}

function classifyRunnerFailureStage({
  value,
  providerHardFailFailures,
  fallbackGateFailures,
  fallbackGateEvidence,
  scriptGoalFailures,
  acceptedSnapshotFailures,
  storyFactFrameFailures,
  promptTextBoundaryFailures,
  runtimeValidatorHardGateFailures,
  validatorGateEvidence,
  runnerAssertFailures,
}) {
  if (fallbackGateFailures.includes("qa_hard_fail_evidence_missing")) {
    return input.expectedModelGateRole === "alias_compatibility_reference"
      ? "alias_compatibility_reference_not_required_gate"
      : "qa_hard_fail_evidence_missing";
  }
  const availability = fallbackGateEvidence.model_availability;
  if (availability?.unavailable) {
    return availability.error_category;
  }
  if (providerHardFailFailures.length) {
    return "provider_retry_exhausted_hard_fail";
  }
  if (scriptGoalFailures.length) {
    return "script_goal_hard_gate_fail";
  }
  if (runtimeValidatorHardGateFailures.length) {
    return "validator_hard_gate_fail";
  }
  if (
    validatorGateEvidence.validator_gate_present === true &&
    validatorGateEvidence.validator_gate_passed !== true
  ) {
    return "validator_hard_gate_fail";
  }
  if (asArray(validatorGateEvidence.validator_gate_failures).length) {
    return "validator_hard_gate_fail";
  }
  if (promptTextBoundaryFailures.length) {
    return "prompt_text_boundary";
  }
  if (acceptedSnapshotFailures.length || storyFactFrameFailures.length) {
    return "story_fact_frame_or_accepted_snapshot_hard_gate_fail";
  }
  if (fallbackGateFailures.some((failure) => /fallback|local_candidate|http_403|validator_pseudo_success/i.test(failure))) {
    return "fallback_or_provider_hard_gate_fail";
  }
  if (runnerAssertFailures.length) {
    return "contract_hard_gate_fail";
  }
  return value?.stage;
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
    model_gate_role: input.expectedModelGateRole,
    required_gate_model: input.expectedModelGateRole === "required_text_gate_model",
    alias_compatibility_reference: input.expectedModelGateRole === "alias_compatibility_reference",
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
    provider_retry_evidence: {
      retry_recovered: fallbackGateEvidence.retry_recovered,
      retry_recovered_count: fallbackGateEvidence.retry_recovered_count,
      retry_exhausted: fallbackGateEvidence.retry_exhausted,
      retry_exhausted_count: fallbackGateEvidence.retry_exhausted_count,
      qa_hard_fail: fallbackGateEvidence.qa_hard_fail,
      attempts_observed: fallbackGateEvidence.attempts_observed,
      max_attempts_observed: fallbackGateEvidence.max_attempts_observed,
      elapsed_buckets: fallbackGateEvidence.elapsed_buckets,
      attempts_observed_missing_reason: fallbackGateEvidence.attempts_observed_missing_reason,
      elapsed_buckets_missing_reason: fallbackGateEvidence.elapsed_buckets_missing_reason,
      validator_hard_gate_fail_edge: fallbackGateEvidence.validator_hard_gate_fail_edge,
      qa_hard_fail_edge: fallbackGateEvidence.qa_hard_fail_edge,
      provider_retry_exhausted_hard_fail_edge: fallbackGateEvidence.provider_retry_exhausted_hard_fail_edge,
      legacy_provider_retry_exhausted_hard_fail_edge: fallbackGateEvidence.legacy_provider_retry_exhausted_hard_fail_edge,
      provider_retry_observed: fallbackGateEvidence.provider_retry_observed,
      provider_retry_classification: fallbackGateEvidence.provider_retry_classification,
      provider_retry_telemetry_format: fallbackGateEvidence.provider_retry_telemetry_format,
      provider_attempt_metadata_status: fallbackGateEvidence.provider_attempt_metadata_status,
      provider_error_category: fallbackGateEvidence.provider_error_category,
      provider_http_status: fallbackGateEvidence.provider_http_status,
      fallback_used: fallbackGateEvidence.fallback_used,
      local_candidate: fallbackGateEvidence.local_candidate,
      raw_values_redacted: true,
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
  const sourceFactBindingBlocked = contractEvidence.source_fact_binding_status === "hard_fail" ||
    hasFailure(/story_fact_frame|missing_source_facts|forbidden_fact_hits|stale_binding/i);
  const promptBoundaryBlocked = contractEvidence.prompt_boundary_status === "hard_fail" ||
    hasFailure(/prompt_text/i);
  const fallbackBlocked = contractEvidence.fallback_status === "hard_fail" ||
    hasFailure(/fallback|local_candidate/i);
  const outputContractBlocked = contractEvidence.validator_status === "hard_fail" ||
    contractEvidence.validator_gate_status === "hard_fail" ||
    hasFailure(/rows_mismatch|accepted_snapshot|duration|scene_type|validator_hard_gate_fail/i);
  const outputContractReady = failures.length === 0 &&
    !sourceFactBindingBlocked &&
    !promptBoundaryBlocked &&
    !fallbackBlocked &&
    !outputContractBlocked;
  const qualityWarningOnly = failures.length === 0 && asArray(contractEvidence.quality_warnings).length > 0;
  return {
    provider: contractEvidence.provider,
    model: contractEvidence.model,
    model_gate_role: contractEvidence.model_gate_role,
    required_gate_model: contractEvidence.required_gate_model,
    alias_compatibility_reference: contractEvidence.alias_compatibility_reference,
    output_contract_ready: outputContractReady,
    storyboard_gate_ready: outputContractReady,
    prompt_text_gate_ready: contractEvidence.prompt_boundary_status === "passed",
    current_gate_candidate: contractEvidence.required_gate_model === true && outputContractReady,
    blocked_by_provider: hasFailure(/provider|http_403|qa_hard_fail|model_not_found|entitlement|quota|invalid_parameter/i),
    blocked_by_format: hasFailure(/response_invalid|schema|validator_gate_missing/i),
    blocked_by_fact_binding: sourceFactBindingBlocked,
    blocked_by_prompt_boundary: promptBoundaryBlocked,
    blocked_by_output_contract: outputContractBlocked,
    blocked_by_fallback: fallbackBlocked,
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
  const modelAvailability = classifyProviderAvailability(warningText, retryTimeline);
  return {
    no_http_403: !/(^|\D)403(\D|$)|http_403|forbidden/i.test(warningText),
    no_live_fallback: liveFallbackSignals.length === 0,
    live_fallback_signals: liveFallbackSignals,
    retry_timeline: retryTimeline,
    retry_recovered_count: retryTimeline.retry_recovered_count,
    retry_exhausted_count: retryTimeline.retry_exhausted_count,
    attempts_observed: retryTimeline.attempts_observed,
    max_attempts_observed: retryTimeline.max_attempts_observed,
    elapsed_buckets: retryTimeline.elapsed_buckets,
    attempts_observed_missing_reason: retryTimeline.attempts_observed_missing_reason,
    elapsed_buckets_missing_reason: retryTimeline.elapsed_buckets_missing_reason,
    validator_hard_gate_fail_edge: retryTimeline.validator_hard_gate_fail_edge,
    qa_hard_fail_edge: retryTimeline.qa_hard_fail_edge,
    provider_retry_exhausted_hard_fail_edge: retryTimeline.provider_retry_exhausted_hard_fail_edge,
    legacy_provider_retry_exhausted_hard_fail_edge: retryTimeline.legacy_provider_retry_exhausted_hard_fail_edge,
    provider_retry_observed: retryTimeline.provider_retry_observed,
    provider_retry_classification: retryTimeline.provider_retry_classification,
    provider_retry_telemetry_format: retryTimeline.provider_retry_telemetry_format,
    provider_attempt_metadata_status: retryTimeline.provider_attempt_metadata_status,
    model_availability: modelAvailability,
    model_unavailable: modelAvailability.unavailable,
    provider_error_category: modelAvailability.error_category,
    provider_error_code: modelAvailability.error_code,
    provider_http_status: modelAvailability.http_status,
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
    timeout: 480000,
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
  const blockedRuntimeEvidence = compactBlockedRuntimeEvidence(value?.generateTrace);
  const fallbackGateEvidence = compactFallbackGateEvidence(value, validatorGateEvidence, promptTextBoundaryEvidence);
  const creativeFreedomEvidence = compactCreativeFreedomEvidence(value, promptTextBoundaryEvidence);
  const traces = [value?.expandTrace, value?.generateTrace].filter(Boolean);
  const providerHardFailFailures = compactProviderHardFailFailures(fallbackGateEvidence);
  const providerAvailabilityFailures = compactProviderAvailabilityFailures(fallbackGateEvidence);
  const runtimeValidatorHardGateFailures = compactRuntimeValidatorHardGateFailures(traces);
  const scriptGoalFailures = compactScriptGoalFailures(scriptGoalEvidence);
  const acceptedSnapshotFailures = compactAcceptedSnapshotFailures(acceptedSnapshotEvidence);
  const storyFactFrameFailures = compactStoryFactFrameFailures(storyFactFrameEvidence);
  const fallbackGateFailures = compactFallbackGateFailures(fallbackGateEvidence);
  const downstreamGateFailures = fallbackGateEvidence.qa_hard_fail
    ? []
    : [
        ...acceptedSnapshotFailures,
        ...storyFactFrameFailures,
        ...promptTextBoundaryFailures,
      ];
  const assertGateFailures = [
    ...scriptGoalFailures,
    ...runtimeValidatorHardGateFailures,
    ...downstreamGateFailures,
    ...fallbackGateFailures,
    ...providerAvailabilityFailures,
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
  const resultStage = runnerAssertFailures.length
    ? classifyRunnerFailureStage({
        value,
        providerHardFailFailures,
        fallbackGateFailures,
        fallbackGateEvidence,
        scriptGoalFailures,
        acceptedSnapshotFailures,
        storyFactFrameFailures,
        promptTextBoundaryFailures,
        runtimeValidatorHardGateFailures,
        validatorGateEvidence,
        runnerAssertFailures,
      })
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
          expectedModelGateRole: input.expectedModelGateRole,
          requiredTextGateModels: input.requiredTextGateModels,
          aliasCompatibilityModels: input.aliasCompatibilityModels,
          allowedModels: value?.allowedModels,
          modelPriority: value?.modelPriority,
          validator_gate_evidence: validatorGateEvidence,
          accepted_snapshot_evidence: acceptedSnapshotEvidence,
          story_fact_frame_binding_evidence: storyFactFrameEvidence,
          prompt_text_boundary_evidence: promptTextBoundaryEvidence,
          prompt_text_boundary_failures: promptTextBoundaryFailures,
          blocked_runtime_evidence: blockedRuntimeEvidence,
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
          retry_recovered_count: fallbackGateEvidence.retry_recovered_count,
          retry_exhausted: fallbackGateEvidence.retry_exhausted,
          retry_exhausted_count: fallbackGateEvidence.retry_exhausted_count,
          attempts_observed: fallbackGateEvidence.attempts_observed,
          max_attempts_observed: fallbackGateEvidence.max_attempts_observed,
          elapsed_buckets: fallbackGateEvidence.elapsed_buckets,
          attempts_observed_missing_reason: fallbackGateEvidence.attempts_observed_missing_reason,
          elapsed_buckets_missing_reason: fallbackGateEvidence.elapsed_buckets_missing_reason,
          validator_hard_gate_fail_edge: fallbackGateEvidence.validator_hard_gate_fail_edge,
          qa_hard_fail_edge: fallbackGateEvidence.qa_hard_fail_edge,
          provider_retry_exhausted_hard_fail_edge: fallbackGateEvidence.provider_retry_exhausted_hard_fail_edge,
          legacy_provider_retry_exhausted_hard_fail_edge: fallbackGateEvidence.legacy_provider_retry_exhausted_hard_fail_edge,
          provider_retry_observed: fallbackGateEvidence.provider_retry_observed,
          provider_retry_classification: fallbackGateEvidence.provider_retry_classification,
          provider_retry_telemetry_format: fallbackGateEvidence.provider_retry_telemetry_format,
          provider_attempt_metadata_status: fallbackGateEvidence.provider_attempt_metadata_status,
          model_availability: fallbackGateEvidence.model_availability,
          provider_error_category: fallbackGateEvidence.provider_error_category,
          provider_error_code: fallbackGateEvidence.provider_error_code,
          provider_http_status: fallbackGateEvidence.provider_http_status,
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
                blocked_runtime_evidence: blockedRuntimeEvidence,
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
