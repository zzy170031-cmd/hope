#![forbid(unsafe_code)]

pub mod contract;
pub mod repair;
pub mod rules;
pub mod shared_fixture;
pub mod week3;

pub use contract::{
    ValidationDecision, ValidationEnvelope, ValidationFinding, ValidationReport,
    ValidationReportRow, ValidationSeverity, contains_placeholder_marker,
};
pub use repair::{RepairRecommendation, derive_repair_recommendations};
pub use rules::{
    ContinuityInput, ContinuityValidator, HandoffCoverageInput, HandoffCoverageValidator,
    HardLockInjectorGuard, HardLockInjectorGuardInput, LayerTraceabilityInput,
    LayerTraceabilityValidator, NegativeGlobalGuard, NegativeGlobalGuardInput,
    SegmentDurationInput, SegmentDurationValidator, StyleUnityInput, StyleUnityValidator,
};
pub use shared_fixture::Week3SharedFixture;
pub use week3::{
    WEEK3_SHARED_FIXTURE_PATH, WEEK3_VALIDATION_REPORT_PATH, collect_week3_validation_reports,
    generate_week3_repair_recommendations, generate_week3_validation_report,
    load_week3_shared_fixture, write_week3_validation_report,
};

pub const VALIDATION_SHEET_MACHINE_NAME: &str = "validation_report";
pub const VALIDATION_SHEET_INDEX: usize = 10;
pub const VALIDATION_SHEET_COLUMNS: &[&str] =
    &["校验报告标识", "项目标识", "通过", "问题数", "更新时间戳"];
