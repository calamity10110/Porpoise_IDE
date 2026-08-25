//! Waveshare ESP32-S3 Touch LCD 3.49" board configuration.
//!
//! Hardware specs:
//! - ESP32-S3R8 (8MB PSRAM, 16MB Flash)
//! - 3.49" 480x480 IPS LCD (ST7701S, RGB565 parallel interface)
//! - GT911 capacitive touch (I2C)
//! - QMI8658 6-axis IMU (I2C)
//! - Battery management (AXP2101)
//! - USB-C
//!
//! NOTE: Pin mapping based on Waveshare official wiki. Verify against your board revision.

use crate::board::{
    BoardConfig, DisplayInterface, I2cConfig, I2sConfig, PeripheralMap, TouchConfig,
};

pub struct WaveshareLcd349Touch;

impl BoardConfig for WaveshareLcd349Touch {
    fn name(&self) -> &str {
        "waveshare-esp32s3-touch-lcd-349"
    }

    fn description(&self) -> &str {
        "Waveshare ESP32-S3 Touch LCD 3.49\" — 480x480 ST7701S RGB565, GT911 touch, QMI8658 IMU"
    }

    fn peripheral_map(&self) -> PeripheralMap {
        PeripheralMap {
            has_display: true,
            has_touch: true,
            has_mic: false,   // enable if your revision has MSM261
            has_speaker: false, // enable if your revision has MAX98357A
            has_camera: false,
            has_imu: true,
            has_mouse: false,
            has_keyboard: false,
            custom_peripherals: &[],
        }
    }

    fn display_config(&self) -> Option<DisplayInterface> {
        Some(DisplayInterface::Rgb {
            // RGB565 data pins (16-bit)
            r0: 46,
            r1: 3,
            r2: 8,
            r3: 18,
            r4: 17,
            g0: 14,
            g1: 21,
            g2: 47,
            g3: 48,
            g4: 0,
            g5: 9,
            b0: 10,
            b1: 11,
            b2: 12,
            b3: 13,
            b4: 16,
            // Control pins
            hsync: 42,
            vsync: 41,
            de: 40,
            pclk: 45,
            bl: Some(1),
        })
    }

    fn display_resolution(&self) -> (u16, u16) {
        (480, 480)
    }

    fn touch_config(&self) -> Option<TouchConfig> {
        Some(TouchConfig {
            i2c: I2cConfig {
                sda: 39,
                scl: 38,
                frequency_hz: 400_000,
            },
            interrupt: Some(44),
            reset: None,
        })
    }

    fn imu_i2c_config(&self) -> Option<I2cConfig> {
        // QMI8658 shares I2C bus with GT911 touch
        Some(I2cConfig {
            sda: 39,
            scl: 38,
            frequency_hz: 400_000,
        })
    }

    fn cpu_frequency_mhz(&self) -> u32 {
        240
    }

    fn psram_size(&self) -> usize {
        8 * 1024 * 1024
    }

    fn flash_size(&self) -> usize {
        16 * 1024 * 1024
    }
}
