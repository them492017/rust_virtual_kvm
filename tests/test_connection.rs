use std::{error::Error, net::SocketAddr, time::Duration};

use client::connection::Connection;
use server::actors::server::resource::ServerResource;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

async fn connect_client(
    client_addr: SocketAddr,
    server_addr: SocketAddr,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let mut conn = Connection::default();
    conn.connect(client_addr, server_addr).await?;
    Ok(conn.is_connected)
}

#[tokio::test]
async fn given_matching_ip_addresses_should_successfully_form_a_connection() {
    // Given
    let client_addr: SocketAddr = "127.0.0.1:15342".parse().unwrap();
    let server_addr: SocketAddr = "127.0.0.1:15343".parse().unwrap();

    let server = ServerResource::new(server_addr).await;

    let (client_sender, _rx1) = mpsc::channel(10);
    let (client_message_sender, _rx2) = mpsc::channel(10);

    let cancellation_token = CancellationToken::new();

    // When
    tokio::spawn(async move {
        server
            .start_listening(client_sender, client_message_sender, cancellation_token)
            .await
    });
    tokio::time::sleep(Duration::from_millis(100)).await;
    let response = connect_client(client_addr, server_addr).await;

    // Then
    assert!(response.is_ok());
    assert!(response.unwrap()); // is connected
}
