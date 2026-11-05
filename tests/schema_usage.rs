use agent_delivery_telemetry::schema::{
    Actor, ActorKind, MeasurementPrecision, Provider, TokenUsage,
};

#[test]
fn usage_keeps_precision_and_stage_attribution_explicit() {
    let usage = TokenUsage {
        actor: Actor {
            kind: ActorKind::Agent,
            provider: Some(Provider::OpenAi),
            model: Some("codex".to_owned()),
        },
        stage: None,
        precision: MeasurementPrecision::SessionTotal,
        input: Some(100),
        cached_input: Some(20),
        output: Some(30),
        reasoning: None,
        total: Some(130),
    };

    assert_eq!(usage.stage, None);
    assert_eq!(usage.precision, MeasurementPrecision::SessionTotal);
}
