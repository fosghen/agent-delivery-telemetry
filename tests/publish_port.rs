use agent_delivery_telemetry::publish::{CommentPort, CommentSnapshot, PublishOutcome, publish};

#[derive(Default)]
struct FakePort {
    comments: Vec<CommentSnapshot>,
    created: Option<String>,
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

    fn update(&mut self, _: u64, _: &str) -> Result<(), Self::Error> {
        unreachable!()
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
