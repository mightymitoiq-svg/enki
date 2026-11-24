use crate::core::Result;
use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AuditEntry {
    pub id: i64,
    pub ts: String,
    pub category: String,
    pub event: String,
    pub detail: String,
}

const CATEGORIES: &[&str] = &["security", "agent", "gateway", "app"];

pub fn log(conn: &Connection, category: &str, event: &str, detail: &serde_json::Value) -> Result<()> {
    debug_assert!(CATEGORIES.contains(&category), "unknown audit category");
    conn.execute(
        "INSERT INTO audit_log (category, event, detail) VALUES (?1, ?2, ?3)",
        params![category, event, detail.to_string()],
    )?;
    Ok(())
}

pub fn recent(conn: &Connection, limit: u32) -> Result<Vec<AuditEntry>> {
    let limit = limit.clamp(1, 500) as i64;
    let mut stmt = conn.prepare(
        "SELECT id, ts, category, event, detail FROM audit_log ORDER BY id DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |r| {
        Ok(AuditEntry {
            id: r.get(0)?,
            ts: r.get(1)?,
            category: r.get(2)?,
            event: r.get(3)?,
            detail: r.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage;

    #[test]
    fn log_and_query_recent() {
        let conn = Connection::open_in_memory().unwrap();
        storage::migrate_only(&conn);
        log(&conn, "security", "path_denied", &serde_json::json!({"path": "/etc/passwd"})).unwrap();
        log(&conn, "app", "settings_saved", &serde_json::json!({"theme": "dark"})).unwrap();
        let entries = recent(&conn, 10).unwrap();
        assert_eq!(entries.len(), 2);
        // DESC order: last inserted first.
        assert_eq!(entries[0].event, "settings_saved");
        assert_eq!(entries[1].category, "security");
        let one = recent(&conn, 1).unwrap();
        assert_eq!(one.len(), 1);
    }
}
