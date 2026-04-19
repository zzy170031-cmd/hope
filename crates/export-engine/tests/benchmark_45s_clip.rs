mod benchmark_support;

use std::{fs, path::Path};

use benchmark_support::{read_json, write_45s_clip_fixture};
use export_engine::{
    WEEK3_EXPORT_JSON_PATH, WEEK3_EXPORT_MARKDOWN_PATH, WEEK3_EXPORT_XLSX_PATH,
    export_week3_from_fixtures,
};
use storyboard_pipeline::validate_render_segment_window;
use validators::{
    WEEK3_VALIDATION_REPORT_PATH, generate_week3_validation_report, load_week3_shared_fixture,
    write_week3_validation_report,
};
use writer_pipeline::{
    DurationPolicy, JsonDocument, ScreenplayInput, ScreenplayOutput, StalePropagation,
    StaleTrigger, StoryInput, StoryOutput, StructuredOutputMode, SynopsisInput,
};

const CLIP_BRIEF_PATH: &str = r"E:\codex\hope\tests\e2e\45s-clip\clip-brief.json";
const SYNOPSIS_PATH: &str = r"E:\codex\hope\tests\e2e\45s-clip\synopsis.json";
const STORY_PATH: &str = r"E:\codex\hope\tests\e2e\45s-clip\story.json";
const SCREENPLAY_PATH: &str = r"E:\codex\hope\tests\e2e\45s-clip\screenplay.json";

#[test]
fn benchmark_45s_clip_runs_end_to_end() {
    write_45s_clip_fixture();

    let clip_brief = read_json(CLIP_BRIEF_PATH);
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

    assert_eq!(clip_brief["clip_duration_seconds"].as_u64(), Some(45));
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
    assert_eq!(screenplay_output.narrative_scenes.len(), 1);
    assert_eq!(screenplay_output.dialogue_turns.len(), 3);
    assert_eq!(story_output.stale.trigger, StaleTrigger::SynopsisChanged);
    assert!(synopsis.document.json.contains("45"));
    assert!(screenplay_json.contains("开场任务"));

    let fixture = load_week3_shared_fixture(validators::WEEK3_SHARED_FIXTURE_PATH)
        .expect("shared fixture should load");
    assert_eq!(fixture.project_meta.len(), 1);
    assert!(fixture.narrative_scene.len() >= 1);
    assert!(fixture.render_segment.len() >= 1);
    assert!(fixture.cut.len() >= 3);
    assert!(fixture.handoff_zone.len() >= 1);
    assert!(fixture.prompt_package.len() >= 2);

    let render_segment = &fixture.render_segment[0];
    validate_render_segment_window(
        render_segment.target_duration_seconds as u16,
        render_segment.start_shot_sequence_no,
        render_segment.end_shot_sequence_no,
    )
    .expect("render segment should stay in the frozen duration window");
    assert_eq!(render_segment.target_duration_seconds, 45);
    assert_eq!(render_segment.actual_duration_seconds, 45);

    for cut in fixture.cut.iter().take(3) {
        assert_eq!(cut.render_segment_id, render_segment.render_segment_id);
        assert_eq!(cut.duration_seconds, 15);
    }

    let handoff = &fixture.handoff_zone[0];
    assert_eq!(handoff.render_segment_id, render_segment.render_segment_id);
    assert_ne!(handoff.start_boundary, handoff.end_boundary);

    let scene_id = &fixture.narrative_scene[0].narrative_scene_id;
    assert_eq!(&render_segment.narrative_scene_id, scene_id);

    for hard_lock in &fixture.hard_lock {
        assert_eq!(hard_lock.scope, "project");
        assert_eq!(hard_lock.project_id, fixture.project_meta[0].project_id);
    }

    let validation = generate_week3_validation_report(&fixture)
        .expect("validation report should generate from shared fixture");
    assert!(validation.validation_report.len() >= 7);
    assert!(validation.validation_report.iter().all(|row| row.passed));
    assert!(
        validation
            .validation_report
            .iter()
            .all(|row| row.problem_count == 0)
    );

    write_week3_validation_report(WEEK3_VALIDATION_REPORT_PATH)
        .expect("validation report file should be written");
    let export_bundle = export_week3_from_fixtures().expect("export should succeed");
    assert_eq!(export_bundle.workbook.sheets.len(), 17);
    assert!(Path::new(WEEK3_EXPORT_XLSX_PATH).exists());
    assert!(Path::new(WEEK3_EXPORT_JSON_PATH).exists());
    assert!(Path::new(WEEK3_EXPORT_MARKDOWN_PATH).exists());

    let export_json = read_json(WEEK3_EXPORT_JSON_PATH);
    assert_eq!(
        export_json["workbook_chinese_name"].as_str(),
        Some("Hope 导出工作簿")
    );
    assert_eq!(
        export_json["render_segment"][0]["目标时长秒"].as_str(),
        Some("45")
    );
    assert_eq!(export_json["cut"][0]["时长秒"].as_str(), Some("15"));
    assert_eq!(
        export_json["validation_report"]
            .as_array()
            .expect("validation_report should exist")
            .len(),
        validation.validation_report.len()
    );

    let markdown =
        fs::read_to_string(WEEK3_EXPORT_MARKDOWN_PATH).expect("markdown export should exist");
    assert!(markdown.contains("# Hope 导出工作簿"));
    assert!(markdown.contains("## 校验报告 / `validation_report`"));
    assert!(markdown.contains("render-segment-week3-001"));
}
