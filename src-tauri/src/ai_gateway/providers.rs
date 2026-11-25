use crate::core::{AppError, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Wire protocol a provider speaks. Drives request building in the gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Protocol {
    OpenaiCompletions,
    OpenaiResponses,
    AnthropicMessages,
    GoogleGenerative,
    Ollama,
}

/// A configured provider (built-in or custom OpenAI-compatible endpoint).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub kind: String, // "builtin" | "custom"
    pub display_name: String,
    pub protocol: Protocol,
    pub base_url: Option<String>,
    pub extra_headers: BTreeMap<String, String>,
    pub extra_params: BTreeMap<String, serde_json::Value>,
    pub enabled: bool,
    pub created_at: String,
    /// Whether an API key is present in the credential store (never the key itself).
    pub has_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub provider_id: String,
    pub model_id: String,
    pub display_name: String,
    pub context_limit: Option<i64>,
    pub output_limit: Option<i64>,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_json_schema: bool,
    pub is_default: bool,
}

/// Payload to create or update a custom provider.
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderInput {
    pub display_name: String,
    pub protocol: Protocol,
    pub base_url: Option<String>,
    pub extra_headers: Option<BTreeMap<String, String>>,
    pub extra_params: Option<BTreeMap<String, serde_json::Value>>,
    pub enabled: Option<bool>,
}

impl ProviderInput {
    pub fn validate(&self) -> Result<()> {
        if self.display_name.trim().is_empty() {
            return Err(AppError::InvalidInput("display_name is required".into()));
        }
        if self.display_name.len() > 80 {
            return Err(AppError::InvalidInput("display_name too long".into()));
        }
        if matches!(self.protocol, Protocol::Ollama) {
            // Ollama is local; base_url optional (default http://localhost:11434).
        } else {
            let url = self.base_url.as_deref().unwrap_or("");
            if url.is_empty() {
                return Err(AppError::InvalidInput(
                    "base_url is required for this protocol".into(),
                ));
            }
            validate_url(url)?;
        }
        for (k, v) in self.extra_headers.iter().flat_map(|h| h.iter()) {
            if k.trim().is_empty() {
                return Err(AppError::InvalidInput("header name cannot be empty".into()));
            }
            if v.len() > 2000 {
                return Err(AppError::InvalidInput("header value too long".into()));
            }
        }
        Ok(())
    }
}

pub fn validate_url(url: &str) -> Result<()> {
    let parsed = url::Url::parse(url)
        .map_err(|_| AppError::InvalidInput(format!("invalid URL: {url}")))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::InvalidInput(
            "only http/https URLs are allowed".into(),
        ));
    }
    // Transport policy: credentials are attached to requests against this URL,
    // so non-local endpoints must use HTTPS. Loopback (Ollama, LM Studio,
    // local gateways) never leaves the machine and is exempt.
    if parsed.scheme() == "http" && !is_loopback_host(parsed.host_str()) {
        return Err(AppError::InvalidInput(
            "insecure URL: HTTPS is required for non-local providers \
             (http is allowed only for localhost/127.0.0.1/::1)"
                .into(),
        ));
    }
    Ok(())
}

/// True for loopback hosts that never leave the machine.
pub fn is_loopback_host(host: Option<&str>) -> bool {
    match host {
        Some(h) => {
            let h = h.trim_end_matches('.').to_ascii_lowercase();
            h == "localhost" || h == "127.0.0.1" || h == "[::1]" || h == "::1"
        }
        None => false,
    }
}

/// Payload to create or update a model entry.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelInput {
    pub provider_id: String,
    pub model_id: String,
    pub display_name: String,
    pub context_limit: Option<i64>,
    pub output_limit: Option<i64>,
    pub supports_streaming: Option<bool>,
    pub supports_tools: Option<bool>,
    pub supports_vision: Option<bool>,
    pub supports_json_schema: Option<bool>,
}

impl ModelInput {
    pub fn validate(&self) -> Result<()> {
        if self.model_id.trim().is_empty() {
            return Err(AppError::InvalidInput("model_id is required".into()));
        }
        if self.display_name.trim().is_empty() {
            return Err(AppError::InvalidInput("display_name is required".into()));
        }
        if let Some(c) = self.context_limit {
            if c < 1024 {
                return Err(AppError::InvalidInput(
                    "context_limit looks too small (min 1024)".into(),
                ));
            }
        }
        if let Some(o) = self.output_limit {
            if o < 1 {
                return Err(AppError::InvalidInput("output_limit must be >= 1".into()));
            }
        }
        Ok(())
    }
}

// --- Provider CRUD ---

pub fn create_provider(conn: &Connection, input: &ProviderInput) -> Result<Provider> {
    input.validate()?;
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO providers (id, kind, display_name, protocol, base_url, extra_headers, extra_params, enabled)
         VALUES (?1, 'custom', ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            input.display_name.trim(),
            serde_json::to_string(&input.protocol).unwrap(),
            input.base_url,
            serde_json::to_string(&unwrap_or_empty(&input.extra_headers)).unwrap(),
            serde_json::to_string(&unwrap_or_empty(&input.extra_params)).unwrap(),
            input.enabled.unwrap_or(true) as i64,
        ],
    )?;
    get_provider(conn, &id, None)
}

pub fn update_provider(conn: &Connection, id: &str, input: &ProviderInput) -> Result<Provider> {
    input.validate()?;
    let n = conn.execute(
        "UPDATE providers SET display_name = ?2, protocol = ?3, base_url = ?4,
                extra_headers = ?5, extra_params = ?6, enabled = ?7
         WHERE id = ?1",
        params![
            id,
            input.display_name.trim(),
            serde_json::to_string(&input.protocol).unwrap(),
            input.base_url,
            serde_json::to_string(&unwrap_or_empty(&input.extra_headers)).unwrap(),
            serde_json::to_string(&unwrap_or_empty(&input.extra_params)).unwrap(),
            input.enabled.unwrap_or(true) as i64,
        ],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(format!("provider {id}")));
    }
    get_provider(conn, id, None)
}

pub fn delete_provider(conn: &Connection, id: &str) -> Result<()> {
    let n = conn.execute("DELETE FROM providers WHERE id = ?1", params![id])?;
    if n == 0 {
        return Err(AppError::NotFound(format!("provider {id}")));
    }
    Ok(())
}

/// Load one provider. `keyring` supplies API-key presence; pass None to report false.
pub fn get_provider(
    conn: &Connection,
    id: &str,
    keyring: Option<&dyn crate::ai_gateway::credentials::CredentialStore>,
) -> Result<Provider> {
    conn.query_row(
        "SELECT id, kind, display_name, protocol, base_url, extra_headers, extra_params, enabled, created_at
         FROM providers WHERE id = ?1",
        params![id],
        |r| map_provider(r, keyring),
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("provider {id}")),
        other => other.into(),
    })
}

pub fn list_providers(
    conn: &Connection,
    keyring: Option<&dyn crate::ai_gateway::credentials::CredentialStore>,
) -> Result<Vec<Provider>> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, display_name, protocol, base_url, extra_headers, extra_params, enabled, created_at
         FROM providers ORDER BY created_at",
    )?;
    let rows = stmt.query_map([], |r| map_provider(r, keyring))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn map_provider(
    r: &rusqlite::Row,
    keyring: Option<&dyn crate::ai_gateway::credentials::CredentialStore>,
) -> rusqlite::Result<Provider> {
    let protocol_json: String = r.get(3)?;
    let headers_json: String = r.get(5)?;
    let params_json: String = r.get(6)?;
    let id: String = r.get(0)?;
    Ok(Provider {
        has_api_key: keyring
            .map(|k| k.has_credential(&id))
            .unwrap_or(false),
        id,
        kind: r.get(1)?,
        display_name: r.get(2)?,
        protocol: serde_json::from_str(&protocol_json).unwrap_or(Protocol::OpenaiCompletions),
        base_url: r.get(4)?,
        extra_headers: serde_json::from_str(&headers_json).unwrap_or_default(),
        extra_params: serde_json::from_str(&params_json).unwrap_or_default(),
        enabled: r.get::<_, i64>(7)? != 0,
        created_at: r.get(8)?,
    })
}

fn unwrap_or_empty<T>(opt: &Option<BTreeMap<String, T>>) -> BTreeMap<String, T>
where
    T: Clone,
{
    opt.clone().unwrap_or_default()
}

// --- Model CRUD ---

pub fn upsert_model(conn: &Connection, input: &ModelInput) -> Result<Model> {
    input.validate()?;
    let provider_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM providers WHERE id = ?1)",
            params![input.provider_id],
            |r| r.get::<_, i64>(0),
        )
        .map(|n| n != 0)?;
    if !provider_exists {
        return Err(AppError::NotFound(format!(
            "provider {}",
            input.provider_id
        )));
    }
    let id = format!("{}:{}", input.provider_id, input.model_id);
    conn.execute(
        "INSERT INTO models (id, provider_id, model_id, display_name, context_limit, output_limit,
                supports_streaming, supports_tools, supports_vision, supports_json_schema, is_default)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0)
         ON CONFLICT(id) DO UPDATE SET display_name = ?4, context_limit = ?5, output_limit = ?6,
                supports_streaming = ?7, supports_tools = ?8, supports_vision = ?9, supports_json_schema = ?10",
        params![
            id,
            input.provider_id,
            input.model_id,
            input.display_name.trim(),
            input.context_limit,
            input.output_limit,
            input.supports_streaming.unwrap_or(true) as i64,
            input.supports_tools.unwrap_or(false) as i64,
            input.supports_vision.unwrap_or(false) as i64,
            input.supports_json_schema.unwrap_or(false) as i64,
        ],
    )?;
    get_model(conn, &id)
}

pub fn get_model(conn: &Connection, id: &str) -> Result<Model> {
    conn.query_row(
        "SELECT id, provider_id, model_id, display_name, context_limit, output_limit,
                supports_streaming, supports_tools, supports_vision, supports_json_schema, is_default
         FROM models WHERE id = ?1",
        params![id],
        map_model,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("model {id}")),
        other => other.into(),
    })
}

pub fn list_models(conn: &Connection, provider_id: Option<&str>) -> Result<Vec<Model>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider_id, model_id, display_name, context_limit, output_limit,
                supports_streaming, supports_tools, supports_vision, supports_json_schema, is_default
         FROM models WHERE (?1 IS NULL OR provider_id = ?1) ORDER BY provider_id, display_name",
    )?;
    let rows = stmt.query_map(params![provider_id], map_model)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn delete_model(conn: &Connection, id: &str) -> Result<()> {
    let n = conn.execute("DELETE FROM models WHERE id = ?1", params![id])?;
    if n == 0 {
        return Err(AppError::NotFound(format!("model {id}")));
    }
    Ok(())
}

/// Exactly one default model globally; clearing others is part of the same transaction.
pub fn set_default_model(conn: &Connection, id: &str) -> Result<()> {
    if get_model(conn, id).is_err() {
        return Err(AppError::NotFound(format!("model {id}")));
    }
    conn.execute("UPDATE models SET is_default = 0 WHERE is_default = 1", [])?;
    conn.execute(
        "UPDATE models SET is_default = 1 WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

fn map_model(r: &rusqlite::Row) -> rusqlite::Result<Model> {
    Ok(Model {
        id: r.get(0)?,
        provider_id: r.get(1)?,
        model_id: r.get(2)?,
        display_name: r.get(3)?,
        context_limit: r.get(4)?,
        output_limit: r.get(5)?,
        supports_streaming: r.get::<_, i64>(6)? != 0,
        supports_tools: r.get::<_, i64>(7)? != 0,
        supports_vision: r.get::<_, i64>(8)? != 0,
        supports_json_schema: r.get::<_, i64>(9)? != 0,
        is_default: r.get::<_, i64>(10)? != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage;

    fn mem() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        storage::migrate_only(&conn);
        conn
    }

    #[test]
    fn provider_validation() {
        let i = ProviderInput {
            display_name: "  ".into(),
            protocol: Protocol::OpenaiCompletions,
            base_url: Some("https://api.example.com/v1".into()),
            extra_headers: None,
            extra_params: None,
            enabled: None,
        };
        assert!(i.validate().is_err());

        let i = ProviderInput {
            display_name: "Custom LLM".into(),
            protocol: Protocol::OpenaiCompletions,
            base_url: Some("not-a-url".into()),
            extra_headers: None,
            extra_params: None,
            enabled: None,
        };
        assert!(i.validate().is_err());

        let i = ProviderInput {
            display_name: "Custom LLM".into(),
            protocol: Protocol::OpenaiCompletions,
            base_url: Some("https://api.example.com/v1".into()),
            extra_headers: None,
            extra_params: None,
            enabled: None,
        };
        assert!(i.validate().is_ok());
    }

    #[test]
    fn provider_and_model_crud() {
        let conn = mem();
        let p = create_provider(
            &conn,
            &ProviderInput {
                display_name: "My OpenAI-compat".into(),
                protocol: Protocol::OpenaiCompletions,
                base_url: Some("https://api.example.com/v1".into()),
                extra_headers: None,
                extra_params: None,
                enabled: None,
            },
        )
        .unwrap();
        assert_eq!(p.kind, "custom");
        assert!(!p.has_api_key);

        let m = upsert_model(
            &conn,
            &ModelInput {
                provider_id: p.id.clone(),
                model_id: "gpt-test".into(),
                display_name: "GPT Test".into(),
                context_limit: Some(128000),
                output_limit: Some(4096),
                supports_streaming: Some(true),
                supports_tools: Some(true),
                supports_vision: Some(false),
                supports_json_schema: Some(true),
            },
        )
        .unwrap();
        assert_eq!(m.id, format!("{}:gpt-test", p.id));
        // Upsert updates rather than duplicates.
        let m2 = upsert_model(
            &conn,
            &ModelInput {
                provider_id: p.id.clone(),
                model_id: "gpt-test".into(),
                display_name: "GPT Test 2".into(),
                context_limit: Some(200000),
                output_limit: None,
                supports_streaming: Some(true),
                supports_tools: None,
                supports_vision: None,
                supports_json_schema: None,
            },
        )
        .unwrap();
        assert_eq!(m2.display_name, "GPT Test 2");
        assert_eq!(list_models(&conn, Some(&p.id)).unwrap().len(), 1);

        // Default model: set and uniqueness.
        set_default_model(&conn, &m.id).unwrap();
        assert!(get_model(&conn, &m.id).unwrap().is_default);

        delete_model(&conn, &m.id).unwrap();
        delete_provider(&conn, &p.id).unwrap();
        assert!(list_providers(&conn, None).unwrap().is_empty());
    }

    #[test]
    fn url_validation() {
        assert!(validate_url("https://x.example.com").is_ok());
        assert!(validate_url("http://localhost:11434").is_ok());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("garbage").is_err());
    }
}
