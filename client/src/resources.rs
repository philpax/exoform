use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

pub struct RenderParameters {
    pub wireframe: bool,
}

#[derive(Default)]
pub struct OccupiedScreenSpace {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub _bottom: f32,
}

pub struct NetworkState {
    shutdown: Arc<AtomicBool>,
    pub tx: Arc<Mutex<Vec<shared::GraphCommand>>>,
    pub rx: Arc<Mutex<Vec<shared::GraphChange>>>,
}

impl NetworkState {
    pub fn new(
        shutdown: Arc<AtomicBool>,
        tx: Arc<Mutex<Vec<shared::GraphCommand>>>,
        rx: Arc<Mutex<Vec<shared::GraphChange>>>,
    ) -> Self {
        Self { shutdown, tx, rx }
    }

    pub fn send(&mut self, commands: &[shared::GraphCommand]) {
        self.tx.lock().unwrap().extend_from_slice(commands);
    }
}

impl Drop for NetworkState {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}
