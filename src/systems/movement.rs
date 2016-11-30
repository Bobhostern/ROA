use specs;
use time::Duration;
use components::{Physical, Movement, MovementCommand};
use nalgebra::Vector2;

// We actually receive MovementCommands through a receiver in MovementComponent teling us how to move
// We then modify the SpatialComponent and any other components needed to signify this change.

// TODO Use physics when system becomes available
pub struct MovementSystem {

}

impl MovementSystem {
    pub fn new() -> MovementSystem {
        MovementSystem {

        }
    }
}

// TODO Switch from Duration to (Duration, Logger)
impl specs::System<Duration> for MovementSystem {
    fn run(&mut self, arg: specs::RunArg, _: Duration) {
        use specs::Join;

        let (mut phy, mvmt, ents) = arg.fetch(|w| {
            (w.write::<Physical>(), w.read::<Movement>(), w.entities())
        });

        for (p, m, _) in (&mut phy, &mvmt, &ents).iter() {
            // For now, we directly modify the position of spatial, but once physics
            // comes rolling around, we want to modify an object's velocity, not its position.
            let recv = m.receiver().lock().unwrap();
            while let Ok(mov) = recv.try_recv() {
                match mov {
                    MovementCommand::Up => p.acceleration.y = 64.0,
                    MovementCommand::Down => p.acceleration.y = -64.0,
                    MovementCommand::Left => p.acceleration.x = -64.0,
                    MovementCommand::Right => p.acceleration.x = 64.0,
                }
            }
        }
    }
}
