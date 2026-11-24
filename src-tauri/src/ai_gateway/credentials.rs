use crate::core::{AppError, Result};

/// Credential storage abstraction. Production uses the OS keyring
/// (Windows Credential Manager via the `keyring` crate). Tests use an in-memory map.
pub trait CredentialStore: Send + Sync {
    fn set_credential(&self, service_id: &str, secret: &str) -> Result<()>;
    fn get_credential(&self, service_id: &str) -> Result<Option<String>>;
    fn delete_credential(&self, service_id: &str) -> Result<()>;
    fn has_credential(&self, service_id: &str) -> bool;
}

/// OS keyring-backed store. Service name is fixed so entries are attributable
/// to ENKI in the OS credential manager UI.
pub struct OsCredentialStore {
    service: &'static str,
}

impl OsCredentialStore {
    pub fn new() -> Self {
        Self {
            service: "dev.enki.desktop",
        }
    }
}

impl Default for OsCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore for OsCredentialStore {
    fn set_credential(&self, service_id: &str, secret: &str) -> Result<()> {
        let entry = keyring::Entry::new(self.service, service_id)
            .map_err(|e| AppError::CredentialStore(e.to_string()))?;
        entry
            .set_password(secret)
            .map_err(|e| AppError::CredentialStore(e.to_string()))?;
        Ok(())
    }

    fn get_credential(&self, service_id: &str) -> Result<Option<String>> {
        let entry = keyring::Entry::new(self.service, service_id)
            .map_err(|e| AppError::CredentialStore(e.to_string()))?;
        match entry.get_password() {
            Ok(s) => Ok(Some(s)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::CredentialStore(e.to_string())),
        }
    }

    fn delete_credential(&self, service_id: &str) -> Result<()> {
        let entry = keyring::Entry::new(self.service, service_id)
            .map_err(|e| AppError::CredentialStore(e.to_string()))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::CredentialStore(e.to_string())),
        }
    }

    fn has_credential(&self, service_id: &str) -> bool {
        matches!(self.get_credential(service_id), Ok(Some(_)))
    }
}

/// In-memory store for tests.
#[cfg(test)]
pub struct MemoryCredentialStore {
    map: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

#[cfg(test)]
impl MemoryCredentialStore {
    pub fn new() -> Self {
        Self {
            map: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

#[cfg(test)]
impl CredentialStore for MemoryCredentialStore {
    fn set_credential(&self, service_id: &str, secret: &str) -> Result<()> {
        self.map
            .lock()
            .unwrap()
            .insert(service_id.to_string(), secret.to_string());
        Ok(())
    }
    fn get_credential(&self, service_id: &str) -> Result<Option<String>> {
        Ok(self.map.lock().unwrap().get(service_id).cloned())
    }
    fn delete_credential(&self, service_id: &str) -> Result<()> {
        self.map.lock().unwrap().remove(service_id);
        Ok(())
    }
    fn has_credential(&self, service_id: &str) -> bool {
        self.map.lock().unwrap().contains_key(service_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_round_trip() {
        let s = MemoryCredentialStore::new();
        assert!(!s.has_credential("p1"));
        s.set_credential("p1", "sk-secret").unwrap();
        assert!(s.has_credential("p1"));
        assert_eq!(s.get_credential("p1").unwrap().as_deref(), Some("sk-secret"));
        s.delete_credential("p1").unwrap();
        assert!(!s.has_credential("p1"));
        // Delete of missing entry is not an error.
        s.delete_credential("p1").unwrap();
    }
}
