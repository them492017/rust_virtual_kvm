mod state;

use core::panic;

use std::net::UdpSocket;
use std::sync::Arc;
use std::{env, thread};
use tokio::net::TcpStream;

use evdev::Device;
use common::net::{self, send_event, send_message, Message};
use common::state::{EventTarget, State};
use common::dev::pick_device;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let (server_addr, client_addr) = parse_args(&args);

    // Open devices
    println!("Pick a keyboard");
    let keyboard = pick_device();
    println!("Pick a mouse");
    let mouse = pick_device();

    let devices: Vec<_> = vec![&keyboard.physical_path(), &mouse.physical_path()]
        .iter_mut()
        .map(|path| Device::open(path.unwrap()).unwrap())
        .collect();

    // Initialise state
    let mut state = State::new(server_addr.to_string(), devices);
    state.add_target(client_addr.to_string());
    let state = Arc::new(RwLock::new(state));

    // Initialise tcp socket
    let listener = TcpStream::connect(client_addr);
    let cloned_state = Arc::clone(&state);

    // TODO: connect tcp socket to server and sync state

    // tokio::spawn(async move {
    //     loop {
    //         let (socket, _) = match listener.accept().await {
    //             Ok(conn) => conn,
    //             Err(e) => {
    //                 eprintln!("Failed to accept connection: {}", e);
    //                 continue;
    //             }
    //         };
    //
    //         let cloned_state = Arc::clone(&cloned_state);
    //         if let Err(e) = process_socket(socket, cloned_state).await {
    //             eprintln!("Failed to process socket: {}", e);
    //         }
    //     }
    // });

    // Initialise input event sockets
    let socket = UdpSocket::bind(server_addr).expect("Could not bind event socket");

    // Start input listener threads
    let threads = vec![keyboard, mouse].into_iter().map(|dev| {
        let cloned_socket = socket.try_clone().unwrap();
        let cloned_state = Arc::clone(&state);
        thread::spawn(move || thread_loop(dev, &cloned_socket, cloned_state))
    });

    // TODO: why join when they both loop infinietly
    threads.for_each(|thread| {
        thread.join().expect("Keyboard thread panicked");
    });

    Ok(())
}

fn thread_loop(mut device: Device, socket: &UdpSocket, state: Arc<RwLock<State>>) {
    // TODO: try to make state read only from here
    let rt = tokio::runtime::Runtime::new().unwrap();
    loop {
        for ev in device.fetch_events().unwrap() {
            println!("{ev:?}");
            rt.block_on(async {
                let state = state.read().await;
                match state.active_target() {
                    EventTarget::Server { .. } => {
                        // Don't send over udp
                        println!("Server");
                    }
                    EventTarget::Client { address: addr } => send_event(&socket, addr, &ev)
                        .expect("error handling on failed send event? {err:?}"),
                }
            });
        }
    }
}

fn parse_args(args: &[String]) -> (&str, &str) {
    if args.len() < 3 {
        panic!("Not enough arguments");
    }

    (&args[1], &args[2])
}
