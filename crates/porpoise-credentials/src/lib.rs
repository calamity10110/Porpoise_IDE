use std::path::{Path, PathBuf};

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use fs2::FileExt;
use porpoise_core::error::{PorpoiseError, Result};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use zeroize::Zeroize;

fn kdf_iterations() -> u32 {
    if cfg!(debug_assertions) { 1_000 } else { 600_000 }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialEntry {
    pub id: String,
    pub name: String,
    pub credential_type: String,
    pub encrypted_value: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialVault {
    entries: Vec<CredentialEntry>,
    key_salt: Vec<u8>,
}

pub struct CredentialStore {
    vault_path: PathBuf,
    key: [u8; 32],
}

impl CredentialStore {
    pub fn new(data_dir: &Path, master_key: &[u8]) -> Result<Self> {
        std::fs::create_dir_all(data_dir).map_err(|e| PorpoiseError::Config(format!("create cred dir: {e}")))?;

        let vault_path = data_dir.join("credentials.json");

        let salt = if vault_path.exists() {
            let mut file = std::fs::File::open(&vault_path)
                .map_err(|e| PorpoiseError::Config(format!("open vault for salt: {e}")))?;
            file.try_lock_shared()
                .map_err(|e| PorpoiseError::Config(format!("lock vault: {e}")))?;
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content)
                .map_err(|e| PorpoiseError::Config(format!("read vault: {e}")))?;
            let vault: CredentialVault =
                serde_json::from_str(&content).map_err(|e| PorpoiseError::Config(format!("parse vault: {e}")))?;
            file.unlock().ok();
            if vault.key_salt.is_empty() {
                vec![]
            } else {
                vault.key_salt
            }
        } else {
            let mut salt = vec![0u8; 32];
            rand::thread_rng().fill_bytes(&mut salt);
            let empty_vault = CredentialVault {
                entries: Vec::new(),
                key_salt: salt.clone(),
            };
            let json = serde_json::to_string_pretty(&empty_vault)
                .map_err(|e| PorpoiseError::Config(format!("serialize: {e}")))?;
            std::fs::write(&vault_path, json).map_err(|e| PorpoiseError::Config(format!("write vault: {e}")))?;
            salt
        };

        let mut key = [0u8; 32];
        pbkdf2::pbkdf2_hmac::<Sha256>(master_key, &salt, kdf_iterations(), &mut key);

        Ok(Self { vault_path, key })
    }

    pub fn store(&mut self, name: &str, cred_type: &str, value: &str) -> Result<String> {
        let mut vault = self.load_vault()?;

        let nonce_bytes = self.generate_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let cipher =
            Aes256Gcm::new_from_slice(&self.key).map_err(|e| PorpoiseError::Internal(format!("cipher init: {e}")))?;

        let encrypted = cipher
            .encrypt(nonce, value.as_bytes())
            .map_err(|e| PorpoiseError::Internal(format!("encrypt: {e}")))?;

        let id = format!("cred_{}", chrono::Utc::now().timestamp_millis());
        let now = chrono::Utc::now().to_rfc3339();

        vault.entries.push(CredentialEntry {
            id: id.clone(),
            name: name.to_string(),
            credential_type: cred_type.to_string(),
            encrypted_value: encrypted,
            nonce: nonce_bytes.to_vec(),
            created_at: now.clone(),
            updated_at: now,
        });

        self.save_vault(&vault)?;
        Ok(id)
    }

    pub fn get(&self, id: &str) -> Result<String> {
        let vault = self.load_vault()?;
        let entry = vault
            .entries
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(|| PorpoiseError::Config(format!("credential '{id}' not found")))?;

        self.decrypt(entry)
    }

    pub fn get_by_name(&self, name: &str) -> Result<String> {
        let vault = self.load_vault()?;
        let entry = vault
            .entries
            .iter()
            .find(|e| e.name == name)
            .ok_or_else(|| PorpoiseError::Config(format!("credential '{name}' not found")))?;

        self.decrypt(entry)
    }

    pub fn list(&self) -> Result<Vec<CredentialEntry>> {
        let vault = self.load_vault()?;
        Ok(vault.entries)
    }

    pub fn delete(&mut self, id: &str) -> Result<()> {
        let mut vault = self.load_vault()?;
        vault.entries.retain(|e| e.id != id);
        self.save_vault(&vault)?;
        Ok(())
    }

    pub fn list_by_type(&self, cred_type: &str) -> Result<Vec<CredentialEntry>> {
        let vault = self.load_vault()?;
        Ok(vault
            .entries
            .iter()
            .filter(|e| e.credential_type == cred_type)
            .cloned()
            .collect())
    }

    fn decrypt(&self, entry: &CredentialEntry) -> Result<String> {
        let nonce = Nonce::from_slice(&entry.nonce);
        let cipher =
            Aes256Gcm::new_from_slice(&self.key).map_err(|e| PorpoiseError::Internal(format!("cipher init: {e}")))?;

        let plaintext = cipher
            .decrypt(nonce, entry.encrypted_value.as_ref())
            .map_err(|e| PorpoiseError::Internal(format!("decrypt: {e}")))?;

        String::from_utf8(plaintext).map_err(|e| PorpoiseError::Internal(format!("utf8: {e}")))
    }

    fn generate_nonce(&self) -> Vec<u8> {
        let mut nonce = vec![0u8; 12];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut nonce);
        nonce
    }

    fn load_vault(&self) -> Result<CredentialVault> {
        if !self.vault_path.exists() {
            return Ok(CredentialVault {
                entries: Vec::new(),
                key_salt: vec![],
            });
        }
        let mut file =
            std::fs::File::open(&self.vault_path).map_err(|e| PorpoiseError::Config(format!("open vault: {e}")))?;
        file.lock_shared()
            .map_err(|e| PorpoiseError::Config(format!("lock vault: {e}")))?;
        let mut content = String::new();
        std::io::Read::read_to_string(&mut file, &mut content)
            .map_err(|e| PorpoiseError::Config(format!("read vault: {e}")))?;
        let vault: CredentialVault =
            serde_json::from_str(&content).map_err(|e| PorpoiseError::Config(format!("parse vault: {e}")))?;
        file.unlock().ok();
        Ok(vault)
    }

    fn save_vault(&self, vault: &CredentialVault) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.vault_path)
            .map_err(|e| PorpoiseError::Config(format!("open vault for write: {e}")))?;
        file.lock_exclusive()
            .map_err(|e| PorpoiseError::Config(format!("lock vault exclusive: {e}")))?;
        let json =
            serde_json::to_string_pretty(vault).map_err(|e| PorpoiseError::Config(format!("serialize vault: {e}")))?;
        std::io::Write::write_all(&mut file, json.as_bytes())
            .map_err(|e| PorpoiseError::Config(format!("write vault: {e}")))?;
        file.sync_all()
            .map_err(|e| PorpoiseError::Config(format!("sync vault: {e}")))?;
        file.unlock().ok();
        Ok(())
    }
}

impl Drop for CredentialStore {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_store_and_retrieve() {
        let dir = TempDir::new().unwrap();
        let mut store = CredentialStore::new(dir.path(), b"test-master-key-1234567890").unwrap();

        let id = store.store("my-api-key", "api_key", "sk-1234567890abcdef").unwrap();
        let value = store.get(&id).unwrap();
        assert_eq!(value, "sk-1234567890abcdef");
    }

    #[test]
    fn test_get_by_name() {
        let dir = TempDir::new().unwrap();
        let mut store = CredentialStore::new(dir.path(), b"test-key-123456").unwrap();

        store.store("github-token", "api_key", "ghp_xxxxxxxxxxxx").unwrap();
        let value = store.get_by_name("github-token").unwrap();
        assert_eq!(value, "ghp_xxxxxxxxxxxx");
    }

    #[test]
    fn test_delete() {
        let dir = TempDir::new().unwrap();
        let mut store = CredentialStore::new(dir.path(), b"test-key").unwrap();

        let id = store.store("temp", "password", "supersecret").unwrap();
        assert_eq!(store.list().unwrap().len(), 1);
        store.delete(&id).unwrap();
        assert_eq!(store.list().unwrap().len(), 0);
    }

    #[test]
    fn test_list_by_type() {
        let dir = TempDir::new().unwrap();
        let mut store = CredentialStore::new(dir.path(), b"test-key").unwrap();

        store.store("k1", "api_key", "v1").unwrap();
        store.store("k2", "password", "v2").unwrap();
        store.store("k3", "api_key", "v3").unwrap();

        let apis = store.list_by_type("api_key").unwrap();
        assert_eq!(apis.len(), 2);
    }

    #[test]
    fn test_persistence() {
        let dir = TempDir::new().unwrap();
        let key = b"persist-test-key-32bytes-long!!";

        {
            let mut store = CredentialStore::new(dir.path(), key).unwrap();
            store.store("persist-key", "api_key", "persist-value").unwrap();
        }

        let store = CredentialStore::new(dir.path(), key).unwrap();
        let value = store.get_by_name("persist-key").unwrap();
        assert_eq!(value, "persist-value");
    }
}
