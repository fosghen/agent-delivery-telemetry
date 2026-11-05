use agent_delivery_telemetry::comment::{UsagePrecision, UsageRecord};

#[test]
fn token_usage_preserves_actor_model_and_precision() {
    let usage = UsageRecord {
        actor: "primary_agent".to_owned(),
        provider: "openai".to_owned(),
        model: Some("codex".to_owned()),
        stage: None,
        attempt: None,
        precision: UsagePrecision::SessionTotal,
        input_tokens: Some(100),
        cached_input_tokens: Some(20),
        output_tokens: Some(30),
        reasoning_tokens: None,
        total_tokens: Some(130),
    };

    let value = serde_json::to_value(usage).unwrap();

    assert_eq!(value["precision"], "session_total");
    assert_eq!(value["actor"], "primary_agent");
}
