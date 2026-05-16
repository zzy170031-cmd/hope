import type { StoryboardRow } from "./types";

function findJsonCandidate(input: string): string {
  const trimmed = input.trim();
  if (!trimmed) {
    throw new Error("模型返回为空。");
  }
  if (trimmed.startsWith("{") || trimmed.startsWith("[")) {
    return trimmed;
  }

  const objectStart = trimmed.indexOf("{");
  const arrayStart = trimmed.indexOf("[");
  const start = [objectStart, arrayStart].filter((item) => item >= 0).sort((left, right) => left - right)[0];

  if (start === undefined) {
    throw new Error("模型返回中没有找到 JSON。");
  }

  const objectEnd = trimmed.lastIndexOf("}");
  const arrayEnd = trimmed.lastIndexOf("]");
  const end = Math.max(objectEnd, arrayEnd);

  if (end <= start) {
    throw new Error("模型返回中的 JSON 片段不完整。");
  }

  return trimmed.slice(start, end + 1);
}

export function parseJsonResponse<T>(input: string): T {
  const candidate = findJsonCandidate(input);
  return JSON.parse(candidate) as T;
}

export function normalizeStoryboardRows(input: unknown): StoryboardRow[] {
  if (!Array.isArray(input)) {
    throw new Error("rows 不是数组。");
  }

  return input.map((item, index) => {
    const record = (item ?? {}) as Record<string, unknown>;
    return {
      shot_index: Number(record.shot_index ?? index + 1),
      person: String(record.person ?? "").trim(),
      shot_size: String(record.shot_size ?? record.scene_scale ?? "").trim(),
      camera: String(record.camera ?? record.camera_movement ?? "").trim(),
      visual_description: String(record.visual_description ?? "").trim(),
      character_action: String(record.character_action ?? "").trim(),
      dialogue_or_narration: String(record.dialogue_or_narration ?? "").trim(),
      prompt_text: String(record.prompt_text ?? "").trim(),
      duration_seconds: Number(record.duration_seconds ?? 0),
      status: String(record.status ?? "draft").trim(),
      note: String(record.note ?? "").trim(),
    };
  });
}
