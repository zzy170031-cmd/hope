import sceneTypeEntries from "../config/scene-type-canonical.json" with { type: "json" };

export const SCENE_TYPE_OPTIONS = [...sceneTypeEntries];
export const SCENE_TYPE_CANONICAL_LIST = SCENE_TYPE_OPTIONS.map((item) => item.value);

export function sceneTypeLabelForValue(value) {
  return SCENE_TYPE_OPTIONS.find((item) => item.value === value)?.label ?? value;
}
