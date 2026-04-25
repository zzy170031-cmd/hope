use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use core_domain::contracts::{
    ExpandScriptResponse, FinalizedStoryboardShotResult, GenerateStoryboardResponse,
};
use core_domain::kb::KbGoldenSampleRuntimePackage;
use project_store::{KbKnowledgeBundle, KbRuntimeHandle, StoreSkeleton};

#[derive(Debug, Clone)]
pub struct AppState {
    pub store: StoreSkeleton,
    pub kb_runtime: KbRuntimeHandle,
    pub kb_knowledge: KbKnowledgeBundle,
    pub kb_golden_sample_runtime: KbGoldenSampleRuntimePackage,
    bridge_scripts: Arc<Mutex<HashMap<String, ExpandScriptResponse>>>,
    bridge_storyboards: Arc<Mutex<HashMap<String, GenerateStoryboardResponse>>>,
    storyboard_tasks: Arc<Mutex<HashMap<String, String>>>,
    finalized_storyboard_bank:
        Arc<Mutex<HashMap<String, HashMap<String, FinalizedStoryboardShotResult>>>>,
}

impl AppState {
    pub fn new(
        store: StoreSkeleton,
        kb_runtime: KbRuntimeHandle,
        kb_knowledge: KbKnowledgeBundle,
        kb_golden_sample_runtime: KbGoldenSampleRuntimePackage,
    ) -> Self {
        Self {
            store,
            kb_runtime,
            kb_knowledge,
            kb_golden_sample_runtime,
            bridge_scripts: Arc::new(Mutex::new(HashMap::new())),
            bridge_storyboards: Arc::new(Mutex::new(HashMap::new())),
            storyboard_tasks: Arc::new(Mutex::new(HashMap::new())),
            finalized_storyboard_bank: Arc::new(Mutex::new(HashMap::new())),
        }
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
}
