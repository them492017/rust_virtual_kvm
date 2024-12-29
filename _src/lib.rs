pub mod net;
pub mod temp;
pub mod dev;
pub mod state;

use evdev::{EventType, InputEvent};
use serde::{Deserialize, Serialize};

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

