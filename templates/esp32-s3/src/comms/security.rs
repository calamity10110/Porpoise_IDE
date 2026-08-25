//! Security module — authentication, rate limiting, and input validation.
//!
//! # Security Model
//!
//! The ESP32-S3 device uses a **pre-shared token** (PSK) model:
//! - On first boot, the device generates a random 32-byte token and stores it
//!   in NVS (non-volatile storage).
//! - The token is displayed on the device screen (or printed to UART) for the
//!   user to copy into the Porpoise IDE orchestrator config.
//! - Every WebSocket/HTTP connection must present this token in the first
//!   message (WS) or as an `Authorization: Bearer <token>` header (HTTP).
//! - Tokens are compared using constant-time comparison to prevent timing attacks.
//!
//! # Rate Limiting
//!
//! A simple sliding-window rate limiter prevents brute-force token guessing:
//! - Max 5 failed auth attempts per IP per 60-second window.
//! - After 5 failures, the IP is blocked for 300 seconds.
//!
//! # Input Validation
//!
//! All incoming commands are validated before dispatch:
//! - Peripheral names must be alphanumeric + underscore, max 32 chars.
//! - Action names must be alphanumeric + underscore, max 32 chars.
//! - Payload size is capped at 4096 bytes.
//! - Known dangerous actions (e.g., raw GPIO write) require explicit opt-in.

use core::fmt;

// ---------------------------------------------------------------------------
// Pre-Shared Token
// ---------------------------------------------------------------------------

/// Token length in bytes.
pub const TOKEN_LEN: usize = 32;

/// A pre-shared authentication token (raw bytes).
#[derive(Clone, Copy)]
pub struct AuthToken {
    bytes: [u8; TOKEN_LEN],
}

impl AuthToken {
    /// Create from raw bytes.
    pub fn from_bytes(bytes: [u8; TOKEN_LEN]) -> Self {
        Self { bytes }
    }

    /// Generate a token from a hex string (64 hex chars).
    /// Returns `None` if the string is not valid hex or wrong length.
    pub fn from_hex(hex: &str) -> Option<Self> {
        if hex.len() != TOKEN_LEN * 2 {
            return None;
        }
        let mut bytes = [0u8; TOKEN_LEN];
        for i in 0..TOKEN_LEN {
            bytes[i] = hex_nibble(hex.as_bytes()[i * 2])? << 4
                | hex_nibble(hex.as_bytes()[i * 2 + 1])?;
        }
        Some(Self { bytes })
    }

    /// Get raw bytes.
    pub fn as_bytes(&self) -> &[u8; TOKEN_LEN] {
        &self.bytes
    }

    /// Get hex representation (64 chars).
    pub fn to_hex<'a>(&self, buf: &'a mut [u8; TOKEN_LEN * 2]) -> &'a str {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for i in 0..TOKEN_LEN {
            buf[i * 2] = HEX[(self.bytes[i] >> 4) as usize];
            buf[i * 2 + 1] = HEX[(self.bytes[i] & 0x0F) as usize];
        }
        // SAFETY: we only wrote valid ASCII hex chars
        unsafe { core::str::from_utf8_unchecked(buf) }
    }

    /// Constant-time comparison against another token.
    /// Returns `true` if tokens match.
    pub fn constant_time_eq(&self, other: &AuthToken) -> bool {
        let mut diff: u8 = 0;
        for i in 0..TOKEN_LEN {
            diff |= self.bytes[i] ^ other.bytes[i];
        }
        diff == 0
    }

    /// Constant-time comparison against a hex string.
    pub fn eq_hex(&self, hex: &str) -> bool {
        match Self::from_hex(hex) {
            Some(other) => self.constant_time_eq(&other),
            None => false,
        }
    }
}

impl fmt::Debug for AuthToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AuthToken(***)")
    }
}

fn hex_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Rate Limiter
// ---------------------------------------------------------------------------

/// Maximum failed auth attempts per IP before lockout.
const MAX_ATTEMPTS: u8 = 5;

/// Window duration in seconds.
const WINDOW_SECS: u32 = 60;

/// Lockout duration in seconds.
const LOCKOUT_SECS: u32 = 300;

/// Maximum tracked IPs.
const MAX_CLIENTS: usize = 16;

/// Rate limiter entry for a single client IP.
#[derive(Debug, Clone)]
struct RateLimitEntry {
    ip: u32,
    failed_count: u8,
    first_fail_tick: u32,
    locked_until_tick: u32,
}

/// Sliding-window rate limiter with lockout.
pub struct RateLimiter {
    entries: heapless::Vec<RateLimitEntry, MAX_CLIENTS>,
}

impl RateLimiter {
    pub const fn new() -> Self {
        Self {
            entries: heapless::Vec::new(),
        }
    }

    /// Record a failed authentication attempt.
    /// Returns `Err(RateLimitError::Locked)` if the IP is currently locked out.
    /// Returns `Err(RateLimitError::TooManyAttempts)` if this attempt exceeds the limit
    /// (and locks the IP).
    pub fn record_failure(&mut self, ip: u32, now_secs: u32) -> Result<(), RateLimitError> {
        // Find or create entry
        let entry = if let Some(e) = self.entries.iter_mut().find(|e| e.ip == ip) {
            e
        } else {
            // Evict oldest if full
            if self.entries.len() >= MAX_CLIENTS {
                if let Some(oldest_idx) = self
                    .entries
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, e)| e.first_fail_tick)
                    .map(|(i, _)| i)
                {
                    let _ = self.entries.swap_remove(oldest_idx);
                }
            }
            let _ = self.entries.push(RateLimitEntry {
                ip,
                failed_count: 0,
                first_fail_tick: now_secs,
                locked_until_tick: 0,
            });
            self.entries.last_mut().unwrap()
        };

        // Check lockout
        if now_secs < entry.locked_until_tick {
            return Err(RateLimitError::Locked {
                remaining_secs: entry.locked_until_tick - now_secs,
            });
        }

        // Reset window if expired
        if now_secs.saturating_sub(entry.first_fail_tick) > WINDOW_SECS {
            entry.failed_count = 0;
            entry.first_fail_tick = now_secs;
        }

        entry.failed_count += 1;

        if entry.failed_count >= MAX_ATTEMPTS {
            entry.locked_until_tick = now_secs + LOCKOUT_SECS;
            Err(RateLimitError::TooManyAttempts {
                lockout_secs: LOCKOUT_SECS,
            })
        } else {
            Ok(())
        }
    }

    /// Record a successful authentication (resets failure count).
    pub fn record_success(&mut self, ip: u32) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.ip == ip) {
            e.failed_count = 0;
            e.locked_until_tick = 0;
        }
    }

    /// Check if an IP is currently locked out.
    pub fn is_locked(&self, ip: u32, now_secs: u32) -> bool {
        self.entries
            .iter()
            .any(|e| e.ip == ip && now_secs < e.locked_until_tick)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitError {
    /// IP is locked out.
    Locked { remaining_secs: u32 },
    /// Too many attempts — just locked.
    TooManyAttempts { lockout_secs: u32 },
}

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Locked { remaining_secs } => {
                write!(f, "IP locked. Try again in {}s", remaining_secs)
            }
            Self::TooManyAttempts { lockout_secs } => {
                write!(
                    f,
                    "Too many failed attempts. Locked for {}s",
                    lockout_secs
                )
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Input Validation
// ---------------------------------------------------------------------------

/// Maximum peripheral/action name length.
const MAX_NAME_LEN: usize = 32;

/// Maximum payload size in bytes.
pub const MAX_PAYLOAD_BYTES: usize = 4096;

/// Validate a peripheral or action name.
///
/// Rules:
/// - Only ASCII alphanumeric and underscore.
/// - Must start with a letter or underscore.
/// - Length 1..=32.
pub fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() || name.len() > MAX_NAME_LEN {
        return Err(ValidationError::InvalidName {
            reason: "length must be 1..=32",
        });
    }
    let bytes = name.as_bytes();
    // First char: letter or underscore
    if !bytes[0].is_ascii_alphabetic() && bytes[0] != b'_' {
        return Err(ValidationError::InvalidName {
            reason: "must start with letter or underscore",
        });
    }
    // Rest: alphanumeric or underscore
    for &b in &bytes[1..] {
        if !b.is_ascii_alphanumeric() && b != b'_' {
            return Err(ValidationError::InvalidName {
                reason: "only alphanumeric and underscore allowed",
            });
        }
    }
    Ok(())
}

/// Validate payload size.
pub fn validate_payload_size(len: usize) -> Result<(), ValidationError> {
    if len > MAX_PAYLOAD_BYTES {
        Err(ValidationError::PayloadTooLarge {
            size: len,
            max: MAX_PAYLOAD_BYTES,
        })
    } else {
        Ok(())
    }
}

/// Actions that require explicit security opt-in (compile-time feature flag).
const DANGEROUS_ACTIONS: &[&str] = &[
    "raw_gpio_write",
    "raw_i2c_write",
    "raw_spi_write",
    "factory_reset",
    "disable_auth",
];

/// Check if an action is in the dangerous list.
pub fn is_dangerous_action(action: &str) -> bool {
    DANGEROUS_ACTIONS.contains(&action)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    InvalidName { reason: &'static str },
    PayloadTooLarge { size: usize, max: usize },
    DangerousAction { action: &'static str },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName { reason } => write!(f, "Invalid name: {}", reason),
            Self::PayloadTooLarge { size, max } => {
                write!(f, "Payload {} bytes exceeds max {}", size, max)
            }
            Self::DangerousAction { action } => {
                write!(f, "Action '{}' requires security opt-in", action)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Auth Session
// ---------------------------------------------------------------------------

/// Tracks authentication state for a single connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthState {
    /// Not yet authenticated.
    Pending,
    /// Successfully authenticated.
    Authenticated,
    /// Authentication failed (wrong token).
    Rejected,
}

/// Per-connection auth session.
pub struct AuthSession {
    state: AuthState,
    client_ip: u32,
}

impl AuthSession {
    pub fn new(client_ip: u32) -> Self {
        Self {
            state: AuthState::Pending,
            client_ip,
        }
    }

    pub fn state(&self) -> AuthState {
        self.state
    }

    pub fn client_ip(&self) -> u32 {
        self.client_ip
    }

    pub fn is_authenticated(&self) -> bool {
        self.state == AuthState::Authenticated
    }

    /// Attempt authentication with a token string.
    /// Returns `true` if authenticated.
    pub fn authenticate(
        &mut self,
        token_hex: &str,
        device_token: &AuthToken,
        rate_limiter: &mut RateLimiter,
        now_secs: u32,
    ) -> Result<bool, RateLimitError> {
        // Check rate limit first
        if rate_limiter.is_locked(self.client_ip, now_secs) {
            return Err(RateLimitError::Locked {
                remaining_secs: 0, // approximate
            });
        }

        if device_token.eq_hex(token_hex) {
            self.state = AuthState::Authenticated;
            rate_limiter.record_success(self.client_ip);
            Ok(true)
        } else {
            self.state = AuthState::Rejected;
            rate_limiter.record_failure(self.client_ip, now_secs)?;
            Ok(false)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name() {
        assert!(validate_name("display").is_ok());
        assert!(validate_name("my_sensor_1").is_ok());
        assert!(validate_name("_private").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name("123bad").is_err());
        assert!(validate_name("has-dash").is_err());
        assert!(validate_name("has space").is_err());
        assert!(validate_name(&"a".repeat(33)).is_err());
    }

    #[test]
    fn test_validate_payload() {
        assert!(validate_payload_size(100).is_ok());
        assert!(validate_payload_size(4096).is_ok());
        assert!(validate_payload_size(4097).is_err());
    }

    #[test]
    fn test_token_hex_roundtrip() {
        let bytes = [0xABu8; 32];
        let token = AuthToken::from_bytes(bytes);
        let mut buf = [0u8; 64];
        let hex = token.to_hex(&mut buf);
        assert_eq!(hex.len(), 64);
        assert!(token.eq_hex(hex));
    }

    #[test]
    fn test_token_constant_time_eq() {
        let a = AuthToken::from_bytes([0xAA; 32]);
        let b = AuthToken::from_bytes([0xAA; 32]);
        let c = AuthToken::from_bytes([0xBB; 32]);
        assert!(a.constant_time_eq(&b));
        assert!(!a.constant_time_eq(&c));
    }

    #[test]
    fn test_rate_limiter() {
        let mut rl = RateLimiter::new();
        let ip = 0xC0A80001; // 192.168.0.1

        // 4 failures should be OK
        for i in 0..4 {
            assert!(rl.record_failure(ip, i * 10).is_ok());
        }

        // 5th failure should lock
        let result = rl.record_failure(ip, 50);
        assert!(matches!(
            result,
            Err(RateLimitError::TooManyAttempts { .. })
        ));

        // Subsequent attempts should be locked
        assert!(rl.is_locked(ip, 60));

        // Success resets
        rl.record_success(ip);
        assert!(!rl.is_locked(ip, 60));
    }

    #[test]
    fn test_auth_session() {
        let token = AuthToken::from_bytes([0x42; 32]);
        let mut rl = RateLimiter::new();
        let mut session = AuthSession::new(0xC0A80001);

        assert!(!session.is_authenticated());

        // Wrong token
        let result = session.authenticate("00".repeat(32).as_str(), &token, &mut rl, 0);
        assert_eq!(result.unwrap(), false);
        assert_eq!(session.state(), AuthState::Rejected);

        // Correct token
        let mut buf = [0u8; 64];
        let hex = token.to_hex(&mut buf);
        let mut session2 = AuthSession::new(0xC0A80002);
        let result = session2.authenticate(hex, &token, &mut rl, 10);
        assert_eq!(result.unwrap(), true);
        assert!(session2.is_authenticated());
    }
}
