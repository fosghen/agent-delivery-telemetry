use agent_delivery_telemetry::codex_usage::{
    TokenSnapshot, UsageCaptureError, exact_exec_usage, latest_snapshot, session_delta,
};
use agent_delivery_telemetry::comment::UsagePrecision;

#[test]
fn completed_exec_turn_becomes_exact_usage() {
    let trace = r#"{"type":"thread.started"}
{"type":"turn.completed","usage":{"input_tokens":13892,"cached_input_tokens":11264,"cache_write_input_tokens":0,"output_tokens":5,"reasoning_output_tokens":2}}"#;

    let usage = exact_exec_usage(trace).unwrap();

    assert_eq!(usage.precision, UsagePrecision::Exact);
    assert_eq!(usage.input_tokens, Some(13_892));
    assert_eq!(usage.cached_input_tokens, Some(11_264));
    assert_eq!(usage.output_tokens, Some(5));
    assert_eq!(usage.reasoning_tokens, Some(2));
    assert_eq!(usage.total_tokens, Some(13_897));
}

#[test]
fn incomplete_or_ambiguous_exec_stream_is_rejected() {
    let completed = r#"{"type":"turn.completed","usage":{"input_tokens":1,"cached_input_tokens":0,"output_tokens":2,"reasoning_output_tokens":0}}"#;

    assert_eq!(exact_exec_usage("{}"), Err(UsageCaptureError::Missing));
    assert_eq!(
        exact_exec_usage(&format!("{completed}\n{completed}")),
        Err(UsageCaptureError::Duplicate)
    );
    assert_eq!(
        exact_exec_usage("not json"),
        Err(UsageCaptureError::Malformed)
    );
}

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
