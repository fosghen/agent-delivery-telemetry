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
