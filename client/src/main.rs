mod config;
mod connection;
mod listeners;

use common::error::DynError;
use config::parse_args;
use listeners::{input_event::input_event_listener, special_event::special_event_processor};
use std::{cmp::max, env, time::Duration};

use crate::connection::Connection;

const INITIAL_RETRY_SECONDS: u64 = 1;
const MAX_RETRY_SECONDS: u64 = 180;
const RETRY_MUTLIPLIER: u64 = 2;

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let (server_addr, client_addr) = parse_args(env::args())?;
    let mut connection: Connection = Connection::default();
    let mut retry_seconds = INITIAL_RETRY_SECONDS;

    println!("Beginning main loop");
    loop {
        let transport = connection
            .connect(client_addr, server_addr)
            .await
            .inspect_err(|err| println!("Could not reconnect: {}", err));

        if connection.is_connected && transport.is_ok() {
            retry_seconds = INITIAL_RETRY_SECONDS;
            // process events
            println!("{:?}", connection);

            let key = connection.symmetric_key.clone();
            let input_event_processor =
                tokio::spawn(
                    async move { input_event_listener(key, client_addr, server_addr).await },
                );
            let special_event_processor =
                tokio::spawn(async move { special_event_processor(transport.unwrap()).await });

            tokio::select! {
                result = input_event_processor => {
                    match result {
                        Ok(Ok(())) => println!("Input event processor finished gracefully"),
                        Ok(Err(err)) => eprintln!("Input event processor exited with error: {}", err),
                        Err(err) => eprintln!("Input event processor panicked: {}", err),
                    }
                }
                result = special_event_processor => {
                    match result {
                        Ok(Ok(())) => println!("Special event processor finished gracefully"),
                        Ok(Err(err)) => eprintln!("Special event processor exited with error: {}", err),
                        Err(err) => eprintln!("Special event processor panicked: {}", err),
                    }
                }
            }
            // TODO: should broadcast CLOSED message or something when any finish
            connection.sender.send(()).await?;
        } else {
            tokio::time::sleep(Duration::from_secs(retry_seconds)).await;
            retry_seconds = max(retry_seconds * RETRY_MUTLIPLIER, MAX_RETRY_SECONDS);
        }
    }
}
