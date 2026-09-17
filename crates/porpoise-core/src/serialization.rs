use serde::{Serialize, de::DeserializeOwned};

use crate::error::{PorpoiseError, Result};

pub fn to_json_pretty<T: Serialize>(val: &T) -> Result<String> {
    serde_json::to_string_pretty(val).map_err(|e| PorpoiseError::Internal(format!("json serialize: {e}")))
}

pub fn to_json<T: Serialize>(val: &T) -> Result<String> {
    serde_json::to_string(val).map_err(|e| PorpoiseError::Internal(format!("json serialize: {e}")))
}

pub fn from_json<T: DeserializeOwned>(json: &str) -> Result<T> {
    serde_json::from_str(json).map_err(|e| PorpoiseError::Internal(format!("json deserialize: {e}")))
}

pub fn to_bincode<T: Serialize>(val: &T) -> Result<Vec<u8>> {
    let config = bincode::config::standard();
    bincode::serde::encode_to_vec(val, config).map_err(|e| PorpoiseError::Internal(format!("bincode encode: {e}")))
}

pub fn from_bincode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let config = bincode::config::standard();
    let (val, _) = bincode::serde::decode_from_slice(bytes, config)
        .map_err(|e| PorpoiseError::Internal(format!("bincode decode: {e}")))?;
    Ok(val)
}

pub fn to_value<T: Serialize>(val: &T) -> Result<serde_json::Value> {
    serde_json::to_value(val).map_err(|e| PorpoiseError::Internal(format!("json to_value: {e}")))
}

pub fn from_value<T: DeserializeOwned>(val: serde_json::Value) -> Result<T> {
    serde_json::from_value(val).map_err(|e| PorpoiseError::Internal(format!("json from_value: {e}")))
}

pub fn from_bincode_options<T: DeserializeOwned>(bytes: &[u8], config: bincode::config::Configuration) -> Result<T> {
    let (val, _) = bincode::serde::decode_from_slice(bytes, config)
        .map_err(|e| PorpoiseError::Internal(format!("bincode decode: {e}")))?;
    Ok(val)
}
