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
  "qwen-max",
  "qvq-max-2025-03-25",
  "qwen-math-turbo",
  "qwen-plus",
  "qwen3-max-preview",
  "qwen3-max-2025-09-23",
  "qwen3-max",
  "qwen3-max-2026-01-23",
  "qwen3-max-thinking",
  "qwen3.5-plus",
  "qwen-long",
];

const modelPriority = [
  "qwen-max",
  "qvq-max-2025-03-25",
  "qwen-math-turbo",
];

const input = {
  caseId: args.case ?? "case",
  sceneLabel: args.scene ?? "",
  sourceText: args.source ?? "",
  durationSeconds: Number(args.duration ?? 15),
  expectedProvider: args["expected-provider"] || "qwen",
  expectedModel: args["expected-model"] || "qwen-max",
  allowedModels: allowedQwenTextModels,
  modelPriority,
  assertBinding: args["assert-binding"] === "1",
};

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
    expectedModel: String(input.expectedModel ?? "qwen-max"),
  };
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

  const expandButton =
    findButton("扩写剧本", { enabled: true }) ||
    findButton("改写剧本", { enabled: true }) ||
    findButton("扩写故事", { enabled: true });
  if (!expandButton) {
    throw new Error("expand button not found");
  }
  const expandButtonText = textOf(expandButton);
  expandButton.scrollIntoView({ block: "center", inline: "center" });
  expandButton.click();
  await waitFor(() => currentTrace()?.command === "expand_script" && !buttons().some((button) => ["扩写中", "处理中"].some((item) => textOf(button).includes(item))), "expand_script trace", 180000);
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
    expectedProvider: options.expectedProvider,
    expectedModel: options.expectedModel,
    allowedModels,
    modelPriority,
    sceneSelection,
    durationSelection,
    sourceEditorNormalization,
    providerStatus: status,
    expandButtonText,
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
  const payload = args.compact
    ? {
        cdp_target: { id: cdp.target.id, url: cdp.target.url, title: cdp.target.title },
        result: {
          ok: value?.ok,
          stage: value?.stage,
          warningCodes: value?.warningCodes,
          bindingFailures: value?.bindingFailures,
          bindingEvidence: value?.bindingEvidence,
          caseId: value?.caseId,
          expectedProvider: value?.expectedProvider,
          expectedModel: value?.expectedModel,
          allowedModels: value?.allowedModels,
          modelPriority: value?.modelPriority,
          validator_gate_evidence: validatorGateEvidence,
          sceneSelection: value?.sceneSelection,
          durationSelection: value?.durationSelection,
          providerStatus: value?.providerStatus,
          expandButtonText: value?.expandButtonText,
          expandedText: value?.expandedText,
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
} finally {
  cdp.ws.close();
}
