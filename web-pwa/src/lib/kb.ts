import { sha256Hex, stableStringify } from "./hash";
import { getSceneTypeLabel, getSceneTypeOption, SCENE_TYPE_CANONICAL_LIST } from "./sceneTypes";
import type { DurationCapacityProfile, KbSnapshot, RulePack, SanitizedKbSummary } from "./types";

const writingRulePacks: RulePack[] = [
  {
    id: "wg-complete-body",
    name: "正文完整性",
    intent: "确保正文是用户可确认的完整叙事，而不是分镜清单或规则转述。",
    influenceAxes: ["body_completeness", "event_progression", "character_grounding"],
    directives: [
      "正文必须包含人物、空间、冲突推进和阶段落点。",
      "优先输出连续故事文本，不要写成镜头表或提示词。",
      "保留源文本里的核心人物、关系、地点和事件顺序。",
    ],
    negativeConstraints: ["不要输出 prompt 结构", "不要输出 raw KB 行", "不要输出内部控制字段"],
  },
  {
    id: "wg-scene-adaptation",
    name: "场景表达适配",
    intent: "让正文在不改事实源的前提下，体现当前 scene_type 的节奏与空间气质。",
    influenceAxes: ["scene_tone", "world_texture", "pace_control"],
    directives: [
      "scene_type 只控制表达结构，不改变人物、地点和主事件。",
      "要把当前场景的情绪密度、行动方式和空间感写进正文。",
      "target_duration_seconds 要反映成叙事容量，而不是机械字数。",
    ],
    negativeConstraints: ["不要把 scene_type 当作人物或地点", "不要引入源文本之外的新世界观事实"],
  },
  {
    id: "wg-dialogue-discipline",
    name: "对白节制",
    intent: "在动作、对白和叙述之间保持可拆镜的节拍。",
    influenceAxes: ["dialogue_balance", "narration_precision", "beat_density"],
    directives: [
      "对白只在必要时出现，不能整段堆对白。",
      "每段推进一个主要动作或情绪节点。",
      "保证后续导演组可以从正文拆出连续可见镜头。",
    ],
    negativeConstraints: ["不要把对白写成说明书", "不要整段抽象抒情代替动作"],
  },
];

const directorRulePacks: RulePack[] = [
  {
    id: "dg-visible-frame",
    name: "当前镜头可见帧",
    intent: "每条分镜都必须落到当前镜头真的能看到的内容。",
    influenceAxes: ["visible_subject", "frame_composition", "light_material"],
    directives: [
      "visual_description 只能描述当前镜头里可见的人、动作、空间和冲突。",
      "禁止把抽象分析、KB 规则名或 trace 内容写进画面描述。",
      "人物字段只能来自 accepted body 中真实存在的名字或角色标签。",
    ],
    negativeConstraints: ["不要输出抽象词代替画面", "不要把镜头、构图、UI 等当作 person"],
  },
  {
    id: "dg-prompt-compilation",
    name: "提示词编译",
    intent: "把 accepted body、场景规则和导演意图编译成可执行 prompt_text。",
    influenceAxes: ["subject_action_sync", "camera_language", "negative_constraints"],
    directives: [
      "prompt_text 必须包含事实源、主体、动作、画面、运镜、时长节奏、场景谱系和负面约束。",
      "prompt_text 必须和 visual_description 对齐，不能在最后一刻脱离 accepted body。",
      "导演规则要体现在 prompt_text 中，但不能把规则原文伪装成最终画面。",
    ],
    negativeConstraints: ["不要输出空 prompt_text", "不要遗漏负面约束", "不要泄漏 raw sample_text"],
  },
  {
    id: "dg-duration-pacing",
    name: "时长节奏分配",
    intent: "按 5/10/15/30/45/60 六档容量分配镜头数与节拍。",
    influenceAxes: ["row_count", "duration_distribution", "beat_focus"],
    directives: [
      "镜头时长总和必须严格等于 target_duration_seconds。",
      "每条镜头都要承载一个可见动作、情绪推进或空间信息。",
      "不允许靠空行、零时长或占位镜头凑数。",
    ],
    negativeConstraints: ["不要 rows=0 也当成功", "不要输出零时长镜头"],
  },
];

const durationProfiles: DurationCapacityProfile[] = [
  { duration: 5, targetShotCount: 3, beatDensity: "high", pacingDirective: "三镜内完成冲突起落与钩子落点。" },
  { duration: 10, targetShotCount: 4, beatDensity: "high", pacingDirective: "四镜内明确空间、人物关系和主冲突推进。" },
  { duration: 15, targetShotCount: 5, beatDensity: "medium", pacingDirective: "五镜形成完整小节拍，允许 10 + 5 的层次分配。" },
  { duration: 30, targetShotCount: 7, beatDensity: "medium", pacingDirective: "七镜内保留一组呼吸镜头，但主冲突持续可见。" },
  { duration: 45, targetShotCount: 9, beatDensity: "layered", pacingDirective: "九镜兼顾主线与副动作，允许空间调度与视线转换。" },
  { duration: 60, targetShotCount: 12, beatDensity: "layered", pacingDirective: "十二镜完成完整段落，允许多层推进但不能失焦。" },
];

function resolveWritingRulePackIds(sceneType: string): string[] {
  const family = getSceneTypeOption(sceneType).family;
  if (family === "conflict") {
    return ["wg-complete-body", "wg-scene-adaptation"];
  }
  if (family === "epic") {
    return ["wg-complete-body", "wg-scene-adaptation", "wg-dialogue-discipline"];
  }
  return ["wg-complete-body", "wg-dialogue-discipline", "wg-scene-adaptation"];
}

function resolveDirectorRulePackIds(sceneType: string): string[] {
  const family = getSceneTypeOption(sceneType).family;
  if (family === "performance") {
    return ["dg-visible-frame", "dg-prompt-compilation"];
  }
  return ["dg-visible-frame", "dg-prompt-compilation", "dg-duration-pacing"];
}

function resolveInfluenceAxes(writingRulePackIds: string[], directorRulePackIds: string[]): string[] {
  return Array.from(
    new Set(
      [...writingRulePackIds, ...directorRulePackIds].flatMap((id) => {
        const pack = [...writingRulePacks, ...directorRulePacks].find((item) => item.id === id);
        return pack?.influenceAxes ?? [];
      }),
    ),
  );
}

function resolveSceneProfile(sceneType: string): string {
  const option = getSceneTypeOption(sceneType);
  switch (option.family) {
    case "conflict":
      return `${option.label}要求画面持续可见冲突压力、位移关系和动作落点，不能只剩抽象紧张感。`;
    case "performance":
      return `${option.label}要求人物关系、情绪推力和表演细节同时可见，镜头要服务于角色互动。`;
    case "spectacle":
      return `${option.label}要求奇观、空间层次和视觉焦点清楚落地，不能退化为泛环境描写。`;
    case "epic":
      return `${option.label}要求秩序关系、阵线变化和宏观推进具备可视化组织。`;
    default:
      return `${option.label}要求正文与分镜都建立清晰的空间、情绪和动作推进。`;
  }
}

function resolveNegativeConstraints(sceneType: string): string[] {
  const option = getSceneTypeOption(sceneType);
  const shared = [
    "禁止泄漏 API Key、raw KB rows、source_register、overlay JSON、prompt_body",
    "禁止把人物、地点、KB、UI、构图词当作 scene_type 或 person",
    "禁止把旧 rows、旧 task、旧 accepted body 当作当前事实源",
  ];
  if (option.family === "epic") {
    return [...shared, "禁止空泛宏大叙述掩盖当前镜头应见的具体动作"];
  }
  if (option.family === "performance") {
    return [...shared, "禁止只写情绪标签而不写可见表演动作"];
  }
  return [...shared, "禁止抽象词替代当前镜头可见画面"];
}

function resolveSelectedSampleIds(sceneType: string): string[] {
  return [`sample-${sceneType}-A`, `sample-${sceneType}-B`, `sample-${sceneType}-C`];
}

function resolveSelectedKbRules(sceneType: string): string[] {
  const option = getSceneTypeOption(sceneType);
  return [`scene-profile:${sceneType}`, `family:${option.family}`, `label:${option.label}`];
}

function buildKbContextSummary(sceneType: string, sceneProfile: string, durationProfile: DurationCapacityProfile): string {
  const option = getSceneTypeOption(sceneType);
  return [
    `${option.label} 的写作组会先校正文正文完整性与当前 scene_type 表达适配。`,
    `${option.label} 的导演组会根据当前镜头可见帧、提示词编译和时长节奏规则生成分镜。`,
    `本次时长档位为 ${durationProfile.duration} 秒，建议镜头数 ${durationProfile.targetShotCount} 条，节奏要求：${durationProfile.pacingDirective}`,
    `scene profile：${sceneProfile}`,
  ].join(" ");
}

const sceneMappings = Object.fromEntries(
  SCENE_TYPE_CANONICAL_LIST.map((sceneType) => {
    const writingRulePackIds = resolveWritingRulePackIds(sceneType);
    const directorRulePackIds = resolveDirectorRulePackIds(sceneType);
    const sceneProfile = resolveSceneProfile(sceneType);
    return [
      sceneType,
      {
        writingRulePackIds,
        directorRulePackIds,
        selectedSampleIds: resolveSelectedSampleIds(sceneType),
        selectedKbRules: resolveSelectedKbRules(sceneType),
        sceneProfile,
        negativeConstraints: resolveNegativeConstraints(sceneType),
        influenceAxes: resolveInfluenceAxes(writingRulePackIds, directorRulePackIds),
      },
    ];
  }),
) as KbSnapshot["sceneMappings"] &
  Record<
    string,
    {
      writingRulePackIds: string[];
      directorRulePackIds: string[];
      selectedSampleIds: string[];
      selectedKbRules: string[];
      sceneProfile: string;
      negativeConstraints: string[];
      influenceAxes: string[];
    }
  >;

export const KB_SNAPSHOT: KbSnapshot = {
  snapshotVersion: "hope-web-kb-snapshot-v3",
  sceneTypes: [...SCENE_TYPE_CANONICAL_LIST],
  allowedDurations: durationProfiles.map((item) => item.duration),
  writingRulePacks,
  directorRulePacks,
  sceneMappings,
  durationProfiles,
};

let kbSnapshotHashPromise: Promise<string> | null = null;

export function getRulePackById(rulePackId: string): RulePack | undefined {
  return [...writingRulePacks, ...directorRulePacks].find((item) => item.id === rulePackId);
}

export function getDurationProfile(duration: number): DurationCapacityProfile {
  return durationProfiles.find((item) => item.duration === duration) ?? durationProfiles[2];
}

export function isAllowedDuration(duration: number): boolean {
  return KB_SNAPSHOT.allowedDurations.includes(duration);
}

export async function getKbSnapshotHash(): Promise<string> {
  if (!kbSnapshotHashPromise) {
    kbSnapshotHashPromise = sha256Hex(stableStringify(KB_SNAPSHOT));
  }
  return kbSnapshotHashPromise;
}

export async function buildSanitizedKbSummary(sceneType: string, duration: number): Promise<SanitizedKbSummary> {
  const mapping = sceneMappings[sceneType] ?? sceneMappings[SCENE_TYPE_CANONICAL_LIST[0]];
  const sceneProfile = mapping.sceneProfile;
  const durationProfile = getDurationProfile(duration);

  return {
    kb_snapshot_hash: await getKbSnapshotHash(),
    scene_type_count: SCENE_TYPE_CANONICAL_LIST.length,
    scene_type_id: sceneType,
    scene_type_label: getSceneTypeLabel(sceneType),
    duration_options: [...KB_SNAPSHOT.allowedDurations],
    selected_sample_ids: [...mapping.selectedSampleIds],
    selected_kb_rules: [...mapping.selectedKbRules],
    writing_group_rule_pack_ids: [...mapping.writingRulePackIds],
    director_group_rule_pack_ids: [...mapping.directorRulePackIds],
    kb_context_summary: buildKbContextSummary(sceneType, sceneProfile, durationProfile),
    applied_to: ["narrative_body", "storyboard_prompt"],
    influence_axes: [...mapping.influenceAxes],
    scene_profile: sceneProfile,
    negative_constraints: [...mapping.negativeConstraints],
    raw_kb_rows_included: 0,
    raw_sample_text_absent: true,
    source_register_absent: true,
    overlay_json_absent: true,
    prompt_body_absent: true,
  };
}
