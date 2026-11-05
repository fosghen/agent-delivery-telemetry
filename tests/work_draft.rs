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
