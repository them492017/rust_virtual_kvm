use chacha20poly1305::ChaCha20Poly1305;
use common::dev::{make_keyboard, release_all};
use evdev::{uinput::VirtualDevice, Device};

pub struct State {
    targets: Vec<EventTarget>,
    active_idx: usize,
    own_idx: usize, // own_idx: index of the client
    devices: Vec<Device>,
    kbd: VirtualDevice,
    cipher: ChaCha20Poly1305
}

impl State {
    pub fn new(server_addr: String, devices: Vec<Device>, cipher: ChaCha20Poly1305) -> Self {
        let server = EventTarget::Server {
            address: server_addr,
        };
        Self {
            targets: vec![server.clone()],
            active_idx: 0,
            own_idx: 0,
            devices,
            kbd: make_keyboard().unwrap(),
            cipher,
        }
    }

    pub fn add_target(&mut self, addr: String) {
        let target = EventTarget::Client { address: addr };
        self.targets.push(target);
    }

    pub fn set_target(&mut self, idx: usize) {
        if idx == self.own_idx {
            self.ungrab_devices()
        }

        if idx < self.targets.len() {
            self.active_idx = idx
        }

        if self.active_idx == self.own_idx {
            self.grab_devices();
            let _ = release_all(&mut self.kbd);
        }
    }

    pub fn grab_devices(&mut self) {
        self.devices.iter_mut().for_each(|dev| dev.grab().unwrap())
    }

    pub fn ungrab_devices(&mut self) {
        self.devices.iter_mut().for_each(|dev| dev.ungrab().unwrap())
    }

    pub fn active_index(&self) -> usize {
        self.active_idx
    }

    pub fn active_target(&self) -> &EventTarget {
        &self.targets[self.active_idx]
    }

    pub fn target_address(&self) -> &str {
        self.active_target().address()
    }

    pub fn cipher(&self) -> &ChaCha20Poly1305 {
        &self.cipher
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum EventTarget {
    Server { address: String },
    Client { address: String },
}

impl EventTarget {
    pub fn address(&self) -> &str {
        match self {
            EventTarget::Server { address: addr } => &addr,
            EventTarget::Client { address: addr } => &addr,
        }
    }
}
