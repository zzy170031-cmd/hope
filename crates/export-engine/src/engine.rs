use std::fs;
use std::path::{Path, PathBuf};

use core_domain::{Exporter, GeneratedStoryboardRow};
use serde_json::{Map, Value};

use crate::contract::CANONICAL_WORKBOOK_CONTRACT;

pub const WEEK3_SHARED_FIXTURE_PATH: &str =
    r"E:\codex\hope\contracts\fixtures\week3-shared-fixture.json";
pub const WEEK3_VALIDATION_REPORT_PATH: &str =
    r"E:\codex\hope\contracts\fixtures\week3-validation-report.json";
pub const WEEK3_EXPORT_DIR: &str = r"E:\codex\hope\contracts\fixtures\exports";
pub const WEEK3_EXPORT_XLSX_PATH: &str =
    r"E:\codex\hope\contracts\fixtures\exports\week3-export.xlsx";
pub const WEEK3_EXPORT_JSON_PATH: &str =
    r"E:\codex\hope\contracts\fixtures\exports\week3-export.json";
pub const WEEK3_EXPORT_MARKDOWN_PATH: &str =
    r"E:\codex\hope\contracts\fixtures\exports\week3-export.md";
const WEEK3_EXPORT_XLSX_FILE: &str = "week3-export.xlsx";
const WEEK3_EXPORT_JSON_FILE: &str = "week3-export.json";
const WEEK3_EXPORT_MARKDOWN_FILE: &str = "week3-export.md";
pub const V120_STORYBOARD_EXPORT_DIR: &str = r"E:\codex\hope\exports\v120";
pub const V120_STORYBOARD_SHEET_MACHINE_NAME: &str = "v120_storyboard_rows";
pub const V120_STORYBOARD_COLUMNS: &[&str] = &[
    "shot_script",
    "primary_scene_type",
    "primary_scene_label",
    "primary_scene_category",
    "shot_scene_type",
    "shot_scene_label",
    "shot_intent",
    "adaptation_reason",
    "grounding_source",
    "序号",
    "人物",
    "镜头",
    "景别",
    "运镜",
    "画面描述",
    "角色动作",
    "对白/旁白",
    "分镜提示词",
    "时长(秒)",
    "shot_duration_seconds",
    "duration_source",
];

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExportRequest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkbookSheetManifest {
    pub machine_name: &'static str,
    pub chinese_name: &'static str,
    pub columns: Vec<&'static str>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkbookManifest {
    pub workbook_machine_name: &'static str,
    pub workbook_chinese_name: &'static str,
    pub sheets: Vec<WorkbookSheetManifest>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportBundle {
    pub workbook: WorkbookManifest,
    pub excel_path: PathBuf,
    pub json_path: PathBuf,
    pub markdown_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct V120StoryboardExportRequest {
    pub export_manifest_id: String,
    pub result_id: String,
    pub selected_total_duration_seconds: u16,
    pub source_result_id: String,
    pub edited_rows_applied: bool,
    pub rows: Vec<GeneratedStoryboardRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct V120StoryboardExportArtifact {
    pub artifact_kind: &'static str,
    pub export_format: &'static str,
    pub path: PathBuf,
    pub byte_size: u64,
    pub content_hash: String,
    pub row_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct V120StoryboardExportBundle {
    pub workbook: WorkbookManifest,
    pub artifacts: Vec<V120StoryboardExportArtifact>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExportError {
    Io {
        path: String,
        message: String,
    },
    Parse {
        path: String,
        key: &'static str,
        message: String,
    },
    Contract {
        sheet_machine_name: &'static str,
        message: String,
    },
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct ExportEngine;

impl ExportEngine {
    pub fn fixed_request() -> ExportRequest {
        ExportRequest
    }

    pub fn export_from_fixtures() -> Result<ExportBundle, ExportError> {
        Self::export_from_fixture_paths(
            WEEK3_SHARED_FIXTURE_PATH,
            WEEK3_VALIDATION_REPORT_PATH,
            WEEK3_EXPORT_DIR,
        )
    }

    pub fn export_from_fixture_paths(
        shared_fixture_path: impl AsRef<Path>,
        validation_report_path: impl AsRef<Path>,
        export_dir: impl AsRef<Path>,
    ) -> Result<ExportBundle, ExportError> {
        let workbook =
            load_workbook_from_paths(shared_fixture_path.as_ref(), validation_report_path.as_ref())?;
        let (excel_path, json_path, markdown_path) =
            write_export_files_to_dir(&workbook, export_dir.as_ref())?;

        Ok(ExportBundle {
            workbook,
            excel_path,
            json_path,
            markdown_path,
        })
    }

    pub fn export_v120_storyboard(
        request: &V120StoryboardExportRequest,
    ) -> Result<V120StoryboardExportBundle, ExportError> {
        let workbook = build_v120_storyboard_workbook(request);
        let artifacts = write_v120_storyboard_export_files(request, &workbook)?;

        Ok(V120StoryboardExportBundle {
            workbook,
            artifacts,
        })
    }
}

pub fn fixed_export_request() -> ExportRequest {
    ExportEngine::fixed_request()
}

pub fn export_week3_from_fixtures() -> Result<ExportBundle, ExportError> {
    ExportEngine::export_from_fixtures()
}

pub fn export_week3_from_paths(
    shared_fixture_path: impl AsRef<Path>,
    validation_report_path: impl AsRef<Path>,
    export_dir: impl AsRef<Path>,
) -> Result<ExportBundle, ExportError> {
    ExportEngine::export_from_fixture_paths(shared_fixture_path, validation_report_path, export_dir)
}

pub fn export_v120_storyboard_bundle(
    request: &V120StoryboardExportRequest,
) -> Result<V120StoryboardExportBundle, ExportError> {
    ExportEngine::export_v120_storyboard(request)
}

impl Exporter for ExportEngine {
    type Input = ExportRequest;
    type Output = ExportBundle;
    type Error = ExportError;

    fn export(&self, _input: &Self::Input) -> Result<Self::Output, Self::Error> {
        ExportEngine::export_from_fixtures()
    }
}

fn load_workbook_from_paths(
    shared_fixture_path: &Path,
    validation_report_path: &Path,
) -> Result<WorkbookManifest, ExportError> {
    let shared = read_json(shared_fixture_path)?;
    let validation = read_json(validation_report_path)?;

    let mut sheets = Vec::new();
    for sheet in CANONICAL_WORKBOOK_CONTRACT.sheets {
        let source = if sheet.machine_name == "validation_report" {
            &validation
        } else {
            &shared
        };
        let rows = parse_sheet_rows(
            source,
            sheet.machine_name,
            sheet.columns,
            if sheet.machine_name == "validation_report" {
                validation_report_path
            } else {
                shared_fixture_path
            },
        )?;

        sheets.push(WorkbookSheetManifest {
            machine_name: sheet.machine_name,
            chinese_name: sheet.chinese_name,
            columns: sheet.columns.to_vec(),
            rows,
        });
    }

    Ok(WorkbookManifest {
        workbook_machine_name: CANONICAL_WORKBOOK_CONTRACT.workbook_machine_name,
        workbook_chinese_name: CANONICAL_WORKBOOK_CONTRACT.workbook_chinese_name,
        sheets,
    })
}

fn write_export_files_to_dir(
    workbook: &WorkbookManifest,
    export_dir: &Path,
) -> Result<(PathBuf, PathBuf, PathBuf), ExportError> {
    fs::create_dir_all(export_dir).map_err(|error| ExportError::Io {
        path: export_dir.display().to_string(),
        message: error.to_string(),
    })?;

    let excel_path = export_dir.join(WEEK3_EXPORT_XLSX_FILE);
    let json_path = export_dir.join(WEEK3_EXPORT_JSON_FILE);
    let markdown_path = export_dir.join(WEEK3_EXPORT_MARKDOWN_FILE);

    fs::write(&json_path, render_json(workbook)).map_err(|error| ExportError::Io {
        path: json_path.display().to_string(),
        message: error.to_string(),
    })?;
    fs::write(&markdown_path, render_markdown(workbook)).map_err(|error| {
        ExportError::Io {
            path: markdown_path.display().to_string(),
            message: error.to_string(),
        }
    })?;
    fs::write(&excel_path, render_xlsx(workbook)).map_err(|error| ExportError::Io {
        path: excel_path.display().to_string(),
        message: error.to_string(),
    })?;

    Ok((excel_path, json_path, markdown_path))
}

fn build_v120_storyboard_workbook(request: &V120StoryboardExportRequest) -> WorkbookManifest {
    let rows = request
        .rows
        .iter()
        .map(|row| v120_storyboard_row_cells(request, row))
        .collect();
    let mut columns = V120_STORYBOARD_COLUMNS.to_vec();
    columns.push("prompt_text_compilation_status");
    columns.push("prompt_text_compilation_warnings");
    columns.push("prompt_text_source_row_id");
    columns.push("selected_total_duration_seconds");
    columns.push("source_result_id");
    columns.push("edited_rows_applied");
    WorkbookManifest {
        workbook_machine_name: "v120_storyboard_export_bundle",
        workbook_chinese_name: "V120 Storyboard Export Bundle",
        sheets: vec![WorkbookSheetManifest {
            machine_name: V120_STORYBOARD_SHEET_MACHINE_NAME,
            chinese_name: "V120 Storyboard Rows",
            columns,
            rows,
        }],
    }
}

fn v120_storyboard_row_cells(
    request: &V120StoryboardExportRequest,
    row: &GeneratedStoryboardRow,
) -> Vec<String> {
    vec![
        row.shot_script.clone(),
        row.primary_scene_type.clone(),
        row.primary_scene_label.clone(),
        row.primary_scene_category.clone(),
        row.shot_scene_type.clone(),
        row.shot_scene_label.clone(),
        row.shot_intent.clone(),
        row.adaptation_reason.clone(),
        row.grounding_source.as_str().to_string(),
        row.order.to_string(),
        row.person.clone(),
        row.shot_title.clone(),
        row.scene_scale.clone(),
        row.camera_movement.clone(),
        row.visual_description.clone(),
        row.character_action.clone(),
        row.dialogue.clone(),
        row.prompt_text.clone(),
        row.duration_seconds.to_string(),
        row.shot_duration_seconds.to_string(),
        row.duration_source.clone(),
        format!("{:?}", row.prompt_text_compilation_status),
        row.prompt_text_compilation_warnings
            .iter()
            .map(|warning| warning.code.as_str())
            .collect::<Vec<_>>()
            .join("|"),
        row.prompt_text_source_row_id.clone(),
        request.selected_total_duration_seconds.to_string(),
        request.source_result_id.clone(),
        request.edited_rows_applied.to_string(),
    ]
}

fn write_v120_storyboard_export_files(
    request: &V120StoryboardExportRequest,
    workbook: &WorkbookManifest,
) -> Result<Vec<V120StoryboardExportArtifact>, ExportError> {
    fs::create_dir_all(V120_STORYBOARD_EXPORT_DIR).map_err(|error| ExportError::Io {
        path: V120_STORYBOARD_EXPORT_DIR.to_string(),
        message: error.to_string(),
    })?;

    let file_stem = sanitize_file_stem(&request.export_manifest_id);
    let output_dir = PathBuf::from(V120_STORYBOARD_EXPORT_DIR);
    let json_path = output_dir.join(format!("{file_stem}.json"));
    let csv_path = output_dir.join(format!("{file_stem}.csv"));
    let xlsx_path = output_dir.join(format!("{file_stem}.xlsx"));
    let row_count = request.rows.len() as u32;

    let sheet = workbook
        .sheets
        .iter()
        .find(|sheet| sheet.machine_name == V120_STORYBOARD_SHEET_MACHINE_NAME)
        .expect("v120 storyboard workbook must include storyboard row sheet");

    let json_bytes = render_json(workbook).into_bytes();
    let csv_bytes = render_csv(sheet).into_bytes();
    let xlsx_bytes = render_xlsx(workbook);

    let mut artifacts = Vec::new();
    artifacts.push(write_v120_export_artifact(
        "storyboard_json",
        "json",
        json_path,
        json_bytes,
        row_count,
    )?);
    artifacts.push(write_v120_export_artifact(
        "storyboard_csv",
        "csv",
        csv_path,
        csv_bytes,
        row_count,
    )?);
    artifacts.push(write_v120_export_artifact(
        "excel_workbook",
        "xlsx",
        xlsx_path,
        xlsx_bytes,
        row_count,
    )?);

    Ok(artifacts)
}

fn write_v120_export_artifact(
    artifact_kind: &'static str,
    export_format: &'static str,
    path: PathBuf,
    bytes: Vec<u8>,
    row_count: u32,
) -> Result<V120StoryboardExportArtifact, ExportError> {
    fs::write(&path, &bytes).map_err(|error| ExportError::Io {
        path: path.display().to_string(),
        message: format!("{}: {}", path.display(), error),
    })?;

    Ok(V120StoryboardExportArtifact {
        artifact_kind,
        export_format,
        path,
        byte_size: bytes.len() as u64,
        content_hash: format!("crc32:{:08x}", crc32(&bytes)),
        row_count,
    })
}

fn sanitize_file_stem(value: &str) -> String {
    let sanitized = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .collect::<String>();

    if sanitized.is_empty() {
        "v120-storyboard-export".to_string()
    } else {
        sanitized
    }
}

fn read_json(path: &Path) -> Result<Value, ExportError> {
    let text = fs::read_to_string(path).map_err(|error| ExportError::Io {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    serde_json::from_str(&text).map_err(|error| ExportError::Parse {
        path: path.display().to_string(),
        key: "root",
        message: error.to_string(),
    })
}

fn parse_sheet_rows(
    source: &Value,
    key: &'static str,
    columns: &[&'static str],
    path: &Path,
) -> Result<Vec<Vec<String>>, ExportError> {
    let path_display = path.display().to_string();
    let rows = source
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| ExportError::Parse {
            path: path_display.clone(),
            key,
            message: "missing sheet section".to_string(),
        })?;

    rows.iter()
        .map(|row| {
            let object = row.as_object().ok_or_else(|| ExportError::Parse {
                path: path_display.clone(),
                key,
                message: "row must be an object".to_string(),
            })?;
            extract_row(source, object, columns, key)
        })
        .collect()
}

fn extract_row(
    source: &Value,
    object: &Map<String, Value>,
    columns: &[&'static str],
    sheet_machine_name: &'static str,
) -> Result<Vec<String>, ExportError> {
    columns
        .iter()
        .map(|column| {
            if sheet_machine_name == "prompt_package"
                && *column == canonical_column("prompt_package", 2)
            {
                return build_external_prompt_body(source, object);
            }

            let value = object.get(*column).ok_or_else(|| ExportError::Contract {
                sheet_machine_name,
                message: format!("missing column contract: {column}"),
            })?;
            Ok(stringify_cell(value, sheet_machine_name, column)?)
        })
        .collect()
}

fn stringify_cell(
    value: &Value,
    sheet_machine_name: &'static str,
    column: &'static str,
) -> Result<String, ExportError> {
    Ok(match value {
        Value::Null => String::new(),
        Value::Bool(v) => v.to_string(),
        Value::Number(v) => v.to_string(),
        Value::String(v) => v.clone(),
        _ => {
            return Err(ExportError::Contract {
                sheet_machine_name,
                message: format!("unsupported cell shape for column: {column}"),
            });
        }
    })
}

fn build_external_prompt_body(
    source: &Value,
    prompt_row: &Map<String, Value>,
) -> Result<String, ExportError> {
    let prompt_package_columns = canonical_columns("prompt_package");
    let prompt_package_id =
        required_string(prompt_row, prompt_package_columns[0], "prompt_package")?;
    let source_level = required_string(prompt_row, prompt_package_columns[1], "prompt_package")?;
    let original_body = required_string(prompt_row, prompt_package_columns[2], "prompt_package")?;

    let render_segment_id = extract_render_segment_id(prompt_package_id);
    let scene_summary = sanitize_natural_text(
        find_linked_scene_summary(source, &render_segment_id).unwrap_or("???????????"),
    );
    let action_anchor = sanitize_natural_text(
        find_action_anchor(source, &render_segment_id).unwrap_or("???????????????"),
    );
    let appearance_rule = build_character_appearance_constraint(
        source,
        &scene_summary,
        &action_anchor,
        original_body,
        source_level,
    );
    let style_clause = build_style_lock_clause(original_body);
    let shot_clause = build_shot_clause(source_level, &scene_summary);
    let continuity_clause = build_continuity_clause(&scene_summary, &action_anchor);
    let prompt_intro = if source_level == "layout_prompt" {
        "??????????????????????"
    } else {
        "????????????????????????"
    };

    Ok(format!(
        "{prompt_intro}{}?{}?{}?{}?{}???????{}?{}?{}?",
        appearance_rule.reference_lock_clause,
        appearance_rule.identity_baseline_clause,
        appearance_rule.state_delta_clause,
        style_clause,
        shot_clause,
        action_anchor,
        continuity_clause,
        appearance_rule.negative_guard_clause,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CharacterAppearanceConstraint {
    reference_lock_clause: String,
    identity_baseline_clause: String,
    state_delta_clause: String,
    negative_guard_clause: String,
}

fn build_character_appearance_constraint(
    source: &Value,
    scene_summary: &str,
    action_anchor: &str,
    original_body: &str,
    source_level: &str,
) -> CharacterAppearanceConstraint {
    let style_hint = sanitize_natural_text(original_body);
    let reference_lock_clause =
        "???????????????????????????????????????????????????????????????????????????".to_string();

    let identity_baseline_clause = if style_hint.contains("??") {
        "????????????????????????????????????????????????????????????????????????????????????"
            .to_string()
    } else if scene_summary.contains("??") || action_anchor.contains("??") {
        "????????????????????????????????????????????????????????????????????????????".to_string()
    } else {
        "????????????????????????????????????????????????????????????".to_string()
    };
    let identity_baseline_clause = format!(
        "{}{}",
        identity_baseline_clause,
        facial_detail_clause(source_level)
    );

    let state_delta_clause = format!(
        "????????????????????????????????????????????????????{}",
        scene_summary
    );

    let hard_lock_negative = find_negative_prompt_guard(source)
        .map(sanitize_natural_text)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "?????????????????????????????????????????".to_string());
    let negative_guard_clause = format!(
        "????????????????????????{}??????????????????????????{}",
        hard_lock_negative,
        facial_negative_defaults()
    );

    CharacterAppearanceConstraint {
        reference_lock_clause,
        identity_baseline_clause,
        state_delta_clause,
        negative_guard_clause,
    }
}

fn facial_detail_clause(source_level: &str) -> &'static str {
    if source_level == "layout_prompt" {
        " 补充约束：面部细节保持干净清晰，五官边缘自然稳定，不出现脏污、涂抹或糊脸质感。"
    } else {
        " 补充约束：保持写实皮肤质感与干净清晰的面部细节，眼周、嘴周和鼻梁结构自然稳定，不出现脏污颗粒、涂抹痕迹或糊脸质感。"
    }
}

fn facial_negative_defaults() -> &'static str {
    " 默认负向约束补充：避免脏脸、花脸、糊脸、五官错位、重复眉眼、额外眼耳口鼻、局部涂抹、过度磨皮、压缩噪点和低清晰度面部纹理。"
}

fn build_style_lock_clause(original_body: &str) -> String {
    let style_hint = sanitize_natural_text(original_body);

    if style_hint.is_empty() {
        "??????????????????????????????????????????????????????????????????".to_string()
    } else {
        format!("???????{}????????????????????????????????", style_hint)
    }
}

fn build_shot_clause(source_level: &str, scene_summary: &str) -> String {
    let shot_prefix = if source_level == "layout_prompt" {
        "???????"
    } else {
        "??????????"
    };

    format!(
        "{shot_prefix}??????????????????????????????????????????????????{}",
        scene_summary
    )
}

fn build_continuity_clause(scene_summary: &str, action_anchor: &str) -> String {
    format!(
        "??????????????????????????????????????????????????????????????????????????????????????{}??????????????{}",
        scene_summary, action_anchor
    )
}

fn find_negative_prompt_guard<'a>(source: &'a Value) -> Option<&'a str> {
    let hard_lock_columns = canonical_columns("hard_lock");
    source
        .get("hard_lock")?
        .as_array()?
        .iter()
        .find(|row| {
            row.get(hard_lock_columns[2])
                .and_then(serde_json::Value::as_str)
                == Some("negative_prompt_guard")
        })?
        .get(hard_lock_columns[3])?
        .as_str()
}

fn required_string<'a>(
    object: &'a Map<String, Value>,
    key: &'static str,
    sheet_machine_name: &'static str,
) -> Result<&'a str, ExportError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| ExportError::Contract {
            sheet_machine_name,
            message: format!("missing string column contract: {key}"),
        })
}

fn canonical_columns(sheet_machine_name: &'static str) -> &'static [&'static str] {
    CANONICAL_WORKBOOK_CONTRACT
        .sheets
        .iter()
        .find(|sheet| sheet.machine_name == sheet_machine_name)
        .map(|sheet| sheet.columns)
        .expect("canonical workbook sheet must exist")
}

fn canonical_column(sheet_machine_name: &'static str, index: usize) -> &'static str {
    canonical_columns(sheet_machine_name)
        .get(index)
        .copied()
        .expect("canonical workbook column must exist")
}

fn extract_render_segment_id(prompt_package_id: &str) -> String {
    prompt_package_id
        .split("render-segment-")
        .nth(1)
        .map(|suffix| format!("render-segment-{suffix}"))
        .unwrap_or_else(|| prompt_package_id.to_string())
}

fn find_linked_scene_summary<'a>(source: &'a Value, render_segment_id: &str) -> Option<&'a str> {
    let render_segment_columns = canonical_columns("render_segment");
    let narrative_scene_columns = canonical_columns("narrative_scene");
    let narrative_scene_id = source
        .get("render_segment")?
        .as_array()?
        .iter()
        .find(|row| {
            row.get(render_segment_columns[0]).and_then(Value::as_str) == Some(render_segment_id)
        })?
        .get(render_segment_columns[1])?
        .as_str()?;

    source
        .get("narrative_scene")?
        .as_array()?
        .iter()
        .find(|row| {
            row.get(narrative_scene_columns[0]).and_then(Value::as_str) == Some(narrative_scene_id)
        })?
        .get(narrative_scene_columns[4])?
        .as_str()
}

fn find_action_anchor<'a>(source: &'a Value, render_segment_id: &str) -> Option<&'a str> {
    let cut_columns = canonical_columns("cut");
    source
        .get("cut")?
        .as_array()?
        .iter()
        .find(|row| row.get(cut_columns[1]).and_then(Value::as_str) == Some(render_segment_id))
        .and_then(|row| row.get(cut_columns[3]))
        .and_then(Value::as_str)
}

fn sanitize_natural_text(input: &str) -> String {
    input
        .replace("render-segment-week3-001", "当前连续画面")
        .replace("render-segment-week3-002", "当前连续画面")
        .replace("render-segment-week3-003", "当前连续画面")
        .replace("prompt-package-", "")
        .replace("episode-week3-001", "当前集")
        .replace("episode-week3-002", "当前集")
        .replace("episode-week3-003", "当前集")
        .replace("render-segment-", "")
        .replace("week3-", "")
        .replace("-001", "")
        .replace("-002", "")
        .replace("-003", "")
        .replace("  ", " ")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use core_domain::{
        ProductWarning, PromptTextCompilationStatus, ScenePerformanceProjection,
        SequenceFieldState, SequenceGrouping, ShotGroundingSource, StructureMode,
    };
    use serde_json::json;

    #[test]
    fn external_prompt_body_removes_machine_identifier_and_preserves_traceability_outside_body() {
        let narrative_scene_columns = canonical_columns("narrative_scene");
        let render_segment_columns = canonical_columns("render_segment");
        let cut_columns = canonical_columns("cut");
        let hard_lock_columns = canonical_columns("hard_lock");
        let prompt_package_columns = canonical_columns("prompt_package");

        let source = json!({
            "narrative_scene": [
                json_object(&[
                    (narrative_scene_columns[0], json!("narrative-scene-week3-001")),
                    (narrative_scene_columns[4], json!("clean scene summary")),
                ])
            ],
            "render_segment": [
                json_object(&[
                    (render_segment_columns[0], json!("render-segment-week3-001")),
                    (render_segment_columns[1], json!("narrative-scene-week3-001")),
                ])
            ],
            "cut": [
                json_object(&[
                    (cut_columns[1], json!("render-segment-week3-001")),
                    (cut_columns[3], json!("actor crosses the doorway")),
                ])
            ],
            "hard_lock": [
                json_object(&[
                    (hard_lock_columns[2], json!("negative_prompt_guard")),
                    (hard_lock_columns[3], json!("avoid visual drift")),
                ])
            ]
        });
        let prompt_row = json_object(&[
            (
                prompt_package_columns[0],
                json!("prompt-package-render-episode-week3-001-render-segment-week3-001"),
            ),
            (prompt_package_columns[1], json!("render_prompt")),
            (
                prompt_package_columns[2],
                json!("render prompt body with render-segment-week3-001 machine id"),
            ),
        ]);

        let body = build_external_prompt_body(&source, prompt_row.as_object().unwrap())
            .expect("external prompt should build");

        assert!(!body.contains("render-segment-week3-001"));
        assert!(body.contains("clean scene summary"));
        assert!(body.contains("actor crosses the doorway"));
        assert!(body.contains("render prompt body with"));
        assert!(body.contains("avoid visual drift"));
        assert!(source.to_string().contains("render-segment-week3-001"));
    }

    fn json_object(entries: &[(&str, Value)]) -> Value {
        let mut object = Map::new();
        for (key, value) in entries {
            object.insert((*key).to_string(), value.clone());
        }
        Value::Object(object)
    }

    #[test]
    fn render_face_detail_clause_is_stronger_than_layout_clause() {
        let layout = facial_detail_clause("layout_prompt");
        let render = facial_detail_clause("render_prompt");

        assert!(layout.contains("面部细节"));
        assert!(render.contains("面部细节"));
        assert!(render.contains("写实皮肤质感"));
        assert!(render.contains("眼周、嘴周和鼻梁结构"));
    }

    #[test]
    fn appearance_constraint_appends_face_cleanup_defaults() {
        let constraint = build_character_appearance_constraint(
            &json!({}),
            "同一场景推进",
            "人物保持中景推进",
            "克制写实",
            "render_prompt",
        );

        assert!(constraint.identity_baseline_clause.contains("面部细节"));
        assert!(constraint.negative_guard_clause.contains("脏脸"));
        assert!(constraint.negative_guard_clause.contains("五官错位"));
        assert!(
            constraint
                .negative_guard_clause
                .contains("低清晰度面部纹理")
        );
    }
    #[test]
    fn export_v120_storyboard_bundle_writes_ready_structured_artifacts() {
        let enriched_visual_description =
            "主体为两人对话，近景正侧面构图；桥边碎石和冷雾压紧空间；当前视觉事件是视线停住后的一次克制点头；画面突出出发前的静默拉扯感。"
                .to_string();
        let request = V120StoryboardExportRequest {
            export_manifest_id: "storyboard-export-152".to_string(),
            result_id: "storyboard-152".to_string(),
            selected_total_duration_seconds: 10,
            source_result_id: "storyboard-152".to_string(),
            edited_rows_applied: true,
            rows: vec![GeneratedStoryboardRow {
                shot_id: "GS-BRIDGE-001".to_string(),
                order: 1,
                shot_script: "Two leads hold a restrained dialogue beat.".to_string(),
                primary_scene_type: "daily_dialogue".to_string(),
                primary_scene_label: "Daily Dialogue".to_string(),
                primary_scene_category: "dialogue".to_string(),
                shot_scene_type: "daily_dialogue".to_string(),
                shot_scene_label: "Daily Dialogue".to_string(),
                shot_intent: "dialogue".to_string(),
                adaptation_reason: String::new(),
                grounding_source: ShotGroundingSource::ShotScript,
                person: "lead_pair".to_string(),
                shot_title: "Bridge dialogue sample".to_string(),
                scene_scale: "MCU".to_string(),
                camera_movement: "MCU static camera observes the dialogue beat.".to_string(),
                visual_description: enriched_visual_description.clone(),
                character_action: "One lead answers with a quiet nod.".to_string(),
                dialogue: "We move before dawn.".to_string(),
                prompt_text: "鏅ご鎻愮ず璇?Seedance2.0 stub".to_string(),
                prompt_text_compilation_status: PromptTextCompilationStatus::ReadyStub,
                prompt_text_compilation_warnings: vec![ProductWarning {
                    code: "seedance_prompt_text_stub".to_string(),
                    message: "prompt_text is compiled by deterministic stub".to_string(),
                    related_sample_id: Some("GS-BRIDGE-001".to_string()),
                }],
                prompt_text_source_row_id: "GS-BRIDGE-001".to_string(),
                duration_seconds: 8,
                shot_duration_seconds: 8,
                duration_source: "storyboard_duration_plan.allocated_row_duration_seconds"
                    .to_string(),
                scene_performance_projection: ScenePerformanceProjection {
                    source_sample_id: "GS-BRIDGE-001".to_string(),
                    source_sample_title: "Bridge dialogue sample".to_string(),
                    scene_scale: "MCU".to_string(),
                    person: "lead_pair".to_string(),
                    visual_description: enriched_visual_description.clone(),
                    character_action: "One lead answers with a quiet nod.".to_string(),
                    fused_source_text: "Dialogue bridge evidence".to_string(),
                    sequence_grouping: SequenceGrouping {
                        structure_mode: StructureMode::SingleShot,
                        sequence_id: None,
                        shot_order: None,
                        sequence_field_state: SequenceFieldState::NotApplicable,
                    },
                },
                external_reference_handle_candidates: vec![],
                sequence_grouping: SequenceGrouping {
                    structure_mode: StructureMode::SingleShot,
                    sequence_id: None,
                    shot_order: None,
                    sequence_field_state: SequenceFieldState::NotApplicable,
                },
            }],
        };

        let bundle =
            export_v120_storyboard_bundle(&request).expect("v120 storyboard export should succeed");

        assert_eq!(bundle.workbook.sheets.len(), 1);
        assert_eq!(bundle.workbook.sheets[0].rows.len(), 1);
        assert_eq!(
            &bundle.workbook.sheets[0].columns[..V120_STORYBOARD_COLUMNS.len()],
            V120_STORYBOARD_COLUMNS
        );
        assert_eq!(bundle.workbook.sheets[0].rows[0][14], enriched_visual_description);
        assert_eq!(bundle.artifacts.len(), 3);
        assert!(
            bundle
                .artifacts
                .iter()
                .all(|artifact| artifact.row_count == 1 && artifact.path.is_file())
        );

        let json_artifact = bundle
            .artifacts
            .iter()
            .find(|artifact| artifact.artifact_kind == "storyboard_json")
            .expect("json artifact should exist");
        let json_text =
            fs::read_to_string(&json_artifact.path).expect("json artifact should be readable");

        assert!(json_text.contains("Bridge dialogue sample"));
        assert!(json_text.contains("运镜"));
        assert!(json_text.contains("MCU static camera observes the dialogue beat."));
        assert!(json_text.contains("桥边碎石和冷雾压紧空间"));
        assert!(json_text.contains("ReadyStub"));
        assert!(json_text.contains("seedance_prompt_text_stub"));
        assert!(json_text.contains("storyboard-152"));
        assert!(json_text.contains("true"));
        assert!(json_text.contains("10"));
        assert!(!json_text.contains("raw prompt body should stay out of prompt_text"));
    }
}

fn render_json(workbook: &WorkbookManifest) -> String {
    let mut root = Map::new();
    root.insert(
        "workbook_machine_name".to_string(),
        Value::String(workbook.workbook_machine_name.to_string()),
    );
    root.insert(
        "workbook_chinese_name".to_string(),
        Value::String(workbook.workbook_chinese_name.to_string()),
    );

    for sheet in &workbook.sheets {
        let rows = sheet
            .rows
            .iter()
            .map(|row| {
                let mut map = Map::new();
                for (column, value) in sheet.columns.iter().zip(row.iter()) {
                    map.insert((*column).to_string(), Value::String(value.clone()));
                }
                Value::Object(map)
            })
            .collect();
        root.insert(sheet.machine_name.to_string(), Value::Array(rows));
    }

    serde_json::to_string_pretty(&Value::Object(root)).expect("json rendering should succeed")
}

fn render_markdown(workbook: &WorkbookManifest) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!("# {}\n\n", workbook.workbook_chinese_name));
    markdown.push_str(&format!(
        "- workbook machine name: `{}`\n",
        workbook.workbook_machine_name
    ));
    markdown.push_str("- workbook is sourced from the shared fixture and validation report\n\n");

    for sheet in &workbook.sheets {
        markdown.push_str(&format!(
            "## {} / `{}`\n\n",
            sheet.chinese_name, sheet.machine_name
        ));
        markdown.push_str("| ");
        markdown.push_str(&sheet.columns.join(" | "));
        markdown.push_str(" |\n|");
        markdown.push_str(
            &sheet
                .columns
                .iter()
                .map(|_| "---")
                .collect::<Vec<_>>()
                .join("|"),
        );
        markdown.push_str("|\n");
        for row in &sheet.rows {
            markdown.push_str("| ");
            markdown.push_str(&row.join(" | "));
            markdown.push_str(" |\n");
        }
        markdown.push('\n');
    }

    markdown
}

fn render_csv(sheet: &WorkbookSheetManifest) -> String {
    let mut csv = String::new();
    csv.push_str(&render_csv_row(
        &sheet
            .columns
            .iter()
            .map(|column| (*column).to_string())
            .collect::<Vec<_>>(),
    ));
    csv.push('\n');

    for row in &sheet.rows {
        csv.push_str(&render_csv_row(row));
        csv.push('\n');
    }

    csv
}

fn render_csv_row(values: &[String]) -> String {
    values
        .iter()
        .map(|value| escape_csv(value))
        .collect::<Vec<_>>()
        .join(",")
}

fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn render_xlsx(workbook: &WorkbookManifest) -> Vec<u8> {
    let mut entries = Vec::new();
    entries.push(ZipEntry::new(
        "[Content_Types].xml",
        render_content_types(workbook),
    ));
    entries.push(ZipEntry::new("_rels/.rels", render_root_rels()));
    entries.push(ZipEntry::new(
        "xl/workbook.xml",
        render_workbook_xml(workbook),
    ));
    entries.push(ZipEntry::new(
        "xl/_rels/workbook.xml.rels",
        render_workbook_rels(workbook),
    ));
    entries.push(ZipEntry::new("xl/styles.xml", render_styles_xml()));

    for (sheet_index, sheet) in workbook.sheets.iter().enumerate() {
        entries.push(ZipEntry::new(
            &format!("xl/worksheets/sheet{}.xml", sheet_index + 1),
            render_sheet_xml(sheet),
        ));
    }

    write_zip(entries)
}

fn render_content_types(workbook: &WorkbookManifest) -> Vec<u8> {
    let mut xml = String::new();
    xml.push_str(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>"#,
    );

    for index in 0..workbook.sheets.len() {
        xml.push_str(&format!(
            "\n  <Override PartName=\"/xl/worksheets/sheet{}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>",
            index + 1
        ));
    }

    xml.push_str("\n</Types>\n");
    xml.into_bytes()
}

fn render_root_rels() -> Vec<u8> {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>
"#;
    xml.as_bytes().to_vec()
}

fn render_workbook_xml(workbook: &WorkbookManifest) -> Vec<u8> {
    let mut xml = String::new();
    xml.push_str(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <bookViews>
    <workbookView activeTab="0"/>
  </bookViews>
  <sheets>"#,
    );

    for (index, sheet) in workbook.sheets.iter().enumerate() {
        xml.push_str(&format!(
            "\n    <sheet name=\"{}\" sheetId=\"{}\" r:id=\"rId{}\"/>",
            escape_xml(sheet.machine_name),
            index + 1,
            index + 2
        ));
    }

    xml.push_str("\n  </sheets>\n</workbook>\n");
    xml.into_bytes()
}

fn render_workbook_rels(workbook: &WorkbookManifest) -> Vec<u8> {
    let mut xml = String::new();
    xml.push_str(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#,
    );

    for (index, _) in workbook.sheets.iter().enumerate() {
        xml.push_str(&format!(
            "\n  <Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet{}.xml\"/>",
            index + 2,
            index + 1
        ));
    }

    xml.push_str("\n</Relationships>\n");
    xml.into_bytes()
}

fn render_styles_xml() -> Vec<u8> {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <fonts count="1">
    <font>
      <sz val="11"/>
      <color theme="1"/>
      <name val="Calibri"/>
      <family val="2"/>
    </font>
  </fonts>
  <fills count="1">
    <fill>
      <patternFill patternType="none"/>
    </fill>
  </fills>
  <borders count="1">
    <border>
      <left/>
      <right/>
      <top/>
      <bottom/>
      <diagonal/>
    </border>
  </borders>
  <cellStyleXfs count="1">
    <xf numFmtId="0" fontId="0" fillId="0" borderId="0"/>
  </cellStyleXfs>
  <cellXfs count="1">
    <xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>
  </cellXfs>
  <cellStyles count="1">
    <cellStyle name="Normal" xfId="0" builtinId="0"/>
  </cellStyles>
</styleSheet>
"#;
    xml.as_bytes().to_vec()
}

fn render_sheet_xml(sheet: &WorkbookSheetManifest) -> Vec<u8> {
    let mut xml = String::new();
    xml.push_str(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>"#,
    );

    let mut all_rows = Vec::new();
    all_rows.push(
        sheet
            .columns
            .iter()
            .map(|column| column.to_string())
            .collect::<Vec<_>>(),
    );
    all_rows.extend(sheet.rows.iter().cloned());

    for (row_index, row) in all_rows.iter().enumerate() {
        let excel_row = row_index + 1;
        xml.push_str(&format!("\n    <row r=\"{}\">", excel_row));
        for (column_index, value) in row.iter().enumerate() {
            let cell_ref = format!("{}{}", excel_column_name(column_index), excel_row);
            xml.push_str(&format!(
                "\n      <c r=\"{}\" t=\"inlineStr\"><is><t>{}</t></is></c>",
                cell_ref,
                escape_xml(value)
            ));
        }
        xml.push_str("\n    </row>");
    }

    xml.push_str("\n  </sheetData>\n</worksheet>\n");
    xml.into_bytes()
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&apos;")
}

fn excel_column_name(index: usize) -> String {
    let mut n = index + 1;
    let mut name = String::new();
    while n > 0 {
        let rem = (n - 1) % 26;
        name.insert(0, (b'A' + rem as u8) as char);
        n = (n - 1) / 26;
    }
    name
}

struct ZipEntry {
    name: String,
    data: Vec<u8>,
}

impl ZipEntry {
    fn new(name: &str, data: Vec<u8>) -> Self {
        Self {
            name: name.to_string(),
            data,
        }
    }
}

fn write_zip(entries: Vec<ZipEntry>) -> Vec<u8> {
    let mut output = Vec::new();
    let mut central_directory = Vec::new();
    let mut offset = 0u32;

    for entry in entries {
        let crc = crc32(&entry.data);
        let name_bytes = entry.name.as_bytes();
        let name_len = name_bytes.len() as u16;
        let data_len = entry.data.len() as u32;

        write_u32(&mut output, 0x0403_4b50);
        write_u16(&mut output, 20);
        write_u16(&mut output, 0);
        write_u16(&mut output, 0);
        write_u16(&mut output, 0);
        write_u16(&mut output, 0);
        write_u32(&mut output, crc);
        write_u32(&mut output, data_len);
        write_u32(&mut output, data_len);
        write_u16(&mut output, name_len);
        write_u16(&mut output, 0);
        output.extend_from_slice(name_bytes);
        output.extend_from_slice(&entry.data);

        write_u32(&mut central_directory, 0x0201_4b50);
        write_u16(&mut central_directory, 20);
        write_u16(&mut central_directory, 20);
        write_u16(&mut central_directory, 0);
        write_u16(&mut central_directory, 0);
        write_u16(&mut central_directory, 0);
        write_u16(&mut central_directory, 0);
        write_u32(&mut central_directory, crc);
        write_u32(&mut central_directory, data_len);
        write_u32(&mut central_directory, data_len);
        write_u16(&mut central_directory, name_len);
        write_u16(&mut central_directory, 0);
        write_u16(&mut central_directory, 0);
        write_u16(&mut central_directory, 0);
        write_u16(&mut central_directory, 0);
        write_u32(&mut central_directory, 0);
        write_u32(&mut central_directory, offset);
        central_directory.extend_from_slice(name_bytes);

        offset = output.len() as u32;
    }

    let central_offset = output.len() as u32;
    let central_size = central_directory.len() as u32;
    output.extend_from_slice(&central_directory);

    let entry_count = entries_count(&output) as u16;

    write_u32(&mut output, 0x0605_4b50);
    write_u16(&mut output, 0);
    write_u16(&mut output, 0);
    write_u16(&mut output, entry_count);
    write_u16(&mut output, entry_count);
    write_u32(&mut output, central_size);
    write_u32(&mut output, central_offset);
    write_u16(&mut output, 0);

    output
}

fn entries_count(output: &[u8]) -> usize {
    let mut count = 0usize;
    let mut idx = 0usize;
    while idx + 4 <= output.len() {
        if &output[idx..idx + 4] == b"PK\x03\x04" {
            count += 1;
        }
        idx += 1;
    }
    count
}

fn write_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}
