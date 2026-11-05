use agent_delivery_telemetry::evidence::CommitEvidence;
use agent_delivery_telemetry::github::{CommandRunner, GhCommentPort};

struct FixtureRunner;

impl CommandRunner for FixtureRunner {
    type Error = ();

    fn run(&mut self, arguments: &[String], _input: Option<&str>) -> Result<String, Self::Error> {
        assert_eq!(
            arguments,
            [
                "api",
                "--paginate",
                "--slurp",
                "repos/fosghen/project/pulls/42/commits"
            ]
        );
        Ok(r#"[[{"sha":"abc","commit":{"author":{"date":null},"committer":{"date":"2026-11-05T00:05:00Z"}}}],[{"sha":"def","commit":{"author":{"date":"2026-11-05T00:06:00Z"},"committer":{"date":"2026-11-05T00:07:00Z"}}}]]"#.to_owned())
    }
}

#[test]
fn paginated_commits_are_flattened_without_contents() {
    let mut port = GhCommentPort::new(FixtureRunner, "fosghen/project", 42);

    assert_eq!(
        port.commit_evidence().unwrap(),
        vec![
            CommitEvidence {
                sha: "abc".to_owned(),
                authored_at: None,
                committed_at: "2026-11-05T00:05:00Z".to_owned(),
            },
            CommitEvidence {
                sha: "def".to_owned(),
                authored_at: Some("2026-11-05T00:06:00Z".to_owned()),
                committed_at: "2026-11-05T00:07:00Z".to_owned(),
            },
        ]
    );
}
