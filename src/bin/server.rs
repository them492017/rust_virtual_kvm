use std::{env, thread};
use std::net::UdpSocket;

use evdev::Device;
use rust_virtual_kvm::net::send_event;
use rust_virtual_kvm::temp::pick_device;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let (server_addr, client_addr) = parse_args(&args);

    println!("Pick a keyboard");
    let mut keyboard = pick_device();
    println!("Pick a mouse");
    let mut mouse = pick_device();

    let socket = UdpSocket::bind(server_addr).expect("Could not bind socket");
    socket.connect(client_addr).expect("Could not connect to socket");

    let cloned_socket = socket.try_clone().unwrap();
    let keyboard_thread = thread::spawn(move || {thread_loop(&mut keyboard, &cloned_socket)});

    let cloned_socket = socket.try_clone().unwrap();
    let mouse_thread = thread::spawn(move || {thread_loop(&mut mouse, &cloned_socket)});

    // TODO: why join when they both loop infinietly
    keyboard_thread.join().expect("Keyboard thread panicked");
    mouse_thread.join().expect("Mouse thread panicked");

    Ok(())
}

fn thread_loop(device: &mut Device, socket: &UdpSocket) {
    device.grab().expect("Could not grab device: {device:?}");
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
