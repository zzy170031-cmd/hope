#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KbSnapshotRecord {
    pub snapshot_id: String,
    pub snapshot_hash: String,
    pub seed_format: String,
    pub source_name: String,
    pub created_at_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KbRuntimeSummary {
    pub snapshot_id: String,
    pub snapshot_path: String,
    pub mirror_tables: Vec<String>,
    pub has_scene_taxonomy: bool,
    pub has_failure_patterns: bool,
    pub has_repair_template_mapping: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptTemplateRecord {
    pub prompt_template_id: String,
    pub stage: String,
    pub name: String,
    pub target_model_family: String,
    pub repairs_failure_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairTemplateLink {
    pub failure_code: String,
    pub prompt_template_id: String,
    pub prompt_stage: Option<String>,
    pub prompt_name: Option<String>,
}
