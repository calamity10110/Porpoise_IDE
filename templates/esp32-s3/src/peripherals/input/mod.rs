//! Input device traits (mouse, keyboard).

use super::{Peripheral, PeripheralError};

// ---------------------------------------------------------------------------
// Mouse
// ---------------------------------------------------------------------------

/// Mouse button flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseButtons {
    pub bits: u8,
}

impl MouseButtons {
    pub const LEFT: u8 = 0x01;
    pub const RIGHT: u8 = 0x02;
    pub const MIDDLE: u8 = 0x04;

    pub fn left(&self) -> bool { self.bits & Self::LEFT != 0 }
    pub fn right(&self) -> bool { self.bits & Self::RIGHT != 0 }
    pub fn middle(&self) -> bool { self.bits & Self::MIDDLE != 0 }
}

/// Mouse report (relative movement).
#[derive(Debug, Clone, Copy, Default)]
pub struct MouseReport {
    pub dx: i8,
    pub dy: i8,
    pub wheel: i8,
    pub buttons: u8,
}

/// Trait for mouse peripherals (USB HID, PS/2, BLE).
pub trait MousePeripheral: Peripheral {
    /// Poll for a mouse report. Returns None if no new data.
    fn poll(&mut self) -> Result<Option<MouseReport>, PeripheralError>;

    /// Set mouse sensitivity/speed (1-10).
    fn set_sensitivity(&mut self, level: u8) -> Result<(), PeripheralError> {
        Err(PeripheralError::NotSupported)
    }
}

// ---------------------------------------------------------------------------
// Keyboard
// ---------------------------------------------------------------------------

/// Keyboard event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvent {
    /// Key pressed with HID usage code.
    Press(u8),
    /// Key released.
    Release(u8),
}

/// Modifier key flags.
#[derive(Debug, Clone, Copy)]
pub struct Modifiers {
    pub bits: u8,
}

impl Modifiers {
    pub const LCTRL: u8 = 0x01;
    pub const LSHIFT: u8 = 0x02;
    pub const LALT: u8 = 0x04;
    pub const LGUI: u8 = 0x08;
    pub const RCTRL: u8 = 0x10;
    pub const RSHIFT: u8 = 0x20;
    pub const RALT: u8 = 0x40;
    pub const RGUI: u8 = 0x80;
}

/// Keyboard report (HID-style).
#[derive(Debug, Clone, Copy)]
pub struct KeyboardReport {
    pub modifiers: u8,
    pub keycodes: [u8; 6],
}

/// Trait for keyboard peripherals.
pub trait KeyboardPeripheral: Peripheral {
    /// Poll for a keyboard event. Returns None if no new data.
    fn poll(&mut self) -> Result<Option<KeyEvent>, PeripheralError>;

    /// Set LED state (caps lock, num lock, etc.).
    fn set_leds(&mut self, leds: u8) -> Result<(), PeripheralError> {
        Err(PeripheralError::NotSupported)
    }
}
