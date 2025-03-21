use std::{
    ffi::{c_uint, c_ulong},
    ptr,
    sync::Mutex,
};

use input_event::{InputEvent, Key, KeyboardEvent, KeyboardEventType};
use strum::IntoEnumIterator;
use x11::{
    xlib::{self, Display, XCloseDisplay, XFlush, XOpenDisplay},
    xtest::{XTestFakeButtonEvent, XTestFakeKeyEvent, XTestFakeRelativeMotionEvent},
};

use crate::{DeviceOutputError, VirtualDevice};

pub(crate) struct X11VirtualDevice {
    // TODO: probably can remove the mutex since it is only ever used in one thread
    display: Mutex<*mut Display>, // TODO: Consider the use of a blocking lock with tokio
}

// Assume display never gets closed
unsafe impl Send for X11VirtualDevice {}
unsafe impl Sync for X11VirtualDevice {}

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

impl Drop for X11VirtualDevice {
    fn drop(&mut self) {
        unsafe {
            XCloseDisplay(*self.display.lock().unwrap());
        }
    }
}

impl VirtualDevice for X11VirtualDevice {
    fn emit(&mut self, event: InputEvent) -> Result<(), DeviceOutputError> {
        let display = self.display.lock().unwrap();
        unsafe {
            emit(*display, event)?;
            flush(*display)?;
        }
        Ok(())
    }

    fn release_all(&mut self) -> Result<(), DeviceOutputError> {
        let display = self.display.lock().unwrap();
        let event_type = KeyboardEventType::KeyReleased;
        Key::iter()
            .try_for_each(|key| {
                let event = InputEvent::Keyboard(KeyboardEvent { event_type, key });
                unsafe { emit(*display, event) }
            })
            .and(unsafe { flush(*display) })
    }
}

unsafe fn emit(display: *mut Display, event: InputEvent) -> Result<(), DeviceOutputError> {
    match event {
        InputEvent::Keyboard(event) => {
            let is_press = match event.event_type {
                KeyboardEventType::KeyPressed | KeyboardEventType::KeyHeld => 1,
                KeyboardEventType::KeyReleased => 0,
            };
            unsafe {
                if XTestFakeKeyEvent(
                    display,
                    event.key.to_x11_keycode(display)?.into(),
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
                    if XTestFakeRelativeMotionEvent(display, dx, dy, 0, 0) == 0 {
                        eprintln!("Could not emit Xtest fake motion event");
                        return Err(DeviceOutputError::EmitError(
                            "Could not emit Xtest fake key event".into(),
                        ));
                    }
                }
            }
            input_event::MouseEvent::Button { event_type, button } => {
                let is_press = match event_type {
                    KeyboardEventType::KeyPressed | KeyboardEventType::KeyHeld => 1,
                    KeyboardEventType::KeyReleased => 0,
                };
                let button = button.to_x11_button_num();
                unsafe {
                    if XTestFakeButtonEvent(display, button, is_press, 0) == 0 {
                        eprintln!("Could not emit Xtest fake motion event");
                        return Err(DeviceOutputError::EmitError(
                            "Could not emit Xtest fake key event".into(),
                        ));
                    }
                }
            }
            input_event::MouseEvent::Scroll { axis, diff } => {
                let button = axis.to_x11_button_num(diff > 0);
                unsafe {
                    for _ in 0..diff.abs() {
                        tap_button(display, button, 0)?;
                    }
                }
            }
        },
    }
    Ok(())
}

unsafe fn tap_button(
    display: *mut Display,
    button: c_uint,
    delay: c_ulong,
) -> Result<(), DeviceOutputError> {
    let is_press = 1;
    if XTestFakeButtonEvent(display, button, is_press, delay) == 0 {
        eprintln!("Could not emit Xtest fake motion event");
        return Err(DeviceOutputError::EmitError(
            "Could not emit Xtest fake key event".into(),
        ));
    }
    let is_press = 0;
    if XTestFakeButtonEvent(display, button, is_press, delay) == 0 {
        eprintln!("Could not emit Xtest fake motion event");
        return Err(DeviceOutputError::EmitError(
            "Could not emit Xtest fake key event".into(),
        ));
    }
    Ok(())
}

unsafe fn flush(display: *mut Display) -> Result<(), DeviceOutputError> {
    unsafe {
        if XFlush(display) == 0 {
            eprintln!("Error in XFlush call");
            Err(DeviceOutputError::EmitError(
                "Could not flush Xtest fake key event".into(),
            ))
        } else {
            Ok(())
        }
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
