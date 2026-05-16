import type { ProviderDefinition, ProviderFormState, ProviderModelRecord } from "./types";

export const CANONICAL_RELEASE_PROVIDER = "qwen";
export const CANONICAL_RELEASE_MODEL = "qwen3.6-plus";
export const CANONICAL_RELEASE_BASE_URL = "https://dashscope.aliyuncs.com/compatible-mode/v1";

export const PROVIDER_DEFINITIONS: ProviderDefinition[] = [
  {
    providerId: "qwen",
    displayName: "Qwen",
    transportCapability: "browser_direct",
    defaultBaseUrl: CANONICAL_RELEASE_BASE_URL,
    endpoint: "/chat/completions",
    defaultModels: ["qwen3.6-plus", "qwen-plus", "qwen3.6-max-preview", "qwen3.6-flash"],
    statusHint: "浏览器可直接连接 DashScope 兼容接口。",
  },
  {
    providerId: "openai-compatible",
    displayName: "OpenAI-Compatible Custom",
    transportCapability: "browser_direct",
    defaultBaseUrl: "",
    endpoint: "/chat/completions",
    defaultModels: ["custom-model"],
    statusHint: "支持自定义 base_url / model，交付后可由使用者自行录入。",
  },
  {
    providerId: "deepseek",
    displayName: "DeepSeek（预留）",
    transportCapability: "unsupported",
    defaultBaseUrl: "",
    endpoint: "/chat/completions",
    defaultModels: ["deepseek-chat"],
    statusHint: "当前仅保留可见入口，尚未开放浏览器直连。",
  },
  {
    providerId: "doubao",
    displayName: "Doubao（预留）",
    transportCapability: "unsupported",
    defaultBaseUrl: "",
    endpoint: "/chat/completions",
    defaultModels: ["doubao-seed-1-6"],
    statusHint: "当前仅保留可见入口，尚未开放浏览器直连。",
  },
];

function baseUrlHost(baseUrl: string): string {
  try {
    return new URL(baseUrl).host;
  } catch {
    return "";
  }
}

export function getProviderDefinition(providerId: string): ProviderDefinition {
  return PROVIDER_DEFINITIONS.find((item) => item.providerId === providerId) ?? PROVIDER_DEFINITIONS[0];
}

export function createDefaultProviderState(): ProviderFormState {
  return {
    providerId: CANONICAL_RELEASE_PROVIDER,
    baseUrl: CANONICAL_RELEASE_BASE_URL,
    endpoint: "/chat/completions",
    apiKey: "",
    model: CANONICAL_RELEASE_MODEL,
    writingModel: CANONICAL_RELEASE_MODEL,
    directorModel: CANONICAL_RELEASE_MODEL,
    validatorModel: "",
    persistApiKey: false,
    connectionStatus: "missing_config",
    connectionMessage: "等待输入 API Key 后进行连接测试。默认只在当前会话中保存密钥。",
    corsCheckResult: "unknown",
    lastConnectionTest: null,
  };
}

export function deriveProviderModelRecord(state: ProviderFormState): ProviderModelRecord {
  const definition = getProviderDefinition(state.providerId);
  return {
    provider_id: state.providerId,
    model_id: state.model,
    display_name: `${definition.displayName} / ${state.model}`,
    base_url_host: baseUrlHost(state.baseUrl),
    endpoint: state.endpoint,
    transport_capability: definition.transportCapability,
    cors_check_result: state.corsCheckResult,
    last_connection_test: state.lastConnectionTest,
    status: state.connectionStatus,
  };
}

export function deriveConnectionStatus(state: ProviderFormState): ProviderFormState["connectionStatus"] {
  const definition = getProviderDefinition(state.providerId);
  if (definition.transportCapability === "unsupported") {
    return "unsupported";
  }
  if (!state.baseUrl.trim() || !state.apiKey.trim() || !state.model.trim()) {
    return "missing_config";
  }
  return state.connectionStatus;
}

export function applyProviderChoice(currentState: ProviderFormState, providerId: string): ProviderFormState {
  const definition = getProviderDefinition(providerId);
  const nextModel = definition.defaultModels[0] ?? "";
  const preserveCustomUrl = providerId === "openai-compatible" && currentState.providerId === "openai-compatible";

  return {
    ...currentState,
    providerId,
    baseUrl: preserveCustomUrl ? currentState.baseUrl : definition.defaultBaseUrl,
    endpoint: definition.endpoint,
    model: providerId === currentState.providerId ? currentState.model : nextModel,
    writingModel: providerId === currentState.providerId ? currentState.writingModel : nextModel,
    directorModel: providerId === currentState.providerId ? currentState.directorModel : nextModel,
    validatorModel: providerId === currentState.providerId ? currentState.validatorModel : "",
    connectionStatus: definition.transportCapability === "unsupported" ? "unsupported" : "missing_config",
    connectionMessage:
      definition.transportCapability === "unsupported"
        ? "该 Provider 目前仅预留展示，不参与浏览器直连。"
        : "请完成配置后点击“测试连接”。",
  };
}
