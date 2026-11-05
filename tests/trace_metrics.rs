use agent_delivery_telemetry::comment::TelemetryPayload;
use agent_delivery_telemetry::evidence::EvidenceSnapshot;
use agent_delivery_telemetry::metrics::{
    ReviewMetrics, compute_token_metrics, compute_trace_metrics,
};
use agent_delivery_telemetry::reconcile::ReconciledTrace;

#[test]
fn trace_metrics_pair_time_with_outcomes() {
    let evidence: EvidenceSnapshot = serde_json::from_str(r#"{
      "pull_request":{"number":42,"state":"closed","author":"fosghen","head_sha":"abc","created_at":"2026-11-05T00:00:00Z","merged_at":"2026-11-05T00:00:10Z"},
      "linked_issues":[7],"commits":[],"checks":[],"reviews":[],
      "visibility":{"commits":true,"checks":true,"reviews":true},"failures":[]
    }"#).unwrap();
    let telemetry: TelemetryPayload = serde_json::from_str(r#"{
      "schema_version":1,"head_sha":"abc",
      "attempts":[{"number":1,"outcome":"failed"},{"number":2,"outcome":"succeeded"}],
      "stages":[{"name":"implementation","outcome":"passed","duration_ms":3000},{"name":"rework","outcome":"passed","duration_ms":2000}],
      "verification":{"outcome":"passed","checks":3},
      "review":{"decision":"requested","trigger":"correctness_uncertainty","requested_at":"2026-11-05T00:00:05Z","completed_at":"2026-11-05T00:00:08Z","outcome":"findings_confirmed","confirmed_findings":2,"rework_ms":120000},
      "usage":[
        {"actor":"primary","provider":"openai","model":"codex","stage":"implementation","attempt":1,"precision":"exact","input_tokens":100,"cached_input_tokens":20,"output_tokens":30,"reasoning_tokens":10,"total_tokens":130},
        {"actor":"primary","provider":"openai","model":"codex","stage":"verification","attempt":2,"precision":"exact","input_tokens":50,"cached_input_tokens":10,"output_tokens":20,"reasoning_tokens":null,"total_tokens":70},
        {"actor":"reviewer","provider":"openai","model":null,"stage":null,"attempt":null,"precision":"session_total","input_tokens":400,"cached_input_tokens":null,"output_tokens":100,"reasoning_tokens":null,"total_tokens":500},
        {"actor":"legacy","provider":"unknown","model":null,"stage":null,"attempt":null,"precision":"unavailable","input_tokens":null,"cached_input_tokens":null,"output_tokens":null,"reasoning_tokens":null,"total_tokens":null}
      ]
    }"#).unwrap();
    let trace = ReconciledTrace {
        evidence,
        comment_id: 9,
        telemetry,
    };

    let metrics = compute_trace_metrics(&trace).unwrap();

    assert_eq!(metrics.lead_time_ms, Some(10_000));
    assert_eq!(metrics.active_time_ms, Some(5_000));
    assert_eq!(metrics.wait_time_ms, Some(5_000));
    assert_eq!(metrics.retries, 1);
    assert_eq!(metrics.rework_ms, Some(2_000));
    assert_eq!(metrics.rework_share_bps, Some(4_000));
    assert_eq!(metrics.outcome, "closed");
    assert_eq!(metrics.verification, "passed");
    assert_eq!(
        metrics.review,
        ReviewMetrics::Requested {
            duration_ms: Some(3_000),
            confirmed_findings: 2,
            rework_ms: Some(120_000),
        }
    );
    let tokens = compute_token_metrics(&trace).unwrap();
    assert_eq!(tokens.exact_totals.total, Some(200));
    assert_eq!(tokens.cache_share_bps, Some(2_000));
    assert_eq!(tokens.failed_attempt_tokens, Some(130));
    assert_eq!(tokens.exact.len(), 2);
    assert_eq!(tokens.session_totals.len(), 1);
    assert_eq!(tokens.estimates.len(), 0);
    assert_eq!(tokens.unavailable.len(), 1);
}
