use std::path::Path;

use rusqlite::{Connection, Result, params};

use crate::schema::{IdempotencyKey, Timestamp, TraceId};

const MIGRATION_1: &str = "
    CREATE TABLE schema_migrations (
        version INTEGER PRIMARY KEY
    );
    CREATE TABLE events (
        sequence INTEGER PRIMARY KEY AUTOINCREMENT,
        trace_id TEXT NOT NULL,
        idempotency_key TEXT NOT NULL UNIQUE,
        observed_at INTEGER NOT NULL,
        payload TEXT NOT NULL
    );
    INSERT INTO schema_migrations(version) VALUES (1);
";

pub struct Store {
    connection: Connection,
}

pub struct NewEvent {
    pub trace_id: TraceId,
    pub key: IdempotencyKey,
    pub observed_at: Timestamp,
    pub payload: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppendOutcome {
    Inserted,
    Duplicate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredEvent {
    pub key: String,
    pub observed_at: Timestamp,
    pub payload: String,
}

impl Store {
    /// # Errors
    ///
    /// Returns an error when SQLite cannot open or migrate the database.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_connection(Connection::open(path)?)
    }

    /// # Errors
    ///
    /// Returns an error when the in-memory database cannot be migrated.
    pub fn open_in_memory() -> Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(mut connection: Connection) -> Result<Self> {
        connection.pragma_update(None, "foreign_keys", true)?;
        let version: u32 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version == 0 {
            let transaction = connection.transaction()?;
            transaction.execute_batch(MIGRATION_1)?;
            transaction.pragma_update(None, "user_version", 1)?;
            transaction.commit()?;
        }
        Ok(Self { connection })
    }

    /// # Errors
    ///
    /// Returns an error when SQLite cannot read the schema version.
    pub fn migration_version(&self) -> Result<u32> {
        self.connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
    }

    /// # Errors
    ///
    /// Returns an error when SQLite cannot persist the event transaction.
    pub fn append(&mut self, event: &NewEvent) -> Result<AppendOutcome> {
        let transaction = self.connection.transaction()?;
        let inserted = transaction.execute(
            "INSERT OR IGNORE INTO events
             (trace_id, idempotency_key, observed_at, payload) VALUES (?1, ?2, ?3, ?4)",
            params![
                event.trace_id.as_str(),
                event.key.as_str(),
                event.observed_at.0,
                event.payload,
            ],
        )?;
        transaction.commit()?;
        Ok(if inserted == 1 {
            AppendOutcome::Inserted
        } else {
            AppendOutcome::Duplicate
        })
    }

    /// # Errors
    ///
    /// Returns an error when SQLite cannot query or decode stored events.
    pub fn load_trace(&self, trace_id: &TraceId) -> Result<Vec<StoredEvent>> {
        let mut statement = self.connection.prepare(
            "SELECT idempotency_key, observed_at, payload FROM events
             WHERE trace_id = ?1 ORDER BY observed_at, sequence",
        )?;
        let rows = statement.query_map([trace_id.as_str()], |row| {
            Ok(StoredEvent {
                key: row.get(0)?,
                observed_at: Timestamp(row.get(1)?),
                payload: row.get(2)?,
            })
        })?;
        rows.collect()
    }
}
