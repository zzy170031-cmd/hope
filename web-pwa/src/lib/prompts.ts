import { getDurationProfile, getRulePackById } from "./kb";
import { summarizeFacts } from "./facts";
import type { GenerationContext, SourceFacts, StoryboardRow, StoryboardTask } from "./types";

interface ChatMessage {
  role: "system" | "user";
  content: string;
}

function ruleLines(ids: string[]): string[] {
  return ids.flatMap((id) => {
    const pack = getRulePackById(id);
    if (!pack) {
      return [];
    }
    return [
      `- ${pack.id} / ${pack.name}`,
      ...pack.directives.map((item) => `  - ${item}`),
      ...pack.negativeConstraints.map((item) => `  - 禁止：${item}`),
    ];
  });
}

function compactExcerpt(value: string): string {
  return value.replace(/\s+/g, " ").trim().slice(0, 88);
}

export function buildConnectionTestMessages(model: string): ChatMessage[] {
  return [
    {
      role: "system",
      content: "You are a connection probe. Respond with one short sentence confirming the model is reachable.",
    },
    {
      role: "user",
      content: `Model handshake check for ${model}. Reply with: reachable`,
    },
  ];
}

export function buildWritingMessages(context: GenerationContext, facts: SourceFacts): ChatMessage[] {
  const durationProfile = getDurationProfile(context.targetDuration);
  return [
    {
      role: "system",
      content: [
        "你是 Hope Web 的写作组，不是提示词生成器。",
        "你的任务是把用户输入和 KB 写作组摘要编译成可确认的完整正文。",
        "正文必须是用户可直接确认的故事文本，不得输出分镜表、prompt、raw KB、overlay JSON 或内部控制行。",
      ].join("\n"),
    },
    {
      role: "user",
      content: [
        `scene_type_id: ${context.sceneTypeId}`,
        `scene_type_label: ${context.sceneTypeLabel}`,
        `target_duration_seconds: ${context.targetDuration}`,
        `mode: ${context.mode}`,
        `kb_context_summary: ${context.kbSummary.kb_context_summary}`,
        `scene_profile: ${context.kbSummary.scene_profile}`,
        `duration_capacity: ${durationProfile.pacingDirective}`,
        "writing_group_rule_pack_ids:",
        ...context.kbSummary.writing_group_rule_pack_ids.map((item) => `- ${item}`),
        "writing_group_rules:",
        ...ruleLines(context.kbSummary.writing_group_rule_pack_ids),
        "negative_constraints:",
        ...context.kbSummary.negative_constraints.map((item) => `- ${item}`),
        "local_source_facts:",
        summarizeFacts(facts),
        "user_source_text:",
        context.sourceText,
        "输出要求：",
        "- 输出 JSON 对象",
        "- title：故事标题",
        "- body：完整正文，至少两段",
        "- 必须保留真实人物、地点、冲突和事件顺序，scene_type 只改变表达结构和镜头感。",
      ].join("\n"),
    },
  ];
}

export function buildDirectorMessages(
  context: GenerationContext,
  facts: SourceFacts,
  acceptedBody: string,
  task: StoryboardTask,
): ChatMessage[] {
  const durationProfile = getDurationProfile(context.targetDuration);
  return [
    {
      role: "system",
      content: [
        "你是 Hope Web 的导演组。",
        "accepted body 是唯一事实源，storyboard task 是唯一当前任务。",
        "返回严格 JSON，不要解释，不要 markdown。",
      ].join("\n"),
    },
    {
      role: "user",
      content: [
        `scene_type_id: ${context.sceneTypeId}`,
        `scene_type_label: ${context.sceneTypeLabel}`,
        `target_duration_seconds: ${context.targetDuration}`,
        `target_shot_count: ${durationProfile.targetShotCount}`,
        `storyboard_task_hash: ${task.task_hash}`,
        `accepted_body_hash: ${task.accepted_body_hash}`,
        `kb_context_summary: ${context.kbSummary.kb_context_summary}`,
        `scene_profile: ${context.kbSummary.scene_profile}`,
        "director_group_rule_pack_ids:",
        ...context.kbSummary.director_group_rule_pack_ids.map((item) => `- ${item}`),
        "director_group_rules:",
        ...ruleLines(context.kbSummary.director_group_rule_pack_ids),
        "negative_constraints:",
        ...context.kbSummary.negative_constraints.map((item) => `- ${item}`),
        "local_source_facts:",
        summarizeFacts(facts),
        "accepted_body:",
        acceptedBody,
        "输出要求：",
        "- 输出 JSON 对象，字段 rows 为数组。",
        "- 每条 row 必须包含 shot_index, person, shot_size, camera, visual_description, character_action, dialogue_or_narration, duration_seconds, status, note。",
        "- rows 至少 3 条，并尽量贴合 target_shot_count。",
        "- visual_description 只写当前镜头可见画面。",
        "- person 必须来自 accepted_body 中真实存在或明确可归属的人物/角色标签。",
        "- duration_seconds 总和必须严格等于 target_duration_seconds。",
        "- 不要输出 raw KB、source_register、overlay JSON、prompt_body 或 validator trace。",
      ].join("\n"),
    },
  ];
}

export function compilePromptText(
  row: Omit<StoryboardRow, "prompt_text">,
  params: {
    acceptedBody: string;
    sceneLabel: string;
    sceneProfile: string;
    durationPacing: string;
    directorRuleLabels: string[];
    negativeConstraints: string[];
  },
): string {
  return [
    `事实源：${compactExcerpt(params.acceptedBody)}`,
    `主体：${row.person}`,
    `动作：${row.character_action}`,
    `画面：${row.visual_description}`,
    `运镜：${row.camera}`,
    `景别：${row.shot_size}`,
    `时长节奏：${params.durationPacing}；当前第 ${row.shot_index} 镜 ${row.duration_seconds} 秒`,
    `场景调性：${params.sceneLabel}；${params.sceneProfile}`,
    `导演规则：${params.directorRuleLabels.join("、")}`,
    `对白/旁白：${row.dialogue_or_narration || "以动作和表情传达情绪"}`,
    `负面约束：${params.negativeConstraints.join("；")}`,
  ].join(" | ");
}
