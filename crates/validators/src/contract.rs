#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationDecision {
    Pass,
    Warn,
    Block,
}

impl ValidationDecision {
    pub fn is_block(self) -> bool {
        matches!(self, Self::Block)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Warn,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationFinding {
    pub code: &'static str,
    pub severity: ValidationSeverity,
    pub subject: &'static str,
    pub message: &'static str,
    pub failure_sample: &'static str,
}

impl ValidationFinding {
    pub fn warn(
        code: &'static str,
        subject: &'static str,
        message: &'static str,
        failure_sample: &'static str,
    ) -> Self {
        Self {
            code,
            severity: ValidationSeverity::Warn,
            subject,
            message,
            failure_sample,
        }
    }

    pub fn block(
        code: &'static str,
        subject: &'static str,
        message: &'static str,
        failure_sample: &'static str,
    ) -> Self {
        Self {
            code,
            severity: ValidationSeverity::Block,
            subject,
            message,
            failure_sample,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationEnvelope {
    pub validation_report_id: String,
    pub project_id: String,
    pub updated_at_timestamp: i64,
}

impl ValidationEnvelope {
    pub fn new(
        validation_report_id: impl Into<String>,
        project_id: impl Into<String>,
        updated_at_timestamp: i64,
    ) -> Self {
        Self {
            validation_report_id: validation_report_id.into(),
            project_id: project_id.into(),
            updated_at_timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationReportRow {
    #[serde(rename = "校验报告标识")]
    pub validation_report_id: String,
    #[serde(rename = "项目标识")]
    pub project_id: String,
    #[serde(rename = "通过")]
    pub passed: bool,
    #[serde(rename = "问题数")]
    pub problem_count: u32,
    #[serde(rename = "更新时间戳")]
    pub updated_at_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    pub envelope: ValidationEnvelope,
    pub findings: Vec<ValidationFinding>,
}

impl ValidationReport {
    pub fn new(envelope: ValidationEnvelope, findings: Vec<ValidationFinding>) -> Self {
        Self { envelope, findings }
    }

    pub fn decision(&self) -> ValidationDecision {
        if self
            .findings
            .iter()
            .any(|finding| matches!(finding.severity, ValidationSeverity::Block))
        {
            ValidationDecision::Block
        } else if self
            .findings
            .iter()
            .any(|finding| matches!(finding.severity, ValidationSeverity::Warn))
        {
            ValidationDecision::Warn
        } else {
            ValidationDecision::Pass
        }
    }

    pub fn problem_count(&self) -> u32 {
        self.findings.len() as u32
    }

    pub fn to_sheet_row(&self) -> ValidationReportRow {
        ValidationReportRow {
            validation_report_id: self.envelope.validation_report_id.clone(),
            project_id: self.envelope.project_id.clone(),
            passed: !self.decision().is_block(),
            problem_count: self.problem_count(),
            updated_at_timestamp: self.envelope.updated_at_timestamp,
        }
    }
}

pub fn contains_placeholder_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();

    lower.contains("todo")
        || lower.contains("tbd")
        || value.contains("__PLACEHOLDER__")
        || value.contains("待补")
        || value.contains("待定")
}

pub fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}
