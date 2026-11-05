use agent_delivery_telemetry::comment::{
    AttemptRecord, StageRecord, TelemetryPayload, VerificationRecord,
};

#[test]
fn core_payload_round_trips_as_json() {
    let payload = TelemetryPayload {
        schema_version: 1,
        head_sha: "abc".to_owned(),
        attempts: vec![AttemptRecord {
            number: 1,
            outcome: "succeeded".to_owned(),
        }],
        stages: vec![StageRecord {
            name: "implementation".to_owned(),
            outcome: "passed".to_owned(),
            duration_ms: Some(1200),
        }],
        verification: VerificationRecord {
            outcome: "passed".to_owned(),
            checks: 4,
        },
        usage: Vec::new(),
    };

    let json = serde_json::to_string(&payload).unwrap();
    let decoded: TelemetryPayload = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded, payload);
}
