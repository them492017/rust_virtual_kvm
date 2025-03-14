use uuid::Uuid;

#[derive(Debug)]
pub enum ServerMessage {
    Cycle,
    ClientDisconnect { id: Uuid },
}
