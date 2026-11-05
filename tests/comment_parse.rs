use agent_delivery_telemetry::comment::{
    COMMENT_MARKER, CommentError, TelemetryPayload, VerificationRecord, parse_comment,
    render_comment,
};

fn payload() -> TelemetryPayload {
    TelemetryPayload {
        schema_version: 1,
        attempts: Vec::new(),
        stages: Vec::new(),
        verification: VerificationRecord {
            outcome: "passed".to_owned(),
            checks: 1,
        },
        usage: Vec::new(),
    }
}

#[test]
fn parser_extracts_rendered_payload() {
    let parsed = parse_comment(&render_comment(&payload()).unwrap()).unwrap();
    assert_eq!(parsed, payload());
}

#[test]
fn parser_rejects_missing_marker_and_malformed_json() {
    assert_eq!(
        parse_comment("ordinary comment"),
        Err(CommentError::MissingMarker)
    );
    let malformed = format!("{COMMENT_MARKER}\n```json\n{{\n```");
    assert_eq!(parse_comment(&malformed), Err(CommentError::MalformedJson));
}
