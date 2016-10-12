#![allow(dead_code)] // TODO Remove when we get to working on the state system
use glium::{Surface, Display};
use time::{Duration, SteadyTime};
use std::rc::Rc;
// TODO Adapt to how we do things.

pub enum Update {
    Nothing,
    Halt,
    Push(Box<State>),
    Pop,
    Swap(Box<State>),
}

pub trait State {
    fn draw(&self);
}

pub struct StateMachine {
    screen: Display,
    stack: Vec<Box<State>>,
    // Last time fixed_update executed
    last_tick: SteadyTime,
    // Last time update executed
    last_time: SteadyTime,
    fixed_duration: Duration,
}

impl StateMachine {
    pub fn new(d: Display) -> StateMachine {
        StateMachine {
            screen: d,
            stack: vec![],
            last_tick: SteadyTime::now(),
            last_time: SteadyTime::now(),
            fixed_duration: Duration::milliseconds(1666),
        }
    }

    pub fn handle_update(&mut self, u: Update) -> bool {
        match u {
            Update::Nothing => false,
            Update::Halt => true,
            Update::Push(state) => { self.stack.push(state); true },
            Update::Pop => { self.stack.pop().unwrap(); true },
            Update::Swap(state) => {
                self.stack.pop();
                self.stack.push(state);
                true
            }
        }
    }

    pub fn update(&mut self) {
        // for state in self.stack.iter_mut().rev() {
        //     let dur = SteadyTime::now() - self.last_time;
        //     self.last_time = SteadyTime::now();
        //     let update = state.update(dur);
        //     if self.handle_update(update) { break }
        //
        //     let dur = SteadyTime::now() - self.last_tick;
        //     self.last_tick = SteadyTime::now();
        //     while dur > self.fixed_duration {
        //         dur = dur - self.fixed_duration;
        //         let update = state.fixed_update(self.fixed_duration);
        //         if self.handle_update(update) { break }
        //     }
        // }
    }
}
