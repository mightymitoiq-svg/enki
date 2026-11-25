use crate::agent_runtime::{self, EngineInfo};
use crate::ai_gateway::catalog;
use crate::ai_gateway::connection_test::{self, ConnectionTestResult};
use crate::ai_gateway::credentials::{CredentialStore, OsCredentialStore};
use crate::ai_gateway::providers::{self, Model, ModelInput, Provider, ProviderInput};
use crate::audit;
use crate::core::{AppError, AppPaths, Result};
use crate::settings::AppSettings;
use crate::workspace::{self, NewProject, NewSession, Project, Session};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::{AppHandle, State};

/// Shared application state managed by Tauri.
pub struct AppState {
    pub conn: Mutex<Connection>,
    pub keyring: OsCredentialStore,
    pub engines: agent_runtime::EngineRegistry,
}

impl AppState {
    /// Build state for tests or headless use with a custom data root.
    pub fn with_root(root: &std::path::Path) -> Result<Self> {
        let paths = AppPaths::new(root);
        paths.ensure_all()?;
        let conn = crate::storage::open(&paths.db_path())?;
        catalog::seed_builtins(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            keyring: OsCredentialStore::new(),
            engines: agent_runtime::production_registry(),
        })
    }

    /// Build state from the OS default data location.
    pub fn from_default_dir() -> Result<Self> {
        let root = AppPaths::default_root().map_err(AppError::Internal)?;
        Self::with_root(&root)
    }
}

// ---------- settings ----------

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Result<AppSettings> {
    let conn = state.conn.lock().unwrap();
    AppSettings::load(&conn)
}

#[tauri::command]
fn save_settings(state: State<'_, AppState>, settings: AppSettings) -> Result<AppSettings> {
    let conn = state.conn.lock().unwrap();
    settings.save(&conn)?;
    audit::log(
        &conn,
        "app",
        "settings_saved",
        &serde_json::json!({ "theme": settings.theme, "font": settings.editor_font_size }),
    )?;
    AppSettings::load(&conn)
}

// ---------- projects ----------

#[tauri::command]
fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>> {
    let conn = state.conn.lock().unwrap();
    workspace::list_projects(&conn)
}

#[tauri::command]
fn create_project(state: State<'_, AppState>, input: NewProject) -> Result<Project> {
    let conn = state.conn.lock().unwrap();
    workspace::create_project(&conn, &input)
}

#[tauri::command]
fn open_project(state: State<'_, AppState>, project_id: String) -> Result<Project> {
    let conn = state.conn.lock().unwrap();
    workspace::touch_project(&conn, &project_id)?;
    workspace::get_project(&conn, &project_id)
}

#[tauri::command]
fn delete_project(state: State<'_, AppState>, project_id: String) -> Result<()> {
    let conn = state.conn.lock().unwrap();
    workspace::delete_project(&conn, &project_id)
}

// ---------- sessions ----------

#[tauri::command]
fn list_sessions(state: State<'_, AppState>, project_id: Option<String>) -> Result<Vec<Session>> {
    let conn = state.conn.lock().unwrap();
    workspace::list_sessions(&conn, project_id.as_deref())
}

#[tauri::command]
fn create_session(state: State<'_, AppState>, input: NewSession) -> Result<Session> {
    let conn = state.conn.lock().unwrap();
    workspace::create_session(&conn, &input)
}

#[tauri::command]
fn rename_session(
    state: State<'_, AppState>,
    session_id: String,
    title: String,
) -> Result<Session> {
    let conn = state.conn.lock().unwrap();
    workspace::rename_session(&conn, &session_id, &title)
}

#[tauri::command]
fn delete_session(state: State<'_, AppState>, session_id: String) -> Result<()> {
    let conn = state.conn.lock().unwrap();
    workspace::delete_session(&conn, &session_id)
}

// ---------- providers & models ----------

#[tauri::command]
fn list_providers(state: State<'_, AppState>) -> Result<Vec<Provider>> {
    let conn = state.conn.lock().unwrap();
    providers::list_providers(&conn, Some(&state.keyring))
}

#[tauri::command]
fn create_provider(state: State<'_, AppState>, input: ProviderInput) -> Result<Provider> {
    let conn = state.conn.lock().unwrap();
    providers::create_provider(&conn, &input)
}

#[tauri::command]
fn update_provider(
    state: State<'_, AppState>,
    provider_id: String,
    input: ProviderInput,
) -> Result<Provider> {
    let conn = state.conn.lock().unwrap();
    providers::update_provider(&conn, &provider_id, &input)
}

#[tauri::command]
fn delete_provider(state: State<'_, AppState>, provider_id: String) -> Result<()> {
    let conn = state.conn.lock().unwrap();
    // Remove the secret first so we never orphan credentials.
    state.keyring.delete_credential(&provider_id)?;
    providers::delete_provider(&conn, &provider_id)
}

#[tauri::command]
fn list_models(state: State<'_, AppState>, provider_id: Option<String>) -> Result<Vec<Model>> {
    let conn = state.conn.lock().unwrap();
    providers::list_models(&conn, provider_id.as_deref())
}

#[tauri::command]
fn upsert_model(state: State<'_, AppState>, input: ModelInput) -> Result<Model> {
    let conn = state.conn.lock().unwrap();
    providers::upsert_model(&conn, &input)
}

#[tauri::command]
fn delete_model(state: State<'_, AppState>, model_id: String) -> Result<()> {
    let conn = state.conn.lock().unwrap();
    providers::delete_model(&conn, &model_id)
}

#[tauri::command]
fn set_default_model(state: State<'_, AppState>, model_id: String) -> Result<()> {
    let conn = state.conn.lock().unwrap();
    providers::set_default_model(&conn, &model_id)
}

/// Store a provider API key in the OS credential store. The key never touches
/// the database, logs, or the frontend beyond this call.
#[tauri::command]
fn set_provider_api_key(
    state: State<'_, AppState>,
    provider_id: String,
    api_key: String,
) -> Result<()> {
    if api_key.len() > 4096 {
        return Err(AppError::InvalidInput("API key too long".into()));
    }
    // Verify the provider exists so we don't store orphan secrets.
    {
        let conn = state.conn.lock().unwrap();
        providers::get_provider(&conn, &provider_id, None)?;
    }
    state.keyring.set_credential(&provider_id, &api_key)?;
    let conn = state.conn.lock().unwrap();
    audit::log(
        &conn,
        "security",
        "api_key_stored",
        &serde_json::json!({ "provider_id": provider_id }),
    )
}

#[tauri::command]
fn delete_provider_api_key(state: State<'_, AppState>, provider_id: String) -> Result<()> {
    state.keyring.delete_credential(&provider_id)?;
    let conn = state.conn.lock().unwrap();
    audit::log(
        &conn,
        "security",
        "api_key_removed",
        &serde_json::json!({ "provider_id": provider_id }),
    )
}

#[tauri::command]
fn test_provider_connection(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<ConnectionTestResult> {
    let (protocol, base_url) = {
        let conn = state.conn.lock().unwrap();
        let provider = providers::get_provider(&conn, &provider_id, None)?;
        (provider.protocol, provider.base_url.unwrap_or_default())
    };
    let result =
        connection_test::test_connection_for(&protocol, &base_url, &state.keyring, &provider_id)?;
    let conn = state.conn.lock().unwrap();
    audit::log(
        &conn,
        "gateway",
        "connection_tested",
        &serde_json::json!({ "provider_id": provider_id, "ok": result.ok }),
    )?;
    Ok(result)
}

// ---------- agent engines ----------

#[tauri::command]
fn discover_engines(state: State<'_, AppState>) -> Result<Vec<EngineInfo>> {
    Ok(state.engines.discover_all())
}

// ---------- audit ----------

#[tauri::command]
fn recent_audit(state: State<'_, AppState>, limit: Option<u32>) -> Result<Vec<audit::AuditEntry>> {
    let conn = state.conn.lock().unwrap();
    audit::recent(&conn, limit.unwrap_or(50))
}

// ---------- app info ----------

#[derive(Deserialize)]
struct CheckPathArgs {
    project_id: String,
    path: String,
    write: bool,
}

/// Security gate used by the frontend to preview whether the agent may access
/// a path. Reads honor `allow_reads_outside_workspace`; writes never go outside.
#[tauri::command]
fn check_workspace_path(
    state: State<'_, AppState>,
    args: CheckPathArgs,
) -> Result<serde_json::Value> {
    let conn = state.conn.lock().unwrap();
    let settings = AppSettings::load(&conn)?;
    let target = std::path::Path::new(&args.path);
    let allowed = workspace::check_project_path(
        &conn,
        &args.project_id,
        target,
        settings.allow_reads_outside_workspace && !args.write,
    )
    .is_ok();
    if !allowed {
        audit::log(
            &conn,
            "security",
            "path_denied",
            &serde_json::json!({ "project_id": args.project_id, "path": args.path, "write": args.write }),
        )?;
    }
    Ok(serde_json::json!({ "allowed": allowed }))
}

#[tauri::command]
fn app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

pub fn run() -> Result<()> {
    let state = AppState::from_default_dir()?;
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            list_projects,
            create_project,
            open_project,
            delete_project,
            list_sessions,
            create_session,
            rename_session,
            delete_session,
            list_providers,
            create_provider,
            update_provider,
            delete_provider,
            list_models,
            upsert_model,
            delete_model,
            set_default_model,
            set_provider_api_key,
            delete_provider_api_key,
            test_provider_connection,
            discover_engines,
            recent_audit,
            check_workspace_path,
            app_version,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| AppError::Internal(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> (AppState, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!("enki-cmd-test-{}", uuid::Uuid::new_v4()));
        let state = AppState::with_root(&root).unwrap();
        (state, root)
    }

    #[test]
    fn state_initializes_and_seeds() {
        let (state, root) = test_state();
        let conn = state.conn.lock().unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM providers WHERE kind = 'builtin'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 8);
        drop(conn);
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn schema_migrations_persist() {
        let root = std::env::temp_dir().join(format!("enki-persist-{}", uuid::Uuid::new_v4()));
        {
            let state = AppState::with_root(&root).unwrap();
            let conn = state.conn.lock().unwrap();
            let mut s = AppSettings::load(&conn).unwrap();
            s.editor_font_size = 18;
            s.save(&conn).unwrap();
        }
        // Simulate restart.
        {
            let state = AppState::with_root(&root).unwrap();
            let conn = state.conn.lock().unwrap();
            let s = AppSettings::load(&conn).unwrap();
            assert_eq!(s.editor_font_size, 18);
        }
        std::fs::remove_dir_all(root).ok();
    }
}
