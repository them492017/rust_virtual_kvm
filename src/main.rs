use std::net::SocketAddr;

use common::error::DynError;

mod client;
mod common;
mod server;
mod ui;

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--server".to_string()) {
        let a = parse_server_args(args)?;
        server::server_loop::run(a).await
    } else if args.contains(&"--client".to_string()) {
        let (a1, a2) = parse_client_args(args)?;
        client::client_loop::run(a1, a2).await
    } else {
        println!("Should run ui");
        ui::ui::ui();
        Ok(())
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
