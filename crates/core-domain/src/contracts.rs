use crate::domain::HierarchyRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelChannel {
    QwenCompatible,
    GptCompatible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptPackageRecord {
    pub prompt_package_id: String,
    pub source: HierarchyRef,
    pub body: String,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRequest {
    pub request_id: String,
    pub channel: ModelChannel,
    pub prompt_body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelResponse {
    pub request_id: String,
    pub body: String,
    pub raw_response: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationSummary {
    pub subject_id: String,
    pub passed: bool,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportSummary {
    pub export_id: String,
    pub workbook_machine_name: String,
    pub workbook_chinese_name: String,
    pub sheet_machine_names: Vec<String>,
}
