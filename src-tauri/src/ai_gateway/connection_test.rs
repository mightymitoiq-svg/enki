use super::credentials::CredentialStore;
use super::providers::{validate_url, Protocol};
use crate::core::{AppError, Result};
use rusqlite::Connection;
use std::time::Duration;

/// Result of a provider connection test. Honest about what was checked.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectionTestResult {
    pub ok: bool,
    /// What was actually verified, e.g. "GET /models returned 200 with 14 models".
    pub detail: String,
    /// Number of models discovered if the endpoint supports listing.
    pub models_discovered: Option<usize>,
}

/// Test connectivity to a provider by hitting its model-listing endpoint.
/// This only verifies reachability + authentication, never that a model exists.
pub fn test_connection(
    conn: &Connection,
    keyring: &dyn CredentialStore,
    provider_id: &str,
) -> Result<ConnectionTestResult> {
    let (protocol, base_url): (Protocol, String) = conn
        .query_row(
            "SELECT protocol, COALESCE(base_url, '') FROM providers WHERE id = ?1",
            rusqlite::params![provider_id],
            |r| {
                let p: String = r.get(0)?;
                Ok((
                    serde_json::from_str(&p).unwrap_or(Protocol::OpenaiCompletions),
                    r.get(1)?,
                ))
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("provider {provider_id}"))
            }
            other => other.into(),
        })?;
    if base_url.is_empty() {
        return Ok(ConnectionTestResult {
            ok: false,
            detail: "provider has no base_url configured".into(),
            models_discovered: None,
        });
    }
    test_connection_for(&protocol, &base_url, keyring, provider_id)
}

/// Perform the network request after the caller has released any database lock.
pub fn test_connection_for(
    protocol: &Protocol,
    base_url: &str,
    keyring: &dyn CredentialStore,
    provider_id: &str,
) -> Result<ConnectionTestResult> {
    if base_url.is_empty() {
        return Ok(ConnectionTestResult {
            ok: false,
            detail: "provider has no base URL configured".into(),
            models_discovered: None,
        });
    }
    validate_url(base_url)?;
    let api_key = keyring.get_credential(provider_id)?;
    let parsed = url::Url::parse(base_url)
        .map_err(|_| AppError::InvalidInput("invalid provider URL".into()))?;
    let local_http = parsed.scheme() == "http"
        && matches!(
            parsed.host_str(),
            Some("localhost" | "127.0.0.1" | "[::1]" | "::1")
        );
    if api_key.is_some() && parsed.scheme() == "http" && !local_http {
        return Err(AppError::InvalidInput(
            "API keys require HTTPS. Plain HTTP is allowed only for localhost providers.".into(),
        ));
    }

    let (url, headers) = build_models_request(protocol, base_url, api_key.as_deref())?;
    let response = http_get_json(&url, &headers)?;
    let count = count_models(protocol, &response);
    Ok(ConnectionTestResult {
        ok: true,
        detail: format!(
            "model listing returned 200 ({})",
            summarize_status(&response)
        ),
        models_discovered: count,
    })
}

fn summarize_status(_r: &serde_json::Value) -> &'static str {
    "reachable"
}

/// Build the /models request per protocol. Returns (url, headers).
fn build_models_request(
    protocol: &Protocol,
    base_url: &str,
    api_key: Option<&str>,
) -> Result<(String, Vec<(String, String)>)> {
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut url = base_url.trim_end_matches('/').to_string();
    match protocol {
        Protocol::OpenaiCompletions | Protocol::OpenaiResponses => {
            url.push_str("/models");
            if let Some(k) = api_key {
                headers.push(("Authorization".into(), format!("Bearer {k}")));
            }
        }
        Protocol::AnthropicMessages => {
            url.push_str("/models");
            if let Some(k) = api_key {
                headers.push(("x-api-key".into(), k.to_string()));
            }
            headers.push(("anthropic-version".into(), "2023-06-01".into()));
        }
        Protocol::GoogleGenerative => {
            url.push_str("/models");
            if let Some(k) = api_key {
                headers.push(("x-goog-api-key".into(), k.to_string()));
            }
        }
        Protocol::Ollama => {
            url.push_str("/api/tags");
        }
    }
    Ok((url, headers))
}

/// Minimal HTTP GET using std TcpStream (http only after validation) or ureq-like
/// approach. To keep dependencies lean we shell out nothing and use a tiny
/// hand-rolled HTTP client over TLS via `native-tls`? That is heavy — instead we
/// use the `tauri` ecosystem standard: `reqwest` is common but pulls many deps.
/// Decision (see DECISIONS.md #4): use `ureq` — small, blocking, rustls.
fn http_get_json(url: &str, headers: &[(String, String)]) -> Result<serde_json::Value> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(30))
        .build();
    let mut req = agent.get(url);
    for (k, v) in headers {
        req = req.set(k, v);
    }
    let resp = req.call().map_err(|e| match e {
        ureq::Error::Status(code, _) => AppError::Network(format!("provider returned HTTP {code}")),
        ureq::Error::Transport(_) => {
            AppError::Network("could not reach the provider; check its URL and try again".into())
        }
    })?;
    let body = resp
        .into_string()
        .map_err(|_| AppError::Network("could not read the provider response".into()))?;
    serde_json::from_str(&body)
        .map_err(|_| AppError::Network("provider returned an invalid JSON response".into()))
}

fn count_models(protocol: &Protocol, body: &serde_json::Value) -> Option<usize> {
    match protocol {
        Protocol::OpenaiCompletions | Protocol::OpenaiResponses => {
            body.get("data").and_then(|d| d.as_array()).map(|a| a.len())
        }
        Protocol::AnthropicMessages => body.get("data").and_then(|d| d.as_array()).map(|a| a.len()),
        Protocol::GoogleGenerative => body
            .get("models")
            .and_then(|d| d.as_array())
            .map(|a| a.len()),
        Protocol::Ollama => body
            .get("models")
            .and_then(|d| d.as_array())
            .map(|a| a.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn models_request_building() {
        let (url, headers) = build_models_request(
            &Protocol::OpenaiCompletions,
            "https://api.example.com/v1/",
            Some("sk-test"),
        )
        .unwrap();
        assert_eq!(url, "https://api.example.com/v1/models");
        assert!(headers
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer sk-test"));

        let (url, _) =
            build_models_request(&Protocol::Ollama, "http://localhost:11434", None).unwrap();
        assert_eq!(url, "http://localhost:11434/api/tags");

        let (url, headers) = build_models_request(
            &Protocol::AnthropicMessages,
            "https://api.anthropic.com/v1",
            Some("sk-ant"),
        )
        .unwrap();
        assert_eq!(url, "https://api.anthropic.com/v1/models");
        assert!(headers.iter().any(|(k, _)| k == "anthropic-version"));
    }

    #[test]
    fn model_counting_per_protocol() {
        let openai_body = serde_json::json!({"data": [{"id": "a"}, {"id": "b"}]});
        assert_eq!(
            count_models(&Protocol::OpenaiCompletions, &openai_body),
            Some(2)
        );
        let ollama_body = serde_json::json!({"models": [{"name": "llama"}]});
        assert_eq!(count_models(&Protocol::Ollama, &ollama_body), Some(1));
        let empty = serde_json::json!({});
        assert_eq!(count_models(&Protocol::OpenaiResponses, &empty), None);
    }
}
