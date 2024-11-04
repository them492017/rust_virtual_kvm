use std::{io::Result, net::UdpSocket, thread::sleep, time::Duration};
use evdev::EventType;
use rust_virtual_kvm::{dev::make_virtual_devices, net::recv_event};

fn main() -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:8888").expect("Couldn't bind to address");

    let (mut keyboard, mut mouse) = make_virtual_devices()?;

    loop {
        if let Ok(event) = recv_event(&socket) {
            match event.event_type() {
                EventType::KEY => {
                    println!("Emitting: {event:?}");
                    keyboard.emit(&[event]).unwrap();
                    sleep(Duration::from_millis(500));
                }
                EventType::RELATIVE => {
                    println!("Emitting: {event:?}");
                    mouse.emit(&[event]).unwrap();
                }
                _ => {
                    // let t = event.event_type();
                    // println!("Ignoring {t:?} events");
                }
            }
        } else {
            println!("Error in received event");
        }
    }
}
