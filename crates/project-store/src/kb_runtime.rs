use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use core_domain::kb::{
    FailurePatternRecord, GoldenSampleFailureMappingAsset, GoldenSampleFieldCoverageRuleAsset,
    GoldenSampleLibraryAsset, GoldenSampleRepairMappingAsset, GoldenSampleSourceRegister,
    KbBundleManifestRecord, KbGoldenSampleRuntimePackage, KbRuntimeSummary, KbSnapshotRecord,
    PromptTemplateRecord, RepairTemplateLink,
};
use serde::Deserialize;

pub const HOPE_KB_MIRROR_TABLES: &[&str] = &[
    "kb_snapshot",
    "director_profile",
    "director_cut_sample",
    "committee_template",
    "committee_handoff_rule",
    "visual_term",
    "cinematography_term",
    "continuity_rule",
    "scene_taxonomy",
    "prompt_template",
    "failure_pattern",
    "seed_import_batch",
    "golden_sample_library",
    "golden_sample_field_coverage_rule",
    "golden_sample_failure_mapping",
    "golden_sample_repair_mapping",
    "golden_sample_source",
    "golden_sample_provenance",
];

pub const GOLDEN_SAMPLE_V120_PACKAGE_FILES: &[&str] = &[
    "manifest.json",
    "golden_sample_library.json",
    "golden_sample_field_coverage_rules.json",
    "golden_sample_failure_mapping.json",
    "golden_sample_repair_mapping.json",
    "source_register.json",
];

pub const SCENE_TAXONOMY_COLUMNS: &[&str] = &[
    "scene_taxonomy_id",
    "scene_type",
    "display_name",
    "definition",
    "default_duration_band",
    "typical_committee_roles",
    "default_handoff_out",
    "risk_flags",
    "continuity_priority",
    "prompt_focus",
    "source_type",
    "source_notes",
    "confidence_level",
    "last_reviewed_at",
];

pub const FAILURE_PATTERN_COLUMNS: &[&str] = &[
    "failure_pattern_id",
    "failure_code",
    "failure_name",
    "failure_category",
    "symptom",
    "common_causes",
    "detection_hint",
    "repair_strategy",
    "affected_layers",
    "validator_hint",
    "repair_template_ids",
    "repair_priority",
    "repair_scope",
    "suggested_followup_validators",
    "source_type",
    "source_notes",
    "confidence_level",
    "last_reviewed_at",
];

pub const PROMPT_TEMPLATE_COLUMNS: &[&str] = &[
    "prompt_template_id",
    "stage",
    "name",
    "required_inputs",
    "body",
    "expected_output_schema",
    "repairs_failure_codes",
    "target_model_family",
    "is_structured_output",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KbRuntimeHandle {
    pub snapshot: KbSnapshotRecord,
    pub summary: KbRuntimeSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KbKnowledgeBundle {
    pub scene_taxonomies: Vec<core_domain::SceneTaxonomyRecord>,
    pub failure_patterns: Vec<FailurePatternRecord>,
    pub prompt_templates: Vec<PromptTemplateRecord>,
}

impl KbRuntimeHandle {
    pub fn supports_scene_taxonomy(&self) -> bool {
        self.summary.has_scene_taxonomy
    }

    pub fn supports_failure_repairs(&self) -> bool {
        self.summary.has_failure_patterns && self.summary.has_repair_template_mapping
    }

    pub fn scene_taxonomy_columns(&self) -> &'static [&'static str] {
        SCENE_TAXONOMY_COLUMNS
    }

    pub fn failure_pattern_columns(&self) -> &'static [&'static str] {
        FAILURE_PATTERN_COLUMNS
    }

    pub fn prompt_template_columns(&self) -> &'static [&'static str] {
        PROMPT_TEMPLATE_COLUMNS
    }

    pub fn supports_golden_sample_v120_package(&self) -> bool {
        self.summary.has_golden_sample_v120_package
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KbRuntimeError {
    SnapshotMissing { path: PathBuf },
    SnapshotMetadataUnreadable { path: PathBuf, message: String },
    SeedBundlePathMissing { path: PathBuf },
    SeedBundleUnreadable { path: PathBuf, message: String },
    SeedBundleInvalid { path: PathBuf, message: String },
}

pub fn load_kb_runtime(snapshot_path: PathBuf) -> Result<KbRuntimeHandle, KbRuntimeError> {
    if !snapshot_path.is_file() {
        return Err(KbRuntimeError::SnapshotMissing {
            path: snapshot_path,
        });
    }

    let metadata = fs::metadata(&snapshot_path).map_err(|error| {
        KbRuntimeError::SnapshotMetadataUnreadable {
            path: snapshot_path.clone(),
            message: error.to_string(),
        }
    })?;

    let created_at_timestamp = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default();

    let snapshot_id = snapshot_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("hope-kb-snapshot")
        .to_string();
    let snapshot_path_string = snapshot_path.display().to_string();

    let snapshot = KbSnapshotRecord {
        snapshot_id: snapshot_id.clone(),
        snapshot_hash: "runtime-unverified".to_string(),
        seed_format: "hope-kb-sqlite-snapshot-v0.1+golden-sample-v0.2".to_string(),
        source_name: "hope-kb".to_string(),
        created_at_timestamp,
    };
    let v120_summary = summarize_optional_golden_sample_v120_package(&snapshot_path);

    let summary = KbRuntimeSummary {
        snapshot_id,
        snapshot_path: snapshot_path_string,
        mirror_tables: HOPE_KB_MIRROR_TABLES
            .iter()
            .map(|table| (*table).to_string())
            .collect(),
        has_scene_taxonomy: true,
        has_failure_patterns: true,
        has_repair_template_mapping: true,
        has_golden_sample_v120_package: v120_summary.is_some(),
        golden_sample_record_count: v120_summary
            .as_ref()
            .map(|summary| summary.golden_sample_record_count)
            .unwrap_or_default(),
        golden_sample_source_count: v120_summary
            .map(|summary| summary.golden_sample_source_count)
            .unwrap_or_default(),
    };

    Ok(KbRuntimeHandle { snapshot, summary })
}

pub fn load_kb_knowledge_bundle(
    runtime: &KbRuntimeHandle,
) -> Result<KbKnowledgeBundle, KbRuntimeError> {
    let seed_root = derive_seed_bundle_root(Path::new(&runtime.summary.snapshot_path))?;

    Ok(KbKnowledgeBundle {
        scene_taxonomies: load_scene_taxonomies(&seed_root)?,
        failure_patterns: load_failure_patterns(&seed_root)?,
        prompt_templates: load_prompt_templates(&seed_root)?,
    })
}

pub fn load_kb_golden_sample_runtime_package(
    runtime: &KbRuntimeHandle,
) -> Result<KbGoldenSampleRuntimePackage, KbRuntimeError> {
    let seed_root =
        derive_versioned_seed_bundle_root(Path::new(&runtime.summary.snapshot_path), "v0.2")?;

    let manifest: KbBundleManifestRecord = load_seed_json(&seed_root.join("manifest.json"))?;
    let golden_sample_library: GoldenSampleLibraryAsset =
        load_seed_json(&seed_root.join("golden_sample_library.json"))?;
    let field_coverage_rules: GoldenSampleFieldCoverageRuleAsset =
        load_seed_json(&seed_root.join("golden_sample_field_coverage_rules.json"))?;
    let failure_mapping: GoldenSampleFailureMappingAsset =
        load_seed_json(&seed_root.join("golden_sample_failure_mapping.json"))?;
    let repair_mapping: GoldenSampleRepairMappingAsset =
        load_seed_json(&seed_root.join("golden_sample_repair_mapping.json"))?;
    let source_register: GoldenSampleSourceRegister =
        load_seed_json(&seed_root.join("source_register.json"))?;

    validate_golden_sample_package_counts(
        &seed_root,
        &manifest,
        &golden_sample_library,
        &field_coverage_rules,
        &failure_mapping,
        &repair_mapping,
        &source_register,
    )?;

    Ok(KbGoldenSampleRuntimePackage {
        manifest,
        golden_sample_library,
        field_coverage_rules,
        failure_mapping,
        repair_mapping,
        source_register,
    })
}

pub fn bind_failure_repair_links(
    failure_pattern: &FailurePatternRecord,
    prompt_templates: &[PromptTemplateRecord],
) -> Vec<RepairTemplateLink> {
    failure_pattern
        .repair_template_ids
        .iter()
        .map(|template_id| {
            let template = prompt_templates
                .iter()
                .find(|candidate| candidate.prompt_template_id == *template_id);

            RepairTemplateLink {
                failure_code: failure_pattern.failure_code.clone(),
                prompt_template_id: template_id.clone(),
                prompt_stage: template.map(|item| item.stage.clone()),
                prompt_name: template.map(|item| item.name.clone()),
            }
        })
        .collect()
}

fn derive_seed_bundle_root(snapshot_path: &Path) -> Result<PathBuf, KbRuntimeError> {
    derive_versioned_seed_bundle_root(snapshot_path, "v0.1")
}

fn derive_versioned_seed_bundle_root(
    snapshot_path: &Path,
    version: &str,
) -> Result<PathBuf, KbRuntimeError> {
    let Some(repo_root) = snapshot_path.parent().and_then(|path| path.parent()) else {
        return Err(KbRuntimeError::SeedBundlePathMissing {
            path: snapshot_path.to_path_buf(),
        });
    };
    let seed_root = repo_root.join("seed").join(version);
    if !seed_root.is_dir() {
        return Err(KbRuntimeError::SeedBundlePathMissing { path: seed_root });
    }
    Ok(seed_root)
}

struct GoldenSampleV120Summary {
    golden_sample_record_count: usize,
    golden_sample_source_count: usize,
}

fn summarize_optional_golden_sample_v120_package(
    snapshot_path: &Path,
) -> Option<GoldenSampleV120Summary> {
    let seed_root = derive_versioned_seed_bundle_root(snapshot_path, "v0.2").ok()?;
    if GOLDEN_SAMPLE_V120_PACKAGE_FILES
        .iter()
        .any(|file| !seed_root.join(file).is_file())
    {
        return None;
    }

    let manifest: KbBundleManifestRecord = load_seed_json(&seed_root.join("manifest.json")).ok()?;
    Some(GoldenSampleV120Summary {
        golden_sample_record_count: manifest.record_counts.golden_sample_library,
        golden_sample_source_count: manifest.record_counts.golden_sample_sources,
    })
}

fn validate_golden_sample_package_counts(
    seed_root: &Path,
    manifest: &KbBundleManifestRecord,
    golden_sample_library: &GoldenSampleLibraryAsset,
    field_coverage_rules: &GoldenSampleFieldCoverageRuleAsset,
    failure_mapping: &GoldenSampleFailureMappingAsset,
    repair_mapping: &GoldenSampleRepairMappingAsset,
    source_register: &GoldenSampleSourceRegister,
) -> Result<(), KbRuntimeError> {
    let expected = &manifest.record_counts;
    let actuals = [
        (
            "golden_sample_library",
            expected.golden_sample_library,
            golden_sample_library.records.len(),
        ),
        (
            "golden_sample_field_coverage_rules",
            expected.golden_sample_field_coverage_rules,
            field_coverage_rules.records.len(),
        ),
        (
            "golden_sample_failure_mapping",
            expected.golden_sample_failure_mapping,
            failure_mapping.records.len(),
        ),
        (
            "golden_sample_repair_mapping",
            expected.golden_sample_repair_mapping,
            repair_mapping.records.len(),
        ),
        (
            "golden_sample_sources",
            expected.golden_sample_sources,
            source_register.sources.len(),
        ),
        (
            "golden_sample_provenance_entries",
            expected.golden_sample_provenance_entries,
            source_register.provenance_entries.len(),
        ),
    ];

    for (label, expected, actual) in actuals {
        if expected != actual {
            return Err(KbRuntimeError::SeedBundleInvalid {
                path: seed_root.join("manifest.json"),
                message: format!(
                    "{} record count mismatch: expected {}, got {}",
                    label, expected, actual
                ),
            });
        }
    }

    if manifest.snapshot_version != "v0.2"
        || manifest.seed_import_format != "hope-kb-golden-sample-seed-bundle-v0.2"
    {
        return Err(KbRuntimeError::SeedBundleInvalid {
            path: seed_root.join("manifest.json"),
            message: "manifest is not a V120/v0.2 golden sample package".to_string(),
        });
    }

    Ok(())
}

fn load_scene_taxonomies(
    seed_root: &Path,
) -> Result<Vec<core_domain::SceneTaxonomyRecord>, KbRuntimeError> {
    let path = seed_root.join("scene_taxonomy.json");
    let rows: Vec<SceneTaxonomySeedRow> = load_seed_json(&path)?;

    Ok(rows
        .into_iter()
        .map(|row| core_domain::SceneTaxonomyRecord {
            scene_taxonomy_id: row.machine_id,
            scene_type: row.scene_type,
            display_name: row.display_name,
            definition: row.definition,
            default_duration_band: row.default_duration_band,
            typical_committee_roles: row.typical_committee_roles,
            default_handoff_out: row.default_handoff_out,
            risk_flags: row.risk_flags,
            continuity_priority: row.continuity_priority,
            prompt_focus: row.prompt_focus,
            source_type: row.source_type,
            source_notes: row.source_notes,
            confidence_level: row.confidence_level,
            last_reviewed_at: row.last_reviewed_at,
        })
        .collect())
}

fn load_failure_patterns(seed_root: &Path) -> Result<Vec<FailurePatternRecord>, KbRuntimeError> {
    let path = seed_root.join("failure_pattern_library.json");
    let rows: Vec<FailurePatternSeedRow> = load_seed_json(&path)?;

    Ok(rows
        .into_iter()
        .map(|row| FailurePatternRecord {
            failure_pattern_id: row.machine_id,
            failure_code: row.failure_code,
            failure_name: row.failure_name,
            failure_category: row.failure_category,
            symptom: row.symptom,
            common_causes: row.common_causes,
            detection_hint: row.detection_hint,
            repair_strategy: row.repair_strategy,
            affected_layers: row.affected_layers,
            validator_hint: row.validator_hint,
            repair_template_ids: row.repair_template_ids,
            repair_priority: row.repair_priority,
            repair_scope: row.repair_scope,
            suggested_followup_validators: row.suggested_followup_validator,
            source_type: row.source_type,
            source_notes: row.source_notes,
            confidence_level: row.confidence_level,
            last_reviewed_at: row.last_reviewed_at,
        })
        .collect())
}

fn load_prompt_templates(seed_root: &Path) -> Result<Vec<PromptTemplateRecord>, KbRuntimeError> {
    let path = seed_root.join("prompt_templates.json");
    let rows: Vec<PromptTemplateSeedRow> = load_seed_json(&path)?;

    Ok(rows
        .into_iter()
        .map(|row| PromptTemplateRecord {
            prompt_template_id: row.machine_id,
            stage: row.stage,
            name: row.name,
            target_model_family: row.target_model_family,
            repairs_failure_codes: row.repairs_failure_codes.unwrap_or_default(),
        })
        .collect())
}

fn load_seed_json<T>(path: &Path) -> Result<T, KbRuntimeError>
where
    T: for<'de> Deserialize<'de>,
{
    let json = fs::read_to_string(path).map_err(|error| KbRuntimeError::SeedBundleUnreadable {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;

    serde_json::from_str(&json).map_err(|error| KbRuntimeError::SeedBundleInvalid {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct SceneTaxonomySeedRow {
    machine_id: String,
    scene_type: String,
    display_name: String,
    definition: String,
    default_duration_band: String,
    typical_committee_roles: Vec<String>,
    default_handoff_out: Vec<String>,
    risk_flags: Vec<String>,
    continuity_priority: String,
    prompt_focus: Vec<String>,
    source_type: String,
    source_notes: String,
    confidence_level: String,
    last_reviewed_at: String,
}

#[derive(Debug, Deserialize)]
struct FailurePatternSeedRow {
    machine_id: String,
    failure_code: String,
    failure_name: String,
    failure_category: String,
    symptom: String,
    common_causes: Vec<String>,
    detection_hint: String,
    repair_strategy: String,
    affected_layers: Vec<String>,
    validator_hint: String,
    repair_template_ids: Vec<String>,
    repair_priority: String,
    repair_scope: String,
    suggested_followup_validator: Vec<String>,
    source_type: String,
    source_notes: String,
    confidence_level: String,
    last_reviewed_at: String,
}

#[derive(Debug, Deserialize)]
struct PromptTemplateSeedRow {
    machine_id: String,
    stage: String,
    name: String,
    target_model_family: String,
    repairs_failure_codes: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};

    #[test]
    fn load_kb_runtime_requires_existing_snapshot() {
        let path = PathBuf::from("E:/codex/hope-kb/snapshots/does-not-exist.sqlite3");
        let error = load_kb_runtime(path.clone()).expect_err("missing snapshot should fail");

        assert_eq!(error, KbRuntimeError::SnapshotMissing { path });
    }

    #[test]
    fn bind_failure_repair_links_joins_failure_codes_to_templates() {
        let failure_pattern = FailurePatternRecord {
            failure_pattern_id: "failure_01".to_string(),
            failure_code: "style_drift".to_string(),
            failure_name: "风格漂移".to_string(),
            failure_category: "style".to_string(),
            symptom: "相邻 cut 风格不一致".to_string(),
            common_causes: vec!["hard locks 漏注".to_string()],
            detection_hint: "检查 Style Unity Validator".to_string(),
            repair_strategy: "重新注入 hard locks".to_string(),
            affected_layers: vec!["prompt_packages".to_string()],
            validator_hint: "Style Unity Validator".to_string(),
            repair_template_ids: vec!["prompt_09".to_string(), "prompt_15".to_string()],
            repair_priority: "high".to_string(),
            repair_scope: "render_prompt_only".to_string(),
            suggested_followup_validators: vec!["Style Unity Validator".to_string()],
            source_type: "team_distillation".to_string(),
            source_notes: "test".to_string(),
            confidence_level: "high".to_string(),
            last_reviewed_at: "2026-04-19".to_string(),
        };
        let prompt_templates = vec![
            PromptTemplateRecord {
                prompt_template_id: "prompt_09".to_string(),
                stage: "repair_pass".to_string(),
                name: "结构修复回合".to_string(),
                target_model_family: "qwen-compatible".to_string(),
                repairs_failure_codes: vec!["style_drift".to_string()],
            },
            PromptTemplateRecord {
                prompt_template_id: "prompt_15".to_string(),
                stage: "repair_hardlocks".to_string(),
                name: "Hard Locks 修复模板".to_string(),
                target_model_family: "qwen-compatible".to_string(),
                repairs_failure_codes: vec!["style_drift".to_string()],
            },
        ];

        let links = bind_failure_repair_links(&failure_pattern, &prompt_templates);

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].failure_code, "style_drift");
        assert_eq!(links[0].prompt_stage.as_deref(), Some("repair_pass"));
        assert_eq!(links[1].prompt_name.as_deref(), Some("Hard Locks 修复模板"));
    }

    #[test]
    fn load_kb_runtime_builds_summary_from_existing_file() {
        let temp_path = std::env::temp_dir().join("hope-kb-runtime-test.sqlite3");
        File::create(&temp_path).expect("temp snapshot should be creatable");

        let runtime = load_kb_runtime(temp_path.clone()).expect("snapshot should load");

        assert!(runtime.supports_scene_taxonomy());
        assert!(runtime.supports_failure_repairs());
        assert!(runtime
            .summary
            .mirror_tables
            .contains(&"scene_taxonomy".to_string()));
        assert_eq!(runtime.snapshot.source_name, "hope-kb");

        fs::remove_file(temp_path).expect("temp snapshot should be removable");
    }

    #[test]
    fn load_kb_knowledge_bundle_reads_scene_taxonomy_failures_and_templates() {
        let repo_root = std::env::temp_dir().join("hope-kb-runtime-seed-test");
        let snapshots_dir = repo_root.join("snapshots");
        let seed_dir = repo_root.join("seed").join("v0.1");
        fs::create_dir_all(&snapshots_dir).expect("snapshots dir should be creatable");
        fs::create_dir_all(&seed_dir).expect("seed dir should be creatable");

        let snapshot_path = snapshots_dir.join("hope-kb-v0.1.sqlite3");
        File::create(&snapshot_path).expect("snapshot file should be creatable");

        fs::write(
            seed_dir.join("scene_taxonomy.json"),
            r#"[
              {
                "machine_id": "scene_tax_01",
                "scene_type": "日常对白",
                "display_name": "日常对白",
                "definition": "角色在稳定空间里推进关系。",
                "default_duration_band": "30-60s",
                "typical_committee_roles": ["chief","scene","emotion"],
                "default_handoff_out": ["scene->emotion"],
                "risk_flags": ["节奏过平"],
                "continuity_priority": "high",
                "prompt_focus": ["微表演","站位"],
                "source_type": "team_distillation",
                "source_notes": "test",
                "confidence_level": "high",
                "last_reviewed_at": "2026-04-20"
              }
            ]"#,
        )
        .expect("scene taxonomy seed should be writable");

        fs::write(
            seed_dir.join("failure_pattern_library.json"),
            r#"[
              {
                "machine_id": "failure_01",
                "failure_code": "style_drift",
                "failure_name": "风格漂移",
                "failure_category": "style",
                "symptom": "相邻 cut 风格断裂。",
                "common_causes": ["hard locks 漏注"],
                "detection_hint": "检查 Style Unity Validator。",
                "repair_strategy": "重新注入 hard locks。",
                "affected_layers": ["prompt_packages"],
                "validator_hint": "Style Unity Validator",
                "repair_template_ids": ["prompt_09"],
                "repair_priority": "high",
                "repair_scope": "render_prompt_only",
                "suggested_followup_validator": ["Style Unity Validator"],
                "source_type": "team_distillation",
                "source_notes": "test",
                "confidence_level": "high",
                "last_reviewed_at": "2026-04-20"
              }
            ]"#,
        )
        .expect("failure pattern seed should be writable");

        fs::write(
            seed_dir.join("prompt_templates.json"),
            r#"[
              {
                "machine_id": "prompt_09",
                "stage": "repair_pass",
                "name": "结构修复回合",
                "target_model_family": "qwen-compatible",
                "repairs_failure_codes": ["style_drift"]
              }
            ]"#,
        )
        .expect("prompt template seed should be writable");

        let runtime = load_kb_runtime(snapshot_path.clone()).expect("runtime should load");
        let bundle = load_kb_knowledge_bundle(&runtime).expect("knowledge bundle should load");

        assert_eq!(bundle.scene_taxonomies.len(), 1);
        assert_eq!(bundle.failure_patterns.len(), 1);
        assert_eq!(bundle.prompt_templates.len(), 1);
        assert_eq!(bundle.scene_taxonomies[0].scene_type, "日常对白");
        assert_eq!(bundle.failure_patterns[0].failure_code, "style_drift");
        assert_eq!(bundle.prompt_templates[0].stage, "repair_pass");

        fs::remove_dir_all(repo_root).expect("temp repo root should be removable");
    }
}
