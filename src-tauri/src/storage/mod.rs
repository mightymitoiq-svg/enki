use rusqlite::{params, Connection};
use std::path::Path;

/// Ordered schema migrations. Append new steps at the end; never edit old ones.
const MIGRATIONS: &[&str] = &[
    // 1 — core tables
    "CREATE TABLE settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );",
    "CREATE TABLE projects (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        root_path TEXT NOT NULL UNIQUE,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        last_opened_at TEXT NOT NULL DEFAULT (datetime('now'))
    );",
    "CREATE TABLE providers (
        id TEXT PRIMARY KEY,
        kind TEXT NOT NULL,              -- builtin | custom
        display_name TEXT NOT NULL,
        protocol TEXT NOT NULL,          -- openai-completions | openai-responses | anthropic-messages | google-generative | ollama
        base_url TEXT,
        extra_headers TEXT NOT NULL DEFAULT '{}',   -- JSON object
        extra_params TEXT NOT NULL DEFAULT '{}',    -- JSON object
        enabled INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );",
    "CREATE TABLE models (
        id TEXT PRIMARY KEY,
        provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
        model_id TEXT NOT NULL,          -- provider-side model identifier
        display_name TEXT NOT NULL,
        context_limit INTEGER,
        output_limit INTEGER,
        supports_streaming INTEGER NOT NULL DEFAULT 1,
        supports_tools INTEGER NOT NULL DEFAULT 0,
        supports_vision INTEGER NOT NULL DEFAULT 0,
        supports_json_schema INTEGER NOT NULL DEFAULT 0,
        is_default INTEGER NOT NULL DEFAULT 0,
        UNIQUE (provider_id, model_id)
    );",
    "CREATE TABLE sessions (
        id TEXT PRIMARY KEY,
        project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
        engine_id TEXT NOT NULL,
        title TEXT NOT NULL DEFAULT 'New session',
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );",
    "CREATE INDEX idx_models_provider ON models(provider_id);",
    "CREATE INDEX idx_sessions_project ON sessions(project_id);",
    // 2 — audit log
    "CREATE TABLE audit_log (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        ts TEXT NOT NULL DEFAULT (datetime('now')),
        category TEXT NOT NULL,          -- security | agent | gateway | app
        event TEXT NOT NULL,
        detail TEXT NOT NULL DEFAULT '{}'
    );",
    "CREATE INDEX idx_audit_ts ON audit_log(ts);",
];

/// Open (or create) the database at `path`, apply migrations, return the connection.
pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;
    let current: i64 = conn
        .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_migrations", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    for (idx, sql) in MIGRATIONS.iter().enumerate() {
        let version = (idx + 1) as i64;
        if version > current {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO schema_migrations (version) VALUES (?1)",
                params![version],
            )?;
        }
    }
    Ok(())
}

/// Apply migrations to an already-open connection (used by tests and by `open`).
pub fn migrate_only(conn: &Connection) {
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    migrate(conn).unwrap();
}

#[cfg(test)]
pub fn open_at(conn: &Connection) {
    migrate_only(conn);
}

/// Reset for tests: drop everything and re-migrate.
#[cfg(test)]
pub fn reset_for_test(conn: &Connection) -> rusqlite::Result<()> {
    for table in ["audit_log", "sessions", "models", "providers", "projects", "settings", "schema_migrations"] {
        conn.execute_batch(&format!("DROP TABLE IF EXISTS {table};"))?;
    }
    migrate(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_apply_and_are_idempotent() {
        let mut path = std::env::temp_dir();
        path.push(format!("enki-mig-test-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let conn = open(&path).unwrap();
        // Re-open: must not fail or duplicate.
        let conn2 = open(&path).unwrap();
        let count: i64 = conn2
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, MIGRATIONS.len() as i64);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_reset_round_trip() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_only(&conn);
        reset_for_test(&conn).unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM providers", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0);
    }
}
