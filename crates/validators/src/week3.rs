#![forbid(unsafe_code)]

use std::{fs, io, path::Path};

use serde::Serialize;

use crate::{
    ContinuityInput, ContinuityValidator, HandoffCoverageInput, HandoffCoverageValidator,
    HardLockInjectorGuard, HardLockInjectorGuardInput, LayerTraceabilityInput,
    LayerTraceabilityValidator, NegativeGlobalGuard, NegativeGlobalGuardInput,
    RepairRecommendation, SegmentDurationInput, SegmentDurationValidator, StyleUnityInput,
    StyleUnityValidator, ValidationEnvelope, ValidationReport, ValidationReportRow,
    Week3SharedFixture, derive_repair_recommendations,
};
use core_domain::{FailurePatternRecord, PromptTemplateRecord};

pub const WEEK3_SHARED_FIXTURE_PATH: &str =
    r"E:\codex\hope\contracts\fixtures\week3-shared-fixture.json";
pub const WEEK3_VALIDATION_REPORT_PATH: &str =
    r"E:\codex\hope\contracts\fixtures\week3-validation-report.json";

#[derive(Debug, Clone, Serialize)]
pub struct Week3ValidationWorkbook {
    #[serde(rename = "validation_report")]
    pub validation_report: Vec<ValidationReportRow>,
}

pub fn load_week3_shared_fixture(path: impl AsRef<Path>) -> io::Result<Week3SharedFixture> {
    let json = fs::read_to_string(path)?;
    serde_json::from_str(&json).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid shared fixture: {error}"),
        )
    })
}

pub fn generate_week3_validation_report(
    fixture: &Week3SharedFixture,
) -> io::Result<Week3ValidationWorkbook> {
    let reports = collect_week3_validation_reports(fixture)?;

    Ok(Week3ValidationWorkbook {
        validation_report: reports
            .into_iter()
            .map(|report| report.to_sheet_row())
            .collect(),
    })
}

pub fn generate_week3_repair_recommendations(
    fixture: &Week3SharedFixture,
    failure_patterns: &[FailurePatternRecord],
    prompt_templates: &[PromptTemplateRecord],
) -> io::Result<Vec<RepairRecommendation>> {
    let reports = collect_week3_validation_reports(fixture)?;
    let mut recommendations = Vec::new();

    for report in &reports {
        recommendations.extend(derive_repair_recommendations(
            report,
            failure_patterns,
            prompt_templates,
        ));
    }

    recommendations.sort_by(|left, right| left.failure_code.cmp(&right.failure_code));
    recommendations.dedup_by(|left, right| left.failure_code == right.failure_code);

    Ok(recommendations)
}

pub fn collect_week3_validation_reports(
    fixture: &Week3SharedFixture,
) -> io::Result<Vec<ValidationReport>> {
    let project = fixture
        .project_meta
        .first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "project_meta is empty"))?;
    let director_profile = fixture
        .director_profile
        .first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "director_profile is empty"))?;
    let director_cut_sample = fixture.director_cut_sample.first().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "director_cut_sample is empty")
    })?;
    let committee_template = fixture
        .committee_template
        .first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "committee_template is empty"))?;

    let visual_terms: Vec<&str> = fixture
        .visual_term
        .iter()
        .map(|row| row.chinese_term.as_str())
        .collect();
    let cinematography_terms: Vec<&str> = fixture
        .cinematography_term
        .iter()
        .map(|row| row.chinese_term.as_str())
        .collect();
    let allowed_lock_names: Vec<&str> = fixture
        .hard_lock
        .iter()
        .map(|row| row.lock_name.as_str())
        .collect();
    let blocked_phrases: Vec<&str> = fixture
        .hard_lock
        .iter()
        .map(|row| row.lock_value.as_str())
        .collect();

    let mut validation_reports = Vec::new();

    for hard_lock in &fixture.hard_lock {
        validation_reports.push(HardLockInjectorGuard.validate(&HardLockInjectorGuardInput {
            envelope: ValidationEnvelope::new(
                format!("validation_report_{}_hard_lock", hard_lock.hard_lock_id),
                project.project_id.clone(),
                project.updated_at_timestamp,
            ),
            hard_lock_id: hard_lock.hard_lock_id.as_str(),
            lock_name: hard_lock.lock_name.as_str(),
            lock_value: hard_lock.lock_value.as_str(),
            scope: hard_lock.scope.as_str(),
            allowed_lock_names: &allowed_lock_names,
        }));
    }

    for prompt_package in &fixture.prompt_package {
        validation_reports.push(StyleUnityValidator.validate(&StyleUnityInput {
            envelope: ValidationEnvelope::new(
                format!(
                    "validation_report_{}_style",
                    prompt_package.prompt_package_id
                ),
                project.project_id.clone(),
                project.updated_at_timestamp,
            ),
            prompt_package_id: prompt_package.prompt_package_id.as_str(),
            director_profile_id: director_profile.director_profile_id.as_str(),
            director_cut_sample_id: director_cut_sample.director_cut_sample_id.as_str(),
            committee_template_id: committee_template.committee_template_id.as_str(),
            prompt_text: prompt_package.body.as_str(),
            visual_terms: &visual_terms,
            cinematography_terms: &cinematography_terms,
        }));
        validation_reports.push(NegativeGlobalGuard.validate(&NegativeGlobalGuardInput {
            envelope: ValidationEnvelope::new(
                format!(
                    "validation_report_{}_negative",
                    prompt_package.prompt_package_id
                ),
                project.project_id.clone(),
                project.updated_at_timestamp,
            ),
            prompt_package_id: prompt_package.prompt_package_id.as_str(),
            prompt_text: prompt_package.body.as_str(),
            blocked_phrases: &blocked_phrases,
        }));
    }

    for stale_event in &fixture.stale_event {
        let trace_cut = fixture
            .cut
            .iter()
            .find(|cut| cut.cut_id == stale_event.target_id);
        let trace_render_segment = fixture
            .render_segment
            .iter()
            .find(|segment| segment.render_segment_id == stale_event.source_id);

        validation_reports.push(ContinuityValidator.validate(&ContinuityInput {
            envelope: ValidationEnvelope::new(
                format!(
                    "validation_report_{}_continuity",
                    stale_event.stale_event_id
                ),
                project.project_id.clone(),
                stale_event.timestamp,
            ),
            stale_event_id: stale_event.stale_event_id.as_str(),
            source_level: stale_event.source_level.as_str(),
            source_id: stale_event.source_id.as_str(),
            target_level: stale_event.target_level.as_str(),
            target_id: stale_event.target_id.as_str(),
            trace_id: stale_event.trace_id.as_str(),
        }));
        validation_reports.push(
            LayerTraceabilityValidator.validate(&LayerTraceabilityInput {
                envelope: ValidationEnvelope::new(
                    format!("validation_report_{}_trace", stale_event.target_id),
                    project.project_id.clone(),
                    stale_event.timestamp,
                ),
                trace_id: stale_event.trace_id.as_str(),
                source_level: stale_event.source_level.as_str(),
                source_id: stale_event.source_id.as_str(),
                target_level: stale_event.target_level.as_str(),
                target_id: stale_event.target_id.as_str(),
                render_segment_id: trace_render_segment
                    .map(|render_segment| render_segment.render_segment_id.as_str()),
                cut_id: trace_cut.map(|cut| cut.cut_id.as_str()),
            }),
        );
    }

    for render_segment in &fixture.render_segment {
        validation_reports.push(SegmentDurationValidator.validate(&SegmentDurationInput {
            envelope: ValidationEnvelope::new(
                format!(
                    "validation_report_{}_duration",
                    render_segment.render_segment_id
                ),
                project.project_id.clone(),
                project.updated_at_timestamp,
            ),
            render_segment_id: render_segment.render_segment_id.as_str(),
            narrative_scene_id: render_segment.narrative_scene_id.as_str(),
            target_duration_seconds: render_segment.target_duration_seconds,
            actual_duration_seconds: Some(render_segment.actual_duration_seconds),
        }));
    }

    for handoff_zone in &fixture.handoff_zone {
        validation_reports.push(HandoffCoverageValidator.validate(&HandoffCoverageInput {
            envelope: ValidationEnvelope::new(
                format!("validation_report_{}_handoff", handoff_zone.handoff_zone_id),
                project.project_id.clone(),
                project.updated_at_timestamp,
            ),
            handoff_zone_id: handoff_zone.handoff_zone_id.as_str(),
            render_segment_id: handoff_zone.render_segment_id.as_str(),
            start_boundary: handoff_zone.start_boundary.as_str(),
            end_boundary: handoff_zone.end_boundary.as_str(),
            boundary_type: handoff_zone.boundary_type.as_str(),
        }));
    }

    Ok(validation_reports)
}

pub fn write_week3_validation_report(path: impl AsRef<Path>) -> io::Result<()> {
    let fixture = load_week3_shared_fixture(WEEK3_SHARED_FIXTURE_PATH)?;
    let workbook = generate_week3_validation_report(&fixture)?;
    let json = serde_json::to_string_pretty(&workbook).map_err(io::Error::other)?;
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::{
        WEEK3_SHARED_FIXTURE_PATH, generate_week3_repair_recommendations, load_week3_shared_fixture,
    };
    use core_domain::{FailurePatternRecord, PromptTemplateRecord};

    #[test]
    fn generate_week3_repair_recommendations_returns_kb_mapped_repairs() {
        let mut fixture = load_week3_shared_fixture(WEEK3_SHARED_FIXTURE_PATH)
            .expect("shared fixture should load");
        fixture.prompt_package[0].body =
            "TODO: 淇濇寔鍚屼竴涓讳綋浣嗘鏂囦粛鏄崰浣嶆枃鏈?.".to_string();
        let failure_patterns = vec![
            FailurePatternRecord {
                failure_pattern_id: "failure_07".to_string(),
                failure_code: "chinese_prompt_noise".to_string(),
                failure_name: "中文 Prompt 表达失真".to_string(),
                failure_category: "language".to_string(),
                symptom: "存在 placeholder 或过于机械化的文本".to_string(),
                common_causes: vec!["字段直拼".to_string()],
                detection_hint: "检查 PromptPackage 正文".to_string(),
                repair_strategy: "回到 PromptRenderer 分层后重写自然语言正文".to_string(),
                affected_layers: vec!["prompt_packages".to_string()],
                validator_hint: "Prompt quality review / Export sanity check".to_string(),
                repair_template_ids: vec!["prompt_09".to_string()],
                repair_priority: "medium".to_string(),
                repair_scope: "prompt_rendering_layer".to_string(),
                suggested_followup_validators: vec![
                    "Prompt quality review / Export sanity check".to_string(),
                ],
                source_type: "team_distillation".to_string(),
                source_notes: "test".to_string(),
                confidence_level: "high".to_string(),
                last_reviewed_at: "2026-04-20".to_string(),
            },
            FailurePatternRecord {
                failure_pattern_id: "failure_03".to_string(),
                failure_code: "continuity_break".to_string(),
                failure_name: "连续性断裂".to_string(),
                failure_category: "continuity".to_string(),
                symptom: "traceability / continuity 断裂".to_string(),
                common_causes: vec!["stale 链不完整".to_string()],
                detection_hint: "检查 Continuity Validator".to_string(),
                repair_strategy: "回补 continuity refs 和 trace chain".to_string(),
                affected_layers: vec!["storyboard_cuts".to_string()],
                validator_hint: "Continuity Validator".to_string(),
                repair_template_ids: vec!["prompt_16".to_string()],
                repair_priority: "high".to_string(),
                repair_scope: "continuity_fields_and_adjacent_cuts".to_string(),
                suggested_followup_validators: vec!["Continuity Validator".to_string()],
                source_type: "team_distillation".to_string(),
                source_notes: "test".to_string(),
                confidence_level: "high".to_string(),
                last_reviewed_at: "2026-04-20".to_string(),
            },
        ];
        let prompt_templates = vec![
            PromptTemplateRecord {
                prompt_template_id: "prompt_09".to_string(),
                stage: "repair_pass".to_string(),
                name: "结构修复回合".to_string(),
                target_model_family: "qwen-compatible".to_string(),
                repairs_failure_codes: vec!["chinese_prompt_noise".to_string()],
            },
            PromptTemplateRecord {
                prompt_template_id: "prompt_16".to_string(),
                stage: "repair_continuity".to_string(),
                name: "连续性修复模板".to_string(),
                target_model_family: "qwen-compatible".to_string(),
                repairs_failure_codes: vec!["continuity_break".to_string()],
            },
        ];

        let recommendations =
            generate_week3_repair_recommendations(&fixture, &failure_patterns, &prompt_templates)
                .expect("repair recommendations should generate");

        assert!(
            recommendations
                .iter()
                .any(|item| item.failure_code == "chinese_prompt_noise")
        );
    }
}
