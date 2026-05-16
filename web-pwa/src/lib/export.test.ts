import { describe, expect, it } from "vitest";
import { auditBundle, containsSecretLeakage, createStoryboardExcelExport } from "./export";
import type { GenerationEvidence, SanitizedKbSummary, StoryboardRow } from "./types";

const rows: StoryboardRow[] = [
  {
    shot_index: 1,
    person: "林峯",
    shot_size: "中景",
    camera: "推进",
    visual_description: "林峯在旧城巷口回头，雨丝和霓虹同时落进镜头里。",
    character_action: "他把证物塞进苏瑶手心，示意她先走。",
    dialogue_or_narration: "",
    prompt_text: "事实源：林峯把证物交给苏瑶 | 主体：林峯 | 动作：递交证物 | 画面：雨夜巷口交接 | 运镜：推进 | 景别：中景 | 时长节奏：短促 | 场景谱系：都市奇幻 | 导演规则：visible-frame | 对白/旁白：以动作传达情绪 | 负面约束：不泄漏内部信息",
    duration_seconds: 5,
    status: "confirmed",
    note: "",
  },
];

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
  kb_context_summary: "都市奇幻写作组和导演组都参与。",
  applied_to: ["narrative_body", "storyboard_prompt"],
  influence_axes: ["visible_subject"],
  scene_profile: "都市奇幻要求奇观、空间层次和视觉焦点清晰落地。",
  negative_constraints: ["禁止泄漏 API Key、raw KB rows、source_register、overlay JSON、prompt_body"],
  raw_kb_rows_included: 0,
  raw_sample_text_absent: true,
  source_register_absent: true,
  overlay_json_absent: true,
  prompt_body_absent: true,
};

const evidence: GenerationEvidence = {
  artifact_identity: "z".repeat(64),
  source_lineage: "accepted_body",
  source_lineage_evidence: "accepted_body -> storyboard_task -> director_group",
  source_hash: "b".repeat(64),
  output_intent: "storyboard_rows",
  story_fact_frame: "accepted_body_locked -> storyboard_task -> director_group",
  accepted_body_hash: "c".repeat(64),
  storyboard_task: "task",
  storyboard_task_hash: "e".repeat(64),
  kb_snapshot_hash: "a".repeat(64),
  selected_sample_ids: kbSummary.selected_sample_ids,
  selected_kb_rules: kbSummary.selected_kb_rules,
  writing_group_rule_pack_ids: kbSummary.writing_group_rule_pack_ids,
  director_group_rule_pack_ids: kbSummary.director_group_rule_pack_ids,
  kb_context_summary: kbSummary.kb_context_summary,
  applied_to: kbSummary.applied_to,
  influence_axes: kbSummary.influence_axes,
  kb_oracle_affects_structure: true,
  raw_kb_rows_included: 0,
  raw_sample_text_absent: true,
  source_register_absent: true,
  overlay_json_absent: true,
  prompt_body_absent: true,
  scene_type_id: "urban_fantasy",
  scene_type_label: "都市奇幻",
  scene_type_canonical_coverage: 21,
  scene_type_valid: true,
  target_duration_seconds: 5,
  rows_hash: "d".repeat(64),
  confirmed_row_hash: "f".repeat(64),
  export_source_hash: "d".repeat(64),
  prompt_compilation_version: "hope-web-pwa-v4",
  warning_taxonomy: [],
  warning_taxonomy_classified: true,
  hardfail_warning_absent: true,
  stale_state: false,
  stale_task_detected: false,
  stale_rows_detected: false,
  task_sync_status: "fresh",
  rows_count: 1,
  rows_match: true,
  prompt_text_present: true,
  prompt_text_boundary_passed: true,
  prompt_text_not_summary_only: true,
  visual_description_visible_frame_passed: true,
  visual_description_no_trace: true,
  prompt_compiled_after_final_row: true,
  person_field_valid: true,
  location_not_in_person: true,
  action_fragment_not_in_person: true,
  scene_term_not_in_person: true,
  generic_role_placeholder_absent: true,
  validator_pseudo_success_detected: false,
  fallback_used: false,
  provider_failover_used: false,
  local_candidate: false,
  row_prompt_visual_gate_evidence: "ok",
  field_aware_entity_gate_evidence: "ok",
  manual_edit_confirmed_row_evidence: "manual_edit_detected=false",
  blocked_warning_ui_evidence: "none",
  human_review_spotcheck_evidence: "review rows=1",
  export_evidence: ["export_storyboard_uses_confirmed_rows=true", "export_script_uses_confirmed_body_and_rows=true"],
  validator_result: { passed: true, summary: "ok", issues: [] },
  provider_id: "qwen",
  model_id: "qwen3.6-plus",
  base_url_host: "dashscope.aliyuncs.com",
  cors_check_result: "passed",
  generated_at: new Date().toISOString(),
};

describe("export", () => {
  it("keeps sanitized fields only", () => {
    const bundle = createStoryboardExcelExport({
      body: "林峯把证物交给苏瑶，阿青留在雨夜巷口断后。",
      sceneTypeId: "urban_fantasy",
      sceneTypeLabel: "都市奇幻",
      targetDurationSeconds: 5,
      rows,
      kbSummary,
      evidence,
    });
    const audit = auditBundle(bundle);
    expect(containsSecretLeakage(bundle.content)).toBe(false);
    expect(bundle.content).toContain("confirmed_narrative_body");
    expect(bundle.content).toContain("scene_type_id");
    expect(bundle.content).toContain("raw_kb_rows_included");
    expect(audit.usable).toBe(true);
    expect(audit.contains_local_path).toBe(false);
    expect(audit.sampled_export_preview_present).toBe(true);
  });
});
