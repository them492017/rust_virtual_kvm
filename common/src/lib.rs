pub mod net;
pub mod dev;
pub mod state;
pub mod event;

use evdev::{EventType, InputEvent};
use serde::{Deserialize, Serialize};

// TODO: make the event type more general for multiplatform support
#[derive(Serialize, Deserialize)]
struct SerializableInputEvent {
    type_: EventType,
    code: u16,
    value: i32,
}

impl From<&InputEvent> for SerializableInputEvent {
    fn from(value: &InputEvent) -> Self {
        SerializableInputEvent {
            type_: value.event_type(),
            code: value.code(),
            value: value.value()
        }
    }
}

impl Into<InputEvent> for SerializableInputEvent {
    fn into(self) -> InputEvent {
        InputEvent::new(self.type_, self.code, self.value)
    }
}

