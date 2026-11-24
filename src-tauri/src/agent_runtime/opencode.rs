use super::{EngineCapabilities, EngineInfo, EngineStatus, AgentEngineAdapter};
use std::process::Command;

/// Adapter for the OpenCode CLI (https://opencode.ai — SST project).
///
/// Discovery runs `opencode --version` on PATH. Capability claims below are
/// conservative and documented; the deep integration (session run + event
/// streaming via `opencode run --json`-style output) lands in milestone 2 —
/// until then this adapter only reports honest discovery state, and the UI
/// must not offer chat through it.
pub struct OpenCodeAdapter;

impl OpenCodeAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Run `opencode --version`; return trimmed stdout on success.
    fn probe_version() -> Option<String> {
        let out = Command::new("opencode")
            .arg("--version")
            .stdin(std::process::Stdio::null())
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if s.is_empty() {
            // Some CLIs print to stderr.
            let e = String::from_utf8_lossy(&out.stderr).trim().to_string();
            if e.is_empty() { None } else { Some(e) }
        } else {
            Some(s)
        }
    }
}

impl Default for OpenCodeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentEngineAdapter for OpenCodeAdapter {
    fn id(&self) -> &'static str {
        "opencode"
    }

    fn display_name(&self) -> &'static str {
        "OpenCode"
    }

    fn adapter_version(&self) -> u32 {
        1
    }

    fn discover(&self) -> EngineInfo {
        let version = Self::probe_version();
        let installed = version.is_some();
        EngineInfo {
            id: self.id().into(),
            display_name: self.display_name().into(),
            adapter_version: self.adapter_version(),
            status: match version {
                Some(_) => EngineStatus::Available,
                None => EngineStatus::NotInstalled,
            },
            // Capabilities are only reported as available when the engine itself
            // is present. These reflect what the OpenCode CLI exposes today;
            // streaming/session integration is milestone 2 and the adapter says so.
            capabilities: EngineCapabilities {
                streaming: installed,
                tool_calls: installed,
                file_editing: installed,
                terminal_execution: installed,
                session_persistence: installed,
                interrupt: installed,
                multi_agent: false,
                extras: [
                    ("headless_mode", installed),
                    ("tui_mode", installed),
                    ("mcp_support", installed),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            },
            detected_version: version.clone(),
            notes: vec![
                format!(
                    "Discovery via `opencode --version` on PATH{}",
                    version
                        .as_deref()
                        .map(|v| format!(" — detected {v}"))
                        .unwrap_or_else(|| " (not found)".into())
                ),
                "Chat/session streaming integration lands in milestone 2; \
                 this adapter currently reports discovery only."
                    .into(),
            ],
        }
    }
}
