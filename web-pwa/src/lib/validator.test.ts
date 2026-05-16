import { describe, expect, it } from "vitest";
import { normalizeAndValidateStoryboardRows, validateNarrativeBody } from "./validator";
import type { SanitizedKbSummary, StoryboardRow } from "./types";

const kbSummary: SanitizedKbSummary = {
  kb_snapshot_hash: "a".repeat(64),
  scene_type_count: 21,
  scene_type_id: "urban_fantasy",
  scene_type_label: "都市奇幻",
  duration_options: [5, 10, 15, 30, 45, 60],
  selected_sample_ids: ["sample-urban_fantasy-A", "sample-urban_fantasy-B", "sample-urban_fantasy-C"],
  selected_kb_rules: ["scene-profile:urban_fantasy", "family:spectacle", "label:都市奇幻"],
  writing_group_rule_pack_ids: ["wg-complete-body"],
  director_group_rule_pack_ids: ["dg-visible-frame", "dg-prompt-compilation"],
  kb_context_summary: "都市奇幻写作组和导演组都会参与。",
  applied_to: ["narrative_body", "storyboard_prompt"],
  influence_axes: ["visible_subject"],
  scene_profile: "都市奇幻要求奇观、空间层次和视觉焦点清楚落地。",
  negative_constraints: ["禁止泄漏 API Key、raw KB rows、source_register、overlay JSON、prompt_body"],
  raw_kb_rows_included: 0,
  raw_sample_text_absent: true,
  source_register_absent: true,
  overlay_json_absent: true,
  prompt_body_absent: true,
};

describe("validator", () => {
  it("rejects prompt-like narrative text", () => {
    const result = validateNarrativeBody("scene_type: urban_fantasy\n- 主体：主角\n- 画面：追逐");
    expect(result.passed).toBe(false);
  });

  it("fills prompt text when row structure is otherwise valid", () => {
    const rows: StoryboardRow[] = [
      {
        shot_index: 1,
        person: "林峰",
        shot_size: "中景",
        camera: "推进",
        visual_description: "林峰站在积水巷口回头，霓虹映在雨水里，追兵的影子从巷尾逼近。",
        character_action: "他把证物塞进苏瑶手里，同时回身观察退路。",
        dialogue_or_narration: "快走。",
        prompt_text: "",
        duration_seconds: 5,
        status: "draft",
        note: "",
      },
      {
        shot_index: 2,
        person: "苏瑶",
        shot_size: "近景",
        camera: "摇镜",
        visual_description: "苏瑶接过证物时手指发抖，潮湿灯牌映在她的眼睛里。",
        character_action: "她低头确认手中的东西，再抬头看向林峰。",
        dialogue_or_narration: "",
        prompt_text: "",
        duration_seconds: 5,
        status: "draft",
        note: "",
      },
      {
        shot_index: 3,
        person: "阿青",
        shot_size: "远景",
        camera: "跟拍",
        visual_description: "阿青横挡在巷口中央，雨幕后的脚步声和火光同时逼近。",
        character_action: "他向两人示意立即撤离，自己留下断后。",
        dialogue_or_narration: "",
        prompt_text: "",
        duration_seconds: 5,
        status: "draft",
        note: "",
      },
    ];

    const result = normalizeAndValidateStoryboardRows(rows, kbSummary, 15, "林峰护着苏瑶穿过雨夜旧城，阿青断后。");
    expect(result.passed).toBe(true);
    expect(rows[0].prompt_text).toContain("主体：林峰");
    expect(rows[0].prompt_text).toContain("负面约束：");
  });
});
