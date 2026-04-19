use std::fs;
use std::path::PathBuf;

use core_domain::Exporter;
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
pub enum ExportError {
    Io {
        path: &'static str,
        message: String,
    },
    Parse {
        path: &'static str,
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
        let workbook = load_workbook_from_fixtures()?;
        write_export_files(&workbook)?;

        Ok(ExportBundle {
            workbook,
            excel_path: PathBuf::from(WEEK3_EXPORT_XLSX_PATH),
            json_path: PathBuf::from(WEEK3_EXPORT_JSON_PATH),
            markdown_path: PathBuf::from(WEEK3_EXPORT_MARKDOWN_PATH),
        })
    }
}

pub fn fixed_export_request() -> ExportRequest {
    ExportEngine::fixed_request()
}

pub fn export_week3_from_fixtures() -> Result<ExportBundle, ExportError> {
    ExportEngine::export_from_fixtures()
}

impl Exporter for ExportEngine {
    type Input = ExportRequest;
    type Output = ExportBundle;
    type Error = ExportError;

    fn export(&self, _input: &Self::Input) -> Result<Self::Output, Self::Error> {
        ExportEngine::export_from_fixtures()
    }
}

fn load_workbook_from_fixtures() -> Result<WorkbookManifest, ExportError> {
    let shared = read_json(WEEK3_SHARED_FIXTURE_PATH)?;
    let validation = read_json(WEEK3_VALIDATION_REPORT_PATH)?;

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
                WEEK3_VALIDATION_REPORT_PATH
            } else {
                WEEK3_SHARED_FIXTURE_PATH
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

fn write_export_files(workbook: &WorkbookManifest) -> Result<(), ExportError> {
    fs::create_dir_all(WEEK3_EXPORT_DIR).map_err(|error| ExportError::Io {
        path: WEEK3_EXPORT_DIR,
        message: error.to_string(),
    })?;

    fs::write(WEEK3_EXPORT_JSON_PATH, render_json(workbook)).map_err(|error| ExportError::Io {
        path: WEEK3_EXPORT_JSON_PATH,
        message: error.to_string(),
    })?;
    fs::write(WEEK3_EXPORT_MARKDOWN_PATH, render_markdown(workbook)).map_err(|error| {
        ExportError::Io {
            path: WEEK3_EXPORT_MARKDOWN_PATH,
            message: error.to_string(),
        }
    })?;
    fs::write(WEEK3_EXPORT_XLSX_PATH, render_xlsx(workbook)).map_err(|error| ExportError::Io {
        path: WEEK3_EXPORT_XLSX_PATH,
        message: error.to_string(),
    })?;

    Ok(())
}

fn read_json(path: &'static str) -> Result<Value, ExportError> {
    let text = fs::read_to_string(path).map_err(|error| ExportError::Io {
        path,
        message: error.to_string(),
    })?;
    serde_json::from_str(&text).map_err(|error| ExportError::Parse {
        path,
        key: "root",
        message: error.to_string(),
    })
}

fn parse_sheet_rows(
    source: &Value,
    key: &'static str,
    columns: &[&'static str],
    path: &'static str,
) -> Result<Vec<Vec<String>>, ExportError> {
    let rows = source
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| ExportError::Parse {
            path,
            key,
            message: "missing sheet section".to_string(),
        })?;

    rows.iter()
        .map(|row| {
            let object = row.as_object().ok_or_else(|| ExportError::Parse {
                path,
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
            if sheet_machine_name == "prompt_package" && *column == "正文" {
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
    let prompt_package_id = required_string(prompt_row, "PromptPackage??", "prompt_package")?;
    let source_level = required_string(prompt_row, "????", "prompt_package")?;
    let original_body = required_string(prompt_row, "??", "prompt_package")?;

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

    let state_delta_clause = format!(
        "????????????????????????????????????????????????????{}",
        scene_summary
    );

    let hard_lock_negative = find_negative_prompt_guard(source)
        .map(sanitize_natural_text)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "?????????????????????????????????????????".to_string());
    let negative_guard_clause = format!(
        "????????????????????????{}??????????????????????????",
        hard_lock_negative
    );

    CharacterAppearanceConstraint {
        reference_lock_clause,
        identity_baseline_clause,
        state_delta_clause,
        negative_guard_clause,
    }
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
    source
        .get("hard_lock")?
        .as_array()?
        .iter()
        .find(|row| {
            row.get("??").and_then(serde_json::Value::as_str) == Some("negative_prompt_guard")
        })?
        .get("??")?
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

fn extract_render_segment_id(prompt_package_id: &str) -> String {
    prompt_package_id
        .split("render-segment-")
        .nth(1)
        .map(|suffix| format!("render-segment-{suffix}"))
        .unwrap_or_else(|| prompt_package_id.to_string())
}

fn find_linked_scene_summary<'a>(source: &'a Value, render_segment_id: &str) -> Option<&'a str> {
    let narrative_scene_id = source
        .get("render_segment")?
        .as_array()?
        .iter()
        .find(|row| {
            row.get("RenderSegment标识").and_then(Value::as_str) == Some(render_segment_id)
        })?
        .get("叙事场景标识")?
        .as_str()?;

    source
        .get("narrative_scene")?
        .as_array()?
        .iter()
        .find(|row| row.get("叙事场景标识").and_then(Value::as_str) == Some(narrative_scene_id))?
        .get("内容摘要")?
        .as_str()
}

fn find_action_anchor<'a>(source: &'a Value, render_segment_id: &str) -> Option<&'a str> {
    source
        .get("cut")?
        .as_array()?
        .iter()
        .find(|row| row.get("RenderSegment标识").and_then(Value::as_str) == Some(render_segment_id))
        .and_then(|row| row.get("镜头描述"))
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
    use serde_json::json;

    #[test]
    fn external_prompt_body_removes_machine_identifier_and_preserves_traceability_outside_body() {
        let source = json!({
            "narrative_scene": [
                {
                    "??????": "narrative-scene-week3-001",
                    "????": "??????????????"
                }
            ],
            "render_segment": [
                {
                    "RenderSegment??": "render-segment-week3-001",
                    "??????": "narrative-scene-week3-001"
                }
            ],
            "cut": [
                {
                    "RenderSegment??": "render-segment-week3-001",
                    "????": "?????????????????????"
                }
            ],
            "hard_lock": [
                {
                    "??": "negative_prompt_guard",
                    "??": "???????????????"
                }
            ]
        });
        let prompt_row = json!({
            "PromptPackage??": "prompt-package-render-episode-week3-001-render-segment-week3-001",
            "????": "render_prompt",
            "??": "???????????????????? render-segment-week3-001 ?????"
        });

        let body = build_external_prompt_body(&source, prompt_row.as_object().unwrap())
            .expect("external prompt should build");

        assert!(!body.contains("render-segment-week3-001"));
        assert!(body.contains("?????"));
        assert!(body.contains("???????"));
        assert!(body.contains("???????"));
        assert!(body.contains("??????"));
        assert!(body.contains("??????"));
        assert!(body.contains("????"));
        assert!(body.contains("???"));
        assert!(source.to_string().contains("render-segment-week3-001"));
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
