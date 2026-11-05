use agent_delivery_telemetry::github::{CommandRunner, GhCommentPort};

struct FakeRunner(String);

impl CommandRunner for FakeRunner {
    type Error = ();

    fn run(&mut self, _: &[String], _: Option<&str>) -> Result<String, Self::Error> {
        Ok(self.0.clone())
    }
}

#[test]
fn list_flattens_paginated_github_comments() {
    let output = r#"[[{"id":7,"user":{"login":"bot"},"body":"one"}],
                    [{"id":8,"user":{"login":"human"},"body":"two"}]]"#;
    let mut port = GhCommentPort::new(FakeRunner(output.to_owned()), "fosghen/project", 42);

    let comments = port.list_comments().unwrap();

    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].author, "bot");
}
