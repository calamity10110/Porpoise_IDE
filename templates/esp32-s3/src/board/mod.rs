//! Board configuration trait and registry.
//!
//! Each board defines its pin mapping, peripheral list, and clock config.
//! Users create a board config struct implementing `BoardConfig` for their hardware.

use crate::peripherals::PeripheralError;

/// Pin number type (GPIO index).
pub type Pin = u8;

/// SPI bus configuration.
#[derive(Debug, Clone)]
pub struct SpiConfig {
    pub sclk: Pin,
    pub mosi: Pin,
    pub miso: Pin,
    pub cs: Option<Pin>,
    pub frequency_hz: u32,
}

/// I2C bus configuration.
#[derive(Debug, Clone)]
pub struct I2cConfig {
    pub sda: Pin,
    pub scl: Pin,
    pub frequency_hz: u32,
}

/// I2S bus configuration (audio).
#[derive(Debug, Clone)]
pub struct I2sConfig {
    pub bclk: Pin,
    pub ws: Pin,
    pub din: Option<Pin>,  // for mic
    pub dout: Option<Pin>, // for speaker
    pub mclk: Option<Pin>,
}

/// DVP camera interface pins.
#[derive(Debug, Clone)]
pub struct DvpConfig {
    pub pclk: Pin,
    pub vsync: Pin,
    pub href: Pin,
    pub d0: Pin,
    pub d1: Pin,
    pub d2: Pin,
    pub d3: Pin,
    pub d4: Pin,
    pub d5: Pin,
    pub d6: Pin,
    pub d7: Pin,
    pub xclk: Pin,
    pub powerdown: Option<Pin>,
    pub reset: Option<Pin>,
}

/// Display interface type.
#[derive(Debug, Clone)]
pub enum DisplayInterface {
    Spi {
        spi: SpiConfig,
        dc: Pin,
        cs: Pin,
        reset: Option<Pin>,
        bl: Option<Pin>,
    },
    Rgb {
        r0: Pin, r1: Pin, r2: Pin, r3: Pin, r4: Pin,
        g0: Pin, g1: Pin, g2: Pin, g3: Pin, g4: Pin, g5: Pin,
        b0: Pin, b1: Pin, b2: Pin, b3: Pin, b4: Pin,
        hsync: Pin,
        vsync: Pin,
        de: Pin,
        pclk: Pin,
        bl: Option<Pin>,
    },
    Mcu8080 {
        d0: Pin, d1: Pin, d2: Pin, d3: Pin,
        d4: Pin, d5: Pin, d6: Pin, d7: Pin,
        wr: Pin, rd: Pin, cs: Pin, dc: Pin,
        reset: Option<Pin>,
        bl: Option<Pin>,
    },
}

/// Touch interface configuration.
#[derive(Debug, Clone)]
pub struct TouchConfig {
    pub i2c: I2cConfig,
    pub interrupt: Option<Pin>,
    pub reset: Option<Pin>,
}

/// Peripheral presence flags for a board.
#[derive(Debug, Clone, Default)]
pub struct PeripheralMap {
    pub has_display: bool,
    pub has_touch: bool,
    pub has_mic: bool,
    pub has_speaker: bool,
    pub has_camera: bool,
    pub has_imu: bool,
    pub has_mouse: bool,
    pub has_keyboard: bool,
    pub custom_peripherals: &'static [&'static str],
}

/// Trait that every board configuration must implement.
pub trait BoardConfig {
    /// Board name (e.g., "waveshare-esp32s3-touch-lcd-349").
    fn name(&self) -> &str;

    /// Board description.
    fn description(&self) -> &str;

    /// Which peripherals are available on this board.
    fn peripheral_map(&self) -> PeripheralMap;

    /// Display interface configuration (if any).
    fn display_config(&self) -> Option<DisplayInterface> { None }

    /// Display resolution (width, height).
    fn display_resolution(&self) -> (u16, u16) { (0, 0) }

    /// Touch configuration (if any).
    fn touch_config(&self) -> Option<TouchConfig> { None }

    /// I2S audio configuration (if any).
    fn i2s_config(&self) -> Option<I2sConfig> { None }

    /// Camera DVP configuration (if any).
    fn dvp_config(&self) -> Option<DvpConfig> { None }

    /// IMU I2C configuration (if any).
    fn imu_i2c_config(&self) -> Option<I2cConfig> { None }

    /// CPU frequency in MHz.
    fn cpu_frequency_mhz(&self) -> u32 { 240 }

    /// PSRAM size in bytes (0 if none).
    fn psram_size(&self) -> usize { 0 }

    /// Flash size in bytes.
    fn flash_size(&self) -> usize { 4 * 1024 * 1024 }
}
