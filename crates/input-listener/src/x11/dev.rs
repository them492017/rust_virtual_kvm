use std::{future::Future, pin::Pin};

use evdev::InputEvent;

use crate::{DeviceInputError, DeviceInputStreamTrait};

impl DeviceInputStreamTrait for evdev::EventStream {
    fn next_event(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<InputEvent, DeviceInputError>> + Send + '_>> {
        Box::pin(async { Ok(self.next_event().await?) })
    }

    fn grab_device(&mut self) -> Result<(), DeviceInputError> {
        self.device_mut().grab()?;
        Ok(())
    }

    fn ungrab_device(&mut self) -> Result<(), DeviceInputError> {
        self.device_mut().ungrab()?;
        Ok(())
    }
}

// TODO: move this into a new method for a generic struct
pub fn pick_device(name: &str) -> evdev::Device {
    use std::io::prelude::*;

    let mut devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();
    // readdir returns them in reverse order from their eventN names for some reason
    devices.reverse();
    for (i, d) in devices.iter().enumerate() {
        println!("{}: {}", i, d.name().unwrap_or("Unnamed device"));
    }
    print!("Select the {} device [0-{}]: ", name, devices.len());
    let _ = std::io::stdout().flush();
    let mut chosen = String::new();
    std::io::stdin().read_line(&mut chosen).unwrap();
    let n = chosen.trim().parse::<usize>().unwrap();
    devices.into_iter().nth(n).unwrap()
}
