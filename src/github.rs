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

use crate::publish::CommentSnapshot;

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
}
