use agent_delivery_telemetry::schema::{IdempotencyKey, Timestamp, TraceId};
use agent_delivery_telemetry::store::{NewEvent, Store};

fn event(key: &'static str, observed_at: i64) -> NewEvent {
    NewEvent {
        trace_id: TraceId::try_from("trace-2").unwrap(),
        key: IdempotencyKey::try_from(key).unwrap(),
        observed_at: Timestamp(observed_at),
        payload: key.to_owned(),
    }
}

#[test]
fn trace_events_are_loaded_in_deterministic_order() {
    let mut store = Store::open_in_memory().unwrap();
    store.append(&event("later", 20)).unwrap();
    store.append(&event("earlier", 10)).unwrap();

    let events = store
        .load_trace(&TraceId::try_from("trace-2").unwrap())
        .unwrap();

    assert_eq!(events[0].key, "earlier");
    assert_eq!(events[1].key, "later");
}
