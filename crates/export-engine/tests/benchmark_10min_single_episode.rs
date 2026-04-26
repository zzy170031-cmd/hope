mod benchmark_support;

use std::{collections::HashMap, fs};

use benchmark_support::{
    EpisodeSpec, TrackedWeek3ExportSnapshot, read_json, write_fixture, write_validation_report,
};
use storyboard_pipeline::{
    StoryboardPlanRequest, build_storyboard_plan, validate_render_segment_window,
};
use validators::{
    generate_week3_validation_report, load_week3_shared_fixture,
};
use writer_pipeline::{
    DurationPolicy, JsonDocument, ScreenplayInput, ScreenplayOutput, StalePropagation,
    StaleTrigger, StoryInput, StoryOutput, StructuredOutputMode, SynopsisInput,
};

const EPISODE_BRIEF_PATH: &str = r"E:\codex\hope\tests\e2e\10min-single-episode\episode-brief.json";
const SYNOPSIS_PATH: &str = r"E:\codex\hope\tests\e2e\10min-single-episode\synopsis.json";
const STORY_PATH: &str = r"E:\codex\hope\tests\e2e\10min-single-episode\story.json";
const SCREENPLAY_PATH: &str = r"E:\codex\hope\tests\e2e\10min-single-episode\screenplay.json";

fn daily_dialogue_taxonomy() -> storyboard_pipeline::SceneTaxonomyRecord {
    storyboard_pipeline::SceneTaxonomyRecord {
        scene_taxonomy_id: "scene-taxonomy-daily-dialogue".to_string(),
        scene_type: "daily_dialogue".to_string(),
        display_name: "日常对白".to_string(),
        definition: "以人物对白和关系推进为主的日常叙事段落。".to_string(),
        default_duration_band: "30s-90s".to_string(),
        typical_committee_roles: vec!["scene".to_string(), "emotion".to_string()],
        default_handoff_out: vec!["emotion".to_string()],
        risk_flags: vec!["performance_drift".to_string()],
        continuity_priority: "high".to_string(),
        prompt_focus: vec!["人物关系".to_string(), "细微表演".to_string()],
        source_type: "team_curated".to_string(),
        source_notes: "week3 benchmark fixture".to_string(),
        confidence_level: "high".to_string(),
        last_reviewed_at: "2026-04-20".to_string(),
    }
}

fn overlapping_signal_taxonomy() -> storyboard_pipeline::SceneTaxonomyRecord {
    storyboard_pipeline::SceneTaxonomyRecord {
        scene_taxonomy_id: "scene-taxonomy-overlap-001".to_string(),
        scene_type: "knowledge_overlap".to_string(),
        display_name: "知识重叠".to_string(),
        definition: "knowledge-side signals overlap with runtime inputs".to_string(),
        default_duration_band: "30s-90s".to_string(),
        typical_committee_roles: vec![
            "scene".to_string(),
            "action".to_string(),
            "knowledge".to_string(),
        ],
        default_handoff_out: vec!["action".to_string()],
        risk_flags: vec!["signal_collision".to_string()],
        continuity_priority: "high".to_string(),
        prompt_focus: vec!["重叠知识信号".to_string(), "taxonomy emphasis".to_string()],
        source_type: "team_curated".to_string(),
        source_notes: "10min benchmark overlap fixture".to_string(),
        confidence_level: "high".to_string(),
        last_reviewed_at: "2026-04-20".to_string(),
    }
}

fn conflicting_signal_taxonomy() -> storyboard_pipeline::SceneTaxonomyRecord {
    storyboard_pipeline::SceneTaxonomyRecord {
        scene_taxonomy_id: "scene-taxonomy-conflict-001".to_string(),
        scene_type: "knowledge_conflict".to_string(),
        display_name: "知识冲突".to_string(),
        definition: "knowledge-side signals attempt to conflict with runtime inputs"
            .to_string(),
        default_duration_band: "90s-120s".to_string(),
        typical_committee_roles: vec![
            "taxonomy_override".to_string(),
            "knowledge".to_string(),
            "scene".to_string(),
        ],
        default_handoff_out: vec!["knowledge".to_string()],
        risk_flags: vec![
            "signal_collision".to_string(),
            "runtime_override_attempt".to_string(),
        ],
        continuity_priority: "low".to_string(),
        prompt_focus: vec![
            "taxonomy override attempt".to_string(),
            "conflicting knowledge emphasis".to_string(),
        ],
        source_type: "team_curated".to_string(),
        source_notes: "10min benchmark conflict fixture".to_string(),
        confidence_level: "high".to_string(),
        last_reviewed_at: "2026-04-20".to_string(),
    }
}

#[test]
fn benchmark_10min_single_episode_runs_end_to_end() {
    let _tracked_exports = TrackedWeek3ExportSnapshot::capture();
    let fixture_paths = write_fixture(
        "10min single episode",
        10,
        &[EpisodeSpec {
            episode_id: "episode-week3-001",
            title: "第 1 集",
            duration_minutes: 10,
            scene_count: 3,
        }],
    );

    let episode_brief = read_json(EPISODE_BRIEF_PATH);
    let synopsis_json = fs::read_to_string(SYNOPSIS_PATH).expect("synopsis fixture should exist");
    let story_json = fs::read_to_string(STORY_PATH).expect("story fixture should exist");
    let screenplay_json =
        fs::read_to_string(SCREENPLAY_PATH).expect("screenplay fixture should exist");

    let synopsis = SynopsisInput {
        document: JsonDocument::new(synopsis_json.clone()),
    };
    let story_input = StoryInput {
        synopsis: synopsis.clone(),
        duration_policy: DurationPolicy::frozen(),
    };
    let story_output = StoryOutput {
        document: JsonDocument::new(story_json.clone()),
        stale: StalePropagation {
            trigger: StaleTrigger::SynopsisChanged,
        },
    };
    let _screenplay_input = ScreenplayInput {
        story: story_output.clone(),
    };
    let screenplay = read_json(SCREENPLAY_PATH);
    let screenplay_output = ScreenplayOutput {
        narrative_scenes: screenplay["narrative_scenes"]
            .as_array()
            .expect("narrative_scenes should be an array")
            .iter()
            .map(|scene| writer_pipeline::NarrativeSceneDraft {
                document: JsonDocument::new(scene.to_string()),
            })
            .collect(),
        dialogue_turns: screenplay["dialogue_turns"]
            .as_array()
            .expect("dialogue_turns should be an array")
            .iter()
            .map(|turn| writer_pipeline::DialogueTurnDraft {
                document: JsonDocument::new(turn.to_string()),
            })
            .collect(),
        output_mode: StructuredOutputMode::JsonOnly,
    };

    assert_eq!(episode_brief["episode_duration_minutes"].as_u64(), Some(10));
    assert!(synopsis.document.json.contains("10"));
    assert_eq!(
        story_input
            .duration_policy
            .render_segment_target_min_seconds,
        30
    );
    assert_eq!(
        story_input
            .duration_policy
            .render_segment_target_max_seconds,
        90
    );
    assert_eq!(
        screenplay_output.output_mode,
        StructuredOutputMode::JsonOnly
    );
    assert_eq!(screenplay_output.narrative_scenes.len(), 3);
    assert_eq!(screenplay_output.dialogue_turns.len(), 27);
    assert_eq!(story_output.stale.trigger, StaleTrigger::SynopsisChanged);
    assert!(screenplay_json.contains("结果确认"));

    let fixture = load_week3_shared_fixture(fixture_paths.shared_fixture_path())
        .expect("shared fixture should load");
    assert_eq!(fixture.episode_meta.len(), 1);
    assert_eq!(fixture.narrative_scene.len(), 3);
    assert_eq!(fixture.render_segment.len(), 9);
    assert_eq!(fixture.cut.len(), 27);
    assert_eq!(fixture.handoff_zone.len(), 9);
    assert_eq!(fixture.prompt_package.len(), 18);
    assert_eq!(fixture.stale_event.len(), 9);

    let total_segment_seconds: u32 = fixture
        .render_segment
        .iter()
        .map(|segment| segment.target_duration_seconds)
        .sum();
    assert_eq!(total_segment_seconds, 600);

    let scene_to_episode: HashMap<_, _> = fixture
        .narrative_scene
        .iter()
        .map(|scene| (scene.narrative_scene_id.as_str(), scene.episode_id.as_str()))
        .collect();
    let segment_to_scene: HashMap<_, _> = fixture
        .render_segment
        .iter()
        .map(|segment| {
            (
                segment.render_segment_id.as_str(),
                segment.narrative_scene_id.as_str(),
            )
        })
        .collect();

    for render_segment in &fixture.render_segment {
        validate_render_segment_window(
            render_segment.target_duration_seconds as u16,
            render_segment.start_shot_sequence_no,
            render_segment.end_shot_sequence_no,
        )
        .expect("all render segments should stay in the frozen duration window");
        assert_eq!(
            scene_to_episode.get(render_segment.narrative_scene_id.as_str()),
            Some(&"episode-week3-001")
        );
    }

    let profile_id = fixture.director_profile[0].director_profile_id.clone();
    for render_segment in &fixture.render_segment {
        let representative_cut = fixture
            .cut
            .iter()
            .find(|cut| cut.render_segment_id == render_segment.render_segment_id)
            .expect("each render segment should have at least one cut");
        let layout_prompt = fixture
            .prompt_package
            .iter()
            .find(|prompt| {
                prompt.source_level == "layout_prompt"
                    && prompt
                        .prompt_package_id
                        .contains(render_segment.render_segment_id.as_str())
            })
            .expect("each render segment should have a layout prompt");
        let render_prompt = fixture
            .prompt_package
            .iter()
            .find(|prompt| {
                prompt.source_level == "render_prompt"
                    && prompt
                        .prompt_package_id
                        .contains(render_segment.render_segment_id.as_str())
            })
            .expect("each render segment should have a render prompt");

        let plan = build_storyboard_plan(StoryboardPlanRequest {
            render_segment_id: render_segment.render_segment_id.clone(),
            narrative_scene_id: render_segment.narrative_scene_id.clone(),
            render_segment_sequence_no: render_segment.sequence_no,
            start_shot_sequence_no: render_segment.start_shot_sequence_no,
            end_shot_sequence_no: render_segment.end_shot_sequence_no,
            target_duration_seconds: render_segment.target_duration_seconds as u16,
            cut_id: representative_cut.cut_id.clone(),
            cut_sequence_no: representative_cut.sequence_no,
            shot_description: representative_cut.shot_description.clone(),
            dialogue: representative_cut.dialogue.clone(),
            scene_director_id: Some(profile_id.clone()),
            action_director_id: None,
            scene_taxonomy: Some(daily_dialogue_taxonomy()),
            layout_prompt: layout_prompt.body.clone(),
            render_prompt: render_prompt.body.clone(),
        })
        .expect("storyboard plan should build for every render segment");

        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_scene_director_id,
            profile_id
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_action_director_id,
            profile_id
        );
        assert_eq!(
            plan.committee_runtime.prompt_layers.layout_prompt,
            layout_prompt.body
        );
        assert_eq!(
            plan.committee_runtime.prompt_layers.render_prompt,
            render_prompt.body
        );

        let handoff = fixture
            .handoff_zone
            .iter()
            .find(|handoff| handoff.render_segment_id == render_segment.render_segment_id)
            .expect("each render segment should have a handoff zone");
        let taxonomy_variants = vec![
            ("none", None, None, None, None),
            (
                "daily",
                Some(daily_dialogue_taxonomy()),
                Some("scene-taxonomy-daily-dialogue"),
                Some("daily_dialogue"),
                Some("high"),
            ),
            (
                "overlap",
                Some(overlapping_signal_taxonomy()),
                Some("scene-taxonomy-overlap-001"),
                Some("knowledge_overlap"),
                Some("high"),
            ),
            (
                "conflict",
                Some(conflicting_signal_taxonomy()),
                Some("scene-taxonomy-conflict-001"),
                Some("knowledge_conflict"),
                Some("low"),
            ),
        ];

        for (
            label,
            scene_taxonomy,
            expected_taxonomy_id,
            expected_scene_type,
            expected_continuity_priority,
        ) in taxonomy_variants
        {
            let plan = build_storyboard_plan(StoryboardPlanRequest {
                render_segment_id: render_segment.render_segment_id.clone(),
                narrative_scene_id: render_segment.narrative_scene_id.clone(),
                render_segment_sequence_no: render_segment.sequence_no,
                start_shot_sequence_no: render_segment.start_shot_sequence_no,
                end_shot_sequence_no: render_segment.end_shot_sequence_no,
                target_duration_seconds: render_segment.target_duration_seconds as u16,
                cut_id: representative_cut.cut_id.clone(),
                cut_sequence_no: representative_cut.sequence_no,
                shot_description: representative_cut.shot_description.clone(),
                dialogue: representative_cut.dialogue.clone(),
                scene_director_id: Some(profile_id.clone()),
                action_director_id: None,
                scene_taxonomy,
                layout_prompt: layout_prompt.body.clone(),
                render_prompt: render_prompt.body.clone(),
            })
            .unwrap_or_else(|_| {
                panic!(
                    "storyboard plan should stay valid for every taxonomy variant: {}",
                    label
                )
            });

            assert_eq!(
                plan.committee_runtime
                    .director_assignment
                    .primary_scene_director_id,
                profile_id,
                "scene director changed under taxonomy variant {}",
                label
            );
            assert_eq!(
                plan.committee_runtime
                    .director_assignment
                    .primary_action_director_id,
                profile_id,
                "action director changed under taxonomy variant {}",
                label
            );
            assert_eq!(
                plan.committee_runtime.prompt_layers.layout_prompt,
                layout_prompt.body,
                "layout prompt changed under taxonomy variant {}",
                label
            );
            assert_eq!(
                plan.committee_runtime.prompt_layers.render_prompt,
                render_prompt.body,
                "render prompt changed under taxonomy variant {}",
                label
            );
            assert_eq!(
                plan.handoff_zone.start_boundary,
                handoff.start_boundary,
                "handoff start changed under taxonomy variant {}",
                label
            );
            assert_eq!(
                plan.handoff_zone.end_boundary,
                handoff.end_boundary,
                "handoff end changed under taxonomy variant {}",
                label
            );
            assert_eq!(
                plan.handoff_zone.boundary_type,
                handoff.boundary_type,
                "handoff boundary changed under taxonomy variant {}",
                label
            );
            assert_eq!(
                plan.render_segment.target_duration_seconds as u32,
                render_segment.target_duration_seconds,
                "target duration changed under taxonomy variant {}",
                label
            );
            assert_eq!(
                plan.render_segment.start_shot_sequence_no,
                render_segment.start_shot_sequence_no,
                "shot window start changed under taxonomy variant {}",
                label
            );
            assert_eq!(
                plan.render_segment.end_shot_sequence_no,
                render_segment.end_shot_sequence_no,
                "shot window end changed under taxonomy variant {}",
                label
            );
            assert_eq!(
                plan.render_segment.scene_taxonomy_id.as_deref(),
                expected_taxonomy_id,
                "render-segment taxonomy visibility changed unexpectedly under variant {}",
                label
            );
            assert_eq!(
                plan.render_segment.scene_type.as_deref(),
                expected_scene_type,
                "render-segment scene type visibility changed unexpectedly under variant {}",
                label
            );
            assert_eq!(
                plan.render_segment.continuity_priority.as_deref(),
                expected_continuity_priority,
                "continuity priority visibility changed unexpectedly under variant {}",
                label
            );
            assert_eq!(
                plan.committee_runtime.scene_taxonomy_id.as_deref(),
                expected_taxonomy_id,
                "runtime taxonomy visibility changed unexpectedly under variant {}",
                label
            );
            assert_eq!(
                plan.committee_runtime.scene_type.as_deref(),
                expected_scene_type,
                "runtime scene type visibility changed unexpectedly under variant {}",
                label
            );
        }
    }

    for hard_lock in &fixture.hard_lock {
        assert_eq!(hard_lock.scope, "project");
        for cut in &fixture.cut {
            let render_segment_scene = segment_to_scene
                .get(cut.render_segment_id.as_str())
                .expect("cut should map to a render segment");
            let episode_id = scene_to_episode
                .get(*render_segment_scene)
                .expect("render segment scene should map to episode");
            assert_eq!(hard_lock.project_id, fixture.project_meta[0].project_id);
            assert_eq!(*episode_id, "episode-week3-001");
        }
    }

    for prompt in &fixture.prompt_package {
        let referenced_segment = fixture
            .render_segment
            .iter()
            .find(|segment| {
                prompt
                    .prompt_package_id
                    .contains(segment.render_segment_id.as_str())
            })
            .expect("prompt package should trace to a render segment");
        let narrative_scene_id = segment_to_scene
            .get(referenced_segment.render_segment_id.as_str())
            .expect("render segment should trace to a narrative scene");
        let episode_id = scene_to_episode
            .get(*narrative_scene_id)
            .expect("narrative scene should trace to the episode");

        assert!(prompt.prompt_package_id.contains("episode-week3-001"));
        assert!(
            fixture
                .cut
                .iter()
                .any(|cut| cut.render_segment_id == referenced_segment.render_segment_id)
        );
        assert_eq!(*episode_id, "episode-week3-001");
    }

    let validation = generate_week3_validation_report(&fixture)
        .expect("validation report should generate from shared fixture");
    assert_eq!(validation.validation_report.len(), 73);
    assert!(validation.validation_report.iter().all(|row| row.passed));
    assert!(
        validation
            .validation_report
            .iter()
            .all(|row| row.problem_count == 0)
    );

    write_validation_report(
        fixture_paths.validation_report_path(),
        serde_json::to_string_pretty(&validation)
            .expect("validation report should serialize for export"),
    );
    let export_bundle = fixture_paths.export_week3().expect("export should succeed");
    assert_eq!(export_bundle.workbook.sheets.len(), 17);
    assert!(export_bundle.excel_path.exists());
    assert!(export_bundle.json_path.exists());
    assert!(export_bundle.markdown_path.exists());

    let export_json = read_json(&export_bundle.json_path);
    assert_eq!(
        export_json["episode_meta"]
            .as_array()
            .expect("episode_meta")
            .len(),
        1
    );
    assert_eq!(
        export_json["narrative_scene"]
            .as_array()
            .expect("narrative_scene")
            .len(),
        3
    );
    assert_eq!(
        export_json["render_segment"]
            .as_array()
            .expect("render_segment")
            .len(),
        9
    );
    assert_eq!(export_json["cut"].as_array().expect("cut").len(), 27);
    assert_eq!(
        export_json["validation_report"]
            .as_array()
            .expect("validation_report")
            .len(),
        validation.validation_report.len()
    );
    assert_eq!(
        export_json["render_segment"][8]["目标时长秒"].as_str(),
        Some("66")
    );

    let markdown =
        fs::read_to_string(&export_bundle.markdown_path).expect("markdown export should exist");
    assert!(markdown.contains("## 集元数据 / `episode_meta`"));
    assert!(markdown.contains("## 叙事场景 / `narrative_scene`"));
    assert!(markdown.contains("## RenderSegment / `render_segment`"));
}
