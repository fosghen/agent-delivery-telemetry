use agent_delivery_telemetry::metrics::{
    PortfolioMetrics, ReviewMetrics, StageMetric, TokenMetrics, TokenTotals, TraceAnalysis,
    TraceMetrics, aggregate_metrics,
};

fn analysis(duration: u64, tokens: u64, review: ReviewMetrics) -> TraceAnalysis {
    TraceAnalysis {
        linked_issues: 1,
        delivery: TraceMetrics {
            pull_request: duration,
            outcome: "closed".to_owned(),
            verification: "passed".to_owned(),
            delivery_lead_time_ms: Some(1_000),
            pr_cycle_time_ms: Some(500),
            active_time_ms: Some(duration),
            wait_time_ms: Some(1_000 - duration),
            retries: u32::from(duration == 100),
            rework_ms: Some(0),
            rework_share_bps: Some(0),
            review,
            stages: vec![StageMetric {
                name: "implementation".to_owned(),
                duration_ms: duration,
            }],
        },
        tokens: TokenMetrics {
            exact_totals: TokenTotals {
                total: Some(tokens),
                ..TokenTotals::default()
            },
            exact: vec![],
            session_totals: vec![],
            estimates: vec![],
            unavailable: vec![],
            cache_share_bps: None,
            failed_attempt_tokens: (duration == 100).then_some(130),
        },
    }
}

#[test]
fn portfolio_uses_nearest_rank_and_exact_coverage() {
    let requested = ReviewMetrics::Requested {
        duration_ms: Some(20),
        confirmed_findings: 2,
        rework_ms: Some(10),
    };
    let metrics: PortfolioMetrics = aggregate_metrics(&[
        analysis(100, 200, requested),
        analysis(300, 400, ReviewMetrics::NotRequested),
    ])
    .unwrap();

    assert_eq!(metrics.traces, 2);
    assert_eq!(metrics.merged_issues, 2);
    assert_eq!(metrics.tokens_per_merged_issue, Some(300));
    assert_eq!(metrics.failed_attempt_tokens, Some(130));
    assert_eq!(metrics.review_invocation_bps, Some(5_000));
    assert_eq!(metrics.confirmed_findings, 2);
    assert_eq!(metrics.findings_per_review_milli, Some(2_000));
    assert_eq!(metrics.stages[0].samples, 2);
    assert_eq!(metrics.stages[0].p50_ms, 100);
    assert_eq!(metrics.stages[0].p90_ms, 300);
    assert_eq!(metrics.stages[0].p95_ms, 300);
}
