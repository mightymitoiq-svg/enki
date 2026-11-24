pub mod opencode;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What an agent engine can do. Discovered at runtime, never hardcoded per-engine
/// beyond safe assumptions documented in each adapter. Engines are not assumed
/// equal; the UI must render unsupported features as unsupported.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EngineCapabilities {
    pub streaming: bool,
    pub tool_calls: bool,
    pub file_editing: bool,
    pub terminal_execution: bool,
    pub session_persistence: bool,
    pub interrupt: bool,
    pub multi_agent: bool,
    /// Arbitrary extra capability keys an adapter wants to report.
    pub extras: BTreeMap<String, bool>,
}

/// Health of an engine at discovery time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EngineStatus {
    /// Binary found and version check succeeded.
    Available,
    /// Engine known but not installed / not on PATH.
    NotInstalled,
    /// Installed but the version probe failed or reported an unsupported version.
    Error(String),
}

/// A discovered engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    pub id: String,
    pub display_name: String,
    pub adapter_version: u32,
    pub status: EngineStatus,
    pub capabilities: EngineCapabilities,
    /// e.g. "opencode 0.14.2" — from the engine itself, or None.
    pub detected_version: Option<String>,
    /// Human explanation of what is and is not supported (honesty requirement).
    pub notes: Vec<String>,
}

/// Engine-neutral adapter contract. Each engine (OpenCode now; Codex, Claude Code,
/// custom later) implements this. All methods must do real work — no simulation.
pub trait AgentEngineAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn adapter_version(&self) -> u32;

    /// Discover whether the engine is installed and what it supports.
    /// Must perform a real probe (e.g. run `<engine> --version`) — may be called
    /// repeatedly (e.g. a "refresh" button); keep it fast and side-effect free.
    fn discover(&self) -> EngineInfo;
}

/// Registry of installed adapters. Constructed at app start.
pub struct EngineRegistry {
    adapters: Vec<Box<dyn AgentEngineAdapter>>,
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self { adapters: Vec::new() }
    }

    pub fn register(&mut self, adapter: Box<dyn AgentEngineAdapter>) {
        self.adapters.push(adapter);
    }

    pub fn discover_all(&self) -> Vec<EngineInfo> {
        self.adapters.iter().map(|a| a.discover()).collect()
    }

    pub fn get(&self, id: &str) -> Option<&dyn AgentEngineAdapter> {
        self.adapters.iter().map(|a| a.as_ref()).find(|a| a.id() == id)
    }

    pub fn ids(&self) -> Vec<&'static str> {
        self.adapters.iter().map(|a| a.id()).collect()
    }
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Construct the production registry with all shipped adapters.
pub fn production_registry() -> EngineRegistry {
    let mut reg = EngineRegistry::new();
    reg.register(Box::new(opencode::OpenCodeAdapter::new()));
    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeEngine {
        id: &'static str,
        installed: bool,
    }

    impl AgentEngineAdapter for FakeEngine {
        fn id(&self) -> &'static str {
            self.id
        }
        fn display_name(&self) -> &'static str {
            self.id
        }
        fn adapter_version(&self) -> u32 {
            1
        }
        fn discover(&self) -> EngineInfo {
            EngineInfo {
                id: self.id.into(),
                display_name: self.display_name().into(),
                adapter_version: 1,
                status: if self.installed {
                    EngineStatus::Available
                } else {
                    EngineStatus::NotInstalled
                },
                capabilities: EngineCapabilities {
                    streaming: self.installed,
                    tool_calls: false,
                    file_editing: false,
                    terminal_execution: false,
                    session_persistence: false,
                    interrupt: false,
                    multi_agent: false,
                    extras: BTreeMap::new(),
                },
                detected_version: self.installed.then(|| "1.0.0".into()),
                notes: vec![],
            }
        }
    }

    #[test]
    fn registry_discovers_all() {
        let mut reg = EngineRegistry::new();
        reg.register(Box::new(FakeEngine { id: "a", installed: true }));
        reg.register(Box::new(FakeEngine { id: "b", installed: false }));
        let infos = reg.discover_all();
        assert_eq!(infos.len(), 2);
        assert_eq!(infos[0].status, EngineStatus::Available);
        assert_eq!(infos[1].status, EngineStatus::NotInstalled);
        assert!(reg.get("a").is_some());
        assert!(reg.get("zzz").is_none());
        assert_eq!(reg.ids(), vec!["a", "b"]);
    }
}
