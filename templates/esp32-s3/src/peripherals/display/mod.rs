//! Display peripheral trait.

use super::{Peripheral, PeripheralError};

/// Color format for framebuffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    Rgb565,
    Rgb888,
    Argb8888,
    Monochrome,
}

/// Display orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Portrait,
    Landscape,
    PortraitFlipped,
    LandscapeFlipped,
}

/// Display capabilities reported to the orchestrator.
#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub width: u16,
    pub height: u16,
    pub color_format: ColorFormat,
    pub orientation: Orientation,
    pub has_framebuffer: bool,
    pub brightness: u8,
}

/// Trait for display peripherals (LCD, OLED, e-ink, etc.).
pub trait DisplayPeripheral: Peripheral {
    /// Return display dimensions and capabilities.
    fn info(&self) -> DisplayInfo;

    /// Set display orientation.
    fn set_orientation(&mut self, orientation: Orientation) -> Result<(), PeripheralError>;

    /// Set backlight brightness (0-255).
    fn set_brightness(&mut self, brightness: u8) -> Result<(), PeripheralError>;

    /// Write a raw pixel buffer to a rectangular region.
    fn write_pixels(
        &mut self,
        x: u16, y: u16,
        w: u16, h: u16,
        data: &[u8],
    ) -> Result<(), PeripheralError>;

    /// Fill a rectangle with a solid color (in native pixel format).
    fn fill_rect(
        &mut self,
        x: u16, y: u16,
        w: u16, h: u16,
        color: &[u8],
    ) -> Result<(), PeripheralError>;

    /// Clear the entire display to a color.
    fn clear(&mut self, color: &[u8]) -> Result<(), PeripheralError> {
        let info = self.info();
        self.fill_rect(0, 0, info.width, info.height, color)
    }
}
