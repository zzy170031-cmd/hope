import { describe, expect, it } from "vitest";
import { applyProviderChoice, createDefaultProviderState, deriveProviderModelRecord } from "./providers";

describe("provider registry", () => {
  it("preserves canonical qwen defaults", () => {
    const state = createDefaultProviderState();
    const record = deriveProviderModelRecord(state);
    expect(record.provider_id).toBe("qwen");
    expect(record.model_id).toBe("qwen3.6-plus");
  });

  it("marks reserved providers as unsupported", () => {
    const next = applyProviderChoice(createDefaultProviderState(), "deepseek");
    expect(next.connectionStatus).toBe("unsupported");
  });
});
