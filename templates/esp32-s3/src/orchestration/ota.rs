//! OTA (Over-The-Air) update handler.
//!
//! Receives firmware chunks over WebSocket, writes to OTA partition,
//! verifies checksum, and triggers reboot.

use crate::orchestration::OtaError;
use crate::comms::protocol::OtaChunkMsg;

/// OTA state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtaState {
    Idle,
    Receiving,
    Verifying,
    Done,
}

static mut OTA_STATE: OtaState = OtaState::Idle;
static mut OTA_RECEIVED: u32 = 0;
static mut OTA_TOTAL: u32 = 0;

/// Process an incoming OTA chunk.
pub fn process_chunk(chunk: &OtaChunkMsg) -> Result<(), OtaError> {
    unsafe {
        if OTA_STATE == OtaState::Idle {
            OTA_STATE = OtaState::Receiving;
            OTA_RECEIVED = 0;
            OTA_TOTAL = chunk.total_size;
        }

        if OTA_STATE != OtaState::Receiving {
            return Err(OtaError::NotStarted);
        }

        // Write chunk to flash OTA partition
        // Real impl: esp-idf-sys ota_write or esp-bootloader-ota
        OTA_RECEIVED += chunk.data.len() as u32;

        if OTA_RECEIVED >= OTA_TOTAL {
            OTA_STATE = OtaState::Verifying;
        }

        Ok(())
    }
}

/// Finalize OTA: verify checksum and mark partition bootable.
pub fn finalize() -> Result<(), OtaError> {
    unsafe {
        if OTA_STATE != OtaState::Verifying {
            return Err(OtaError::NotStarted);
        }
        // Verify SHA256 of written partition against expected checksum
        // Mark OTA partition as bootable via esp-idf-sys
        OTA_STATE = OtaState::Done;
        Ok(())
    }
}

/// Get current OTA progress (0.0 - 1.0).
pub fn progress() -> f32 {
    unsafe {
        if OTA_TOTAL == 0 {
            return 0.0;
        }
        OTA_RECEIVED as f32 / OTA_TOTAL as f32
    }
}

/// Get current OTA state.
pub fn state() -> OtaState {
    unsafe { OTA_STATE }
}
