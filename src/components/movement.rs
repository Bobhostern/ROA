use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use specs;

#[derive(Clone, Copy, Debug)]
pub enum MovementCommand {
    // These move us in polar directions.
    Up, Down, Left, Right
}

pub struct Movement {
    rec: Mutex<Receiver<MovementCommand>>
}

pub type MoveSender = Sender<MovementCommand>;

impl Movement {
    pub fn new() -> (Movement, MoveSender) {
        let (tx, rx) = channel();
        (Movement { rec: Mutex::new(rx) }, tx)
    }

    pub fn receiver(&self) -> &Mutex<Receiver<MovementCommand>> { &self.rec }
}

impl specs::Component for Movement {
    type Storage = specs::VecStorage<Movement>;
}
