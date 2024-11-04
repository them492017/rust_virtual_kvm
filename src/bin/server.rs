use std::env;
use std::net::UdpSocket;

use evdev::Key;

use rust_virtual_kvm::net::send_event;
use rust_virtual_kvm::temp::pick_device;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let (server_addr, client_addr) = parse_args(&args);

    let mut device = pick_device();
    // check if the device has an ENTER key
    if device
        .supported_keys()
        .map_or(false, |keys| keys.contains(Key::KEY_ENTER))
    {
        println!("Supports ENTER");
    } else {
        println!("No ENTER");
    }

    let socket = UdpSocket::bind(server_addr).expect("Could not bind socket");
    socket.connect(client_addr).expect("Could not connect to socket");

    loop {
        for ev in device.fetch_events().unwrap() {
            println!("{ev:?}");
            if let Err(err) = send_event(&socket, &ev) {
                todo!("error handling on failed send event? {err:?}")
            }
        }
    }
}

fn parse_args(args: &[String]) -> (&str, &str) {
    if args.len() < 3 {
        panic!("Not enough arguments");
    }

    (&args[1], &args[2])
}
