use std::net::SocketAddr;

use tokio::net::TcpListener;

pub struct ServerResource {
    pub listener: TcpListener,
}

impl ServerResource {
    pub async fn new(addr: SocketAddr) -> Self {
        // TODO: remove unwrap
        let listener = TcpListener::bind(addr).await.unwrap();
        println!("Bound TCP listener to {}", addr);

        ServerResource { listener }
    }
}
