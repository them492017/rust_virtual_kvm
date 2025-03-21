use std::{cmp::min, net::SocketAddr, time::Duration};

use thiserror::Error;

use crate::connection::{Connection, ConnectionError, ListenerHandles};

const INITIAL_RETRY_SECONDS: u64 = 1;
const MAX_RETRY_SECONDS: u64 = 180;
const RETRY_MUTLIPLIER: u64 = 2;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Connection error")]
    ConnectionError(#[from] ConnectionError),
}

pub async fn run(server_addr: SocketAddr, client_addr: SocketAddr) -> Result<(), ClientError> {
    let mut connection: Connection = Connection::default();
    let mut retry_seconds = INITIAL_RETRY_SECONDS;

    println!("Beginning main loop");
    loop {
        let transport = connection
            .connect(client_addr, server_addr)
            .await
            .inspect_err(|err| eprintln!("Could not connect to server: {}", err));

        if connection.is_connected && transport.is_ok() {
            retry_seconds = INITIAL_RETRY_SECONDS;
            // process events
            let ListenerHandles {
                input_event: input_event_processor,
                special_event: special_event_processor,
                cancellation_token,
            } = connection
                .spawn_listeners(transport.unwrap(), server_addr, client_addr)
                .await?;

            tokio::select! {
                result = input_event_processor => {
                    match result {
                        Ok(Ok(())) => {},
                        Ok(Err(err)) => eprintln!("Input event processor exited with error: {}", err),
                        Err(err) => {
                            eprintln!("Input event processor panicked: {}", err);
                            panic!();
                        },
                    }
                }
                result = special_event_processor => {
                    match result {
                        Ok(Ok(())) => {},
                        Ok(Err(err)) => eprintln!("Special event processor exited with error: {}", err),
                        Err(err) => {
                            eprintln!("Special event processor panicked: {}", err);
                            panic!();
                        },
                    }
                }
            }
            cancellation_token.cancel();
        } else {
            tokio::time::sleep(Duration::from_secs(retry_seconds)).await;
            retry_seconds = min(retry_seconds * RETRY_MUTLIPLIER, MAX_RETRY_SECONDS);
            println!(
                "Could not connect to server. Retrying in {} seconds",
                retry_seconds
            );
        }
    }
}
