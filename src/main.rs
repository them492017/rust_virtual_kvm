use std::net::SocketAddr;

use rust_virtual_kvm::client;
use rust_virtual_kvm::common::error::DynError;
use rust_virtual_kvm::server;
use rust_virtual_kvm::ui;

const WELCOME_STRING: &str = r#"
====================================================================
    Software KVM
====================================================================
    In order to have access to keyboard and mouse
    devices, this binary should be run with root permissions
====================================================================
"#;

#[tokio::main]
async fn main() -> Result<(), DynError> {
    println!("{WELCOME_STRING}");
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--server".to_string()) {
        let server_addr = parse_server_args(args)?;
        server::server_loop::run(server_addr).await
    } else if args.contains(&"--client".to_string()) {
        let (server_addr, client_addr) = parse_client_args(args)?;
        client::client_loop::run(server_addr, client_addr).await
    } else {
        ui::ui::ui().await
    }
}

pub fn parse_client_args(args: Vec<String>) -> Result<(SocketAddr, SocketAddr), DynError> {
    if args.len() < 4 {
        panic!("Not enough arguments. Please provide a server address and client address followed by a flag");
    }

    Ok((args[1].parse()?, args[2].parse()?))
}

pub fn parse_server_args(args: Vec<String>) -> Result<SocketAddr, DynError> {
    if args.len() < 3 {
        panic!("Not enough arguments. Please provide a server address and client address followed by a flag");
    }

    Ok(args[1].parse()?)
}
