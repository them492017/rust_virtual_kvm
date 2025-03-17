use std::{ffi::c_uint, ptr};

use input_event::{InputEvent, Key, KeyboardEventType};
use strum::IntoEnumIterator;
use x11::{
    xlib::{self, Display, XFlush, XOpenDisplay},
    xtest::{XTestFakeKeyEvent, XTestFakeRelativeMotionEvent},
};

use crate::{DeviceOutputError, VirtualDevice};

pub(crate) struct X11VirtualDevice {
    display: *mut Display,
}

// TODO: ENSURE THIS IS A VALID IMPL
unsafe impl Send for X11VirtualDevice {}

impl X11VirtualDevice {
    pub fn new() -> Self {
        let display = unsafe {
            match XOpenDisplay(ptr::null()) {
                d if d == ptr::null::<xlib::Display>() as *mut xlib::Display => {
                    panic!()
                }
                display => display,
            }
        };

        Self { display }
    }
}

fn keycode_from_key(key: Key) -> c_uint {
    evdev::Key::from(key).code().into()
}

impl VirtualDevice for X11VirtualDevice {
    fn emit(&mut self, event: InputEvent) -> Result<(), DeviceOutputError> {
        match event {
            InputEvent::Keyboard { key, event_type } => {
                let keycode = keycode_from_key(key);
                let is_press = match event_type {
                    KeyboardEventType::KeyPressed | KeyboardEventType::KeyHeld => 1,
                    KeyboardEventType::KeyReleased => 0,
                };
                unsafe {
                    if XTestFakeKeyEvent(self.display, keycode, is_press, 0) == 0 {
                        return Err(DeviceOutputError::EmitError(
                            "Could not emit Xtest fake key event".into(),
                        ));
                    }
                }
            }
            InputEvent::Pointer { axis, diff } => {
                let (dx, dy) = match axis {
                    input_event::PointerAxis::Horizontal => (diff, 0),
                    input_event::PointerAxis::Vertical => (0, diff),
                };
                unsafe {
                    if XTestFakeRelativeMotionEvent(self.display, dx, dy, 0, 0) == 0 {
                        return Err(DeviceOutputError::EmitError(
                            "Could not emit Xtest fake key event".into(),
                        ));
                    }
                }
            }
        }
        unsafe {
            if XFlush(self.display) == 0 {
                return Err(DeviceOutputError::EmitError(
                    "Could not flush Xtest fake key event".into(),
                ));
            }
        }
        Ok(())
    }

    fn release_all(&mut self) -> Result<(), DeviceOutputError> {
        Key::iter().try_for_each(|key| {
            let event = InputEvent::Keyboard {
                key,
                event_type: KeyboardEventType::KeyPressed,
            };
            self.emit(event)
        })
    }
}

#[cfg(test)]
mod test {
    use std::{ptr, time::Duration};

    use x11::{
        xlib::{self, XFlush, XOpenDisplay},
        xtest::XTestFakeRelativeMotionEvent,
    };

    #[test]
    fn test_motion() {
        unsafe {
            let display = match XOpenDisplay(ptr::null()) {
                d if d == ptr::null::<xlib::Display>() as *mut xlib::Display => {
                    panic!()
                }
                display => display,
            };
            println!("Display: {:?}", display);
            (1..10).for_each(|_| {
                println!("Emitting motion event");
                XTestFakeRelativeMotionEvent(display, 10, 0, 10, 0);
                XFlush(display);
                std::thread::sleep(Duration::from_millis(400));
            })
        }
    }
}
