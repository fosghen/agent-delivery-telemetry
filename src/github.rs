pub trait CommandRunner {
    type Error;

    /// # Errors
    ///
    /// Returns a command-execution error without exposing authentication data.
    fn run(&mut self, arguments: &[String], input: Option<&str>) -> Result<String, Self::Error>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GhError<E> {
    Command(E),
    Decode,
}

pub struct GhCommentPort<R> {
    runner: R,
    repository: String,
    pull_request: u64,
}

impl<R> GhCommentPort<R> {
    #[must_use]
    pub fn new(runner: R, repository: impl Into<String>, pull_request: u64) -> Self {
        Self {
            runner,
            repository: repository.into(),
            pull_request,
        }
    }

    #[must_use]
    pub fn into_runner(self) -> R {
        self.runner
    }

    #[must_use]
    pub fn repository(&self) -> &str {
        &self.repository
    }

    #[must_use]
    pub fn pull_request(&self) -> u64 {
        self.pull_request
    }
}
use serde::Deserialize;

use crate::publish::{CommentPort, CommentSnapshot};

#[derive(Deserialize)]
struct ApiUser {
    login: String,
}

#[derive(Deserialize)]
struct ApiComment {
    id: u64,
    user: ApiUser,
    body: String,
}

#[derive(Deserialize)]
struct ApiIdentifier {
    id: u64,
}

impl<R: CommandRunner> GhCommentPort<R> {
    /// # Errors
    ///
    /// Returns a command or response-decoding error.
    pub fn list_comments(&mut self) -> Result<Vec<CommentSnapshot>, GhError<R::Error>> {
        let endpoint = format!(
            "repos/{}/issues/{}/comments",
            self.repository, self.pull_request
        );
        let arguments = vec![
            "api".to_owned(),
            "--paginate".to_owned(),
            "--slurp".to_owned(),
            endpoint,
        ];
        let output = self
            .runner
            .run(&arguments, None)
            .map_err(GhError::Command)?;
        let pages: Vec<Vec<ApiComment>> =
            serde_json::from_str(&output).map_err(|_| GhError::Decode)?;
        Ok(pages
            .into_iter()
            .flatten()
            .map(|comment| CommentSnapshot {
                id: comment.id,
                author: comment.user.login,
                body: comment.body,
            })
            .collect())
    }

    /// # Errors
    ///
    /// Returns a command or response-decoding error.
    pub fn create_comment(&mut self, body: &str) -> Result<u64, GhError<R::Error>> {
        let endpoint = format!(
            "repos/{}/issues/{}/comments",
            self.repository, self.pull_request
        );
        let arguments = vec![
            "api".to_owned(),
            "--method".to_owned(),
            "POST".to_owned(),
            endpoint,
            "--input".to_owned(),
            "-".to_owned(),
        ];
        let input = serde_json::json!({ "body": body }).to_string();
        let output = self
            .runner
            .run(&arguments, Some(&input))
            .map_err(GhError::Command)?;
        let response: ApiIdentifier = serde_json::from_str(&output).map_err(|_| GhError::Decode)?;
        Ok(response.id)
    }

    /// # Errors
    ///
    /// Returns a command error when GitHub rejects the update.
    pub fn update_comment(&mut self, id: u64, body: &str) -> Result<(), GhError<R::Error>> {
        let endpoint = format!("repos/{}/issues/comments/{id}", self.repository);
        let arguments = vec![
            "api".to_owned(),
            "--method".to_owned(),
            "PATCH".to_owned(),
            endpoint,
            "--input".to_owned(),
            "-".to_owned(),
        ];
        let input = serde_json::json!({ "body": body }).to_string();
        self.runner
            .run(&arguments, Some(&input))
            .map_err(GhError::Command)?;
        Ok(())
    }
}

impl<R: CommandRunner> CommentPort for GhCommentPort<R> {
    type Error = GhError<R::Error>;

    fn list(&mut self) -> Result<Vec<CommentSnapshot>, Self::Error> {
        self.list_comments()
    }

    fn create(&mut self, body: &str) -> Result<u64, Self::Error> {
        self.create_comment(body)
    }

    fn update(&mut self, id: u64, body: &str) -> Result<(), Self::Error> {
        self.update_comment(id, body)
    }
}
