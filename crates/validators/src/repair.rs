#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use core_domain::{FailurePatternRecord, PromptTemplateRecord, RepairTemplateLink};

use crate::ValidationReport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairRecommendation {
    pub failure_code: String,
    pub failure_name: String,
    pub repair_strategy: String,
    pub repair_priority: String,
    pub repair_scope: String,
    pub validator_hint: String,
    pub prompt_templates: Vec<RepairTemplateLink>,
}

pub fn derive_repair_recommendations(
    report: &ValidationReport,
    failure_patterns: &[FailurePatternRecord],
    prompt_templates: &[PromptTemplateRecord],
) -> Vec<RepairRecommendation> {
    let mut recommendation_index: BTreeMap<String, RepairRecommendation> = BTreeMap::new();

    for finding in &report.findings {
        for failure_code in failure_codes_for_finding(finding.code) {
            let Some(pattern) = failure_patterns
                .iter()
                .find(|candidate| candidate.failure_code == *failure_code)
            else {
                continue;
            };

            recommendation_index
                .entry(pattern.failure_code.clone())
                .or_insert_with(|| RepairRecommendation {
                    failure_code: pattern.failure_code.clone(),
                    failure_name: pattern.failure_name.clone(),
                    repair_strategy: pattern.repair_strategy.clone(),
                    repair_priority: pattern.repair_priority.clone(),
                    repair_scope: pattern.repair_scope.clone(),
                    validator_hint: pattern.validator_hint.clone(),
                    prompt_templates: bind_repair_links(pattern, prompt_templates),
                });
        }
    }

    recommendation_index.into_values().collect()
}

fn bind_repair_links(
    failure_pattern: &FailurePatternRecord,
    prompt_templates: &[PromptTemplateRecord],
) -> Vec<RepairTemplateLink> {
    failure_pattern
        .repair_template_ids
        .iter()
        .map(|template_id| {
            let template = prompt_templates
                .iter()
                .find(|candidate| candidate.prompt_template_id == *template_id);

            RepairTemplateLink {
                failure_code: failure_pattern.failure_code.clone(),
                prompt_template_id: template_id.clone(),
                prompt_stage: template.map(|item| item.stage.clone()),
                prompt_name: template.map(|item| item.name.clone()),
            }
        })
        .collect()
}

fn failure_codes_for_finding(code: &str) -> &'static [&'static str] {
    match code {
        "style_unity_not_proven"
        | "one_family_only"
        | "frozen_term_set_missing"
        | "anchor_id_missing" => &["style_drift", "character_inconsistency"],
        "scope_not_project"
        | "lock_name_not_frozen"
        | "frozen_lock_set_missing"
        | "hard_lock_id_missing"
        | "lock_name_missing"
        | "lock_value_missing" => &["hard_lock_loss"],
        "stale_chain_incomplete"
        | "self_targeted_stale_event"
        | "trace_chain_incomplete"
        | "partial_entity_anchor"
        | "no_entity_anchor" => &["continuity_break"],
        "target_duration_out_of_bounds" | "actual_duration_out_of_bounds" => {
            &["segment_cross_scene"]
        }
        "handoff_boundary_missing" | "handoff_boundary_collapsed" | "handoff_identity_missing" => {
            &["handoff_gap"]
        }
        "blocked_phrase_hit" | "prompt_text_missing" | "placeholder_marker_detected" => {
            &["chinese_prompt_noise"]
        }
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use core_domain::{FailurePatternRecord, PromptTemplateRecord};

    use crate::{ValidationEnvelope, ValidationFinding, ValidationReport};

    use super::derive_repair_recommendations;

    #[test]
    fn derive_repair_recommendations_maps_validator_codes_to_kb_failures() {
        let report = ValidationReport::new(
            ValidationEnvelope::new("validation_report_test", "project_01", 1_714_000_000),
            vec![
                ValidationFinding::block(
                    "style_unity_not_proven",
                    "prompt_package",
                    "style drift found",
                    "prompt text missing anchors",
                ),
                ValidationFinding::block(
                    "handoff_boundary_missing",
                    "handoff_zone",
                    "handoff gap found",
                    "boundary missing",
                ),
            ],
        );
        let failure_patterns = vec![
            FailurePatternRecord {
                failure_pattern_id: "failure_01".to_string(),
                failure_code: "style_drift".to_string(),
                failure_name: "风格漂移".to_string(),
                failure_category: "style".to_string(),
                symptom: "相邻 cut 风格不统一".to_string(),
                common_causes: vec!["局部导演覆盖过强".to_string()],
                detection_hint: "检查 style unity".to_string(),
                repair_strategy: "重新注入 hard locks".to_string(),
                affected_layers: vec!["prompt_packages".to_string()],
                validator_hint: "Style Unity Validator".to_string(),
                repair_template_ids: vec!["prompt_09".to_string()],
                repair_priority: "high".to_string(),
                repair_scope: "render_prompt_only".to_string(),
                suggested_followup_validators: vec!["Style Unity Validator".to_string()],
                source_type: "team_distillation".to_string(),
                source_notes: "test".to_string(),
                confidence_level: "high".to_string(),
                last_reviewed_at: "2026-04-20".to_string(),
            },
            FailurePatternRecord {
                failure_pattern_id: "failure_02".to_string(),
                failure_code: "handoff_gap".to_string(),
                failure_name: "导演交接缺口".to_string(),
                failure_category: "handoff".to_string(),
                symptom: "没有 handoff zone".to_string(),
                common_causes: vec!["handoff 规则缺失".to_string()],
                detection_hint: "检查 Handoff Coverage".to_string(),
                repair_strategy: "补 buffer cut".to_string(),
                affected_layers: vec!["project_handoff_zones".to_string()],
                validator_hint: "Handoff Coverage".to_string(),
                repair_template_ids: vec!["prompt_14".to_string()],
                repair_priority: "high".to_string(),
                repair_scope: "handoff_zone_and_buffer_cuts".to_string(),
                suggested_followup_validators: vec!["Handoff Coverage".to_string()],
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
                repairs_failure_codes: vec!["style_drift".to_string()],
            },
            PromptTemplateRecord {
                prompt_template_id: "prompt_14".to_string(),
                stage: "transition_director_variant".to_string(),
                name: "交接过渡模板".to_string(),
                target_model_family: "qwen-compatible".to_string(),
                repairs_failure_codes: vec!["handoff_gap".to_string()],
            },
        ];

        let recommendations =
            derive_repair_recommendations(&report, &failure_patterns, &prompt_templates);

        assert_eq!(recommendations.len(), 2);
        assert_eq!(recommendations[0].failure_code, "handoff_gap");
        assert_eq!(recommendations[1].failure_code, "style_drift");
        assert_eq!(
            recommendations[0].prompt_templates[0]
                .prompt_name
                .as_deref(),
            Some("交接过渡模板")
        );
        assert_eq!(
            recommendations[1].prompt_templates[0]
                .prompt_stage
                .as_deref(),
            Some("repair_pass")
        );
    }

    #[test]
    fn derive_repair_recommendations_deduplicates_same_failure_code() {
        let report = ValidationReport::new(
            ValidationEnvelope::new("validation_report_test", "project_01", 1_714_000_000),
            vec![
                ValidationFinding::block(
                    "lock_name_missing",
                    "hard_lock",
                    "lock name missing",
                    "lock_name = \"\"",
                ),
                ValidationFinding::block(
                    "lock_value_missing",
                    "hard_lock",
                    "lock value missing",
                    "lock_value = \"\"",
                ),
            ],
        );
        let failure_patterns = vec![FailurePatternRecord {
            failure_pattern_id: "failure_05".to_string(),
            failure_code: "hard_lock_loss".to_string(),
            failure_name: "Hard Locks 丢失".to_string(),
            failure_category: "style".to_string(),
            symptom: "hard locks 缺失".to_string(),
            common_causes: vec!["PromptRenderer 未强制追加".to_string()],
            detection_hint: "检查 Hard Lock Injector Guard".to_string(),
            repair_strategy: "只修 hard_locks".to_string(),
            affected_layers: vec!["prompt_packages".to_string()],
            validator_hint: "Hard Lock Injector Guard".to_string(),
            repair_template_ids: vec!["prompt_15".to_string()],
            repair_priority: "critical".to_string(),
            repair_scope: "render_prompt_only".to_string(),
            suggested_followup_validators: vec!["Hard Lock Injector Guard".to_string()],
            source_type: "team_distillation".to_string(),
            source_notes: "test".to_string(),
            confidence_level: "high".to_string(),
            last_reviewed_at: "2026-04-20".to_string(),
        }];
        let prompt_templates = vec![PromptTemplateRecord {
            prompt_template_id: "prompt_15".to_string(),
            stage: "repair_hardlocks".to_string(),
            name: "Hard Locks 修复模板".to_string(),
            target_model_family: "qwen-compatible".to_string(),
            repairs_failure_codes: vec!["hard_lock_loss".to_string()],
        }];

        let recommendations =
            derive_repair_recommendations(&report, &failure_patterns, &prompt_templates);

        assert_eq!(recommendations.len(), 1);
        assert_eq!(recommendations[0].failure_code, "hard_lock_loss");
    }
}
