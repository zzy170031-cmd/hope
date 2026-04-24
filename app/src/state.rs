use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use core_domain::contracts::{ExpandScriptResponse, GenerateStoryboardResponse};
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
}
