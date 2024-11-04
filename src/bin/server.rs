use std::net::UdpSocket;

use evdev::Key;

use rust_virtual_kvm::net::send_event;
use rust_virtual_kvm::temp::pick_device;

fn main() -> std::io::Result<()> {
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

    let socket = UdpSocket::bind("127.0.0.1:8000").expect("Could not bind socket");
    socket.connect("127.0.0.1:8888").expect("Could not connect to socket");

    loop {
        for ev in device.fetch_events().unwrap() {
            println!("{ev:?}");
            if let Err(err) = send_event(&socket, &ev) {
                todo!("error handling on failed send event? {err:?}")
            }
        }
    }
}
