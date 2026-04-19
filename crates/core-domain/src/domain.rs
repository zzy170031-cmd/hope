#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HierarchyLevel {
    Project,
    Episode,
    NarrativeScene,
    RenderSegment,
    Cut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HierarchyRef {
    pub level: HierarchyLevel,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRecord {
    pub project_id: String,
    pub title: String,
    pub status: String,
    pub target_duration_minutes: u16,
    pub created_at_timestamp: i64,
    pub updated_at_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeRecord {
    pub episode_id: String,
    pub project_id: String,
    pub sequence_no: u32,
    pub title: String,
    pub status: String,
    pub target_duration_minutes: u16,
    pub created_at_timestamp: i64,
    pub updated_at_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NarrativeSceneRecord {
    pub narrative_scene_id: String,
    pub episode_id: String,
    pub sequence_no: u32,
    pub title: String,
    pub summary: String,
    pub created_at_timestamp: i64,
    pub updated_at_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSegmentRecord {
    pub render_segment_id: String,
    pub narrative_scene_id: String,
    pub sequence_no: u32,
    pub start_shot_sequence_no: u32,
    pub end_shot_sequence_no: u32,
    pub target_duration_seconds: u16,
    pub actual_duration_seconds: Option<u16>,
    pub status: String,
    pub created_at_timestamp: i64,
    pub updated_at_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutRecord {
    pub cut_id: String,
    pub render_segment_id: String,
    pub sequence_no: u32,
    pub shot_description: String,
    pub dialogue: String,
    pub duration_seconds: u16,
    pub status: String,
    pub created_at_timestamp: i64,
    pub updated_at_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffZoneRecord {
    pub handoff_zone_id: String,
    pub render_segment_id: String,
    pub start_boundary: String,
    pub end_boundary: String,
    pub boundary_type: String,
    pub note: String,
    pub created_at_timestamp: i64,
    pub updated_at_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardLockRecord {
    pub hard_lock_id: String,
    pub project_id: String,
    pub lock_name: String,
    pub lock_value: String,
    pub scope: String,
    pub updated_at_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentBoundary {
    pub render_segment_id: String,
    pub start_shot_sequence_no: u32,
    pub end_shot_sequence_no: u32,
    pub target_duration_seconds: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleEventRecord {
    pub stale_event_id: String,
    pub source_level: HierarchyLevel,
    pub source_id: String,
    pub target_level: HierarchyLevel,
    pub target_id: String,
    pub trace_id: String,
    pub timestamp: i64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorProfileRecord {
    pub director_profile_id: String,
    pub director_name: String,
    pub position: String,
    pub primary_style: String,
    pub shot_preference: String,
    pub continuity_preference: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorCutSampleRecord {
    pub director_cut_sample_id: String,
    pub director_profile_id: String,
    pub sample_title: String,
    pub sample_body: String,
    pub representative_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptPackageSource {
    pub source: HierarchyRef,
    pub body: String,
}
