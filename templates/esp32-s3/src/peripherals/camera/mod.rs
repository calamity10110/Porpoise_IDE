//! Camera peripheral trait.

use super::{Peripheral, PeripheralError};

/// Pixel format for captured frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Rgb565,
    Rgb888,
    Yuv422,
    Jpeg,
    Grayscale,
}

/// Camera resolution preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Qvga,   // 320x240
    Vga,    // 640x480
    Svga,   // 800x600
    Xga,    // 1024x768
    Custom(u16, u16),
}

impl Resolution {
    pub fn dimensions(&self) -> (u16, u16) {
        match self {
            Self::Qvga => (320, 240),
            Self::Vga => (640, 480),
            Self::Svga => (800, 600),
            Self::Xga => (1024, 768),
            Self::Custom(w, h) => (*w, *h),
        }
    }
}

/// Camera capabilities.
#[derive(Debug, Clone)]
pub struct CameraInfo {
    pub max_resolution: Resolution,
    pub supported_formats: &'static [PixelFormat],
    pub has_flash: bool,
    pub has_autofocus: bool,
}

/// Trait for camera peripherals (DVP, MIPI-CSI).
pub trait CameraPeripheral: Peripheral {
    fn info(&self) -> CameraInfo;

    /// Configure capture parameters.
    fn configure(&mut self, resolution: Resolution, format: PixelFormat) -> Result<(), PeripheralError>;

    /// Capture a single frame. Returns frame data in configured format.
    fn capture(&mut self, buf: &mut [u8]) -> Result<usize, PeripheralError>;

    /// Start continuous capture (streaming).
    fn start_stream(&mut self) -> Result<(), PeripheralError>;

    /// Stop continuous capture.
    fn stop_stream(&mut self) -> Result<(), PeripheralError>;

    /// Set brightness/contrast/exposure (0-255 each).
    fn set_params(&mut self, brightness: u8, contrast: u8, exposure: u8) -> Result<(), PeripheralError> {
        Err(PeripheralError::NotSupported)
    }
}
