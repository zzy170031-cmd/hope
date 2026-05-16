import { createDefaultProviderState } from "./providers";
import type { ProviderFormState } from "./types";

const PROVIDER_KEY = "hope-web-pwa.provider-config";
const API_KEY_KEY = "hope-web-pwa.provider-api-key";

type PersistedProviderState = Omit<ProviderFormState, "apiKey">;

export function loadProviderState(): ProviderFormState {
  const fallback = createDefaultProviderState();
  try {
    const raw = localStorage.getItem(PROVIDER_KEY);
    const parsed = raw ? (JSON.parse(raw) as Partial<PersistedProviderState>) : {};
    const apiKey = parsed.persistApiKey
      ? localStorage.getItem(API_KEY_KEY) ?? ""
      : sessionStorage.getItem(API_KEY_KEY) ?? "";

    return {
      ...fallback,
      ...parsed,
      apiKey,
    };
  } catch {
    return fallback;
  }
}

export function saveProviderState(state: ProviderFormState): void {
  const { apiKey: _apiKey, ...payload } = state;
  localStorage.setItem(PROVIDER_KEY, JSON.stringify(payload));

  if (state.apiKey) {
    sessionStorage.setItem(API_KEY_KEY, state.apiKey);
  } else {
    sessionStorage.removeItem(API_KEY_KEY);
  }

  if (state.persistApiKey && state.apiKey) {
    localStorage.setItem(API_KEY_KEY, state.apiKey);
  } else {
    localStorage.removeItem(API_KEY_KEY);
  }
}
