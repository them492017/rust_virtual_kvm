use std::net::UdpSocket;
use evdev::InputEvent;

use crate::SerializableInputEvent;

pub fn send_event(socket: &UdpSocket, event: &InputEvent) -> bincode::Result<()> {
    let serialised_event: SerializableInputEvent = event.into();
    let encoded: Vec<u8> = bincode::serialize(&serialised_event)?;
    socket.send(&encoded)?;
    Ok(())
}

pub fn recv_event(socket: &UdpSocket) -> bincode::Result<InputEvent> {
    let mut buf = [0;128];
    socket.recv(&mut buf).unwrap();

    let serialised_event: SerializableInputEvent = bincode::deserialize(&buf)?;

    Ok(serialised_event.into())
}
