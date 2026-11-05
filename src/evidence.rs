use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PullRequestEvidence {
    pub number: u64,
    pub state: String,
    pub author: String,
    pub head_sha: String,
    pub created_at: String,
    pub merged_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommitEvidence {
    pub sha: String,
    pub authored_at: Option<String>,
    pub committed_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckEvidence {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewEvidence {
    pub id: u64,
    pub author: String,
    pub state: String,
    pub submitted_at: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceVisibility {
    pub commits: bool,
    pub checks: bool,
    pub reviews: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    Commits,
    Checks,
    Reviews,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceFailureKind {
    Command,
    Decode,
    RateLimited,
    RetryExhausted,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceFailure {
    pub source: EvidenceSource,
    pub kind: EvidenceFailureKind,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSnapshot {
    pub pull_request: PullRequestEvidence,
    pub linked_issues: Vec<u64>,
    pub commits: Vec<CommitEvidence>,
    pub checks: Vec<CheckEvidence>,
    pub reviews: Vec<ReviewEvidence>,
    pub visibility: EvidenceVisibility,
    pub failures: Vec<EvidenceFailure>,
}
