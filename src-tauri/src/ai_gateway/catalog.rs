use super::providers::{Protocol, ProviderInput};
use crate::core::Result;
use rusqlite::Connection;

/// Built-in provider definitions. Only stable, publicly documented endpoints.
/// Model lists are intentionally NOT seeded: the user adds models they actually
/// have access to (or imports them from a `/models` listing in a later milestone).
/// This avoids shipping stale or fabricated model IDs and pricing.
pub struct BuiltinProviderDef {
    pub display_name: &'static str,
    pub protocol: Protocol,
    pub base_url: &'static str,
    pub needs_key: bool,
}

pub fn builtin_defs() -> Vec<BuiltinProviderDef> {
    vec![
        BuiltinProviderDef {
            display_name: "OpenAI",
            protocol: Protocol::OpenaiResponses,
            base_url: "https://api.openai.com/v1",
            needs_key: true,
        },
        BuiltinProviderDef {
            display_name: "Google Gemini (OpenAI-compatible)",
            protocol: Protocol::OpenaiCompletions,
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
            needs_key: true,
        },
        BuiltinProviderDef {
            display_name: "NVIDIA NIM",
            protocol: Protocol::OpenaiCompletions,
            base_url: "https://integrate.api.nvidia.com/v1",
            needs_key: true,
        },
        BuiltinProviderDef {
            display_name: "Cloudflare Workers AI (OpenAI-compatible)",
            protocol: Protocol::OpenaiCompletions,
            base_url: "https://api.cloudflare.com/client/v4/accounts/{ACCOUNT_ID}/ai/v1",
            needs_key: true,
        },
        BuiltinProviderDef {
            display_name: "DeepSeek",
            protocol: Protocol::OpenaiCompletions,
            base_url: "https://api.deepseek.com/v1",
            needs_key: true,
        },
        BuiltinProviderDef {
            display_name: "OpenRouter",
            protocol: Protocol::OpenaiCompletions,
            base_url: "https://openrouter.ai/api/v1",
            needs_key: true,
        },
        BuiltinProviderDef {
            display_name: "Ollama (local)",
            protocol: Protocol::Ollama,
            base_url: "http://localhost:11434",
            needs_key: false,
        },
        BuiltinProviderDef {
            display_name: "LM Studio (local)",
            protocol: Protocol::OpenaiCompletions,
            base_url: "http://localhost:1234/v1",
            needs_key: false,
        },
    ]
}

/// Seed the built-in providers (idempotent: keyed by display_name + kind).
/// Called at startup.
pub fn seed_builtins(conn: &Connection) -> Result<()> {
    for def in builtin_defs() {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM providers WHERE kind = 'builtin' AND display_name = ?1)",
                rusqlite::params![def.display_name],
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n != 0)?;
        if !exists {
            let input = ProviderInput {
                display_name: def.display_name.to_string(),
                protocol: def.protocol,
                base_url: Some(def.base_url.to_string()),
                extra_headers: None,
                extra_params: None,
                enabled: None,
            };
            input.validate()?;
            conn.execute(
                "INSERT INTO providers (id, kind, display_name, protocol, base_url, enabled)
                 VALUES (?1, 'builtin', ?2, ?3, ?4, 1)",
                rusqlite::params![
                    uuid::Uuid::new_v4().to_string(),
                    def.display_name,
                    serde_json::to_string(&def.protocol).unwrap(),
                    def.base_url
                ],
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage;

    #[test]
    fn seeding_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        storage::migrate_only(&conn);
        seed_builtins(&conn).unwrap();
        seed_builtins(&conn).unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM providers WHERE kind = 'builtin'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, builtin_defs().len() as i64);
    }

    #[test]
    fn builtin_urls_are_valid() {
        for def in builtin_defs() {
            let url = def.base_url;
            if url.contains("{ACCOUNT_ID}") {
                // Cloudflare template URL — validate with placeholder substituted.
                super::super::providers::validate_url(&url.replace("{ACCOUNT_ID}", "12345"))
                    .expect("cloudflare URL template must be valid");
            } else {
                super::super::providers::validate_url(url).expect("builtin URL must be valid");
            }
        }
    }
}
