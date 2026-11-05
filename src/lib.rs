#![doc = "Core types for agent delivery telemetry."]

pub mod cli;
pub mod comment;
pub mod evidence;
pub mod github;
pub mod metrics;
pub mod publish;
pub mod reconcile;
pub mod schema;

/// Current persisted schema version.
pub const SCHEMA_VERSION: u16 = 1;

#[cfg(test)]
mod tests {
    #[test]
    fn schema_starts_at_one() {
        assert_eq!(super::SCHEMA_VERSION, 1);
    }
}
