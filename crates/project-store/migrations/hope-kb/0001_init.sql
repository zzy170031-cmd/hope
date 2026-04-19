PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS schema_meta (
    schema_name TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS kb_snapshot (
    snapshot_id TEXT PRIMARY KEY,
    snapshot_hash TEXT NOT NULL,
    seed_format TEXT NOT NULL,
    source_name TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS director_profile (
    director_profile_id TEXT PRIMARY KEY,
    director_name TEXT NOT NULL,
    category TEXT NOT NULL,
    director_role_tags TEXT NOT NULL,
    core_kernel_summary TEXT NOT NULL,
    visual_style_tokens TEXT NOT NULL,
    signature_camera_works TEXT NOT NULL,
    signature_transitions TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_notes TEXT NOT NULL,
    confidence_level TEXT NOT NULL,
    last_reviewed_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS director_cut_sample (
    director_cut_sample_id TEXT PRIMARY KEY,
    director_profile_id TEXT NOT NULL REFERENCES director_profile(director_profile_id) ON DELETE CASCADE,
    sample_title TEXT NOT NULL,
    sample_body TEXT NOT NULL,
    representative_note TEXT NOT NULL,
    scene_type TEXT NOT NULL,
    layout_prompt TEXT NOT NULL,
    render_prompt TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_notes TEXT NOT NULL,
    confidence_level TEXT NOT NULL,
    last_reviewed_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS committee_template (
    template_id TEXT PRIMARY KEY,
    template_name TEXT NOT NULL,
    applicable_scope TEXT NOT NULL,
    member_layout TEXT NOT NULL,
    responsibility_notes TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_notes TEXT NOT NULL,
    confidence_level TEXT NOT NULL,
    last_reviewed_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS committee_handoff_rule (
    handoff_rule_id TEXT PRIMARY KEY,
    from_role TEXT NOT NULL,
    to_role TEXT NOT NULL,
    transition_type TEXT NOT NULL,
    buffer_guidance TEXT NOT NULL,
    continuity_notes TEXT NOT NULL,
    before_cut_pattern TEXT NOT NULL,
    after_cut_pattern TEXT NOT NULL,
    buffer_cut_pattern TEXT NOT NULL,
    applicable_director_pairs TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_notes TEXT NOT NULL,
    confidence_level TEXT NOT NULL,
    last_reviewed_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS visual_term (
    term_id TEXT PRIMARY KEY,
    chinese_term TEXT NOT NULL,
    category TEXT NOT NULL,
    prompt_token TEXT NOT NULL,
    definition TEXT NOT NULL,
    usage_rule TEXT NOT NULL,
    example_usage TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cinematography_term (
    term_id TEXT PRIMARY KEY,
    chinese_term TEXT NOT NULL,
    category TEXT NOT NULL,
    prompt_token TEXT NOT NULL,
    aliases TEXT NOT NULL,
    definition TEXT NOT NULL,
    usage_rule TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS continuity_rule (
    continuity_rule_id TEXT PRIMARY KEY,
    rule_name TEXT NOT NULL,
    trigger_condition TEXT NOT NULL,
    judgement_rule TEXT NOT NULL,
    example TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS scene_taxonomy (
    scene_taxonomy_id TEXT PRIMARY KEY,
    scene_type TEXT NOT NULL,
    display_name TEXT NOT NULL,
    definition TEXT NOT NULL,
    default_duration_band TEXT NOT NULL,
    typical_committee_roles TEXT NOT NULL,
    default_handoff_out TEXT NOT NULL,
    risk_flags TEXT NOT NULL,
    continuity_priority TEXT NOT NULL,
    prompt_focus TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_notes TEXT NOT NULL,
    confidence_level TEXT NOT NULL,
    last_reviewed_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS prompt_template (
    prompt_template_id TEXT PRIMARY KEY,
    stage TEXT NOT NULL,
    name TEXT NOT NULL,
    required_inputs TEXT NOT NULL,
    body TEXT NOT NULL,
    expected_output_schema TEXT NOT NULL,
    repairs_failure_codes TEXT,
    target_model_family TEXT NOT NULL,
    is_structured_output INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS failure_pattern (
    failure_pattern_id TEXT PRIMARY KEY,
    failure_code TEXT NOT NULL,
    failure_name TEXT NOT NULL,
    failure_category TEXT NOT NULL,
    symptom TEXT NOT NULL,
    common_causes TEXT NOT NULL,
    detection_hint TEXT NOT NULL,
    repair_strategy TEXT NOT NULL,
    affected_layers TEXT NOT NULL,
    validator_hint TEXT NOT NULL,
    repair_template_ids TEXT NOT NULL,
    repair_priority TEXT NOT NULL,
    repair_scope TEXT NOT NULL,
    suggested_followup_validators TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_notes TEXT NOT NULL,
    confidence_level TEXT NOT NULL,
    last_reviewed_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS seed_import_batch (
    batch_id TEXT PRIMARY KEY,
    source_name TEXT NOT NULL,
    seed_format TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    imported_at INTEGER NOT NULL
);
