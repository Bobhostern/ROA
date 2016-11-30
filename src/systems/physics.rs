use specs;
use time::Duration;
use components::{Spatial, Physical, Friction};
use nalgebra::Vector2;

pub struct PhysicsSystem {

}

impl PhysicsSystem {
    pub fn new() -> PhysicsSystem {
        PhysicsSystem {}
    }
}

impl specs::System<Duration> for PhysicsSystem {
    fn run(&mut self, arg: specs::RunArg, c: Duration) {
        use specs::Join;

        let (mut spat, mut phy, fric, ents) = arg.fetch(|w| {
            (w.write::<Spatial>(), w.write::<Physical>(), w.read::<Friction>(), w.entities())
        });

        for (s, p, _) in (&mut spat, &mut phy, &ents).iter() {
            // Integrate acceleration into velocity
            let delta = c.num_milliseconds() as f32 / 1000.0;
            p.velocity += p.acceleration * delta;
            s.pos += p.velocity * delta;
        }

        for (p, f, _) in (&mut phy, &fric, &ents).iter() {
            use nalgebra::{ApproxEq, Absolute, PartialOrder};
            if p.acceleration.approx_eq(&Vector2::new(0.0, 0.0)) {
                p.velocity -= f.decay * p.velocity;
            }
            p.acceleration -= f.mu * p.acceleration;
            if Vector2::abs(&p.acceleration).partial_lt(&f.still) {
                p.acceleration = Vector2::new(0.0, 0.0);
            }
        }
    }
}
