//! Foundation layer for the Porpoise system — core types, traits, errors, and configuration.
//! Every other crate depends on this module for its shared vocabulary.

/// Event bus — central pub/sub for system-wide events.
pub mod bus;
/// Configuration types — `AppConfig` and per-domain sub-configs.
pub mod config;
/// Error types — `PorpoiseError` enum with typed variants across all domains.
pub mod error;
/// Platform detection — compile-time OS/arch constants.
pub mod platform;
/// Serialization helpers — JSON and bincode round-trip utilities.
pub mod serialization;
/// Shared application state — `AppState` with interior mutability.
pub mod state;
/// Core traits — `Command`, `EventHandler`, and related abstractions.
pub mod traits;
/// Domain types — ID newtypes, event enums, capability model.
pub mod types;
/// Diff annotation — unified diff parsing with agent attribution.
pub mod diff;
/// Application version tracking — parsed from `CARGO_PKG_VERSION`.
pub mod version;

pub use bus::EventBus;
pub use error::{PorpoiseError, Result};
pub use platform::Platform;
pub use serialization::{from_bincode, from_json, to_bincode, to_json, to_json_pretty};
pub use state::AppState;
pub use types::{event::*, id::*};
pub use version::AppVersion;
