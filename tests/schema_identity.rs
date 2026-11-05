use agent_delivery_telemetry::SCHEMA_VERSION;
use agent_delivery_telemetry::schema::{Envelope, IdempotencyKey, TraceId};

#[test]
fn envelope_carries_current_schema_version() {
    let trace_id = TraceId::try_from("trace-7").expect("valid trace id");
    let envelope = Envelope::new(trace_id);

    assert_eq!(envelope.schema_version, SCHEMA_VERSION);
    assert_eq!(envelope.payload.as_str(), "trace-7");
}

#[test]
fn identifiers_reject_blank_values() {
    assert!(TraceId::try_from("  ").is_err());
    assert!(IdempotencyKey::try_from("").is_err());
}
