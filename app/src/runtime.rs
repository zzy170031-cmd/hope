use std::hash::{Hash, Hasher};
use std::io;

use crate::state::AppState;

use core_domain::{
    BridgeCallStatus, ExpandScriptRequest, ExpandScriptResponse, ExportArtifactRecord,
    ExportBundleRequest, ExportBundleResponse, ExternalReferenceHandleCandidate,
    GenerateStoryboardRequest, GenerateStoryboardResponse, GeneratedStoryboardRow,
    GoldenSampleLibraryRecord, ProductWarning, PromptBodyCandidate, ScenePerformanceProjection,
    SequenceFieldState, SequenceGrouping, StoryboardDurationPlan, StoryboardExportStatus,
    StructureMode,
};
use storyboard_pipeline::{StoryboardPlan, StoryboardPlanRequest, StoryboardPlanningError};
use validators::{
    generate_week3_repair_recommendations, generate_week3_validation_report,
    load_week3_shared_fixture, project_v120_evidence_aware_findings, RepairRecommendation,
    Week3SharedFixture, WEEK3_SHARED_FIXTURE_PATH,
};

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
pub struct ValidationExportPanelSnapshotRequest {
    pub project_id: String,
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

pub fn expand_script(state: &AppState, request: ExpandScriptRequest) -> ExpandScriptResponse {
    let script_hash = stable_hash_hex(&format!(
        "{}\n{}",
        request.scene_type.trim(),
        request.synopsis_text.trim()
    ));
    let script_id = format!("script-{}", &script_hash[..12]);

    let mut warnings = vec![ProductWarning {
        code: "qwen_live_generation_closed".to_string(),
        message:
            "Expanded script is a deterministic bridge envelope; live Qwen expansion remains gated."
                .to_string(),
        related_sample_id: None,
    }];
    if !state
        .kb_golden_sample_runtime
        .manifest
        .seed_import_format
        .contains("v0.2")
    {
        warnings.push(ProductWarning {
            code: "v120_package_not_confirmed".to_string(),
            message: "Loaded KB package does not advertise the accepted v0.2 seed import format."
                .to_string(),
            related_sample_id: None,
        });
    }

    let response = ExpandScriptResponse {
        script_id,
        expanded_script_text: format!(
            "scene_type: {}\nsynopsis: {}\nsource_package: {}",
            request.scene_type.trim(),
            request.synopsis_text.trim(),
            state.kb_golden_sample_runtime.manifest.snapshot_name
        ),
        script_hash,
        warnings,
    };
    state.remember_script(response.clone());
    response
}

pub fn generate_storyboard(
    state: &AppState,
    request: GenerateStoryboardRequest,
) -> GenerateStoryboardResponse {
    let script_text = request
        .expanded_script_text
        .clone()
        .or_else(|| {
            request
                .script_id
                .as_deref()
                .and_then(|script_id| state.find_script(script_id))
                .map(|script| script.expanded_script_text)
        })
        .unwrap_or_default();

    let selected_records = select_golden_sample_records(state, &script_text, &request);
    let row_count = selected_records.len().max(1);
    let per_row_seconds = (request.selected_total_duration_seconds / row_count as u16).max(1);
    let mut rows = Vec::new();
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();

    for (index, record) in selected_records.iter().enumerate() {
        let failure_mapping = state
            .kb_golden_sample_runtime
            .failure_mapping
            .records
            .iter()
            .find(|mapping| mapping.sample_id == record.sample_id);
        let evidence = project_v120_evidence_aware_findings(record, failure_mapping);
        blockers.extend(
            evidence
                .blockers
                .into_iter()
                .map(product_warning_from_evidence),
        );
        warnings.extend(
            evidence
                .warnings
                .into_iter()
                .map(product_warning_from_evidence),
        );

        let prompt_candidate = project_prompt_body_candidate(record);
        let scene_projection = project_scene_performance(record);
        if prompt_candidate.candidate_text.is_some() {
            warnings.push(ProductWarning {
                code: "prompt_body_candidate_not_compiled".to_string(),
                message: "V120 prompt_body is held as candidate evidence and is not compiled into runtime prompt_text.".to_string(),
                related_sample_id: Some(record.sample_id.clone()),
            });
        }

        rows.push(GeneratedStoryboardRow {
            shot_id: record.source_fields.shot_id.clone(),
            order: (index + 1) as u32,
            person: scene_projection.person.clone(),
            shot_title: record.source_fields.sample_title.clone(),
            scene_scale: scene_projection.scene_scale.clone(),
            visual_description: scene_projection.visual_description.clone(),
            character_action: scene_projection.character_action.clone(),
            dialogue: String::new(),
            prompt_text: String::new(),
            duration_seconds: per_row_seconds,
            prompt_body_candidate: prompt_candidate,
            scene_performance_projection: scene_projection.clone(),
            external_reference_handle_candidates: project_reference_handle_candidates(record),
            sequence_grouping: scene_projection.sequence_grouping,
        });
    }

    if rows.is_empty() {
        blockers.push(ProductWarning {
            code: "v120_no_selectable_rows".to_string(),
            message: "No V120 rows matched the bridge selector.".to_string(),
            related_sample_id: None,
        });
    }
    if script_text.trim().is_empty() {
        blockers.push(ProductWarning {
            code: "script_input_missing".to_string(),
            message: "generate_storyboard requires script_id or expanded_script_text.".to_string(),
            related_sample_id: None,
        });
    }

    warnings.push(ProductWarning {
        code: "model_generation_closed".to_string(),
        message:
            "Rows are V120 bridge projections; live model-generated storyboard text remains gated."
                .to_string(),
        related_sample_id: None,
    });

    let status = if !blockers.is_empty() {
        BridgeCallStatus::Blocked
    } else if !warnings.is_empty() {
        BridgeCallStatus::WarningOnly
    } else {
        BridgeCallStatus::Ready
    };
    let blocked_row_count = blockers
        .iter()
        .filter_map(|item| item.related_sample_id.as_deref())
        .collect::<std::collections::BTreeSet<_>>()
        .len() as u32;
    let ready_row_count = rows.len() as u32 - blocked_row_count.min(rows.len() as u32);
    let allocated_seconds = rows.iter().map(|row| row.duration_seconds).sum::<u16>();
    let result_id = format!(
        "storyboard-{}",
        &stable_hash_hex(&format!(
            "{}\n{}\n{}",
            request.task_name, script_text, request.selected_total_duration_seconds
        ))[..12]
    );

    let response = GenerateStoryboardResponse {
        result_id,
        rows,
        duration_plan: StoryboardDurationPlan {
            total_duration_seconds: request.selected_total_duration_seconds,
            row_count: row_count as u32,
            per_row_seconds,
            allocated_seconds,
        },
        export_status: StoryboardExportStatus {
            status,
            blockers,
            warnings,
            ready_row_count,
            blocked_row_count,
        },
    };
    state.remember_storyboard(response.clone());
    response
}

pub fn export_bundle(state: &AppState, request: ExportBundleRequest) -> ExportBundleResponse {
    let export_manifest_id = format!(
        "export-manifest-{}",
        &stable_hash_hex(&format!("{}\n{}", request.result_id, request.export_format))[..12]
    );
    let storyboard = state.find_storyboard(&request.result_id);
    let mut artifacts = vec![ExportArtifactRecord {
        artifact_id: format!("{}-bridge-manifest", export_manifest_id),
        artifact_kind: "v120_bridge_manifest".to_string(),
        export_format: request.export_format.clone(),
        ready: storyboard.is_some(),
        blocked_reason: storyboard
            .is_none()
            .then(|| "storyboard_result_not_found".to_string()),
    }];

    artifacts.push(ExportArtifactRecord {
        artifact_id: format!("{}-workbook", export_manifest_id),
        artifact_kind: "excel_workbook".to_string(),
        export_format: request.export_format,
        ready: false,
        blocked_reason: Some("excel_exporter_gate_closed".to_string()),
    });

    ExportBundleResponse {
        export_manifest_id,
        artifacts,
    }
}

fn select_golden_sample_records<'a>(
    state: &'a AppState,
    script_text: &str,
    request: &GenerateStoryboardRequest,
) -> Vec<&'a GoldenSampleLibraryRecord> {
    let query = format!(
        "{} {} {}",
        request.task_name,
        script_text,
        request.expanded_script_text.as_deref().unwrap_or_default()
    )
    .to_lowercase();
    let target_rows = ((request.selected_total_duration_seconds / 8).max(1) as usize).clamp(1, 8);
    let records = &state.kb_golden_sample_runtime.golden_sample_library.records;

    let mut selected = records
        .iter()
        .filter(|record| record.classification.library_status == "official")
        .filter(|record| {
            let fields = &record.source_fields;
            query.contains(&fields.scene_category.to_lowercase())
                || query.contains(&fields.scene_tag.to_lowercase())
                || query.contains(&fields.style_cluster.to_lowercase())
        })
        .take(target_rows)
        .collect::<Vec<_>>();

    if selected.is_empty() {
        selected = records
            .iter()
            .filter(|record| record.classification.library_status == "official")
            .take(target_rows)
            .collect();
    }

    selected
}

fn product_warning_from_evidence(item: validators::EvidenceAwareValidationItem) -> ProductWarning {
    ProductWarning {
        code: item.code,
        message: item.message,
        related_sample_id: item.source_sample_id,
    }
}

fn project_prompt_body_candidate(record: &GoldenSampleLibraryRecord) -> PromptBodyCandidate {
    let blocked = validators::contains_placeholder_marker(&record.source_fields.prompt_body)
        || record.validator_evidence.has_placeholder_signal;
    PromptBodyCandidate {
        source_sample_id: record.sample_id.clone(),
        source_prompt_body: record.source_fields.prompt_body.clone(),
        candidate_text: (!blocked).then(|| record.source_fields.prompt_body.clone()),
        blocked,
        blocker_codes: blocked
            .then(|| vec!["prompt_body_blocked_by_placeholder".to_string()])
            .unwrap_or_default(),
    }
}

fn project_scene_performance(record: &GoldenSampleLibraryRecord) -> ScenePerformanceProjection {
    ScenePerformanceProjection {
        source_sample_id: record.sample_id.clone(),
        source_sample_title: record.source_fields.sample_title.clone(),
        scene_scale: derive_scene_scale(&record.source_fields.technical_profile),
        person: "not_specified_by_v120_bridge".to_string(),
        visual_description: record.source_fields.scene_performance_core.clone(),
        character_action: "fused_scene_performance_core_preserved".to_string(),
        fused_source_text: record.source_fields.scene_performance_core.clone(),
        sequence_grouping: project_sequence_grouping(record),
    }
}

fn project_sequence_grouping(record: &GoldenSampleLibraryRecord) -> SequenceGrouping {
    let structure_mode = StructureMode::from_sample_type(&record.source_fields.sample_type);
    match structure_mode {
        StructureMode::SingleShot => SequenceGrouping {
            structure_mode,
            sequence_id: None,
            shot_order: None,
            sequence_field_state: SequenceFieldState::NotApplicable,
        },
        StructureMode::SequenceShot => {
            let sequence_id = non_blank_string(&record.source_fields.sequence_id);
            let shot_order = record.source_fields.shot_order.trim().parse::<u32>().ok();
            SequenceGrouping {
                structure_mode,
                sequence_id,
                shot_order,
                sequence_field_state: if shot_order.is_some() {
                    SequenceFieldState::Present
                } else {
                    SequenceFieldState::Missing
                },
            }
        }
    }
}

fn project_reference_handle_candidates(
    record: &GoldenSampleLibraryRecord,
) -> Vec<ExternalReferenceHandleCandidate> {
    extract_reference_tokens(&record.source_fields.reference_bundle)
        .into_iter()
        .map(
            |(reference_name, reference_kind, strength)| ExternalReferenceHandleCandidate {
                source_sample_id: record.sample_id.clone(),
                reference_name,
                reference_kind,
                strength,
            },
        )
        .collect()
}

fn extract_reference_tokens(value: &str) -> Vec<(String, String, Option<String>)> {
    let chars = value.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    for (index, character) in chars.iter().enumerate() {
        if *character != '(' {
            continue;
        }
        let mut start = index;
        while start > 0 {
            let previous = chars[start - 1];
            if previous.is_ascii_alphanumeric() || previous == '_' || previous == '-' {
                start -= 1;
            } else {
                break;
            }
        }
        if start == index {
            continue;
        }
        let reference_name = chars[start..index].iter().collect::<String>();
        if looks_like_path_or_url(&reference_name) {
            continue;
        }
        let Some(end_offset) = chars[index + 1..].iter().position(|item| *item == ')') else {
            continue;
        };
        let args = chars[index + 1..index + 1 + end_offset]
            .iter()
            .collect::<String>();
        let mut parts = args.split(',').map(str::trim);
        let reference_kind = parts.next().unwrap_or("name").to_string();
        let lower_reference_kind = reference_kind.to_ascii_lowercase();
        if matches!(
            lower_reference_kind.as_str(),
            "path" | "url" | "uri" | "media" | "asset" | "file"
        ) {
            continue;
        }
        let strength = parts
            .next()
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        tokens.push((reference_name, reference_kind, strength));
    }
    tokens
}

fn looks_like_path_or_url(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("http")
        || lower.contains("://")
        || lower.contains('\\')
        || lower.contains('/')
        || lower.contains(':')
}

fn derive_scene_scale(technical_profile: &str) -> String {
    for token in ["MCU", "CU", "LS", "MS", "WS"] {
        if technical_profile.contains(token) {
            return token.to_string();
        }
    }
    "source_technical_profile".to_string()
}

fn non_blank_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn stable_hash_hex<T: Hash>(value: &T) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn build_storyboard_preview_plan(
    state: &AppState,
    request: StoryboardPreviewPlanRequest,
) -> Result<StoryboardPlan, StoryboardPlanningError> {
    let scene_taxonomy = resolve_scene_taxonomy(state, request.scene_type.as_deref());

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
        scene_director_id: request.scene_director_id,
        action_director_id: request.action_director_id,
        scene_taxonomy,
        layout_prompt: request.layout_prompt,
        render_prompt: request.render_prompt,
    })
}

pub fn build_validation_export_panel_snapshot(
    state: &AppState,
    request: ValidationExportPanelSnapshotRequest,
) -> io::Result<ValidationExportPanelSnapshot> {
    let fixture = load_week3_shared_fixture(WEEK3_SHARED_FIXTURE_PATH)?;
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
    use std::collections::BTreeMap;

    use core_domain::{
        BridgeCallStatus, ExpandScriptRequest, ExportBundleRequest, FailurePatternRecord,
        GenerateStoryboardRequest, GoldenSampleAssetSource, GoldenSampleClassification,
        GoldenSampleComparisonBaseline, GoldenSampleCoverageSummary,
        GoldenSampleFailureCodeDefinition, GoldenSampleFailureMappingAsset,
        GoldenSampleFailureMappingRecord, GoldenSampleFailureValidatorEvidence,
        GoldenSampleFewshotGate, GoldenSampleFewshotState, GoldenSampleFieldCoverageRuleAsset,
        GoldenSampleFieldCoverageRuleRecord, GoldenSampleLibraryAsset,
        GoldenSampleLibraryProvenance, GoldenSampleLibraryRecord, GoldenSampleNegativeSample,
        GoldenSampleNegativeSampleGate, GoldenSampleRepairInputs, GoldenSampleRepairMappingAsset,
        GoldenSampleRepairMappingPlanning, GoldenSampleRepairMappingRecord,
        GoldenSampleSourceContext, GoldenSampleSourceFields, GoldenSampleSourceRegister,
        GoldenSampleSourceRegisterEntry, GoldenSampleSourceRegisterProvenance,
        GoldenSampleV3CoreCoverage, GoldenSampleValidatorEvidence, KbBundleManifestRecord,
        KbBundleRecordCounts, KbGoldenSampleRuntimePackage, KbRuntimeSummary, KbSnapshotRecord,
        PromptTemplateRecord, SceneTaxonomyRecord,
    };
    use project_store::{
        DualSqliteConnectionPolicy, KbKnowledgeBundle, KbRuntimeHandle, StoreSkeleton,
    };

    use super::{
        build_storyboard_preview_plan, build_validation_export_panel_snapshot_from_fixture,
        expand_script, export_bundle, generate_storyboard, resolve_scene_taxonomy,
        StoryboardPreviewPlanRequest, ValidationExportPanelSnapshotRequest,
        ValidationExportPanelState,
    };
    use crate::state::AppState;
    use validators::{load_week3_shared_fixture, WEEK3_SHARED_FIXTURE_PATH};

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
                has_golden_sample_v120_package: true,
                golden_sample_record_count: 1,
                golden_sample_source_count: 1,
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

        AppState::new(
            store,
            kb_runtime,
            kb_knowledge,
            test_golden_sample_package(),
        )
    }

    fn test_golden_sample_package() -> KbGoldenSampleRuntimePackage {
        let record = GoldenSampleLibraryRecord {
            machine_id: "golden_sample_bridge_01".to_string(),
            sample_id: "GS-BRIDGE-01".to_string(),
            schema_version: "golden_sample_library.v0.2".to_string(),
            source_fields: GoldenSampleSourceFields {
                shot_id: "GS-BRIDGE-01".to_string(),
                library_status: "official".to_string(),
                reserve_reason: String::new(),
                sample_type: "single_shot".to_string(),
                sequence_id: String::new(),
                shot_order: String::new(),
                sample_title: "Bridge dialogue sample".to_string(),
                style_cluster: "dialogue".to_string(),
                scene_category: "daily_dialogue".to_string(),
                scene_tag: "daily_dialogue,interior".to_string(),
                quality_grade: "good".to_string(),
                usable_for_fewshot: "Yes".to_string(),
                technical_profile: "duration 8s / MCU / 24fps".to_string(),
                scene_performance_core: "Two people hold a restrained dialogue beat.".to_string(),
                camera_directing_core: "Hold a stable medium close composition.".to_string(),
                audio_directing_core: "Room tone and breath stay low.".to_string(),
                continuity_negative_core: "Do not drift eyeline or prop handoff.".to_string(),
                reference_bundle: "scene_room(layout,strong) char_lead(face,reference)".to_string(),
                ip_abstraction_note: "abstracted".to_string(),
                covered_points: "dialogue / eyeline".to_string(),
                missed_points: String::new(),
                teaching_note: "Use as bridge evidence only.".to_string(),
                prompt_body: "Compose a restrained dialogue shot with stable eyeline.".to_string(),
            },
            provenance: GoldenSampleLibraryProvenance {
                source_workbook: "workbook.xlsx".to_string(),
                source_workbook_sha256: "hash".to_string(),
                source_control_memo: "memo.docx".to_string(),
                source_control_memo_sha256: "hash".to_string(),
                source_row_index: 1,
                source_library_status: "official".to_string(),
                source_sample_type: "single_shot".to_string(),
                comparison_baseline_source_id: "v108".to_string(),
                control_review_doc: "review.md".to_string(),
                control_dispatch_doc: "dispatch.md".to_string(),
            },
            classification: GoldenSampleClassification {
                core: "full_stack_single_shot_sample".to_string(),
                library_status: "official".to_string(),
                sample_type: "single_shot".to_string(),
                sequence_id: String::new(),
                shot_order: String::new(),
                style_cluster: "dialogue".to_string(),
                scene_category: "daily_dialogue".to_string(),
                scene_tags: vec!["daily_dialogue".to_string()],
                quality_grade: "good".to_string(),
                usable_for_fewshot: true,
                coverage_surfaces: vec![
                    "scene_performance_core".to_string(),
                    "continuity_negative_core".to_string(),
                ],
            },
            fewshot: GoldenSampleFewshotState {
                eligible: true,
                source_value: "Yes".to_string(),
                retrieval_status: "candidate_positive_fewshot".to_string(),
            },
            validator_evidence: GoldenSampleValidatorEvidence {
                covered_points: "dialogue / eyeline".to_string(),
                missed_points: String::new(),
                teaching_note: "Use as bridge evidence only.".to_string(),
                source_quality_grade: "good".to_string(),
                source_library_status: "official".to_string(),
                source_sample_type: "single_shot".to_string(),
                has_coverage_gap: false,
                has_placeholder_signal: false,
                surface_completeness: BTreeMap::from([
                    ("scene_performance_core".to_string(), true),
                    ("continuity_negative_core".to_string(), true),
                ]),
                reference_bundle_present: true,
            },
            negative_sample: GoldenSampleNegativeSample {
                is_negative_sample: false,
                signal_codes: vec![],
                reserve_reason: String::new(),
            },
            v3_core_coverage: GoldenSampleV3CoreCoverage {
                core: "full_stack_single_shot_sample".to_string(),
                coverage_rule_ids: vec!["gs_field_rule_scene_performance_core".to_string()],
                source_coverage_statement: "covered".to_string(),
                source_missing_statement: String::new(),
                source_field_presence: BTreeMap::from([
                    ("scene_performance_core".to_string(), true),
                    ("continuity_negative_core".to_string(), true),
                ]),
                comparison_baseline: GoldenSampleComparisonBaseline {
                    primary_source_id: "v120".to_string(),
                    comparison_source_id: "v108".to_string(),
                },
            },
            repair_mapping_planning: GoldenSampleRepairMappingPlanning {
                failure_mapping_id: "failure_map_01".to_string(),
                repair_mapping_id: "repair_map_01".to_string(),
                planning_only: true,
            },
        };

        KbGoldenSampleRuntimePackage {
            manifest: KbBundleManifestRecord {
                snapshot_version: "v0.2".to_string(),
                snapshot_name: "hope-kb-golden-sample-library-v0.2".to_string(),
                content_hash_algo: "sha256".to_string(),
                content_hash: "bundle-sha256:test".to_string(),
                seed_import_format: "hope-kb-golden-sample-seed-bundle-v0.2".to_string(),
                imported_at: "2026-04-23".to_string(),
                primary_key: "machine_id".to_string(),
                bundle_order: vec!["seed/v0.2/golden_sample_library.json".to_string()],
                record_counts: KbBundleRecordCounts {
                    golden_sample_library: 1,
                    golden_sample_field_coverage_rules: 1,
                    golden_sample_failure_mapping: 1,
                    golden_sample_repair_mapping: 1,
                    golden_sample_sources: 1,
                    golden_sample_provenance_entries: 1,
                },
            },
            golden_sample_library: GoldenSampleLibraryAsset {
                schema_version: "golden_sample_library.v0.2".to_string(),
                asset_name: "golden_sample_library".to_string(),
                generated_at: "2026-04-23".to_string(),
                source: GoldenSampleAssetSource {
                    primary_workbook: "workbook.xlsx".to_string(),
                    primary_workbook_sha256: "hash".to_string(),
                    control_memo: "memo.docx".to_string(),
                    control_memo_sha256: "hash".to_string(),
                    control_review: "review.md".to_string(),
                    control_dispatch: "dispatch.md".to_string(),
                    comparison_baseline_source_id: "v108".to_string(),
                },
                record_count: 1,
                source_field_order: vec!["shot_id".to_string()],
                records: vec![record],
            },
            field_coverage_rules: GoldenSampleFieldCoverageRuleAsset {
                schema_version: "golden_sample_field_coverage_rules.v0.2".to_string(),
                asset_name: "golden_sample_field_coverage_rules".to_string(),
                generated_at: "2026-04-23".to_string(),
                source_context: GoldenSampleSourceContext {
                    primary_source_id: "v120".to_string(),
                    comparison_source_id: "v108".to_string(),
                },
                record_count: 1,
                records: vec![GoldenSampleFieldCoverageRuleRecord {
                    rule_id: "gs_field_rule_scene_performance_core".to_string(),
                    core: "scene_performance_core".to_string(),
                    row_count: 1,
                    source_sample_ids: vec!["GS-BRIDGE-01".to_string()],
                    required_source_fields: vec!["scene_performance_core".to_string()],
                    validator_evidence_fields: vec!["covered_points".to_string()],
                    fewshot_gate: GoldenSampleFewshotGate {
                        eligible_source_field: "usable_for_fewshot".to_string(),
                        positive_value: "Yes".to_string(),
                        negative_value: "No".to_string(),
                        library_status_field: "library_status".to_string(),
                        official_value: "official".to_string(),
                        reserve_value: "reserve".to_string(),
                        reserve_rows_excluded_from_positive_fewshot: true,
                        negative_quality_value: "bad".to_string(),
                    },
                    negative_sample_gate: GoldenSampleNegativeSampleGate {
                        quality_field: "quality_grade".to_string(),
                        negative_quality_value: "bad".to_string(),
                        library_status_field: "library_status".to_string(),
                        reserve_value: "reserve".to_string(),
                        reserve_reason_field: "reserve_reason".to_string(),
                    },
                    coverage_summary: GoldenSampleCoverageSummary {
                        library_status: BTreeMap::from([("official".to_string(), 1)]),
                        quality_grade: BTreeMap::from([("good".to_string(), 1)]),
                        sample_type: BTreeMap::from([("single_shot".to_string(), 1)]),
                        scene_category: BTreeMap::from([("daily_dialogue".to_string(), 1)]),
                        sequence_groups: BTreeMap::new(),
                        style_cluster: BTreeMap::from([("dialogue".to_string(), 1)]),
                        surface_completeness: BTreeMap::from([(
                            "scene_performance_core".to_string(),
                            1,
                        )]),
                        usable_for_fewshot: BTreeMap::from([("Yes".to_string(), 1)]),
                    },
                    future_gate_owner: vec!["Hope runtime ingest planning".to_string()],
                    planning_only: true,
                }],
            },
            failure_mapping: GoldenSampleFailureMappingAsset {
                schema_version: "golden_sample_failure_mapping.v0.2".to_string(),
                asset_name: "golden_sample_failure_mapping".to_string(),
                generated_at: "2026-04-23".to_string(),
                source_context: GoldenSampleSourceContext {
                    primary_source_id: "v120".to_string(),
                    comparison_source_id: "v108".to_string(),
                },
                failure_code_definitions: vec![GoldenSampleFailureCodeDefinition {
                    failure_code: "golden_coverage_gap".to_string(),
                    source_fields: vec!["missed_points".to_string()],
                    meaning: "coverage gap".to_string(),
                }],
                record_count: 1,
                records: vec![GoldenSampleFailureMappingRecord {
                    mapping_id: "failure_map_01".to_string(),
                    sample_id: "GS-BRIDGE-01".to_string(),
                    core: "full_stack_single_shot_sample".to_string(),
                    tier: "good".to_string(),
                    usable_for_fewshot: "Yes".to_string(),
                    negative_sample_signal: false,
                    planned_failure_codes: vec![],
                    validator_evidence: GoldenSampleFailureValidatorEvidence {
                        covered_points: "dialogue".to_string(),
                        missed_points: String::new(),
                        teaching_note: "bridge evidence".to_string(),
                        library_status: "official".to_string(),
                        sample_type: "single_shot".to_string(),
                        sequence_id: String::new(),
                        shot_order: String::new(),
                        reference_bundle_present: true,
                    },
                    source_field_refs: vec!["covered_points".to_string()],
                }],
            },
            repair_mapping: GoldenSampleRepairMappingAsset {
                schema_version: "golden_sample_repair_mapping.v0.2".to_string(),
                asset_name: "golden_sample_repair_mapping".to_string(),
                generated_at: "2026-04-23".to_string(),
                source_context: GoldenSampleSourceContext {
                    primary_source_id: "v120".to_string(),
                    comparison_source_id: "v108".to_string(),
                },
                record_count: 1,
                records: vec![GoldenSampleRepairMappingRecord {
                    mapping_id: "repair_map_01".to_string(),
                    sample_id: "GS-BRIDGE-01".to_string(),
                    core: "full_stack_single_shot_sample".to_string(),
                    repair_planning_mode: "positive_fewshot_candidate".to_string(),
                    linked_failure_mapping_id: "failure_map_01".to_string(),
                    planned_repair_inputs: GoldenSampleRepairInputs {
                        library_status: "official".to_string(),
                        missed_points: String::new(),
                        reserve_reason: String::new(),
                        sample_type: "single_shot".to_string(),
                        scene_category: "daily_dialogue".to_string(),
                        sequence_id: String::new(),
                        style_cluster: "dialogue".to_string(),
                        teaching_note: "bridge evidence".to_string(),
                        usable_for_fewshot: "Yes".to_string(),
                    },
                    future_repair_gate: "fewshot_retrieval_gate".to_string(),
                    planning_only: true,
                }],
            },
            source_register: GoldenSampleSourceRegister {
                schema_version: "hope-kb-v0.2-source-register".to_string(),
                register_name: "hope-kb-v0.2-source-register".to_string(),
                updated_at: "2026-04-23".to_string(),
                sources: vec![GoldenSampleSourceRegisterEntry {
                    source_id: "v120".to_string(),
                    source_type: "immutable_raw_xlsx".to_string(),
                    label: "V120".to_string(),
                    path: "workbook.xlsx".to_string(),
                    sha256: Some("hash".to_string()),
                    applies_to: vec!["golden_sample_library".to_string()],
                    notes: "test source".to_string(),
                }],
                provenance_entries: vec![GoldenSampleSourceRegisterProvenance {
                    provenance_id: "v120_package".to_string(),
                    source_ids: vec!["v120".to_string()],
                    generated_files: vec!["seed/v0.2/golden_sample_library.json".to_string()],
                    preservation_contract: BTreeMap::from([(
                        "imports_into_live_hope_runtime".to_string(),
                        false,
                    )]),
                }],
            },
        }
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
    fn v120_bridge_expands_generates_and_exports_without_live_model() {
        let state = test_state();
        let script = expand_script(
            &state,
            ExpandScriptRequest {
                scene_type: "daily_dialogue".to_string(),
                synopsis_text: "Two leads negotiate quietly before dawn.".to_string(),
            },
        );

        assert!(script.script_id.starts_with("script-"));
        assert!(script.expanded_script_text.contains("daily_dialogue"));
        assert!(script
            .warnings
            .iter()
            .any(|warning| warning.code == "qwen_live_generation_closed"));

        let storyboard = generate_storyboard(
            &state,
            GenerateStoryboardRequest {
                task_name: "daily_dialogue_scene".to_string(),
                script_id: Some(script.script_id.clone()),
                expanded_script_text: None,
                selected_total_duration_seconds: 8,
            },
        );

        assert_eq!(storyboard.rows.len(), 1);
        assert_eq!(storyboard.rows[0].shot_id, "GS-BRIDGE-01");
        assert!(storyboard.rows[0].prompt_text.is_empty());
        assert!(storyboard.rows[0]
            .prompt_body_candidate
            .candidate_text
            .is_some());
        assert_eq!(
            storyboard.rows[0].sequence_grouping.sequence_field_state,
            core_domain::SequenceFieldState::NotApplicable
        );
        assert!(storyboard.rows[0]
            .external_reference_handle_candidates
            .iter()
            .any(|candidate| candidate.reference_name == "scene_room"));
        assert_eq!(
            storyboard.export_status.status,
            BridgeCallStatus::WarningOnly
        );

        let export = export_bundle(
            &state,
            ExportBundleRequest {
                result_id: storyboard.result_id,
                export_format: "xlsx".to_string(),
            },
        );

        assert_eq!(export.artifacts.len(), 2);
        assert!(export
            .artifacts
            .iter()
            .any(|artifact| artifact.artifact_kind == "v120_bridge_manifest" && artifact.ready));
        assert!(export.artifacts.iter().any(
            |artifact| artifact.blocked_reason.as_deref() == Some("excel_exporter_gate_closed")
        ));
    }

    #[test]
    fn build_storyboard_preview_plan_keeps_runtime_directors_with_scene_taxonomy_metadata() {
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
                scene_director_id: Some("scene-director-preview".to_string()),
                action_director_id: Some("action-director-preview".to_string()),
                scene_type: Some("daily_dialogue".to_string()),
                layout_prompt: "cool palette, medium shot".to_string(),
                render_prompt: "restrained realism".to_string(),
            },
        )
        .expect("runtime directors with taxonomy metadata should build preview plan");

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
            "scene-director-preview"
        );
        assert_eq!(
            plan.committee_runtime
                .director_assignment
                .primary_action_director_id,
            "action-director-preview"
        );
        assert_eq!(
            plan.committee_runtime.prompt_layers.layout_prompt,
            "cool palette, medium shot"
        );
        assert_eq!(
            plan.committee_runtime.prompt_layers.render_prompt,
            "restrained realism"
        );
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
    fn build_validation_export_panel_snapshot_surfaces_kb_repairs() {
        let state = test_state();
        let mut fixture = load_week3_shared_fixture(WEEK3_SHARED_FIXTURE_PATH)
            .expect("shared fixture should load");
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
        let mut fixture = load_week3_shared_fixture(WEEK3_SHARED_FIXTURE_PATH)
            .expect("shared fixture should load");
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
