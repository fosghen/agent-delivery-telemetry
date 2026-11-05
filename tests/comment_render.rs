use agent_delivery_telemetry::comment::{
    COMMENT_MARKER, TelemetryPayload, VerificationRecord, render_comment,
};

#[test]
fn rendered_comment_has_human_summary_and_machine_payload() {
    let payload = TelemetryPayload {
        schema_version: 1,
        attempts: Vec::new(),
        stages: Vec::new(),
        verification: VerificationRecord {
            outcome: "passed".to_owned(),
            checks: 3,
        },
        usage: Vec::new(),
    };

    let comment = render_comment(&payload).unwrap();

    assert!(comment.starts_with(COMMENT_MARKER));
    assert!(comment.contains("Verification: passed (3 checks)"));
    assert!(comment.contains("```json"));
    assert!(comment.contains("\"schema_version\": 1"));
}
