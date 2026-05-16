import { getDurationProfile, isAllowedDuration } from "./kb";
import { compilePromptText } from "./prompts";
import type { SanitizedKbSummary, StoryboardRow, ValidationIssue, ValidationResult } from "./types";

const forbiddenNarrativeTokens = ["prompt", "提示词", "shot_index", "scene_type", "raw kb", "overlay json"];
const forbiddenPersonTokens = ["环境", "镜头", "构图", "画面", "场景", "UI", "空间", "光影", "氛围", "/"];

function buildValidationResult(issues: ValidationIssue[], successSummary: string): ValidationResult {
  const hasError = issues.some((item) => item.severity === "error");
  return {
    passed: !hasError,
    summary: hasError ? issues.find((item) => item.severity === "error")?.message ?? "校验失败。" : successSummary,
    issues,
  };
}

export function validateNarrativeBody(body: string): ValidationResult {
  const issues: ValidationIssue[] = [];
  const trimmed = body.trim();

  if (trimmed.length < 120) {
    issues.push({ code: "body_too_short", message: "正文过短，无法支撑完整链路。", severity: "error" });
  }
  if (trimmed.split(/\n+/).filter(Boolean).length < 2) {
    issues.push({ code: "body_not_paragraph", message: "正文至少需要两段连续文本。", severity: "error" });
  }
  if (forbiddenNarrativeTokens.some((token) => trimmed.toLowerCase().includes(token.toLowerCase()))) {
    issues.push({ code: "body_prompt_like", message: "正文混入了 prompt 或 meta 痕迹。", severity: "error" });
  }
  if (/^\s*[-*]\s+/m.test(trimmed)) {
    issues.push({ code: "body_bullet_list", message: "正文不能写成动作清单。", severity: "error" });
  }

  return buildValidationResult(issues, "正文通过基础校验，可以进入“确定使用”。");
}

export function normalizeAndValidateStoryboardRows(
  rows: StoryboardRow[],
  kbSummary: SanitizedKbSummary,
  targetDuration: number,
  acceptedBody: string,
): ValidationResult {
  const issues: ValidationIssue[] = [];
  const durationProfile = getDurationProfile(targetDuration);

  if (rows.length < 3) {
    issues.push({ code: "rows_too_few", message: "分镜条数不足，至少需要 3 条。", severity: "error" });
  }
  if (!isAllowedDuration(targetDuration)) {
    issues.push({ code: "duration_not_allowed", message: "目标时长不在 5/10/15/30/45/60 白名单内。", severity: "error" });
  }

  const totalDuration = rows.reduce((sum, row) => sum + row.duration_seconds, 0);
  if (totalDuration !== targetDuration) {
    issues.push({
      code: "duration_mismatch",
      message: `分镜总时长为 ${totalDuration} 秒，与目标 ${targetDuration} 秒不一致。`,
      severity: "error",
    });
  }

  rows.forEach((row) => {
    if (!row.person.trim() || forbiddenPersonTokens.includes(row.person.trim())) {
      issues.push({
        code: `row_${row.shot_index}_person_invalid`,
        message: `第 ${row.shot_index} 镜的 person 字段为空或使用了错误对象。`,
        severity: "error",
      });
    }
    if (row.visual_description.trim().length < 16) {
      issues.push({
        code: `row_${row.shot_index}_visual_short`,
        message: `第 ${row.shot_index} 镜的 visual_description 过短。`,
        severity: "error",
      });
    }
    if (/(抽象|感觉|寓意|象征)/.test(row.visual_description)) {
      issues.push({
        code: `row_${row.shot_index}_visual_abstract`,
        message: `第 ${row.shot_index} 镜的 visual_description 过于抽象，不像当前可见帧。`,
        severity: "error",
      });
    }
    if (row.duration_seconds <= 0) {
      issues.push({
        code: `row_${row.shot_index}_duration_invalid`,
        message: `第 ${row.shot_index} 镜时长必须大于 0。`,
        severity: "error",
      });
    }
    if (!row.prompt_text.trim()) {
      row.prompt_text = compilePromptText(
        {
          shot_index: row.shot_index,
          person: row.person,
          shot_size: row.shot_size,
          camera: row.camera,
          visual_description: row.visual_description,
          character_action: row.character_action,
          dialogue_or_narration: row.dialogue_or_narration,
          duration_seconds: row.duration_seconds,
          status: row.status,
          note: row.note,
        },
        {
          acceptedBody,
          sceneLabel: kbSummary.scene_type_label,
          sceneProfile: kbSummary.scene_profile,
          durationPacing: durationProfile.pacingDirective,
          directorRuleLabels: kbSummary.director_group_rule_pack_ids,
          negativeConstraints: kbSummary.negative_constraints,
        },
      );
    }
    if (
      !row.prompt_text.includes("事实源：") ||
      !row.prompt_text.includes("主体：") ||
      !row.prompt_text.includes("画面：") ||
      !row.prompt_text.includes("负面约束：")
    ) {
      issues.push({
        code: `row_${row.shot_index}_prompt_incomplete`,
        message: `第 ${row.shot_index} 镜的 prompt_text 结构不完整。`,
        severity: "error",
      });
    }
  });

  return buildValidationResult(issues, "分镜通过结构与可视化校验。");
}
