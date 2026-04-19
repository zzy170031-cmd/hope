#![forbid(unsafe_code)]

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Week3SharedFixture {
    #[serde(rename = "project_meta")]
    pub project_meta: Vec<ProjectMetaRow>,
    #[serde(rename = "episode_meta")]
    pub episode_meta: Vec<EpisodeMetaRow>,
    #[serde(rename = "narrative_scene")]
    pub narrative_scene: Vec<NarrativeSceneRow>,
    #[serde(rename = "render_segment")]
    pub render_segment: Vec<RenderSegmentRow>,
    #[serde(rename = "cut")]
    pub cut: Vec<CutRow>,
    #[serde(rename = "prompt_package")]
    pub prompt_package: Vec<PromptPackageRow>,
    #[serde(rename = "handoff_zone")]
    pub handoff_zone: Vec<HandoffZoneRow>,
    #[serde(rename = "hard_lock")]
    pub hard_lock: Vec<HardLockRow>,
    #[serde(rename = "stale_event")]
    pub stale_event: Vec<StaleEventRow>,
    #[serde(rename = "director_profile")]
    pub director_profile: Vec<DirectorProfileRow>,
    #[serde(rename = "director_cut_sample")]
    pub director_cut_sample: Vec<DirectorCutSampleRow>,
    #[serde(rename = "committee_template")]
    pub committee_template: Vec<CommitteeTemplateRow>,
    #[serde(rename = "visual_term")]
    pub visual_term: Vec<VisualTermRow>,
    #[serde(rename = "cinematography_term")]
    pub cinematography_term: Vec<CinematographyTermRow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectMetaRow {
    #[serde(rename = "项目标识")]
    pub project_id: String,
    #[serde(rename = "标题")]
    pub title: String,
    #[serde(rename = "状态")]
    pub status: String,
    #[serde(rename = "目标时长分钟")]
    pub target_duration_minutes: u16,
    #[serde(rename = "更新时间戳")]
    pub updated_at_timestamp: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EpisodeMetaRow {
    #[serde(rename = "集标识")]
    pub episode_id: String,
    #[serde(rename = "项目标识")]
    pub project_id: String,
    #[serde(rename = "序号")]
    pub sequence_no: u32,
    #[serde(rename = "标题")]
    pub title: String,
    #[serde(rename = "目标时长分钟")]
    pub target_duration_minutes: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NarrativeSceneRow {
    #[serde(rename = "叙事场景标识")]
    pub narrative_scene_id: String,
    #[serde(rename = "集标识")]
    pub episode_id: String,
    #[serde(rename = "序号")]
    pub sequence_no: u32,
    #[serde(rename = "标题")]
    pub title: String,
    #[serde(rename = "内容摘要")]
    pub summary: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RenderSegmentRow {
    #[serde(rename = "RenderSegment标识")]
    pub render_segment_id: String,
    #[serde(rename = "叙事场景标识")]
    pub narrative_scene_id: String,
    #[serde(rename = "序号")]
    pub sequence_no: u32,
    #[serde(rename = "起始镜头序号")]
    pub start_shot_sequence_no: u32,
    #[serde(rename = "结束镜头序号")]
    pub end_shot_sequence_no: u32,
    #[serde(rename = "目标时长秒")]
    pub target_duration_seconds: u32,
    #[serde(rename = "实际时长秒")]
    pub actual_duration_seconds: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CutRow {
    #[serde(rename = "Cut标识")]
    pub cut_id: String,
    #[serde(rename = "RenderSegment标识")]
    pub render_segment_id: String,
    #[serde(rename = "序号")]
    pub sequence_no: u32,
    #[serde(rename = "镜头描述")]
    pub shot_description: String,
    #[serde(rename = "对白")]
    pub dialogue: String,
    #[serde(rename = "时长秒")]
    pub duration_seconds: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PromptPackageRow {
    #[serde(rename = "PromptPackage标识")]
    pub prompt_package_id: String,
    #[serde(rename = "来源层级")]
    pub source_level: String,
    #[serde(rename = "正文")]
    pub body: String,
    #[serde(rename = "版本")]
    pub version: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HandoffZoneRow {
    #[serde(rename = "HandoffZone标识")]
    pub handoff_zone_id: String,
    #[serde(rename = "RenderSegment标识")]
    pub render_segment_id: String,
    #[serde(rename = "起始边界")]
    pub start_boundary: String,
    #[serde(rename = "结束边界")]
    pub end_boundary: String,
    #[serde(rename = "边界类型")]
    pub boundary_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HardLockRow {
    #[serde(rename = "HardLock标识")]
    pub hard_lock_id: String,
    #[serde(rename = "项目标识")]
    pub project_id: String,
    #[serde(rename = "锁名")]
    pub lock_name: String,
    #[serde(rename = "锁值")]
    pub lock_value: String,
    #[serde(rename = "作用范围")]
    pub scope: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StaleEventRow {
    #[serde(rename = "StaleEvent标识")]
    pub stale_event_id: String,
    #[serde(rename = "来源层级")]
    pub source_level: String,
    #[serde(rename = "来源标识")]
    pub source_id: String,
    #[serde(rename = "目标层级")]
    pub target_level: String,
    #[serde(rename = "目标标识")]
    pub target_id: String,
    #[serde(rename = "追踪标识")]
    pub trace_id: String,
    #[serde(rename = "时间戳")]
    pub timestamp: i64,
    #[serde(rename = "原因")]
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirectorProfileRow {
    #[serde(rename = "导演档案标识")]
    pub director_profile_id: String,
    #[serde(rename = "导演名称")]
    pub director_name: String,
    #[serde(rename = "定位")]
    pub position: String,
    #[serde(rename = "主风格")]
    pub primary_style: String,
    #[serde(rename = "镜头偏好")]
    pub shot_preference: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirectorCutSampleRow {
    #[serde(rename = "导演切样标识")]
    pub director_cut_sample_id: String,
    #[serde(rename = "导演档案标识")]
    pub director_profile_id: String,
    #[serde(rename = "样例标题")]
    pub sample_title: String,
    #[serde(rename = "样例内容")]
    pub sample_body: String,
    #[serde(rename = "代表性说明")]
    pub representative_note: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitteeTemplateRow {
    #[serde(rename = "模板标识")]
    pub committee_template_id: String,
    #[serde(rename = "模板名称")]
    pub template_name: String,
    #[serde(rename = "适用场景")]
    pub applicable_scene: String,
    #[serde(rename = "成员构成")]
    pub member_composition: String,
    #[serde(rename = "职责描述")]
    pub responsibility: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VisualTermRow {
    #[serde(rename = "术语标识")]
    pub term_id: String,
    #[serde(rename = "中文术语")]
    pub chinese_term: String,
    #[serde(rename = "类别")]
    pub category: String,
    #[serde(rename = "定义")]
    pub definition: String,
    #[serde(rename = "别名")]
    pub alias: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CinematographyTermRow {
    #[serde(rename = "术语标识")]
    pub term_id: String,
    #[serde(rename = "中文术语")]
    pub chinese_term: String,
    #[serde(rename = "类别")]
    pub category: String,
    #[serde(rename = "定义")]
    pub definition: String,
    #[serde(rename = "别名")]
    pub alias: String,
}
