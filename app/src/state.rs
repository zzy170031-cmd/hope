use project_store::{KbKnowledgeBundle, KbRuntimeHandle, StoreSkeleton};

#[derive(Debug, Clone)]
pub struct AppState {
    pub store: StoreSkeleton,
    pub kb_runtime: KbRuntimeHandle,
    pub kb_knowledge: KbKnowledgeBundle,
}

impl AppState {
    pub fn new(
        store: StoreSkeleton,
        kb_runtime: KbRuntimeHandle,
        kb_knowledge: KbKnowledgeBundle,
    ) -> Self {
        Self {
            store,
            kb_runtime,
            kb_knowledge,
        }
    }
}
