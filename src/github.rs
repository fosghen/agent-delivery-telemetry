use std::io::Write;
use std::process::{Command, Stdio};

pub trait CommandRunner {
    type Error;

    /// # Errors
    ///
    /// Returns a command-execution error without exposing authentication data.
    fn run(&mut self, arguments: &[String], input: Option<&str>) -> Result<String, Self::Error>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GhCommandError {
    Start,
    Input,
    Failed(Option<i32>),
    Output,
}

#[derive(Default)]
pub struct SystemGh;

impl CommandRunner for SystemGh {
    type Error = GhCommandError;

    fn run(&mut self, arguments: &[String], input: Option<&str>) -> Result<String, Self::Error> {
        let mut child = Command::new("gh")
            .args(arguments)
            .stdin(if input.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| GhCommandError::Start)?;
        if let Some(value) = input {
            child
                .stdin
                .take()
                .ok_or(GhCommandError::Input)?
                .write_all(value.as_bytes())
                .map_err(|_| GhCommandError::Input)?;
        }
        let output = child
            .wait_with_output()
            .map_err(|_| GhCommandError::Output)?;
        if !output.status.success() {
            return Err(GhCommandError::Failed(output.status.code()));
        }
        String::from_utf8(output.stdout).map_err(|_| GhCommandError::Output)
    }
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

use crate::evidence::{CheckEvidence, CommitEvidence, PullRequestEvidence, ReviewEvidence};
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

#[derive(Deserialize)]
struct ApiHead {
    sha: String,
}

#[derive(Deserialize)]
struct ApiPullRequest {
    number: u64,
    state: String,
    user: ApiUser,
    head: ApiHead,
    created_at: String,
    merged_at: Option<String>,
    body: Option<String>,
}

#[derive(Deserialize)]
struct ApiCommitDate {
    date: Option<String>,
}

#[derive(Deserialize)]
struct ApiCommitMetadata {
    author: ApiCommitDate,
    committer: ApiCommitDate,
}

#[derive(Deserialize)]
struct ApiCommit {
    sha: String,
    commit: ApiCommitMetadata,
}

#[derive(Deserialize)]
struct ApiCheckPage {
    check_runs: Vec<ApiCheck>,
}

#[derive(Deserialize)]
struct ApiCheck {
    id: u64,
    name: String,
    status: String,
    conclusion: Option<String>,
    started_at: Option<String>,
    completed_at: Option<String>,
}

#[derive(Deserialize)]
struct ApiReview {
    id: u64,
    user: ApiUser,
    state: String,
    submitted_at: Option<String>,
}

fn closing_issues(body: Option<&str>) -> Vec<u64> {
    body.into_iter()
        .flat_map(str::lines)
        .filter(|line| {
            let line = line.to_ascii_lowercase();
            ["closes ", "fixes ", "resolves "]
                .iter()
                .any(|keyword| line.starts_with(keyword))
        })
        .flat_map(str::split_whitespace)
        .filter_map(|word| word.trim_matches([',', '.', ';']).strip_prefix('#'))
        .filter_map(|number| number.parse().ok())
        .collect()
}

impl<R: CommandRunner> GhCommentPort<R> {
    /// # Errors
    ///
    /// Returns a command or response-decoding error.
    pub fn review_evidence(&mut self) -> Result<Vec<ReviewEvidence>, GhError<R::Error>> {
        let endpoint = format!(
            "repos/{}/pulls/{}/reviews",
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
        let pages: Vec<Vec<ApiReview>> =
            serde_json::from_str(&output).map_err(|_| GhError::Decode)?;
        Ok(pages
            .into_iter()
            .flatten()
            .map(|value| ReviewEvidence {
                id: value.id,
                author: value.user.login,
                state: value.state,
                submitted_at: value.submitted_at,
            })
            .collect())
    }

    /// # Errors
    ///
    /// Returns a command or response-decoding error.
    pub fn check_evidence(
        &mut self,
        head_sha: &str,
    ) -> Result<Vec<CheckEvidence>, GhError<R::Error>> {
        let endpoint = format!("repos/{}/commits/{head_sha}/check-runs", self.repository);
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
        let pages: Vec<ApiCheckPage> =
            serde_json::from_str(&output).map_err(|_| GhError::Decode)?;
        Ok(pages
            .into_iter()
            .flat_map(|page| page.check_runs)
            .map(|value| CheckEvidence {
                id: value.id,
                name: value.name,
                status: value.status,
                conclusion: value.conclusion,
                started_at: value.started_at,
                completed_at: value.completed_at,
            })
            .collect())
    }

    /// # Errors
    ///
    /// Returns a command or response-decoding error.
    pub fn commit_evidence(&mut self) -> Result<Vec<CommitEvidence>, GhError<R::Error>> {
        let endpoint = format!(
            "repos/{}/pulls/{}/commits",
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
        let pages: Vec<Vec<ApiCommit>> =
            serde_json::from_str(&output).map_err(|_| GhError::Decode)?;
        pages
            .into_iter()
            .flatten()
            .map(|value| {
                Ok(CommitEvidence {
                    sha: value.sha,
                    authored_at: value.commit.author.date,
                    committed_at: value.commit.committer.date.ok_or(GhError::Decode)?,
                })
            })
            .collect()
    }

    /// # Errors
    ///
    /// Returns a command or response-decoding error.
    pub fn pull_request_evidence(
        &mut self,
    ) -> Result<(PullRequestEvidence, Vec<u64>), GhError<R::Error>> {
        let endpoint = format!("repos/{}/pulls/{}", self.repository, self.pull_request);
        let output = self
            .runner
            .run(&["api".to_owned(), endpoint], None)
            .map_err(GhError::Command)?;
        let value: ApiPullRequest = serde_json::from_str(&output).map_err(|_| GhError::Decode)?;
        let issues = closing_issues(value.body.as_deref());
        Ok((
            PullRequestEvidence {
                number: value.number,
                state: value.state,
                author: value.user.login,
                head_sha: value.head.sha,
                created_at: value.created_at,
                merged_at: value.merged_at,
            },
            issues,
        ))
    }

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
