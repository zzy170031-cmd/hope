use storyboard_pipeline::shot_language_selection::{
    CameraLanguagePlanningSignal, InternalStyleLaneId, V108ShotLanguageSelectionBlocker,
    V108ShotLanguageSelectionInput, V108ShotLanguageSelectionMetadata,
    V108ShotLanguageSelectionReadiness, select_v108_shot_language_metadata,
};

fn input_with_evidence(evidence: &str) -> V108ShotLanguageSelectionInput {
    V108ShotLanguageSelectionInput {
        style_cluster: evidence.to_string(),
        scene_category: evidence.to_string(),
        scene_tag: evidence.to_string(),
        camera_directing_core: evidence.to_string(),
        covered_points: vec![evidence.to_string()],
        teaching_note: evidence.to_string(),
        continuity_negative_core: evidence.to_string(),
        schema_field_missing: true,
        promotion_gate_not_accepted: true,
        reference_control_core_closed: true,
        ..V108ShotLanguageSelectionInput::default()
    }
}

fn assert_lane(metadata: &V108ShotLanguageSelectionMetadata, lane_id: InternalStyleLaneId) {
    assert!(
        metadata
            .lane_candidates
            .iter()
            .any(|candidate| candidate.lane_id == lane_id),
        "expected lane {} in {:?}",
        lane_id.as_str(),
        metadata.lane_candidates
    );
}

fn assert_no_lane(metadata: &V108ShotLanguageSelectionMetadata, lane_id: InternalStyleLaneId) {
    assert!(
        !metadata
            .lane_candidates
            .iter()
            .any(|candidate| candidate.lane_id == lane_id),
        "did not expect lane {} in {:?}",
        lane_id.as_str(),
        metadata.lane_candidates
    );
}

fn assert_signal(
    metadata: &V108ShotLanguageSelectionMetadata,
    signal: CameraLanguagePlanningSignal,
) {
    assert!(
        metadata.camera_language_planning_signals.contains(&signal),
        "expected signal {:?} in {:?}",
        signal,
        metadata.camera_language_planning_signals
    );
}

fn assert_blocker(
    metadata: &V108ShotLanguageSelectionMetadata,
    blocker: V108ShotLanguageSelectionBlocker,
) {
    assert!(
        metadata.blockers.contains(&blocker),
        "expected blocker {:?} in {:?}",
        blocker,
        metadata.blockers
    );
}

fn assert_no_runtime_output(metadata: &V108ShotLanguageSelectionMetadata) {
    assert!(!metadata.emits_final_storyboard_words);
    assert!(!metadata.emits_runtime_prompt_text);
    assert_eq!(
        metadata
            .no_runtime_import_assertions
            .raw_source_rows_ready_for_future_dry_run,
        115
    );
    assert_eq!(
        metadata
            .no_runtime_import_assertions
            .rows_ready_for_product_import,
        0
    );
    assert_eq!(
        metadata
            .no_runtime_import_assertions
            .rows_ready_for_positive_fewshot,
        0
    );
    assert_eq!(
        metadata
            .no_runtime_import_assertions
            .product_ready_external_reference_handles,
        0
    );
    assert_eq!(
        metadata
            .no_runtime_import_assertions
            .reference_control_core_coverage,
        0
    );
    assert_eq!(metadata.no_runtime_import_assertions.qwen_calls, 0);
    assert_eq!(metadata.no_runtime_import_assertions.seedance_calls, 0);
}

#[test]
fn routes_guofeng_action_to_director_08_metadata_only() {
    let input = input_with_evidence(
        "guoman hot blood action guofeng wuxia weapon qinggong bamboo roof rain blade burst landing",
    );

    let metadata = select_v108_shot_language_metadata(&input);

    assert_lane(&metadata, InternalStyleLaneId::Director08);
    assert_lane(&metadata, InternalStyleLaneId::Director01);
    assert_signal(&metadata, CameraLanguagePlanningSignal::WeaponContinuity);
    assert_signal(
        &metadata,
        CameraLanguagePlanningSignal::QinggongMovementAxis,
    );
    assert_eq!(
        metadata.readiness,
        V108ShotLanguageSelectionReadiness::PlanningMetadataOnly
    );
    assert_blocker(
        &metadata,
        V108ShotLanguageSelectionBlocker::SchemaFieldMissing,
    );
    assert_blocker(
        &metadata,
        V108ShotLanguageSelectionBlocker::PromotionGateNotAccepted,
    );
    assert_blocker(
        &metadata,
        V108ShotLanguageSelectionBlocker::ReferenceControlCoreClosed,
    );
    assert_no_runtime_output(&metadata);
}

#[test]
fn routes_epic_group_or_war_to_director_09_metadata_only() {
    let input = input_with_evidence(
        "guoman group performance war oath army formation battlefield command flag drum epic crowd rhythm",
    );

    let metadata = select_v108_shot_language_metadata(&input);

    assert_lane(&metadata, InternalStyleLaneId::Director09);
    assert_signal(&metadata, CameraLanguagePlanningSignal::FormationDepth);
    assert_signal(&metadata, CameraLanguagePlanningSignal::FlagDrumBridge);
    assert_no_runtime_output(&metadata);
}

#[test]
fn routes_urban_apocalypse_pressure_to_director_10_metadata_only() {
    let input = input_with_evidence(
        "urban apocalypse industrial subway overpass ruins evacuation alarm infrastructure pressure",
    );

    let metadata = select_v108_shot_language_metadata(&input);

    assert_lane(&metadata, InternalStyleLaneId::Director10);
    assert_signal(&metadata, CameraLanguagePlanningSignal::EvacuationAxis);
    assert_signal(
        &metadata,
        CameraLanguagePlanningSignal::IndustrialCompression,
    );
    assert_signal(&metadata, CameraLanguagePlanningSignal::AlarmBridge);
    assert_no_runtime_output(&metadata);
}

#[test]
fn routes_stage_group_performance_to_director_11_metadata_only() {
    let input = input_with_evidence(
        "original group performance stage group dance music spotlight entrance final freeze",
    );

    let metadata = select_v108_shot_language_metadata(&input);

    assert_lane(&metadata, InternalStyleLaneId::Director11);
    assert_signal(&metadata, CameraLanguagePlanningSignal::StageAxis);
    assert_signal(&metadata, CameraLanguagePlanningSignal::MusicSyncCue);
    assert_signal(&metadata, CameraLanguagePlanningSignal::FreezeFramePlanning);
    assert_no_runtime_output(&metadata);
}

#[test]
fn keeps_original_seven_lanes_for_broad_action_emotion_suspense() {
    let broad_action =
        select_v108_shot_language_metadata(&input_with_evidence("broad action impact rhythm"));
    let emotion = select_v108_shot_language_metadata(&input_with_evidence(
        "emotion landscape weather light transition",
    ));
    let suspense = select_v108_shot_language_metadata(&input_with_evidence(
        "suspense match cut reality displacement",
    ));

    assert_lane(&broad_action, InternalStyleLaneId::Director01);
    assert_no_lane(&broad_action, InternalStyleLaneId::Director08);
    assert_lane(&emotion, InternalStyleLaneId::Director04);
    assert_no_lane(&emotion, InternalStyleLaneId::Director11);
    assert_lane(&suspense, InternalStyleLaneId::Director07);
    assert_no_lane(&suspense, InternalStyleLaneId::Director10);
}

#[test]
fn placeholder_blocks_selection_to_raw_evidence_only() {
    let mut input = input_with_evidence("guofeng wuxia weapon qinggong action");
    input.placeholder_present = true;

    let metadata = select_v108_shot_language_metadata(&input);

    assert!(metadata.lane_candidates.is_empty());
    assert_eq!(
        metadata.readiness,
        V108ShotLanguageSelectionReadiness::RawEvidenceOnly
    );
    assert_signal(&metadata, CameraLanguagePlanningSignal::RawEvidenceOnly);
    assert_blocker(
        &metadata,
        V108ShotLanguageSelectionBlocker::PlaceholderPresent,
    );
    assert_no_runtime_output(&metadata);
}

#[test]
fn reference_unresolved_preserves_canonical_name_missing() {
    let mut input = input_with_evidence("stage music spotlight performance");
    input.reference_handle_unresolved = true;
    input.canonical_name_missing = true;

    let metadata = select_v108_shot_language_metadata(&input);

    assert!(metadata.lane_candidates.is_empty());
    assert_eq!(
        metadata.readiness,
        V108ShotLanguageSelectionReadiness::RawEvidenceOnly
    );
    assert_blocker(
        &metadata,
        V108ShotLanguageSelectionBlocker::ReferenceHandleUnresolved,
    );
    assert_blocker(
        &metadata,
        V108ShotLanguageSelectionBlocker::CanonicalNameMissing,
    );
    assert_eq!(
        metadata
            .no_runtime_import_assertions
            .product_ready_external_reference_handles,
        0
    );
    assert_no_runtime_output(&metadata);
}

#[test]
fn schema_and_promotion_gates_keep_runtime_closed() {
    let input = input_with_evidence("urban apocalypse evacuation alarm industrial pressure");

    let metadata = select_v108_shot_language_metadata(&input);

    assert_lane(&metadata, InternalStyleLaneId::Director10);
    assert_blocker(
        &metadata,
        V108ShotLanguageSelectionBlocker::SchemaFieldMissing,
    );
    assert_blocker(
        &metadata,
        V108ShotLanguageSelectionBlocker::PromotionGateNotAccepted,
    );
    assert_eq!(
        metadata.readiness,
        V108ShotLanguageSelectionReadiness::PlanningMetadataOnly
    );
    assert_no_runtime_output(&metadata);
}

#[test]
fn never_emits_final_storyboard_words_or_runtime_prompt_text() {
    let input = input_with_evidence(
        "war army formation command flag drum with close combat body axis and impact rhythm",
    );

    let metadata = select_v108_shot_language_metadata(&input);

    assert_lane(&metadata, InternalStyleLaneId::Director09);
    assert_lane(&metadata, InternalStyleLaneId::Director03);
    assert_no_runtime_output(&metadata);
    assert_eq!(
        metadata
            .no_runtime_import_assertions
            .desktop_or_intake_changes,
        0
    );
    assert_eq!(metadata.no_runtime_import_assertions.v3_branch_edits, 0);
}
