use agent_delivery_telemetry::codex_usage::{
    TokenSnapshot, UsageBindError, UsageCaptureError, bind_exact_usage, exact_exec_usage,
    exact_exec_usage_with_provenance, latest_snapshot, session_delta, unavailable_usage,
};
use agent_delivery_telemetry::comment::{
    ReviewRecord, TelemetryPayload, UsagePrecision, VerificationRecord,
};

fn payload() -> TelemetryPayload {
    TelemetryPayload {
        schema_version: 1,
        head_sha: "abc".to_owned(),
        work_started_at: Some("2026-11-06T00:00:00Z".to_owned()),
        attempts: Vec::new(),
        stages: Vec::new(),
        verification: VerificationRecord {
            outcome: "passed".to_owned(),
            checks: 2,
        },
        review: ReviewRecord::NotRequested,
        usage: Vec::new(),
    }
}

fn artifact() -> agent_delivery_telemetry::comment::UsageRecord {
    exact_exec_usage(r#"{"type":"turn.completed","usage":{"input_tokens":3,"cached_input_tokens":1,"output_tokens":2,"reasoning_output_tokens":1}}"#).unwrap()
}

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
fn controlled_capture_preserves_explicit_model_provenance() {
    let trace = r#"{"type":"turn.completed","usage":{"input_tokens":3,"cached_input_tokens":1,"output_tokens":2,"reasoning_output_tokens":1}}"#;

    let usage = exact_exec_usage_with_provenance(trace, "gpt-6-luna", "medium").unwrap();

    assert_eq!(usage.model.as_deref(), Some("gpt-6-luna"));
    assert_eq!(usage.reasoning_effort.as_deref(), Some("medium"));
    assert_eq!(
        exact_exec_usage_with_provenance(trace, " ", "medium"),
        Err(UsageCaptureError::BlankProvenance)
    );
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
fn exact_artifact_binds_to_explicit_run_without_changing_other_fields() {
    let before = payload();
    let mut after = before.clone();
    after.usage.push(unavailable_usage());

    bind_exact_usage(&mut after, artifact(), "subagent-1", "implementation", 2).unwrap();

    assert_eq!(after.usage.len(), 1);
    assert_eq!(after.usage[0].actor, "subagent-1");
    assert_eq!(after.usage[0].stage.as_deref(), Some("implementation"));
    assert_eq!(after.usage[0].attempt, Some(2));
    assert_eq!(after.usage[0].total_tokens, Some(5));
    after.usage.clear();
    assert_eq!(after, before);
}

#[test]
fn distinct_runs_bind_but_invalid_identity_and_duplicates_do_not_mutate() {
    let mut payload = payload();
    bind_exact_usage(&mut payload, artifact(), "primary", "implementation", 1).unwrap();
    bind_exact_usage(&mut payload, artifact(), "subagent", "implementation", 1).unwrap();
    bind_exact_usage(&mut payload, artifact(), "primary", "verification", 1).unwrap();
    let before = payload.clone();

    assert_eq!(
        bind_exact_usage(&mut payload, artifact(), "primary", "implementation", 1),
        Err(UsageBindError::DuplicateIdentity)
    );
    assert_eq!(
        bind_exact_usage(&mut payload, artifact(), " ", "implementation", 2),
        Err(UsageBindError::BlankIdentity)
    );
    assert_eq!(
        bind_exact_usage(&mut payload, artifact(), "primary", "verification", 0),
        Err(UsageBindError::ZeroAttempt)
    );
    assert_eq!(payload, before);
}

#[test]
fn unavailable_or_non_capture_artifacts_are_rejected_without_mutation() {
    let mut payload = payload();
    let before = payload.clone();
    let mut unavailable = artifact();
    unavailable.precision = UsagePrecision::Unavailable;
    unavailable.input_tokens = None;
    unavailable.cached_input_tokens = None;
    unavailable.output_tokens = None;
    unavailable.reasoning_tokens = None;
    unavailable.total_tokens = None;

    assert_eq!(
        bind_exact_usage(&mut payload, unavailable, "primary", "implementation", 1),
        Err(UsageBindError::InvalidArtifact)
    );
    let mut attributed = artifact();
    attributed.stage = Some("tampered".to_owned());
    assert_eq!(
        bind_exact_usage(&mut payload, attributed, "primary", "implementation", 1),
        Err(UsageBindError::InvalidArtifact)
    );
    assert_eq!(payload, before);
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
