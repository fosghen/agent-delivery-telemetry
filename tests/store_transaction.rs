use agent_delivery_telemetry::schema::{IdempotencyKey, Timestamp, TraceId};
use agent_delivery_telemetry::store::{NewEvent, Store};

fn event(key: &'static str, payload: &'static str) -> NewEvent {
    NewEvent {
        trace_id: TraceId::try_from("trace-3").unwrap(),
        key: IdempotencyKey::try_from(key).unwrap(),
        observed_at: Timestamp(10),
        payload: payload.to_owned(),
    }
}

#[test]
fn failed_batch_rolls_back_every_event() {
    let mut store = Store::open_in_memory().unwrap();
    let result = store.append_batch(&[event("valid", "payload"), event("invalid", "")]);

    assert!(result.is_err());
    let events = store
        .load_trace(&TraceId::try_from("trace-3").unwrap())
        .unwrap();
    assert_eq!(events.len(), 0);
}
