use crate::core::{path_is_within, AppError, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A registered project (a folder on disk the user opened in ENKI).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub created_at: String,
    pub last_opened_at: String,
}

/// Validated payload for registering a project.
#[derive(Debug, Clone, Deserialize)]
pub struct NewProject {
    pub name: String,
    pub root_path: String,
}

impl NewProject {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::InvalidInput("project name is required".into()));
        }
        if self.name.len() > 120 {
            return Err(AppError::InvalidInput("project name too long".into()));
        }
        let p = Path::new(&self.root_path);
        if !p.is_absolute() {
            return Err(AppError::InvalidInput(
                "root_path must be absolute".into(),
            ));
        }
        if !p.exists() {
            return Err(AppError::InvalidInput(format!(
                "path does not exist: {}",
                self.root_path
            )));
        }
        if !p.is_dir() {
            return Err(AppError::InvalidInput(format!(
                "path is not a directory: {}",
                self.root_path
            )));
        }
        Ok(())
    }
}

pub fn create_project(conn: &Connection, input: &NewProject) -> Result<Project> {
    input.validate()?;
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO projects (id, name, root_path) VALUES (?1, ?2, ?3)",
        params![id, input.name.trim(), input.root_path],
    )?;
    get_project(conn, &id)
}

pub fn get_project(conn: &Connection, id: &str) -> Result<Project> {
    conn.query_row(
        "SELECT id, name, root_path, created_at, last_opened_at FROM projects WHERE id = ?1",
        params![id],
        map_project,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("project {id}")),
        other => other.into(),
    })
}

pub fn list_projects(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, root_path, created_at, last_opened_at
         FROM projects ORDER BY last_opened_at DESC",
    )?;
    let rows = stmt.query_map([], map_project)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Touch last_opened_at (called when a workspace is opened).
pub fn touch_project(conn: &Connection, id: &str) -> Result<()> {
    let n = conn.execute(
        "UPDATE projects SET last_opened_at = datetime('now') WHERE id = ?1",
        params![id],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(format!("project {id}")));
    }
    Ok(())
}

pub fn delete_project(conn: &Connection, id: &str) -> Result<()> {
    let n = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    if n == 0 {
        return Err(AppError::NotFound(format!("project {id}")));
    }
    Ok(())
}

/// Security gate: every filesystem operation the agent or app performs on a
/// project must pass through here. Returns Ok(root) when access is allowed.
pub fn check_project_path(
    conn: &Connection,
    project_id: &str,
    target: &Path,
    allow_outside_reads: bool,
) -> Result<std::path::PathBuf> {
    let project = get_project(conn, project_id)?;
    let root = Path::new(&project.root_path);
    if path_is_within(root, target) {
        return Ok(root.to_path_buf());
    }
    if allow_outside_reads && target.is_file() {
        // Audited read outside workspace — the caller must audit.
        return Ok(root.to_path_buf());
    }
    Err(AppError::InvalidInput(format!(
        "path is outside the project workspace: {}",
        target.display()
    )))
}

fn map_project(r: &rusqlite::Row) -> rusqlite::Result<Project> {
    Ok(Project {
        id: r.get(0)?,
        name: r.get(1)?,
        root_path: r.get(2)?,
        created_at: r.get(3)?,
        last_opened_at: r.get(4)?,
    })
}

// --- Sessions ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub engine_id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewSession {
    pub project_id: String,
    pub engine_id: String,
    pub title: Option<String>,
}

pub fn create_session(conn: &Connection, input: &NewSession) -> Result<Session> {
    if input.engine_id.trim().is_empty() {
        return Err(AppError::InvalidInput("engine_id is required".into()));
    }
    // Verify project exists (FK also enforces, but a clean error is better).
    get_project(conn, &input.project_id)?;
    let id = uuid::Uuid::new_v4().to_string();
    let title = input
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .unwrap_or("New session");
    conn.execute(
        "INSERT INTO sessions (id, project_id, engine_id, title) VALUES (?1, ?2, ?3, ?4)",
        params![id, input.project_id, input.engine_id, title],
    )?;
    get_session(conn, &id)
}

pub fn get_session(conn: &Connection, id: &str) -> Result<Session> {
    conn.query_row(
        "SELECT id, project_id, engine_id, title, created_at, updated_at
         FROM sessions WHERE id = ?1",
        params![id],
        map_session,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("session {id}")),
        other => other.into(),
    })
}

pub fn list_sessions(conn: &Connection, project_id: Option<&str>) -> Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, engine_id, title, created_at, updated_at
         FROM sessions WHERE (?1 IS NULL OR project_id = ?1)
         ORDER BY updated_at DESC LIMIT 200",
    )?;
    let rows = stmt.query_map(params![project_id], map_session)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn rename_session(conn: &Connection, id: &str, title: &str) -> Result<Session> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::InvalidInput("title cannot be empty".into()));
    }
    let n = conn.execute(
        "UPDATE sessions SET title = ?2, updated_at = datetime('now') WHERE id = ?1",
        params![id, title],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(format!("session {id}")));
    }
    get_session(conn, id)
}

pub fn delete_session(conn: &Connection, id: &str) -> Result<()> {
    let n = conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
    if n == 0 {
        return Err(AppError::NotFound(format!("session {id}")));
    }
    Ok(())
}

fn map_session(r: &rusqlite::Row) -> rusqlite::Result<Session> {
    Ok(Session {
        id: r.get(0)?,
        project_id: r.get(1)?,
        engine_id: r.get(2)?,
        title: r.get(3)?,
        created_at: r.get(4)?,
        updated_at: r.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage;

    fn mem() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        storage::migrate_only(&conn);
        conn
    }

    fn tmpdir(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("enki-ws-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn project_crud_and_ordering() {
        let conn = mem();
        let dir = tmpdir("crud");
        let p = create_project(
            &conn,
            &NewProject {
                name: "My Project".into(),
                root_path: dir.to_string_lossy().into(),
            },
        )
        .unwrap();
        assert!(!p.id.is_empty());
        assert_eq!(p.name, "My Project");
        let listed = list_projects(&conn).unwrap();
        assert_eq!(listed.len(), 1);
        touch_project(&conn, &p.id).unwrap();
        delete_project(&conn, &p.id).unwrap();
        assert!(list_projects(&conn).unwrap().is_empty());
    }

    #[test]
    fn rejects_relative_and_missing_paths() {
        let conn = mem();
        let e = create_project(
            &conn,
            &NewProject {
                name: "x".into(),
                root_path: "relative/path".into(),
            },
        )
        .unwrap_err();
        assert!(matches!(e, AppError::InvalidInput(_)));
        let e = create_project(
            &conn,
            &NewProject {
                name: "x".into(),
                root_path: "/definitely/not/existing/enki".into(),
            },
        )
        .unwrap_err();
        assert!(matches!(e, AppError::InvalidInput(_)));
    }

    #[test]
    fn path_boundary_enforced() {
        let conn = mem();
        let dir = tmpdir("boundary");
        let p = create_project(
            &conn,
            &NewProject {
                name: "B".into(),
                root_path: dir.to_string_lossy().into(),
            },
        )
        .unwrap();
        let inside = dir.join("src").join("main.rs");
        assert!(check_project_path(&conn, &p.id, &inside, false).is_ok());
        let outside = std::env::temp_dir().join("enki-outside-xyz");
        assert!(check_project_path(&conn, &p.id, &outside, false).is_err());
        // Traversal attempt
        let escape = dir.join("..").join("..").join("etc").join("passwd");
        assert!(check_project_path(&conn, &p.id, &escape, false).is_err());
    }

    #[test]
    fn session_crud() {
        let conn = mem();
        let dir = tmpdir("sess");
        let p = create_project(
            &conn,
            &NewProject {
                name: "S".into(),
                root_path: dir.to_string_lossy().into(),
            },
        )
        .unwrap();
        let s = create_session(
            &conn,
            &NewSession {
                project_id: p.id.clone(),
                engine_id: "opencode".into(),
                title: None,
            },
        )
        .unwrap();
        assert_eq!(s.title, "New session");
        let s2 = rename_session(&conn, &s.id, "Refactor auth").unwrap();
        assert_eq!(s2.title, "Refactor auth");
        assert_eq!(list_sessions(&conn, Some(&p.id)).unwrap().len(), 1);
        delete_session(&conn, &s.id).unwrap();
        assert!(list_sessions(&conn, Some(&p.id)).unwrap().is_empty());
        // Session with unknown project must fail
        let e = create_session(
            &conn,
            &NewSession {
                project_id: "nope".into(),
                engine_id: "opencode".into(),
                title: None,
            },
        )
        .unwrap_err();
        assert!(matches!(e, AppError::NotFound(_)));
    }
}
