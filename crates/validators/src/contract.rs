#![forbid(unsafe_code)]

use core_domain::kb::{GoldenSampleFailureMappingRecord, GoldenSampleLibraryRecord};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceAwareValidationItem {
    pub code: String,
    pub severity: ValidationSeverity,
    pub subject: String,
    pub message: String,
    pub source_sample_id: Option<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EvidenceAwareValidationProjection {
    pub blockers: Vec<EvidenceAwareValidationItem>,
    pub warnings: Vec<EvidenceAwareValidationItem>,
}

impl EvidenceAwareValidationProjection {
    pub fn push_blocker(
        &mut self,
        code: impl Into<String>,
        subject: impl Into<String>,
        message: impl Into<String>,
        source_sample_id: Option<String>,
        evidence_refs: Vec<String>,
    ) {
        self.blockers.push(EvidenceAwareValidationItem {
            code: code.into(),
            severity: ValidationSeverity::Block,
            subject: subject.into(),
            message: message.into(),
            source_sample_id,
            evidence_refs,
        });
    }

    pub fn push_warning(
        &mut self,
        code: impl Into<String>,
        subject: impl Into<String>,
        message: impl Into<String>,
        source_sample_id: Option<String>,
        evidence_refs: Vec<String>,
    ) {
        self.warnings.push(EvidenceAwareValidationItem {
            code: code.into(),
            severity: ValidationSeverity::Warn,
            subject: subject.into(),
            message: message.into(),
            source_sample_id,
            evidence_refs,
        });
    }
}

pub fn project_v120_evidence_aware_findings(
    record: &GoldenSampleLibraryRecord,
    failure_mapping: Option<&GoldenSampleFailureMappingRecord>,
) -> EvidenceAwareValidationProjection {
    let mut projection = EvidenceAwareValidationProjection::default();
    let sample_id = Some(record.sample_id.clone());

    if contains_placeholder_marker(&record.source_fields.prompt_body)
        || record.validator_evidence.has_placeholder_signal
    {
        projection.push_blocker(
            "prompt_body_blocked_by_placeholder",
            "prompt_body_candidate",
            "Prompt body stays blocked until placeholder-bearing source text is removed.",
            sample_id.clone(),
            vec!["prompt_body".to_string(), "validator_evidence".to_string()],
        );
    }

    if !record.source_fields.reference_bundle.trim().is_empty() {
        projection.push_warning(
            "reference_handle_unresolved",
            "external_reference_handle_candidates",
            "Reference bundle may surface name-level handle candidates only; product-ready external handles remain closed.",
            sample_id.clone(),
            vec!["reference_bundle".to_string()],
        );
    }

    if record.validator_evidence.has_coverage_gap
        || !is_blank(&record.validator_evidence.missed_points)
    {
        projection.push_warning(
            "golden_coverage_gap",
            "validator_evidence",
            "Missed points remain evidence-aware warnings and keep the row out of silent runtime promotion.",
            sample_id.clone(),
            vec!["covered_points".to_string(), "missed_points".to_string()],
        );
    }

    if record.negative_sample.is_negative_sample {
        projection.push_warning(
            "golden_negative_sample",
            "negative_sample",
            "Negative-quality rows stay available as evidence only and must not be promoted into positive runtime use.",
            sample_id.clone(),
            vec!["quality_grade".to_string(), "usable_for_fewshot".to_string()],
        );
    }

    if record.classification.library_status == "reserve" {
        projection.push_warning(
            "golden_reserve_sample",
            "library_status",
            "Reserve rows remain governed holdouts and must not silently become runtime truth.",
            sample_id.clone(),
            vec![
                "library_status".to_string(),
                "reserve_reason".to_string(),
                "sequence_id".to_string(),
                "shot_order".to_string(),
            ],
        );
    }

    if !is_blank(&record.source_fields.continuity_negative_core) {
        projection.push_warning(
            "continuity_negative_core_present",
            "continuity_negative_core",
            "Continuity and negative-boundary evidence stays fused and is projected as explainable warnings instead of opaque prompt text.",
            sample_id.clone(),
            vec!["continuity_negative_core".to_string()],
        );
    }

    if let Some(mapping) = failure_mapping {
        for code in &mapping.planned_failure_codes {
            match code.as_str() {
                "golden_coverage_gap" | "golden_negative_sample" | "golden_fewshot_closed"
                | "golden_reserve_sample" => {}
                other => projection.push_warning(
                    other.to_string(),
                    "failure_mapping",
                    "Failure mapping contributes evidence-aware warnings but does not open hidden repair magic in this bridge.",
                    sample_id.clone(),
                    vec!["failure_mapping".to_string()],
                ),
            }
        }
    }

    projection
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use core_domain::kb::{
        GoldenSampleClassification, GoldenSampleComparisonBaseline,
        GoldenSampleFailureMappingRecord, GoldenSampleFailureValidatorEvidence,
        GoldenSampleFewshotState, GoldenSampleLibraryProvenance, GoldenSampleLibraryRecord,
        GoldenSampleNegativeSample, GoldenSampleRepairMappingPlanning, GoldenSampleSourceFields,
        GoldenSampleV3CoreCoverage, GoldenSampleValidatorEvidence,
    };

    use super::{contains_placeholder_marker, project_v120_evidence_aware_findings};

    fn sample_record() -> GoldenSampleLibraryRecord {
        GoldenSampleLibraryRecord {
            machine_id: "golden_sample_test_01".to_string(),
            sample_id: "GS-T-01".to_string(),
            schema_version: "golden_sample_library.v0.2".to_string(),
            source_fields: GoldenSampleSourceFields {
                shot_id: "GS-T-01".to_string(),
                library_status: "reserve".to_string(),
                reserve_reason: "holdout".to_string(),
                sample_type: "single_shot".to_string(),
                sequence_id: String::new(),
                shot_order: String::new(),
                sample_title: "Bridge Test".to_string(),
                style_cluster: "base".to_string(),
                scene_category: "dialogue".to_string(),
                scene_tag: "dialogue,close".to_string(),
                quality_grade: "good".to_string(),
                usable_for_fewshot: "No".to_string(),
                technical_profile: "MCU / 24fps".to_string(),
                scene_performance_core: "A restrained close dialogue beat.".to_string(),
                camera_directing_core: "Hold the line and reaction.".to_string(),
                audio_directing_core: "Breathing and room tone.".to_string(),
                continuity_negative_core: "Do not drift eyeline or props.".to_string(),
                reference_bundle: "char_test(face,strong) scene_test(layout,reference)".to_string(),
                ip_abstraction_note: "abstracted".to_string(),
                covered_points: "eyeline / props".to_string(),
                missed_points: "prompt polish".to_string(),
                teaching_note: "Use as reserve evidence only.".to_string(),
                prompt_body: "TODO: fill prompt".to_string(),
            },
            provenance: GoldenSampleLibraryProvenance {
                source_workbook: "workbook.xlsx".to_string(),
                source_workbook_sha256: "hash".to_string(),
                source_control_memo: "memo.docx".to_string(),
                source_control_memo_sha256: "hash".to_string(),
                source_row_index: 1,
                source_library_status: "reserve".to_string(),
                source_sample_type: "single_shot".to_string(),
                comparison_baseline_source_id: "baseline".to_string(),
                control_review_doc: "review.md".to_string(),
                control_dispatch_doc: "dispatch.md".to_string(),
            },
            classification: GoldenSampleClassification {
                core: "full_stack_single_shot_sample".to_string(),
                library_status: "reserve".to_string(),
                sample_type: "single_shot".to_string(),
                sequence_id: String::new(),
                shot_order: String::new(),
                style_cluster: "base".to_string(),
                scene_category: "dialogue".to_string(),
                scene_tags: vec!["dialogue".to_string()],
                quality_grade: "good".to_string(),
                usable_for_fewshot: false,
                coverage_surfaces: vec!["scene_performance_core".to_string()],
            },
            fewshot: GoldenSampleFewshotState {
                eligible: false,
                source_value: "No".to_string(),
                retrieval_status: "closed".to_string(),
            },
            validator_evidence: GoldenSampleValidatorEvidence {
                covered_points: "eyeline / props".to_string(),
                missed_points: "prompt polish".to_string(),
                teaching_note: "reserve evidence".to_string(),
                source_quality_grade: "good".to_string(),
                source_library_status: "reserve".to_string(),
                source_sample_type: "single_shot".to_string(),
                has_coverage_gap: true,
                has_placeholder_signal: true,
                surface_completeness: BTreeMap::from([(
                    "scene_performance_core".to_string(),
                    true,
                )]),
                reference_bundle_present: true,
            },
            negative_sample: GoldenSampleNegativeSample {
                is_negative_sample: true,
                signal_codes: vec!["coverage_gap".to_string()],
                reserve_reason: "holdout".to_string(),
            },
            v3_core_coverage: GoldenSampleV3CoreCoverage {
                core: "full_stack_single_shot_sample".to_string(),
                coverage_rule_ids: vec!["rule_01".to_string()],
                source_coverage_statement: "covered".to_string(),
                source_missing_statement: "missed".to_string(),
                source_field_presence: BTreeMap::from([(
                    "scene_performance_core".to_string(),
                    true,
                )]),
                comparison_baseline: GoldenSampleComparisonBaseline {
                    primary_source_id: "v120".to_string(),
                    comparison_source_id: "v108".to_string(),
                },
            },
            repair_mapping_planning: GoldenSampleRepairMappingPlanning {
                failure_mapping_id: "failure_map".to_string(),
                repair_mapping_id: "repair_map".to_string(),
                planning_only: true,
            },
        }
    }

    #[test]
    fn projects_placeholder_and_reference_bundle_as_explainable_findings() {
        let record = sample_record();
        let failure_mapping = GoldenSampleFailureMappingRecord {
            mapping_id: "failure_map".to_string(),
            sample_id: "GS-T-01".to_string(),
            core: "full_stack_single_shot_sample".to_string(),
            tier: "good".to_string(),
            usable_for_fewshot: "No".to_string(),
            negative_sample_signal: true,
            planned_failure_codes: vec![
                "golden_coverage_gap".to_string(),
                "golden_negative_sample".to_string(),
            ],
            validator_evidence: GoldenSampleFailureValidatorEvidence {
                covered_points: "covered".to_string(),
                missed_points: "missed".to_string(),
                teaching_note: "teaching".to_string(),
                library_status: "reserve".to_string(),
                sample_type: "single_shot".to_string(),
                sequence_id: String::new(),
                shot_order: String::new(),
                reference_bundle_present: true,
            },
            source_field_refs: vec!["prompt_body".to_string()],
        };

        let projection = project_v120_evidence_aware_findings(&record, Some(&failure_mapping));

        assert!(
            projection
                .blockers
                .iter()
                .any(|item| item.code == "prompt_body_blocked_by_placeholder")
        );
        assert!(
            projection
                .warnings
                .iter()
                .any(|item| item.code == "reference_handle_unresolved")
        );
        assert!(
            projection
                .warnings
                .iter()
                .any(|item| item.code == "golden_reserve_sample")
        );
    }

    #[test]
    fn placeholder_marker_detection_keeps_ascii_and_chinese_guards() {
        assert!(contains_placeholder_marker("TODO: fill me"));
        assert!(contains_placeholder_marker("__PLACEHOLDER__"));
        assert!(!contains_placeholder_marker("stable prompt candidate"));
    }
}
