use std::sync::Arc;

use chacha20poly1305::ChaCha20Poly1305;
use common::{error::DynError, net::Message, tcp2::TokioTcpTransport};
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{self, Receiver},
        RwLock,
    },
};

use crate::{
    client::{Client, ClientInterface},
    handlers::monitor::monitor_client,
    state::State,
    CHANNEL_BUF_LEN,
};

pub async fn handle_client(
    stream: TcpStream,
    state: Arc<RwLock<State<ChaCha20Poly1305>>>,
) -> Result<(), DynError> {
    let transport = TokioTcpTransport::new(stream);
    let (tx, rx) = mpsc::channel(CHANNEL_BUF_LEN);
    let client: Client<ChaCha20Poly1305> = Client::connect(transport, tx).await?;

    // add client to state vector
    let client_idx = {
        let mut state_writer = state.write().await;
        state_writer.add_client(client)
    };

    process_events(client_idx, state, rx).await
}

async fn process_events(
    client_idx: usize,
    state: Arc<RwLock<State<ChaCha20Poly1305>>>,
    mut receiver: Receiver<Message>,
) -> Result<(), DynError> {
    // handle the sending and processing of heartbeat events
    let state_clone = state.clone();
    tokio::spawn(async move {
        let _ = monitor_client(client_idx, state_clone).await;
    });
    // get event from channel
    while let Some(event) = receiver.recv().await {
        let state_reader = state.read().await;
        println!("Received from TCP: {:?}", event);
        // TODO: add error handling
        match event {
            Message::InputEvent { event, .. } => {
                println!("{:?}", event);
                unimplemented!("Handling input events over tcp is unimplemented")
            }
            Message::ClipboardChanged { content } => {
                println!("New clipboard item: [{:?}]", content);
                let message = Message::ClipboardChanged { content };
                state_reader
                    .send_message_to_client(client_idx, message)
                    .await?;
            }
            _ => {}
        }
    }

    // when channel is closed return err or return () on shutdown
    println!("Connection failed / channel closed or something???");
    Ok(()) // TODO: connection failed?
}
