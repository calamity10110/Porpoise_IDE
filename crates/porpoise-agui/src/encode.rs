//! Server-Sent Events encoding for AG-UI streams.
//!
//! Mirrors the SSE path of `@ag-ui/encoder`: one `data:` frame per event,
//! blank-line separated, `text/event-stream` as the media type. The
//! protobuf transport the spec also defines is out of scope for Porpoise —
//! SSE/JSON is what the CLI, the GUI and the mobile companion consume.

use crate::events::Event;

/// The media type an AG-UI SSE stream serves.
pub const AGUI_MEDIA_TYPE: &str = "text/event-stream";

/// Encode one event as an SSE frame: `data: {json}\n\n`.
pub fn encode_sse(event: &Event) -> Result<String, serde_json::Error> {
    Ok(format!("data: {}\n\n", serde_json::to_string(event)?))
}

/// Encode a batch of events as consecutive SSE frames.
pub fn encode_sse_batch(events: &[Event]) -> Result<String, serde_json::Error> {
    let mut out = String::new();
    for event in events {
        out.push_str(&encode_sse(event)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{
        BaseEvent, Event, PROTOCOL_VERSION, RunFinishedEvent, RunStartedEvent, TextMessageContentEvent,
    };

    #[test]
    fn sse_frame_shape() {
        let event = Event::RunStarted(RunStartedEvent {
            base: BaseEvent::now(),
            thread_id: "t1".into(),
            run_id: "r1".into(),
            protocol_version: Some(PROTOCOL_VERSION.into()),
            parent_run_id: None,
        });
        let frame = encode_sse(&event).unwrap();
        assert!(frame.starts_with("data: {"));
        assert!(frame.ends_with("\n\n"));
        // The payload inside the frame is the event itself.
        let json = frame.trim_start_matches("data: ").trim_end();
        let back: Event = serde_json::from_str(json).unwrap();
        assert_eq!(back.event_type(), crate::events::EventType::RunStarted);
        assert_eq!(AGUI_MEDIA_TYPE, "text/event-stream");
    }

    #[test]
    fn batch_is_consecutive_frames() {
        let events = vec![
            Event::TextMessageContent(TextMessageContentEvent {
                base: BaseEvent::default(),
                message_id: "m1".into(),
                delta: "hi".into(),
            }),
            Event::RunFinished(RunFinishedEvent {
                base: BaseEvent::default(),
                thread_id: "t1".into(),
                run_id: "r1".into(),
                result: None,
                outcome: None,
            }),
        ];
        let batch = encode_sse_batch(&events).unwrap();
        assert_eq!(batch.matches("data: ").count(), 2);
        assert_eq!(batch.matches("\n\n").count(), 2);
    }
}
