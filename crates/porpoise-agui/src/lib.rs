//! AG-UI 1.0 support for Porpoise: the protocol events an agent run
//! streams, Server-Sent Events encoding, and the translator that turns
//! Porpoise agent output into those events.
//!
//! AG-UI (https://ag-ui.com) is the open event protocol that connects any
//! agent to any app. With this crate a Porpoise agent run becomes a
//! standard AG-UI stream, consumable by the TypeScript/Python SDKs,
//! CopilotKit, or any spec-conformant client — over SSE, the relay's
//! WebSocket event bus, or the CLI's `agent agui-events` printer.
//!
//! ```no_run
//! use porpoise_agui::{AgUiInput, AgUiSession, encode_sse_batch};
//!
//! let mut session = AgUiSession::with_ids("thread-1", "run-1");
//! let mut events = session.begin();
//! events.extend(session.feed(AgUiInput::Text("Hello!".into())));
//! events.extend(session.feed(AgUiInput::Done { exit_code: Some(0) }));
//! let sse = encode_sse_batch(&events)?;
//! # Ok::<(), serde_json::Error>(())
//! ```

pub mod encode;
pub mod events;
pub mod session;

pub use encode::{AGUI_MEDIA_TYPE, encode_sse, encode_sse_batch};
pub use events::{
    BaseEvent, CustomEvent, Event, EventType, Interrupt, PROTOCOL_VERSION, ReasoningMessageContentEvent,
    ReasoningMessageEndEvent, ReasoningMessageStartEvent, Role, RunErrorEvent, RunFinishedEvent, RunFinishedOutcome,
    RunStartedEvent, StepFinishedEvent, StepStartedEvent, TextMessageContentEvent, TextMessageEndEvent,
    TextMessageStartEvent, ToolCallArgsEvent, ToolCallEndEvent, ToolCallResultEvent, ToolCallStartEvent,
};
pub use session::{AgUiInput, AgUiSession};
