import { PROVIDER_CONFIG, printJson } from "./web-runner-lib.mjs";

const canonical = {
  provider: "qwen",
  model: "qwen3.6-plus",
  base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
  endpoint: "/chat/completions",
  transport_capability: "browser_direct",
  canonical_release: true,
};

async function main() {
  const result = {
    canonical_release_model: canonical,
    runtime_selection: {
      provider: PROVIDER_CONFIG.provider,
      model: PROVIDER_CONFIG.model,
      base_url: PROVIDER_CONFIG.baseUrl,
      endpoint: PROVIDER_CONFIG.endpoint,
      api_key_present: Boolean(PROVIDER_CONFIG.apiKey),
    },
    supported_rules: [
      "qwen3.6-plus is canonical only after targeted/full16/formal403 live evidence",
      "openai-compatible custom may be marked supported after connection + targeted + full16",
      "deepseek and doubao remain reserved until browser direct verification is completed",
    ],
  };
  printJson(result);
}

main().catch((error) => {
  printJson({
    ok: false,
    error: error instanceof Error ? error.message : String(error),
  });
  process.exitCode = 1;
});
