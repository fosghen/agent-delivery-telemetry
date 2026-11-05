use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryPayload {
    pub schema_version: u16,
    pub attempts: Vec<AttemptRecord>,
    pub stages: Vec<StageRecord>,
    pub verification: VerificationRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptRecord {
    pub number: u32,
    pub outcome: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageRecord {
    pub name: String,
    pub outcome: String,
    pub duration_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationRecord {
    pub outcome: String,
    pub checks: u32,
}
