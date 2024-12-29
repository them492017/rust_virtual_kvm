use std::{env, io::Result, net::UdpSocket};
use evdev::{Device, EventType};
use rust_virtual_kvm::{dev::make_virtual_devices, net::recv_event, state::State, temp::pick_device};

#[tokio::main]
async fn main() -> Result<()> {
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
    let listener = TcpListener::bind(server_addr).await?;
    let cloned_state = Arc::clone(&state);
    tokio::spawn(async move {
        loop {
            let (socket, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                    continue;
                }
            };

            let cloned_state = Arc::clone(&cloned_state);
            if let Err(e) = process_socket(socket, cloned_state).await {
                eprintln!("Failed to process socket: {}", e);
            }
        }
    });

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

// TODO: same function is used in server.rs
fn parse_args(args: &[String]) -> (&str, &str) {
    if args.len() < 3 {
        panic!("Not enough arguments");
    }

    (&args[1], &args[2])
}
