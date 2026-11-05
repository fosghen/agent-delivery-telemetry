use agent_delivery_telemetry::codex_usage::{TokenSnapshot, latest_snapshot};

#[test]
fn latest_cumulative_codex_event_is_parsed() {
    let snapshot = latest_snapshot(
        r#"{"payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":100,"cached_input_tokens":40,"output_tokens":20,"reasoning_output_tokens":5,"total_tokens":120}}}}"#,
    )
    .unwrap();
    assert_eq!(snapshot, TokenSnapshot::new(100, 40, 20, 5, 120));
}
