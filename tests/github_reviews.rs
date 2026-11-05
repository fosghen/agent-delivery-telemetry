use agent_delivery_telemetry::evidence::ReviewEvidence;
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
                "repos/fosghen/project/pulls/42/reviews"
            ]
        );
        Ok(r#"[[{"id":10,"user":{"login":"reviewer"},"state":"APPROVED","submitted_at":null}],[]]"#.to_owned())
    }
}

#[test]
fn paginated_reviews_are_flattened() {
    let mut port = GhCommentPort::new(FixtureRunner, "fosghen/project", 42);

    assert_eq!(
        port.review_evidence().unwrap(),
        vec![ReviewEvidence {
            id: 10,
            author: "reviewer".to_owned(),
            state: "APPROVED".to_owned(),
            submitted_at: None,
        }]
    );
}
