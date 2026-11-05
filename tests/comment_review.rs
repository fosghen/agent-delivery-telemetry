use agent_delivery_telemetry::comment::ReviewRecord;

#[test]
fn review_decision_is_separate_from_execution_details() {
    let requested = ReviewRecord::Requested {
        trigger: "correctness_uncertainty".to_owned(),
        requested_at: "2026-11-05T00:05:00Z".to_owned(),
        completed_at: Some("2026-11-05T00:08:00Z".to_owned()),
        outcome: Some("findings_confirmed".to_owned()),
        confirmed_findings: 2,
        rework_ms: Some(120_000),
    };

    let json = serde_json::to_string(&requested).unwrap();
    assert!(json.contains(r#""decision":"requested""#));
    assert_eq!(
        serde_json::from_str::<ReviewRecord>(&json).unwrap(),
        requested
    );
}

#[test]
fn not_requested_is_a_complete_valid_state() {
    assert_eq!(
        serde_json::to_string(&ReviewRecord::NotRequested).unwrap(),
        r#"{"decision":"not_requested"}"#
    );
}
