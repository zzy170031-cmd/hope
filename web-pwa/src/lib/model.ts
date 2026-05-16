import { buildConnectionTestMessages } from "./prompts";
import type { ModelResponse, ProviderFormState } from "./types";

interface ChatCompletionPayload {
  model: string;
  messages: Array<{ role: string; content: string }>;
  temperature?: number;
  response_format?: { type: "json_object" };
}

interface ConnectionTestResult {
  status: ProviderFormState["connectionStatus"];
  message: string;
  corsCheckResult: string;
  lastConnectionTest: string;
}

function joinUrl(baseUrl: string, endpoint: string): string {
  const base = baseUrl.replace(/\/+$/, "");
  const path = endpoint.startsWith("/") ? endpoint : `/${endpoint}`;
  return `${base}${path}`;
}

function classifyNetworkError(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  if (/Failed to fetch/i.test(message)) {
    return "浏览器 fetch 失败，通常是 CORS、网络或 DNS 问题。";
  }
  if (/abort/i.test(message) || /timed out/i.test(message)) {
    return "请求超时。";
  }
  return message;
}

export async function runChatCompletion<T>(
  provider: ProviderFormState,
  model: string,
  messages: Array<{ role: string; content: string }>,
  options: { expectJson?: boolean; timeoutMs?: number } = {},
): Promise<ModelResponse<T>> {
  const controller = new AbortController();
  const timeout = window.setTimeout(() => controller.abort(new Error("request timeout")), options.timeoutMs ?? 90000);

  try {
    const payload: ChatCompletionPayload = {
      model,
      messages,
      temperature: 0.7,
    };

    if (options.expectJson) {
      payload.response_format = { type: "json_object" };
    }

    const response = await fetch(joinUrl(provider.baseUrl, provider.endpoint), {
      method: "POST",
      signal: controller.signal,
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${provider.apiKey}`,
      },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status} ${response.statusText}`);
    }

    const json = (await response.json()) as {
      choices?: Array<{ message?: { content?: string } }>;
      usage?: unknown;
    };
    const content = json.choices?.[0]?.message?.content?.trim() ?? "";

    if (!content) {
      throw new Error("模型返回空内容。");
    }

    return {
      content,
      parsed: content as T,
      rawUsage: json.usage,
    };
  } catch (error) {
    throw new Error(classifyNetworkError(error));
  } finally {
    window.clearTimeout(timeout);
  }
}

export async function testProviderConnection(provider: ProviderFormState): Promise<ConnectionTestResult> {
  const lastConnectionTest = new Date().toISOString();

  if (!provider.baseUrl.trim() || !provider.apiKey.trim() || !provider.model.trim()) {
    return {
      status: "missing_config",
      message: "缺少 base_url、model 或 API Key。",
      corsCheckResult: "skipped_missing_config",
      lastConnectionTest,
    };
  }

  try {
    const response = await runChatCompletion<string>(
      provider,
      provider.model,
      buildConnectionTestMessages(provider.model),
      { timeoutMs: 30000 },
    );
    const reachable = /reachable/i.test(response.content);

    return {
      status: reachable ? "ready" : "failed",
      message: reachable ? "连接成功，浏览器直连可用。" : `连接返回了异常内容：${response.content.slice(0, 80)}`,
      corsCheckResult: "passed",
      lastConnectionTest,
    };
  } catch (error) {
    return {
      status: "failed",
      message: error instanceof Error ? error.message : String(error),
      corsCheckResult: "failed",
      lastConnectionTest,
    };
  }
}
