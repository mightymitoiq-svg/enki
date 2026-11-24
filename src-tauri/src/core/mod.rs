use std::path::{Path, PathBuf};

/// Errors that can surface at the application boundary.
/// Serialized to the frontend as `{ error: "<code>", message: "<human readable>" }`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    InvalidInput(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("storage error: {0}")]
    Storage(#[from] rusqlite::Error),

    #[error("credential store error: {0}")]
    CredentialStore(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("network error: {0}")]
    Network(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let code = match self {
            AppError::InvalidInput(_) => "invalid_input",
            AppError::NotFound(_) => "not_found",
            AppError::Conflict(_) => "conflict",
            AppError::Storage(_) => "storage",
            AppError::CredentialStore(_) => "credential_store",
            AppError::Io(_) => "io",
            AppError::Network(_) => "network",
            AppError::Internal(_) => "internal",
        };
        let mut st = s.serialize_struct("AppError", 2)?;
        st.serialize_field("error", code)?;
        st.serialize_field("message", &self.to_string())?;
        st.end()
    }
}

/// Canonical application data layout:
/// `<base>/enki/enki.db`       — SQLite database
/// `<base>/enki/logs/`         — audit + run logs
/// `<base>/enki/credentials/`  — fallback credential store (only used when the OS
///                                keyring is unavailable; file is 0600)
pub struct AppPaths {
    pub root: PathBuf,
}

impl AppPaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Default root from the OS app-data convention (works without a Tauri app handle,
    /// so services remain testable headlessly).
    pub fn default_root() -> std::result::Result<PathBuf, String> {
        let base = dirs::data_dir().ok_or_else(|| "no OS data dir".to_string())?;
        Ok(base.join("enki"))
    }

    pub fn db_path(&self) -> PathBuf {
        self.root.join("enki.db")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    pub fn credentials_dir(&self) -> PathBuf {
        self.root.join("credentials")
    }

    pub fn ensure_all(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(self.logs_dir())?;
        Ok(())
    }
}

/// True when `path` is inside `base` (both are canonicalized first so `..` and
/// symlinks cannot escape the workspace). `base` itself counts as inside.
/// For a not-yet-existing target, the deepest existing ancestor is canonicalized
/// and the remaining components are appended — this is safe against traversal
/// (`..` is resolved before canonicalization) while still allowing writes to
/// new files inside the workspace.
pub fn path_is_within(base: &Path, path: &Path) -> bool {
    let Ok(canonical_base) = base.canonicalize() else {
        return false;
    };
    // Normalize lexically first: resolves `.` and `..` without requiring existence.
    let normalized: PathBuf = normalize_lexically(path);
    // Find deepest existing ancestor, canonicalize it, re-append the rest.
    let mut ancestor = normalized.clone();
    let mut suffix: Vec<std::ffi::OsString> = Vec::new();
    loop {
        match ancestor.canonicalize() {
            Ok(c) => {
                let mut resolved = c;
                for part in &suffix {
                    resolved.push(part);
                }
                return resolved.starts_with(&canonical_base);
            }
            Err(_) => match (ancestor.file_name(), ancestor.parent()) {
                (Some(name), Some(parent)) => {
                    suffix.insert(0, name.to_os_string());
                    ancestor = parent.to_path_buf();
                }
                _ => return false,
            },
        }
    }
}

/// Lexically normalize a path: drop `.` components and resolve `..` where possible.
fn normalize_lexically(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_layout() {
        let p = AppPaths::new("/tmp/enki-test-root");
        assert_eq!(p.db_path(), Path::new("/tmp/enki-test-root/enki.db"));
        assert_eq!(p.logs_dir(), Path::new("/tmp/enki-test-root/logs"));
    }

    #[test]
    fn within_accepts_base_and_children() {
        let base = std::env::temp_dir();
        assert!(path_is_within(&base, &base));
        assert!(path_is_within(&base, &base.join("child").join("grandchild")));
    }

    #[test]
    fn within_rejects_escaping_paths() {
        let base = std::env::temp_dir().join("enki-boundary-test-base");
        std::fs::create_dir_all(&base).unwrap();
        let outside = base.parent().unwrap().join("definitely-outside-enki");
        assert!(!path_is_within(&base, &outside));
        // Non-existent path far outside.
        assert!(!path_is_within(
            &base,
            &std::env::temp_dir().join("no-such-ancestor-xyz").join("a")
        ));
    }
}
