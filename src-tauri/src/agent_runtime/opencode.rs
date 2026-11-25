use super::{AgentEngineAdapter, EngineCapabilities, EngineInfo, EngineStatus};
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
            if e.is_empty() {
                None
            } else {
                Some(e)
            }
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
        EngineInfo {
            id: self.id().into(),
            display_name: self.display_name().into(),
            adapter_version: self.adapter_version(),
            status: match version {
                Some(_) => EngineStatus::Available,
                None => EngineStatus::NotInstalled,
            },
            // Finding the CLI proves only that it is installed. ENKI does not yet
            // implement the session and tool integrations represented here.
            capabilities: EngineCapabilities {
                streaming: false,
                tool_calls: false,
                file_editing: false,
                terminal_execution: false,
                session_persistence: false,
                interrupt: false,
                multi_agent: false,
                extras: Default::default(),
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
                "ENKI can detect OpenCode, but does not yet run or control sessions. \
                 Capabilities will appear when the ENKI integration is implemented."
                    .into(),
            ],
        }
    }
}
