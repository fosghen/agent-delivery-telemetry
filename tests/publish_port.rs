use agent_delivery_telemetry::comment::COMMENT_MARKER;
use agent_delivery_telemetry::publish::{CommentPort, CommentSnapshot, PublishOutcome, publish};

#[derive(Default)]
struct FakePort {
    comments: Vec<CommentSnapshot>,
    created: Option<String>,
    updated: Option<(u64, String)>,
}

impl CommentPort for FakePort {
    type Error = ();

    fn list(&mut self) -> Result<Vec<CommentSnapshot>, Self::Error> {
        Ok(self.comments.clone())
    }

    fn create(&mut self, body: &str) -> Result<u64, Self::Error> {
        self.created = Some(body.to_owned());
        Ok(9)
    }

    fn update(&mut self, id: u64, body: &str) -> Result<(), Self::Error> {
        self.updated = Some((id, body.to_owned()));
        Ok(())
    }
}

#[test]
fn publish_creates_absent_comment_through_port() {
    let mut port = FakePort::default();

    assert_eq!(
        publish(&mut port, "bot", "desired").unwrap(),
        PublishOutcome::Created { id: 9 }
    );
    assert_eq!(port.created.as_deref(), Some("desired"));
}

#[test]
fn publish_updates_changed_comment_and_skips_equal_comment() {
    let desired = format!("{COMMENT_MARKER}\nnew");
    let old = format!("{COMMENT_MARKER}\nold");
    let mut port = FakePort {
        comments: vec![CommentSnapshot {
            id: 7,
            author: "bot".to_owned(),
            body: old,
        }],
        ..FakePort::default()
    };
    assert_eq!(
        publish(&mut port, "bot", &desired).unwrap(),
        PublishOutcome::Updated { id: 7 }
    );
    assert_eq!(port.updated, Some((7, desired.clone())));

    let mut equal = FakePort {
        comments: vec![CommentSnapshot {
            id: 8,
            author: "bot".to_owned(),
            body: desired.clone(),
        }],
        ..FakePort::default()
    };
    assert_eq!(
        publish(&mut equal, "bot", &desired).unwrap(),
        PublishOutcome::Unchanged { id: 8 }
    );
}
