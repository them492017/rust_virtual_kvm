pub struct State {
    targets: Vec<String>,
    active_idx: usize,
}

impl State {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            active_idx: 0,
        }
    }

    pub fn add_target(&mut self, addr: String) {
        self.targets.push(addr);
    }

    pub fn set_target(&mut self, idx: usize) {
        if idx < self.targets.len() {
            self.active_idx = idx
        } else {
            panic!("New target out of bounds")
        }
    }

    pub fn active_index(&self) -> usize {
        self.active_idx
    }

    pub fn target_address(&self) -> &str {
        &self.targets[self.active_idx]
    }
}
