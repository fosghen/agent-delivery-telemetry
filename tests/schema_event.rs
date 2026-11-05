use agent_delivery_telemetry::schema::{
    Actor, ActorKind, Event, EventKind, IdempotencyKey, Provider, Timestamp,
};

#[test]
fn event_records_actor_and_missing_source_time() {
    let actor = Actor {
        kind: ActorKind::Agent,
        provider: Some(Provider::OpenAi),
        model: Some("codex".to_owned()),
    };
    let event = Event {
        key: IdempotencyKey::try_from("event-1").unwrap(),
        kind: EventKind::StageStarted,
        actor,
        observed_at: Timestamp(10),
        source_at: None,
    };
    assert_eq!(event.actor.provider, Some(Provider::OpenAi));
    assert_eq!(event.source_at, None);
}
