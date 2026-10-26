//! Credential vault commands: the real consumer of
//! `porpoise-credentials::CredentialStore`.
//!
//! Works directly on the encrypted vault file — no daemon required — so
//! secrets are stored and read with the same AES-256-GCM + PBKDF2 scheme
//! the rest of the system will use. The master key comes from
//! `PORPOISE_MASTER_KEY` and is required: there is no silent default for
//! a key that protects secrets. The vault lives under the data directory
//! (`PORPOISE_DATA_DIR` override or the platform default).

use porpoise_core::error::{PorpoiseError, Result};
use porpoise_credentials::CredentialStore;

use crate::{app::CredentialAction, output::OutputFormat};

fn data_dir() -> Result<std::path::PathBuf> {
    if let Ok(dir) = std::env::var("PORPOISE_DATA_DIR") {
        if !dir.trim().is_empty() {
            return Ok(std::path::PathBuf::from(dir));
        }
    }
    porpoise_core::config::AppConfig::default_data_dir()
}

fn master_key() -> Result<Vec<u8>> {
    match std::env::var("PORPOISE_MASTER_KEY") {
        Ok(k) if !k.trim().is_empty() => Ok(k.into_bytes()),
        _ => Err(PorpoiseError::Config(
            "PORPOISE_MASTER_KEY is not set — refusing to use a default for the \
             credential vault. Export it first, e.g. `export PORPOISE_MASTER_KEY=...`"
                .into(),
        )),
    }
}

fn open_store() -> Result<CredentialStore> {
    let dir = data_dir()?;
    let key = master_key()?;
    CredentialStore::new(&dir, &key)
}

fn read_value_from_stdin() -> Result<String> {
    let mut value = String::new();
    std::io::stdin()
        .read_line(&mut value)
        .map_err(|e| PorpoiseError::Config(format!("read stdin: {e}")))?;
    let trimmed = value.trim_end_matches(['\n', '\r']).to_string();
    if trimmed.is_empty() {
        return Err(PorpoiseError::Config("no value provided on stdin".into()));
    }
    Ok(trimmed)
}

pub async fn handle(args: crate::app::CredentialArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        CredentialAction::Set {
            name,
            credential_type,
            value,
        } => {
            let value = match value {
                Some(v) => v,
                None => read_value_from_stdin()?,
            };
            let mut store = open_store()?;
            let id = store.set(&name, &credential_type, &value)?;
            Ok(format.format(&serde_json::json!({
                "stored": name,
                "type": credential_type,
                "id": id,
            })))
        }
        CredentialAction::Get { name } => {
            let store = open_store()?;
            let value = store.get_by_name(&name)?;
            Ok(value)
        }
        CredentialAction::List => {
            let store = open_store()?;
            let entries = store.list()?;
            let items: Vec<serde_json::Value> = entries
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "name": e.name,
                        "type": e.credential_type,
                        "updated_at": e.updated_at,
                    })
                })
                .collect();
            Ok(format.format(&serde_json::json!({ "credentials": items })))
        }
        CredentialAction::Delete { name } => {
            let mut store = open_store()?;
            let removed = store.delete_by_name(&name)?;
            if removed {
                Ok(format!("deleted {name}"))
            } else {
                Err(PorpoiseError::Config(format!("credential '{name}' not found")))
            }
        }
    }
}
