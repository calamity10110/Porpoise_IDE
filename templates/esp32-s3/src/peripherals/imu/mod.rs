//! IMU (Inertial Measurement Unit) peripheral trait.

use super::{Peripheral, PeripheralError};

/// 3-axis sensor reading.
#[derive(Debug, Clone, Copy, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Combined IMU reading.
#[derive(Debug, Clone, Copy, Default)]
pub struct ImuReading {
    pub accelerometer: Vec3,  // m/s²
    pub gyroscope: Vec3,      // rad/s
    pub magnetometer: Option<Vec3>, // µT (if available)
    pub temperature: Option<f32>,   // °C (if available)
}

/// IMU data rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataRate {
    Hz10,
    Hz50,
    Hz100,
    Hz200,
    Hz400,
    Hz800,
    Hz1600,
}

/// Supported IMU chip types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImuChip {
    Bmi270,
    Mpu6050,
    Qmi8658,
    Lsm6dso,
    Custom,
}

/// Trait for IMU peripherals.
pub trait ImuPeripheral: Peripheral {
    /// Which IMU chip this driver supports.
    fn chip(&self) -> ImuChip;

    /// Configure data rate and full-scale range.
    fn configure(&mut self, rate: DataRate) -> Result<(), PeripheralError>;

    /// Read a single IMU sample.
    fn read(&mut self) -> Result<ImuReading, PeripheralError>;

    /// Read raw register values (for debugging).
    fn read_raw(&mut self, buf: &mut [u8]) -> Result<usize, PeripheralError>;

    /// Calibrate the sensor (zero-offset).
    fn calibrate(&mut self) -> Result<(), PeripheralError>;

    /// Check if new data is available.
    fn data_ready(&self) -> bool;
}
