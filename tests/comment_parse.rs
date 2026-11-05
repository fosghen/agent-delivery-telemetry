use agent_delivery_telemetry::comment::{
    COMMENT_MARKER, CommentError, ReviewRecord, TelemetryPayload, VerificationRecord,
    parse_comment, render_comment,
};

fn payload() -> TelemetryPayload {
    TelemetryPayload {
        schema_version: 1,
        head_sha: "abc".to_owned(),
        attempts: Vec::new(),
        stages: Vec::new(),
        verification: VerificationRecord {
            outcome: "passed".to_owned(),
            checks: 1,
        },
        review: ReviewRecord::NotRequested,
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

#[test]
fn parser_rejects_ambiguous_or_unsupported_payloads() {
    let rendered = render_comment(&payload()).unwrap();
    let duplicate_marker = format!("{rendered}\n{COMMENT_MARKER}");
    assert_eq!(
        parse_comment(&duplicate_marker),
        Err(CommentError::DuplicateMarker)
    );

    let duplicate_payload = format!("{rendered}\n```json\n{{}}\n```");
    assert_eq!(
        parse_comment(&duplicate_payload),
        Err(CommentError::DuplicatePayload)
    );

    let mut future = payload();
    future.schema_version = 2;
    assert_eq!(
        parse_comment(&render_comment(&future).unwrap()),
        Err(CommentError::UnsupportedVersion(2)),
    );
}

#[test]
fn parser_rejects_blank_head_revision() {
    let mut value = payload();
    value.head_sha = " ".to_owned();
    assert_eq!(
        parse_comment(&render_comment(&value).unwrap()),
        Err(CommentError::BlankHeadRevision)
    );
}
