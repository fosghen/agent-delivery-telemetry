use agent_delivery_telemetry::codex_usage::{
    TokenSnapshot, UsageCaptureError, latest_snapshot, session_delta,
};

#[test]
fn latest_cumulative_codex_event_is_parsed() {
    let snapshot = latest_snapshot(
        r#"{"payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":100,"cached_input_tokens":40,"output_tokens":20,"reasoning_output_tokens":5,"total_tokens":120}}}}"#,
    )
    .unwrap();
    assert_eq!(snapshot, TokenSnapshot::new(100, 40, 20, 5, 120));
}

#[test]
fn monotonic_counters_become_an_unattributed_session_total() {
    let before = TokenSnapshot::new(100, 40, 20, 5, 120);
    let after = TokenSnapshot::new(160, 70, 35, 9, 195);

    let usage = session_delta(&before, &after).unwrap();

    assert_eq!(usage.input_tokens, Some(60));
    assert_eq!(usage.cached_input_tokens, Some(30));
    assert_eq!(usage.output_tokens, Some(15));
    assert_eq!(usage.reasoning_tokens, Some(4));
    assert_eq!(usage.total_tokens, Some(75));
    assert_eq!(usage.stage, None);
    assert_eq!(usage.attempt, None);
}

#[test]
fn unchanged_and_non_monotonic_counters_are_not_fabricated() {
    let before = TokenSnapshot::new(100, 40, 20, 5, 120);
    assert_eq!(
        session_delta(&before, &before),
        Err(UsageCaptureError::Unchanged)
    );
    assert_eq!(
        session_delta(&before, &TokenSnapshot::new(99, 40, 20, 5, 120)),
        Err(UsageCaptureError::NonMonotonic)
    );
    assert_eq!(latest_snapshot("not json"), Err(UsageCaptureError::Missing));
}
