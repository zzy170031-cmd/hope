import sceneTypeEntries from "../../config/scene-type-canonical.json";

export interface SceneTypeOption {
  value: string;
  label: string;
  family: string;
}

export const SCENE_TYPE_OPTIONS = [...sceneTypeEntries] as SceneTypeOption[];
export const SCENE_TYPE_CANONICAL_LIST = SCENE_TYPE_OPTIONS.map((item) => item.value);

export function isCanonicalSceneType(sceneType: string): boolean {
  return SCENE_TYPE_CANONICAL_LIST.includes(sceneType);
}

export function getSceneTypeOption(sceneType: string): SceneTypeOption {
  return SCENE_TYPE_OPTIONS.find((item) => item.value === sceneType) ?? SCENE_TYPE_OPTIONS[0];
}

export function getSceneTypeLabel(sceneType: string): string {
  return getSceneTypeOption(sceneType).label;
}
