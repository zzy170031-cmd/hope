#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InternalStyleLaneId {
    Director01,
    Director02,
    Director03,
    Director04,
    Director05,
    Director06,
    Director07,
    Director08,
    Director09,
    Director10,
    Director11,
}

impl InternalStyleLaneId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Director01 => "director_01",
            Self::Director02 => "director_02",
            Self::Director03 => "director_03",
            Self::Director04 => "director_04",
            Self::Director05 => "director_05",
            Self::Director06 => "director_06",
            Self::Director07 => "director_07",
            Self::Director08 => "director_08",
            Self::Director09 => "director_09",
            Self::Director10 => "director_10",
            Self::Director11 => "director_11",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraLanguagePlanningSignal {
    ActionIntensity,
    ImpactRhythm,
    AttackChainClarity,
    CrowdPressure,
    SpaceCompression,
    DangerAdvance,
    EnvironmentEmotionSupport,
    LightWeatherTransition,
    DetailObservation,
    LowViewpointMicroAction,
    SubjectiveDeformation,
    RealityDisplacement,
    WeaponContinuity,
    QinggongMovementAxis,
    GuofengSpaceRelation,
    BurstAndLandingRhythm,
    ChaseAxis,
    VerticalMovementBridge,
    FormationDepth,
    CommandCenter,
    FlagDrumBridge,
    CrowdRhythm,
    EvacuationAxis,
    IndustrialCompression,
    AlarmBridge,
    ObstacleSource,
    StageAxis,
    FormationContinuity,
    MusicSyncCue,
    FreezeFramePlanning,
    RawEvidenceOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V108ShotLanguageSelectionBlocker {
    PlaceholderPresent,
    PromptBodyBlockedByPlaceholder,
    ReferenceHandleUnresolved,
    CanonicalNameMissing,
    V3AlignmentGap,
    SchemaFieldMissing,
    PromotionGateNotAccepted,
    ReferenceControlCoreClosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V108ShotLanguageSelectionReadiness {
    PlanningMetadataOnly,
    RawEvidenceOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V108SampleType {
    SingleShot,
    SequenceShot,
}

impl Default for V108SampleType {
    fn default() -> Self {
        Self::SingleShot
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct V108ShotLanguageSelectionInput {
    pub style_cluster: String,
    pub scene_category: String,
    pub scene_tag: String,
    pub camera_directing_core: String,
    pub sample_type: V108SampleType,
    pub sequence_id: Option<String>,
    pub shot_order: Option<u32>,
    pub covered_points: Vec<String>,
    pub missed_points: Vec<String>,
    pub teaching_note: String,
    pub ip_abstraction_note: String,
    pub continuity_negative_core: String,
    pub placeholder_present: bool,
    pub prompt_body_blocked_by_placeholder: bool,
    pub reference_handle_unresolved: bool,
    pub canonical_name_missing: bool,
    pub v3_alignment_gap: bool,
    pub schema_field_missing: bool,
    pub promotion_gate_not_accepted: bool,
    pub reference_control_core_closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalStyleLaneCandidate {
    pub lane_id: InternalStyleLaneId,
    pub priority: u8,
    pub planning_signals: Vec<CameraLanguagePlanningSignal>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V108NoRuntimeImportAssertions {
    pub raw_source_rows_ready_for_future_dry_run: u16,
    pub rows_ready_for_product_import: u8,
    pub rows_ready_for_positive_fewshot: u8,
    pub product_ready_external_reference_handles: u8,
    pub reference_control_core_coverage: u8,
    pub qwen_calls: u8,
    pub seedance_calls: u8,
    pub desktop_or_intake_changes: u8,
    pub v3_branch_edits: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V108ShotLanguageSelectionMetadata {
    pub lane_candidates: Vec<InternalStyleLaneCandidate>,
    pub camera_language_planning_signals: Vec<CameraLanguagePlanningSignal>,
    pub blockers: Vec<V108ShotLanguageSelectionBlocker>,
    pub readiness: V108ShotLanguageSelectionReadiness,
    pub emits_final_storyboard_words: bool,
    pub emits_runtime_prompt_text: bool,
    pub no_runtime_import_assertions: V108NoRuntimeImportAssertions,
}

pub fn frozen_no_runtime_import_assertions() -> V108NoRuntimeImportAssertions {
    V108NoRuntimeImportAssertions {
        raw_source_rows_ready_for_future_dry_run: 115,
        rows_ready_for_product_import: 0,
        rows_ready_for_positive_fewshot: 0,
        product_ready_external_reference_handles: 0,
        reference_control_core_coverage: 0,
        qwen_calls: 0,
        seedance_calls: 0,
        desktop_or_intake_changes: 0,
        v3_branch_edits: 0,
    }
}

pub fn select_v108_shot_language_metadata(
    input: &V108ShotLanguageSelectionInput,
) -> V108ShotLanguageSelectionMetadata {
    let blockers = collect_blockers(input);
    let content_blocked = input.placeholder_present
        || input.prompt_body_blocked_by_placeholder
        || input.reference_handle_unresolved
        || input.canonical_name_missing
        || input.v3_alignment_gap;

    let lane_candidates = if content_blocked {
        Vec::new()
    } else {
        select_lane_candidates(input)
    };
    let camera_language_planning_signals = if content_blocked {
        vec![CameraLanguagePlanningSignal::RawEvidenceOnly]
    } else {
        collect_planning_signals(&lane_candidates)
    };

    V108ShotLanguageSelectionMetadata {
        lane_candidates,
        camera_language_planning_signals,
        blockers,
        readiness: if content_blocked {
            V108ShotLanguageSelectionReadiness::RawEvidenceOnly
        } else {
            V108ShotLanguageSelectionReadiness::PlanningMetadataOnly
        },
        emits_final_storyboard_words: false,
        emits_runtime_prompt_text: false,
        no_runtime_import_assertions: frozen_no_runtime_import_assertions(),
    }
}

fn collect_blockers(
    input: &V108ShotLanguageSelectionInput,
) -> Vec<V108ShotLanguageSelectionBlocker> {
    let mut blockers = Vec::new();
    push_if(
        &mut blockers,
        input.placeholder_present,
        V108ShotLanguageSelectionBlocker::PlaceholderPresent,
    );
    push_if(
        &mut blockers,
        input.prompt_body_blocked_by_placeholder,
        V108ShotLanguageSelectionBlocker::PromptBodyBlockedByPlaceholder,
    );
    push_if(
        &mut blockers,
        input.reference_handle_unresolved,
        V108ShotLanguageSelectionBlocker::ReferenceHandleUnresolved,
    );
    push_if(
        &mut blockers,
        input.canonical_name_missing,
        V108ShotLanguageSelectionBlocker::CanonicalNameMissing,
    );
    push_if(
        &mut blockers,
        input.v3_alignment_gap,
        V108ShotLanguageSelectionBlocker::V3AlignmentGap,
    );
    push_if(
        &mut blockers,
        input.schema_field_missing,
        V108ShotLanguageSelectionBlocker::SchemaFieldMissing,
    );
    push_if(
        &mut blockers,
        input.promotion_gate_not_accepted,
        V108ShotLanguageSelectionBlocker::PromotionGateNotAccepted,
    );
    push_if(
        &mut blockers,
        input.reference_control_core_closed,
        V108ShotLanguageSelectionBlocker::ReferenceControlCoreClosed,
    );
    blockers
}

fn select_lane_candidates(
    input: &V108ShotLanguageSelectionInput,
) -> Vec<InternalStyleLaneCandidate> {
    let evidence = combined_evidence(input);
    let mut candidates = Vec::new();

    if contains_any(
        &evidence,
        &[
            "guofeng", "wuxia", "xianxia", "weapon", "qinggong", "bamboo", "roof", "eave", "blade",
            "rain",
        ],
    ) {
        candidates.push(InternalStyleLaneCandidate {
            lane_id: InternalStyleLaneId::Director08,
            priority: 10,
            planning_signals: vec![
                CameraLanguagePlanningSignal::WeaponContinuity,
                CameraLanguagePlanningSignal::QinggongMovementAxis,
                CameraLanguagePlanningSignal::GuofengSpaceRelation,
                CameraLanguagePlanningSignal::BurstAndLandingRhythm,
            ],
        });
    }

    if contains_any(
        &evidence,
        &[
            "war",
            "army",
            "formation",
            "battlefield",
            "oath",
            "command",
            "flag",
            "drum",
            "epic crowd",
        ],
    ) {
        candidates.push(InternalStyleLaneCandidate {
            lane_id: InternalStyleLaneId::Director09,
            priority: 10,
            planning_signals: vec![
                CameraLanguagePlanningSignal::FormationDepth,
                CameraLanguagePlanningSignal::CommandCenter,
                CameraLanguagePlanningSignal::FlagDrumBridge,
                CameraLanguagePlanningSignal::CrowdRhythm,
            ],
        });
    }

    if contains_any(
        &evidence,
        &[
            "urban",
            "apocalypse",
            "industrial",
            "subway",
            "overpass",
            "ruins",
            "evacuation",
            "alarm",
            "infrastructure",
        ],
    ) {
        candidates.push(InternalStyleLaneCandidate {
            lane_id: InternalStyleLaneId::Director10,
            priority: 10,
            planning_signals: vec![
                CameraLanguagePlanningSignal::EvacuationAxis,
                CameraLanguagePlanningSignal::IndustrialCompression,
                CameraLanguagePlanningSignal::AlarmBridge,
                CameraLanguagePlanningSignal::ObstacleSource,
            ],
        });
    }

    if contains_any(
        &evidence,
        &[
            "stage",
            "group dance",
            "performance",
            "music",
            "spotlight",
            "entrance",
            "final freeze",
        ],
    ) {
        candidates.push(InternalStyleLaneCandidate {
            lane_id: InternalStyleLaneId::Director11,
            priority: 10,
            planning_signals: vec![
                CameraLanguagePlanningSignal::StageAxis,
                CameraLanguagePlanningSignal::FormationContinuity,
                CameraLanguagePlanningSignal::MusicSyncCue,
                CameraLanguagePlanningSignal::FreezeFramePlanning,
            ],
        });
    }

    if contains_any(
        &evidence,
        &["action", "fight", "battle", "impact", "hot blood"],
    ) {
        candidates.push(InternalStyleLaneCandidate {
            lane_id: InternalStyleLaneId::Director01,
            priority: 5,
            planning_signals: vec![
                CameraLanguagePlanningSignal::ActionIntensity,
                CameraLanguagePlanningSignal::ImpactRhythm,
            ],
        });
    }

    if contains_any(&evidence, &["close combat", "hand to hand", "body axis"]) {
        candidates.push(InternalStyleLaneCandidate {
            lane_id: InternalStyleLaneId::Director03,
            priority: 5,
            planning_signals: vec![CameraLanguagePlanningSignal::AttackChainClarity],
        });
    }

    if contains_any(
        &evidence,
        &["crowd pressure", "crisis", "danger", "compression"],
    ) {
        candidates.push(InternalStyleLaneCandidate {
            lane_id: InternalStyleLaneId::Director02,
            priority: 5,
            planning_signals: vec![
                CameraLanguagePlanningSignal::CrowdPressure,
                CameraLanguagePlanningSignal::SpaceCompression,
                CameraLanguagePlanningSignal::DangerAdvance,
            ],
        });
    }

    if contains_any(&evidence, &["emotion", "landscape", "weather", "light"]) {
        candidates.push(InternalStyleLaneCandidate {
            lane_id: InternalStyleLaneId::Director04,
            priority: 4,
            planning_signals: vec![
                CameraLanguagePlanningSignal::EnvironmentEmotionSupport,
                CameraLanguagePlanningSignal::LightWeatherTransition,
            ],
        });
    }

    if contains_any(
        &evidence,
        &[
            "micro performance",
            "quiet relation",
            "detail observation",
            "low viewpoint",
        ],
    ) {
        candidates.push(InternalStyleLaneCandidate {
            lane_id: InternalStyleLaneId::Director05,
            priority: 4,
            planning_signals: vec![
                CameraLanguagePlanningSignal::DetailObservation,
                CameraLanguagePlanningSignal::LowViewpointMicroAction,
            ],
        });
    }

    if contains_any(
        &evidence,
        &["dream", "subjective", "deformation", "psychological"],
    ) {
        candidates.push(InternalStyleLaneCandidate {
            lane_id: InternalStyleLaneId::Director06,
            priority: 4,
            planning_signals: vec![CameraLanguagePlanningSignal::SubjectiveDeformation],
        });
    }

    if contains_any(
        &evidence,
        &["suspense", "match cut", "reality displacement"],
    ) {
        candidates.push(InternalStyleLaneCandidate {
            lane_id: InternalStyleLaneId::Director07,
            priority: 4,
            planning_signals: vec![CameraLanguagePlanningSignal::RealityDisplacement],
        });
    }

    if candidates.is_empty() {
        candidates.push(InternalStyleLaneCandidate {
            lane_id: InternalStyleLaneId::Director02,
            priority: 1,
            planning_signals: vec![CameraLanguagePlanningSignal::CrowdPressure],
        });
    }

    candidates
}

fn collect_planning_signals(
    candidates: &[InternalStyleLaneCandidate],
) -> Vec<CameraLanguagePlanningSignal> {
    let mut signals = Vec::new();
    for candidate in candidates {
        for signal in &candidate.planning_signals {
            if !signals.contains(signal) {
                signals.push(*signal);
            }
        }
    }
    signals
}

fn combined_evidence(input: &V108ShotLanguageSelectionInput) -> String {
    let mut evidence = vec![
        input.style_cluster.as_str(),
        input.scene_category.as_str(),
        input.scene_tag.as_str(),
        input.camera_directing_core.as_str(),
        input.teaching_note.as_str(),
        input.ip_abstraction_note.as_str(),
        input.continuity_negative_core.as_str(),
    ]
    .join(" ");
    for value in &input.covered_points {
        evidence.push(' ');
        evidence.push_str(value);
    }
    for value in &input.missed_points {
        evidence.push(' ');
        evidence.push_str(value);
    }
    if matches!(input.sample_type, V108SampleType::SequenceShot) {
        evidence.push_str(" sequence shot");
    }
    if input.sequence_id.is_some() || input.shot_order.is_some() {
        evidence.push_str(" sequence continuity");
    }
    evidence.to_lowercase()
}

fn contains_any(evidence: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| evidence.contains(needle))
}

fn push_if<T>(items: &mut Vec<T>, condition: bool, item: T) {
    if condition {
        items.push(item);
    }
}
