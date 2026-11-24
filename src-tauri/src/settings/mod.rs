use crate::core::{AppError, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// All persisted application settings, with defaults.
/// Frontend Zod schema mirrors this shape (see ui/src/lib/settingsSchema.ts).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    /// "dark" | "light" — ENKI is dark-first; light exists for accessibility.
    pub theme: String,
    /// Accent family: "electric" (blue/violet) — reserved for future options.
    pub accent: String,
    /// Editor font size in px.
    pub editor_font_size: i64,
    /// Show the agent "thinking" phase in the conversation view.
    pub show_agent_thoughts: bool,
    /// Ask before the agent runs any terminal command.
    pub confirm_terminal_commands: bool,
    /// Ask before the agent writes/patches files outside an approved batch.
    pub confirm_file_writes: bool,
    /// Allow the agent to read files outside the project root (still audited).
    pub allow_reads_outside_workspace: bool,
    /// Default provider id (empty = not chosen yet).
    pub default_provider_id: String,
    /// Default model id (empty = not chosen yet).
    pub default_model_id: String,
    /// Monthly budget in USD cents for gateway usage; 0 = unlimited.
    pub monthly_budget_usd_cents: i64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            accent: "electric".into(),
            editor_font_size: 14,
            show_agent_thoughts: true,
            confirm_terminal_commands: true,
            confirm_file_writes: true,
            allow_reads_outside_workspace: false,
            default_provider_id: String::new(),
            default_model_id: String::new(),
            monthly_budget_usd_cents: 0,
        }
    }
}

impl AppSettings {
    pub const KEY: &'static str = "app_settings";

    /// Validate invariants. Returns InvalidInput with a specific message.
    pub fn validate(&self) -> Result<()> {
        if !matches!(self.theme.as_str(), "dark" | "light") {
            return Err(AppError::InvalidInput(format!("unknown theme: {}", self.theme)));
        }
        if !matches!(self.accent.as_str(), "electric") {
            return Err(AppError::InvalidInput(format!("unknown accent: {}", self.accent)));
        }
        if !(10..=28).contains(&self.editor_font_size) {
            return Err(AppError::InvalidInput(
                "editor_font_size must be between 10 and 28".into(),
            ));
        }
        if self.monthly_budget_usd_cents < 0 {
            return Err(AppError::InvalidInput("budget cannot be negative".into()));
        }
        Ok(())
    }

    /// Load from the settings table; missing row → defaults.
    pub fn load(conn: &Connection) -> Result<Self> {
        let row: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![Self::KEY],
                |r| r.get(0),
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            })?;
        match row {
            Some(json) => {
                let mut s: AppSettings = serde_json::from_str(&json)
                    .map_err(|e| AppError::Internal(format!("corrupt settings JSON: {e}")))?;
                // Merge forward: fields added after the row was written keep defaults.
                let d = Self::default();
                s.theme = if s.theme.is_empty() { d.theme } else { s.theme };
                Ok(s)
            }
            None => Ok(Self::default()),
        }
    }

    /// Validate then persist atomically.
    pub fn save(&self, conn: &Connection) -> Result<()> {
        self.validate()?;
        conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
            params![Self::KEY, serde_json::to_string(self).unwrap()],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage;

    fn mem() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        storage::open_at(&conn);
        conn
    }

    #[test]
    fn defaults_round_trip() {
        let conn = mem();
        let s = AppSettings::load(&conn).unwrap();
        assert_eq!(s, AppSettings::default());
        s.save(&conn).unwrap();
        let loaded = AppSettings::load(&conn).unwrap();
        assert_eq!(s, loaded);
    }

    #[test]
    fn rejects_bad_values() {
        let bad_theme = AppSettings {
            theme: "neon".into(),
            ..AppSettings::default()
        };
        assert!(bad_theme.validate().is_err());
        let bad_font = AppSettings {
            editor_font_size: 99,
            ..AppSettings::default()
        };
        assert!(bad_font.validate().is_err());
        let bad_budget = AppSettings {
            monthly_budget_usd_cents: -5,
            ..AppSettings::default()
        };
        assert!(bad_budget.validate().is_err());
    }

    #[test]
    fn persists_changes() {
        let conn = mem();
        let mut s = AppSettings::load(&conn).unwrap();
        s.editor_font_size = 16;
        s.monthly_budget_usd_cents = 2000;
        s.save(&conn).unwrap();
        let loaded = AppSettings::load(&conn).unwrap();
        assert_eq!(loaded.editor_font_size, 16);
        assert_eq!(loaded.monthly_budget_usd_cents, 2000);
    }
}
