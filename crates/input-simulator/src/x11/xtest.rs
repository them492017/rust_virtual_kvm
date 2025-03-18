use std::{ptr, sync::Mutex};

use input_event::{InputEvent, Key, KeyboardEvent, KeyboardEventType};
use strum::IntoEnumIterator;
use x11::{
    xlib::{self, Display, XFlush, XOpenDisplay},
    xtest::{XTestFakeKeyEvent, XTestFakeRelativeMotionEvent},
};

use crate::{DeviceOutputError, VirtualDevice};

pub(crate) struct X11VirtualDevice {
    display: Mutex<*mut Display>, // TODO: Consider the use of a blocking lock with tokio
}

// Assume display never gets closed
unsafe impl Send for X11VirtualDevice {}

impl X11VirtualDevice {
    pub fn new() -> Self {
        let display = unsafe {
            match XOpenDisplay(ptr::null()) {
                d if d == ptr::null::<xlib::Display>() as *mut xlib::Display => {
                    panic!("Could not open x11 display")
                }
                display => display,
            }
        };

        Self {
            display: Mutex::new(display),
        }
    }
}

impl VirtualDevice for X11VirtualDevice {
    fn emit(&mut self, event: InputEvent) -> Result<(), DeviceOutputError> {
        let display = self.display.lock().unwrap();
        match event {
            InputEvent::Keyboard(event) => {
                let (key, is_press) = match event {
                    KeyboardEvent::KeyPressed(key) | KeyboardEvent::KeyReleased(key) => (key, 1),
                    KeyboardEvent::KeyHeld(key) => (key, 0),
                };
                unsafe {
                    if XTestFakeKeyEvent(
                        *display,
                        key.to_x11_keycode(*display)?.into(),
                        is_press,
                        0,
                    ) == 0
                    {
                        eprintln!("Could not emit Xtest fake key event");
                        return Err(DeviceOutputError::EmitError(
                            "Could not emit Xtest fake key event".into(),
                        ));
                    }
                }
            }
            InputEvent::Mouse(event) => match event {
                input_event::MouseEvent::Motion { axis, diff } => {
                    let (dx, dy) = match axis {
                        input_event::PointerAxis::Horizontal => (diff, 0),
                        input_event::PointerAxis::Vertical => (0, diff),
                    };
                    unsafe {
                        if XTestFakeRelativeMotionEvent(*display, dx, dy, 0, 0) == 0 {
                            eprintln!("Could not emit Xtest fake motion event");
                            return Err(DeviceOutputError::EmitError(
                                "Could not emit Xtest fake key event".into(),
                            ));
                        }
                    }
                }
                input_event::MouseEvent::Button { event_type, button } => todo!(),
                input_event::MouseEvent::Scroll { axis, diff } => todo!(),
            },
        }
        unsafe {
            if XFlush(*display) == 0 {
                eprintln!("Error in XFlush call");
                return Err(DeviceOutputError::EmitError(
                    "Could not flush Xtest fake key event".into(),
                ));
            }
        }
        Ok(())
    }

    fn release_all(&mut self) -> Result<(), DeviceOutputError> {
        Key::iter().try_for_each(|key| {
            let event = InputEvent::Keyboard(KeyboardEvent::KeyReleased(key));
            // TODO: consider refactoring to avoid repeatedly locking and unlocking
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
