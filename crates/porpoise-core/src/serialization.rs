use serde::{de::DeserializeOwned, Serialize};

use crate::error::{PorpoiseError, Result};

/// Serializes a value to pretty-printed JSON string.
pub fn to_json_pretty<T: Serialize>(val: &T) -> Result<String> {
    serde_json::to_string_pretty(val)
        .map_err(|e| PorpoiseError::Internal(format!("json serialize: {e}")))
}

/// Serializes a value to compact JSON string.
pub fn to_json<T: Serialize>(val: &T) -> Result<String> {
    serde_json::to_string(val)
        .map_err(|e| PorpoiseError::Internal(format!("json serialize: {e}")))
}

/// Deserializes a value from a JSON string.
pub fn from_json<T: DeserializeOwned>(json: &str) -> Result<T> {
    serde_json::from_str(json)
        .map_err(|e| PorpoiseError::Internal(format!("json deserialize: {e}")))
}

/// Serializes a value to a bincode byte vector (compact binary format).
pub fn to_bincode<T: Serialize>(val: &T) -> Result<Vec<u8>> {
    let config = bincode::config::standard();
    bincode::serde::encode_to_vec(val, config)
        .map_err(|e| PorpoiseError::Internal(format!("bincode encode: {e}")))
}

/// Deserializes a value from a bincode byte slice.
pub fn from_bincode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let config = bincode::config::standard();
    let (val, _) = bincode::serde::decode_from_slice(bytes, config)
        .map_err(|e| PorpoiseError::Internal(format!("bincode decode: {e}")))?;
    Ok(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_roundtrip() {
        let data = vec!["hello".to_string(), "world".to_string()];
        let json = to_json_pretty(&data).unwrap();
        let back: Vec<String> = from_json(&json).unwrap();
        assert_eq!(data, back);
    }

    #[test]
    fn test_bincode_roundtrip() {
        let data = vec![1u32, 2, 3];
        let bytes = to_bincode(&data).unwrap();
        let back: Vec<u32> = from_bincode(&bytes).unwrap();
        assert_eq!(data, back);
    }

    #[test]
    fn test_json_pretty_has_newlines() {
        let data = vec![1, 2, 3];
        let json = to_json_pretty(&data).unwrap();
        assert!(json.contains('\n'));
    }
}
