use agent_delivery_telemetry::reconcile::ReconciledTrace;
use agent_delivery_telemetry::report::{REPORT_SCHEMA_VERSION, WarningKind, build_report};

#[test]
fn compatibility_fixture_produces_deterministic_versioned_json() {
    let trace: ReconciledTrace =
        serde_json::from_str(include_str!("fixtures/reconciled-v1.json")).unwrap();

    let report = build_report(vec![trace]).unwrap();
    let first = serde_json::to_string_pretty(&report).unwrap();
    let second = serde_json::to_string_pretty(&report).unwrap();

    assert_eq!(first, second);
    assert_eq!(report.schema_version, REPORT_SCHEMA_VERSION);
    assert_eq!(report.portfolio.tokens_per_merged_issue, Some(130));
    assert!(report.warnings.iter().any(|warning| {
        matches!(
            warning.kind,
            WarningKind::PartialEvidence { .. } | WarningKind::NonExactUsage { .. }
        )
    }));
}
