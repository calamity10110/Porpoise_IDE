//! Sensor abstraction layer.
//!
//! Higher-level sensor wrappers that combine raw peripheral reads
//! into useful data (e.g., orientation from IMU, gesture from touch).

pub mod orientation;
pub mod gesture;

/// 3D orientation (Euler angles in degrees).
#[derive(Debug, Clone, Copy, Default)]
pub struct Orientation {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

/// Gesture types recognized from touch input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    Tap,
    DoubleTap,
    LongPress,
    SwipeLeft,
    SwipeRight,
    SwipeUp,
    SwipeDown,
    PinchIn,
    PinchOut,
    None,
}
