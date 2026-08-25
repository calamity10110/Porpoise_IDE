//! Orientation estimation from IMU data.
//!
//! Simple complementary filter combining accelerometer and gyroscope.

use crate::sensors::Orientation;
use crate::peripherals::imu::ImuReading;

/// Complementary filter state.
pub struct OrientationFilter {
    alpha: f32,
    roll: f32,
    pitch: f32,
    yaw: f32,
    dt: f32,
}

impl OrientationFilter {
    pub fn new(alpha: f32, dt: f32) -> Self {
        Self { alpha, roll: 0.0, pitch: 0.0, yaw: 0.0, dt }
    }

    /// Update filter with new IMU reading.
    pub fn update(&mut self, reading: &ImuReading) {
        // Accelerometer angles
        let accel_roll = reading.accelerometer.y.atan2(reading.accelerometer.z);
        let accel_pitch = (-reading.accelerometer.x)
            .atan2((reading.accelerometer.y.powi(2) + reading.accelerometer.z.powi(2)).sqrt());

        // Gyroscope integration
        self.roll = self.alpha * (self.roll + reading.gyroscope.x * self.dt)
            + (1.0 - self.alpha) * accel_roll;
        self.pitch = self.alpha * (self.pitch + reading.gyroscope.y * self.dt)
            + (1.0 - self.alpha) * accel_pitch;
        self.yaw += reading.gyroscope.z * self.dt;
    }

    /// Get current orientation estimate.
    pub fn orientation(&self) -> Orientation {
        Orientation {
            roll: self.roll.to_degrees(),
            pitch: self.pitch.to_degrees(),
            yaw: self.yaw.to_degrees(),
        }
    }

    /// Reset filter to zero.
    pub fn reset(&mut self) {
        self.roll = 0.0;
        self.pitch = 0.0;
        self.yaw = 0.0;
    }
}
