use agent_delivery_telemetry::comment::{UsageError, UsagePrecision, UsageRecord, validate_usage};

fn usage(precision: UsagePrecision) -> UsageRecord {
    UsageRecord {
        actor: "primary".to_owned(),
        provider: "openai".to_owned(),
        model: Some("gpt".to_owned()),
        stage: None,
        precision,
        input_tokens: Some(10),
        cached_input_tokens: Some(4),
        output_tokens: Some(5),
        reasoning_tokens: Some(2),
        total_tokens: Some(15),
    }
}

#[test]
fn exact_usage_accepts_consistent_total_and_keeps_actor_separate() {
    let primary = usage(UsagePrecision::Exact);
    let mut reviewer = primary.clone();
    reviewer.actor = "reviewer".to_owned();

    assert_eq!(validate_usage(&[primary, reviewer]), Ok(()));
}

#[test]
fn usage_rejects_inconsistent_totals_and_stage_attributed_session_totals() {
    let mut inconsistent = usage(UsagePrecision::Exact);
    inconsistent.total_tokens = Some(16);
    assert_eq!(
        validate_usage(&[inconsistent]),
        Err(UsageError::InconsistentTotal)
    );

    let mut session = usage(UsagePrecision::SessionTotal);
    session.stage = Some("implementation".to_owned());
    assert_eq!(
        validate_usage(&[session]),
        Err(UsageError::SessionTotalHasStage)
    );
}

#[test]
fn unavailable_usage_cannot_contain_invented_counts() {
    let unavailable = usage(UsagePrecision::Unavailable);

    assert_eq!(
        validate_usage(&[unavailable]),
        Err(UsageError::UnavailableHasCounts)
    );
}
