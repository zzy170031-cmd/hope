#![forbid(unsafe_code)]

use crate::contract::{
    ValidationEnvelope, ValidationFinding, ValidationReport, contains_placeholder_marker, is_blank,
};

const PROJECT_SCOPE: &str = "project";
const LEVELS: &[&str] = &[
    "project",
    "episode",
    "narrative_scene",
    "render_segment",
    "cut",
];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct HardLockInjectorGuard;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardLockInjectorGuardInput<'a> {
    pub envelope: ValidationEnvelope,
    pub hard_lock_id: &'a str,
    pub lock_name: &'a str,
    pub lock_value: &'a str,
    pub scope: &'a str,
    pub allowed_lock_names: &'a [&'a str],
}

impl HardLockInjectorGuard {
    pub fn validate(&self, input: &HardLockInjectorGuardInput<'_>) -> ValidationReport {
        let mut findings = Vec::new();

        if is_blank(&input.envelope.validation_report_id) {
            findings.push(ValidationFinding::block(
                "validation_report_id_missing",
                "hard_lock",
                "validation_report_id is required for Validation sheet mapping",
                "validation_report_id = \"\"",
            ));
        }

        if is_blank(&input.envelope.project_id) {
            findings.push(ValidationFinding::block(
                "project_id_missing",
                "hard_lock",
                "project_id is required for project-scoped hard locks",
                "project_id = \"\"",
            ));
        }

        if is_blank(input.hard_lock_id) {
            findings.push(ValidationFinding::block(
                "hard_lock_id_missing",
                "hard_lock",
                "hard_lock_id cannot be empty",
                "hard_lock_id = \"\"",
            ));
        }

        if is_blank(input.lock_name) {
            findings.push(ValidationFinding::block(
                "lock_name_missing",
                "hard_lock",
                "lock_name cannot be empty",
                "lock_name = \"\"",
            ));
        }

        if is_blank(input.lock_value) {
            findings.push(ValidationFinding::block(
                "lock_value_missing",
                "hard_lock",
                "lock_value cannot be empty",
                "lock_value = \"\"",
            ));
        }

        if input.scope != PROJECT_SCOPE {
            findings.push(ValidationFinding::block(
                "scope_not_project",
                "hard_lock",
                "hard locks are project-scoped only",
                "scope = \"episode\"",
            ));
        }

        if input.allowed_lock_names.is_empty() {
            findings.push(ValidationFinding::block(
                "frozen_lock_set_missing",
                "hard_lock",
                "frozen hard lock names must be supplied from Track A",
                "allowed_lock_names = []",
            ));
        } else if !input
            .allowed_lock_names
            .iter()
            .any(|candidate| *candidate == input.lock_name)
        {
            findings.push(ValidationFinding::block(
                "lock_name_not_frozen",
                "hard_lock",
                "lock_name is not in the frozen hard lock set",
                "lock_name = \"unfrozen_lock\"",
            ));
        }

        if contains_placeholder_marker(input.lock_name)
            || contains_placeholder_marker(input.lock_value)
            || contains_placeholder_marker(input.scope)
        {
            findings.push(ValidationFinding::warn(
                "placeholder_marker_detected",
                "hard_lock",
                "placeholder markers should be removed before export",
                "lock_value = \"TODO\"",
            ));
        }

        ValidationReport::new(input.envelope.clone(), findings)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StyleUnityValidator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleUnityInput<'a> {
    pub envelope: ValidationEnvelope,
    pub prompt_package_id: &'a str,
    pub director_profile_id: &'a str,
    pub director_cut_sample_id: &'a str,
    pub committee_template_id: &'a str,
    pub prompt_text: &'a str,
    pub visual_terms: &'a [&'a str],
    pub cinematography_terms: &'a [&'a str],
}

impl StyleUnityValidator {
    pub fn validate(&self, input: &StyleUnityInput<'_>) -> ValidationReport {
        let mut findings = Vec::new();

        if is_blank(&input.envelope.validation_report_id) {
            findings.push(ValidationFinding::block(
                "validation_report_id_missing",
                "prompt_package",
                "validation_report_id is required for Validation sheet mapping",
                "validation_report_id = \"\"",
            ));
        }

        if is_blank(&input.envelope.project_id) {
            findings.push(ValidationFinding::block(
                "project_id_missing",
                "prompt_package",
                "project_id is required for prompt-package validation",
                "project_id = \"\"",
            ));
        }

        if is_blank(input.prompt_package_id)
            || is_blank(input.director_profile_id)
            || is_blank(input.director_cut_sample_id)
            || is_blank(input.committee_template_id)
        {
            findings.push(ValidationFinding::block(
                "anchor_id_missing",
                "prompt_package",
                "prompt package, director profile, director cut sample, and committee template anchors are required",
                "director_profile_id = \"\"",
            ));
        }

        if input.visual_terms.is_empty() || input.cinematography_terms.is_empty() {
            findings.push(ValidationFinding::block(
                "frozen_term_set_missing",
                "prompt_package",
                "style and cinematography vocabularies must come from the frozen Track C seed set",
                "visual_terms = []",
            ));
        }

        let matches_visual = !input.visual_terms.is_empty()
            && input
                .visual_terms
                .iter()
                .any(|term| !term.trim().is_empty() && input.prompt_text.contains(term));
        let matches_cinematography = !input.cinematography_terms.is_empty()
            && input
                .cinematography_terms
                .iter()
                .any(|term| !term.trim().is_empty() && input.prompt_text.contains(term));

        if is_blank(input.prompt_text) {
            findings.push(ValidationFinding::block(
                "prompt_text_missing",
                "prompt_package",
                "prompt_text cannot be empty",
                "prompt_text = \"\"",
            ));
        } else if !matches_visual || !matches_cinematography {
            findings.push(ValidationFinding::block(
                "style_unity_not_proven",
                "prompt_package",
                "prompt_text must anchor both visual language and cinematography language",
                "prompt_text = \"scene without any style or cinematography term\"",
            ));
        }

        if contains_placeholder_marker(input.prompt_text)
            || input
                .visual_terms
                .iter()
                .any(|term| contains_placeholder_marker(term))
            || input
                .cinematography_terms
                .iter()
                .any(|term| contains_placeholder_marker(term))
        {
            findings.push(ValidationFinding::warn(
                "placeholder_marker_detected",
                "prompt_package",
                "placeholder markers should be removed from prompt content and vocab sets",
                "prompt_text = \"TODO\"",
            ));
        }

        if matches_visual ^ matches_cinematography {
            findings.push(ValidationFinding::warn(
                "one_family_only",
                "prompt_package",
                "only one vocabulary family matched; style unity should be reviewed",
                "prompt_text = \"visual term only\"",
            ));
        }

        ValidationReport::new(input.envelope.clone(), findings)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NegativeGlobalGuard;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegativeGlobalGuardInput<'a> {
    pub envelope: ValidationEnvelope,
    pub prompt_package_id: &'a str,
    pub prompt_text: &'a str,
    pub blocked_phrases: &'a [&'a str],
}

impl NegativeGlobalGuard {
    pub fn validate(&self, input: &NegativeGlobalGuardInput<'_>) -> ValidationReport {
        let mut findings = Vec::new();

        if is_blank(&input.envelope.validation_report_id) {
            findings.push(ValidationFinding::block(
                "validation_report_id_missing",
                "prompt_package",
                "validation_report_id is required for Validation sheet mapping",
                "validation_report_id = \"\"",
            ));
        }

        if is_blank(&input.envelope.project_id) {
            findings.push(ValidationFinding::block(
                "project_id_missing",
                "prompt_package",
                "project_id is required for negative-global checks",
                "project_id = \"\"",
            ));
        }

        if is_blank(input.prompt_package_id) {
            findings.push(ValidationFinding::block(
                "prompt_package_id_missing",
                "prompt_package",
                "prompt_package_id cannot be empty",
                "prompt_package_id = \"\"",
            ));
        }

        if input.blocked_phrases.is_empty() {
            findings.push(ValidationFinding::block(
                "blocked_phrase_set_missing",
                "prompt_package",
                "blocked phrases must come from the frozen hard-lock set",
                "blocked_phrases = []",
            ));
        }

        if is_blank(input.prompt_text) {
            findings.push(ValidationFinding::block(
                "prompt_text_missing",
                "prompt_package",
                "prompt_text cannot be empty",
                "prompt_text = \"\"",
            ));
        }

        if input
            .blocked_phrases
            .iter()
            .any(|phrase| !phrase.trim().is_empty() && input.prompt_text.contains(phrase))
        {
            findings.push(ValidationFinding::block(
                "blocked_phrase_hit",
                "prompt_package",
                "prompt_text contains a frozen blocked phrase",
                "prompt_text = \"...blocked phrase...\"",
            ));
        }

        if contains_placeholder_marker(input.prompt_text)
            || input
                .blocked_phrases
                .iter()
                .any(|phrase| contains_placeholder_marker(phrase))
        {
            findings.push(ValidationFinding::warn(
                "placeholder_marker_detected",
                "prompt_package",
                "placeholder markers should be removed before export",
                "prompt_text = \"TBD\"",
            ));
        }

        ValidationReport::new(input.envelope.clone(), findings)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ContinuityValidator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuityInput<'a> {
    pub envelope: ValidationEnvelope,
    pub stale_event_id: &'a str,
    pub source_level: &'a str,
    pub source_id: &'a str,
    pub target_level: &'a str,
    pub target_id: &'a str,
    pub trace_id: &'a str,
}

impl ContinuityValidator {
    pub fn validate(&self, input: &ContinuityInput<'_>) -> ValidationReport {
        let mut findings = Vec::new();

        if is_blank(&input.envelope.validation_report_id) {
            findings.push(ValidationFinding::block(
                "validation_report_id_missing",
                "stale_event",
                "validation_report_id is required for Validation sheet mapping",
                "validation_report_id = \"\"",
            ));
        }

        if is_blank(&input.envelope.project_id) {
            findings.push(ValidationFinding::block(
                "project_id_missing",
                "stale_event",
                "project_id is required for stale propagation checks",
                "project_id = \"\"",
            ));
        }

        if is_blank(input.stale_event_id)
            || is_blank(input.source_level)
            || is_blank(input.source_id)
            || is_blank(input.target_level)
            || is_blank(input.target_id)
            || is_blank(input.trace_id)
        {
            findings.push(ValidationFinding::block(
                "stale_chain_incomplete",
                "stale_event",
                "source, target, and trace identifiers must all be present",
                "trace_id = \"\"",
            ));
        }

        if !LEVELS.contains(&input.source_level) || !LEVELS.contains(&input.target_level) {
            findings.push(ValidationFinding::block(
                "level_out_of_contract",
                "stale_event",
                "stale propagation levels must stay within the frozen project hierarchy",
                "source_level = \"chapter\"",
            ));
        }

        if input.source_level == input.target_level && input.source_id == input.target_id {
            findings.push(ValidationFinding::block(
                "self_targeted_stale_event",
                "stale_event",
                "stale propagation must point to a different target than the source",
                "source_id = target_id",
            ));
        }

        if contains_placeholder_marker(input.trace_id)
            || contains_placeholder_marker(input.source_id)
            || contains_placeholder_marker(input.target_id)
        {
            findings.push(ValidationFinding::warn(
                "placeholder_marker_detected",
                "stale_event",
                "placeholder markers should be removed from stale traces",
                "trace_id = \"TODO\"",
            ));
        }

        ValidationReport::new(input.envelope.clone(), findings)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SegmentDurationValidator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentDurationInput<'a> {
    pub envelope: ValidationEnvelope,
    pub render_segment_id: &'a str,
    pub narrative_scene_id: &'a str,
    pub target_duration_seconds: u32,
    pub actual_duration_seconds: Option<u32>,
}

impl SegmentDurationValidator {
    pub fn validate(&self, input: &SegmentDurationInput<'_>) -> ValidationReport {
        let mut findings = Vec::new();

        if is_blank(&input.envelope.validation_report_id) {
            findings.push(ValidationFinding::block(
                "validation_report_id_missing",
                "render_segment",
                "validation_report_id is required for Validation sheet mapping",
                "validation_report_id = \"\"",
            ));
        }

        if is_blank(&input.envelope.project_id) {
            findings.push(ValidationFinding::block(
                "project_id_missing",
                "render_segment",
                "project_id is required for segment duration checks",
                "project_id = \"\"",
            ));
        }

        if is_blank(input.render_segment_id) || is_blank(input.narrative_scene_id) {
            findings.push(ValidationFinding::block(
                "segment_identity_missing",
                "render_segment",
                "render_segment_id and narrative_scene_id cannot be empty",
                "render_segment_id = \"\"",
            ));
        }

        if !(30..=90).contains(&input.target_duration_seconds) {
            findings.push(ValidationFinding::block(
                "target_duration_out_of_bounds",
                "render_segment",
                "target duration must stay within 30s to 90s",
                "target_duration_seconds = 120",
            ));
        }

        if let Some(actual_duration_seconds) = input.actual_duration_seconds {
            if !(30..=90).contains(&actual_duration_seconds) {
                findings.push(ValidationFinding::block(
                    "actual_duration_out_of_bounds",
                    "render_segment",
                    "actual duration must stay within the frozen render-segment bound",
                    "actual_duration_seconds = 12",
                ));
            }
        } else {
            findings.push(ValidationFinding::warn(
                "actual_duration_missing",
                "render_segment",
                "actual duration is still missing; the report can pass, but export should keep checking",
                "actual_duration_seconds = None",
            ));
        }

        if contains_placeholder_marker(input.render_segment_id)
            || contains_placeholder_marker(input.narrative_scene_id)
        {
            findings.push(ValidationFinding::warn(
                "placeholder_marker_detected",
                "render_segment",
                "placeholder markers should be removed from segment identity fields",
                "render_segment_id = \"TODO\"",
            ));
        }

        ValidationReport::new(input.envelope.clone(), findings)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LayerTraceabilityValidator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerTraceabilityInput<'a> {
    pub envelope: ValidationEnvelope,
    pub trace_id: &'a str,
    pub source_level: &'a str,
    pub source_id: &'a str,
    pub target_level: &'a str,
    pub target_id: &'a str,
    pub render_segment_id: Option<&'a str>,
    pub cut_id: Option<&'a str>,
}

impl LayerTraceabilityValidator {
    pub fn validate(&self, input: &LayerTraceabilityInput<'_>) -> ValidationReport {
        let mut findings = Vec::new();

        if is_blank(&input.envelope.validation_report_id) {
            findings.push(ValidationFinding::block(
                "validation_report_id_missing",
                "validation_report",
                "validation_report_id is required for Validation sheet mapping",
                "validation_report_id = \"\"",
            ));
        }

        if is_blank(&input.envelope.project_id) {
            findings.push(ValidationFinding::block(
                "project_id_missing",
                "validation_report",
                "project_id is required for traceability checks",
                "project_id = \"\"",
            ));
        }

        if is_blank(input.trace_id)
            || is_blank(input.source_level)
            || is_blank(input.source_id)
            || is_blank(input.target_level)
            || is_blank(input.target_id)
        {
            findings.push(ValidationFinding::block(
                "trace_chain_incomplete",
                "validation_report",
                "trace_id, source, and target identifiers must all be present",
                "trace_id = \"\"",
            ));
        }

        if input.render_segment_id.is_none() && input.cut_id.is_none() {
            findings.push(ValidationFinding::block(
                "no_entity_anchor",
                "validation_report",
                "traceability needs at least one entity anchor from render_segment or cut",
                "render_segment_id = None",
            ));
        }

        if input.render_segment_id.is_some() ^ input.cut_id.is_some() {
            findings.push(ValidationFinding::warn(
                "partial_entity_anchor",
                "validation_report",
                "only one entity anchor is present; traceability is partial but recoverable",
                "cut_id = None",
            ));
        }

        if !LEVELS.contains(&input.source_level) || !LEVELS.contains(&input.target_level) {
            findings.push(ValidationFinding::block(
                "level_out_of_contract",
                "validation_report",
                "traceability levels must stay within the frozen project hierarchy",
                "source_level = \"chapter\"",
            ));
        }

        if contains_placeholder_marker(input.trace_id)
            || contains_placeholder_marker(input.source_id)
            || contains_placeholder_marker(input.target_id)
        {
            findings.push(ValidationFinding::warn(
                "placeholder_marker_detected",
                "validation_report",
                "placeholder markers should be removed from trace identifiers",
                "trace_id = \"TODO\"",
            ));
        }

        ValidationReport::new(input.envelope.clone(), findings)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct HandoffCoverageValidator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffCoverageInput<'a> {
    pub envelope: ValidationEnvelope,
    pub handoff_zone_id: &'a str,
    pub render_segment_id: &'a str,
    pub start_boundary: &'a str,
    pub end_boundary: &'a str,
    pub boundary_type: &'a str,
}

impl HandoffCoverageValidator {
    pub fn validate(&self, input: &HandoffCoverageInput<'_>) -> ValidationReport {
        let mut findings = Vec::new();

        if is_blank(&input.envelope.validation_report_id) {
            findings.push(ValidationFinding::block(
                "validation_report_id_missing",
                "handoff_zone",
                "validation_report_id is required for Validation sheet mapping",
                "validation_report_id = \"\"",
            ));
        }

        if is_blank(&input.envelope.project_id) {
            findings.push(ValidationFinding::block(
                "project_id_missing",
                "handoff_zone",
                "project_id is required for handoff coverage checks",
                "project_id = \"\"",
            ));
        }

        if is_blank(input.handoff_zone_id) || is_blank(input.render_segment_id) {
            findings.push(ValidationFinding::block(
                "handoff_identity_missing",
                "handoff_zone",
                "handoff_zone_id and render_segment_id cannot be empty",
                "handoff_zone_id = \"\"",
            ));
        }

        if is_blank(input.start_boundary)
            || is_blank(input.end_boundary)
            || is_blank(input.boundary_type)
        {
            findings.push(ValidationFinding::block(
                "handoff_boundary_missing",
                "handoff_zone",
                "start_boundary, end_boundary, and boundary_type must all be present",
                "start_boundary = \"\"",
            ));
        }

        if input.start_boundary == input.end_boundary {
            findings.push(ValidationFinding::block(
                "handoff_boundary_collapsed",
                "handoff_zone",
                "handoff boundaries must not collapse to the same value",
                "start_boundary = end_boundary",
            ));
        }

        if contains_placeholder_marker(input.start_boundary)
            || contains_placeholder_marker(input.end_boundary)
            || contains_placeholder_marker(input.boundary_type)
        {
            findings.push(ValidationFinding::warn(
                "placeholder_marker_detected",
                "handoff_zone",
                "placeholder markers should be removed from handoff boundaries",
                "boundary_type = \"TODO\"",
            ));
        }

        ValidationReport::new(input.envelope.clone(), findings)
    }
}
