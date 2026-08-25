//! Waveshare ESP32-S3 Touch LCD 1.28" (349) board configuration.
//!
//! Hardware specs:
//! - ESP32-S3R8 (8MB PSRAM, 16MB Flash)
//! - 1.28" 240x240 IPS LCD (GC9A01, SPI)
//! - CST816S capacitive touch (I2C)
//! - QMI8658 6-axis IMU (I2C)
//! - Battery management (AXP2101)
//! - USB-C for programming and power

use crate::board::*;

/// Waveshare ESP32-S3 Touch LCD 1.28" (SKU 349) pin mapping.
pub struct WaveshareLcd349;

impl WaveshareLcd349 {
    pub fn new() -> Self { Self }
}

impl Default for WaveshareLcd349 {
    fn default() -> Self { Self::new() }
}

impl BoardConfig for WaveshareLcd349 {
    fn name(&self) -> &str {
        "waveshare-esp32s3-touch-lcd-349"
    }

    fn description(&self) -> &str {
        "Waveshare ESP32-S3 Touch LCD 1.28\" — 240x240 IPS, CST816S touch, QMI8658 IMU, 8MB PSRAM"
    }

    fn peripheral_map(&self) -> PeripheralMap {
        PeripheralMap {
            has_display: true,
            has_touch: true,
            has_mic: false,
            has_speaker: false,
            has_camera: false,
            has_imu: true,
            has_mouse: false,
            has_keyboard: false,
            custom_peripherals: &[],
        }
    }

    fn display_config(&self) -> Option<DisplayInterface> {
        Some(DisplayInterface::Spi {
            spi: SpiConfig {
                sclk: 6,   // GPIO6  — SCLK
                mosi: 7,   // GPIO7  — MOSI
                miso: -1i8 as u8, // Not connected (we use u8, -1 wraps; handled in driver)
                cs: 5,     // GPIO5  — CS
                frequency_hz: 80_000_000, // 80MHz SPI
            },
            dc: 4,         // GPIO4  — Data/Command
            cs: 5,         // GPIO5  — CS
            reset: Some(8), // GPIO8  — Reset
            bl: Some(15),  // GPIO15 — Backlight
        })
    }

    fn display_resolution(&self) -> (u16, u16) {
        (240, 240)
    }

    fn touch_config(&self) -> Option<TouchConfig> {
        Some(TouchConfig {
            i2c: I2cConfig {
                sda: 1,    // GPIO1  — SDA
                scl: 2,    // GPIO2  — SCL
                frequency_hz: 400_000, // 400kHz
            },
            interrupt: Some(3), // GPIO3 — INT
            reset: None,
        })
    }

    fn imu_i2c_config(&self) -> Option<I2cConfig> {
        // QMI8658 shares I2C bus with touch on some revisions
        Some(I2cConfig {
            sda: 1,
            scl: 2,
            frequency_hz: 400_000,
        })
    }

    fn cpu_frequency_mhz(&self) -> u32 { 240 }

    fn psram_size(&self) -> usize { 8 * 1024 * 1024 } // 8MB

    fn flash_size(&self) -> usize { 16 * 1024 * 1024 } // 16MB
}
