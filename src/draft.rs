use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::codex_usage::{TokenSnapshot, UsageCaptureError, latest_snapshot, session_delta};
use crate::comment::{
    AttemptRecord, ReviewRecord, StageRecord, TelemetryPayload, UsagePrecision, UsageRecord,
    VerificationRecord,
};

const MAX_DRAFT_AGE_MS: u64 = 7 * 24 * 60 * 60 * 1_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DraftError {
    Invalid,
    Mismatch,
    Stale,
    Finished,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenRefreshError {
    Unfinished,
    Mismatch,
    AlreadyRefreshed,
    Usage(UsageCaptureError),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkDraft {
    pub schema_version: u16,
    pub repository: String,
    pub issue: u64,
    pub work_started_at: String,
    pub finished_at: Option<String>,
    #[serde(default)]
    pub head_sha: Option<String>,
    #[serde(default)]
    pub session_log: Option<String>,
    #[serde(default)]
    pub token_baseline: Option<TokenSnapshot>,
    #[serde(default)]
    pub require_tokens: bool,
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
            head_sha: None,
            session_log: None,
            token_baseline: None,
            require_tokens: false,
        })
    }

    pub fn attach_token_source(&mut self, session_log: &str, baseline: TokenSnapshot) {
        self.session_log = Some(session_log.to_owned());
        self.token_baseline = Some(baseline);
    }

    /// # Errors
    /// Returns an error when token metadata is missing, unchanged, or invalid.
    pub fn token_usage(&self, session_json_lines: &str) -> Result<UsageRecord, UsageCaptureError> {
        let baseline = self
            .token_baseline
            .as_ref()
            .ok_or(UsageCaptureError::Missing)?;
        session_delta(baseline, &latest_snapshot(session_json_lines)?)
    }

    /// # Errors
    /// Returns an error unless a matching finished payload can receive a fresh delta.
    pub fn refresh_tokens(
        &self,
        payload: &mut TelemetryPayload,
        session_json_lines: &str,
    ) -> Result<(), TokenRefreshError> {
        if self.finished_at.is_none() {
            return Err(TokenRefreshError::Unfinished);
        }
        if payload.work_started_at.as_deref() != Some(&self.work_started_at)
            || self.head_sha.as_deref() != Some(&payload.head_sha)
        {
            return Err(TokenRefreshError::Mismatch);
        }
        if payload
            .usage
            .iter()
            .any(|usage| usage.precision == UsagePrecision::SessionTotal)
        {
            return Err(TokenRefreshError::AlreadyRefreshed);
        }
        let usage = self
            .token_usage(session_json_lines)
            .map_err(TokenRefreshError::Usage)?;
        payload.usage.clear();
        payload.usage.push(usage);
        Ok(())
    }

    /// # Errors
    /// Returns an error when the draft cannot be safely bound to this result.
    pub fn finish(
        &mut self,
        repository: &str,
        head_sha: &str,
        checks: u32,
        now: &str,
    ) -> Result<TelemetryPayload, DraftError> {
        if repository != self.repository || head_sha.trim().is_empty() {
            return Err(DraftError::Mismatch);
        }
        if self.finished_at.is_some() {
            return Err(DraftError::Finished);
        }
        let start = OffsetDateTime::parse(&self.work_started_at, &Rfc3339)
            .map_err(|_| DraftError::Invalid)?;
        let finish = OffsetDateTime::parse(now, &Rfc3339).map_err(|_| DraftError::Invalid)?;
        let duration_ms = u64::try_from((finish - start).whole_milliseconds())
            .map_err(|_| DraftError::Invalid)?;
        if duration_ms > MAX_DRAFT_AGE_MS {
            return Err(DraftError::Stale);
        }
        self.finished_at = Some(now.to_owned());
        self.head_sha = Some(head_sha.to_owned());
        Ok(TelemetryPayload {
            schema_version: crate::SCHEMA_VERSION,
            head_sha: head_sha.to_owned(),
            work_started_at: Some(self.work_started_at.clone()),
            attempts: vec![AttemptRecord {
                number: 1,
                outcome: "succeeded".to_owned(),
            }],
            stages: vec![StageRecord {
                name: "implementation".to_owned(),
                outcome: "passed".to_owned(),
                duration_ms: Some(duration_ms),
            }],
            verification: VerificationRecord {
                outcome: "passed".to_owned(),
                checks,
            },
            review: ReviewRecord::NotRequested,
            usage: Vec::new(),
        })
    }
}
