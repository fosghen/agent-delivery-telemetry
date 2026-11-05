use agent_delivery_telemetry::schema::{Attempt, AttemptOutcome, Stage, StageKind};

#[test]
fn attempt_represents_retry_rework_and_missing_timestamps() {
    let mut failed = Attempt::new(1);
    failed.outcome = Some(AttemptOutcome::Failed);

    let mut retry = Attempt::new(2);
    retry.stages.push(Stage::new(StageKind::Rework));
    retry.outcome = Some(AttemptOutcome::Abandoned);

    assert_eq!(failed.outcome, Some(AttemptOutcome::Failed));
    assert_eq!(retry.stages[0].started_at, None);
    assert_eq!(retry.stages[0].finished_at, None);
}
