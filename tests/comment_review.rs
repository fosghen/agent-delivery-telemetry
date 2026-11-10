use agent_delivery_telemetry::comment::{
    ReviewError, ReviewRecord, TelemetryPayload, record_completed_review,
};

fn payload() -> TelemetryPayload {
    serde_json::from_str(
        r#"{"schema_version":1,"head_sha":"abc","attempts":[],"stages":[],"verification":{"outcome":"passed","checks":1},"review":{"decision":"not_requested"},"usage":[]}"#,
    )
    .unwrap()
}

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

#[test]
fn completed_review_replaces_the_explicit_decision() {
    let mut payload = payload();

    assert_eq!(
        record_completed_review(
            &mut payload,
            "correctness_uncertainty",
            "2026-11-05T00:05:00Z",
            "2026-11-05T00:08:00Z",
            "no_findings",
            0,
            0,
        ),
        Ok(())
    );
    assert!(matches!(payload.review, ReviewRecord::Requested { .. }));
}

#[test]
fn completed_review_rejects_invalid_or_duplicate_evidence() {
    let mut payload = payload();
    let record = |payload: &mut TelemetryPayload, requested, completed| {
        record_completed_review(payload, "risk", requested, completed, "clean", 0, 0)
    };

    assert_eq!(
        record(&mut payload, "bad", "also-bad"),
        Err(ReviewError::InvalidTimestamp)
    );
    assert_eq!(
        record(&mut payload, "2026-11-05T00:08:00Z", "2026-11-05T00:05:00Z"),
        Err(ReviewError::InvalidOrder)
    );
    assert_eq!(
        record(&mut payload, "2026-11-05T00:05:00Z", "2026-11-05T00:08:00Z"),
        Ok(())
    );
    assert_eq!(
        record(&mut payload, "2026-11-05T00:05:00Z", "2026-11-05T00:08:00Z"),
        Err(ReviewError::AlreadyRecorded)
    );
}
