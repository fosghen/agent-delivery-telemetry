use agent_delivery_telemetry::comment::{
    AttemptError, AttemptRecord, TelemetryPayload, record_failed_attempt,
};

fn payload() -> TelemetryPayload {
    serde_json::from_str(
        r#"{"schema_version":1,"head_sha":"abc","attempts":[{"number":1,"outcome":"succeeded"}],"stages":[],"verification":{"outcome":"passed","checks":1},"review":{"decision":"not_requested"},"usage":[]}"#,
    )
    .unwrap()
}

#[test]
fn each_failed_run_preserves_a_final_success_placeholder() {
    let mut payload = payload();

    assert_eq!(record_failed_attempt(&mut payload), Ok(1));
    assert_eq!(record_failed_attempt(&mut payload), Ok(2));
    assert_eq!(
        payload.attempts,
        vec![
            AttemptRecord {
                number: 1,
                outcome: "failed".to_owned()
            },
            AttemptRecord {
                number: 2,
                outcome: "failed".to_owned()
            },
            AttemptRecord {
                number: 3,
                outcome: "succeeded".to_owned()
            },
        ]
    );
}

#[test]
fn invalid_attempt_lifecycle_is_not_rewritten() {
    let mut payload = payload();
    payload.attempts[0].number = 2;
    let before = payload.clone();

    assert_eq!(
        record_failed_attempt(&mut payload),
        Err(AttemptError::InvalidLifecycle)
    );
    assert_eq!(payload, before);
}
