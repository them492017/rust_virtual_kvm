use std::{env::args, net::SocketAddr};

use common::{dev::pick_device, error::DynError};
use evdev::Device;

pub struct Config {
    pub server_address: SocketAddr,
    pub keyboard: Device,
    pub mouse: Device,
}

pub fn init() -> Config {
    let args: Vec<_> = args().collect();
    let Ok(server_address) = parse_args(&args) else {
        panic!("Could not parse server address")
    };
    let keyboard = pick_device("keyboard");
    let mouse = pick_device("mouse");

    Config {
        server_address,
        keyboard,
        mouse,
    }
}

pub fn parse_args(args: &[String]) -> Result<SocketAddr, DynError> {
    if args.len() < 2 {
        panic!("Not enough arguments. Please provide a server address");
    }

    Ok(args[1].parse()?)
}
