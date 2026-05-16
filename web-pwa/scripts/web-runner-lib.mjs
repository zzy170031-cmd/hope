import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { spawn } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { SCENE_TYPE_CANONICAL_LIST, SCENE_TYPE_OPTIONS, sceneTypeLabelForValue } from "./scene-types.mjs";

export { SCENE_TYPE_CANONICAL_LIST, SCENE_TYPE_OPTIONS, sceneTypeLabelForValue };

export const APP_URL = process.env.HOPE_WEB_URL ?? "http://127.0.0.1:4174/";

export const PROVIDER_CONFIG = {
  provider: process.env.HOPE_WEB_PROVIDER ?? "qwen",
  baseUrl: process.env.HOPE_WEB_BASE_URL ?? "https://dashscope.aliyuncs.com/compatible-mode/v1",
  endpoint: process.env.HOPE_WEB_ENDPOINT ?? "/chat/completions",
  model: process.env.HOPE_WEB_MODEL ?? "qwen3.6-plus",
  writingModel: process.env.HOPE_WEB_WRITING_MODEL ?? process.env.HOPE_WEB_MODEL ?? "qwen3.6-plus",
  directorModel: process.env.HOPE_WEB_DIRECTOR_MODEL ?? process.env.HOPE_WEB_MODEL ?? "qwen3.6-plus",
  apiKey: process.env.HOPE_WEB_API_KEY ?? process.env.HOPE_TEXT_MODEL_API_KEY ?? "",
};

export const TARGETED_CASES = [
  {
    caseId: "targeted-01-expand-hot-blood",
    sceneType: "hot_blood_battle",
    duration: 15,
    mode: "expand_story",
    sourceText:
      "暴雨夜里，林峯护着苏瑶穿过旧城戏台后的狭窄通道。阿青在后方提醒追兵已经逼近巷口，林峯必须在极短时间内把证物交到苏瑶手中，同时决定是继续撤离还是转身断后。",
  },
  {
    caseId: "targeted-02-rewrite-dialogue",
    sceneType: "emotional_dialogue",
    duration: 10,
    mode: "rewrite_script",
    sourceText:
      "天台风很大，林峯把证物压在手心里不肯松开。苏瑶明知继续争执会错过撤离时机，却还是逼他讲出隐藏真相，阿青在楼梯口压低声音催两人立刻离开。",
  },
  {
    caseId: "targeted-03-strong-diff-battle-report",
    sceneType: "slg_battle_report",
    duration: 30,
    mode: "expand_story",
    sourceText:
      "主城外的围攻刚结束，指挥官必须在战报界面里判断损耗、增援与下一轮推进路线，同时确认前线回传的关键信息没有遗漏。",
  },
  {
    caseId: "targeted-04-manual-edit-export",
    sceneType: "urban_fantasy",
    duration: 15,
    mode: "expand_story",
    sourceText:
      "霓虹倒映在积水路面上，林峯把刚取回的底片递给苏瑶。阿青回头看见黑衣追兵逼近，三人必须在数秒内完成交接并冲出巷口。",
    manualEdit: {
      rowIndex: 0,
      field: "visual_description",
      append: " 近景前景带出湿墙反光与追兵逼近的压迫感。",
    },
  },
];

export const FULL16_CASES = SCENE_TYPE_CANONICAL_LIST.slice(0, 16).map((sceneType, index) => ({
  caseId: `full16-${String(index + 1).padStart(2, "0")}`,
  sceneType,
  duration: [5, 10, 15, 30, 45, 60][index % 6],
  mode: index < 8 ? "expand_story" : "rewrite_script",
  sourceText: `${sceneTypeLabelForValue(sceneType)}样例：人物必须在有限时长内处理冲突、空间变化和关系推进，确保后续分镜可以拆解为连续、可见、可导出的镜头。`,
  manualEdit: index === 7 || index === 15
    ? {
        rowIndex: 0,
        field: "prompt_text",
        append: " 补充前景动作与镜头节奏约束。",
      }
    : undefined,
}));

export function parseCli(argv = process.argv.slice(2)) {
  const flags = new Set();
  const values = new Map();
  for (let index = 0; index < argv.length; index += 1) {
    const item = argv[index];
    if (!item.startsWith("--")) {
      continue;
    }
    const key = item.slice(2);
    const next = argv[index + 1];
    if (!next || next.startsWith("--")) {
      flags.add(key);
      continue;
    }
    values.set(key, next);
    index += 1;
  }
  return { flags, values };
}

export function requireLiveApiKey() {
  if (!PROVIDER_CONFIG.apiKey) {
    throw new Error("Missing HOPE_WEB_API_KEY or HOPE_TEXT_MODEL_API_KEY for live runs.");
  }
}

export async function ensureUrlReady(url, timeoutMs = 45000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return true;
      }
    } catch {
      // retry
    }
    await sleep(500);
  }
  throw new Error(`App did not become ready: ${url}`);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function waitForChildExit(child, timeoutMs = 15000) {
  return new Promise((resolve) => {
    if (!child?.pid) {
      resolve(false);
      return;
    }
    let settled = false;
    const finish = (value) => {
      if (settled) {
        return;
      }
      settled = true;
      resolve(value);
    };
    child.once("exit", () => finish(true));
    setTimeout(() => finish(false), timeoutMs);
  });
}

async function killProcessTree(child) {
  if (!child?.pid) {
    return false;
  }
  try {
    child.kill();
  } catch {
    // ignore and fall back to taskkill
  }
  const exited = await waitForChildExit(child, 5000);
  if (exited) {
    return true;
  }
  await new Promise((resolve) => {
    const killer = spawn("taskkill", ["/PID", String(child.pid), "/T", "/F"], { stdio: "ignore" });
    killer.once("exit", () => resolve());
    killer.once("error", () => resolve());
  });
  return waitForChildExit(child, 10000);
}

async function removeDirWithRetries(targetDir, attempts = 10, waitMs = 500) {
  let lastError = null;
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      await rm(targetDir, { recursive: true, force: true });
      return true;
    } catch (error) {
      lastError = error;
      await sleep(waitMs);
    }
  }
  if (lastError) {
    throw lastError;
  }
  return false;
}

function timestampToken() {
  const now = new Date();
  const pad = (value) => String(value).padStart(2, "0");
  return `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}-${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`;
}

export async function prepareArtifactRoot(label, explicitRoot) {
  const baseRoot = explicitRoot
    ? path.join(path.resolve(explicitRoot), `${label}-fresh-${timestampToken()}`)
    : path.join(process.cwd(), ".codex-run", `${label}-fresh-${timestampToken()}`);
  await mkdir(baseRoot, { recursive: true });
  return baseRoot;
}

function chromeCandidates() {
  return [
    process.env.HOPE_WEB_CHROME_PATH,
    "C:/Program Files/Google/Chrome/Application/chrome.exe",
    "C:/Program Files (x86)/Google/Chrome/Application/chrome.exe",
    path.join(process.env.LOCALAPPDATA ?? "", "Google/Chrome/Application/chrome.exe"),
    "C:/Program Files/Microsoft/Edge/Application/msedge.exe",
  ].filter(Boolean);
}

async function findChromeExecutable() {
  const { access } = await import("node:fs/promises");
  for (const candidate of chromeCandidates()) {
    try {
      await access(candidate);
      return candidate;
    } catch {
      // keep searching
    }
  }
  throw new Error("Chrome or Edge executable not found. Set HOPE_WEB_CHROME_PATH.");
}

async function fetchJson(url) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status} for ${url}`);
  }
  return response.json();
}

async function createChromeSession(url, port) {
  const executable = await findChromeExecutable();
  const userDataDir = await mkdtemp(path.join(os.tmpdir(), "hope-web-pwa-"));
  const child = spawn(
    executable,
    [
      `--remote-debugging-port=${port}`,
      "--no-first-run",
      "--no-default-browser-check",
      `--user-data-dir=${userDataDir}`,
      "--new-window",
      url,
    ],
    { stdio: "ignore" },
  );
  return { child, userDataDir };
}

async function connectToCdp(url, port, timeoutMs = 45000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    try {
      const targets = await fetchJson(`http://127.0.0.1:${port}/json/list`);
      const target = targets.find((item) => String(item.url).startsWith(url));
      if (target?.webSocketDebuggerUrl) {
        const ws = new WebSocket(target.webSocketDebuggerUrl);
        await new Promise((resolve, reject) => {
          ws.addEventListener("open", resolve, { once: true });
          ws.addEventListener("error", reject, { once: true });
        });
        let nextId = 1;
        const pending = new Map();
        ws.addEventListener("message", (event) => {
          const message = JSON.parse(String(event.data));
          if (!message.id || !pending.has(message.id)) {
            return;
          }
          const { resolve, reject, timer } = pending.get(message.id);
          clearTimeout(timer);
          pending.delete(message.id);
          if (message.error) {
            reject(new Error(message.error.message));
          } else {
            resolve(message.result);
          }
        });
        const send = (method, params = {}) => {
          const id = nextId += 1;
          ws.send(JSON.stringify({ id, method, params }));
          return new Promise((resolve, reject) => {
            const timer = setTimeout(() => {
              pending.delete(id);
              reject(new Error(`CDP timeout: ${method}`));
            }, 600000);
            pending.set(id, { resolve, reject, timer });
          });
        };
        await send("Runtime.enable");
        await send("Page.enable");
        return { ws, send };
      }
    } catch {
      // retry
    }
    await sleep(500);
  }
  throw new Error(`CDP target not ready on port ${port}`);
}

function appWorkflowExpression(caseDef) {
  return `
  const input = ${JSON.stringify(caseDef)};
  const provider = ${JSON.stringify(PROVIDER_CONFIG)};
  (async () => {
    const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
    const q = (id) => document.querySelector('[data-testid="' + id + '"]');
    const text = (selector) => document.querySelector(selector)?.textContent?.replace(/\\s+/g, ' ').trim() ?? '';
    const bodyText = () => document.body.innerText.replace(/\\s+/g, ' ').trim();
    const setValue = (element, value) => {
      const proto =
        element instanceof HTMLTextAreaElement
          ? HTMLTextAreaElement.prototype
          : element instanceof HTMLSelectElement
            ? HTMLSelectElement.prototype
            : HTMLInputElement.prototype;
      const setter = Object.getOwnPropertyDescriptor(proto, 'value')?.set;
      setter?.call(element, value);
      element.dispatchEvent(new Event('input', { bubbles: true }));
      element.dispatchEvent(new Event('change', { bubbles: true }));
    };
    const waitFor = async (fn, label, timeout = 180000) => {
      const started = Date.now();
      let last = null;
      while (Date.now() - started < timeout) {
        try {
          const result = await fn();
          if (result) {
            return result;
          }
          last = result;
        } catch (error) {
          last = error instanceof Error ? error.message : String(error);
        }
        await sleep(300);
      }
      throw new Error('waitFor timeout: ' + label + '; last=' + String(last ?? ''));
    };
    const click = async (id) => {
      const element = q(id);
      if (!element) {
        throw new Error('missing element: ' + id);
      }
      if (element.disabled) {
        throw new Error('disabled element: ' + id);
      }
      element.click();
      await sleep(250);
    };
    const clickText = async (label) => {
      const element = Array.from(document.querySelectorAll('button, a')).find((item) =>
        item.textContent?.replace(/\\s+/g, ' ').trim() === label,
      );
      if (!element) {
        throw new Error('missing clickable text: ' + label);
      }
      element.click();
      await sleep(250);
    };
    const clickApiInterface = async () => {
      const element = Array.from(document.querySelectorAll('button')).find((item) => {
        const textValue = item.textContent?.replace(/\\s+/g, '').trim() ?? '';
        return textValue.includes('API接口') || (textValue.includes('API') && !textValue.includes('文档'));
      });
      if (!element) {
        throw new Error('missing API interface button');
      }
      element.click();
      await sleep(250);
    };
    const tableRows = () => Array.from(document.querySelectorAll('table tbody tr'));
    const pickField = (rowIndex, cellIndex, tag) => tableRows()[rowIndex]?.querySelectorAll(tag)[cellIndex] ?? null;
    const initialDownloads = (window.__hopeLastDownloads ?? []).length;

    if (input.kind === 'entry') {
      const entryResult = {
        ok: true,
        case_id: input.caseId,
        group: 'entry',
        scene_type: '',
        scene_type_label: '',
        duration: 0,
        script_goal: 'entry_baseline',
        description: input.description,
        trace: window.__hopeQaTrace ?? {},
        ui_evidence: {
          page_loaded: Boolean(document.body),
          provider_modal_ready: Boolean(document.body.innerText.includes('API')),
          scene_selector_ready: Boolean(q('scene-type-select')),
          duration_selector_ready: Boolean(q('duration-select')),
          export_storyboard_visible: Boolean(q('export-storyboard')),
          export_script_visible: Boolean(q('export-script')),
          export_storyboard_disabled: Boolean(q('export-storyboard')?.disabled),
          export_script_disabled: Boolean(q('export-script')?.disabled),
        },
        downloads: [],
      };
      return entryResult;
    }

    await waitFor(
      () => document.readyState === 'complete' && document.body && document.body.innerText.replace(/\\s+/g, '').length > 20,
      'app hydrated',
      60000,
    );

    if (!q('provider-select')) {
      await clickApiInterface();
    }
    await waitFor(() => q('provider-select'), 'provider select ready', 30000);
    setValue(q('provider-select'), provider.provider);
    setValue(q('base-url-input'), provider.baseUrl);
    setValue(q('endpoint-input'), provider.endpoint);
    setValue(q('model-input'), provider.model);
    setValue(q('writing-model-select'), provider.writingModel);
    setValue(q('director-model-select'), provider.directorModel);
    setValue(q('api-key-input'), provider.apiKey);
    await click('test-connection-button');
    await waitFor(() => {
      const body = bodyText();
      if (body.includes('连接成功，浏览器直连可用。')) return true;
      if (body.includes('HTTP 401') || body.includes('HTTP 403') || body.includes('Failed to fetch') || body.includes('请求超时')) {
        throw new Error(body.slice(0, 400));
      }
      return false;
    }, 'provider ready', 120000);
    await clickText('关闭');

    const prechecks = {
      generate_disabled_before_task: Boolean(q('generate-storyboard-button')?.disabled),
      export_storyboard_disabled_initial: Boolean(q('export-storyboard')?.disabled),
      export_script_disabled_initial: Boolean(q('export-script')?.disabled),
    };

    setValue(q('scene-type-select'), input.sceneType);
    setValue(q('duration-select'), String(input.duration));
    setValue(q('source-textarea'), input.sourceText);

    if (input.mode === 'rewrite_script') {
      await click('rewrite-button');
    } else {
      await click('expand-button');
    }

    await waitFor(() => {
      const value = String(q('body-textarea')?.value ?? '');
      if (value.trim().length > 120) return value.trim();
      const pageText = bodyText();
      if (pageText.includes('正文生成失败')) {
        throw new Error(pageText.slice(0, 400));
      }
      return false;
    }, 'body generated', 240000);

    if (input.mode === 'storyboard_from_body') {
      await click('expand-button');
      await waitFor(() => String(q('body-textarea')?.value ?? '').trim().length > 120, 'body regenerated', 240000);
    }

    await click('accept-body-button');
    await click('create-task-button');
    await waitFor(() => q('task-create-confirm-button'), 'task create modal ready', 30000);
    if (q('task-name-input')) {
      setValue(q('task-name-input'), 'QA-' + input.caseId);
    }
    await click('task-create-confirm-button');

    const traceAfterTask = window.__hopeQaTrace ?? {};
    if (!traceAfterTask.storyboard_task_hash) {
      throw new Error('storyboard task hash missing after task creation');
    }

    await click('generate-storyboard-button');
    await waitFor(() => {
      const trace = window.__hopeQaTrace;
      if (
        trace?.rows_count >= 3 &&
        trace?.rows_match &&
        trace?.prompt_text_present &&
        trace?.visual_description_visible_frame_passed
      ) {
        return trace;
      }
      const pageText = bodyText();
      if (pageText.includes('分镜生成失败')) {
        throw new Error(pageText.slice(0, 500));
      }
      return false;
    }, 'storyboard generated', 300000);

    let editBlocked = null;
    if (input.manualEdit) {
      const target = pickField(input.manualEdit.rowIndex, 0, 'textarea');
      if (!target) {
        throw new Error('manual edit target missing');
      }
      setValue(target, String(target.value || '') + input.manualEdit.append);
      await sleep(250);
      editBlocked = {
        task_sync_after_edit: window.__hopeQaTrace?.storyboard_task_sync_status ?? '',
        export_storyboard_disabled_after_edit: Boolean(q('export-storyboard')?.disabled),
        export_script_disabled_after_edit: Boolean(q('export-script')?.disabled),
      };
      if (q('confirm-current-shot-button')) {
        await click('confirm-current-shot-button');
      } else {
        await clickText('保存修改');
      }
      await waitFor(() => window.__hopeQaTrace?.storyboard_task_sync_status === 'fresh', 'confirm current shot', 120000);
    }

    if (q('confirm-current-shot-button') && !q('confirm-current-shot-button').disabled) {
      await click('confirm-current-shot-button');
    }
    await waitFor(
      () => q('export-storyboard') && q('export-script') && !q('export-storyboard').disabled && !q('export-script').disabled,
      'exports enabled',
      120000,
    );

    await click('export-storyboard');
    await click('export-script');
    await waitFor(() => (window.__hopeLastDownloads ?? []).length >= initialDownloads + 2, 'downloads ready', 30000);

    const downloads = (window.__hopeLastDownloads ?? []).slice(-2);
    const trace = window.__hopeQaTrace ?? {};
    const rowsPreview = tableRows().slice(0, 3).map((row) => row.textContent?.replace(/\\s+/g, ' ').trim() ?? '');
    const rowPrompts = Array.from(document.querySelectorAll('table tbody textarea'))
      .map((item) => item.value)
      .filter((_, index) => index % 4 === 3)
      .slice(0, 3);
    const rowVisuals = Array.from(document.querySelectorAll('table tbody textarea'))
      .map((item) => item.value)
      .filter((_, index) => index % 4 === 0)
      .slice(0, 3);

    return {
      ok: true,
      case_id: input.caseId,
      scene_type: input.sceneType,
      scene_type_label: ${JSON.stringify(SCENE_TYPE_OPTIONS)}.find((item) => item.value === input.sceneType)?.label ?? input.sceneType,
      duration: input.duration,
      script_goal: input.mode,
      prechecks,
      edit_blocked: editBlocked,
      export_notice: text('.export-notice'),
      task_status: text('[data-testid="task-status-text"]'),
      rows_preview: rowsPreview,
      prompt_text_preview: rowPrompts,
      visual_description_preview: rowVisuals,
      downloads,
      trace,
      ui_evidence: {
        accepted_body_text_length: String(q('body-textarea')?.value ?? '').trim().length,
        source_text_length: String(q('source-textarea')?.value ?? '').trim().length,
        table_row_count: tableRows().length,
      },
    };
  })().catch((error) => ({
    ok: false,
    case_id: input.caseId,
    scene_type: input.sceneType,
    duration: input.duration,
    script_goal: input.mode,
    error: error instanceof Error ? error.message : String(error),
    trace: window.__hopeQaTrace ?? null,
    body_text: document.body.innerText.slice(0, 2000),
    downloads: window.__hopeLastDownloads ?? [],
    ui_evidence: {
      export_notice: document.querySelector('.export-notice')?.textContent ?? '',
        task_status: document.querySelector('[data-testid="task-status-text"]')?.textContent ?? '',
    },
  }));
  `;
}

async function evaluate(send, expression) {
  const result = await send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  if (result.exceptionDetails) {
    throw new Error(result.exceptionDetails.text ?? "Runtime evaluation failed.");
  }
  return result.result?.value;
}

function sanitizeDownloads(downloads = []) {
  return downloads.map((item) => ({
    file_name: item.fileName,
    kind: item.kind,
    format: item.format,
    contains_secret_leakage: Boolean(item.contains_secret_leakage),
    contains_local_path: Boolean(item.contains_local_path),
    contains_raw_kb: Boolean(item.contains_raw_kb),
    usable: Boolean(item.usable),
    sampled_export_preview_present: Boolean(item.sampled_export_preview_present),
    preview: Array.isArray(item.preview) ? item.preview : [],
  }));
}

export async function runWebWorkflowCase(caseDef, options = {}) {
  requireLiveApiKey();
  const appUrl = options.appUrl ?? APP_URL;
  const port = options.port ?? 9333;
  const artifactDir = options.artifactDir ?? path.join(process.cwd(), ".codex-run");

  await ensureUrlReady(appUrl);
  const { child, userDataDir } = await createChromeSession(appUrl, port);
  let connection;
  let cleanup = { port, userDataDirRemoved: false, browserClosed: false };
  try {
    connection = await connectToCdp(appUrl, port);
    let rawResult;
    try {
      rawResult = await evaluate(connection.send, appWorkflowExpression(caseDef));
    } catch (error) {
      if (!/Execution context was destroyed/i.test(String(error))) {
        throw error;
      }
      await sleep(1500);
      rawResult = await evaluate(connection.send, appWorkflowExpression(caseDef));
    }
    const downloads = sanitizeDownloads(rawResult.downloads ?? []);
    const result = {
      ...rawResult,
      downloads,
      provider: PROVIDER_CONFIG.provider,
      provider_model: PROVIDER_CONFIG.model,
      no_http_403: !/HTTP 403/i.test(rawResult.error ?? ""),
      fallback_used: rawResult.trace?.fallback_used ?? false,
      local_candidate: rawResult.trace?.local_candidate ?? false,
      artifact_dir: artifactDir,
      runner_isolation: cleanup,
    };
    return result;
  } finally {
    try {
      connection?.ws?.close();
    } catch {
      // ignore
    }
    try {
      if (child?.pid) {
        cleanup = { ...cleanup, browserClosed: await killProcessTree(child) };
      }
    } catch {
      // ignore
    }
    cleanup = { ...cleanup, userDataDirRemoved: await removeDirWithRetries(userDataDir) };
  }
}

export async function writeCaseArtifact(artifactDir, caseId, payload) {
  await mkdir(artifactDir, { recursive: true });
  const target = path.join(artifactDir, `${caseId}.json`);
  await writeFile(target, JSON.stringify(payload, null, 2), "utf8");
  return target;
}

export async function writeSummaryArtifact(artifactDir, fileName, payload) {
  await mkdir(artifactDir, { recursive: true });
  const target = path.join(artifactDir, fileName);
  await writeFile(target, JSON.stringify(payload, null, 2), "utf8");
  return target;
}

export function summarizeResults(results, totalExpected = results.length) {
  const passed = results.filter((item) => item.ok).length;
  const failed = results.filter((item) => !item.ok).length;
  return {
    expected: totalExpected,
    executed: results.length,
    passed,
    failed,
    not_run: Math.max(0, totalExpected - results.length),
    case_ids: results.map((item) => item.case_id),
  };
}

export async function readJsonFile(filePath) {
  return JSON.parse(await readFile(filePath, "utf8"));
}

