use agent_delivery_telemetry::schema::{IdempotencyKey, Timestamp, TraceId};
use agent_delivery_telemetry::store::{AppendOutcome, NewEvent, Store};

#[test]
fn duplicate_event_is_idempotent() {
    let mut store = Store::open_in_memory().unwrap();
    let event = NewEvent {
        trace_id: TraceId::try_from("trace-1").unwrap(),
        key: IdempotencyKey::try_from("event-1").unwrap(),
        observed_at: Timestamp(10),
        payload: "stage-started".to_owned(),
    };

    assert_eq!(store.append(&event).unwrap(), AppendOutcome::Inserted);
    assert_eq!(store.append(&event).unwrap(), AppendOutcome::Duplicate);
}
