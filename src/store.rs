use std::path::Path;

use rusqlite::{Connection, Result};

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
}
