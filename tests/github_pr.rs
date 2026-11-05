use agent_delivery_telemetry::evidence::PullRequestEvidence;
use agent_delivery_telemetry::github::{CommandRunner, GhCommentPort};

struct FixtureRunner;

impl CommandRunner for FixtureRunner {
    type Error = ();

    fn run(&mut self, arguments: &[String], _input: Option<&str>) -> Result<String, Self::Error> {
        assert_eq!(arguments, ["api", "repos/fosghen/project/pulls/42"]);
        Ok(r#"{
          "number": 42,
          "state": "closed",
          "user": {"login": "fosghen"},
          "head": {"sha": "abc"},
          "created_at": "2026-11-05T00:00:00Z",
          "merged_at": "2026-11-05T00:10:00Z",
          "body": "Closes #7\nResolves #9"
        }"#
        .to_owned())
    }
}

#[test]
fn pull_request_metadata_and_closing_issues_are_extracted() {
    let mut port = GhCommentPort::new(FixtureRunner, "fosghen/project", 42);

    let (pull_request, issues) = port.pull_request_evidence().unwrap();

    assert_eq!(
        pull_request,
        PullRequestEvidence {
            number: 42,
            state: "closed".to_owned(),
            author: "fosghen".to_owned(),
            head_sha: "abc".to_owned(),
            created_at: "2026-11-05T00:00:00Z".to_owned(),
            merged_at: Some("2026-11-05T00:10:00Z".to_owned()),
        }
    );
    assert_eq!(issues, vec![7, 9]);
}
