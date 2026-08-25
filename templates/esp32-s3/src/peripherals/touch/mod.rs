//! Touch peripheral trait.

use super::{Peripheral, PeripheralError};

/// Touch event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchEvent {
    /// Finger/pen down at (x, y).
    Down { x: u16, y: u16, id: u8 },
    /// Finger/pen moved to (x, y).
    Move { x: u16, y: u16, id: u8 },
    /// Finger/pen lifted.
    Up { id: u8 },
}

/// Touch controller capabilities.
#[derive(Debug, Clone)]
pub struct TouchInfo {
    pub max_touch_points: u8,
    pub has_gesture: bool,
    pub has_pressure: bool,
    pub x_range: (u16, u16),
    pub y_range: (u16, u16),
}

/// Trait for touch input peripherals.
pub trait TouchPeripheral: Peripheral {
    /// Return touch controller capabilities.
    fn info(&self) -> TouchInfo;

    /// Poll for a touch event. Returns `None` if no event pending.
    fn poll_event(&mut self) -> Result<Option<TouchEvent>, PeripheralError>;

    /// Read raw touch coordinates. Returns (x, y, pressure) or None if not touching.
    fn read_raw(&mut self) -> Result<Option<(u16, u16, u8)>, PeripheralError>;

    /// Calibrate the touch controller (optional).
    fn calibrate(&mut self) -> Result<(), PeripheralError> {
        Err(PeripheralError::NotSupported)
    }
}
