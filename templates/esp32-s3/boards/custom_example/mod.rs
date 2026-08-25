//! Custom board example — template for user-defined ESP32-S3 hardware.
//!
//! Copy this directory, rename it, and modify the config.toml and this file
//! to match your hardware. Then add a Cargo feature in the workspace Cargo.toml.

use crate::board::{BoardConfig, DisplayInterface, SpiConfig, I2sConfig, I2cConfig, CameraConfig};

pub struct CustomBoard;

impl BoardConfig for CustomBoard {
    fn name(&self) -> &str {
        "Custom ESP32-S3 Board"
    }

    fn has_display(&self) -> bool {
        false // set true after wiring display
    }

    fn has_touch(&self) -> bool {
        false
    }

    fn has_mic(&self) -> bool {
        false
    }

    fn has_speaker(&self) -> bool {
        false
    }

    fn has_camera(&self) -> bool {
        false
    }

    fn has_imu(&self) -> bool {
        false
    }

    fn has_mouse(&self) -> bool {
        false
    }

    fn has_keyboard(&self) -> bool {
        false
    }

    fn display_config(&self) -> Option<DisplayInterface> {
        // Example for ST7789 240x320:
        // Some(DisplayInterface::Spi {
        //     spi: SpiConfig { sclk: 12, mosi: 11, cs: 10, dc: 8, rst: 9, bl: Some(7), freq_hz: 80_000_000 },
        //     width: 240, height: 320,
        // })
        None
    }

    fn touch_config(&self) -> Option<I2cConfig> {
        None
    }

    fn audio_config(&self) -> Option<(Option<I2sConfig>, Option<I2sConfig>)> {
        None
    }

    fn camera_config(&self) -> Option<CameraConfig> {
        None
    }

    fn imu_config(&self) -> Option<I2cConfig> {
        None
    }
}
