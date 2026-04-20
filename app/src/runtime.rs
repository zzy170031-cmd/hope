use std::{collections::HashSet, io};

use crate::{
    ipc::{
        ProjectCreateOrSwitchRequest, StoryboardRenderSegmentCutPreviewSnapshotRequest,
        ValidationExportPanelSnapshotRequest, WriterEntrySnapshotRequest,
    },
    state::{load_desktop_shared_fixture, AppState},
};

use storyboard_pipeline::{StoryboardPlan, StoryboardPlanRequest, StoryboardPlanningError};
use validators::{
    generate_week3_repair_recommendations, generate_week3_validation_report, RepairRecommendation,
    Week3SharedFixture,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCreateOrSwitchSnapshot {
    pub current_project_id: Option<String>,
    pub projects: Vec<ProjectSummaryItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSummaryItem {
    pub project_id: String,
    pub name: String,
    pub status: String,
    pub updated_at: String,
    pub episode_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriterEntrySnapshot {
    pub project_id: String,
    pub synopsis: String,
    pub story: String,
    pub screenplay: String,
    pub storyboard: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryboardRenderSegmentCutPreviewSnapshot {
    pub project_id: String,
    pub storyboard: Vec<PreviewItem>,
    pub render_segment: Vec<PreviewItem>,
    pub cuts: Vec<PreviewItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewItem {
    pub id: String,
    pub label: String,
    pub duration: String,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryboardPreviewPlanRequest {
    pub render_segment_id: String,
    pub narrative_scene_id: String,
    pub render_segment_sequence_no: u32,
    pub start_shot_sequence_no: u32,
    pub end_shot_sequence_no: u32,
    pub target_duration_seconds: u16,
    pub cut_id: String,
    pub cut_sequence_no: u32,
    pub shot_description: String,
    pub dialogue: String,
    pub scene_director_id: Option<String>,
    pub action_director_id: Option<String>,
    pub scene_type: Option<String>,
    pub layout_prompt: String,
    pub render_prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationExportPanelSnapshot {
    pub project_id: String,
    pub summary_items: Vec<ValidationExportPanelItem>,
    pub repair_recommendations: Vec<ValidationRepairRecommendationItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationExportPanelItem {
    pub label: String,
    pub value: String,
    pub state: ValidationExportPanelState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationExportPanelState {
    Ready,
    Pending,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationRepairRecommendationItem {
    pub failure_code: String,
    pub failure_name: String,
    pub repair_strategy: String,
    pub repair_priority: String,
    pub repair_scope: String,
    pub validator_hint: String,
    pub prompt_template_names: Vec<String>,
}

pub fn build_project_create_or_switch_snapshot(
    request: ProjectCreateOrSwitchRequest,
) -> io::Result<ProjectCreateOrSwitchSnapshot> {
    let fixture = load_desktop_shared_fixture()?;
    Ok(build_project_create_or_switch_snapshot_from_fixture(
        request, &fixture,
    ))
}

pub fn build_writer_entry_snapshot(
    request: WriterEntrySnapshotRequest,
) -> io::Result<WriterEntrySnapshot> {
    let fixture = load_desktop_shared_fixture()?;
    Ok(build_writer_entry_snapshot_from_fixture(request, &fixture))
}

pub fn build_storyboard_rendersegment_cut_preview_snapshot(
    state: &AppState,
    request: StoryboardRenderSegmentCutPreviewSnapshotRequest,
) -> io::Result<StoryboardRenderSegmentCutPreviewSnapshot> {
    let fixture = state.load_shared_fixture()?;
    Ok(build_storyboard_rendersegment_cut_preview_snapshot_from_fixture(state, request, &fixture))
}

pub fn build_storyboard_preview_plan(
    state: &AppState,
    request: StoryboardPreviewPlanRequest,
) -> Result<StoryboardPlan, StoryboardPlanningError> {
    let scene_taxonomy = resolve_scene_taxonomy(state, request.scene_type.as_deref());
    let derived_scene_director_id = request.scene_director_id.clone().or_else(|| {
        scene_taxonomy
            .as_ref()
            .map(|taxonomy| format!("taxonomy:{}:scene", taxonomy.scene_taxonomy_id))
    });
    let derived_action_director_id = request.action_director_id.clone().or_else(|| {
        scene_taxonomy
            .as_ref()
            .map(|taxonomy| format!("taxonomy:{}:action", taxonomy.scene_taxonomy_id))
    });
    let layout_prompt = if let Some(taxonomy) = scene_taxonomy.as_ref() {
        if request
            .layout_prompt
            .contains(&format!("场景分类：{}", taxonomy.scene_type))
        {
            request.layout_prompt.clone()
        } else {
            format!(
                "{}\n场景分类：{}",
                request.layout_prompt, taxonomy.scene_type
            )
        }
    } else {
        request.layout_prompt.clone()
    };
    let render_prompt = if let Some(taxonomy) = scene_taxonomy.as_ref() {
        if request
            .render_prompt
            .contains(&format!("连续性优先级：{}", taxonomy.continuity_priority))
        {
            request.render_prompt.clone()
        } else {
            format!(
                "{}\n连续性优先级：{}",
                request.render_prompt, taxonomy.continuity_priority
            )
        }
    } else {
        request.render_prompt.clone()
    };

    storyboard_pipeline::build_storyboard_plan(StoryboardPlanRequest {
        render_segment_id: request.render_segment_id,
        narrative_scene_id: request.narrative_scene_id,
        render_segment_sequence_no: request.render_segment_sequence_no,
        start_shot_sequence_no: request.start_shot_sequence_no,
        end_shot_sequence_no: request.end_shot_sequence_no,
        target_duration_seconds: request.target_duration_seconds,
        cut_id: request.cut_id,
        cut_sequence_no: request.cut_sequence_no,
        shot_description: request.shot_description,
        dialogue: request.dialogue,
        scene_director_id: derived_scene_director_id,
        action_director_id: derived_action_director_id,
        scene_taxonomy,
        layout_prompt,
        render_prompt,
    })
}

pub fn build_validation_export_panel_snapshot(
    state: &AppState,
    request: ValidationExportPanelSnapshotRequest,
) -> io::Result<ValidationExportPanelSnapshot> {
    let fixture = state.load_shared_fixture()?;
    build_validation_export_panel_snapshot_from_fixture(state, request, &fixture)
}

pub fn resolve_scene_taxonomy(
    state: &AppState,
    scene_type: Option<&str>,
) -> Option<core_domain::SceneTaxonomyRecord> {
    let scene_type = scene_type?;

    state
        .kb_knowledge
        .scene_taxonomies
        .iter()
        .find(|taxonomy| {
            taxonomy.scene_type == scene_type
                || taxonomy.display_name == scene_type
                || taxonomy.scene_taxonomy_id == scene_type
        })
        .cloned()
}

fn build_project_create_or_switch_snapshot_from_fixture(
    request: ProjectCreateOrSwitchRequest,
    fixture: &Week3SharedFixture,
) -> ProjectCreateOrSwitchSnapshot {
    let current_project_id = request.project_id.clone().or_else(|| {
        fixture
            .project_meta
            .first()
            .map(|row| row.project_id.clone())
    });

    let projects = fixture
        .project_meta
        .iter()
        .map(|row| ProjectSummaryItem {
            project_id: row.project_id.clone(),
            name: row.title.clone(),
            status: row.status.clone(),
            updated_at: format!("timestamp {}", row.updated_at_timestamp),
            episode_count: fixture
                .episode_meta
                .iter()
                .filter(|episode| episode.project_id == row.project_id)
                .count(),
        })
        .collect();

    ProjectCreateOrSwitchSnapshot {
        current_project_id,
        projects,
    }
}

fn build_writer_entry_snapshot_from_fixture(
    request: WriterEntrySnapshotRequest,
    fixture: &Week3SharedFixture,
) -> WriterEntrySnapshot {
    let episode_ids = project_episode_ids(fixture, &request.project_id);
    let scene_ids = project_scene_ids(fixture, &request.project_id);
    let render_segment_ids = project_render_segment_ids(fixture, &request.project_id);
    let episode_count = episode_ids.len();

    let scenes: Vec<_> = fixture
        .narrative_scene
        .iter()
        .filter(|scene| scene_ids.contains(&scene.narrative_scene_id))
        .collect();
    let cuts: Vec<_> = fixture
        .cut
        .iter()
        .filter(|cut| render_segment_ids.contains(&cut.render_segment_id))
        .collect();
    let render_segment_count = fixture
        .render_segment
        .iter()
        .filter(|segment| render_segment_ids.contains(&segment.render_segment_id))
        .count();
    let handoff_zone_count = fixture
        .handoff_zone
        .iter()
        .filter(|zone| render_segment_ids.contains(&zone.render_segment_id))
        .count();

    let synopsis = if let Some(project) = fixture
        .project_meta
        .iter()
        .find(|row| row.project_id == request.project_id)
    {
        let first_scene = scenes
            .first()
            .map(|scene| format!(" 首个 NarrativeScene：{}。", scene.title))
            .unwrap_or_default();

        format!(
            "项目《{}》当前目标 {} 分钟，已同步 {} 个 Episode。{}",
            project.title, project.target_duration_minutes, episode_count, first_scene
        )
    } else {
        format!("项目 {} 已接入共享 fixture。", request.project_id)
    };

    let story = if scenes.is_empty() {
        format!("项目 {} 暂无 NarrativeScene。", request.project_id)
    } else {
        scenes
            .iter()
            .map(|scene| {
                format!(
                    "第 {} 场 {}：{}",
                    scene.sequence_no, scene.title, scene.summary
                )
            })
            .collect::<Vec<_>>()
            .join(" | ")
    };

    let screenplay = if cuts.is_empty() {
        "当前共享 fixture 暂无 Cut 预览。".to_string()
    } else {
        cuts.iter()
            .take(3)
            .map(|cut| {
                format!(
                    "Cut {:02} / {} / {}",
                    cut.sequence_no, cut.shot_description, cut.dialogue
                )
            })
            .collect::<Vec<_>>()
            .join(" | ")
    };

    let storyboard = format!(
        "{} 个 RenderSegment / {} 个 Cut / {} 个 HandoffZone / {} 个 PromptPackage",
        render_segment_count,
        cuts.len(),
        handoff_zone_count,
        fixture.prompt_package.len()
    );

    WriterEntrySnapshot {
        project_id: request.project_id,
        synopsis,
        story,
        screenplay,
        storyboard,
    }
}

fn build_storyboard_rendersegment_cut_preview_snapshot_from_fixture(
    state: &AppState,
    request: StoryboardRenderSegmentCutPreviewSnapshotRequest,
    fixture: &Week3SharedFixture,
) -> StoryboardRenderSegmentCutPreviewSnapshot {
    let scene_ids = project_scene_ids(fixture, &request.project_id);
    let project_render_segment_ids = project_render_segment_ids(fixture, &request.project_id);

    let mut scenes: Vec<_> = fixture
        .narrative_scene
        .iter()
        .filter(|scene| scene_ids.contains(&scene.narrative_scene_id))
        .collect();
    if let Some(narrative_scene_id) = request.narrative_scene_id.as_ref() {
        scenes.retain(|scene| scene.narrative_scene_id == *narrative_scene_id);
    }
    scenes.sort_by_key(|scene| scene.sequence_no);

    let filtered_scene_ids: HashSet<_> = scenes
        .iter()
        .map(|scene| scene.narrative_scene_id.clone())
        .collect();

    let mut render_segments: Vec<_> = fixture
        .render_segment
        .iter()
        .filter(|segment| project_render_segment_ids.contains(&segment.render_segment_id))
        .filter(|segment| filtered_scene_ids.contains(&segment.narrative_scene_id))
        .collect();
    if let Some(render_segment_id) = request.render_segment_id.as_ref() {
        render_segments.retain(|segment| segment.render_segment_id == *render_segment_id);
    }
    render_segments.sort_by_key(|segment| segment.sequence_no);

    let filtered_render_segment_ids: HashSet<_> = render_segments
        .iter()
        .map(|segment| segment.render_segment_id.clone())
        .collect();

    let storyboard = scenes
        .iter()
        .map(|scene| PreviewItem {
            id: scene.narrative_scene_id.clone(),
            label: format!("第 {:02} 场 {}", scene.sequence_no, scene.title),
            duration: "Scene".to_string(),
            note: scene.summary.clone(),
        })
        .collect();

    let render_segment = render_segments
        .iter()
        .map(|segment| {
            let scene = scenes
                .iter()
                .find(|scene| scene.narrative_scene_id == segment.narrative_scene_id);
            let cut = fixture
                .cut
                .iter()
                .filter(|cut| cut.render_segment_id == segment.render_segment_id)
                .min_by_key(|cut| cut.sequence_no);
            let prompt_body = fixture
                .prompt_package
                .first()
                .map(|prompt| prompt.body.as_str())
                .unwrap_or("等待真实 PromptPackage 注入。");

            let plan_note = match build_storyboard_preview_plan(
                state,
                StoryboardPreviewPlanRequest {
                    render_segment_id: segment.render_segment_id.clone(),
                    narrative_scene_id: segment.narrative_scene_id.clone(),
                    render_segment_sequence_no: segment.sequence_no,
                    start_shot_sequence_no: segment.start_shot_sequence_no,
                    end_shot_sequence_no: segment.end_shot_sequence_no,
                    target_duration_seconds: segment.target_duration_seconds as u16,
                    cut_id: cut
                        .map(|item| item.cut_id.clone())
                        .unwrap_or_else(|| format!("{}-preview-cut", segment.render_segment_id)),
                    cut_sequence_no: cut.map(|item| item.sequence_no).unwrap_or(1),
                    shot_description: cut
                        .map(|item| item.shot_description.clone())
                        .unwrap_or_else(|| "待补充镜头描述".to_string()),
                    dialogue: cut
                        .map(|item| item.dialogue.clone())
                        .unwrap_or_else(|| "待补充对白".to_string()),
                    scene_director_id: Some("desktop-shell:scene".to_string()),
                    action_director_id: Some("desktop-shell:action".to_string()),
                    scene_type: request.scene_type.clone(),
                    layout_prompt: format!(
                        "{} / {}",
                        scene
                            .map(|item| item.title.as_str())
                            .unwrap_or("未命名场景"),
                        cut.map(|item| item.shot_description.as_str())
                            .unwrap_or("待补充镜头描述")
                    ),
                    render_prompt: prompt_body.to_string(),
                },
            ) {
                Ok(plan) => format!(
                    "{} | shots {}-{} | handoff {} -> {} | scene {} | render {}",
                    scene
                        .map(|item| item.title.as_str())
                        .unwrap_or("未命名场景"),
                    plan.render_segment.start_shot_sequence_no,
                    plan.render_segment.end_shot_sequence_no,
                    plan.handoff_zone.start_boundary,
                    plan.handoff_zone.end_boundary,
                    truncate_preview_text(&plan.committee_runtime.prompt_layers.layout_prompt, 48),
                    truncate_preview_text(&plan.committee_runtime.prompt_layers.render_prompt, 64)
                ),
                Err(error) => format!("Preview plan fallback: {}", error),
            };

            PreviewItem {
                id: segment.render_segment_id.clone(),
                label: format!("RenderSegment {:02}", segment.sequence_no),
                duration: format!("{}s", segment.target_duration_seconds),
                note: plan_note,
            }
        })
        .collect();

    let mut cuts: Vec<_> = fixture
        .cut
        .iter()
        .filter(|cut| filtered_render_segment_ids.contains(&cut.render_segment_id))
        .collect();
    cuts.sort_by_key(|cut| cut.sequence_no);
    let cuts = cuts
        .iter()
        .map(|cut| PreviewItem {
            id: cut.cut_id.clone(),
            label: format!("Cut {:02}", cut.sequence_no),
            duration: format!("{}s", cut.duration_seconds),
            note: format!(
                "{} / {}",
                cut.shot_description,
                truncate_preview_text(&cut.dialogue, 48)
            ),
        })
        .collect();

    StoryboardRenderSegmentCutPreviewSnapshot {
        project_id: request.project_id,
        storyboard,
        render_segment,
        cuts,
    }
}

fn build_validation_export_panel_snapshot_from_fixture(
    state: &AppState,
    request: ValidationExportPanelSnapshotRequest,
    fixture: &Week3SharedFixture,
) -> io::Result<ValidationExportPanelSnapshot> {
    let workbook = generate_week3_validation_report(fixture)?;
    let repair_recommendations = generate_week3_repair_recommendations(
        fixture,
        &state.kb_knowledge.failure_patterns,
        &state.kb_knowledge.prompt_templates,
    )?;

    let validation_row_count = workbook.validation_report.len();
    let block_count = workbook
        .validation_report
        .iter()
        .filter(|row| !row.passed)
        .count();
    let validation_state = if block_count > 0 {
        ValidationExportPanelState::Blocked
    } else {
        ValidationExportPanelState::Ready
    };
    let repair_state = if repair_recommendations.is_empty() {
        ValidationExportPanelState::Pending
    } else {
        ValidationExportPanelState::Ready
    };

    let summary_items = vec![
        ValidationExportPanelItem {
            label: "project".to_string(),
            value: request.project_id.clone(),
            state: ValidationExportPanelState::Ready,
        },
        ValidationExportPanelItem {
            label: "validation_report".to_string(),
            value: format!("{} rows / {} blocked", validation_row_count, block_count),
            state: validation_state,
        },
        ValidationExportPanelItem {
            label: "repair_recommendations".to_string(),
            value: format!(
                "{} kb-backed recommendations / snapshot {}",
                repair_recommendations.len(),
                state.kb_runtime.snapshot.snapshot_id
            ),
            state: repair_state,
        },
    ];

    let repair_recommendations = repair_recommendations
        .into_iter()
        .map(map_repair_recommendation)
        .collect();

    Ok(ValidationExportPanelSnapshot {
        project_id: request.project_id,
        summary_items,
        repair_recommendations,
    })
}

fn project_episode_ids(fixture: &Week3SharedFixture, project_id: &str) -> HashSet<String> {
    fixture
        .episode_meta
        .iter()
        .filter(|episode| episode.project_id == project_id)
        .map(|episode| episode.episode_id.clone())
        .collect()
}

fn project_scene_ids(fixture: &Week3SharedFixture, project_id: &str) -> HashSet<String> {
    let episode_ids = project_episode_ids(fixture, project_id);

    fixture
        .narrative_scene
        .iter()
        .filter(|scene| episode_ids.contains(&scene.episode_id))
        .map(|scene| scene.narrative_scene_id.clone())
        .collect()
}

fn project_render_segment_ids(fixture: &Week3SharedFixture, project_id: &str) -> HashSet<String> {
    let scene_ids = project_scene_ids(fixture, project_id);

    fixture
        .render_segment
        .iter()
        .filter(|segment| scene_ids.contains(&segment.narrative_scene_id))
        .map(|segment| segment.render_segment_id.clone())
        .collect()
}

fn truncate_preview_text(text: &str, max_chars: usize) -> String {
    let truncated: String = text.chars().take(max_chars).collect();
    if text.chars().count() > max_chars {
        format!("{}...", truncated)
    } else {
        truncated
    }
}

fn map_repair_recommendation(
    recommendation: RepairRecommendation,
) -> ValidationRepairRecommendationItem {
    ValidationRepairRecommendationItem {
        failure_code: recommendation.failure_code,
        failure_name: recommendation.failure_name,
        repair_strategy: recommendation.repair_strategy,
        repair_priority: recommendation.repair_priority,
        repair_scope: recommendation.repair_scope,
        validator_hint: recommendation.validator_hint,
        prompt_template_names: recommendation
            .prompt_templates
            .into_iter()
            .filter_map(|link| link.prompt_name)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use core_domain::{
        FailurePatternRecord, KbRuntimeSummary, KbSnapshotRecord, PromptTemplateRecord,
        SceneTaxonomyRecord,
    };
    use project_store::{
        DualSqliteConnectionPolicy, KbKnowledgeBundle, KbRuntimeHandle, StoreSkeleton,
    };

    use super::{
        build_project_create_or_switch_snapshot_from_fixture, build_storyboard_preview_plan,
        build_storyboard_rendersegment_cut_preview_snapshot_from_fixture,
        build_validation_export_panel_snapshot_from_fixture,
        build_writer_entry_snapshot_from_fixture, resolve_scene_taxonomy,
        StoryboardPreviewPlanRequest, ValidationExportPanelState,
    };
    use crate::state::load_desktop_shared_fixture;
    use crate::{
        ipc::{
            ProjectCreateOrSwitchRequest, StoryboardRenderSegmentCutPreviewSnapshotRequest,
            ValidationExportPanelSnapshotRequest, WriterEntrySnapshotRequest,
        },
        state::AppState,
    };

    fn test_state() -> AppState {
        let store = StoreSkeleton::new(DualSqliteConnectionPolicy::new(
            "E:/codex/hope-kb/snapshots/hope-kb-v0.1.sqlite3".into(),
            "E:/codex/hope/data/hope.sqlite3".into(),
        ));
        let kb_runtime = KbRuntimeHandle {
            snapshot: KbSnapshotRecord {
                snapshot_id: "hope-kb-v0.1".to_string(),
                snapshot_hash: "test-hash".to_string(),
                seed_format: "hope-kb-sqlite-snapshot-v0.1".to_string(),
                source_name: "hope-kb".to_string(),
                created_at_timestamp: 1_714_000_000,
            },
            summary: KbRuntimeSummary {
                snapshot_id: "hope-kb-v0.1".to_string(),
                snapshot_path: "E:/codex/hope-kb/snapshots/hope-kb-v0.1.sqlite3".to_string(),
                mirror_tables: vec!["scene_taxonomy".to_string(), "failure_pattern".to_string()],
                has_scene_taxonomy: true,
                has_failure_patterns: true,
                has_repair_template_mapping: true,
            },
        };
        let kb_knowledge = KbKnowledgeBundle {
            scene_taxonomies: vec![SceneTaxonomyRecord {
                scene_taxonomy_id: "scene-taxonomy-daily-dialogue".to_string(),
                scene_type: "daily_dialogue".to_string(),
                display_name: "Daily Dialogue".to_string(),
                definition: "Stable conversational scene in one space.".to_string(),
                default_duration_band: "30-60s".to_string(),
                typical_committee_roles: vec!["chief".to_string(), "scene".to_string()],
                default_handoff_out: vec!["scene->emotion".to_string()],
                risk_flags: vec!["flat_rhythm".to_string()],
                continuity_priority: "high".to_string(),
                prompt_focus: vec!["micro_expression".to_string(), "blocking".to_string()],
                source_type: "team_distillation".to_string(),
                source_notes: "test fixture".to_string(),
                confidence_level: "high".to_string(),
                last_reviewed_at: "2026-04-20".to_string(),
            }],
            failure_patterns: vec![
                FailurePatternRecord {
                    failure_pattern_id: "failure-style-drift".to_string(),
                    failure_code: "style_drift".to_string(),
                    failure_name: "Style Drift".to_string(),
                    failure_category: "style".to_string(),
                    symptom: "Adjacent cuts drift away from the locked style.".to_string(),
                    common_causes: vec!["hard lock injection missing".to_string()],
                    detection_hint: "Check Style Unity Validator.".to_string(),
                    repair_strategy: "Re-inject hard locks into render prompts.".to_string(),
                    affected_layers: vec!["prompt_packages".to_string()],
                    validator_hint: "Style Unity Validator".to_string(),
                    repair_template_ids: vec!["prompt-repair-style".to_string()],
                    repair_priority: "high".to_string(),
                    repair_scope: "render_prompt_only".to_string(),
                    suggested_followup_validators: vec!["Style Unity Validator".to_string()],
                    source_type: "team_distillation".to_string(),
                    source_notes: "test fixture".to_string(),
                    confidence_level: "high".to_string(),
                    last_reviewed_at: "2026-04-20".to_string(),
                },
                FailurePatternRecord {
                    failure_pattern_id: "failure-prompt-noise".to_string(),
                    failure_code: "chinese_prompt_noise".to_string(),
                    failure_name: "Chinese Prompt Noise".to_string(),
                    failure_category: "language".to_string(),
                    symptom: "Placeholder or machine-like prompt text leaks into output."
                        .to_string(),
                    common_causes: vec!["placeholder marker in prompt".to_string()],
                    detection_hint: "Check prompt body placeholders.".to_string(),
                    repair_strategy: "Rewrite prompt body as natural Chinese guidance.".to_string(),
                    affected_layers: vec!["prompt_packages".to_string()],
                    validator_hint: "Prompt quality review".to_string(),
                    repair_template_ids: vec!["prompt-repair-language".to_string()],
                    repair_priority: "medium".to_string(),
                    repair_scope: "prompt_rendering_layer".to_string(),
                    suggested_followup_validators: vec!["Prompt quality review".to_string()],
                    source_type: "team_distillation".to_string(),
                    source_notes: "test fixture".to_string(),
                    confidence_level: "high".to_string(),
                    last_reviewed_at: "2026-04-20".to_string(),
                },
            ],
            prompt_templates: vec![
                PromptTemplateRecord {
                    prompt_template_id: "prompt-repair-style".to_string(),
                    stage: "repair_pass".to_string(),
                    name: "Repair Style Lock".to_string(),
                    target_model_family: "qwen-compatible".to_string(),
                    repairs_failure_codes: vec!["style_drift".to_string()],
                },
                PromptTemplateRecord {
                    prompt_template_id: "prompt-repair-language".to_string(),
                    stage: "repair_pass".to_string(),
                    name: "Repair Prompt Language".to_string(),
                    target_model_family: "qwen-compatible".to_string(),
                    repairs_failure_codes: vec!["chinese_prompt_noise".to_string()],
                },
            ],
        };

        AppState::new(store, kb_runtime, kb_knowledge)
    }

    fn test_state_without_scene_taxonomy() -> AppState {
        let mut state = test_state();
        state.kb_knowledge.scene_taxonomies.clear();
        state
    }

    fn test_state_without_repair_mappings() -> AppState {
        let mut state = test_state();
        state.kb_knowledge.failure_patterns.clear();
        state.kb_knowledge.prompt_templates.clear();
        state
    }

    #[test]
    fn resolve_scene_taxonomy_matches_by_scene_type_and_display_name() {
        let state = test_state();

        let by_scene_type = resolve_scene_taxonomy(&state, Some("daily_dialogue"));
        let by_display_name = resolve_scene_taxonomy(&state, Some("Daily Dialogue"));

        assert_eq!(
            by_scene_type
                .as_ref()
                .map(|item| item.scene_taxonomy_id.as_str()),
            Some("scene-taxonomy-daily-dialogue")
        );
        assert_eq!(
            by_display_name
                .as_ref()
                .map(|item| item.scene_type.as_str()),
            Some("daily_dialogue")
        );
    }

    #[test]
    fn build_storyboard_preview_plan_consumes_runtime_scene_taxonomy() {
        let state = test_state();
        let plan = build_storyboard_preview_plan(
            &state,
            StoryboardPreviewPlanRequest {
                render_segment_id: "render-segment-preview-001".to_string(),
                narrative_scene_id: "narrative-scene-preview-001".to_string(),
                render_segment_sequence_no: 1,
                start_shot_sequence_no: 1,
                end_shot_sequence_no: 3,
                target_duration_seconds: 45,
                cut_id: "cut-preview-001".to_string(),
                cut_sequence_no: 1,
                shot_description: "medium shot dialogue".to_string(),
                dialogue: "Let's close this scene before dawn.".to_string(),
                scene_director_id: None,
                action_director_id: None,
                scene_type: Some("daily_dialogue".to_string()),
                layout_prompt: "cool palette, medium shot".to_string(),
                render_prompt: "restrained realism".to_string(),
            },
        )
        .expect("runtime taxonomy should build preview plan");

        assert_eq!(
            plan.render_segment.scene_taxonomy_id.as_deref(),
            Some("scene-taxonomy-daily-dialogue")
        );
        assert_eq!(
            plan.render_segment.scene_type.as_deref(),
            Some("daily_dialogue")
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_scene_director_id,
            "taxonomy:scene-taxonomy-daily-dialogue:scene"
        );
        assert!(plan
            .committee_runtime
            .prompt_layers
            .layout_prompt
            .contains("场景分类：daily_dialogue"));
    }

    #[test]
    fn build_storyboard_preview_plan_falls_back_when_scene_taxonomy_is_missing() {
        let state = test_state_without_scene_taxonomy();
        let plan = build_storyboard_preview_plan(
            &state,
            StoryboardPreviewPlanRequest {
                render_segment_id: "render-segment-preview-002".to_string(),
                narrative_scene_id: "narrative-scene-preview-002".to_string(),
                render_segment_sequence_no: 1,
                start_shot_sequence_no: 1,
                end_shot_sequence_no: 3,
                target_duration_seconds: 45,
                cut_id: "cut-preview-002".to_string(),
                cut_sequence_no: 1,
                shot_description: "close shot reaction".to_string(),
                dialogue: "Hold on the lead before the handoff.".to_string(),
                scene_director_id: Some("scene-director-manual".to_string()),
                action_director_id: Some("action-director-manual".to_string()),
                scene_type: Some("daily_dialogue".to_string()),
                layout_prompt: "close shot, interior".to_string(),
                render_prompt: "restrained realism".to_string(),
            },
        )
        .expect("missing taxonomy should not panic when manual directors exist");

        assert_eq!(plan.render_segment.scene_taxonomy_id, None);
        assert_eq!(plan.render_segment.scene_type, None);
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_scene_director_id,
            "scene-director-manual"
        );
        assert_eq!(
            plan.committee_runtime.prompt_layers.layout_prompt,
            "close shot, interior"
        );
    }

    #[test]
    fn build_project_create_or_switch_snapshot_tracks_fixture_projects() {
        let fixture = load_desktop_shared_fixture().expect("shared fixture should load");

        let snapshot = build_project_create_or_switch_snapshot_from_fixture(
            ProjectCreateOrSwitchRequest {
                project_id: None,
                project_name: None,
            },
            &fixture,
        );

        assert_eq!(
            snapshot.current_project_id.as_deref(),
            Some("project-week3-001")
        );
        assert_eq!(snapshot.projects.len(), 1);
        assert!(snapshot.projects[0].episode_count >= 1);
        assert!(snapshot.projects[0].name.contains("Hope"));
    }

    #[test]
    fn build_writer_entry_snapshot_summarizes_fixture_layers() {
        let fixture = load_desktop_shared_fixture().expect("shared fixture should load");

        let snapshot = build_writer_entry_snapshot_from_fixture(
            WriterEntrySnapshotRequest {
                project_id: "project-week3-001".to_string(),
            },
            &fixture,
        );

        assert!(snapshot.synopsis.contains("分钟"));
        assert!(snapshot.story.contains("第 1 集"));
        assert!(snapshot.screenplay.contains("Cut"));
        assert!(snapshot.storyboard.contains("RenderSegment"));
    }

    #[test]
    fn build_storyboard_rendersegment_cut_preview_snapshot_includes_runtime_details() {
        let state = test_state();
        let fixture = load_desktop_shared_fixture().expect("shared fixture should load");

        let snapshot = build_storyboard_rendersegment_cut_preview_snapshot_from_fixture(
            &state,
            StoryboardRenderSegmentCutPreviewSnapshotRequest {
                project_id: "project-week3-001".to_string(),
                episode_id: None,
                narrative_scene_id: None,
                render_segment_id: None,
                scene_type: Some("daily_dialogue".to_string()),
            },
            &fixture,
        );

        assert!(!snapshot.storyboard.is_empty());
        assert!(!snapshot.render_segment.is_empty());
        assert!(!snapshot.cuts.is_empty());
        assert!(snapshot.render_segment[0].note.contains("handoff"));
        assert!(snapshot.render_segment[0].note.contains("scene"));
    }

    #[test]
    fn build_validation_export_panel_snapshot_surfaces_kb_repairs() {
        let state = test_state();
        let mut fixture = load_desktop_shared_fixture().expect("shared fixture should load");
        fixture.prompt_package[0].body = "TODO: rewrite this prompt body".to_string();

        let snapshot = build_validation_export_panel_snapshot_from_fixture(
            &state,
            ValidationExportPanelSnapshotRequest {
                project_id: "project-week3-001".to_string(),
            },
            &fixture,
        )
        .expect("panel snapshot should build from fixture");

        assert_eq!(snapshot.project_id, "project-week3-001");
        assert_eq!(snapshot.summary_items.len(), 3);
        assert_eq!(
            snapshot.summary_items[2].state,
            ValidationExportPanelState::Ready
        );
        assert!(snapshot
            .repair_recommendations
            .iter()
            .any(|item| item.failure_code == "chinese_prompt_noise"));
        assert!(snapshot
            .repair_recommendations
            .iter()
            .flat_map(|item| item.prompt_template_names.iter())
            .any(|name| name == "Repair Prompt Language"));
    }

    #[test]
    fn build_validation_export_panel_snapshot_stays_bounded_without_repair_mappings() {
        let state = test_state_without_repair_mappings();
        let mut fixture = load_desktop_shared_fixture().expect("shared fixture should load");
        fixture.prompt_package[0].body = "TODO: rewrite this prompt body".to_string();

        let snapshot = build_validation_export_panel_snapshot_from_fixture(
            &state,
            ValidationExportPanelSnapshotRequest {
                project_id: "project-week3-001".to_string(),
            },
            &fixture,
        )
        .expect("panel snapshot should still build without repair mappings");

        assert_eq!(snapshot.repair_recommendations.len(), 0);
        assert_eq!(
            snapshot.summary_items[2].state,
            ValidationExportPanelState::Pending
        );
        assert!(snapshot.summary_items[1].value.contains("rows"));
    }
}
