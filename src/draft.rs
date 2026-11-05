use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DraftError {
    Invalid,
    Mismatch,
    Stale,
    Finished,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkDraft {
    pub schema_version: u16,
    pub repository: String,
    pub issue: u64,
    pub work_started_at: String,
    pub finished_at: Option<String>,
}

impl WorkDraft {
    /// # Errors
    /// Returns an error for invalid task identity or timestamp.
    pub fn begin(repository: &str, issue: u64, now: &str) -> Result<Self, DraftError> {
        let valid_repository = repository
            .split_once('/')
            .is_some_and(|(owner, name)| !owner.is_empty() && !name.is_empty());
        if !valid_repository || issue == 0 || OffsetDateTime::parse(now, &Rfc3339).is_err() {
            return Err(DraftError::Invalid);
        }
        Ok(Self {
            schema_version: crate::SCHEMA_VERSION,
            repository: repository.to_owned(),
            issue,
            work_started_at: now.to_owned(),
            finished_at: None,
        })
    }
}
