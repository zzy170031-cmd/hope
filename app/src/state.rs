use std::{io, path::PathBuf};

use project_store::{
    load_kb_knowledge_bundle, load_kb_runtime, DualSqliteConnectionPolicy, KbKnowledgeBundle,
    KbRuntimeError, KbRuntimeHandle, StoreSkeleton,
};
use validators::{load_week3_shared_fixture, Week3SharedFixture};

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
}

impl DesktopSourceConfig {
    pub fn new(hope_db_path: PathBuf, shared_fixture_path: PathBuf) -> Self {
        let hope_db_available = hope_db_path.is_file();

        Self {
            hope_db_path,
            shared_fixture_path,
            hope_db_available,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub store: StoreSkeleton,
    pub kb_runtime: KbRuntimeHandle,
    pub kb_knowledge: KbKnowledgeBundle,
    pub desktop_sources: DesktopSourceConfig,
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
        desktop_sources: DesktopSourceConfig,
    ) -> Self {
        Self {
            store,
            kb_runtime,
            kb_knowledge,
            desktop_sources,
        }
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
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            hope_kb_snapshot_path.clone(),
            hope_db_path.clone(),
        ));
        let kb_runtime = load_kb_runtime(hope_kb_snapshot_path).map_err(map_kb_runtime_error)?;
        let kb_knowledge = load_kb_knowledge_bundle(&kb_runtime).map_err(map_kb_runtime_error)?;

        Ok(Self::new_with_sources(
            store,
            kb_runtime,
            kb_knowledge,
            DesktopSourceConfig::new(
                hope_db_path,
                PathBuf::from(DEFAULT_DESKTOP_SHARED_FIXTURE_PATH),
            ),
        ))
    }

    pub fn load_shared_fixture(&self) -> io::Result<Week3SharedFixture> {
        load_shared_fixture_from_path(&self.desktop_sources.shared_fixture_path)
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

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{load_desktop_shared_fixture, AppState, DEFAULT_DESKTOP_SHARED_FIXTURE_PATH};

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
        assert_eq!(
            state.desktop_sources.shared_fixture_path,
            PathBuf::from(DEFAULT_DESKTOP_SHARED_FIXTURE_PATH)
        );

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
