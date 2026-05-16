import { stableStringify } from "./hash";
import type { ExportAudit, ExportBundle, GenerationEvidence, SanitizedKbSummary, StoryboardRow } from "./types";

function shortHash(value: string): string {
  return value ? value.slice(0, 12) : "";
}

function redactedSanitizedProvenance(evidence: GenerationEvidence) {
  return {
    artifact_identity: shortHash(evidence.artifact_identity),
    source_lineage: evidence.source_lineage,
    source_lineage_evidence: evidence.source_lineage_evidence,
    output_intent: evidence.output_intent,
    story_fact_frame: evidence.story_fact_frame,
    accepted_body_ref: shortHash(evidence.accepted_body_hash),
    storyboard_task_ref: shortHash(evidence.storyboard_task_hash),
    kb_snapshot_ref: shortHash(evidence.kb_snapshot_hash),
    rows_ref: shortHash(evidence.rows_hash),
    confirmed_rows_ref: shortHash(evidence.confirmed_row_hash),
    writing_group_rule_pack_ids: evidence.writing_group_rule_pack_ids,
    director_group_rule_pack_ids: evidence.director_group_rule_pack_ids,
    scene_type_id: evidence.scene_type_id,
    scene_type_label: evidence.scene_type_label,
    target_duration_seconds: evidence.target_duration_seconds,
    rows_count: evidence.rows_count,
    rows_match: evidence.rows_match,
    prompt_text_present: evidence.prompt_text_present,
    prompt_text_boundary_passed: evidence.prompt_text_boundary_passed,
    visual_description_visible_frame_passed: evidence.visual_description_visible_frame_passed,
    prompt_compiled_after_final_row: evidence.prompt_compiled_after_final_row,
    warning_taxonomy: evidence.warning_taxonomy,
    stale_state: evidence.stale_state,
    raw_kb_rows_included: evidence.raw_kb_rows_included,
    raw_sample_text_absent: evidence.raw_sample_text_absent,
    source_register_absent: evidence.source_register_absent,
    overlay_json_absent: evidence.overlay_json_absent,
    prompt_body_absent: evidence.prompt_body_absent,
    provider_id: evidence.provider_id,
    model_id: evidence.model_id,
    base_url_host: evidence.base_url_host,
    generated_at: evidence.generated_at.slice(0, 19),
  };
}

function escapeXml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&apos;");
}

function cell(value: string | number): string {
  const normalized = typeof value === "number" ? String(value) : value;
  return `<Cell><Data ss:Type="String">${escapeXml(normalized)}</Data></Cell>`;
}

function row(values: Array<string | number>): string {
  return `<Row>${values.map((value) => cell(value)).join("")}</Row>`;
}

function workbookXml(sheets: Array<{ name: string; rows: Array<Array<string | number>> }>): string {
  return [
    '<?xml version="1.0"?>',
    '<?mso-application progid="Excel.Sheet"?>',
    '<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet" xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:x="urn:schemas-microsoft-com:office:excel" xmlns:ss="urn:schemas-microsoft-com:office:spreadsheet" xmlns:html="http://www.w3.org/TR/REC-html40">',
    ...sheets.map(
      (sheet) =>
        `<Worksheet ss:Name="${escapeXml(sheet.name)}"><Table>${sheet.rows.map((cells) => row(cells)).join("")}</Table></Worksheet>`,
    ),
    "</Workbook>",
  ].join("");
}

function bundlePreview(bundle: ExportBundle): string[] {
  const worksheetNames = Array.from(bundle.content.matchAll(/<Worksheet ss:Name="([^"]+)"/g)).map((match) => match[1]);
  const previewCells = Array.from(bundle.content.matchAll(/<Data ss:Type="String">([^<]+)<\/Data>/g))
    .map((match) => match[1])
    .slice(0, 12);
  return [...worksheetNames, ...previewCells].slice(0, 12);
}

export function createStoryboardExcelExport(params: {
  body: string;
  sceneTypeId: string;
  sceneTypeLabel: string;
  targetDurationSeconds: number;
  rows: StoryboardRow[];
  kbSummary: SanitizedKbSummary;
  evidence: GenerationEvidence;
}): ExportBundle {
  const content = workbookXml([
    {
      name: "导出摘要",
      rows: [
        ["confirmed_narrative_body", params.body],
        ["scene_type_id", params.sceneTypeId],
        ["scene_type_label", params.sceneTypeLabel],
        ["target_duration_seconds", params.targetDurationSeconds],
        ["kb_context_summary", params.kbSummary.kb_context_summary],
        ["selected_sample_ids", params.kbSummary.selected_sample_ids.join(", ")],
        ["selected_kb_rules", params.kbSummary.selected_kb_rules.join(", ")],
        ["writing_group_rule_pack_ids", params.kbSummary.writing_group_rule_pack_ids.join(", ")],
        ["director_group_rule_pack_ids", params.kbSummary.director_group_rule_pack_ids.join(", ")],
        ["raw_kb_rows_included", params.kbSummary.raw_kb_rows_included],
        ["raw_sample_text_absent", String(params.kbSummary.raw_sample_text_absent)],
        ["source_register_absent", String(params.kbSummary.source_register_absent)],
        ["overlay_json_absent", String(params.kbSummary.overlay_json_absent)],
        ["prompt_body_absent", String(params.kbSummary.prompt_body_absent)],
        ["sanitized_provenance_summary", JSON.stringify(redactedSanitizedProvenance(params.evidence))],
      ],
    },
    {
      name: "分镜词",
      rows: [
        ["镜头序号", "人物", "景别", "运镜", "画面描述", "角色动作", "对白/旁白", "分镜提示词", "时长", "备注/状态"],
        ...params.rows.map((item) => [
          item.shot_index,
          item.person,
          item.shot_size,
          item.camera,
          item.visual_description,
          item.character_action,
          item.dialogue_or_narration,
          item.prompt_text,
          item.duration_seconds,
          `${item.note || ""}${item.note ? " / " : ""}${item.status}`,
        ]),
      ],
    },
  ]);

  return {
    fileName: "hope-storyboard-prompts.xls",
    mimeType: "application/vnd.ms-excel;charset=utf-8",
    content,
    kind: "storyboard",
    format: "excel",
  };
}

export function createScriptExcelExport(params: {
  body: string;
  sceneTypeId: string;
  sceneTypeLabel: string;
  targetDurationSeconds: number;
  rows: StoryboardRow[];
  kbSummary: SanitizedKbSummary;
  evidence: GenerationEvidence;
}): ExportBundle {
  const content = workbookXml([
    {
      name: "完整剧本",
      rows: [
        ["confirmed_narrative_body", params.body],
        ["scene_type_id", params.sceneTypeId],
        ["scene_type_label", params.sceneTypeLabel],
        ["target_duration_seconds", params.targetDurationSeconds],
        ["sanitized_kb_summary", params.kbSummary.kb_context_summary],
        ["selected_sample_ids", params.kbSummary.selected_sample_ids.join(", ")],
        ["selected_kb_rules", params.kbSummary.selected_kb_rules.join(", ")],
        ["writing_group_rule_pack_ids", params.kbSummary.writing_group_rule_pack_ids.join(", ")],
        ["director_group_rule_pack_ids", params.kbSummary.director_group_rule_pack_ids.join(", ")],
        ["raw_kb_rows_included", params.kbSummary.raw_kb_rows_included],
        ["raw_sample_text_absent", String(params.kbSummary.raw_sample_text_absent)],
        ["source_register_absent", String(params.kbSummary.source_register_absent)],
        ["overlay_json_absent", String(params.kbSummary.overlay_json_absent)],
        ["prompt_body_absent", String(params.kbSummary.prompt_body_absent)],
        ["sanitized_provenance_summary", JSON.stringify(redactedSanitizedProvenance(params.evidence))],
      ],
    },
    {
      name: "确认分镜",
      rows: [
        ["镜头序号", "人物", "景别", "运镜", "画面描述", "角色动作", "对白/旁白", "分镜提示词", "时长", "备注/状态"],
        ...params.rows.map((item) => [
          item.shot_index,
          item.person,
          item.shot_size,
          item.camera,
          item.visual_description,
          item.character_action,
          item.dialogue_or_narration,
          item.prompt_text,
          item.duration_seconds,
          `${item.note || ""}${item.note ? " / " : ""}${item.status}`,
        ]),
      ],
    },
  ]);

  return {
    fileName: "hope-full-script.xls",
    mimeType: "application/vnd.ms-excel;charset=utf-8",
    content,
    kind: "script",
    format: "excel",
  };
}

export function containsSecretLeakage(serialized: string): boolean {
  return /(api[_ -]?key|authorization|bearer|prompt_body|source_register|overlay[_ -]?json|raw[_ -]?kb[_ -]?rows)/i.test(serialized);
}

export function containsLocalPathLeakage(serialized: string): boolean {
  return /([A-Z]:\\[^<\r\n]+|\/(?:Users|home|root)\/[^\s<]+|\.sandbox-secrets)/i.test(serialized);
}

export function containsRawKbLeakage(serialized: string): boolean {
  return /(raw[_ -]?kb[_ -]?rows|raw[_ -]?sample[_ -]?text|sample_text|source_register|overlay[_ -]?json|prompt_body)/i.test(serialized);
}

export function auditBundle(bundle: ExportBundle): ExportAudit {
  const preview = bundlePreview(bundle);
  return {
    file_name: bundle.fileName,
    format: bundle.format,
    kind: bundle.kind,
    usable: bundle.content.trim().length > 0,
    contains_secret_leakage: containsSecretLeakage(bundle.content),
    contains_local_path: containsLocalPathLeakage(bundle.content),
    contains_raw_kb: containsRawKbLeakage(bundle.content),
    sampled_export_preview_present: preview.length > 0,
    preview,
  };
}

export function downloadBundle(bundle: ExportBundle): void {
  const blob = new Blob([bundle.content], { type: bundle.mimeType });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = bundle.fileName;
  anchor.click();
  URL.revokeObjectURL(url);

  const audit = auditBundle(bundle);
  const win = window as Window & { __hopeLastDownloads?: Array<Record<string, unknown>> };
  const next = win.__hopeLastDownloads ?? [];
  next.push({
    fileName: bundle.fileName,
    mimeType: bundle.mimeType,
    kind: bundle.kind,
    format: bundle.format,
    usable: audit.usable,
    contains_secret_leakage: audit.contains_secret_leakage,
    contains_local_path: audit.contains_local_path,
    contains_raw_kb: audit.contains_raw_kb,
    sampled_export_preview_present: audit.sampled_export_preview_present,
    preview: audit.preview,
  });
  win.__hopeLastDownloads = next.slice(-8);
}

export function stableExportDigest(bundle: ExportBundle): string {
  return stableStringify({ fileName: bundle.fileName, mimeType: bundle.mimeType, content: bundle.content });
}
