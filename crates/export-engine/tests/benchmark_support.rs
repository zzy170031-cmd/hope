use std::fs;

use export_engine::WEEK3_SHARED_FIXTURE_PATH;
use serde_json::{Value, json};

pub struct EpisodeSpec<'a> {
    pub episode_id: &'a str,
    pub title: &'a str,
    pub duration_minutes: u32,
    pub scene_count: u32,
}

pub fn write_45s_clip_fixture() {
    let fixture = json!({
        "project_meta": [
            {
                "项目标识": "project-week3-001",
                "标题": "Hope 45s clip",
                "状态": "冻结中",
                "目标时长分钟": 1,
                "更新时间戳": 1713513600
            }
        ],
        "episode_meta": [
            {
                "集标识": "episode-week3-001",
                "项目标识": "project-week3-001",
                "序号": 1,
                "标题": "第1集",
                "目标时长分钟": 1
            }
        ],
        "narrative_scene": [
            {
                "叙事场景标识": "narrative-scene-week3-001",
                "集标识": "episode-week3-001",
                "序号": 1,
                "标题": "第1集-场景1",
                "内容摘要": "45s clip 的单场景闭环样例。"
            }
        ],
        "render_segment": [
            {
                "RenderSegment标识": "render-segment-week3-001",
                "叙事场景标识": "narrative-scene-week3-001",
                "序号": 1,
                "起始镜头序号": 10,
                "结束镜头序号": 12,
                "目标时长秒": 45,
                "实际时长秒": 45
            }
        ],
        "cut": [
            {
                "Cut标识": "cut-week3-001",
                "RenderSegment标识": "render-segment-week3-001",
                "序号": 1,
                "镜头描述": "render-segment-week3-001 的第 1 个 cut，保持中景推进。",
                "对白": "第1集第 1 拍继续执行。",
                "时长秒": 15
            },
            {
                "Cut标识": "cut-week3-002",
                "RenderSegment标识": "render-segment-week3-001",
                "序号": 2,
                "镜头描述": "render-segment-week3-001 的第 2 个 cut，保持中景推进。",
                "对白": "第1集第 2 拍继续执行。",
                "时长秒": 15
            },
            {
                "Cut标识": "cut-week3-003",
                "RenderSegment标识": "render-segment-week3-001",
                "序号": 3,
                "镜头描述": "render-segment-week3-001 的第 3 个 cut，保持中景推进。",
                "对白": "第1集第 3 拍继续执行。",
                "时长秒": 15
            }
        ],
        "prompt_package": [
            {
                "PromptPackage标识": "prompt-package-layout-episode-week3-001-render-segment-week3-001",
                "来源层级": "layout_prompt",
                "正文": "冷色调，中景，单段 45 秒布局稳定推进。",
                "版本": 1
            },
            {
                "PromptPackage标识": "prompt-package-render-episode-week3-001-render-segment-week3-001",
                "来源层级": "render_prompt",
                "正文": "冷色调、克制写实光线、中景连续调度，围绕 render-segment-week3-001 稳定输出。",
                "版本": 1
            }
        ],
        "handoff_zone": [
            {
                "HandoffZone标识": "handoff-zone-week3-001",
                "RenderSegment标识": "render-segment-week3-001",
                "起始边界": "10",
                "结束边界": "12",
                "边界类型": "render_segment_boundary"
            }
        ],
        "hard_lock": [
            {
                "HardLock标识": "hard-lock-week3-001",
                "项目标识": "project-week3-001",
                "锁名": "negative_prompt_guard",
                "锁值": "禁止出现血腥暴力词汇",
                "作用范围": "project"
            }
        ],
        "stale_event": [
            {
                "StaleEvent标识": "stale-event-week3-001",
                "来源层级": "render_segment",
                "来源标识": "render-segment-week3-001",
                "目标层级": "cut",
                "目标标识": "cut-week3-001",
                "追踪标识": "trace-render-segment-week3-001-cut-week3-001",
                "时间戳": 1713513601,
                "原因": "render_segment updated"
            }
        ],
        "export_manifest": [
            {
                "导出清单标识": "export-manifest-week3-001",
                "项目标识": "project-week3-001",
                "工作簿版本": "45s-clip-fixture",
                "状态": "准备导出"
            }
        ],
        "director_profile": [
            {
                "导演档案标识": "director-profile-week3-001",
                "导演名称": "导演甲",
                "定位": "场景主导",
                "主风格": "克制写实",
                "镜头偏好": "中景与近景结合"
            }
        ],
        "director_cut_sample": [
            {
                "导演切样标识": "director-cut-sample-week3-001",
                "导演档案标识": "director-profile-week3-001",
                "样例标题": "45s clip切样",
                "样例内容": "45s clip 维持冷色调、中景、稳定切分。",
                "代表性说明": "用于 45s benchmark 的统一视觉与动作节奏参考。"
            }
        ],
        "committee_template": [
            {
                "模板标识": "committee-template-week3-001",
                "模板名称": "基础评审委员会",
                "适用场景": "常规叙事段落",
                "成员构成": "场景导演,动作导演",
                "职责描述": "统一镜头节奏、动作推进与提示词风格。"
            }
        ],
        "visual_term": [
            {
                "术语标识": "visual-term-week3-001",
                "中文术语": "冷色调",
                "类别": "色彩",
                "定义": "偏冷且低饱和的整体色彩基调。",
                "别名": "冷调"
            }
        ],
        "cinematography_term": [
            {
                "术语标识": "cinematography-term-week3-001",
                "中文术语": "中景",
                "类别": "景别",
                "定义": "突出人物与环境关系的中距离景别。",
                "别名": "中镜"
            }
        ],
        "continuity_rule": [
            {
                "规则标识": "continuity-rule-week3-001",
                "规则名称": "场景动作连续",
                "规则说明": "同一叙事场景内镜头衔接保持动作方向一致。",
                "适用层级": "NarrativeScene"
            }
        ]
    });

    fs::write(
        WEEK3_SHARED_FIXTURE_PATH,
        serde_json::to_string_pretty(&fixture).expect("fixture should serialize"),
    )
    .expect("shared fixture should be written");
}

pub fn write_fixture(
    benchmark_label: &str,
    project_duration_minutes: u32,
    episode_specs: &[EpisodeSpec<'_>],
) {
    let mut narrative_scenes = Vec::new();
    let mut render_segments = Vec::new();
    let mut cuts = Vec::new();
    let mut prompt_packages = Vec::new();
    let mut handoff_zones = Vec::new();
    let mut stale_events = Vec::new();
    let mut episode_meta = Vec::new();

    let mut global_scene_index = 1u32;
    let mut global_segment_index = 1u32;
    let mut global_cut_index = 1u32;
    let mut shot_cursor = 10u32;

    for (episode_sequence, episode) in episode_specs.iter().enumerate() {
        episode_meta.push(json!({
            "集标识": episode.episode_id,
            "项目标识": "project-week3-001",
            "序号": episode_sequence + 1,
            "标题": episode.title,
            "目标时长分钟": episode.duration_minutes
        }));

        let segment_count = episode.scene_count * 3;
        let total_episode_seconds = episode.duration_minutes * 60;
        let base_segment_seconds = total_episode_seconds / segment_count;
        let segment_remainder = total_episode_seconds % segment_count;

        for scene_sequence in 0..episode.scene_count {
            let scene_id = format!("narrative-scene-week3-{:03}", global_scene_index);
            narrative_scenes.push(json!({
                "叙事场景标识": scene_id,
                "集标识": episode.episode_id,
                "序号": scene_sequence + 1,
                "标题": format!("{}-场景{}", episode.title, scene_sequence + 1),
                "内容摘要": format!("{}的第{}个叙事场景，保持中景与冷色调推进。", episode.title, scene_sequence + 1)
            }));

            for local_segment_index in 0..3u32 {
                let segment_id = format!("render-segment-week3-{:03}", global_segment_index);
                let segment_order = scene_sequence * 3 + local_segment_index;
                let duration_seconds =
                    base_segment_seconds + u32::from(segment_order < segment_remainder);
                let cut_durations = split_cut_durations(duration_seconds);
                let start_shot = shot_cursor;
                let end_shot = shot_cursor + 2;

                render_segments.push(json!({
                    "RenderSegment标识": segment_id,
                    "叙事场景标识": scene_id,
                    "序号": global_segment_index,
                    "起始镜头序号": start_shot,
                    "结束镜头序号": end_shot,
                    "目标时长秒": duration_seconds,
                    "实际时长秒": duration_seconds
                }));

                prompt_packages.push(json!({
                    "PromptPackage标识": format!("prompt-package-layout-{}-{}", episode.episode_id, segment_id),
                    "来源层级": "layout_prompt",
                    "正文": format!("冷色调，中景，{}的{}布局稳定推进。", episode.title, segment_id),
                    "版本": 1
                }));
                prompt_packages.push(json!({
                    "PromptPackage标识": format!("prompt-package-render-{}-{}", episode.episode_id, segment_id),
                    "来源层级": "render_prompt",
                    "正文": format!("冷色调、克制写实光线、中景连续调度，围绕{}稳定输出。", segment_id),
                    "版本": 1
                }));

                handoff_zones.push(json!({
                    "HandoffZone标识": format!("handoff-zone-week3-{:03}", global_segment_index),
                    "RenderSegment标识": segment_id,
                    "起始边界": start_shot.to_string(),
                    "结束边界": end_shot.to_string(),
                    "边界类型": "render_segment_boundary"
                }));

                let first_cut_id = format!("cut-week3-{:03}", global_cut_index);
                stale_events.push(json!({
                    "StaleEvent标识": format!("stale-event-week3-{:03}", global_segment_index),
                    "来源层级": "render_segment",
                    "来源标识": segment_id,
                    "目标层级": "cut",
                    "目标标识": first_cut_id,
                    "追踪标识": format!("trace-{}-{}", segment_id, first_cut_id),
                    "时间戳": 1713513600 + global_segment_index as i64,
                    "原因": "render_segment updated"
                }));

                for (offset, cut_duration) in cut_durations.into_iter().enumerate() {
                    let cut_id = format!("cut-week3-{:03}", global_cut_index);
                    cuts.push(json!({
                        "Cut标识": cut_id,
                        "RenderSegment标识": segment_id,
                        "序号": global_cut_index,
                        "镜头描述": format!("{} 的第{}个 cut，保持中景推进。", segment_id, offset + 1),
                        "对白": format!("{} 第{}拍继续执行。", episode.title, global_cut_index),
                        "时长秒": cut_duration
                    }));
                    global_cut_index += 1;
                }

                shot_cursor += 3;
                global_segment_index += 1;
            }

            global_scene_index += 1;
        }
    }

    let fixture = json!({
        "project_meta": [
            {
                "项目标识": "project-week3-001",
                "标题": format!("Hope {}", benchmark_label),
                "状态": "冻结中",
                "目标时长分钟": project_duration_minutes,
                "更新时间戳": 1713513600
            }
        ],
        "episode_meta": episode_meta,
        "narrative_scene": narrative_scenes,
        "render_segment": render_segments,
        "cut": cuts,
        "prompt_package": prompt_packages,
        "handoff_zone": handoff_zones,
        "hard_lock": [
            {
                "HardLock标识": "hard-lock-week3-001",
                "项目标识": "project-week3-001",
                "锁名": "negative_prompt_guard",
                "锁值": "禁止出现血腥暴力词汇",
                "作用范围": "project"
            }
        ],
        "stale_event": stale_events,
        "export_manifest": [
            {
                "导出清单标识": "export-manifest-week3-001",
                "项目标识": "project-week3-001",
                "工作簿版本": format!("{}-fixture", benchmark_label.to_ascii_lowercase().replace(' ', "-")),
                "状态": "准备导出"
            }
        ],
        "director_profile": [
            {
                "导演档案标识": "director-profile-week3-001",
                "导演名称": "导演甲",
                "定位": "场景主导",
                "主风格": "克制写实",
                "镜头偏好": "中景与近景结合"
            }
        ],
        "director_cut_sample": [
            {
                "导演切样标识": "director-cut-sample-week3-001",
                "导演档案标识": "director-profile-week3-001",
                "样例标题": format!("{}切样", benchmark_label),
                "样例内容": format!("{}维持冷色调、中景、稳定切分。", benchmark_label),
                "代表性说明": "用于 E2E benchmark 的统一视觉与动作节奏参考。"
            }
        ],
        "committee_template": [
            {
                "模板标识": "committee-template-week3-001",
                "模板名称": "基础评审委员会",
                "适用场景": "常规叙事段落",
                "成员构成": "场景导演,动作导演",
                "职责描述": "统一镜头节奏、动作推进与提示词风格。"
            }
        ],
        "visual_term": [
            {
                "术语标识": "visual-term-week3-001",
                "中文术语": "冷色调",
                "类别": "色彩",
                "定义": "偏冷且低饱和的整体色彩基调。",
                "别名": "冷调"
            }
        ],
        "cinematography_term": [
            {
                "术语标识": "cinematography-term-week3-001",
                "中文术语": "中景",
                "类别": "景别",
                "定义": "突出人物与环境关系的中距离景别。",
                "别名": "中镜"
            }
        ],
        "continuity_rule": [
            {
                "规则标识": "continuity-rule-week3-001",
                "规则名称": "场景动作连续",
                "规则说明": "同一叙事场景内镜头衔接保持动作方向一致。",
                "适用层级": "NarrativeScene"
            }
        ]
    });

    fs::write(
        WEEK3_SHARED_FIXTURE_PATH,
        serde_json::to_string_pretty(&fixture).expect("fixture should serialize"),
    )
    .expect("shared fixture should be written");
}

pub fn split_cut_durations(total_seconds: u32) -> [u32; 3] {
    let base = total_seconds / 3;
    let remainder = total_seconds % 3;
    [
        base + u32::from(remainder > 0),
        base + u32::from(remainder > 1),
        base,
    ]
}

pub fn read_json(path: &str) -> Value {
    let text = fs::read_to_string(path).expect("json fixture should exist");
    serde_json::from_str(&text).expect("json fixture should parse")
}
