//! Audio peripheral traits (microphone input, speaker output).

use super::{Peripheral, PeripheralError};

/// Audio sample format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    /// 8-bit unsigned PCM
    PcmU8,
    /// 16-bit signed PCM
    PcmI16,
    /// 24-bit signed PCM
    PcmI24,
    /// 32-bit float
    PcmF32,
}

/// Audio stream configuration.
#[derive(Debug, Clone, Copy)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u8,
    pub format: SampleFormat,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            format: SampleFormat::PcmI16,
        }
    }
}

/// Trait for audio input (microphone).
pub trait AudioInput: Peripheral {
    /// Configure the audio input stream.
    fn configure(&mut self, config: AudioConfig) -> Result<(), PeripheralError>;

    /// Read audio samples into buffer. Returns number of bytes written.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, PeripheralError>;

    /// Start continuous recording (DMA-based).
    fn start_stream(&mut self) -> Result<(), PeripheralError>;

    /// Stop continuous recording.
    fn stop_stream(&mut self) -> Result<(), PeripheralError>;

    /// Get current configuration.
    fn config(&self) -> AudioConfig;
}

/// Trait for audio output (speaker).
pub trait AudioOutput: Peripheral {
    /// Configure the audio output stream.
    fn configure(&mut self, config: AudioConfig) -> Result<(), PeripheralError>;

    /// Write audio samples. Returns number of bytes consumed.
    fn write(&mut self, buf: &[u8]) -> Result<usize, PeripheralError>;

    /// Start continuous playback (DMA-based).
    fn start_stream(&mut self) -> Result<(), PeripheralError>;

    /// Stop continuous playback.
    fn stop_stream(&mut self) -> Result<(), PeripheralError>;

    /// Set volume (0-255).
    fn set_volume(&mut self, volume: u8) -> Result<(), PeripheralError>;

    /// Get current configuration.
    fn config(&self) -> AudioConfig;
}
