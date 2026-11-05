use agent_delivery_telemetry::github::{CommandRunner, GhCommentPort};

struct FakeRunner {
    arguments: Vec<String>,
    input: Option<String>,
}

impl CommandRunner for FakeRunner {
    type Error = ();

    fn run(&mut self, arguments: &[String], input: Option<&str>) -> Result<String, Self::Error> {
        self.arguments = arguments.to_vec();
        self.input = input.map(str::to_owned);
        Ok(r#"{"id":9}"#.to_owned())
    }
}

#[test]
fn create_posts_json_to_issue_comments_endpoint() {
    let runner = FakeRunner {
        arguments: Vec::new(),
        input: None,
    };
    let mut port = GhCommentPort::new(runner, "fosghen/project", 42);

    assert_eq!(port.create_comment("body").unwrap(), 9);
    let runner = port.into_runner();

    assert!(runner.arguments.contains(&"POST".to_owned()));
    assert!(
        runner
            .arguments
            .contains(&"repos/fosghen/project/issues/42/comments".to_owned())
    );
    assert_eq!(runner.input.as_deref(), Some(r#"{"body":"body"}"#));
}

#[test]
fn update_patches_the_selected_issue_comment() {
    let runner = FakeRunner {
        arguments: Vec::new(),
        input: None,
    };
    let mut port = GhCommentPort::new(runner, "fosghen/project", 42);

    port.update_comment(7, "new").unwrap();
    let runner = port.into_runner();

    assert!(runner.arguments.contains(&"PATCH".to_owned()));
    assert!(
        runner
            .arguments
            .contains(&"repos/fosghen/project/issues/comments/7".to_owned())
    );
    assert_eq!(runner.input.as_deref(), Some(r#"{"body":"new"}"#));
}
