pub mod traits;
pub mod detector;
pub mod claude;
pub mod codex;
pub mod generic;
pub mod pool;
pub mod hook;
pub mod types;

pub use traits::{Agent, AgentHandle};
pub use detector::AgentDetector;
pub use pool::AgentPool;
pub use hook::HookServer;
pub use types::*;

