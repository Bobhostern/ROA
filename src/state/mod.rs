use time::{Duration, SteadyTime};
use glium::backend::{Facade, Context};
use glium::{Frame, Texture2d};
use std::rc::Rc;
use glium::glutin::{Event};
use std::collections::HashMap;
// TODO Adapt to how we do things.
use slog::Logger;

mod main_state;
pub use self::main_state::MainGameState;

pub enum Update {
    Nothing,
    Halt,
    Push(Box<State>),
    Pop,
    Swap(Box<State>),
}

pub enum EventUpdate {
    PassOn(Event),
    Update(Update), // Implies Halt
    Halt
}

pub trait State {
    // For logging
    fn name(&self) -> &'static str;

    fn setup(&mut self, &Rc<Context>, Logger) { }
    fn teardown(&mut self, Logger) {}
    fn draw(&mut self, &mut Frame, &Rc<Context>, Logger) { }
    fn fixed_update(&mut self, Duration, Logger) -> Update { Update::Nothing }
    fn update(&mut self, Duration, Logger) -> Update { Update::Nothing }
    fn process_input(&mut self, _: Event, Logger) -> EventUpdate { EventUpdate::Halt }
}

// pub type Library<T> = HashMap<String, T>;

pub struct StateMachine {
    context: Rc<Context>,
    stack: Vec<Box<State>>,
    // Duration since fixed_update executed
    last_tick: Duration,
    // Last time update executed
    last_time: SteadyTime,
    fixed_duration: Duration,
    logger: Logger,
}

impl StateMachine {
    pub fn new<F: Facade>(d: &F, l: Logger) -> StateMachine {
        StateMachine {
            context: d.get_context().clone(),
            stack: vec![],
            last_tick: Duration::seconds(0),
            last_time: SteadyTime::now(),
            fixed_duration: Duration::milliseconds(1666),
            logger: l,
        }
    }

    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    pub fn push_state(&mut self, mut state: Box<State>) {
        let n = state.name();
        state.setup(&self.context, self.logger.new(o!("state"=>n)));
        debug!(self.logger, "Pushed state {:p}: {}", state, state.name());
        self.stack.push(state);
    }

    pub fn pop_state(&mut self) {
        match self.stack.pop() {
            Some(mut state) => {
                let n = state.name();
                state.teardown(self.logger.new(o!("state"=>n)));
                debug!(self.logger, "Popped state {:p}: {}", state, state.name());
            },
            None => ()
        };
    }

    pub fn handle_update(&mut self, u: Update) {
        match u {
            Update::Push(state) => self.push_state(state),
            Update::Pop => self.pop_state(),
            Update::Swap(state) => {
                self.pop_state();
                self.push_state(state);
            },
            _ => warn!(self.logger, "A Nothing or Halt update fell through!")
        }
    }

    pub fn update(&mut self) {
        let dur = SteadyTime::now() - self.last_time;
        self.last_time = SteadyTime::now();
        self.last_tick = self.last_tick + dur;
        let mut st = self.last_tick.clone(); // Clone so we can borrow self.
        let mut new_updates = vec![]; // The new updates to add.
        for state in self.stack.iter_mut() {
            let n = state.name();
            let update = state.update(dur, self.logger.new(o!("state"=>n)));
            match update {
                Update::Nothing => (),
                Update::Halt => break,
                update => {new_updates.push(update); break}
            };
            while st > self.fixed_duration {
                st = st - self.fixed_duration;
                let update = state.fixed_update(self.fixed_duration.clone(), self.logger.new(o!("state"=>n)));
                match update {
                    // Update::Nothing => (),
                    Update::Halt => break,
                    update => {new_updates.push(update); break}
                };
            }
        }
        for update in new_updates.into_iter() {
            self.handle_update(update);
        }
        self.last_tick = st; // Now merge the edits
    }

    pub fn draw(&mut self, f: &mut Frame) {
        for state in self.stack.iter_mut().rev() {
            let n = state.name();
            state.draw(f, &self.context, self.logger.new(o!("state"=>n)));
        }
    }

    pub fn process_input(&mut self, e: Event) {
        let mut event = e;
        let mut new_updates = vec![];
        for state in self.stack.iter_mut() {
            let n = state.name();
            match state.process_input(event.clone(), self.logger.new(o!("state"=>n))) {
                EventUpdate::PassOn(e) => event = e,
                EventUpdate::Update(u) => match u {
                    Update::Nothing | Update::Halt => {
                        error!(self.logger, "Invalid state return for events")
                    },
                    update => {new_updates.push(update); break}
                },
                EventUpdate::Halt => break
            }
        }
        for update in new_updates.into_iter() {
            self.handle_update(update);
        }
    }
}
