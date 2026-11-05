use crate::SCHEMA_VERSION;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchemaError {
    BlankIdentifier,
}
fn validate_identifier(value: &str) -> Result<String, SchemaError> {
    if value.trim().is_empty() {
        Err(SchemaError::BlankIdentifier)
    } else {
        Ok(value.to_owned())
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceId(String);

impl TraceId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<&str> for TraceId {
    type Error = SchemaError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_identifier(value).map(Self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for IdempotencyKey {
    type Error = SchemaError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_identifier(value).map(Self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Envelope<T> {
    pub schema_version: u16,
    pub payload: T,
}

impl<T> Envelope<T> {
    pub fn new(payload: T) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            payload,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Timestamp(pub i64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttemptOutcome {
    Succeeded,
    Failed,
    Abandoned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StageKind {
    Analysis,
    Implementation,
    Verification,
    Rework,
    Delivery,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stage {
    pub kind: StageKind,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
}

impl Stage {
    #[must_use]
    pub fn new(kind: StageKind) -> Self {
        Self {
            kind,
            started_at: None,
            finished_at: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attempt {
    pub number: u32,
    pub outcome: Option<AttemptOutcome>,
    pub stages: Vec<Stage>,
}

impl Attempt {
    #[must_use]
    pub fn new(number: u32) -> Self {
        Self {
            number,
            outcome: None,
            stages: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Provider {
    Local,
    GitHub,
    GitLab,
    OpenAi,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActorKind {
    Human,
    Agent,
    Automation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Actor {
    pub kind: ActorKind,
    pub provider: Option<Provider>,
    pub model: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    StageStarted,
    StageFinished,
    Verification,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub key: IdempotencyKey,
    pub kind: EventKind,
    pub actor: Actor,
    pub observed_at: Timestamp,
    pub source_at: Option<Timestamp>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewTrigger {
    SchemaMigration,
    Concurrency,
    Security,
    ExternalSideEffect,
    CorrectnessUncertainty,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewSpan {
    pub requested_at: Timestamp,
    pub completed_at: Option<Timestamp>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Review {
    NotRequested,
    Requested {
        trigger: ReviewTrigger,
        span: Option<ReviewSpan>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceOutcome {
    Merged,
    Abandoned,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trace {
    pub id: TraceId,
    pub attempts: Vec<Attempt>,
    pub events: Vec<Event>,
    pub review: Review,
    pub outcome: Option<TraceOutcome>,
}

impl Trace {
    #[must_use]
    pub fn new(id: TraceId) -> Self {
        Self {
            id,
            attempts: Vec::new(),
            events: Vec::new(),
            review: Review::NotRequested,
            outcome: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeasurementPrecision {
    Exact,
    SessionTotal,
    Estimated,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenUsage {
    pub actor: Actor,
    pub stage: Option<StageKind>,
    pub precision: MeasurementPrecision,
    pub input: Option<u64>,
    pub cached_input: Option<u64>,
    pub output: Option<u64>,
    pub reasoning: Option<u64>,
    pub total: Option<u64>,
}
