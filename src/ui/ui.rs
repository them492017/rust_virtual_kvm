use std::{io::Write, net::SocketAddr};

use crate::{client::client_loop, common::error::DynError, server::server_loop};

fn get_input() -> String {
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub async fn ui() -> Result<(), DynError> {
    // TODO: Should start egui application
    print!("Server / Client: ");
    let chosen = get_input();

    match chosen.to_lowercase().as_str() {
        "server" => {
            print!("Server Address: ");
            let addr = get_input();
            let server_addr: SocketAddr = addr
                .as_str()
                .parse()
                .expect("Should provide a valid socket address");
            server_loop::run(server_addr).await?;
        }
        "client" => {
            print!("Server Address: ");
            let server_addr: SocketAddr = get_input()
                .as_str()
                .parse()
                .expect("Should provide a valid socket address");
            print!("Client Address: ");
            let client_addr: SocketAddr = get_input()
                .as_str()
                .parse()
                .expect("Should provide a valid socket address");
            client_loop::run(server_addr, client_addr).await?;
        }
        _ => {
            println!("Response was '{}'", chosen);
            panic!("Response should be one of 'server' or 'client'");
        }
    }

    Ok(())
}
