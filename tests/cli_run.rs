use agent_delivery_telemetry::cli::{Action, CliError, CliOutcome, run};
use agent_delivery_telemetry::publish::{CommentPort, CommentSnapshot};

#[derive(Default)]
struct FakePort {
    comments: Vec<CommentSnapshot>,
    writes: usize,
}

impl CommentPort for FakePort {
    type Error = ();

    fn list(&mut self) -> Result<Vec<CommentSnapshot>, Self::Error> {
        Ok(self.comments.clone())
    }

    fn create(&mut self, _body: &str) -> Result<u64, Self::Error> {
        self.writes += 1;
        Ok(9)
    }

    fn update(&mut self, _id: u64, _body: &str) -> Result<(), Self::Error> {
        self.writes += 1;
        Ok(())
    }
}

const PAYLOAD: &str = r#"{
  "schema_version": 1,
  "head_sha": "abc",
  "attempts": [],
  "stages": [],
  "verification": {"outcome": "passed", "checks": 3},
  "usage": []
}"#;

#[test]
fn dry_run_reports_create_without_writing() {
    let mut port = FakePort::default();

    let outcome = run(&mut port, "fosghen", PAYLOAD, true).unwrap();

    assert_eq!(
        outcome,
        CliOutcome {
            action: Action::Create,
            id: None,
            dry_run: true,
        }
    );
    assert_eq!(port.writes, 0);
    assert_eq!(
        serde_json::to_string(&outcome).unwrap(),
        r#"{"action":"create","id":null,"dry_run":true}"#
    );
}

#[test]
fn live_run_returns_created_identifier() {
    let mut port = FakePort::default();

    assert_eq!(
        run(&mut port, "fosghen", PAYLOAD, false).unwrap(),
        CliOutcome {
            action: Action::Create,
            id: Some(9),
            dry_run: false,
        }
    );
    assert_eq!(port.writes, 1);
}

#[test]
fn inconsistent_token_total_is_rejected_before_publication() {
    let mut port = FakePort::default();
    let payload = PAYLOAD.replace(r#""usage": []"#, r#""usage": [{"actor":"primary","provider":"openai","model":null,"stage":null,"precision":"exact","input_tokens":1,"cached_input_tokens":null,"output_tokens":2,"reasoning_tokens":null,"total_tokens":4}]"#);

    assert_eq!(
        run(&mut port, "fosghen", &payload, false),
        Err(CliError::Input)
    );
    assert_eq!(port.writes, 0);
}
