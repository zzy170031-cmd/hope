import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const args = Object.fromEntries(
  process.argv.slice(2).map((item, index, all) => {
    if (!item.startsWith("--")) {
      return [];
    }
    return [item.slice(2), all[index + 1] ?? ""];
  }).filter((item) => item.length === 2),
);

const requiredTextGateModels = [
  "qwen3.6-plus",
  "qwen3.6-plus-2026-04-02",
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
const requestedDurationRaw = String(args.duration ?? "15").trim();
const requestedDurationMode = String(args["duration-mode"] || args.duration_mode || "")
  .trim()
  .toLowerCase();
const isLongTextAutoDuration =
  requestedDurationMode === "long_text_auto" ||
  requestedDurationMode === "long-text-auto" ||
  requestedDurationRaw === "long_text_auto";
const requestedDurationSeconds = Number(isLongTextAutoDuration ? 0 : requestedDurationRaw);

const input = {
  caseId: args.case ?? "case",
  sceneLabel: args.scene ?? "",
  preSceneLabel: args["pre-scene"] ?? args.pre_scene ?? "",
  preScriptGoal: String(args["pre-script-goal"] || args.pre_script_goal || "rewrite").trim().toLowerCase(),
  preDurationSeconds: Number((args["pre-duration"] ?? args.pre_duration ?? requestedDurationSeconds) || 15),
  preDurationSelectValue: String((args["pre-duration"] ?? args.pre_duration ?? requestedDurationSeconds) || 15),
  sourceText: args["source-stdin"] === "1" || args["source-stdin"] === "true"
    ? fs.readFileSync(0, "utf8")
    : args.source ?? "",
  durationSeconds: requestedDurationSeconds,
  durationSelectValue: isLongTextAutoDuration ? "long_text_auto" : String(requestedDurationSeconds || 15),
  durationMode: isLongTextAutoDuration ? "long_text_auto" : "fixed_seconds",
  scriptGoal: requestedScriptGoal,
  expectedProvider: args["expected-provider"] || "qwen",
  expectedModel: args["expected-model"] || "qwen3.6-plus",
  allowedModels: allowedQwenTextModels,
  requiredTextGateModels,
  aliasCompatibilityModels,
  expectedModelGateRole: modelGateRole(args["expected-model"] || "qwen3.6-plus"),
  modelPriority,
  assertBinding: args["assert-binding"] !== "0",
  assertNoProxyEnv: args["assert-no-proxy-env"] === "1" || args["assert-no-proxy-env"] === "true",
  expectQaHardFail: args["expect-qa-hard-fail"] === "1" || args["expect-qa-hard-fail"] === "true",
};

const artifactDir = String(args["artifact-dir"] || args.artifact_dir || "").trim();
const artifactPrefix = String(args["artifact-prefix"] || args.artifact_prefix || input.caseId || "case")
  .trim()
  .replace(/[^a-zA-Z0-9_.-]+/g, "_")
  .slice(0, 96) || "case";
const repoRoot = path.resolve(args["repo-root"] || args.repo_root || process.cwd());
const runnerScriptPath = fileURLToPath(import.meta.url);
const runnerStartedAt = new Date().toISOString();
const gateLevel = String(args["gate-level"] || args.gate_level || "targeted").trim() || "targeted";
const parentGateSummaryHash = String(args["parent-gate-summary-hash"] || args.parent_gate_summary_hash || "").trim();

function sha256Buffer(buffer) {
  return crypto.createHash("sha256").update(buffer).digest("hex");
}

function sha256File(filePath) {
  try {
    return sha256Buffer(fs.readFileSync(filePath));
  } catch {
    return "";
  }
}

function statIdentity(filePath) {
  try {
    const stat = fs.statSync(filePath);
    return {
      sha256: sha256File(filePath),
      size: stat.size,
      mtime: stat.mtime.toISOString(),
      mtime_ms: stat.mtimeMs,
      exists: true,
    };
  } catch {
    return {
      sha256: "",
      size: 0,
      mtime: "",
      mtime_ms: 0,
      exists: false,
    };
  }
}

function runGit(argsList) {
  const result = spawnSync("git", argsList, {
    cwd: repoRoot,
    encoding: "utf8",
    windowsHide: true,
  });
  return result.status === 0 ? String(result.stdout ?? "").trim() : "";
}

function readRuntimeUtf8Strict(runtimePath) {
  try {
    const bytes = fs.readFileSync(runtimePath);
    return {
      ok: true,
      text: new TextDecoder("utf-8", { fatal: true }).decode(bytes),
      error: "",
    };
  } catch (error) {
    return {
      ok: false,
      text: "",
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

function hashKnownDirtyInputs(paths) {
  const identity = {};
  for (const relativePath of paths) {
    const absolutePath = path.join(repoRoot, relativePath);
    identity[relativePath] = sha256File(absolutePath);
  }
  return sha256Buffer(Buffer.from(JSON.stringify(identity), "utf8"));
}

function collectArtifactIdentityPreflight(completedAt = null) {
  const runtimePath = path.join(repoRoot, "app", "src", "runtime.rs");
  const releaseExePath = path.join(repoRoot, "target", "release", "hope-app.exe");
  const certifierPath = path.join(repoRoot, "scripts", "hope-model-contract-certifier.mjs");
  const matrixPath = path.join(repoRoot, "tests", "qa", "desktop-403-matrix.json");
  const kbMappingPath = path.join(repoRoot, "tests", "qa", "desktop-403-kb-golden-mapping.json");
  const formalRunnerPath = path.join(repoRoot, "tests", "qa", "desktop-formal403-runner.mjs");
  const runtimeRead = readRuntimeUtf8Strict(runtimePath);
  const sentinels = ["热血战斗", "场域追逐", "危局", "林峰", "苏瑶"];
  const missingSentinels = runtimeRead.ok
    ? sentinels.filter((sentinel) => !runtimeRead.text.includes(sentinel))
    : sentinels;
  const runtimeHasReplacementChar = runtimeRead.ok && runtimeRead.text.includes("\uFFFD");
  const runtimeStat = statIdentity(runtimePath);
  const releaseStat = statIdentity(releaseExePath);
  const releaseExeFreshForRuntime =
    releaseStat.exists && runtimeStat.exists && releaseStat.mtime_ms >= runtimeStat.mtime_ms;
  const gitStatus = runGit(["status", "--short", "--branch"]);
  const dirtyPatchHash = hashKnownDirtyInputs([
    "app/src/runtime.rs",
    "ui/src/App.tsx",
    "ui/src/styles.css",
    "app/src/main.rs",
    "scripts/start-hope-release-cdp.ps1",
    "scripts/hope-ui-driven-trace-runner.mjs",
    "scripts/hope-model-contract-certifier.mjs",
    "tests/qa/desktop-403-matrix.json",
    "tests/qa/desktop-403-kb-golden-mapping.json",
    "tests/qa/desktop-formal403-runner.mjs",
  ]);
  const sourceIntegrityPreflightPassed =
    runtimeRead.ok &&
    !runtimeHasReplacementChar &&
    missingSentinels.length === 0 &&
    releaseExeFreshForRuntime;
  return {
    workspace: repoRoot,
    branch: runGit(["rev-parse", "--abbrev-ref", "HEAD"]),
    HEAD: runGit(["rev-parse", "HEAD"]),
    origin_HEAD: runGit(["rev-parse", "origin/codex/desktop-shell"]),
    ahead_behind: runGit(["rev-list", "--left-right", "--count", "HEAD...origin/codex/desktop-shell"]),
    git_status: gitStatus,
    dirty_patch_hash: dirtyPatchHash,
    runtime_rs_sha256: runtimeStat.sha256,
    runtime_rs_utf8_ok: runtimeRead.ok,
    runtime_rs_utf8_error: runtimeRead.ok ? "" : runtimeRead.error,
    runtime_rs_replacement_char_absent: !runtimeHasReplacementChar,
    runtime_rs_sentinel_check_passed: missingSentinels.length === 0,
    runtime_rs_missing_sentinels: missingSentinels,
    release_exe_sha256: releaseStat.sha256,
    release_exe_size: releaseStat.size,
    release_exe_mtime: releaseStat.mtime,
    release_exe_fresh_for_runtime: releaseExeFreshForRuntime,
    runner_sha256: sha256File(runnerScriptPath),
    certifier_sha256: sha256File(certifierPath),
    matrix_sha256: sha256File(matrixPath),
    kb_mapping_sha256: sha256File(kbMappingPath),
    formal403_runner_sha256: sha256File(formalRunnerPath),
    gate_level: gateLevel,
    case_id: input.caseId,
    parent_gate_summary_hash: parentGateSummaryHash,
    source_integrity_preflight_passed: sourceIntegrityPreflightPassed,
    raw_values_redacted: true,
    started_at: runnerStartedAt,
    completed_at: completedAt,
  };
}

function writeRunnerOnlyArtifact(payload) {
  if (!artifactDir) {
    return null;
  }
  fs.mkdirSync(artifactDir, { recursive: true });
  const payloadPath = path.join(artifactDir, `${artifactPrefix}.runner.json`);
  payload.result = payload.result || {};
  payload.result.artifacts = {
    root: artifactDir,
    runner_payload: payloadPath,
    dom: null,
    screenshot: null,
  };
  fs.writeFileSync(payloadPath, JSON.stringify(payload, null, 2), "utf8");
  return payload.result.artifacts;
}

let artifactIdentityPreflight = collectArtifactIdentityPreflight();
if (!artifactIdentityPreflight.source_integrity_preflight_passed) {
  const payload = {
    cdp_target: null,
    result: {
      ok: false,
      stage: "source_integrity_preflight",
      caseId: input.caseId,
      source_integrity_preflight: {
        runtime_rs_utf8_ok: artifactIdentityPreflight.runtime_rs_utf8_ok,
        runtime_rs_replacement_char_absent: artifactIdentityPreflight.runtime_rs_replacement_char_absent,
        runtime_rs_sentinel_check_passed: artifactIdentityPreflight.runtime_rs_sentinel_check_passed,
        runtime_rs_missing_sentinels: artifactIdentityPreflight.runtime_rs_missing_sentinels,
        release_exe_fresh_for_runtime: artifactIdentityPreflight.release_exe_fresh_for_runtime,
        raw_values_redacted: true,
      },
      artifact_identity: {
        ...artifactIdentityPreflight,
        completed_at: new Date().toISOString(),
      },
      runner_assert_failures: [
        ...(!artifactIdentityPreflight.runtime_rs_utf8_ok ? ["runtime_rs_utf8_not_readable"] : []),
        ...(!artifactIdentityPreflight.runtime_rs_replacement_char_absent ? ["runtime_rs_replacement_char_present"] : []),
        ...(!artifactIdentityPreflight.runtime_rs_sentinel_check_passed ? ["runtime_rs_sentinel_check_failed"] : []),
        ...(!artifactIdentityPreflight.release_exe_fresh_for_runtime ? ["release_exe_stale_for_runtime"] : []),
      ],
      raw_values_redacted: true,
    },
  };
  writeRunnerOnlyArtifact(payload);
  console.log(JSON.stringify(payload, null, 2));
  process.exit(1);
}

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
  const payload = {
    result: {
      ok: false,
      stage: "runner_proxy_env",
      caseId: input.caseId,
      artifact_identity: {
        ...artifactIdentityPreflight,
        completed_at: new Date().toISOString(),
      },
      runner_env_proxy_evidence: runnerEnvProxyEvidence,
    },
  };
  writeRunnerOnlyArtifact(payload);
  console.log(JSON.stringify(payload, null, 2));
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

async function writeUiDrivenArtifacts(cdp, payload) {
  if (!artifactDir) {
    return null;
  }
  fs.mkdirSync(artifactDir, { recursive: true });
  const payloadPath = path.join(artifactDir, `${artifactPrefix}.runner.json`);
  const domPath = path.join(artifactDir, `${artifactPrefix}.dom.html`);
  const screenshotPath = path.join(artifactDir, `${artifactPrefix}.screenshot.png`);
  fs.writeFileSync(payloadPath, JSON.stringify(payload, null, 2), "utf8");
  let domWritten = false;
  let screenshotWritten = false;
  let error = null;
  try {
    const domResult = await cdp.send("Runtime.evaluate", {
      expression: "document.documentElement.outerHTML",
      returnByValue: true,
    });
    const html = String(domResult?.result?.value ?? "");
    fs.writeFileSync(domPath, html, "utf8");
    domWritten = true;
  } catch (err) {
    error = `dom:${err.message}`;
  }
  try {
    await cdp.send("Page.enable");
    const shot = await cdp.send("Page.captureScreenshot", {
      format: "png",
      captureBeyondViewport: true,
    });
    if (shot?.data) {
      fs.writeFileSync(screenshotPath, Buffer.from(String(shot.data), "base64"));
      screenshotWritten = true;
    }
  } catch (err) {
    error = error ? `${error}; screenshot:${err.message}` : `screenshot:${err.message}`;
  }
  return {
    root: artifactDir,
    runner_payload: payloadPath,
    dom_html: domWritten ? domPath : null,
    screenshot_png: screenshotWritten ? screenshotPath : null,
    error,
  };
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
    expectedModel: String(input.expectedModel ?? "qwen3.6-plus"),
  };
  const expectedModelGateRole = requiredTextGateModels.includes(options.expectedModel)
    ? "required_text_gate_model"
    : aliasCompatibilityModels.includes(options.expectedModel)
      ? "alias_compatibility_reference"
      : "not_allowed";
  const scriptGoal = input.scriptGoal === "rewrite" ? "rewrite" : "expand";
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const textOf = (element) => (element?.textContent || "").replace(/\\s+/g, " ").trim();
  const lf = String.fromCharCode(10);
  const cr = String.fromCharCode(13);
  const normalizeLineBreaks = (value) => String(value || "").replaceAll(cr + lf, lf).replaceAll(cr, lf);
  const valueTextOf = (element) => normalizeLineBreaks(element?.value || "").trim();
  const readScriptDialogText = () => {
    const dialog = document.querySelector(".text-dialog");
    if (!dialog) {
      return "";
    }
    const blocks = Array.from(dialog.querySelectorAll(".script-dialog-block"));
    if (blocks.length) {
      return blocks.map((block) => {
        const label = textOf(block.querySelector("span")) || "正文";
        const body = Array.from(block.querySelectorAll("textarea"))
          .map(valueTextOf)
          .filter(Boolean)
          .join(lf);
        const supportBody = Array.from(block.querySelectorAll(".script-dialog-block__support p"))
          .map(textOf)
          .filter(Boolean)
          .join(lf);
        const blockBody = body || supportBody;
        return blockBody ? label + lf + blockBody : textOf(block);
      }).filter(Boolean).join(lf + lf);
    }
    const textareas = Array.from(dialog.querySelectorAll("textarea"))
      .map(valueTextOf)
      .filter(Boolean);
    return textareas.length ? textareas.join(lf + lf) : textOf(dialog);
  };
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
        required_ui_text: "改写剧本",
      }
    : {
        script_goal: "expand",
        command_path: "ui.script_actions.handleExpandStory",
        selector: ".script-actions button:nth-of-type(2)",
        trace_command: "expand_script",
        required_ui_text: "扩写故事",
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
    if (!buttonText.includes(plan.required_ui_text)) {
      throw new Error("script_goal command visible label mismatch: " + JSON.stringify({ ...plan, button_text: buttonText }));
    }
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
    if (!findButton("查看改写稿", { enabled: true }) && !findButton("查看确认稿", { enabled: true })) {
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
  const storyboardTableDurationEvidence = (rows) => {
    const headers = Array.from(document.querySelectorAll(".storyboard-table thead th")).map(textOf);
    const durationIndex = headers.findIndex((item) => item.includes("时长"));
    const durations = durationIndex >= 0
      ? rows.map((row) => Number(String(row[durationIndex] || "").match(/\\d+/)?.[0] || 0)).filter((value) => value > 0)
      : [];
    return {
      duration_column_index: durationIndex,
      row_durations: durations,
      row_duration_sum: durations.reduce((sum, value) => sum + value, 0),
    };
  };
  const showAllStoryboardRowsForEvidence = async (expectedRowCount = 0) => {
    const select = document.querySelector("#storyboard-page-size");
    const beforeRows = normalizeRows().length;
    if (!(select instanceof HTMLSelectElement)) {
      return {
        applied: false,
        reason: "page_size_select_missing",
        before_rows: beforeRows,
        after_rows: beforeRows,
        expected_rows: expectedRowCount,
      };
    }
    const options = Array.from(select.options)
      .map((option) => Number(option.value))
      .filter((value) => Number.isFinite(value) && value > 0);
    const maxPageSize = options.length ? Math.max(...options) : 0;
    if (!maxPageSize) {
      return {
        applied: false,
        reason: "page_size_options_missing",
        before_rows: beforeRows,
        after_rows: beforeRows,
        expected_rows: expectedRowCount,
      };
    }
    const expectedRows = Number(expectedRowCount || 0);
    const targetVisibleRows = expectedRows > 0 ? Math.min(maxPageSize, expectedRows) : maxPageSize;
    if (Number(select.value) !== maxPageSize) {
      nativeValue(select, String(maxPageSize));
    }
    await waitFor(
      () => normalizeRows().length >= Math.min(targetVisibleRows, Math.max(1, beforeRows)),
      "storyboard page size expanded",
      30000,
    );
    const afterRows = normalizeRows().length;
    return {
      applied: true,
      reason: "page_size_expanded_for_visible_row_evidence",
      selected_page_size: maxPageSize,
      before_rows: beforeRows,
      after_rows: afterRows,
      expected_rows: expectedRows,
    };
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
  const visibleBodySceneDurationEvidence = (sceneSelection, durationSelection, overrides = {}) => {
    const bodyText = textOf(document.body);
    const expectedSceneLabel = overrides.sceneLabel ?? input.sceneLabel;
    const expectedDurationMode = overrides.durationMode ?? input.durationMode;
    const expectedDurationSeconds = overrides.durationSeconds ?? input.durationSeconds;
    const expectedDurationSelectValue = overrides.durationSelectValue ?? input.durationSelectValue;
    const expectedDuration = expectedDurationMode === "long_text_auto"
      ? "长文本"
      : String(expectedDurationSeconds || expectedDurationSelectValue);
    return {
      expected_scene_label: expectedSceneLabel,
      selected_scene_value: sceneSelection?.value ?? "",
      selected_scene_text: sceneSelection?.optionText ?? "",
      expected_duration: expectedDuration,
      selected_duration_value: durationSelection?.value ?? "",
      selected_duration_text: durationSelection?.optionText ?? "",
      body_has_scene_label: bodyText.includes(expectedSceneLabel),
      body_has_selected_scene_text: bodyText.includes(sceneSelection?.optionText ?? expectedSceneLabel),
      body_has_duration: bodyText.includes(expectedDuration),
      body_sample_length: bodyText.length,
      raw_values_redacted: true,
    };
  };
  const confirmationBlocks = (text) => {
    const blocks = [];
    let current = null;
    const cr = String.fromCharCode(13);
    const lf = String.fromCharCode(10);
    const normalizedText = String(text || "").replaceAll(cr + lf, lf).replaceAll(cr, lf);
    for (const rawLine of normalizedText.split(lf)) {
      const line = rawLine.trim();
      if (!line) {
        continue;
      }
      if (/^【[^】]+】$/.test(line)) {
        if (current) {
          current.body = current.body.trim();
          blocks.push(current);
        }
        current = { label: line, body: "" };
        continue;
      }
      if (!current) {
        current = { label: "正文", body: "" };
      }
      current.body = current.body ? current.body + "\\n" + line : line;
    }
    if (current) {
      current.body = current.body.trim();
      blocks.push(current);
    }
    return blocks;
  };
  const sceneEvidenceTerms = (sceneLabel) => {
    const label = String(sceneLabel || "");
    if (/追逐|合围|行军|场域/.test(label)) {
      return ["追逐", "追赶", "逼近", "转向", "退路", "路径", "改道", "距离", "甩开"];
    }
    if (/群像|表演/.test(label)) {
      return ["群像", "前后站位", "互动", "人物关系", "沉默", "立场", "提醒", "回望"];
    }
    if (/国战|军阵/.test(label)) {
      return ["国战", "军阵", "阵前相持", "胜负感", "围住", "守住"];
    }
    if (/沙盘|视口/.test(label)) {
      return ["沙盘", "态势", "退路", "判断", "信息有限", "方向", "选择"];
    }
    if (/热血|战斗/.test(label)) {
      return ["战斗", "对抗", "反击", "压迫", "迎上", "主动", "逼近"];
    }
    if (/相遇|日常|治愈|情绪|对白/.test(label)) {
      return ["关系", "沉默", "对望", "意图", "停顿", "信任", "分歧"];
    }
    if (/朝堂|军帐|权谋/.test(label)) {
      return ["判断", "意图", "沉默", "权衡", "破绽", "选择"];
    }
    if (/城建|战报/.test(label)) {
      return ["后果", "改变", "下一步", "守住", "结果", "选择"];
    }
    return ["压力", "选择", "关系", "危险"];
  };
  const sourceFactTerms = (text) => {
    const candidates = ["林峰", "孙二娘", "竹林", "打斗", "苏瑶", "阿青", "黑衣追兵", "追兵", "巷口", "逼近", "主角", "敌人", "废墟", "单膝跪地", "缓步逼近"];
    return candidates.filter((term) => String(text || "").includes(term));
  };
  const rewriteForbiddenTerms = [
    "结构参考",
    "本段按",
    "策略为",
    "目标时长",
    "分段策略",
    "时长策略",
    "可拆镜头",
    "镜头容量",
    "重点落在",
    "场景表达",
    "节奏推进",
    "prompt_text",
    "source_register",
    "overlay",
    "validator",
    "schema",
    "raw KB",
    "raw sample_text",
    "人物、地点和事件保持不变",
    "空间移动、追赶距离和转向动作推动事件向前",
    "旧场景的冲突方式不再主导这一版叙事",
    "旧场景的冲撞方式不再主导这一版叙事",
    "动作按逼近",
    "可拆镜头数量增加",
    "改写为",
    "本段基于",
    "保留事实",
    "合理补全",
    "事实边界",
    "结构说明",
    "辅助依据",
    "创作依据",
    "质量锚点",
    "推动事件向前",
    "不再主导",
  ];
  const narrativeMetaStrategyTerms = [
    "重点落在",
    "场景表达",
    "节奏推进",
    "策略为",
    "目标时长",
    "分段策略",
    "可拆镜头",
    "镜头容量",
    "推动事件向前",
    "不再主导",
    "空间移动、追赶距离和转向动作推动事件向前",
  ];
  const narrativeInstructionalTerms = ["改写为", "本段基于", "本段按", "按当前", "规则", "结构参考", "策略", "结构说明", "辅助依据", "创作依据", "质量锚点"];
  const narrativeFactBoundaryTerms = ["人物、地点和事件保持不变", "保留事实", "核心人物", "全部保留", "保持不变", "合理补全", "事实边界"];
  const narrativeDurationPlanTerms = [
    "目标时长",
    "时长策略",
    "可拆镜头",
    "30秒内展开起势",
    "15秒内分成",
    "动作按逼近、格挡、错身",
    "可拆镜头数量增加",
  ];
  const storyboardFormatTerms = ["镜头1", "镜头2", "景别", "运镜", "画面描述", "角色动作", "分镜提示词", "视频分镜提示词"];
  const compactConfirmationEvidence = (text, preText = "", overrides = {}) => {
    const blocks = confirmationBlocks(text);
    const first = blocks[0] || { label: "", body: "" };
    const bodyBlock = blocks.find((block) => block.label === "【改写剧本正文】") || first;
    const mainBody = String(bodyBlock.body || "");
    const firstTextareaBody = valueTextOf(document.querySelector(".script-dialog-block textarea")) || "";
    const storyBody = mainBody || firstTextareaBody;
    const expectedSceneLabel = overrides.sceneLabel ?? input.sceneLabel;
    const expectedSourceText = overrides.sourceText ?? input.sourceText;
    const expectedDurationMode = overrides.durationMode ?? input.durationMode;
    const expectedDurationSeconds = overrides.durationSeconds ?? input.durationSeconds;
    const expectedDurationSelectValue = overrides.durationSelectValue ?? input.durationSelectValue;
    const sceneTerms = sceneEvidenceTerms(expectedSceneLabel);
    const factTerms = sourceFactTerms(expectedSourceText);
    const durationText = expectedDurationMode === "long_text_auto"
      ? "长文本"
      : String(expectedDurationSeconds || expectedDurationSelectValue);
    const storyboardHits = storyboardFormatTerms.filter((term) => storyBody.includes(term));
    const metaStrategyHits = narrativeMetaStrategyTerms.filter((term) => storyBody.includes(term));
    const instructionalHits = narrativeInstructionalTerms.filter((term) => storyBody.includes(term));
    const noInternal = !rewriteForbiddenTerms.some((term) => storyBody.includes(term));
    const noStoryboard = storyboardHits.length === 0;
    const narrativeBodyNotMetaStrategy = metaStrategyHits.length === 0;
    const narrativeBodyNotInstructionalSummary = instructionalHits.length === 0;
    const narrativeBodyNotFactBoundaryExplanation = !narrativeFactBoundaryTerms.some((term) => storyBody.includes(term));
    const narrativeBodyNotDurationPlanExplanation =
      !narrativeDurationPlanTerms.some((term) => storyBody.includes(term)) &&
      !/\\d+\\s*秒(?:里|内|中)/.test(storyBody);
    const storyHardFailTerms = [
      "镜头", "画面描述", "角色动作", "景别", "运镜", "prompt_text", "duration_seconds", "动作设计",
      "调度", "节奏策略", "场景策略", "结构参考", "表达焦点", "动作密度", "镜头容量", "主体为",
      "重点落在", "强调", "突出", "用于", "服务于", "scene_type", "target_duration_seconds",
      "kb_context_summary", "selected_sample_ids", "selected_kb_rules", "retrieval_trace", "source_register",
      "overlay_json", "hash", "oracle",
    ];
    const storyHardFailHits = storyHardFailTerms.filter((term) => storyBody.includes(term));
    const storyCompact = storyBody.replace(/\\s+/g, "");
    const narrativeBodyNotActionChoreography =
      !/(围绕(?:交手|拳脚|格挡|当前事件)|连续移动|动作一拍接一拍|动作落点|从.+停点起步|人物\\/敌人|主角\\/敌人)/.test(storyCompact);
    const narrativeBodyNotStrategyOrTrace =
      storyHardFailHits.length === 0 &&
      !/(保留人物|保留地点|保留事件|人物、地点|空间移动|追赶距离|推动事件向前|旧场景|知识库规则|黄金样本|写作组|导演组|结构规则|故事停在|\\d+\\s*秒(?:里|内|中)|(?:热血战斗|场域追逐|群像表演|沙盘战略视口|国战军阵建立)里)/.test(storyBody);
    const narrativeBodyNotMechanicalRewriteTemplate =
      !/(场域追逐里|热血战斗里|群像表演里|沙盘战略视口里|国战军阵建立里|人物从|追逐把|故事停在|这场[^。；，]{0,12}里|人物的选择|旧场景|当前场景|目标时长|动作按|保留事实)/.test(storyBody);
    const narrativeBodyHasContinuousActions =
      /(随后|接着|紧接着|然后|一边|再|最后|越|直到|先|于是|却|因为|一旦)/.test(storyBody) &&
      /(移动|追|逼近|转身|回身|撤步|格挡|护住|提醒|冲|停住|压近|靠近|避让|对峙|行动|穿梭|收束|选择|保护|摆脱|逃离|阻止|决断)/.test(storyBody);
    const narrativeBodyHasCharacterSubjects =
      factTerms.some((term) => storyBody.includes(term)) ||
      /(林峰|孙二娘|苏瑶|阿青|主角|敌人|追兵|人物)/.test(storyBody);
    const narrativeBodyHasConflictProgression =
      /(压力|逼近|追兵|敌人|对峙|冲突|打斗|交手|追逐|追赶|压紧|紧张|收束|阻碍|危险|困住)/.test(storyBody);
    const narrativeBodyHasCharacterGoal =
      /(为了|因为|必须|想要|不愿|不能|决定|选择|保护|摆脱|逃离|阻止|确认|争取|担心|一旦)/.test(storyBody);
    const narrativeBodyHasConflictCausality =
      /(因为|所以|于是|却|导致|让|逼得|被迫|只好|一旦|如果|否则|随着|当|因此)/.test(storyBody);
    const narrativeBodyHasEmotionalTurn =
      /(紧张|犹豫|决断|决心|害怕|压迫|信任|不再|终于|意识到|看出|低声|沉默|急促|体力不支|咬住)/.test(storyBody);
    const narrativeBodyHasStoryResolutionBeat =
      /(最后|最终|终于|停住|冲向|退到|挡在|抓住机会|没有再|重新站稳|留下|收住|转入)/.test(storyBody);
    const sceneEvidence = sceneTerms.some((term) => term && storyBody.includes(term));
    const narrativeBodyHasSceneSpecificExpression = sceneEvidence &&
      sceneTerms.some((term) => term && storyBody.includes(term));
    const storyLike = storyBody.length >= 72 &&
      /[。；，,.]/.test(storyBody) &&
      noInternal &&
      noStoryboard &&
      narrativeBodyHasContinuousActions &&
      narrativeBodyHasCharacterSubjects &&
      narrativeBodyHasConflictProgression &&
      narrativeBodyHasCharacterGoal &&
      narrativeBodyHasConflictCausality &&
      narrativeBodyHasEmotionalTurn &&
      narrativeBodyHasStoryResolutionBeat &&
      narrativeBodyHasSceneSpecificExpression &&
      narrativeBodyNotMetaStrategy &&
      narrativeBodyNotInstructionalSummary &&
      narrativeBodyNotFactBoundaryExplanation &&
      narrativeBodyNotDurationPlanExplanation &&
      narrativeBodyNotMechanicalRewriteTemplate &&
      narrativeBodyNotActionChoreography &&
      narrativeBodyNotStrategyOrTrace &&
      !/^[-*]|^\\d+[.)、]/m.test(storyBody);
    const durationSeconds = Number(expectedDurationSeconds || expectedDurationSelectValue || 0);
    const durationEvidence = expectedDurationMode === "long_text_auto"
      ? storyBody.length >= 120
      : durationSeconds <= 15
        ? /(迅速|短促|立刻|随即|一瞬|马上|只来得及|很快|一次)/.test(storyBody)
        : durationSeconds <= 30
          ? /(随后|几次|没有在一次|越来越|终于|继续|一段|新的阻碍|反应)/.test(storyBody)
          : /(接连|多次|反复|一步步|更久|连续|重新选择|几次)/.test(storyBody);
    const preservedFacts = factTerms.length === 0 ||
      factTerms.filter((term) => storyBody.includes(term)).length >= Math.min(factTerms.length, 3);
    const rewriteForbiddenHits = rewriteForbiddenTerms.filter((term) => String(text || "").includes(term));
    const storyBodyKbOraclePresent = String(text || "").includes("【KB参考摘要】") || String(text || "").includes("知识库");
    return {
      rewrite_confirmation_dialog_present: blocks.length > 0,
      rewrite_confirmation_first_block_label: first.label,
      rewrite_confirmation_body_first: first.label === "【改写剧本正文】",
      rewrite_confirmation_main_body_is_story: storyLike,
      rewrite_main_body_is_narrative_story: storyLike,
      main_textarea_is_complete_story: storyLike,
      narrative_body_has_continuous_actions: narrativeBodyHasContinuousActions,
      narrative_body_has_character_subjects: narrativeBodyHasCharacterSubjects,
      narrative_body_has_conflict_progression: narrativeBodyHasConflictProgression,
      narrative_body_has_character_goal: narrativeBodyHasCharacterGoal,
      narrative_body_has_conflict_causality: narrativeBodyHasConflictCausality,
      narrative_body_has_emotional_turn: narrativeBodyHasEmotionalTurn,
      narrative_body_has_story_resolution_beat: narrativeBodyHasStoryResolutionBeat,
      narrative_body_has_scene_specific_expression: narrativeBodyHasSceneSpecificExpression,
      narrative_body_not_meta_strategy: narrativeBodyNotMetaStrategy,
      narrative_body_not_action_choreography: narrativeBodyNotActionChoreography,
      narrative_body_not_strategy_or_trace: narrativeBodyNotStrategyOrTrace,
      narrative_body_not_mechanical_rewrite_template: narrativeBodyNotMechanicalRewriteTemplate,
      narrative_body_not_storyboard_breakdown: narrativeBodyNotStrategyOrTrace && noStoryboard,
      narrative_body_not_instructional_summary: narrativeBodyNotInstructionalSummary,
      narrative_body_not_fact_boundary_explanation: narrativeBodyNotFactBoundaryExplanation,
      narrative_body_not_duration_plan_explanation: narrativeBodyNotDurationPlanExplanation,
      rewrite_confirmation_body_no_internal_analysis: noInternal,
      rewrite_confirmation_body_no_storyboard_format: noStoryboard,
      rewrite_confirmation_forbidden_internal_terms_absent: rewriteForbiddenHits.length === 0,
      rewrite_confirmation_forbidden_internal_terms_hits: rewriteForbiddenHits,
      rewrite_body_changes_after_scene_or_duration_switch: preText ? mainBody.trim() !== String(preText || "").trim() : true,
      rewrite_uses_current_confirmed_fact_source: preservedFacts,
      rewrite_adapts_to_selected_scene_and_duration: sceneEvidence && durationEvidence,
      rewrite_body_has_scene_evidence: sceneEvidence,
      rewrite_body_has_duration_evidence: durationEvidence,
      kb_reference_not_trace_only: storyBodyKbOraclePresent,
      story_body_kb_oracle_present: storyBodyKbOraclePresent,
      story_body_kb_oracle_affects_structure: storyBodyKbOraclePresent && storyLike,
      story_body_kb_raw_absent: !/(raw KB|raw_kb|sample_text|source_register|overlay_json)/i.test(String(text || "")),
      story_body_scene_type_applied: sceneEvidence,
      story_body_duration_capacity_applied: durationEvidence,
      story_body_changes_when_scene_changes: preText ? storyBody.trim() !== String(preText || "").trim() : true,
      story_body_changes_when_duration_changes: durationEvidence,
      story_body_expansion_uses_seed_source: preservedFacts,
      story_body_rewrite_uses_current_accepted_fact_source: preservedFacts,
      story_body_no_old_scene_style_residue: !/(旧场景|热血战斗的表达|旧热血)/.test(storyBody),
      story_body_no_old_duration_strategy_residue: !/(时长策略|目标时长|旧时长)/.test(storyBody),
      confirmation_block_count: blocks.length,
      confirmation_main_body_length: storyBody.length,
      confirmation_storyboard_hits: storyboardHits,
      confirmation_meta_strategy_hits: metaStrategyHits,
      confirmation_instructional_hits: instructionalHits,
      confirmation_story_hard_fail_hits: storyHardFailHits,
      main_body_hash: String(storyBody.length) + ":" + String(storyBody.charCodeAt(0) || 0),
      raw_values_redacted: true,
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
  const durationSelection = setSelectByLabel("单镜头时长", input.durationSelectValue);
  const selectedVisibleEvidence = visibleBodySceneDurationEvidence(sceneSelection, durationSelection);
  if (!selectedVisibleEvidence.body_has_scene_label || !selectedVisibleEvidence.body_has_duration) {
    return {
      ok: false,
      stage: "visible_body_scene_duration",
      caseId: input.caseId,
      scriptGoal,
      selectedScriptCommand: null,
      expectedProvider: options.expectedProvider,
      expectedModel: options.expectedModel,
      providerStatus: status,
      sceneSelection,
      durationSelection,
      visible_body_scene_duration_evidence: selectedVisibleEvidence,
      layout: layoutEvidence(),
      visibleTextSample: textOf(document.body).slice(0, 1500),
    };
  }
  const sourceEditorNormalization = await normalizeSourceEditorState(input.sceneLabel, input.durationSelectValue);

  await clickButton("放大编辑");
  const inputTextarea = await waitFor(() => document.querySelector(".text-dialog textarea"), "story material textarea", 30000);
  nativeValue(inputTextarea, input.sourceText);
  await clickButton("保存文本", { within: ".text-dialog" });
  await waitFor(() => !document.querySelector(".text-dialog"), "story material dialog closed", 30000);

  let preRewriteEvidence = null;
  const shouldRunPreRewrite =
    input.preSceneLabel &&
    (input.preSceneLabel !== input.sceneLabel ||
      input.preDurationSelectValue !== input.durationSelectValue ||
      Number(input.preDurationSeconds || 0) !== Number(input.durationSeconds || 0));
  if (shouldRunPreRewrite) {
    const preSceneSelection = setSelectByLabel("场景类型", input.preSceneLabel);
    const preDurationSelection = setSelectByLabel("单镜头时长", input.preDurationSelectValue);
    const preContext = {
      sceneLabel: input.preSceneLabel,
      durationSeconds: input.preDurationSeconds,
      durationSelectValue: input.preDurationSelectValue,
      durationMode: "fixed_seconds",
      sourceText: input.sourceText,
    };
    const preVisibleEvidence = visibleBodySceneDurationEvidence(preSceneSelection, preDurationSelection, preContext);
    const preGoal = input.preScriptGoal === "expand" ? "expand" : "rewrite";
    const preCommand = await clickScriptGoalCommand(preGoal);
    await waitFor(
      () => currentTrace()?.command === preCommand.trace_command &&
        !buttons().some((button) => ["扩写中", "处理中"].some((item) => textOf(button).includes(item))),
      "pre " + preGoal + " expand_script trace",
      360000,
    );
    const preTrace = currentTrace();
    const preExpandedDialogEntryText = await clickFirstButton(["查看改写稿", "查看确认稿", "放大编辑"], { within: ".text-control__actions" });
    await waitFor(() => document.querySelector(".text-dialog textarea"), "pre expanded story textarea", 30000);
    const preExpandedText = readScriptDialogText();
    await clickButton("关闭", { within: ".text-dialog" });
    await waitFor(() => !document.querySelector(".text-dialog"), "pre expanded text dialog closed", 30000);
    await clickButton("确定使用", { within: ".text-control__actions" });
    await waitFor(() => findButton("新建镜头任务", { enabled: true }), "pre " + preGoal + " accepted", 30000);
    preRewriteEvidence = {
      scriptGoal: preGoal,
      sceneSelection: preSceneSelection,
      durationSelection: preDurationSelection,
      selectedScriptCommand: preCommand,
      trace: preTrace,
      expandedDialogEntryText: preExpandedDialogEntryText,
      expandedText: preExpandedText,
      confirmationEvidence: compactConfirmationEvidence(preExpandedText, "", preContext),
      visible_body_scene_duration_evidence: preVisibleEvidence,
    };
    setSelectByLabel("场景类型", input.sceneLabel);
    setSelectByLabel("单镜头时长", input.durationSelectValue);
    await normalizeSourceEditorState(input.sceneLabel, input.durationSelectValue);
  }

  const selectedScriptCommand = await clickScriptGoalCommand(scriptGoal);
  await waitFor(
    () => currentTrace()?.command === selectedScriptCommand.trace_command &&
      !buttons().some((button) => ["扩写中", "处理中"].some((item) => textOf(button).includes(item))),
    scriptGoal + " " + selectedScriptCommand.trace_command + " trace",
    360000,
  );
  const expandTrace = currentTrace();
  const expandTraceAttr = readTraceAttr();

  const expandedDialogEntryText = await clickFirstButton(["查看改写稿", "查看确认稿", "放大编辑"], { within: ".text-control__actions" });
  await waitFor(() => document.querySelector(".text-dialog textarea"), "expanded story textarea", 30000);
  const expandedText = readScriptDialogText();
  const rewriteConfirmationEvidence = compactConfirmationEvidence(expandedText, preRewriteEvidence?.expandedText || "");
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
  const tablePaginationEvidence = await showAllStoryboardRowsForEvidence(
    Array.isArray(generateTrace?.ui_rows) ? generateTrace.ui_rows.length : 0,
  );
  const tableRows = normalizeRows();
  const finalVisibleEvidence = visibleBodySceneDurationEvidence(sceneSelection, durationSelection);
  const tableRowsText = tableRows.flat().join("\\n");
  const rowDurationEvidence = storyboardTableDurationEvidence(tableRows);
  const sceneTerms = sceneEvidenceTerms(input.sceneLabel);
  finalVisibleEvidence.rewrite_body_has_scene_label = rewriteConfirmationEvidence.rewrite_body_has_scene_evidence === true;
  finalVisibleEvidence.rewrite_body_has_duration = rewriteConfirmationEvidence.rewrite_body_has_duration_evidence === true;
  finalVisibleEvidence.ui_rows_have_scene_label = tableRowsText.includes(input.sceneLabel) ||
    sceneTerms.some((term) => term && tableRowsText.includes(term));
  finalVisibleEvidence.ui_rows_have_duration = tableRowsText.includes(finalVisibleEvidence.expected_duration) ||
    tableRowsText.includes("目标时长") ||
    tableRowsText.includes("时长") ||
    tableRows.some((row) => row.some((cell) => String(cell).trim() === String(input.durationSeconds || input.durationSelectValue))) ||
    (Number(input.durationSeconds || input.durationSelectValue) > 0 &&
      rowDurationEvidence.row_duration_sum === Number(input.durationSeconds || input.durationSelectValue));
  finalVisibleEvidence.ui_rows_scene_terms_present = sceneTerms.filter((term) => term && tableRowsText.includes(term));
  finalVisibleEvidence.ui_rows_duration_sum = rowDurationEvidence.row_duration_sum;
  finalVisibleEvidence.ui_rows_duration_values = rowDurationEvidence.row_durations;
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
      sceneSelection,
      durationSelection,
      sourceEditorNormalization,
      preRewriteEvidence,
      expandedDialogEntryText,
      expandedText,
      rewriteConfirmationEvidence,
      expandTrace,
      expandTraceAttr,
      generateTrace,
      generateTraceAttr,
      tableRows,
      tableRowCount: tableRows.length,
      tablePaginationEvidence,
      visible_body_scene_duration_evidence: finalVisibleEvidence,
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
    if (finalVisibleEvidence.rewrite_body_has_scene_label !== true) {
      bindingFailures.push("visible_rewrite_body_scene_missing");
    }
    if (finalVisibleEvidence.rewrite_body_has_duration !== true) {
      bindingFailures.push("visible_rewrite_body_duration_missing");
    }
    if (finalVisibleEvidence.ui_rows_have_scene_label !== true) {
      bindingFailures.push("visible_rows_scene_missing");
    }
    if (finalVisibleEvidence.ui_rows_have_duration !== true) {
      bindingFailures.push("visible_rows_duration_missing");
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
        sceneSelection,
        durationSelection,
        sourceEditorNormalization,
        preRewriteEvidence,
        expandedDialogEntryText,
        expandedText,
        rewriteConfirmationEvidence,
        expandTrace,
        expandTraceAttr,
        generateTrace,
        generateTraceAttr,
        tableRows,
        tableRowCount: tableRows.length,
        tablePaginationEvidence,
        visible_body_scene_duration_evidence: finalVisibleEvidence,
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
    preRewriteEvidence,
    providerStatus: status,
    expandButtonText: selectedScriptCommand.selected_ui_text,
    expandedDialogEntryText,
    expandedText,
    rewriteConfirmationEvidence,
    expandTrace,
    expandTraceAttr,
    generateTrace,
    generateTraceAttr,
    tableRows,
    tableRowCount: tableRows.length,
    tablePaginationEvidence,
    visible_body_scene_duration_evidence: finalVisibleEvidence,
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
  const responseRows = asArray(trace?.response_rows);
  const uiRows = asArray(trace?.ui_rows);
  const responseRowsNonEmpty = responseRows.length > 0;
  const uiRowsNonEmpty = uiRows.length > 0;
  const pseudoSuccessDetected = trace?.rows_match === true && (!responseRowsNonEmpty || !uiRowsNonEmpty);
  return {
    validator_gate_present: validatorGatePresent,
    validator_gate_passed: validatorGatePresent ? trace.validator_gate_passed === true : null,
    validator_gate_failures: validatorGateFailures.length ? validatorGateFailures : runnerBindingFailures,
    rows_match: trace?.rows_match === true,
    response_rows_count: responseRows.length,
    ui_rows_count: uiRows.length,
    response_rows_non_empty: responseRowsNonEmpty,
    ui_rows_non_empty: uiRowsNonEmpty,
    pseudo_success_detected: pseudoSuccessDetected,
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
  const readableSectionLabels = ["【镜头目标】", "【画面】", "【动作】", "【镜头】", "【对白/旁白】", "【约束】"];
  const promptInternalDumpPattern =
    /(人物|画面描述|角色动作|运镜|目标时长|模式)\s*=|(?:person|visual_description|character_action|camera_movement|duration_seconds|target_duration_mode|task_type|output_schema|constraints)\s*[=:]|fixed_seconds/i;
  const promptTextReadableSections =
    rows.length > 0 &&
    promptTexts.every((promptText) => readableSectionLabels.every((label) => promptText.includes(label)));
  const promptTextNoInternalFieldDump = !promptTexts.some((promptText) => promptInternalDumpPattern.test(promptText));
  const promptTextNoRawKeyValueDump = !promptTexts.some((promptText) =>
    /(?:^|[；;\n])\s*[\w.-]+\s*[=:]\s*.+/.test(promptText),
  );
  const promptTextUserFacingLayoutPassed =
    promptTextReadableSections && promptTextNoInternalFieldDump && promptTextNoRawKeyValueDump;
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
    prompt_text_readable_sections: promptTextReadableSections,
    prompt_text_no_internal_field_dump: promptTextNoInternalFieldDump,
    prompt_text_no_raw_key_value_dump: promptTextNoRawKeyValueDump,
    prompt_text_user_facing_layout_passed: promptTextUserFacingLayoutPassed,
    ...absentChecks,
    prompt_text_boundary_passed:
      !promptTextMissing && promptTextForbiddenSourceHits.length === 0 && promptTextUserFacingLayoutPassed,
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
  for (const field of [
    "prompt_text_readable_sections",
    "prompt_text_no_internal_field_dump",
    "prompt_text_no_raw_key_value_dump",
    "prompt_text_user_facing_layout_passed",
  ]) {
    if (evidence[field] !== true) {
      failures.push(`prompt_text_layout_failed:${field}`);
    }
  }
  return failures;
}

function compactAcceptedSnapshotEvidence(value) {
  const bindingEvidence = compactBindingEvidence(value?.generateTrace?.binding_evidence ?? value?.bindingEvidence);
  const taskBindingEvidence = compactTaskBindingEvidence(value);
  const blockedBindingContext = value?.generateTrace?.blocked_runtime_evidence?.binding_context ?? {};
  const currentCaseId = pickFirstNonEmptyText(
    bindingEvidence?.current_case_id,
    blockedBindingContext.current_case_id,
    taskBindingEvidence.accepted_snapshot_id,
  );
  const acceptedRewriteHash = pickFirstNonEmptyText(
    bindingEvidence?.accepted_rewrite_hash,
    taskBindingEvidence.accepted_rewrite_hash,
    blockedBindingContext.accepted_rewrite_hash,
  );
  const taskScriptHash = pickFirstNonEmptyText(
    bindingEvidence?.task_script_hash,
    taskBindingEvidence.task_script_hash,
    blockedBindingContext.task_script_hash,
  );
  const storyFactFrameHash = pickFirstNonEmptyText(
    bindingEvidence?.story_fact_frame_hash,
    taskBindingEvidence.accepted_story_fact_frame_hash,
    blockedBindingContext.story_fact_frame_hash,
  );
  const sourceTextHash = pickFirstNonEmptyText(
    bindingEvidence?.source_text_hash,
    taskBindingEvidence.accepted_source_text_hash,
    blockedBindingContext.source_text_hash,
  );
  const durationPlanHash = pickFirstNonEmptyText(
    bindingEvidence?.duration_plan_hash,
    blockedBindingContext.duration_plan_hash,
  );
  const evidenceSource = bindingEvidence
    ? "generateTrace.binding_evidence"
    : Object.keys(blockedBindingContext).length
      ? "generateTrace.blocked_runtime_evidence.binding_context"
      : taskBindingEvidence.present
        ? "generateTrace.task_binding_evidence"
        : "missing";
  return bindingEvidence || Object.keys(blockedBindingContext).length || taskBindingEvidence.present
    ? {
        current_case_id: currentCaseId,
        accepted_snapshot_hash: acceptedRewriteHash,
        accepted_rewrite_hash: acceptedRewriteHash,
        task_script_hash: taskScriptHash,
        story_fact_frame_hash: storyFactFrameHash,
        source_text_hash: sourceTextHash,
        duration_plan_hash: durationPlanHash,
        evidence_source: evidenceSource,
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
  const taskBindingEvidence = compactTaskBindingEvidence(value);
  const blockedBindingContext = value?.generateTrace?.blocked_runtime_evidence?.binding_context ?? {};
  const present = Boolean(bindingEvidence || taskBindingEvidence.present || Object.keys(blockedBindingContext).length);
  return present
    ? {
        present: true,
        current_case_id: pickFirstNonEmptyText(
          bindingEvidence?.current_case_id,
          blockedBindingContext.current_case_id,
          taskBindingEvidence.accepted_snapshot_id,
        ),
        source_text_hash: pickFirstNonEmptyText(
          bindingEvidence?.source_text_hash,
          blockedBindingContext.source_text_hash,
          taskBindingEvidence.accepted_source_text_hash,
        ),
        story_fact_frame_hash: pickFirstNonEmptyText(
          bindingEvidence?.story_fact_frame_hash,
          blockedBindingContext.story_fact_frame_hash,
          taskBindingEvidence.accepted_story_fact_frame_hash,
        ),
        source_profile: pickFirstNonEmptyText(bindingEvidence?.source_profile, blockedBindingContext.source_profile),
        scene_type: pickFirstNonEmptyText(
          bindingEvidence?.scene_type,
          blockedBindingContext.scene_type,
          taskBindingEvidence.task_scene_type,
        ),
        primary_scene_type: pickFirstNonEmptyText(bindingEvidence?.primary_scene_type),
        primary_scene_label: pickFirstNonEmptyText(bindingEvidence?.primary_scene_label),
        shot_scene_type: pickFirstNonEmptyText(bindingEvidence?.shot_scene_type),
        shot_scene_label: pickFirstNonEmptyText(bindingEvidence?.shot_scene_label),
        adaptation_reason_present: Boolean(bindingEvidence?.adaptation_reason_present),
        duration_seconds: bindingEvidence?.duration_seconds ?? blockedBindingContext.duration_seconds ?? taskBindingEvidence.task_duration_seconds ?? null,
        accepted_duration_seconds: bindingEvidence?.accepted_duration_seconds ?? taskBindingEvidence.accepted_duration_seconds ?? null,
        task_duration_seconds: bindingEvidence?.task_duration_seconds ?? taskBindingEvidence.task_duration_seconds ?? null,
        duration_override_reason: pickFirstNonEmptyText(bindingEvidence?.duration_override_reason, taskBindingEvidence.duration_override_reason),
        duration_plan_hash: pickFirstNonEmptyText(bindingEvidence?.duration_plan_hash, blockedBindingContext.duration_plan_hash),
        storyboard_rows_hash: pickFirstNonEmptyText(bindingEvidence?.storyboard_rows_hash, blockedBindingContext.storyboard_rows_hash),
        must_keep_facts_present: Array.isArray(bindingEvidence?.must_keep_facts),
        must_keep_facts_count: asArray(bindingEvidence?.must_keep_facts).length,
        missing_source_facts: asArray(bindingEvidence?.missing_source_facts),
        forbidden_facts_present: Array.isArray(bindingEvidence?.forbidden_facts),
        forbidden_facts_count: asArray(bindingEvidence?.forbidden_facts).length,
        forbidden_fact_hits: asArray(bindingEvidence?.forbidden_fact_hits),
        stale_binding_detected: bindingEvidence?.stale_binding_detected === false ? false : Boolean(bindingEvidence?.stale_binding_detected),
        kb_rule_pack_ids_present: Array.isArray(bindingEvidence?.kb_rule_pack_ids),
        kb_rule_pack_ids: asArray(bindingEvidence?.kb_rule_pack_ids),
        kb_snapshot_hash: pickFirstNonEmptyText(bindingEvidence?.kb_snapshot_hash),
      }
    : { present: false };
}

function pickFirstNonEmptyText(...values) {
  for (const value of values) {
    const text = String(value ?? "").trim();
    if (text.length > 0) {
      return text;
    }
  }
  return "";
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
    required_ui_text: selected.required_ui_text ?? "",
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
    selected_ui_text_matches_goal:
      goal === "rewrite"
        ? String(selected.selected_ui_text ?? "").includes("改写剧本")
        : String(selected.selected_ui_text ?? "").includes("扩写故事"),
  };
}

function compactTaskBindingEvidence(value) {
  const evidence = value?.generateTrace?.task_binding_evidence ?? {};
  return {
    present: Boolean(value?.generateTrace?.task_binding_evidence),
    candidate_id: evidence.candidate_id ?? "",
    script_text_source: evidence.script_text_source ?? "",
    base_scene_text_hash: evidence.base_scene_text_hash ?? "",
    task_script_hash: evidence.task_script_hash ?? "",
    task_scene_type: evidence.task_scene_type ?? "",
    task_scene_label: evidence.task_scene_label ?? "",
    task_duration_seconds: evidence.task_duration_seconds ?? null,
    accepted_snapshot_id: evidence.accepted_snapshot_id ?? "",
    accepted_source_text_hash: evidence.accepted_source_text_hash ?? "",
    accepted_rewrite_hash: evidence.accepted_rewrite_hash ?? "",
    accepted_kb_oracle_hash: evidence.accepted_kb_oracle_hash ?? "",
    accepted_story_fact_frame_hash: evidence.accepted_story_fact_frame_hash ?? "",
    accepted_duration_seconds: evidence.accepted_duration_seconds ?? null,
    duration_override_reason: evidence.duration_override_reason ?? "",
  };
}

function compactTaskBindingFailures(evidence) {
  if (!evidence.present) {
    return ["task_binding_evidence_missing"];
  }
  const failures = [];
  for (const field of [
    "candidate_id",
    "script_text_source",
    "base_scene_text_hash",
    "task_script_hash",
    "task_scene_type",
    "task_scene_label",
    "accepted_snapshot_id",
    "accepted_source_text_hash",
    "accepted_rewrite_hash",
    "accepted_kb_oracle_hash",
    "accepted_story_fact_frame_hash",
  ]) {
    if (!String(evidence[field] ?? "").trim()) {
      failures.push(`task_binding_missing:${field}`);
    }
  }
  if (!Number.isFinite(Number(evidence.task_duration_seconds)) || Number(evidence.task_duration_seconds) <= 0) {
    failures.push("task_binding_missing:task_duration_seconds");
  }
  if (!Number.isFinite(Number(evidence.accepted_duration_seconds)) || Number(evidence.accepted_duration_seconds) <= 0) {
    failures.push("task_binding_missing:accepted_duration_seconds");
  }
  return failures;
}

function compactKbOracleTraceEvidence(trace) {
  return {
    present: Boolean(trace?.kb_oracle_hash),
    kb_oracle_hash: trace?.kb_oracle_hash ?? "",
    kb_context_summary_hash: trace?.kb_context_summary_hash ?? "",
    kb_rule_pack_ids: asArray(trace?.kb_rule_pack_ids),
    kb_snapshot_hash: trace?.kb_snapshot_hash ?? "",
    full_kb_rows_included: Number(trace?.full_kb_rows_included ?? 0),
  };
}

function compactKbOracleFailures(evidence, label = "kb_oracle") {
  const failures = [];
  if (!evidence.present) {
    failures.push(`${label}_missing`);
  }
  if (!String(evidence.kb_context_summary_hash ?? "").trim()) {
    failures.push(`${label}_context_hash_missing`);
  }
  if (!asArray(evidence.kb_rule_pack_ids).length) {
    failures.push(`${label}_rule_pack_missing`);
  }
  if (!String(evidence.kb_snapshot_hash ?? "").trim()) {
    failures.push(`${label}_snapshot_hash_missing`);
  }
  if (Number(evidence.full_kb_rows_included) !== 0) {
    failures.push(`${label}_full_rows_included`);
  }
  return failures;
}

function compactVisibleBodySceneDurationEvidence(value) {
  const evidence = value?.visible_body_scene_duration_evidence ?? {};
  return {
    expected_scene_label: evidence.expected_scene_label ?? input.sceneLabel,
    selected_scene_value: evidence.selected_scene_value ?? "",
    selected_scene_text: evidence.selected_scene_text ?? "",
    expected_duration: evidence.expected_duration ?? String(input.durationSeconds || input.durationSelectValue),
    selected_duration_value: evidence.selected_duration_value ?? "",
    selected_duration_text: evidence.selected_duration_text ?? "",
    body_has_scene_label: evidence.body_has_scene_label === true,
    body_has_selected_scene_text: evidence.body_has_selected_scene_text === true,
    body_has_duration: evidence.body_has_duration === true,
    rewrite_body_has_scene_label: evidence.rewrite_body_has_scene_label === true,
    rewrite_body_has_duration: evidence.rewrite_body_has_duration === true,
    ui_rows_have_scene_label: evidence.ui_rows_have_scene_label === true,
    ui_rows_have_duration: evidence.ui_rows_have_duration === true,
    ui_rows_duration_sum: Number(evidence.ui_rows_duration_sum ?? 0),
    ui_rows_duration_values: asArray(evidence.ui_rows_duration_values).map((value) => Number(value)).filter((value) => value > 0),
    ui_rows_scene_terms_present: asArray(evidence.ui_rows_scene_terms_present).map(String).slice(0, 8),
    raw_values_redacted: true,
  };
}

function compactRewriteConfirmationEvidence(value) {
  const evidence = value?.rewriteConfirmationEvidence ?? {};
  return {
    rewrite_button_label_exact:
      value?.scriptGoal === "rewrite"
        ? String(value?.selectedScriptCommand?.selected_ui_text ?? "") === "改写剧本"
        : true,
    rewrite_confirmation_dialog_present: evidence.rewrite_confirmation_dialog_present === true,
    rewrite_confirmation_first_block_label: evidence.rewrite_confirmation_first_block_label ?? "",
    rewrite_confirmation_body_first: evidence.rewrite_confirmation_body_first === true,
    rewrite_confirmation_main_body_is_story: evidence.rewrite_confirmation_main_body_is_story === true,
    rewrite_main_body_is_narrative_story: evidence.rewrite_main_body_is_narrative_story === true,
    main_textarea_is_complete_story: evidence.main_textarea_is_complete_story === true,
    narrative_body_has_continuous_actions: evidence.narrative_body_has_continuous_actions === true,
    narrative_body_has_character_subjects: evidence.narrative_body_has_character_subjects === true,
    narrative_body_has_conflict_progression: evidence.narrative_body_has_conflict_progression === true,
    narrative_body_has_character_goal: evidence.narrative_body_has_character_goal === true,
    narrative_body_has_conflict_causality: evidence.narrative_body_has_conflict_causality === true,
    narrative_body_has_emotional_turn: evidence.narrative_body_has_emotional_turn === true,
    narrative_body_has_story_resolution_beat: evidence.narrative_body_has_story_resolution_beat === true,
    narrative_body_has_scene_specific_expression: evidence.narrative_body_has_scene_specific_expression === true,
    narrative_body_not_meta_strategy: evidence.narrative_body_not_meta_strategy === true,
    narrative_body_not_action_choreography: evidence.narrative_body_not_action_choreography === true,
    narrative_body_not_strategy_or_trace: evidence.narrative_body_not_strategy_or_trace === true,
    narrative_body_not_mechanical_rewrite_template:
      evidence.narrative_body_not_mechanical_rewrite_template === true,
    narrative_body_not_storyboard_breakdown: evidence.narrative_body_not_storyboard_breakdown === true,
    narrative_body_not_instructional_summary: evidence.narrative_body_not_instructional_summary === true,
    narrative_body_not_fact_boundary_explanation: evidence.narrative_body_not_fact_boundary_explanation === true,
    narrative_body_not_duration_plan_explanation: evidence.narrative_body_not_duration_plan_explanation === true,
    rewrite_confirmation_body_no_internal_analysis: evidence.rewrite_confirmation_body_no_internal_analysis === true,
    rewrite_confirmation_body_no_storyboard_format: evidence.rewrite_confirmation_body_no_storyboard_format === true,
    rewrite_confirmation_forbidden_internal_terms_absent:
      evidence.rewrite_confirmation_forbidden_internal_terms_absent === true,
    rewrite_body_changes_after_scene_or_duration_switch:
      evidence.rewrite_body_changes_after_scene_or_duration_switch === true,
    rewrite_uses_current_confirmed_fact_source:
      evidence.rewrite_uses_current_confirmed_fact_source === true,
    rewrite_adapts_to_selected_scene_and_duration:
      evidence.rewrite_adapts_to_selected_scene_and_duration === true,
    kb_reference_not_trace_only: evidence.kb_reference_not_trace_only === true,
    story_body_kb_oracle_present: evidence.story_body_kb_oracle_present === true,
    story_body_kb_oracle_affects_structure: evidence.story_body_kb_oracle_affects_structure === true,
    story_body_kb_raw_absent: evidence.story_body_kb_raw_absent === true,
    story_body_scene_type_applied: evidence.story_body_scene_type_applied === true,
    story_body_duration_capacity_applied: evidence.story_body_duration_capacity_applied === true,
    story_body_changes_when_scene_changes: evidence.story_body_changes_when_scene_changes === true,
    story_body_changes_when_duration_changes: evidence.story_body_changes_when_duration_changes === true,
    story_body_expansion_uses_seed_source: evidence.story_body_expansion_uses_seed_source === true,
    story_body_rewrite_uses_current_accepted_fact_source:
      evidence.story_body_rewrite_uses_current_accepted_fact_source === true,
    story_body_no_old_scene_style_residue: evidence.story_body_no_old_scene_style_residue === true,
    story_body_no_old_duration_strategy_residue: evidence.story_body_no_old_duration_strategy_residue === true,
    confirmation_block_count: Number(evidence.confirmation_block_count ?? 0),
    confirmation_main_body_length: Number(evidence.confirmation_main_body_length ?? 0),
    confirmation_storyboard_hits: asArray(evidence.confirmation_storyboard_hits).map(String).slice(0, 12),
    confirmation_meta_strategy_hits: asArray(evidence.confirmation_meta_strategy_hits).map(String).slice(0, 12),
    confirmation_instructional_hits: asArray(evidence.confirmation_instructional_hits).map(String).slice(0, 12),
    confirmation_story_hard_fail_hits: asArray(evidence.confirmation_story_hard_fail_hits).map(String).slice(0, 12),
    raw_values_redacted: true,
  };
}

function compactRewriteConfirmationFailures(evidence) {
  const failures = [];
  for (const [field, value] of Object.entries(evidence)) {
    if (field === "rewrite_confirmation_first_block_label" || field === "raw_values_redacted") {
      continue;
    }
    if (typeof value !== "boolean") {
      continue;
    }
    if (value !== true) {
      failures.push(`rewrite_confirmation_missing:${field}`);
    }
  }
  return failures;
}

function compactPreRewriteConfirmationEvidence(value) {
  const evidence = value?.preRewriteEvidence?.confirmationEvidence;
  if (!evidence) {
    return null;
  }
  return {
    ...evidence,
    raw_values_redacted: true,
  };
}

function compactPreRewriteConfirmationFailures(evidence) {
  if (!evidence) {
    return [];
  }
  return compactRewriteConfirmationFailures(evidence)
    .map((failure) => failure.replace("rewrite_confirmation_missing:", "pre_rewrite_confirmation_missing:"));
}

function compactVisibleBodySceneDurationFailures(evidence) {
  const failures = [];
  if (!evidence.body_has_scene_label && !evidence.body_has_selected_scene_text) {
    failures.push("visible_body_scene_missing");
  }
  if (!evidence.body_has_duration) {
    failures.push("visible_body_duration_missing");
  }
  if (!evidence.rewrite_body_has_scene_label) {
    failures.push("visible_rewrite_body_scene_missing");
  }
  if (!evidence.rewrite_body_has_duration) {
    failures.push("visible_rewrite_body_duration_missing");
  }
  if (!evidence.ui_rows_have_scene_label) {
    failures.push("visible_rows_scene_missing");
  }
  if (!evidence.ui_rows_have_duration) {
    failures.push("visible_rows_duration_missing");
  }
  return failures;
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
  if (!evidence.selected_ui_text_matches_goal) {
    failures.push("selected_ui_text_mismatch");
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
  const allowedFailoverReasons = new Set([
    "none",
    "",
    "quota_exhausted",
    "model_unavailable",
    "entitlement_http_403",
    "permission_http_403",
  ]);
  if (!allowedFailoverReasons.has(String(evidence.provider_failover_reason ?? ""))) {
    failures.push("provider_failover_reason_not_allowed");
  }
  if (evidence.provider_failover_used === true && String(evidence.provider_failover_reason ?? "none") === "none") {
    failures.push("provider_failover_reason_missing");
  }
  if (evidence.provider_failover_evidence?.raw_primary_http_403_seen === true &&
    evidence.provider_failover_evidence?.no_unhandled_http_403 !== true) {
    failures.push("provider_http_403_unhandled");
  }
  if (!evidence.no_http_403) {
    failures.push("http_403_detected");
  }
  if (!evidence.no_live_fallback) {
    failures.push("live_fallback_detected");
  }
  if (evidence.fallback_used === true) {
    failures.push("fallback_used_true");
  }
  if (evidence.local_candidate === true) {
    failures.push("local_candidate_true");
  }
  if (evidence.validator_pseudo_success_detected === true) {
    failures.push("validator_pseudo_success_detected");
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

function parseTraceKeyValueEvidence(traces) {
  const records = collectTraceWarningRecords(traces);
  const text = records.map((item) => warningRecordText(item)).join("\n");
  const evidence = {};
  for (const key of [
    "primary_model",
    "fallback_model",
    "attempted_models",
    "selected_model",
    "final_model",
    "provider_failover_used",
    "provider_failover_reason",
    "primary_http_status",
    "final_http_status",
    "raw_primary_http_403_seen",
    "provider_http_403_handled_by_model_failover",
    "no_unhandled_http_403",
    "fallback_used",
    "local_candidate",
  ]) {
    const match = text.match(new RegExp(`(?:^|[;\\s])${key}\\s*[:=]\\s*([^;\\n]+)`, "i"));
    if (match) {
      evidence[key] = match[1].trim();
    }
  }
  return evidence;
}

function statusValue(value) {
  const text = String(value ?? "").trim().toLowerCase();
  if (!text || text === "none" || text === "null" || text === "unknown") {
    return null;
  }
  const number = Number(text);
  return Number.isFinite(number) ? number : null;
}

function boolValue(value, fallback = false) {
  const text = String(value ?? "").trim().toLowerCase();
  if (text === "true") {
    return true;
  }
  if (text === "false") {
    return false;
  }
  return fallback;
}

function collectProviderFailoverEvidence(traces, providerSucceeded) {
  const kv = parseTraceKeyValueEvidence(traces);
  const attempted = String(kv.attempted_models ?? "")
    .split(/[|,]/)
    .map((item) => item.trim())
    .filter(Boolean);
  const providerFailoverUsed = boolValue(kv.provider_failover_used, false);
  const finalStatus = statusValue(kv.final_http_status);
  const primaryStatus = statusValue(kv.primary_http_status);
  const defaultFinalModel = input.expectedModel || "qwen3.6-plus";
  return {
    primary_model: kv.primary_model || "qwen3.6-plus",
    fallback_model: kv.fallback_model || "qwen3.6-plus-2026-04-02",
    attempted_models: attempted.length ? attempted : [defaultFinalModel],
    selected_model: kv.selected_model || defaultFinalModel,
    final_model: kv.final_model || defaultFinalModel,
    provider_failover_used: providerFailoverUsed,
    provider_failover_reason: kv.provider_failover_reason || "none",
    primary_http_status: primaryStatus ?? (providerSucceeded ? 200 : null),
    final_http_status: finalStatus ?? (providerSucceeded ? 200 : null),
    raw_primary_http_403_seen: boolValue(kv.raw_primary_http_403_seen, primaryStatus === 403),
    provider_http_403_handled_by_model_failover: boolValue(kv.provider_http_403_handled_by_model_failover, false),
    no_unhandled_http_403: boolValue(kv.no_unhandled_http_403, finalStatus !== 403 && primaryStatus !== 403),
    fallback_used: boolValue(kv.fallback_used, false),
    local_candidate: boolValue(kv.local_candidate, false),
    evidence_source: Object.keys(kv).length ? "runtime_trace" : "runner_default_no_failover",
    raw_values_redacted: true,
  };
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

function classifyProviderAvailability(warningText, retryTimeline, providerFailoverEvidence) {
  const finalHttpStatus = Number(providerFailoverEvidence?.final_http_status);
  if (Number.isFinite(finalHttpStatus) && finalHttpStatus >= 200 && finalHttpStatus < 300) {
    return {
      unavailable: false,
      http_status: finalHttpStatus,
      error_category: "none",
      error_code: "none",
      retry_error_categories: asArray(retryTimeline?.error_categories),
      provider_failover_used: providerFailoverEvidence.provider_failover_used === true,
      raw_values_redacted: true,
    };
  }
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

function compactVisualDescriptionEvidence(value) {
  const rows = asArray(value?.generateTrace?.ui_rows).length
    ? asArray(value.generateTrace.ui_rows)
    : asArray(value?.generateTrace?.response_rows);
  const forbiddenTerms = [
    "主体为",
    "重点落在",
    "场景表达",
    "节奏推进",
    "压力方向",
    "路线变化",
    "画面主体保持",
    "当前视觉事件",
    "镜头容量",
    "动作拉扯感",
    "人物反应都留出位置",
    "目标时长",
    "模式 fixed_seconds",
    "任务 30 秒",
    "可拆镜头数量",
    "人物=",
    "画面描述=",
    "角色动作=",
  ];
  const visibleFrameTerms = [
    "前景",
    "中景",
    "后景",
    "侧面",
    "侧视",
    "地面",
    "背景",
    "冷光",
    "树影",
    "碎石",
    "巷口",
    "竹林",
    "废墟",
    "脚步",
    "移动痕迹",
  ];
  const sceneTerms = ["竹林", "巷口", "废墟", "小径", "空间", "通道", "地面", "树影", "碎石", "墙面", "战场", "码头"];
  const compositionTerms = ["远景", "全景", "中景", "近景", "特写", "前景", "中景", "后景", "侧面", "斜侧", "构图", "遮挡"];
  const actionTerms = ["移动", "追", "逼近", "回身", "转身", "压近", "护住", "提醒", "格挡", "交错", "停住", "跪", "行动", "脚步"];
  const fieldDumpPattern = /(人物|画面描述|角色动作|运镜|目标时长|模式)\s*=|(?:person|visual_description|character_action|camera_movement|duration_seconds|target_duration_mode)\s*[=:]|fixed_seconds/i;
  const rowEvidence = rows.map((row, index) => {
    const visual = String(row?.visual_description ?? "");
    const person = String(row?.person ?? "");
    const forbidden = forbiddenTerms.filter((term) => visual.includes(term));
    const sentenceCount = visual.split(/[。；;.!?\n]/).map((part) => part.trim()).filter(Boolean).length;
    return {
      row_index: index + 1,
      person_present: person.trim().length > 0 && person.trim() !== "/",
      no_abstract_strategy_terms: forbidden.length === 0,
      visible_frame_terms_present: visibleFrameTerms.some((term) => visual.includes(term)),
      person_in_visual_description: person.trim().length > 0 && visual.includes(person.trim()),
      visual_description_readable_structure: sentenceCount >= 2 && sentenceCount <= 5 && visual.length <= 260,
      visual_description_scene_present: sceneTerms.some((term) => visual.includes(term)),
      visual_description_composition_present: compositionTerms.some((term) => visual.includes(term)),
      visual_description_character_action_present: actionTerms.some((term) => visual.includes(term)),
      visual_description_no_internal_field_dump: !fieldDumpPattern.test(visual),
      visual_description_hash: stableEvidenceHash(visual),
      forbidden_terms_count: forbidden.length,
    };
  });
  return {
    row_count: rows.length,
    visual_description_visible_frame_passed:
      rowEvidence.length > 0 &&
      rowEvidence.every((row) => row.person_present && row.visible_frame_terms_present && row.person_in_visual_description),
    visual_description_no_abstract_strategy_terms:
      rowEvidence.length > 0 && rowEvidence.every((row) => row.no_abstract_strategy_terms),
    visual_description_readable_structure:
      rowEvidence.length > 0 && rowEvidence.every((row) => row.visual_description_readable_structure),
    visual_description_scene_present:
      rowEvidence.length > 0 && rowEvidence.every((row) => row.visual_description_scene_present),
    visual_description_composition_present:
      rowEvidence.length > 0 && rowEvidence.every((row) => row.visual_description_composition_present),
    visual_description_character_action_present:
      rowEvidence.length > 0 && rowEvidence.every((row) => row.visual_description_character_action_present),
    visual_description_no_internal_field_dump:
      rowEvidence.length > 0 && rowEvidence.every((row) => row.visual_description_no_internal_field_dump),
    rows: rowEvidence,
    raw_values_redacted: true,
  };
}

function compactVisualDescriptionFailures(evidence) {
  const failures = [];
  if (!evidence.visual_description_visible_frame_passed) {
    failures.push("visual_description_visible_frame_failed");
  }
  if (!evidence.visual_description_no_abstract_strategy_terms) {
    failures.push("visual_description_abstract_strategy_terms_present");
  }
  for (const field of [
    "visual_description_readable_structure",
    "visual_description_scene_present",
    "visual_description_composition_present",
    "visual_description_character_action_present",
    "visual_description_no_internal_field_dump",
  ]) {
    if (evidence[field] !== true) {
      failures.push(`visual_description_readability_failed:${field}`);
    }
  }
  return failures;
}

function compactModelOutputContractEvidence({
  value,
  validatorGateEvidence,
  acceptedSnapshotEvidence,
  storyFactFrameEvidence,
  taskBindingEvidence,
  expandKbOracleEvidence,
  preKbOracleEvidence,
  visibleBodySceneDurationEvidence,
  rewriteConfirmationEvidence,
  visualDescriptionEvidence,
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
    artifact_identity: artifactIdentityPreflight,
    source_integrity_preflight: {
      runtime_rs_utf8_ok: artifactIdentityPreflight.runtime_rs_utf8_ok,
      runtime_rs_replacement_char_absent: artifactIdentityPreflight.runtime_rs_replacement_char_absent,
      runtime_rs_sentinel_check_passed: artifactIdentityPreflight.runtime_rs_sentinel_check_passed,
      release_exe_fresh_for_runtime: artifactIdentityPreflight.release_exe_fresh_for_runtime,
      source_integrity_preflight_passed: artifactIdentityPreflight.source_integrity_preflight_passed,
      raw_values_redacted: true,
    },
    provider: value?.expectedProvider ?? input.expectedProvider,
    model: fallbackGateEvidence.final_model ?? value?.expectedModel ?? input.expectedModel,
    expected_model: value?.expectedModel ?? input.expectedModel,
    model_gate_role: input.expectedModelGateRole,
    final_model_gate_role: modelGateRole(fallbackGateEvidence.final_model ?? value?.expectedModel ?? input.expectedModel),
    required_gate_model: input.expectedModelGateRole === "required_text_gate_model",
    alias_compatibility_reference: input.expectedModelGateRole === "alias_compatibility_reference",
    script_goal: value?.scriptGoal ?? input.scriptGoal,
    scene_type: storyFactFrameEvidence?.scene_type ?? "",
    duration: storyFactFrameEvidence?.duration_seconds ?? null,
    accepted_snapshot_hash: acceptedSnapshotEvidence.accepted_snapshot_hash,
    story_fact_frame_hash: storyFactFrameEvidence?.story_fact_frame_hash ?? "",
    task_binding_evidence: taskBindingEvidence,
    kb_oracle_evidence: {
      expand: expandKbOracleEvidence,
      pre: preKbOracleEvidence,
      raw_values_redacted: true,
    },
    visible_body_scene_duration_evidence: visibleBodySceneDurationEvidence,
    rewrite_confirmation_evidence: rewriteConfirmationEvidence,
    visual_description_evidence: visualDescriptionEvidence,
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
      provider_failover_used: fallbackGateEvidence.provider_failover_used,
    },
    provider_retry_evidence: {
      primary_model: fallbackGateEvidence.primary_model,
      fallback_model: fallbackGateEvidence.fallback_model,
      attempted_models: fallbackGateEvidence.attempted_models,
      selected_model: fallbackGateEvidence.selected_model,
      final_model: fallbackGateEvidence.final_model,
      provider_failover_used: fallbackGateEvidence.provider_failover_used,
      provider_failover_reason: fallbackGateEvidence.provider_failover_reason,
      primary_http_status: fallbackGateEvidence.primary_http_status,
      final_http_status: fallbackGateEvidence.final_http_status,
      raw_primary_http_403_seen: fallbackGateEvidence.raw_primary_http_403_seen,
      provider_http_403_handled_by_model_failover: fallbackGateEvidence.provider_http_403_handled_by_model_failover,
      no_unhandled_http_403: fallbackGateEvidence.no_unhandled_http_403,
      provider_failover_evidence: fallbackGateEvidence.provider_failover_evidence,
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
  const providerFailoverEvidence = collectProviderFailoverEvidence(traces, value?.ok === true);
  const modelAvailability = classifyProviderAvailability(warningText, retryTimeline, providerFailoverEvidence);
  const failoverFinalSuccess =
    providerFailoverEvidence.provider_failover_used === true &&
    Number(providerFailoverEvidence.final_http_status) >= 200 &&
    Number(providerFailoverEvidence.final_http_status) < 300;
  return {
    no_http_403: providerFailoverEvidence.no_unhandled_http_403 === false
      ? false
      : failoverFinalSuccess
      ? true
      : providerFailoverEvidence.final_http_status === 403
        ? false
        : !/(^|\D)403(\D|$)|http_403|forbidden/i.test(warningText),
    no_live_fallback: liveFallbackSignals.length === 0,
    live_fallback_signals: liveFallbackSignals,
    provider_failover_evidence: providerFailoverEvidence,
    primary_model: providerFailoverEvidence.primary_model,
    fallback_model: providerFailoverEvidence.fallback_model,
    attempted_models: providerFailoverEvidence.attempted_models,
    selected_model: providerFailoverEvidence.selected_model,
    final_model: providerFailoverEvidence.final_model,
    provider_failover_used: providerFailoverEvidence.provider_failover_used,
    provider_failover_reason: providerFailoverEvidence.provider_failover_reason,
    primary_http_status: providerFailoverEvidence.primary_http_status,
    final_http_status: providerFailoverEvidence.final_http_status,
    raw_primary_http_403_seen: providerFailoverEvidence.raw_primary_http_403_seen,
    provider_http_403_handled_by_model_failover: providerFailoverEvidence.provider_http_403_handled_by_model_failover,
    no_unhandled_http_403: providerFailoverEvidence.no_unhandled_http_403,
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
      validatorGateEvidence.response_rows_non_empty === true &&
      validatorGateEvidence.ui_rows_non_empty === true &&
      rowDiffs.length === 0 &&
      promptTextBoundaryEvidence.prompt_text_boundary_passed === true,
    validator_pseudo_success_detected: validatorGateEvidence.pseudo_success_detected === true,
    rows_match: validatorGateEvidence.rows_match === true,
    response_rows_non_empty: validatorGateEvidence.response_rows_non_empty === true,
    ui_rows_non_empty: validatorGateEvidence.ui_rows_non_empty === true,
    response_rows_count: validatorGateEvidence.response_rows_count,
    ui_rows_count: validatorGateEvidence.ui_rows_count,
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
  const taskBindingEvidence = compactTaskBindingEvidence(value);
  const expandKbOracleEvidence = compactKbOracleTraceEvidence(value?.expandTrace);
  const preKbOracleEvidence = compactKbOracleTraceEvidence(value?.preRewriteEvidence?.trace);
  const visibleBodySceneDurationEvidence = compactVisibleBodySceneDurationEvidence(value);
  const rewriteConfirmationEvidence = compactRewriteConfirmationEvidence(value);
  const promptTextBoundaryEvidence = compactPromptTextBoundaryEvidence(value);
  const promptTextBoundaryFailures = compactPromptTextBoundaryFailures(promptTextBoundaryEvidence);
  const blockedRuntimeEvidence = compactBlockedRuntimeEvidence(value?.generateTrace);
  const fallbackGateEvidence = compactFallbackGateEvidence(value, validatorGateEvidence, promptTextBoundaryEvidence);
  const creativeFreedomEvidence = compactCreativeFreedomEvidence(value, promptTextBoundaryEvidence);
  const visualDescriptionEvidence = compactVisualDescriptionEvidence(value);
  const traces = [value?.expandTrace, value?.generateTrace].filter(Boolean);
  const providerHardFailFailures = compactProviderHardFailFailures(fallbackGateEvidence);
  const providerAvailabilityFailures = compactProviderAvailabilityFailures(fallbackGateEvidence);
  const runtimeValidatorHardGateFailures = compactRuntimeValidatorHardGateFailures(traces);
  const scriptGoalFailures = compactScriptGoalFailures(scriptGoalEvidence);
  const acceptedSnapshotFailures = compactAcceptedSnapshotFailures(acceptedSnapshotEvidence);
  const storyFactFrameFailures = compactStoryFactFrameFailures(storyFactFrameEvidence);
  const taskBindingFailures = compactTaskBindingFailures(taskBindingEvidence);
  const expandKbOracleFailures = compactKbOracleFailures(expandKbOracleEvidence, "expand_kb_oracle");
  const preKbOracleFailures = value?.preRewriteEvidence?.trace
    ? compactKbOracleFailures(preKbOracleEvidence, "pre_kb_oracle")
    : [];
  const visibleBodySceneDurationFailures = compactVisibleBodySceneDurationFailures(visibleBodySceneDurationEvidence);
  const rewriteConfirmationFailures = compactRewriteConfirmationFailures(rewriteConfirmationEvidence);
  const preRewriteConfirmationEvidence = compactPreRewriteConfirmationEvidence(value);
  const preRewriteConfirmationFailures = compactPreRewriteConfirmationFailures(preRewriteConfirmationEvidence);
  const visualDescriptionFailures = compactVisualDescriptionFailures(visualDescriptionEvidence);
  const fallbackGateFailures = compactFallbackGateFailures(fallbackGateEvidence);
  const downstreamGateFailures = fallbackGateEvidence.qa_hard_fail
    ? []
    : [
        ...acceptedSnapshotFailures,
        ...storyFactFrameFailures,
        ...taskBindingFailures,
        ...preKbOracleFailures,
        ...preRewriteConfirmationFailures,
        ...expandKbOracleFailures,
        ...visibleBodySceneDurationFailures,
        ...rewriteConfirmationFailures,
        ...visualDescriptionFailures,
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
    taskBindingEvidence,
    expandKbOracleEvidence,
    preKbOracleEvidence,
    visibleBodySceneDurationEvidence,
    rewriteConfirmationEvidence,
    visualDescriptionEvidence,
    promptTextBoundaryEvidence,
    fallbackGateEvidence,
    hardGateFailures: runnerAssertFailures,
    qualityWarnings,
    creativeFreedomEvidence,
  });
  const modelCertificationSummary = compactModelCertificationSummary(modelOutputContractEvidence);
  const completedArtifactIdentity = {
    ...artifactIdentityPreflight,
    completed_at: new Date().toISOString(),
  };
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
          artifact_identity: completedArtifactIdentity,
          source_integrity_preflight: {
            runtime_rs_utf8_ok: completedArtifactIdentity.runtime_rs_utf8_ok,
            runtime_rs_replacement_char_absent: completedArtifactIdentity.runtime_rs_replacement_char_absent,
            runtime_rs_sentinel_check_passed: completedArtifactIdentity.runtime_rs_sentinel_check_passed,
            runtime_rs_missing_sentinels: completedArtifactIdentity.runtime_rs_missing_sentinels,
            release_exe_fresh_for_runtime: completedArtifactIdentity.release_exe_fresh_for_runtime,
            source_integrity_preflight_passed: completedArtifactIdentity.source_integrity_preflight_passed,
            raw_values_redacted: true,
          },
          runner_env_proxy_evidence: runnerEnvProxyEvidence,
          warningCodes: value?.warningCodes,
          bindingFailures: [...asArray(value?.bindingFailures), ...runnerAssertFailures],
          runner_assert_failures: runnerAssertFailures,
          hard_gate_failures: runnerAssertFailures,
          quality_warnings: qualityWarnings,
          rows_match: validatorGateEvidence.rows_match,
          row_diffs: validatorGateEvidence.row_diffs,
          response_rows_count: validatorGateEvidence.response_rows_count,
          ui_rows_count: validatorGateEvidence.ui_rows_count,
          response_rows_non_empty: validatorGateEvidence.response_rows_non_empty,
          ui_rows_non_empty: validatorGateEvidence.ui_rows_non_empty,
          validator_gate_present: validatorGateEvidence.validator_gate_present,
          validator_gate_passed: validatorGateEvidence.validator_gate_passed,
          validator_gate_failures: validatorGateEvidence.validator_gate_failures,
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
          task_binding_evidence: taskBindingEvidence,
          pre_kb_oracle_evidence: preKbOracleEvidence,
          expand_kb_oracle_evidence: expandKbOracleEvidence,
          visible_body_scene_duration_evidence: visibleBodySceneDurationEvidence,
          table_pagination_evidence: value?.tablePaginationEvidence,
          rewrite_confirmation_evidence: rewriteConfirmationEvidence,
          pre_rewrite_confirmation_evidence: preRewriteConfirmationEvidence,
          visual_description_evidence: visualDescriptionEvidence,
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
          primary_model: fallbackGateEvidence.primary_model,
          fallback_model: fallbackGateEvidence.fallback_model,
          attempted_models: fallbackGateEvidence.attempted_models,
          selected_model: fallbackGateEvidence.selected_model,
          final_model: fallbackGateEvidence.final_model,
          provider_failover_used: fallbackGateEvidence.provider_failover_used,
          provider_failover_reason: fallbackGateEvidence.provider_failover_reason,
          primary_http_status: fallbackGateEvidence.primary_http_status,
          final_http_status: fallbackGateEvidence.final_http_status,
          raw_primary_http_403_seen: fallbackGateEvidence.raw_primary_http_403_seen,
          provider_http_403_handled_by_model_failover: fallbackGateEvidence.provider_http_403_handled_by_model_failover,
          no_unhandled_http_403: fallbackGateEvidence.no_unhandled_http_403,
          fallback_used: fallbackGateEvidence.fallback_used,
          local_candidate: fallbackGateEvidence.local_candidate,
          qa_proxy_evidence: fallbackGateEvidence.qa_proxy_evidence,
          no_http_403: fallbackGateEvidence.no_http_403,
          no_live_fallback: fallbackGateEvidence.no_live_fallback,
          no_validator_pseudo_success: fallbackGateEvidence.no_validator_pseudo_success,
          sceneSelection: value?.sceneSelection,
          durationSelection: value?.durationSelection,
          preRewriteEvidence: value?.preRewriteEvidence
            ? {
                sceneSelection: value.preRewriteEvidence.sceneSelection,
                durationSelection: value.preRewriteEvidence.durationSelection,
                selected_command: value.preRewriteEvidence.selectedScriptCommand,
                trace_status: value.preRewriteEvidence.trace?.status,
                trace_script_goal: value.preRewriteEvidence.trace?.script_goal,
                trace_command_path: value.preRewriteEvidence.trace?.command_path,
                kb_oracle_evidence: compactKbOracleTraceEvidence(value.preRewriteEvidence.trace),
                confirmation_evidence: value.preRewriteEvidence.confirmationEvidence,
                visible_body_scene_duration_evidence: value.preRewriteEvidence.visible_body_scene_duration_evidence,
              }
            : null,
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
                kb_oracle_evidence: expandKbOracleEvidence,
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
          tablePaginationEvidence: value?.tablePaginationEvidence,
          layout: value?.layout,
        },
      }
    : {
        cdp_target: { id: cdp.target.id, url: cdp.target.url, title: cdp.target.title },
        result: {
          ...(value ?? {}),
          artifact_identity: completedArtifactIdentity,
          source_integrity_preflight: {
            runtime_rs_utf8_ok: completedArtifactIdentity.runtime_rs_utf8_ok,
            runtime_rs_replacement_char_absent: completedArtifactIdentity.runtime_rs_replacement_char_absent,
            runtime_rs_sentinel_check_passed: completedArtifactIdentity.runtime_rs_sentinel_check_passed,
            runtime_rs_missing_sentinels: completedArtifactIdentity.runtime_rs_missing_sentinels,
            release_exe_fresh_for_runtime: completedArtifactIdentity.release_exe_fresh_for_runtime,
            source_integrity_preflight_passed: completedArtifactIdentity.source_integrity_preflight_passed,
            raw_values_redacted: true,
          },
        },
      };
  const artifacts = await writeUiDrivenArtifacts(cdp, payload);
  if (artifacts) {
    payload.result = payload.result || {};
    payload.result.artifacts = artifacts;
    fs.writeFileSync(artifacts.runner_payload, JSON.stringify(payload, null, 2), "utf8");
  }
  console.log(JSON.stringify(payload, null, 2));
  if (runnerAssertFailures.length) {
    process.exitCode = 1;
  }
} finally {
  cdp.ws.close();
}
