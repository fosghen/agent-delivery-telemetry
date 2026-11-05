use agent_delivery_telemetry::codex_usage::TokenSnapshot;
use agent_delivery_telemetry::draft::{DraftError, TokenRefreshError, WorkDraft};

#[test]
fn begin_records_only_task_identity_and_observed_start() {
    let draft = WorkDraft::begin("fosghen/service", 56, "2026-11-05T00:00:00Z").unwrap();

    assert_eq!(draft.schema_version, 1);
    assert_eq!(draft.repository, "fosghen/service");
    assert_eq!(draft.issue, 56);
    assert_eq!(draft.work_started_at, "2026-11-05T00:00:00Z");
    assert_eq!(draft.finished_at, None);
}

#[test]
fn finish_emits_publishable_payload_with_observed_duration() {
    let mut draft = WorkDraft::begin("fosghen/service", 56, "2026-11-05T00:00:00Z").unwrap();

    let payload = draft
        .finish("fosghen/service", "abc", 9, "2026-11-05T00:10:00Z")
        .unwrap();

    assert_eq!(
        payload.work_started_at.as_deref(),
        Some("2026-11-05T00:00:00Z")
    );
    assert_eq!(payload.head_sha, "abc");
    assert_eq!(payload.stages[0].duration_ms, Some(600_000));
    assert_eq!(payload.verification.checks, 9);
    assert_eq!(draft.finished_at.as_deref(), Some("2026-11-05T00:10:00Z"));
}

#[test]
fn invalid_stale_mismatched_and_reused_drafts_fail() {
    assert_eq!(
        WorkDraft::begin("invalid", 1, "2026-11-05T00:00:00Z"),
        Err(DraftError::Invalid)
    );
    let mut draft = WorkDraft::begin("fosghen/service", 56, "2026-11-01T00:00:00Z").unwrap();
    assert_eq!(
        draft.finish("other/service", "abc", 1, "2026-11-01T00:01:00Z"),
        Err(DraftError::Mismatch)
    );
    assert_eq!(
        draft.finish("fosghen/service", "abc", 1, "2026-11-09T00:00:01Z"),
        Err(DraftError::Stale)
    );
    draft
        .finish("fosghen/service", "abc", 1, "2026-11-01T00:01:00Z")
        .unwrap();
    assert_eq!(
        draft.finish("fosghen/service", "abc", 1, "2026-11-01T00:02:00Z"),
        Err(DraftError::Finished)
    );
}

#[test]
fn draft_keeps_only_token_baseline_and_local_session_reference() {
    let mut draft = WorkDraft::begin("fosghen/service", 56, "2026-11-05T00:00:00Z").unwrap();
    draft.attach_token_source("session.jsonl", TokenSnapshot::new(100, 40, 20, 5, 120));

    let usage = draft
        .token_usage(
            r#"{"payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":160,"cached_input_tokens":70,"output_tokens":35,"reasoning_output_tokens":9,"total_tokens":195}}}}"#,
        )
        .unwrap();

    assert_eq!(draft.session_log.as_deref(), Some("session.jsonl"));
    assert_eq!(usage.total_tokens, Some(75));
}

#[test]
fn finished_payload_can_refresh_only_token_usage() {
    let mut draft = WorkDraft::begin("fosghen/service", 56, "2026-11-05T00:00:00Z").unwrap();
    draft.attach_token_source("session.jsonl", TokenSnapshot::new(100, 40, 20, 5, 120));
    let mut payload = draft
        .finish("fosghen/service", "abc", 9, "2026-11-05T00:10:00Z")
        .unwrap();
    let before = payload.clone();
    let session = r#"{"payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":160,"cached_input_tokens":70,"output_tokens":35,"reasoning_output_tokens":9,"total_tokens":195}}}}"#;

    draft.refresh_tokens(&mut payload, session).unwrap();

    assert_eq!(payload.usage[0].total_tokens, Some(75));
    assert_eq!(payload.work_started_at, before.work_started_at);
    assert_eq!(payload.stages, before.stages);
    assert_eq!(
        draft.refresh_tokens(&mut payload, session),
        Err(TokenRefreshError::AlreadyRefreshed)
    );
}
