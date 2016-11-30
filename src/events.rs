use input::{Key, ElementState};
use std::sync::mpsc::{Sender, Receiver, channel};
use bus::{Bus, BusReader};
use std::any::Any;

// Used for spreading events
pub type EventBusOut = BusReader<Event>;
type EventBusIn = Bus<Event>;
// Used for collecting events
type EventPipeOut = Receiver<Event>;
pub type EventPipeIn = Sender<Event>;

#[derive(Debug, Clone)]
pub enum Event {
    Key(Key, ElementState),
}

impl From<(Key, ElementState)> for Event {
    fn from(o: (Key, ElementState)) -> Event {
        Event::Key (
            o.0,
            o.1
        )
    }
}

pub trait Subscriber {
    fn is_subscribed(&self, &Event) -> bool;
}

pub struct AnySubscriber;
impl Subscriber for AnySubscriber {
    fn is_subscribed(&self, e: &Event) -> bool { true }
}

pub struct KeySubscriber;
impl Subscriber for KeySubscriber {
    fn is_subscribed(&self, e: &Event) -> bool {
        match e.clone() {
            Event::Key(_, _) => true,
            // _ => false
        }
    }
}

pub trait JoinSubscriber: Subscriber {
    type Target;
    fn join<S: Subscriber>(self, o: S) -> Self::Target;
}

// Represents multiple subscribers joined together
pub struct MultiSubscriber {
    subs: Vec<Box<Subscriber>>
}

impl Subscriber for MultiSubscriber {
    fn is_subscribed(&self, e: &Event) -> bool {
        self.subs.iter().map(|x| x.is_subscribed(e)).any(|x| x)
    }
}

impl<T: 'static> JoinSubscriber for T where T: Subscriber {
    type Target = Box<MultiSubscriber>;
    fn join<S: Subscriber + 'static>(self, o: S) -> Box<MultiSubscriber> {
        Box::new(MultiSubscriber {
            subs: vec![Box::new(self), Box::new(o)]
        })
    }
}

// Collects subsystems that handle events, and pushes important events to subscribers
pub struct EventStation {
    sources: Vec<EventPipeOut>,
    buses: Vec<(Box<Subscriber>, EventBusIn)>,
}

impl EventStation {
    fn add_unboxed_subscriber<S: Subscriber + 'static>(&mut self, s: S, cache_size: usize) -> EventBusOut {
        // TODO Optimise this method to share a bus if the subscriber is exactly the same with another impl
        let mut new_bus = Bus::new(cache_size);
        let new_receiver = new_bus.add_rx();
        self.buses.push((Box::new(s), new_bus));
        new_receiver
    }

    fn add_subscriber(&mut self, s: Box<Subscriber>, cache_size: usize) -> EventBusOut {
        // TODO Optimise this method to share a bus if the subscriber is exactly the same with another impl
        let mut new_bus = Bus::new(cache_size);
        let new_receiver = new_bus.add_rx();
        self.buses.push((s, new_bus));
        new_receiver
    }

    fn add_source(&mut self) -> EventPipeIn {
        let (tx, rx) = channel();
        self.sources.push(rx);
        tx
    }

    fn update(&mut self) {
        let events = self.sources.iter().flat_map(|x| x.try_recv().ok()).collect::<Vec<_>>();
        for &mut (ref sub, ref mut bus) in self.buses.iter_mut() {
            let events = events.iter().cloned().filter(|x| sub.is_subscribed(x)).collect::<Vec<_>>();
            for event in events.into_iter() {
                bus.broadcast(event);
            }
        }
    }
}
