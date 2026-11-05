use agent_delivery_telemetry::comment::COMMENT_MARKER;
use agent_delivery_telemetry::publish::{CommentSnapshot, UpsertDecision, decide_upsert};

fn comment(body: &str) -> CommentSnapshot {
    CommentSnapshot {
        id: 7,
        author: "bot".to_owned(),
        body: body.to_owned(),
    }
}

#[test]
fn absent_comment_is_created() {
    let desired = format!("{COMMENT_MARKER}\nnew");
    assert_eq!(
        decide_upsert(&[], "bot", &desired).unwrap(),
        UpsertDecision::Create { body: desired },
    );
}

#[test]
fn changed_comment_is_updated_and_equal_comment_is_unchanged() {
    let desired = format!("{COMMENT_MARKER}\nnew");
    let old = comment(&format!("{COMMENT_MARKER}\nold"));
    assert_eq!(
        decide_upsert(&[old], "bot", &desired).unwrap(),
        UpsertDecision::Update {
            id: 7,
            body: desired.clone()
        },
    );
    assert_eq!(
        decide_upsert(&[comment(&desired)], "bot", &desired).unwrap(),
        UpsertDecision::Unchanged { id: 7 },
    );
}

#[test]
fn selection_ignores_unowned_comments_and_rejects_multiple_owned_comments() {
    let desired = format!("{COMMENT_MARKER}\nnew");
    let mut unowned = comment(&desired);
    unowned.author = "other".to_owned();
    assert!(matches!(
        decide_upsert(&[unowned], "bot", &desired),
        Ok(UpsertDecision::Create { .. }),
    ));
    assert!(decide_upsert(&[comment(&desired), comment(&desired)], "bot", &desired).is_err());
}
