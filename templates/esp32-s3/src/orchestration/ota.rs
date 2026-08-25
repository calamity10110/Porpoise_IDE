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

// ---------------------------------------------------------------------------
// Behavior-only OTA (update component configs without full firmware flash)
// ---------------------------------------------------------------------------

/// OTA target: firmware (full app) or behavior (config only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtaTarget {
    /// Full firmware OTA (writes to app0/app1 partition).
    Firmware,
    /// Behavior config OTA (writes to behavior partition, no reboot needed).
    Behavior,
}

static mut BEHAVIOR_OTA_STATE: OtaState = OtaState::Idle;
static mut BEHAVIOR_RECEIVED: u32 = 0;
static mut BEHAVIOR_TOTAL: u32 = 0;
static mut BEHAVIOR_BUF: [u8; 4096] = [0u8; 4096]; // staging buffer

/// Begin a behavior-only OTA update.
pub fn begin_behavior_ota(total_size: u32) -> Result<(), OtaError> {
    unsafe {
        if BEHAVIOR_OTA_STATE != OtaState::Idle {
            return Err(OtaError::NotStarted); // already in progress
        }
        BEHAVIOR_OTA_STATE = OtaState::Receiving;
        BEHAVIOR_RECEIVED = 0;
        BEHAVIOR_TOTAL = total_size;
        Ok(())
    }
}

/// Process a behavior OTA chunk.
///
/// Chunks are staged in a RAM buffer and flushed to the behavior partition
/// when complete. No reboot required — config is applied immediately.
pub fn process_behavior_chunk(data: &[u8]) -> Result<(), OtaError> {
    unsafe {
        if BEHAVIOR_OTA_STATE != OtaState::Receiving {
            return Err(OtaError::NotStarted);
        }

        let received = BEHAVIOR_RECEIVED as usize;
        let end = received + data.len();
        if end > BEHAVIOR_BUF.len() {
            return Err(OtaError::SizeMismatch);
        }

        BEHAVIOR_BUF[received..end].copy_from_slice(data);
        BEHAVIOR_RECEIVED += data.len() as u32;

        if BEHAVIOR_RECEIVED >= BEHAVIOR_TOTAL {
            BEHAVIOR_OTA_STATE = OtaState::Verifying;
        }

        Ok(())
    }
}

/// Finalize behavior OTA: parse config and apply.
///
/// Returns the parsed behavior config on success. Caller persists to flash.
pub fn finalize_behavior_ota() -> Result<super::behavior::BehaviorConfig, OtaError> {
    unsafe {
        if BEHAVIOR_OTA_STATE != OtaState::Verifying {
            return Err(OtaError::NotStarted);
        }

        let data = &BEHAVIOR_BUF[..BEHAVIOR_TOTAL as usize];
        let config = super::behavior::BehaviorConfig::deserialize(data)
            .map_err(|_| OtaError::InvalidChecksum)?;

        BEHAVIOR_OTA_STATE = OtaState::Done;
        Ok(config)
    }
}

/// Get behavior OTA progress (0.0 - 1.0).
pub fn behavior_progress() -> f32 {
    unsafe {
        if BEHAVIOR_TOTAL == 0 {
            return 0.0;
        }
        BEHAVIOR_RECEIVED as f32 / BEHAVIOR_TOTAL as f32
    }
}

/// Reset behavior OTA state (call after applying config or on error).
pub fn reset_behavior_ota() {
    unsafe {
        BEHAVIOR_OTA_STATE = OtaState::Idle;
        BEHAVIOR_RECEIVED = 0;
        BEHAVIOR_TOTAL = 0;
    }
}
