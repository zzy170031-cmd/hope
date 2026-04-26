use std::{
    collections::HashMap,
    env, io,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::ipc::{ConfigureTextModelProviderRequest, TextModelProviderStatus};
use core_domain::{
    ExpandScriptResponse, FinalizedStoryboardShotResult, GenerateStoryboardResponse,
    KbGoldenSampleRuntimePackage,
};
use project_store::{
    DualSqliteConnectionPolicy, KbKnowledgeBundle, KbRuntimeError, KbRuntimeHandle, StoreSkeleton,
    load_kb_golden_sample_runtime_package, load_kb_knowledge_bundle, load_kb_runtime,
};
use serde::Serialize;
use validators::{Week3SharedFixture, load_week3_shared_fixture};

pub const DEFAULT_HOPE_KB_SNAPSHOT_PATH: &str = "E:/codex/hope-kb/snapshots/hope-kb-v0.1.sqlite3";
pub const DEFAULT_HOPE_DB_PATH: &str = "E:/codex/hope/data/hope.sqlite3";
pub const DEFAULT_DESKTOP_SHARED_FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../contracts/fixtures/week3-shared-fixture.json"
);

#[derive(Debug, Clone)]
pub struct DesktopSourceConfig {
    pub hope_db_path: PathBuf,
    pub shared_fixture_path: PathBuf,
    pub hope_db_available: bool,
    pub shared_fixture_available: bool,
}

impl DesktopSourceConfig {
    pub fn new(hope_db_path: PathBuf, shared_fixture_path: PathBuf) -> Self {
        let hope_db_available = hope_db_path.is_file();
        let shared_fixture_available = shared_fixture_path.is_file();

        Self {
            hope_db_path,
            shared_fixture_path,
            hope_db_available,
            shared_fixture_available,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SnapshotBootstrapReadonlyState {
    pub snapshot_identity: SnapshotBootstrapSnapshotIdentity,
    pub summary_capabilities: SnapshotBootstrapSummaryCapabilities,
    pub knowledge_bundle: SnapshotBootstrapKnowledgeBundleStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SnapshotBootstrapSnapshotIdentity {
    pub snapshot_id: String,
    pub snapshot_hash: String,
    pub seed_format: String,
    pub source_name: String,
    pub created_at_timestamp: i64,
    pub snapshot_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SnapshotBootstrapSummaryCapabilities {
    pub has_scene_taxonomy: bool,
    pub has_failure_patterns: bool,
    pub has_repair_template_mapping: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SnapshotBootstrapKnowledgeBundleStatus {
    pub scene_taxonomy_count: usize,
    pub failure_pattern_count: usize,
    pub prompt_template_count: usize,
    pub scene_taxonomies_ready: bool,
    pub failure_patterns_ready: bool,
    pub prompt_templates_ready: bool,
    pub repair_mappings_ready: bool,
}

impl SnapshotBootstrapReadonlyState {
    pub fn from_verified_sources(
        kb_runtime: &KbRuntimeHandle,
        kb_knowledge: &KbKnowledgeBundle,
    ) -> Self {
        let scene_taxonomy_count = kb_knowledge.scene_taxonomies.len();
        let failure_pattern_count = kb_knowledge.failure_patterns.len();
        let prompt_template_count = kb_knowledge.prompt_templates.len();
        let summary = &kb_runtime.summary;

        Self {
            snapshot_identity: SnapshotBootstrapSnapshotIdentity {
                snapshot_id: kb_runtime.snapshot.snapshot_id.clone(),
                snapshot_hash: kb_runtime.snapshot.snapshot_hash.clone(),
                seed_format: kb_runtime.snapshot.seed_format.clone(),
                source_name: kb_runtime.snapshot.source_name.clone(),
                created_at_timestamp: kb_runtime.snapshot.created_at_timestamp,
                snapshot_path: summary.snapshot_path.clone(),
            },
            summary_capabilities: SnapshotBootstrapSummaryCapabilities {
                has_scene_taxonomy: summary.has_scene_taxonomy,
                has_failure_patterns: summary.has_failure_patterns,
                has_repair_template_mapping: summary.has_repair_template_mapping,
            },
            knowledge_bundle: SnapshotBootstrapKnowledgeBundleStatus {
                scene_taxonomy_count,
                failure_pattern_count,
                prompt_template_count,
                scene_taxonomies_ready: summary.has_scene_taxonomy && scene_taxonomy_count > 0,
                failure_patterns_ready: summary.has_failure_patterns && failure_pattern_count > 0,
                prompt_templates_ready: summary.has_repair_template_mapping
                    && prompt_template_count > 0,
                repair_mappings_ready: summary.has_repair_template_mapping
                    && failure_pattern_count > 0
                    && prompt_template_count > 0,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationFeedbackReadonlyState {
    pub source_snapshot_id: String,
    pub source_snapshot_hash: String,
    pub source_snapshot_path: String,
    pub has_failure_patterns: bool,
    pub has_repair_template_mapping: bool,
    pub failure_pattern_count: usize,
    pub prompt_template_count: usize,
    pub repair_mapping_ready: bool,
}

impl ValidationFeedbackReadonlyState {
    pub fn from_verified_sources(
        kb_runtime: &KbRuntimeHandle,
        kb_knowledge: &KbKnowledgeBundle,
    ) -> Self {
        let failure_pattern_count = kb_knowledge.failure_patterns.len();
        let prompt_template_count = kb_knowledge.prompt_templates.len();
        let summary = &kb_runtime.summary;

        Self {
            source_snapshot_id: kb_runtime.snapshot.snapshot_id.clone(),
            source_snapshot_hash: kb_runtime.snapshot.snapshot_hash.clone(),
            source_snapshot_path: summary.snapshot_path.clone(),
            has_failure_patterns: summary.has_failure_patterns,
            has_repair_template_mapping: summary.has_repair_template_mapping,
            failure_pattern_count,
            prompt_template_count,
            repair_mapping_ready: summary.has_repair_template_mapping
                && failure_pattern_count > 0
                && prompt_template_count > 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextModelProviderSessionConfig {
    pub provider: String,
    pub model: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub api_key_ref: Option<String>,
    pub enabled: bool,
}

impl TextModelProviderSessionConfig {
    pub fn from_request(request: ConfigureTextModelProviderRequest) -> Self {
        let provider = normalize_provider_id(&request.provider);
        let enabled = provider == "qwen" && request.enabled;
        Self {
            provider,
            model: request.model.trim().to_string(),
            base_url: non_empty_trimmed(request.base_url),
            api_key: request
                .api_key
                .and_then(non_empty_trimmed)
                .map(|value| value.trim().to_string()),
            api_key_ref: request.api_key_ref.and_then(non_empty_trimmed),
            enabled,
        }
    }

    pub fn has_api_key(&self) -> bool {
        self.api_key
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
            || self.resolve_api_key_ref().is_some()
    }

    pub fn resolved_api_key(&self) -> Option<String> {
        self.api_key
            .as_deref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .or_else(|| self.resolve_api_key_ref())
    }

    fn resolve_api_key_ref(&self) -> Option<String> {
        self.api_key_ref
            .as_deref()
            .and_then(|value| value.strip_prefix("env:"))
            .and_then(|name| env::var(name).ok())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn to_status(&self) -> TextModelProviderStatus {
        let base_url_present = self
            .base_url
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty());
        let api_key_present = self.has_api_key();
        let model_present = !self.model.trim().is_empty();
        let live_ready = self.provider == "qwen"
            && self.enabled
            && model_present
            && base_url_present
            && api_key_present;
        let (status, message) = if self.provider != "qwen" {
            ("reserved", "该模型接口为预留状态，当前未启用真实调用。")
        } else if !model_present || !base_url_present || !api_key_present {
            (
                "unconfigured",
                "千问：未配置，缺少 model/base_url/API Key 中的一项。",
            )
        } else if !self.enabled {
            (
                "configured_disabled",
                "千问：已配置未启用，当前仍使用本地候选结果。",
            )
        } else {
            (
                "enabled",
                "千问：已启用，扩写和生成会优先尝试 live 文本生成。",
            )
        };

        TextModelProviderStatus {
            provider: self.provider.clone(),
            model: self.model.clone(),
            enabled: self.enabled,
            base_url_present,
            api_key_present,
            live_ready,
            status: status.to_string(),
            message: message.to_string(),
            storage: "session-only".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub store: StoreSkeleton,
    pub kb_runtime: KbRuntimeHandle,
    pub kb_knowledge: KbKnowledgeBundle,
    pub kb_golden_sample_runtime: Option<KbGoldenSampleRuntimePackage>,
    pub desktop_sources: DesktopSourceConfig,
    pub snapshot_bootstrap_readonly: SnapshotBootstrapReadonlyState,
    pub validation_feedback_readonly: ValidationFeedbackReadonlyState,
    bridge_scripts: Arc<Mutex<HashMap<String, ExpandScriptResponse>>>,
    bridge_storyboards: Arc<Mutex<HashMap<String, GenerateStoryboardResponse>>>,
    storyboard_tasks: Arc<Mutex<HashMap<String, String>>>,
    finalized_storyboard_bank:
        Arc<Mutex<HashMap<String, HashMap<String, FinalizedStoryboardShotResult>>>>,
    text_model_provider_session: Arc<Mutex<Option<TextModelProviderSessionConfig>>>,
}

impl AppState {
    pub fn new(
        store: StoreSkeleton,
        kb_runtime: KbRuntimeHandle,
        kb_knowledge: KbKnowledgeBundle,
    ) -> Self {
        Self::new_with_sources(
            store,
            kb_runtime,
            kb_knowledge,
            None,
            DesktopSourceConfig::new(
                PathBuf::from(DEFAULT_HOPE_DB_PATH),
                PathBuf::from(DEFAULT_DESKTOP_SHARED_FIXTURE_PATH),
            ),
        )
    }

    pub fn new_with_sources(
        store: StoreSkeleton,
        kb_runtime: KbRuntimeHandle,
        kb_knowledge: KbKnowledgeBundle,
        kb_golden_sample_runtime: Option<KbGoldenSampleRuntimePackage>,
        desktop_sources: DesktopSourceConfig,
    ) -> Self {
        let snapshot_bootstrap_readonly =
            SnapshotBootstrapReadonlyState::from_verified_sources(&kb_runtime, &kb_knowledge);
        let validation_feedback_readonly =
            ValidationFeedbackReadonlyState::from_verified_sources(&kb_runtime, &kb_knowledge);

        Self {
            store,
            kb_runtime,
            kb_knowledge,
            kb_golden_sample_runtime,
            desktop_sources,
            snapshot_bootstrap_readonly,
            validation_feedback_readonly,
            bridge_scripts: Arc::new(Mutex::new(HashMap::new())),
            bridge_storyboards: Arc::new(Mutex::new(HashMap::new())),
            storyboard_tasks: Arc::new(Mutex::new(HashMap::new())),
            finalized_storyboard_bank: Arc::new(Mutex::new(HashMap::new())),
            text_model_provider_session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn new_with_golden_sample_runtime(
        store: StoreSkeleton,
        kb_runtime: KbRuntimeHandle,
        kb_knowledge: KbKnowledgeBundle,
        kb_golden_sample_runtime: KbGoldenSampleRuntimePackage,
    ) -> Self {
        Self::new_with_sources(
            store,
            kb_runtime,
            kb_knowledge,
            Some(kb_golden_sample_runtime),
            DesktopSourceConfig::new(
                PathBuf::from(DEFAULT_HOPE_DB_PATH),
                PathBuf::from(DEFAULT_DESKTOP_SHARED_FIXTURE_PATH),
            ),
        )
    }

    pub fn load_desktop_runtime() -> io::Result<Self> {
        Self::load_from_paths(
            PathBuf::from(DEFAULT_HOPE_KB_SNAPSHOT_PATH),
            PathBuf::from(DEFAULT_HOPE_DB_PATH),
        )
    }

    pub fn load_from_paths(
        hope_kb_snapshot_path: PathBuf,
        hope_db_path: PathBuf,
    ) -> io::Result<Self> {
        Self::load_from_paths_with_fixture(
            hope_kb_snapshot_path,
            hope_db_path,
            PathBuf::from(DEFAULT_DESKTOP_SHARED_FIXTURE_PATH),
        )
    }

    pub fn load_from_paths_with_fixture(
        hope_kb_snapshot_path: PathBuf,
        hope_db_path: PathBuf,
        shared_fixture_path: PathBuf,
    ) -> io::Result<Self> {
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            hope_kb_snapshot_path.clone(),
            hope_db_path.clone(),
        ));
        let kb_runtime = load_kb_runtime(hope_kb_snapshot_path).map_err(map_kb_runtime_error)?;
        let kb_knowledge = load_kb_knowledge_bundle(&kb_runtime).map_err(map_kb_runtime_error)?;
        let kb_golden_sample_runtime = load_kb_golden_sample_runtime_package(&kb_runtime).ok();

        Ok(Self::new_with_sources(
            store,
            kb_runtime,
            kb_knowledge,
            kb_golden_sample_runtime,
            DesktopSourceConfig::new(hope_db_path, shared_fixture_path),
        ))
    }

    pub fn load_shared_fixture(&self) -> io::Result<Week3SharedFixture> {
        load_shared_fixture_from_path(&self.desktop_sources.shared_fixture_path)
    }

    pub fn snapshot_bootstrap_readonly_state(&self) -> &SnapshotBootstrapReadonlyState {
        &self.snapshot_bootstrap_readonly
    }

    pub fn validation_feedback_readonly_state(&self) -> &ValidationFeedbackReadonlyState {
        &self.validation_feedback_readonly
    }

    pub fn remember_script(&self, script: ExpandScriptResponse) {
        self.bridge_scripts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(script.script_id.clone(), script);
    }

    pub fn find_script(&self, script_id: &str) -> Option<ExpandScriptResponse> {
        self.bridge_scripts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(script_id)
            .cloned()
    }

    pub fn remember_storyboard(&self, storyboard: GenerateStoryboardResponse) {
        if let Some(task_id) = storyboard.task_id.clone() {
            self.storyboard_tasks
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .insert(task_id, storyboard.result_id.clone());
        }
        self.bridge_storyboards
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(storyboard.result_id.clone(), storyboard);
    }

    pub fn find_storyboard(&self, result_id: &str) -> Option<GenerateStoryboardResponse> {
        self.bridge_storyboards
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(result_id)
            .cloned()
    }

    pub fn find_storyboard_by_task_id(&self, task_id: &str) -> Option<GenerateStoryboardResponse> {
        let result_id = self
            .storyboard_tasks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(task_id)
            .cloned()?;

        self.find_storyboard(&result_id)
    }

    pub fn remember_finalized_storyboard_shot(&self, shot: FinalizedStoryboardShotResult) {
        self.finalized_storyboard_bank
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entry(shot.project_id.clone())
            .or_default()
            .insert(shot.result_id.clone(), shot);
    }

    pub fn find_finalized_storyboard_shot(
        &self,
        project_id: &str,
        result_id: &str,
    ) -> Option<FinalizedStoryboardShotResult> {
        self.finalized_storyboard_bank
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(project_id)
            .and_then(|shots| shots.get(result_id))
            .cloned()
    }

    pub fn list_finalized_storyboard_shots(
        &self,
        project_id: &str,
        script_id: Option<&str>,
        confirmed: Option<bool>,
    ) -> Vec<FinalizedStoryboardShotResult> {
        let mut shots = self
            .finalized_storyboard_bank
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(project_id)
            .map(|shots| shots.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        shots.retain(|shot| {
            script_id.is_none_or(|script_id| shot.script_id == script_id)
                && confirmed.is_none_or(|confirmed| shot.confirmed == confirmed)
        });
        shots.sort_by(|left, right| {
            left.shot_order
                .cmp(&right.shot_order)
                .then(left.updated_at_ms.cmp(&right.updated_at_ms))
                .then(left.result_id.cmp(&right.result_id))
        });
        shots
    }

    pub fn remove_finalized_storyboard_shot(&self, project_id: &str, result_id: &str) -> bool {
        self.finalized_storyboard_bank
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get_mut(project_id)
            .and_then(|shots| shots.remove(result_id))
            .is_some()
    }

    pub fn configure_text_model_provider(
        &self,
        request: ConfigureTextModelProviderRequest,
    ) -> TextModelProviderStatus {
        let config = TextModelProviderSessionConfig::from_request(request);
        let status = config.to_status();
        self.text_model_provider_session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .replace(config);
        status
    }

    pub fn text_model_provider_session_config(&self) -> Option<TextModelProviderSessionConfig> {
        self.text_model_provider_session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

pub fn load_desktop_shared_fixture() -> io::Result<Week3SharedFixture> {
    load_shared_fixture_from_path(&PathBuf::from(DEFAULT_DESKTOP_SHARED_FIXTURE_PATH))
}

fn map_kb_runtime_error(error: KbRuntimeError) -> io::Error {
    match error {
        KbRuntimeError::SnapshotMissing { path } => io::Error::new(
            io::ErrorKind::NotFound,
            format!("KB snapshot is missing: {}", path.display()),
        ),
        KbRuntimeError::SnapshotMetadataUnreadable { path, message } => io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "KB snapshot metadata could not be read at {}: {}",
                path.display(),
                message
            ),
        ),
        KbRuntimeError::SeedBundlePathMissing { path } => io::Error::new(
            io::ErrorKind::NotFound,
            format!("KB seed bundle path is missing: {}", path.display()),
        ),
        KbRuntimeError::SeedBundleUnreadable { path, message } => io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "KB seed bundle could not be read at {}: {}",
                path.display(),
                message
            ),
        ),
        KbRuntimeError::SeedBundleInvalid { path, message } => io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "KB seed bundle is invalid at {}: {}",
                path.display(),
                message
            ),
        ),
    }
}

fn load_shared_fixture_from_path(path: &PathBuf) -> io::Result<Week3SharedFixture> {
    load_week3_shared_fixture(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "desktop shared fixture could not be loaded from {}: {}",
                path.display(),
                error
            ),
        )
    })
}

fn normalize_provider_id(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "doubao" => "doubao".to_string(),
        "custom" => "custom".to_string(),
        _ => "qwen".to_string(),
    }
}

fn non_empty_trimmed(value: impl AsRef<str>) -> Option<String> {
    let trimmed = value.as_ref().trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{AppState, DEFAULT_DESKTOP_SHARED_FIXTURE_PATH, load_desktop_shared_fixture};

    #[test]
    fn load_from_paths_builds_app_state_from_kb_runtime_assets() {
        let repo_root = unique_temp_dir("hope-app-state");
        let snapshots_dir = repo_root.join("snapshots");
        let seed_dir = repo_root.join("seed").join("v0.1");
        fs::create_dir_all(&snapshots_dir).expect("snapshots dir should be creatable");
        fs::create_dir_all(&seed_dir).expect("seed dir should be creatable");

        let snapshot_path = snapshots_dir.join("hope-kb-v0.1.sqlite3");
        File::create(&snapshot_path).expect("snapshot file should be creatable");
        write_seed_bundle(&seed_dir);

        let state =
            AppState::load_from_paths(snapshot_path.clone(), repo_root.join("hope.sqlite3"))
                .expect("app state should load from snapshot and seed bundle");

        assert_eq!(
            state.store.connection_policy.hope_kb.file_path,
            snapshot_path
        );
        assert_eq!(state.kb_runtime.snapshot.snapshot_id, "hope-kb-v0.1");
        assert_eq!(state.kb_knowledge.scene_taxonomies.len(), 1);
        assert_eq!(state.kb_knowledge.failure_patterns.len(), 1);
        assert_eq!(state.kb_knowledge.prompt_templates.len(), 1);
        assert!(!state.desktop_sources.hope_db_available);
        assert!(state.desktop_sources.shared_fixture_available);
        assert_eq!(
            state.desktop_sources.shared_fixture_path,
            PathBuf::from(DEFAULT_DESKTOP_SHARED_FIXTURE_PATH)
        );
        let readonly_state = state.snapshot_bootstrap_readonly_state();
        assert_eq!(readonly_state.snapshot_identity.snapshot_id, "hope-kb-v0.1");
        assert_eq!(
            readonly_state.snapshot_identity.snapshot_hash,
            "runtime-unverified"
        );
        assert_eq!(
            readonly_state.snapshot_identity.seed_format,
            "hope-kb-sqlite-snapshot-v0.1+golden-sample-v0.2"
        );
        assert_eq!(readonly_state.snapshot_identity.source_name, "hope-kb");
        assert!(readonly_state.snapshot_identity.created_at_timestamp >= 0);
        assert_eq!(
            readonly_state.snapshot_identity.snapshot_path,
            snapshot_path.display().to_string()
        );
        assert!(readonly_state.summary_capabilities.has_scene_taxonomy);
        assert!(readonly_state.summary_capabilities.has_failure_patterns);
        assert!(
            readonly_state
                .summary_capabilities
                .has_repair_template_mapping
        );
        assert_eq!(readonly_state.knowledge_bundle.scene_taxonomy_count, 1);
        assert_eq!(readonly_state.knowledge_bundle.failure_pattern_count, 1);
        assert_eq!(readonly_state.knowledge_bundle.prompt_template_count, 1);
        assert!(readonly_state.knowledge_bundle.scene_taxonomies_ready);
        assert!(readonly_state.knowledge_bundle.failure_patterns_ready);
        assert!(readonly_state.knowledge_bundle.prompt_templates_ready);
        assert!(readonly_state.knowledge_bundle.repair_mappings_ready);
        let validation_feedback = state.validation_feedback_readonly_state();
        assert_eq!(validation_feedback.source_snapshot_id, "hope-kb-v0.1");
        assert_eq!(
            validation_feedback.source_snapshot_hash,
            "runtime-unverified"
        );
        assert_eq!(
            validation_feedback.source_snapshot_path,
            snapshot_path.display().to_string()
        );
        assert!(validation_feedback.has_failure_patterns);
        assert!(validation_feedback.has_repair_template_mapping);
        assert_eq!(validation_feedback.failure_pattern_count, 1);
        assert_eq!(validation_feedback.prompt_template_count, 1);
        assert!(validation_feedback.repair_mapping_ready);

        fs::remove_dir_all(repo_root).expect("temp repo root should be removable");
    }

    #[test]
    fn load_from_paths_reports_missing_snapshot() {
        let repo_root = unique_temp_dir("hope-app-missing-snapshot");
        let error = AppState::load_from_paths(
            repo_root.join("snapshots").join("missing.sqlite3"),
            repo_root.join("hope.sqlite3"),
        )
        .expect_err("missing snapshot should fail");

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert!(error.to_string().contains("KB snapshot is missing"));
    }

    #[test]
    fn load_shared_fixture_reports_configured_missing_source() {
        let repo_root = unique_temp_dir("hope-app-missing-fixture");
        let snapshots_dir = repo_root.join("snapshots");
        let seed_dir = repo_root.join("seed").join("v0.1");
        fs::create_dir_all(&snapshots_dir).expect("snapshots dir should be creatable");
        fs::create_dir_all(&seed_dir).expect("seed dir should be creatable");

        let snapshot_path = snapshots_dir.join("hope-kb-v0.1.sqlite3");
        let missing_fixture_path = repo_root.join("fixtures").join("missing.json");
        File::create(&snapshot_path).expect("snapshot file should be creatable");
        write_seed_bundle(&seed_dir);

        let state = AppState::load_from_paths_with_fixture(
            snapshot_path,
            repo_root.join("hope.sqlite3"),
            missing_fixture_path.clone(),
        )
        .expect("app state should load even when the content source is unavailable");

        assert!(!state.desktop_sources.shared_fixture_available);
        let error = state
            .load_shared_fixture()
            .expect_err("missing shared source should fail at panel load time");

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert!(
            error
                .to_string()
                .contains(&missing_fixture_path.display().to_string())
        );

        fs::remove_dir_all(repo_root).expect("temp repo root should be removable");
    }

    #[test]
    fn load_desktop_shared_fixture_prefers_workspace_fixture_copy() {
        let fixture = load_desktop_shared_fixture()
            .expect("workspace fixture should load through desktop source helper");

        assert!(!fixture.project_meta.is_empty());
        assert!(!fixture.render_segment.is_empty());
    }

    fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{}-{}-{}", prefix, std::process::id(), suffix))
    }

    fn write_seed_bundle(seed_dir: &std::path::Path) {
        fs::write(
            seed_dir.join("scene_taxonomy.json"),
            r#"[
              {
                "machine_id": "scene-taxonomy-daily-dialogue",
                "scene_type": "daily_dialogue",
                "display_name": "Daily Dialogue",
                "definition": "Stable conversational scene in one space.",
                "default_duration_band": "30-60s",
                "typical_committee_roles": ["chief", "scene"],
                "default_handoff_out": ["scene->emotion"],
                "risk_flags": ["flat_rhythm"],
                "continuity_priority": "high",
                "prompt_focus": ["micro_expression", "blocking"],
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
                "machine_id": "failure-prompt-noise",
                "failure_code": "chinese_prompt_noise",
                "failure_name": "Chinese Prompt Noise",
                "failure_category": "language",
                "symptom": "Placeholder or machine-like prompt text leaks into output.",
                "common_causes": ["placeholder marker in prompt"],
                "detection_hint": "Check prompt body placeholders.",
                "repair_strategy": "Rewrite prompt body as natural Chinese guidance.",
                "affected_layers": ["prompt_packages"],
                "validator_hint": "Prompt quality review",
                "repair_template_ids": ["prompt-repair-language"],
                "repair_priority": "medium",
                "repair_scope": "prompt_rendering_layer",
                "suggested_followup_validator": ["Prompt quality review"],
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
                "machine_id": "prompt-repair-language",
                "stage": "repair_pass",
                "name": "Repair Prompt Language",
                "target_model_family": "qwen-compatible",
                "repairs_failure_codes": ["chinese_prompt_noise"]
              }
            ]"#,
        )
        .expect("prompt template seed should be writable");
    }
}
