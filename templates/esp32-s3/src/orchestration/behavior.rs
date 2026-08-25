//! Behavior config system — runtime-updatable component configurations.
//!
//! Allows changing component parameters (thresholds, sample rates, brightness,
//! enabled/disabled state, etc.) **without re-flashing firmware**.
//!
//! # How It Works
//!
//! 1. On boot, the device reads the `behavior` partition (SPIFFS/LittleFS at 0x3D0000).
//! 2. If a valid config exists, component parameters are applied.
//! 3. At runtime, configs can be updated via WebSocket commands:
//!    `{ "type": "command", "target": "system", "action": "update_behavior", "params": { ... } }`
//! 4. Updated configs are persisted to the behavior partition and survive reboots.
//! 5. Configs can also be updated via OTA (write binary to behavior partition).
//!
//! # Config Format (JSON)
//!
//! ```json
//! {
//!   "version": 1,
//!   "components": {
//!     "imu": { "enabled": true, "sample_rate_hz": 100, "low_pass_alpha": 0.1 },
//!     "display": { "enabled": true, "brightness": 80, "rotation": 0 },
//!     "touch": { "enabled": true, "debounce_ms": 50 },
//!     "audio_in": { "enabled": false },
//!     "custom_sensor": { "enabled": true, "threshold": 42, "interval_ms": 500 }
//!   }
//! }
//! ```

use alloc::string::String;
use alloc::vec::Vec;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Behavior partition start address (must match partitions.csv).
pub const BEHAVIOR_PARTITION_ADDR: u32 = 0x3D0000;

/// Behavior partition size (128 KB).
pub const BEHAVIOR_PARTITION_SIZE: u32 = 0x20000;

/// Maximum config size (leave room for header).
pub const MAX_CONFIG_SIZE: usize = (BEHAVIOR_PARTITION_SIZE as usize) - 256;

/// Config magic bytes for validation.
pub const CONFIG_MAGIC: [u8; 4] = [b'P', b'O', b'R', b'P'];

/// Current config format version.
pub const CONFIG_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single component's runtime configuration.
#[derive(Debug, Clone)]
pub struct ComponentConfig {
    pub name: String,
    pub enabled: bool,
    pub params: Vec<(String, ConfigValue)>,
}

/// A typed config value (supports JSON-like types without serde).
#[derive(Debug, Clone)]
pub enum ConfigValue {
    Bool(bool),
    Int(i64),
    Float(f32),
    Str(String),
}

/// The full behavior configuration.
#[derive(Debug, Clone)]
pub struct BehaviorConfig {
    pub version: u32,
    pub components: Vec<ComponentConfig>,
}

/// Errors during behavior config operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorError {
    /// Config data is too large for the partition.
    TooLarge,
    /// Invalid magic bytes or checksum.
    InvalidFormat,
    /// Flash write failed.
    WriteFailed,
    /// Flash read failed.
    ReadFailed,
    /// Component not found in config.
    ComponentNotFound,
    /// Parameter not found for a component.
    ParamNotFound,
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl BehaviorConfig {
    /// Create an empty config with default version.
    pub fn new() -> Self {
        Self {
            version: CONFIG_VERSION,
            components: Vec::new(),
        }
    }

    /// Add or update a component config.
    pub fn set_component(&mut self, config: ComponentConfig) {
        if let Some(existing) = self.components.iter_mut().find(|c| c.name == config.name) {
            *existing = config;
        } else {
            self.components.push(config);
        }
    }

    /// Get a component config by name.
    pub fn get_component(&self, name: &str) -> Option<&ComponentConfig> {
        self.components.iter().find(|c| c.name == name)
    }

    /// Check if a component is enabled.
    pub fn is_enabled(&self, name: &str) -> bool {
        self.get_component(name)
            .map(|c| c.enabled)
            .unwrap_or(true) // default: enabled
    }

    /// Get a float parameter for a component. Returns default if not found.
    pub fn get_float(&self, component: &str, param: &str, default: f32) -> f32 {
        self.get_component(component)
            .and_then(|c| c.params.iter().find(|(k, _)| k == param))
            .and_then(|(_, v)| match v {
                ConfigValue::Float(f) => Some(*f),
                ConfigValue::Int(i) => Some(*i as f32),
                _ => None,
            })
            .unwrap_or(default)
    }

    /// Get an int parameter for a component. Returns default if not found.
    pub fn get_int(&self, component: &str, param: &str, default: i64) -> i64 {
        self.get_component(component)
            .and_then(|c| c.params.iter().find(|(k, _)| k == param))
            .and_then(|(_, v)| match v {
                ConfigValue::Int(i) => Some(*i),
                ConfigValue::Float(f) => Some(*f as i64),
                _ => None,
            })
            .unwrap_or(default)
    }

    /// Get a bool parameter for a component. Returns default if not found.
    pub fn get_bool(&self, component: &str, param: &str, default: bool) -> bool {
        self.get_component(component)
            .and_then(|c| c.params.iter().find(|(k, _)| k == param))
            .and_then(|(_, v)| match v {
                ConfigValue::Bool(b) => Some(*b),
                _ => None,
            })
            .unwrap_or(default)
    }

    /// Serialize config to a portable binary format for flash storage.
    ///
    /// Format: [magic:4][version:4][component_count:4][components...]
    /// Each component: [name_len:1][name:N][enabled:1][param_count:2][params...]
    /// Each param: [key_len:1][key:N][value_type:1][value:variable]
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Header
        buf.extend_from_slice(&CONFIG_MAGIC);
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&(self.components.len() as u32).to_le_bytes());

        for comp in &self.components {
            let name_bytes = comp.name.as_bytes();
            buf.push(name_bytes.len() as u8);
            buf.extend_from_slice(name_bytes);
            buf.push(if comp.enabled { 1 } else { 0 });
            buf.extend_from_slice(&(comp.params.len() as u16).to_le_bytes());

            for (key, value) in &comp.params {
                let key_bytes = key.as_bytes();
                buf.push(key_bytes.len() as u8);
                buf.extend_from_slice(key_bytes);
                match value {
                    ConfigValue::Bool(b) => {
                        buf.push(0x01);
                        buf.push(if *b { 1 } else { 0 });
                    }
                    ConfigValue::Int(i) => {
                        buf.push(0x02);
                        buf.extend_from_slice(&i.to_le_bytes());
                    }
                    ConfigValue::Float(f) => {
                        buf.push(0x03);
                        buf.extend_from_slice(&f.to_le_bytes());
                    }
                    ConfigValue::Str(s) => {
                        buf.push(0x04);
                        let s_bytes = s.as_bytes();
                        buf.extend_from_slice(&(s_bytes.len() as u16).to_le_bytes());
                        buf.extend_from_slice(s_bytes);
                    }
                }
            }
        }

        buf
    }

    /// Deserialize config from binary format.
    pub fn deserialize(data: &[u8]) -> Result<Self, BehaviorError> {
        if data.len() < 12 {
            return Err(BehaviorError::InvalidFormat);
        }

        // Check magic
        if data[0..4] != CONFIG_MAGIC {
            return Err(BehaviorError::InvalidFormat);
        }

        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let comp_count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;

        let mut config = BehaviorConfig {
            version,
            components: Vec::with_capacity(comp_count),
        };

        let mut offset = 12;
        for _ in 0..comp_count {
            if offset >= data.len() {
                return Err(BehaviorError::InvalidFormat);
            }

            let name_len = data[offset] as usize;
            offset += 1;
            if offset + name_len > data.len() {
                return Err(BehaviorError::InvalidFormat);
            }
            let name = core::str::from_utf8(&data[offset..offset + name_len])
                .map_err(|_| BehaviorError::InvalidFormat)?
                .to_string();
            offset += name_len;

            if offset >= data.len() {
                return Err(BehaviorError::InvalidFormat);
            }
            let enabled = data[offset] != 0;
            offset += 1;

            if offset + 2 > data.len() {
                return Err(BehaviorError::InvalidFormat);
            }
            let param_count = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;

            let mut params = Vec::with_capacity(param_count);
            for _ in 0..param_count {
                if offset >= data.len() {
                    return Err(BehaviorError::InvalidFormat);
                }
                let key_len = data[offset] as usize;
                offset += 1;
                if offset + key_len > data.len() {
                    return Err(BehaviorError::InvalidFormat);
                }
                let key = core::str::from_utf8(&data[offset..offset + key_len])
                    .map_err(|_| BehaviorError::InvalidFormat)?
                    .to_string();
                offset += key_len;

                if offset >= data.len() {
                    return Err(BehaviorError::InvalidFormat);
                }
                let value_type = data[offset];
                offset += 1;

                let value = match value_type {
                    0x01 => {
                        if offset >= data.len() {
                            return Err(BehaviorError::InvalidFormat);
                        }
                        let b = data[offset] != 0;
                        offset += 1;
                        ConfigValue::Bool(b)
                    }
                    0x02 => {
                        if offset + 8 > data.len() {
                            return Err(BehaviorError::InvalidFormat);
                        }
                        let i = i64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
                        offset += 8;
                        ConfigValue::Int(i)
                    }
                    0x03 => {
                        if offset + 4 > data.len() {
                            return Err(BehaviorError::InvalidFormat);
                        }
                        let f = f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
                        offset += 4;
                        ConfigValue::Float(f)
                    }
                    0x04 => {
                        if offset + 2 > data.len() {
                            return Err(BehaviorError::InvalidFormat);
                        }
                        let s_len =
                            u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
                        offset += 2;
                        if offset + s_len > data.len() {
                            return Err(BehaviorError::InvalidFormat);
                        }
                        let s = core::str::from_utf8(&data[offset..offset + s_len])
                            .map_err(|_| BehaviorError::InvalidFormat)?
                            .to_string();
                        offset += s_len;
                        ConfigValue::Str(s)
                    }
                    _ => return Err(BehaviorError::InvalidFormat),
                };

                params.push((key, value));
            }

            config.components.push(ComponentConfig {
                name,
                enabled,
                params,
            });
        }

        Ok(config)
    }
}

// ---------------------------------------------------------------------------
// Flash I/O (stubs — real impl uses esp-idf-sys or esp-storage)
// ---------------------------------------------------------------------------

/// Load behavior config from flash partition.
///
/// Returns `None` if no valid config exists (first boot).
pub fn load_from_flash() -> Option<BehaviorConfig> {
    // Real implementation:
    //   let data = read_flash(BEHAVIOR_PARTITION_ADDR, BEHAVIOR_PARTITION_SIZE);
    //   BehaviorConfig::deserialize(&data).ok()
    //
    // For now, return None (use compiled defaults).
    None
}

/// Save behavior config to flash partition.
pub fn save_to_flash(config: &BehaviorConfig) -> Result<(), BehaviorError> {
    let data = config.serialize();
    if data.len() > MAX_CONFIG_SIZE {
        return Err(BehaviorError::TooLarge);
    }

    // Real implementation:
    //   erase_flash(BEHAVIOR_PARTITION_ADDR, BEHAVIOR_PARTITION_SIZE);
    //   write_flash(BEHAVIOR_PARTITION_ADDR, &data)
    //     .map_err(|_| BehaviorError::WriteFailed)?;

    Ok(())
}

/// Apply a partial behavior update (merge into existing config).
///
/// Only updates components/params present in `update`. Missing entries
/// in `update` are left unchanged in `current`.
pub fn apply_update(
    current: &mut BehaviorConfig,
    update: &BehaviorConfig,
) -> Result<(), BehaviorError> {
    for update_comp in &update.components {
        if let Some(existing) = current
            .components
            .iter_mut()
            .find(|c| c.name == update_comp.name)
        {
            // Merge: update enabled state
            existing.enabled = update_comp.enabled;
            // Merge: update/add params
            for (key, value) in &update_comp.params {
                if let Some((_, existing_val)) =
                    existing.params.iter_mut().find(|(k, _)| k == key)
                {
                    *existing_val = value.clone();
                } else {
                    existing.params.push((key.clone(), value.clone()));
                }
            }
        } else {
            // New component
            current.components.push(update_comp.clone());
        }
    }

    // Persist to flash
    save_to_flash(current)
}

// ---------------------------------------------------------------------------
// Default configs for known boards
// ---------------------------------------------------------------------------

/// Create default behavior config for Waveshare LCD 349.
pub fn default_waveshare_349() -> BehaviorConfig {
    let mut config = BehaviorConfig::new();

    config.set_component(ComponentConfig {
        name: String::from("display"),
        enabled: true,
        params: alloc::vec![
            (String::from("brightness"), ConfigValue::Int(80)),
            (String::from("rotation"), ConfigValue::Int(0)),
        ],
    });

    config.set_component(ComponentConfig {
        name: String::from("touch"),
        enabled: true,
        params: alloc::vec![
            (String::from("debounce_ms"), ConfigValue::Int(50)),
            (String::from("gesture_enabled"), ConfigValue::Bool(true)),
        ],
    });

    config.set_component(ComponentConfig {
        name: String::from("imu"),
        enabled: true,
        params: alloc::vec![
            (String::from("sample_rate_hz"), ConfigValue::Int(100)),
            (String::from("low_pass_alpha"), ConfigValue::Float(0.1)),
        ],
    });

    config
}

/// Create default behavior config for Waveshare LCD 349 Touch.
pub fn default_waveshare_349_touch() -> BehaviorConfig {
    let mut config = default_waveshare_349();

    config.set_component(ComponentConfig {
        name: String::from("audio_in"),
        enabled: false,
        params: Vec::new(),
    });

    config.set_component(ComponentConfig {
        name: String::from("audio_out"),
        enabled: false,
        params: Vec::new(),
    });

    config
}
