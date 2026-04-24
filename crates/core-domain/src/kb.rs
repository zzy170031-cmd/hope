use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KbSnapshotRecord {
    pub snapshot_id: String,
    pub snapshot_hash: String,
    pub seed_format: String,
    pub source_name: String,
    pub created_at_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KbRuntimeSummary {
    pub snapshot_id: String,
    pub snapshot_path: String,
    pub mirror_tables: Vec<String>,
    pub has_scene_taxonomy: bool,
    pub has_failure_patterns: bool,
    pub has_repair_template_mapping: bool,
    pub has_golden_sample_v120_package: bool,
    pub golden_sample_record_count: usize,
    pub golden_sample_source_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KbBundleManifestRecord {
    pub snapshot_version: String,
    pub snapshot_name: String,
    pub content_hash_algo: String,
    pub content_hash: String,
    pub seed_import_format: String,
    pub imported_at: String,
    pub primary_key: String,
    pub bundle_order: Vec<String>,
    pub record_counts: KbBundleRecordCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KbBundleRecordCounts {
    pub golden_sample_library: usize,
    pub golden_sample_field_coverage_rules: usize,
    pub golden_sample_failure_mapping: usize,
    pub golden_sample_repair_mapping: usize,
    pub golden_sample_sources: usize,
    pub golden_sample_provenance_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KbGoldenSampleRuntimePackage {
    pub manifest: KbBundleManifestRecord,
    pub golden_sample_library: GoldenSampleLibraryAsset,
    pub field_coverage_rules: GoldenSampleFieldCoverageRuleAsset,
    pub failure_mapping: GoldenSampleFailureMappingAsset,
    pub repair_mapping: GoldenSampleRepairMappingAsset,
    pub source_register: GoldenSampleSourceRegister,
}

impl KbGoldenSampleRuntimePackage {
    pub fn manifest_golden_sample_record_count(&self) -> usize {
        self.manifest.record_counts.golden_sample_library
    }

    pub fn official_record_count(&self) -> usize {
        self.golden_sample_library
            .records
            .iter()
            .filter(|record| record.is_official())
            .count()
    }

    pub fn reserve_record_count(&self) -> usize {
        self.golden_sample_library
            .records
            .iter()
            .filter(|record| record.is_reserve())
            .count()
    }

    pub fn positive_fewshot_record_count(&self) -> usize {
        self.golden_sample_library
            .records
            .iter()
            .filter(|record| record.is_positive_fewshot_candidate())
            .count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleLibraryAsset {
    pub schema_version: String,
    pub asset_name: String,
    pub generated_at: String,
    pub source: GoldenSampleAssetSource,
    pub record_count: usize,
    pub source_field_order: Vec<String>,
    pub records: Vec<GoldenSampleLibraryRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleAssetSource {
    pub primary_workbook: String,
    pub primary_workbook_sha256: String,
    pub control_memo: String,
    pub control_memo_sha256: String,
    pub control_review: String,
    pub control_dispatch: String,
    pub comparison_baseline_source_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleLibraryRecord {
    pub machine_id: String,
    pub sample_id: String,
    pub schema_version: String,
    pub source_fields: GoldenSampleSourceFields,
    pub provenance: GoldenSampleLibraryProvenance,
    pub classification: GoldenSampleClassification,
    pub fewshot: GoldenSampleFewshotState,
    pub validator_evidence: GoldenSampleValidatorEvidence,
    pub negative_sample: GoldenSampleNegativeSample,
    pub v3_core_coverage: GoldenSampleV3CoreCoverage,
    pub repair_mapping_planning: GoldenSampleRepairMappingPlanning,
}

impl GoldenSampleLibraryRecord {
    pub fn is_official(&self) -> bool {
        self.classification
            .library_status
            .eq_ignore_ascii_case("official")
    }

    pub fn is_reserve(&self) -> bool {
        self.classification
            .library_status
            .eq_ignore_ascii_case("reserve")
    }

    pub fn is_positive_fewshot_candidate(&self) -> bool {
        self.is_official()
            && self.classification.usable_for_fewshot
            && self.fewshot.eligible
            && self.fewshot.source_value.eq_ignore_ascii_case("Yes")
            && self
                .source_fields
                .usable_for_fewshot
                .eq_ignore_ascii_case("Yes")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleSourceFields {
    pub shot_id: String,
    pub library_status: String,
    pub reserve_reason: String,
    pub sample_type: String,
    pub sequence_id: String,
    pub shot_order: String,
    pub sample_title: String,
    pub style_cluster: String,
    pub scene_category: String,
    pub scene_tag: String,
    pub quality_grade: String,
    pub usable_for_fewshot: String,
    pub technical_profile: String,
    pub scene_performance_core: String,
    pub camera_directing_core: String,
    pub audio_directing_core: String,
    pub continuity_negative_core: String,
    pub reference_bundle: String,
    pub ip_abstraction_note: String,
    pub covered_points: String,
    pub missed_points: String,
    pub teaching_note: String,
    pub prompt_body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleLibraryProvenance {
    pub source_workbook: String,
    pub source_workbook_sha256: String,
    pub source_control_memo: String,
    pub source_control_memo_sha256: String,
    pub source_row_index: u32,
    pub source_library_status: String,
    pub source_sample_type: String,
    pub comparison_baseline_source_id: String,
    pub control_review_doc: String,
    pub control_dispatch_doc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleClassification {
    pub core: String,
    pub library_status: String,
    pub sample_type: String,
    pub sequence_id: String,
    pub shot_order: String,
    pub style_cluster: String,
    pub scene_category: String,
    pub scene_tags: Vec<String>,
    pub quality_grade: String,
    pub usable_for_fewshot: bool,
    pub coverage_surfaces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleFewshotState {
    pub eligible: bool,
    pub source_value: String,
    pub retrieval_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleValidatorEvidence {
    pub covered_points: String,
    pub missed_points: String,
    pub teaching_note: String,
    pub source_quality_grade: String,
    pub source_library_status: String,
    pub source_sample_type: String,
    pub has_coverage_gap: bool,
    pub has_placeholder_signal: bool,
    pub surface_completeness: BTreeMap<String, bool>,
    pub reference_bundle_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleNegativeSample {
    pub is_negative_sample: bool,
    pub signal_codes: Vec<String>,
    pub reserve_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleV3CoreCoverage {
    pub core: String,
    pub coverage_rule_ids: Vec<String>,
    pub source_coverage_statement: String,
    pub source_missing_statement: String,
    pub source_field_presence: BTreeMap<String, bool>,
    pub comparison_baseline: GoldenSampleComparisonBaseline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleComparisonBaseline {
    pub primary_source_id: String,
    pub comparison_source_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleRepairMappingPlanning {
    pub failure_mapping_id: String,
    pub repair_mapping_id: String,
    pub planning_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleFieldCoverageRuleAsset {
    pub schema_version: String,
    pub asset_name: String,
    pub generated_at: String,
    pub source_context: GoldenSampleSourceContext,
    pub record_count: usize,
    pub records: Vec<GoldenSampleFieldCoverageRuleRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleSourceContext {
    pub primary_source_id: String,
    pub comparison_source_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleFieldCoverageRuleRecord {
    pub rule_id: String,
    pub core: String,
    pub row_count: usize,
    pub source_sample_ids: Vec<String>,
    pub required_source_fields: Vec<String>,
    pub validator_evidence_fields: Vec<String>,
    pub fewshot_gate: GoldenSampleFewshotGate,
    pub negative_sample_gate: GoldenSampleNegativeSampleGate,
    pub coverage_summary: GoldenSampleCoverageSummary,
    pub future_gate_owner: Vec<String>,
    pub planning_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleFewshotGate {
    pub eligible_source_field: String,
    pub positive_value: String,
    pub negative_value: String,
    pub library_status_field: String,
    pub official_value: String,
    pub reserve_value: String,
    pub reserve_rows_excluded_from_positive_fewshot: bool,
    pub negative_quality_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleNegativeSampleGate {
    pub quality_field: String,
    pub negative_quality_value: String,
    pub library_status_field: String,
    pub reserve_value: String,
    pub reserve_reason_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleCoverageSummary {
    pub library_status: BTreeMap<String, u32>,
    pub quality_grade: BTreeMap<String, u32>,
    pub sample_type: BTreeMap<String, u32>,
    pub scene_category: BTreeMap<String, u32>,
    pub sequence_groups: BTreeMap<String, u32>,
    pub style_cluster: BTreeMap<String, u32>,
    pub surface_completeness: BTreeMap<String, u32>,
    pub usable_for_fewshot: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleFailureMappingAsset {
    pub schema_version: String,
    pub asset_name: String,
    pub generated_at: String,
    pub source_context: GoldenSampleSourceContext,
    pub failure_code_definitions: Vec<GoldenSampleFailureCodeDefinition>,
    pub record_count: usize,
    pub records: Vec<GoldenSampleFailureMappingRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleFailureCodeDefinition {
    pub failure_code: String,
    pub source_fields: Vec<String>,
    pub meaning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleFailureMappingRecord {
    pub mapping_id: String,
    pub sample_id: String,
    pub core: String,
    pub tier: String,
    pub usable_for_fewshot: String,
    pub negative_sample_signal: bool,
    pub planned_failure_codes: Vec<String>,
    pub validator_evidence: GoldenSampleFailureValidatorEvidence,
    pub source_field_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleFailureValidatorEvidence {
    pub covered_points: String,
    pub missed_points: String,
    pub teaching_note: String,
    pub library_status: String,
    pub sample_type: String,
    pub sequence_id: String,
    pub shot_order: String,
    pub reference_bundle_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleRepairMappingAsset {
    pub schema_version: String,
    pub asset_name: String,
    pub generated_at: String,
    pub source_context: GoldenSampleSourceContext,
    pub record_count: usize,
    pub records: Vec<GoldenSampleRepairMappingRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleRepairMappingRecord {
    pub mapping_id: String,
    pub sample_id: String,
    pub core: String,
    pub repair_planning_mode: String,
    pub linked_failure_mapping_id: String,
    pub planned_repair_inputs: GoldenSampleRepairInputs,
    pub future_repair_gate: String,
    pub planning_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleRepairInputs {
    pub library_status: String,
    pub missed_points: String,
    pub reserve_reason: String,
    pub sample_type: String,
    pub scene_category: String,
    pub sequence_id: String,
    pub style_cluster: String,
    pub teaching_note: String,
    pub usable_for_fewshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleSourceRegister {
    pub schema_version: String,
    pub register_name: String,
    pub updated_at: String,
    pub sources: Vec<GoldenSampleSourceRegisterEntry>,
    pub provenance_entries: Vec<GoldenSampleSourceRegisterProvenance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleSourceRegisterEntry {
    pub source_id: String,
    pub source_type: String,
    pub label: String,
    pub path: String,
    pub sha256: Option<String>,
    pub applies_to: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenSampleSourceRegisterProvenance {
    pub provenance_id: String,
    pub source_ids: Vec<String>,
    pub generated_files: Vec<String>,
    pub preservation_contract: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneTaxonomyRecord {
    pub scene_taxonomy_id: String,
    pub scene_type: String,
    pub display_name: String,
    pub definition: String,
    pub default_duration_band: String,
    pub typical_committee_roles: Vec<String>,
    pub default_handoff_out: Vec<String>,
    pub risk_flags: Vec<String>,
    pub continuity_priority: String,
    pub prompt_focus: Vec<String>,
    pub source_type: String,
    pub source_notes: String,
    pub confidence_level: String,
    pub last_reviewed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptTemplateRecord {
    pub prompt_template_id: String,
    pub stage: String,
    pub name: String,
    pub target_model_family: String,
    pub repairs_failure_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FailurePatternRecord {
    pub failure_pattern_id: String,
    pub failure_code: String,
    pub failure_name: String,
    pub failure_category: String,
    pub symptom: String,
    pub common_causes: Vec<String>,
    pub detection_hint: String,
    pub repair_strategy: String,
    pub affected_layers: Vec<String>,
    pub validator_hint: String,
    pub repair_template_ids: Vec<String>,
    pub repair_priority: String,
    pub repair_scope: String,
    pub suggested_followup_validators: Vec<String>,
    pub source_type: String,
    pub source_notes: String,
    pub confidence_level: String,
    pub last_reviewed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepairTemplateLink {
    pub failure_code: String,
    pub prompt_template_id: String,
    pub prompt_stage: Option<String>,
    pub prompt_name: Option<String>,
}
