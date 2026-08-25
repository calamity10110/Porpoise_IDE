//! Peripheral trait definitions.
//!
//! Every peripheral implements the `Peripheral` trait for lifecycle management.
//! Category-specific traits add domain methods (e.g., `DisplayPeripheral`, `AudioInput`).
//!
//! # Implementing a Custom Peripheral
//!
//! ```ignore
//! struct MySensor { /* ... */ }
//!
//! impl Peripheral for MySensor {
//!     fn name(&self) -> &str { "my_sensor" }
//!     fn kind(&self) -> PeripheralKind { PeripheralKind::Custom("my_sensor") }
//!     fn init(&mut self) -> Result<(), PeripheralError> { Ok(()) }
//!     fn is_ready(&self) -> bool { true }
//!     fn shutdown(&mut self) -> Result<(), PeripheralError> { Ok(()) }
//! }
//!
//! impl CustomPeripheral for MySensor {
//!     fn read_raw(&mut self, buf: &mut [u8]) -> Result<usize, PeripheralError> { Ok(0) }
//!     fn write_raw(&mut self, data: &[u8]) -> Result<(), PeripheralError> { Ok(()) }
//!     fn command(&mut self, cmd: &str, args: &[u8]) -> Result<Vec<u8>, PeripheralError> { Ok(vec![]) }
//! }
//! ```

pub mod audio;
pub mod camera;
pub mod custom;
pub mod display;
pub mod imu;
pub mod input;
pub mod touch;

use core::fmt;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Unified error type for all peripheral operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeripheralError {
    /// Peripheral has not been initialized.
    NotInitialized,
    /// Communication bus error (I2C NACK, SPI fault, etc.).
    BusError(&'static str),
    /// Invalid parameter or configuration.
    InvalidParam(&'static str),
    /// Operation timed out.
    Timeout,
    /// Resource busy / locked.
    Busy,
    /// Hardware fault detected.
    HardwareFault(&'static str),
    /// Feature not supported by this peripheral.
    NotSupported,
    /// Generic error with message.
    Other(&'static str),
}

impl fmt::Display for PeripheralError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "not initialized"),
            Self::BusError(msg) => write!(f, "bus error: {msg}"),
            Self::InvalidParam(msg) => write!(f, "invalid param: {msg}"),
            Self::Timeout => write!(f, "timeout"),
            Self::Busy => write!(f, "busy"),
            Self::HardwareFault(msg) => write!(f, "hardware fault: {msg}"),
            Self::NotSupported => write!(f, "not supported"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Peripheral kind enum
// ---------------------------------------------------------------------------

/// Classification of peripheral types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeripheralKind {
    Display,
    Touch,
    AudioInput,  // microphone
    AudioOutput, // speaker
    Camera,
    Imu,
    Mouse,
    Keyboard,
    Custom(&'static str),
}

impl fmt::Display for PeripheralKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Display => write!(f, "display"),
            Self::Touch => write!(f, "touch"),
            Self::AudioInput => write!(f, "audio_input"),
            Self::AudioOutput => write!(f, "audio_output"),
            Self::Camera => write!(f, "camera"),
            Self::Imu => write!(f, "imu"),
            Self::Mouse => write!(f, "mouse"),
            Self::Keyboard => write!(f, "keyboard"),
            Self::Custom(name) => write!(f, "custom:{name}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Core Peripheral trait
// ---------------------------------------------------------------------------

/// Lifecycle and identity trait for all peripherals.
///
/// Every hardware peripheral must implement this trait.
pub trait Peripheral {
    /// Human-readable name (e.g., "st7789_display", "cst816s_touch").
    fn name(&self) -> &str;

    /// Peripheral classification.
    fn kind(&self) -> PeripheralKind;

    /// Initialize the peripheral (configure pins, send init sequences, etc.).
    fn init(&mut self) -> Result<(), PeripheralError>;

    /// Returns `true` if the peripheral is initialized and ready.
    fn is_ready(&self) -> bool;

    /// Gracefully shut down the peripheral.
    fn shutdown(&mut self) -> Result<(), PeripheralError>;

    /// Optional: return a status string for telemetry.
    fn status(&self) -> &str {
        if self.is_ready() { "ready" } else { "not_ready" }
    }
}

// ---------------------------------------------------------------------------
// Peripheral registry
// ---------------------------------------------------------------------------

/// Maximum number of peripherals in the registry.
pub const MAX_PERIPHERALS: usize = 16;

/// Peripheral registry — holds all registered peripherals.
///
/// Uses a fixed-size array since we're no_std.
pub struct PeripheralRegistry {
    entries: heapless::Vec<PeripheralEntry, MAX_PERIPHERALS>,
}

struct PeripheralEntry {
    name: &'static str,
    kind: PeripheralKind,
    ready: bool,
}

impl PeripheralRegistry {
    pub const fn new() -> Self {
        Self {
            entries: heapless::Vec::new(),
        }
    }

    /// Register a peripheral by name and kind.
    pub fn register(&mut self, name: &'static str, kind: PeripheralKind) -> Result<(), PeripheralError> {
        self.entries
            .push(PeripheralEntry {
                name,
                kind,
                ready: false,
            })
            .map_err(|_| PeripheralError::Other("registry full"))
    }

    /// Mark a peripheral as ready.
    pub fn set_ready(&mut self, name: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.name == name) {
            entry.ready = true;
        }
    }

    /// Check if a peripheral is registered and ready.
    pub fn is_ready(&self, name: &str) -> bool {
        self.entries.iter().any(|e| e.name == name && e.ready)
    }

    /// List all registered peripheral names.
    pub fn list(&self) -> &[PeripheralEntry] {
        &self.entries
    }

    /// Count of registered peripherals.
    pub fn count(&self) -> usize {
        self.entries.len()
    }
}
