//! Custom peripheral trait for user-defined hardware.

use super::{Peripheral, PeripheralError};

/// Trait for custom user-defined peripherals.
///
/// Implement this for any hardware not covered by the standard traits.
/// The orchestration layer can send arbitrary commands via `command()`.
pub trait CustomPeripheral: Peripheral {
    /// Read raw data from the peripheral.
    fn read_raw(&mut self, buf: &mut [u8]) -> Result<usize, PeripheralError>;

    /// Write raw data to the peripheral.
    fn write_raw(&mut self, data: &[u8]) -> Result<(), PeripheralError>;

    /// Execute a named command with optional arguments.
    /// Returns response bytes.
    ///
    /// # Example commands:
    /// - "set_mode" + [mode_byte]
    /// - "get_status" + [] -> status bytes
    /// - "calibrate" + []
    fn command(&mut self, cmd: &str, args: &[u8]) -> Result<heapless::Vec<u8, 256>, PeripheralError>;

    /// List available commands (for remote discovery).
    fn available_commands(&self) -> &[&str] {
        &[]
    }
}
