use std::{env, io::Result, net::UdpSocket};
use evdev::EventType;
use rust_virtual_kvm::{dev::make_virtual_devices, net::recv_event};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let server_addr = parse_args(&args);

    let socket = UdpSocket::bind(server_addr).expect("Couldn't bind to address");

    let (mut keyboard, mut mouse) = make_virtual_devices()?;

    loop {
        if let Ok(event) = recv_event(&socket) {
            match event.event_type() {
                EventType::KEY => {
                    println!("Emitting: {event:?}");
                    keyboard.emit(&[event]).unwrap();
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

fn parse_args(args: &[String]) -> &str {
    if args.len() < 2 {
        panic!("Not enough arguments");
    }

    &args[1]
}
