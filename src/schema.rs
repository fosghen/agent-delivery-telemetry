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
