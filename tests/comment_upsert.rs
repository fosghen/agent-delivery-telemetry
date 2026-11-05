use agent_delivery_telemetry::comment::COMMENT_MARKER;
use agent_delivery_telemetry::publish::{UpsertDecision, decide_upsert};

#[test]
fn absent_comment_is_created() {
    let desired = format!("{COMMENT_MARKER}\nnew");
    assert_eq!(
        decide_upsert(&[], "bot", &desired).unwrap(),
        UpsertDecision::Create { body: desired },
    );
}
