//! Gesture recognition from touch events.

use crate::sensors::Gesture;
use crate::peripherals::touch::TouchEvent;

const TAP_MAX_MS: u64 = 300;
const LONG_PRESS_MS: u64 = 500;
const SWIPE_MIN_DIST: u16 = 50;

/// Gesture recognizer state machine.
pub struct GestureRecognizer {
    start_x: u16,
    start_y: u16,
    start_time: u64,
    touching: bool,
    last_gesture: Gesture,
}

impl GestureRecognizer {
    pub fn new() -> Self {
        Self {
            start_x: 0, start_y: 0, start_time: 0,
            touching: false, last_gesture: Gesture::None,
        }
    }

    /// Feed a touch event, returns recognized gesture (or None).
    pub fn feed(&mut self, event: &TouchEvent, now_ms: u64) -> Gesture {
        match event {
            TouchEvent::Down { x, y, .. } => {
                self.start_x = *x;
                self.start_y = *y;
                self.start_time = now_ms;
                self.touching = true;
                Gesture::None
            }
            TouchEvent::Up { .. } if self.touching => {
                self.touching = false;
                let dt = now_ms.saturating_sub(self.start_time);
                // TODO: compute dx/dy from last Move event
                if dt < TAP_MAX_MS {
                    Gesture::Tap
                } else if dt >= LONG_PRESS_MS {
                    Gesture::LongPress
                } else {
                    Gesture::None
                }
            }
            _ => Gesture::None,
        }
    }

    pub fn last(&self) -> Gesture {
        self.last_gesture
    }
}
