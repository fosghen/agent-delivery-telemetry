use agent_delivery_telemetry::draft::WorkDraft;

#[test]
fn begin_records_only_task_identity_and_observed_start() {
    let draft = WorkDraft::begin(
        "fosghen/service",
        56,
        "2026-11-05T00:00:00Z",
    )
    .unwrap();

    assert_eq!(draft.schema_version, 1);
    assert_eq!(draft.repository, "fosghen/service");
    assert_eq!(draft.issue, 56);
    assert_eq!(draft.work_started_at, "2026-11-05T00:00:00Z");
    assert_eq!(draft.finished_at, None);
}

#[test]
fn finish_emits_publishable_payload_with_observed_duration() {
    let mut draft = WorkDraft::begin(
        "fosghen/service",
        56,
        "2026-11-05T00:00:00Z",
    )
    .unwrap();

    let payload = draft
        .finish("fosghen/service", "abc", 9, "2026-11-05T00:10:00Z")
        .unwrap();

    assert_eq!(payload.work_started_at.as_deref(), Some("2026-11-05T00:00:00Z"));
    assert_eq!(payload.head_sha, "abc");
    assert_eq!(payload.stages[0].duration_ms, Some(600_000));
    assert_eq!(payload.verification.checks, 9);
    assert_eq!(draft.finished_at.as_deref(), Some("2026-11-05T00:10:00Z"));
}
