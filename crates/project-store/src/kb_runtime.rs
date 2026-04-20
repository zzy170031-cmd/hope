use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use core_domain::{
    FailurePatternRecord, KbRuntimeSummary, KbSnapshotRecord, PromptTemplateRecord,
    RepairTemplateLink,
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

pub const SNAPSHOT_BOOTSTRAP_SURFACE: &str = "snapshot_bootstrap";
pub const SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH: &str =
    "bundle-sha256:5f042c10ada3726bdbd71d5f4cbad2187b3895dd85e1e5195d6938a3a22d20b6";
pub const SNAPSHOT_BOOTSTRAP_TRUSTED_INPUTS: &[&str] = &[
    "snapshot_meta",
    "export_template",
    "failure_pattern",
    "degraded_input_example",
    "runtime_consume_contract",
];
pub const SNAPSHOT_BOOTSTRAP_MANIFEST_FILE_NAME: &str = "manifest.json";
pub const SNAPSHOT_BOOTSTRAP_EXPORT_TEMPLATE_FILE_NAME: &str = "export_templates.json";
pub const SNAPSHOT_BOOTSTRAP_FAILURE_PATTERN_FILE_NAME: &str = "failure_pattern_library.json";
pub const SNAPSHOT_BOOTSTRAP_DEGRADED_INPUT_EXAMPLE_FILE_NAME: &str =
    "degraded_input_examples.json";
pub const SNAPSHOT_BOOTSTRAP_RUNTIME_CONSUME_CONTRACT_FILE_NAME: &str =
    "runtime_consume_contracts.json";

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedKbContext {
    pub runtime: KbRuntimeHandle,
    pub bundle: KbKnowledgeBundle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotBootstrapCheckpointArtifacts {
    pub snapshot_path: PathBuf,
    pub manifest_path: PathBuf,
    pub validator_result_path: PathBuf,
    pub snapshot_meta_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotBootstrapPreparePaths {
    pub snapshot_path: PathBuf,
    pub manifest_path: PathBuf,
    pub validator_result_path: PathBuf,
    pub snapshot_meta_path: PathBuf,
    pub export_template_path: PathBuf,
    pub failure_pattern_path: PathBuf,
    pub degraded_input_example_path: PathBuf,
    pub runtime_consume_contract_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotBootstrapPreparation {
    pub snapshot_path: PathBuf,
    pub reviewed_bundle_hash: String,
    pub consistency: SnapshotBootstrapConsistencyGate,
    pub trusted_inputs: SnapshotBootstrapTrustedInputs,
    pub contract_gate: SnapshotBootstrapContractGate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotBootstrapConsistencyGate {
    pub manifest_content_hash: String,
    pub validator_status: String,
    pub validator_content_hash: String,
    pub snapshot_meta_content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotBootstrapTrustedInputs {
    pub snapshot_meta_path: PathBuf,
    pub export_template_path: PathBuf,
    pub failure_pattern_path: PathBuf,
    pub degraded_input_example_path: PathBuf,
    pub runtime_consume_contract_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotBootstrapContractGate {
    pub consumer_surface: String,
    pub required_snapshot_tables: Vec<String>,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KbRuntimeError {
    SnapshotMissing {
        path: PathBuf,
    },
    SnapshotMetadataUnreadable {
        path: PathBuf,
        message: String,
    },
    SnapshotBootstrapManifestUnreadable {
        path: PathBuf,
        message: String,
    },
    SnapshotBootstrapManifestInvalid {
        path: PathBuf,
        message: String,
    },
    SnapshotBootstrapValidatorResultUnreadable {
        path: PathBuf,
        message: String,
    },
    SnapshotBootstrapValidatorResultInvalid {
        path: PathBuf,
        message: String,
    },
    SnapshotBootstrapValidatorFailed {
        path: PathBuf,
        status: String,
    },
    SnapshotBootstrapSnapshotMetaUnreadable {
        path: PathBuf,
        message: String,
    },
    SnapshotBootstrapSnapshotMetaInvalid {
        path: PathBuf,
        message: String,
    },
    SnapshotBootstrapReviewedHashMismatch {
        expected: String,
        manifest_hash: String,
        validator_hash: String,
        snapshot_hash: String,
    },
    SnapshotBootstrapContractUnreadable {
        path: PathBuf,
        message: String,
    },
    SnapshotBootstrapContractInvalid {
        path: PathBuf,
        message: String,
    },
    SnapshotBootstrapContractSurfaceMissing {
        path: PathBuf,
        consumer_surface: String,
    },
    SnapshotBootstrapTrustedInputMismatch {
        path: PathBuf,
        consumer_surface: String,
        missing_inputs: Vec<String>,
        unexpected_inputs: Vec<String>,
    },
    SnapshotBootstrapInputMissing {
        input_name: String,
        path: PathBuf,
    },
    SnapshotBootstrapInputUnreadable {
        input_name: String,
        path: PathBuf,
        message: String,
    },
    SnapshotBootstrapInputInvalid {
        input_name: String,
        path: PathBuf,
        message: String,
    },
    SeedBundlePathMissing {
        path: PathBuf,
    },
    SeedBundleUnreadable {
        path: PathBuf,
        message: String,
    },
    SeedBundleInvalid {
        path: PathBuf,
        message: String,
    },
}

pub fn load_kb_runtime(snapshot_path: PathBuf) -> Result<KbRuntimeHandle, KbRuntimeError> {
    build_kb_runtime_handle(snapshot_path, "runtime-unverified".to_string())
}

pub fn build_snapshot_bootstrap_prepare_paths(
    snapshot_path: PathBuf,
    manifest_path: PathBuf,
    validator_result_path: PathBuf,
    snapshot_meta_path: PathBuf,
) -> SnapshotBootstrapPreparePaths {
    let packet_root = manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    SnapshotBootstrapPreparePaths {
        snapshot_path,
        manifest_path,
        validator_result_path,
        snapshot_meta_path,
        export_template_path: packet_root.join(SNAPSHOT_BOOTSTRAP_EXPORT_TEMPLATE_FILE_NAME),
        failure_pattern_path: packet_root.join(SNAPSHOT_BOOTSTRAP_FAILURE_PATTERN_FILE_NAME),
        degraded_input_example_path: packet_root
            .join(SNAPSHOT_BOOTSTRAP_DEGRADED_INPUT_EXAMPLE_FILE_NAME),
        runtime_consume_contract_path: packet_root
            .join(SNAPSHOT_BOOTSTRAP_RUNTIME_CONSUME_CONTRACT_FILE_NAME),
    }
}

pub fn load_kb_runtime_from_preparation(
    preparation: SnapshotBootstrapPreparation,
) -> Result<KbRuntimeHandle, KbRuntimeError> {
    build_kb_runtime_handle(preparation.snapshot_path, preparation.reviewed_bundle_hash)
}

pub fn bootstrap_verified_kb_runtime(
    paths: SnapshotBootstrapPreparePaths,
) -> Result<KbRuntimeHandle, KbRuntimeError> {
    let preparation = prepare_snapshot_bootstrap(paths)?;
    load_kb_runtime_from_preparation(preparation)
}

pub fn bootstrap_verified_kb_context(
    snapshot_path: PathBuf,
    manifest_path: PathBuf,
    validator_result_path: PathBuf,
    snapshot_meta_path: PathBuf,
) -> Result<VerifiedKbContext, KbRuntimeError> {
    let artifacts = SnapshotBootstrapCheckpointArtifacts {
        snapshot_path,
        manifest_path,
        validator_result_path,
        snapshot_meta_path,
    };

    bootstrap_verified_kb_context_from_checkpoint(artifacts)
}

pub fn bootstrap_verified_kb_context_from_checkpoint(
    artifacts: SnapshotBootstrapCheckpointArtifacts,
) -> Result<VerifiedKbContext, KbRuntimeError> {
    let paths = build_snapshot_bootstrap_prepare_paths(
        artifacts.snapshot_path,
        artifacts.manifest_path,
        artifacts.validator_result_path,
        artifacts.snapshot_meta_path,
    );
    let preparation = prepare_snapshot_bootstrap(paths)?;
    let runtime = load_kb_runtime_from_preparation(preparation)?;
    let bundle = load_kb_knowledge_bundle(&runtime)?;

    Ok(VerifiedKbContext { runtime, bundle })
}

fn build_kb_runtime_handle(
    snapshot_path: PathBuf,
    snapshot_hash: String,
) -> Result<KbRuntimeHandle, KbRuntimeError> {
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
        snapshot_hash,
        seed_format: "hope-kb-sqlite-snapshot-v0.1".to_string(),
        source_name: "hope-kb".to_string(),
        created_at_timestamp,
    };

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
    };

    Ok(KbRuntimeHandle { snapshot, summary })
}

pub fn prepare_snapshot_bootstrap(
    paths: SnapshotBootstrapPreparePaths,
) -> Result<SnapshotBootstrapPreparation, KbRuntimeError> {
    prepare_snapshot_bootstrap_with_reviewed_hash(paths, SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH)
}

pub fn prepare_snapshot_bootstrap_with_reviewed_hash(
    paths: SnapshotBootstrapPreparePaths,
    reviewed_bundle_hash: &str,
) -> Result<SnapshotBootstrapPreparation, KbRuntimeError> {
    if !paths.snapshot_path.is_file() {
        return Err(KbRuntimeError::SnapshotMissing {
            path: paths.snapshot_path,
        });
    }

    let manifest = load_snapshot_bootstrap_manifest(&paths.manifest_path)?;
    let validator_result = load_snapshot_bootstrap_validator_result(&paths.validator_result_path)?;
    if !validator_result.status.eq_ignore_ascii_case("passed") {
        return Err(KbRuntimeError::SnapshotBootstrapValidatorFailed {
            path: paths.validator_result_path,
            status: validator_result.status,
        });
    }

    let snapshot_meta = load_snapshot_bootstrap_snapshot_meta(&paths.snapshot_meta_path)?;
    ensure_snapshot_bootstrap_reviewed_hash(
        reviewed_bundle_hash,
        &manifest.content_hash,
        &validator_result.content_hash,
        &snapshot_meta.content_hash,
    )?;

    let contract_gate =
        load_snapshot_bootstrap_contract_gate(&paths.runtime_consume_contract_path)?;
    ensure_snapshot_bootstrap_trusted_inputs(&paths.runtime_consume_contract_path, &contract_gate)?;

    validate_snapshot_bootstrap_json_input("export_template", &paths.export_template_path)?;
    validate_snapshot_bootstrap_json_input("failure_pattern", &paths.failure_pattern_path)?;
    validate_snapshot_bootstrap_json_input(
        "degraded_input_example",
        &paths.degraded_input_example_path,
    )?;

    Ok(SnapshotBootstrapPreparation {
        snapshot_path: paths.snapshot_path,
        reviewed_bundle_hash: reviewed_bundle_hash.to_string(),
        consistency: SnapshotBootstrapConsistencyGate {
            manifest_content_hash: manifest.content_hash,
            validator_status: validator_result.status,
            validator_content_hash: validator_result.content_hash,
            snapshot_meta_content_hash: snapshot_meta.content_hash,
        },
        trusted_inputs: SnapshotBootstrapTrustedInputs {
            snapshot_meta_path: paths.snapshot_meta_path,
            export_template_path: paths.export_template_path,
            failure_pattern_path: paths.failure_pattern_path,
            degraded_input_example_path: paths.degraded_input_example_path,
            runtime_consume_contract_path: paths.runtime_consume_contract_path,
        },
        contract_gate: SnapshotBootstrapContractGate {
            consumer_surface: contract_gate.consumer_surface,
            required_snapshot_tables: contract_gate.required_snapshot_tables,
        },
    })
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
    let Some(repo_root) = snapshot_path.parent().and_then(|path| path.parent()) else {
        return Err(KbRuntimeError::SeedBundlePathMissing {
            path: snapshot_path.to_path_buf(),
        });
    };
    let seed_root = repo_root.join("seed").join("v0.1");
    if !seed_root.is_dir() {
        return Err(KbRuntimeError::SeedBundlePathMissing { path: seed_root });
    }
    Ok(seed_root)
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

fn load_snapshot_bootstrap_manifest(
    path: &Path,
) -> Result<SnapshotBootstrapManifestSeed, KbRuntimeError> {
    let json = fs::read_to_string(path).map_err(|error| {
        KbRuntimeError::SnapshotBootstrapManifestUnreadable {
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    })?;

    serde_json::from_str(&json).map_err(|error| KbRuntimeError::SnapshotBootstrapManifestInvalid {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}

fn load_snapshot_bootstrap_validator_result(
    path: &Path,
) -> Result<SnapshotBootstrapValidatorResultSeed, KbRuntimeError> {
    let json = fs::read_to_string(path).map_err(|error| {
        KbRuntimeError::SnapshotBootstrapValidatorResultUnreadable {
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    })?;

    serde_json::from_str(&json).map_err(|error| {
        KbRuntimeError::SnapshotBootstrapValidatorResultInvalid {
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    })
}

fn load_snapshot_bootstrap_snapshot_meta(
    path: &Path,
) -> Result<SnapshotBootstrapSnapshotMetaSeed, KbRuntimeError> {
    let json = fs::read_to_string(path).map_err(|error| {
        KbRuntimeError::SnapshotBootstrapSnapshotMetaUnreadable {
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    })?;

    serde_json::from_str(&json).map_err(|error| {
        KbRuntimeError::SnapshotBootstrapSnapshotMetaInvalid {
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    })
}

fn load_snapshot_bootstrap_contract_gate(
    path: &Path,
) -> Result<SnapshotBootstrapContractRow, KbRuntimeError> {
    let json = fs::read_to_string(path).map_err(|error| {
        KbRuntimeError::SnapshotBootstrapContractUnreadable {
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    })?;

    let rows: Vec<SnapshotBootstrapContractRow> = serde_json::from_str(&json).map_err(|error| {
        KbRuntimeError::SnapshotBootstrapContractInvalid {
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    })?;

    rows.into_iter()
        .find(|row| row.consumer_surface == SNAPSHOT_BOOTSTRAP_SURFACE)
        .ok_or_else(|| KbRuntimeError::SnapshotBootstrapContractSurfaceMissing {
            path: path.to_path_buf(),
            consumer_surface: SNAPSHOT_BOOTSTRAP_SURFACE.to_string(),
        })
}

fn ensure_snapshot_bootstrap_reviewed_hash(
    reviewed_bundle_hash: &str,
    manifest_hash: &str,
    validator_hash: &str,
    snapshot_hash: &str,
) -> Result<(), KbRuntimeError> {
    if manifest_hash == reviewed_bundle_hash
        && validator_hash == reviewed_bundle_hash
        && snapshot_hash == reviewed_bundle_hash
    {
        return Ok(());
    }

    Err(KbRuntimeError::SnapshotBootstrapReviewedHashMismatch {
        expected: reviewed_bundle_hash.to_string(),
        manifest_hash: manifest_hash.to_string(),
        validator_hash: validator_hash.to_string(),
        snapshot_hash: snapshot_hash.to_string(),
    })
}

fn ensure_snapshot_bootstrap_trusted_inputs(
    path: &Path,
    contract_gate: &SnapshotBootstrapContractRow,
) -> Result<(), KbRuntimeError> {
    let expected: BTreeSet<String> = SNAPSHOT_BOOTSTRAP_TRUSTED_INPUTS
        .iter()
        .map(|item| (*item).to_string())
        .collect();
    let actual: BTreeSet<String> = contract_gate
        .required_snapshot_tables
        .iter()
        .cloned()
        .collect();

    let missing_inputs = expected.difference(&actual).cloned().collect::<Vec<_>>();
    let unexpected_inputs = actual.difference(&expected).cloned().collect::<Vec<_>>();

    if missing_inputs.is_empty() && unexpected_inputs.is_empty() {
        return Ok(());
    }

    Err(KbRuntimeError::SnapshotBootstrapTrustedInputMismatch {
        path: path.to_path_buf(),
        consumer_surface: contract_gate.consumer_surface.clone(),
        missing_inputs,
        unexpected_inputs,
    })
}

fn validate_snapshot_bootstrap_json_input(
    input_name: &str,
    path: &Path,
) -> Result<(), KbRuntimeError> {
    if !path.is_file() {
        return Err(KbRuntimeError::SnapshotBootstrapInputMissing {
            input_name: input_name.to_string(),
            path: path.to_path_buf(),
        });
    }

    let json = fs::read_to_string(path).map_err(|error| {
        KbRuntimeError::SnapshotBootstrapInputUnreadable {
            input_name: input_name.to_string(),
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    })?;

    serde_json::from_str::<serde_json::Value>(&json).map_err(|error| {
        KbRuntimeError::SnapshotBootstrapInputInvalid {
            input_name: input_name.to_string(),
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    })?;

    Ok(())
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

#[derive(Debug, Deserialize)]
struct SnapshotBootstrapManifestSeed {
    content_hash: String,
}

#[derive(Debug, Deserialize)]
struct SnapshotBootstrapValidatorResultSeed {
    #[serde(alias = "result")]
    status: String,
    content_hash: String,
}

#[derive(Debug, Deserialize)]
struct SnapshotBootstrapSnapshotMetaSeed {
    content_hash: String,
}

#[derive(Debug, Deserialize)]
struct SnapshotBootstrapContractRow {
    consumer_surface: String,
    required_snapshot_tables: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::time::{SystemTime, UNIX_EPOCH};

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
        assert!(
            runtime
                .summary
                .mirror_tables
                .contains(&"scene_taxonomy".to_string())
        );
        assert_eq!(runtime.snapshot.source_name, "hope-kb");
        assert_eq!(runtime.snapshot.snapshot_hash, "runtime-unverified");

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

    #[test]
    fn prepare_snapshot_bootstrap_pins_reviewed_hash_and_trusted_inputs() {
        let fixture = create_snapshot_bootstrap_fixture();

        let preparation =
            prepare_snapshot_bootstrap(fixture.paths.clone()).expect("bootstrap should prepare");

        assert_eq!(
            preparation.reviewed_bundle_hash,
            SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
        );
        assert_eq!(
            preparation.consistency.manifest_content_hash,
            SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
        );
        assert_eq!(
            preparation.consistency.validator_content_hash,
            SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
        );
        assert_eq!(
            preparation.consistency.snapshot_meta_content_hash,
            SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
        );
        assert_eq!(preparation.consistency.validator_status, "passed");
        assert_eq!(
            preparation.contract_gate.consumer_surface,
            SNAPSHOT_BOOTSTRAP_SURFACE
        );
        assert_eq!(
            preparation.contract_gate.required_snapshot_tables,
            SNAPSHOT_BOOTSTRAP_TRUSTED_INPUTS
                .iter()
                .map(|item| (*item).to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            preparation.trusted_inputs.runtime_consume_contract_path,
            fixture.paths.runtime_consume_contract_path
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn prepare_snapshot_bootstrap_rejects_hash_mismatch() {
        let fixture = create_snapshot_bootstrap_fixture();
        fs::write(
            &fixture.paths.validator_result_path,
            r#"{"status":"passed","content_hash":"bundle-sha256:mismatch"}"#,
        )
        .expect("validator result should be writable");

        let error = prepare_snapshot_bootstrap(fixture.paths.clone())
            .expect_err("hash mismatch should fail");

        assert_eq!(
            error,
            KbRuntimeError::SnapshotBootstrapReviewedHashMismatch {
                expected: SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH.to_string(),
                manifest_hash: SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH.to_string(),
                validator_hash: "bundle-sha256:mismatch".to_string(),
                snapshot_hash: SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH.to_string(),
            }
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn prepare_snapshot_bootstrap_rejects_untrusted_contract_expansion() {
        let fixture = create_snapshot_bootstrap_fixture();
        fs::write(
            &fixture.paths.runtime_consume_contract_path,
            r#"[
              {
                "consumer_surface": "snapshot_bootstrap",
                "required_snapshot_tables": [
                  "snapshot_meta",
                  "export_template",
                  "failure_pattern",
                  "degraded_input_example",
                  "runtime_consume_contract",
                  "handoff_projection"
                ]
              }
            ]"#,
        )
        .expect("runtime contract should be writable");

        let error = prepare_snapshot_bootstrap(fixture.paths.clone())
            .expect_err("unexpected trusted input expansion should fail");

        assert_eq!(
            error,
            KbRuntimeError::SnapshotBootstrapTrustedInputMismatch {
                path: fixture.paths.runtime_consume_contract_path.clone(),
                consumer_surface: SNAPSHOT_BOOTSTRAP_SURFACE.to_string(),
                missing_inputs: Vec::new(),
                unexpected_inputs: vec!["handoff_projection".to_string()],
            }
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn load_kb_runtime_from_preparation_uses_reviewed_hash_and_preserves_summary() {
        let fixture = create_snapshot_bootstrap_fixture();
        let preparation =
            prepare_snapshot_bootstrap(fixture.paths.clone()).expect("bootstrap should prepare");
        let expected_snapshot_path = fixture.paths.snapshot_path.display().to_string();

        let runtime = load_kb_runtime_from_preparation(preparation)
            .expect("verified bootstrap runtime should load");

        assert_eq!(
            runtime.snapshot.snapshot_hash,
            SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
        );
        assert_eq!(runtime.summary.snapshot_path, expected_snapshot_path);
        assert!(runtime.supports_scene_taxonomy());
        assert!(runtime.supports_failure_repairs());
        assert_eq!(
            runtime.summary.mirror_tables,
            HOPE_KB_MIRROR_TABLES
                .iter()
                .map(|table| (*table).to_string())
                .collect::<Vec<_>>()
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn build_snapshot_bootstrap_prepare_paths_uses_canonical_packet_file_names() {
        let fixture = create_snapshot_bootstrap_fixture();

        assert_eq!(
            fixture
                .paths
                .manifest_path
                .file_name()
                .and_then(|name| name.to_str()),
            Some(SNAPSHOT_BOOTSTRAP_MANIFEST_FILE_NAME)
        );
        assert_eq!(
            fixture
                .paths
                .export_template_path
                .file_name()
                .and_then(|name| name.to_str()),
            Some(SNAPSHOT_BOOTSTRAP_EXPORT_TEMPLATE_FILE_NAME)
        );
        assert_eq!(
            fixture
                .paths
                .failure_pattern_path
                .file_name()
                .and_then(|name| name.to_str()),
            Some(SNAPSHOT_BOOTSTRAP_FAILURE_PATTERN_FILE_NAME)
        );
        assert_eq!(
            fixture
                .paths
                .degraded_input_example_path
                .file_name()
                .and_then(|name| name.to_str()),
            Some(SNAPSHOT_BOOTSTRAP_DEGRADED_INPUT_EXAMPLE_FILE_NAME)
        );
        assert_eq!(
            fixture
                .paths
                .runtime_consume_contract_path
                .file_name()
                .and_then(|name| name.to_str()),
            Some(SNAPSHOT_BOOTSTRAP_RUNTIME_CONSUME_CONTRACT_FILE_NAME)
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn bootstrap_verified_kb_runtime_uses_reviewed_hash_and_preserves_summary() {
        let fixture = create_snapshot_bootstrap_fixture();
        let expected_snapshot_path = fixture.paths.snapshot_path.display().to_string();

        let runtime = bootstrap_verified_kb_runtime(fixture.paths.clone())
            .expect("verified bootstrap runtime should load");

        assert_eq!(
            runtime.snapshot.snapshot_hash,
            SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
        );
        assert_eq!(runtime.summary.snapshot_path, expected_snapshot_path);
        assert!(runtime.supports_scene_taxonomy());
        assert!(runtime.supports_failure_repairs());
        assert_eq!(
            runtime.summary.mirror_tables,
            HOPE_KB_MIRROR_TABLES
                .iter()
                .map(|table| (*table).to_string())
                .collect::<Vec<_>>()
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn bootstrap_verified_kb_runtime_rejects_hash_mismatch() {
        let fixture = create_snapshot_bootstrap_fixture();
        fs::write(
            &fixture.paths.validator_result_path,
            r#"{"status":"passed","content_hash":"bundle-sha256:mismatch"}"#,
        )
        .expect("validator result should be writable");

        let error = bootstrap_verified_kb_runtime(fixture.paths.clone())
            .expect_err("verified bootstrap should fail on hash mismatch");

        assert_eq!(
            error,
            KbRuntimeError::SnapshotBootstrapReviewedHashMismatch {
                expected: SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH.to_string(),
                manifest_hash: SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH.to_string(),
                validator_hash: "bundle-sha256:mismatch".to_string(),
                snapshot_hash: SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH.to_string(),
            }
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn bootstrap_verified_kb_context_loads_runtime_and_bundle() {
        let fixture = create_snapshot_bootstrap_fixture();
        let expected_snapshot_path = fixture.paths.snapshot_path.display().to_string();

        let context = bootstrap_verified_kb_context(
            fixture.paths.snapshot_path.clone(),
            fixture.paths.manifest_path.clone(),
            fixture.paths.validator_result_path.clone(),
            fixture.paths.snapshot_meta_path.clone(),
        )
        .expect("verified bootstrap context should load");

        assert_eq!(
            context.runtime.snapshot.snapshot_hash,
            SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
        );
        assert_eq!(
            context.runtime.summary.snapshot_path,
            expected_snapshot_path
        );
        assert!(context.runtime.supports_scene_taxonomy());
        assert!(context.runtime.supports_failure_repairs());

        assert_eq!(context.bundle.scene_taxonomies.len(), 1);
        assert_eq!(context.bundle.failure_patterns.len(), 1);
        assert_eq!(context.bundle.prompt_templates.len(), 1);
        assert_eq!(
            context.bundle.scene_taxonomies[0].scene_type,
            "daily_dialogue"
        );
        assert_eq!(
            context.bundle.failure_patterns[0].failure_code,
            "style_drift"
        );
        assert_eq!(context.bundle.prompt_templates[0].stage, "repair_pass");

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn bootstrap_verified_kb_context_rejects_hash_mismatch() {
        let fixture = create_snapshot_bootstrap_fixture();
        fs::write(
            &fixture.paths.validator_result_path,
            r#"{"status":"passed","content_hash":"bundle-sha256:mismatch"}"#,
        )
        .expect("validator result should be writable");

        let error = bootstrap_verified_kb_context(
            fixture.paths.snapshot_path.clone(),
            fixture.paths.manifest_path.clone(),
            fixture.paths.validator_result_path.clone(),
            fixture.paths.snapshot_meta_path.clone(),
        )
        .expect_err("verified bootstrap context should fail on hash mismatch");

        assert_eq!(
            error,
            KbRuntimeError::SnapshotBootstrapReviewedHashMismatch {
                expected: SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH.to_string(),
                manifest_hash: SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH.to_string(),
                validator_hash: "bundle-sha256:mismatch".to_string(),
                snapshot_hash: SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH.to_string(),
            }
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn bootstrap_verified_kb_context_from_checkpoint_loads_runtime_and_bundle() {
        let fixture = create_snapshot_bootstrap_fixture();
        let expected_snapshot_path = fixture.paths.snapshot_path.display().to_string();
        let artifacts = SnapshotBootstrapCheckpointArtifacts {
            snapshot_path: fixture.paths.snapshot_path.clone(),
            manifest_path: fixture.paths.manifest_path.clone(),
            validator_result_path: fixture.paths.validator_result_path.clone(),
            snapshot_meta_path: fixture.paths.snapshot_meta_path.clone(),
        };

        let context = bootstrap_verified_kb_context_from_checkpoint(artifacts)
            .expect("verified checkpoint context should load");

        assert_eq!(
            context.runtime.snapshot.snapshot_hash,
            SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
        );
        assert_eq!(
            context.runtime.summary.snapshot_path,
            expected_snapshot_path
        );
        assert!(context.runtime.supports_scene_taxonomy());
        assert!(context.runtime.supports_failure_repairs());
        assert_eq!(context.bundle.scene_taxonomies.len(), 1);
        assert_eq!(context.bundle.failure_patterns.len(), 1);
        assert_eq!(context.bundle.prompt_templates.len(), 1);

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    #[test]
    fn bootstrap_verified_kb_context_from_checkpoint_rejects_hash_mismatch() {
        let fixture = create_snapshot_bootstrap_fixture();
        let artifacts = SnapshotBootstrapCheckpointArtifacts {
            snapshot_path: fixture.paths.snapshot_path.clone(),
            manifest_path: fixture.paths.manifest_path.clone(),
            validator_result_path: fixture.paths.validator_result_path.clone(),
            snapshot_meta_path: fixture.paths.snapshot_meta_path.clone(),
        };
        fs::write(
            &fixture.paths.validator_result_path,
            r#"{"status":"passed","content_hash":"bundle-sha256:mismatch"}"#,
        )
        .expect("validator result should be writable");

        let error = bootstrap_verified_kb_context_from_checkpoint(artifacts)
            .expect_err("verified checkpoint context should fail on hash mismatch");

        assert_eq!(
            error,
            KbRuntimeError::SnapshotBootstrapReviewedHashMismatch {
                expected: SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH.to_string(),
                manifest_hash: SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH.to_string(),
                validator_hash: "bundle-sha256:mismatch".to_string(),
                snapshot_hash: SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH.to_string(),
            }
        );

        fs::remove_dir_all(fixture.root).expect("fixture root should be removable");
    }

    fn create_snapshot_bootstrap_fixture() -> SnapshotBootstrapFixture {
        let root = unique_test_dir("snapshot-bootstrap-prep");
        let repo_root = root.join("hope-kb-runtime");
        let snapshots_dir = repo_root.join("snapshots");
        let packet_root = repo_root.join("seed").join("v0.1");
        fs::create_dir_all(&snapshots_dir).expect("snapshots dir should be creatable");
        fs::create_dir_all(&packet_root).expect("packet root should be creatable");

        let snapshot_path = snapshots_dir.join("hope-kb-v0.1.sqlite3");
        File::create(&snapshot_path).expect("snapshot file should be creatable");

        let manifest_path = packet_root.join(SNAPSHOT_BOOTSTRAP_MANIFEST_FILE_NAME);
        fs::write(
            &manifest_path,
            format!(
                r#"{{"content_hash":"{}"}}"#,
                SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
            ),
        )
        .expect("manifest should be writable");

        let validator_result_path = root.join("validator_result.json");
        fs::write(
            &validator_result_path,
            format!(
                r#"{{"status":"passed","content_hash":"{}"}}"#,
                SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
            ),
        )
        .expect("validator result should be writable");

        let snapshot_meta_path = root.join("snapshot_meta.json");
        fs::write(
            &snapshot_meta_path,
            format!(
                r#"{{"content_hash":"{}"}}"#,
                SNAPSHOT_BOOTSTRAP_REVIEWED_BUNDLE_HASH
            ),
        )
        .expect("snapshot meta should be writable");

        let export_template_path = packet_root.join(SNAPSHOT_BOOTSTRAP_EXPORT_TEMPLATE_FILE_NAME);
        fs::write(&export_template_path, "[]").expect("export templates should be writable");

        let failure_pattern_path = packet_root.join(SNAPSHOT_BOOTSTRAP_FAILURE_PATTERN_FILE_NAME);
        fs::write(
            &failure_pattern_path,
            r#"[
              {
                "machine_id": "failure_01",
                "failure_code": "style_drift",
                "failure_name": "Style drift",
                "failure_category": "style",
                "symptom": "Adjacent cuts drift apart.",
                "common_causes": ["hard lock missing"],
                "detection_hint": "Check Style Unity Validator",
                "repair_strategy": "Reapply hard locks.",
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
        .expect("failure patterns should be writable");

        let degraded_input_example_path =
            packet_root.join(SNAPSHOT_BOOTSTRAP_DEGRADED_INPUT_EXAMPLE_FILE_NAME);
        fs::write(&degraded_input_example_path, "[]")
            .expect("degraded input examples should be writable");

        let runtime_consume_contract_path =
            packet_root.join(SNAPSHOT_BOOTSTRAP_RUNTIME_CONSUME_CONTRACT_FILE_NAME);
        fs::write(
            &runtime_consume_contract_path,
            r#"[
              {
                "consumer_surface": "snapshot_bootstrap",
                "required_snapshot_tables": [
                  "snapshot_meta",
                  "export_template",
                  "failure_pattern",
                  "degraded_input_example",
                  "runtime_consume_contract"
                ]
              },
              {
                "consumer_surface": "handoff_projection",
                "required_snapshot_tables": [
                  "committee_handoff_rule",
                  "failure_pattern",
                  "degraded_input_example",
                  "export_template",
                  "runtime_consume_contract"
                ]
              }
            ]"#,
        )
        .expect("runtime consume contract should be writable");

        fs::write(
            packet_root.join("scene_taxonomy.json"),
            r#"[
              {
                "machine_id": "scene_tax_01",
                "scene_type": "daily_dialogue",
                "display_name": "Daily Dialogue",
                "definition": "A stable dialogue scene.",
                "default_duration_band": "30-60s",
                "typical_committee_roles": ["chief","scene","emotion"],
                "default_handoff_out": ["scene->emotion"],
                "risk_flags": ["pace_flat"],
                "continuity_priority": "high",
                "prompt_focus": ["micro_expression","blocking"],
                "source_type": "team_distillation",
                "source_notes": "test",
                "confidence_level": "high",
                "last_reviewed_at": "2026-04-20"
              }
            ]"#,
        )
        .expect("scene taxonomy should be writable");

        fs::write(
            packet_root.join("prompt_templates.json"),
            r#"[
              {
                "machine_id": "prompt_09",
                "stage": "repair_pass",
                "name": "Structure Repair Loop",
                "target_model_family": "qwen-compatible",
                "repairs_failure_codes": ["style_drift"]
              }
            ]"#,
        )
        .expect("prompt templates should be writable");

        SnapshotBootstrapFixture {
            root: root.clone(),
            paths: build_snapshot_bootstrap_prepare_paths(
                snapshot_path,
                manifest_path,
                validator_result_path,
                snapshot_meta_path,
            ),
        }
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("current time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("hope-{prefix}-{unique_suffix}"))
    }

    struct SnapshotBootstrapFixture {
        root: PathBuf,
        paths: SnapshotBootstrapPreparePaths,
    }
}
