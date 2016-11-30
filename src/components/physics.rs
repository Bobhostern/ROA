use nalgebra::{Vector2};
use specs;

pub struct Physical {
    world_id: i32,
    pub velocity: Vector2<f32>,
    pub acceleration: Vector2<f32>,
}

impl Physical {
    pub fn new(id: i32) -> Physical {
        Physical {
            velocity: Vector2::new(0.0, 0.0),
            acceleration: Vector2::new(0.0, 0.0),
            world_id: id
        }
    }

    pub fn id(&self) -> i32 { self.world_id }
}

impl specs::Component for Physical {
    type Storage = specs::VecStorage<Physical>;
}

// TODO Add limiter, after which velocity is zeroed.
pub struct Friction {
    pub mu: f32,
    pub decay: f32,
    pub still: Vector2<f32>,
}

impl Friction {
    pub fn new(mu: f32, d: f32) -> Friction {
        Friction {
            mu: mu,
            decay: d,
            still: Vector2::new(0.00001, 0.00001),
        }
    }

    pub fn new_with_still(mu: f32, d: f32, s: Vector2<f32>) -> Friction {
        Friction {
            mu: mu,
            decay: d,
            still: s,
        }
    }
}

impl specs::Component for Friction {
    type Storage = specs::HashMapStorage<Friction>;
}
