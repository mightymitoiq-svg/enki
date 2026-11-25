use super::credentials::CredentialStore;
use super::providers::{validate_url, Protocol};
use crate::core::{AppError, Result};
use rusqlite::Connection;

/// Result of a provider connection test. Honest about what was checked.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectionTestResult {
    pub ok: bool,
    /// What was actually verified, e.g. "GET /models returned 200 with 14 models".
    pub detail: String,
    /// Number of models discovered if the endpoint supports listing.
    pub models_discovered: Option<usize>,
}

/// Read a provider's endpoint configuration (protocol + base URL).
/// Split from the network call so the caller can release the database lock
/// before any HTTP traffic: a slow provider endpoint must not block unrelated
/// database operations (architecture review, backend concurrency).
pub fn load_provider_endpoint(
    conn: &Connection,
    provider_id: &str,
) -> Result<(Protocol, String)> {
    conn.query_row(
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
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("provider {provider_id}")),
        other => other.into(),
    })
}

/// Execute the connectivity test. Must be called WITHOUT any database lock held.
/// This only verifies reachability + authentication, never that a model exists.
pub fn run_test(
    protocol: &Protocol,
    base_url: &str,
    api_key: Option<&str>,
) -> Result<ConnectionTestResult> {
    if base_url.is_empty() {
        return Ok(ConnectionTestResult {
            ok: false,
            detail: "provider has no base_url configured".into(),
            models_discovered: None,
        });
    }
    validate_url(base_url)?;
    let (url, headers) = build_models_request(protocol, base_url, api_key)?;
    let response = http_get_json(&url, &headers)?;
    let count = count_models(protocol, &response);
    Ok(ConnectionTestResult {
        ok: true,
        detail: format!("model listing returned 200 ({})", summarize_status(&response)),
        models_discovered: count,
    })
}

/// Combined form for tests and headless use: read config, fetch the key, run.
/// Production commands use `load_provider_endpoint` + `run_test` separately so
/// the database lock is held only for the two short reads.
pub fn test_connection(
    conn: &Connection,
    keyring: &dyn CredentialStore,
    provider_id: &str,
) -> Result<ConnectionTestResult> {
    let (protocol, base_url) = load_provider_endpoint(conn, provider_id)?;
    let api_key = keyring.get_credential(provider_id)?;
    run_test(&protocol, &base_url, api_key.as_deref())
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
                // Query values must be percent-encoded: raw '&', '#', '+', '='
                // in a key would corrupt or truncate the credential. The `url`
                // crate handles exactly this for query pairs.
                let mut u = url::Url::parse(&url)
                    .map_err(|e| AppError::Internal(format!("invalid provider URL: {e}")))?;
                u.query_pairs_mut()
                    .append_pair("key", k);
                url = u.into();
            }
        }
        Protocol::Ollama => {
            url.push_str("/api/tags");
        }
    }
    Ok((url, headers))
}

/// Minimal HTTP GET using `ureq` — small, blocking, rustls (DECISIONS.md #4).
fn http_get_json(url: &str, headers: &[(String, String)]) -> Result<serde_json::Value> {
    let mut req = ureq::get(url);
    for (k, v) in headers {
        req = req.set(k, v);
    }
    let resp = req.call().map_err(|e| AppError::Network(scrub(&format!("{e}"))))?;
    let body = resp
        .into_string()
        .map_err(|e| AppError::Network(scrub(&format!("failed to read response: {e}"))))?;
    serde_json::from_str(&body)
        .map_err(|e| AppError::Network(scrub(&format!("invalid JSON from endpoint: {e}"))))
}

/// Remove credential material from error text. The Google protocol puts the
/// key in the URL (so transport errors echo it back); header-based protocols
/// never leak it, but scrubbing both costs nothing.
fn scrub(msg: &str) -> String {
    let mut out = msg.to_string();
    // key=... in URLs (Google protocol)
    if let Some(start) = out.find("key=") {
        let end = out[start..]
            .find(['&', ' ', '"', '\''])
            .map(|i| start + i)
            .unwrap_or(out.len());
        out.replace_range(start..end, "key=[redacted]");
    }
    // Bearer tokens echoed in headers — the search for the token's end must
    // start after the "Bearer " prefix, otherwise the prefix's own trailing
    // space terminates it and the token survives.
    if let Some(start) = out.find("Bearer ") {
        let from = start + "Bearer ".len();
        let end = out[from..]
            .find([' ', '"', '\''])
            .map(|i| from + i)
            .unwrap_or(out.len());
        out.replace_range(start..end, "Bearer [redacted]");
    }
    out
}

fn count_models(protocol: &Protocol, body: &serde_json::Value) -> Option<usize> {
    match protocol {
        Protocol::OpenaiCompletions | Protocol::OpenaiResponses => {
            body.get("data").and_then(|d| d.as_array()).map(|a| a.len())
        }
        Protocol::AnthropicMessages => body
            .get("data")
            .and_then(|d| d.as_array())
            .map(|a| a.len()),
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

        let (url, _) = build_models_request(&Protocol::Ollama, "http://localhost:11434", None).unwrap();
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
        assert_eq!(count_models(&Protocol::OpenaiCompletions, &openai_body), Some(2));
        let ollama_body = serde_json::json!({"models": [{"name": "llama"}]});
        assert_eq!(count_models(&Protocol::Ollama, &ollama_body), Some(1));
        let empty = serde_json::json!({});
        assert_eq!(count_models(&Protocol::OpenaiResponses, &empty), None);
    }

    #[test]
    fn google_key_is_percent_encoded() {
        // A key with URL-significant characters must survive round-tripping.
        let (url, headers) = build_models_request(
            &Protocol::GoogleGenerative,
            "https://generativelanguage.googleapis.com/v1beta",
            Some("a&b=c#d+e f"),
        )
        .unwrap();
        assert!(headers.is_empty(), "Google keys ride in the URL, not headers");
        assert_eq!(
            url,
            "https://generativelanguage.googleapis.com/v1beta/models?key=a%26b%3Dc%23d%2Be+f"
        );
        // And parse back to the original key.
        let parsed = url::Url::parse(&url).unwrap();
        assert_eq!(
            parsed.query_pairs().find(|(k, _)| k == "key").map(|(_, v)| v.to_string()),
            Some("a&b=c#d+e f".into())
        );
    }

    #[test]
    fn transport_policy_blocks_remote_http() {
        use super::super::providers::validate_url;
        assert!(validate_url("https://api.example.com/v1").is_ok());
        // Loopback http is allowed (Ollama, LM Studio, local gateways).
        assert!(validate_url("http://localhost:11434").is_ok());
        assert!(validate_url("http://127.0.0.1:1234/v1").is_ok());
        assert!(validate_url("http://[::1]:11434").is_ok());
        // Anything else over http is rejected — the request would carry the
        // API key without transport encryption.
        assert!(validate_url("http://api.example.com/v1").is_err());
        assert!(validate_url("http://192.168.1.5:8080/v1").is_err());
        assert!(validate_url("http://myserver.local/v1").is_err());
        assert!(validate_url("ftp://example.com").is_err());
    }

    #[test]
    fn error_text_is_scrubbed_of_credentials() {
        let scrubbed = scrub("http://host/models?key=SECRET&x=1: connection refused");
        assert!(!scrubbed.contains("SECRET"));
        assert!(scrubbed.contains("key=[redacted]"));
        let scrubbed = scrub("401 Unauthorized: Bearer sk-live-123 rejected");
        assert!(!scrubbed.contains("sk-live-123"));
        assert!(scrubbed.contains("Bearer [redacted]"));
        // Message without credentials passes through untouched.
        assert_eq!(scrub("timed out"), "timed out");
    }
}
