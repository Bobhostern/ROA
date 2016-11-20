use super::{Update, State, EventUpdate};
use time::{Duration, SteadyTime};
use glium::backend::{Facade, Context};
use std::rc::Rc;
use glium::{Frame, Program, IndexBuffer, VertexBuffer, index};
use slog::Logger;
use glium::glutin::Event;

pub struct PauseState {
}

impl PauseState {
    pub fn new() -> Box<PauseState> {
        let state = PauseState {

        };
        Box::new(state)
    }
}

impl State for PauseState {
    fn name(&self) -> &'static str { "Pause" }
    fn draw(&mut self, f: &mut Frame, _: &Rc<Context>, log: Logger) {
        use glium::Surface;

        f.clear_color(1.0, 0.5, 0.3, 1.0);
        // debug!(log, "Drawing Pause State TODO");
        // TODO Draw menu thingies
        // TODO Add font drawing support (fontae)
    }

    fn event(&mut self, e: Event, log: Logger) -> EventUpdate {
        use glium::glutin::{VirtualKeyCode, ElementState};
        match e {
            Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::P)) => EventUpdate::Update(Update::Pop),
            _ => EventUpdate::Halt
        }
    }
}
