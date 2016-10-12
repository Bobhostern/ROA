use specs;
use cgmath::{Point2, Vector3, Basis3, Decomposed};
use graphics::{Vertex, Index};
use image::DynamicImage;

#[derive(Clone)]
// This object physically exists at a point.
pub struct Spatial {
    pub pos: Point2<f32>,
    pub origin: Point2<f32>, // Relative to the bottom left corner
    pub transform: Decomposed<Vector3<f32>, Basis3<f32>>, // NOTE NEVER CHANGE disp! The render system already sets it.
}

impl specs::Component for Spatial {
    type Storage = specs::VecStorage<Spatial>;
}

#[derive(Clone)]
// The Render system stores what "type_index" means, and the other numbers
// are used to support animation.
pub enum VisualType {
    Sprite {
        // TODO Make sure the common interface supports Sprites (animated stuff), too.
    },
    // TODO Hide using a common interface
    Still(Vec<Vertex>, Option<Vec<Index>>, Option<DynamicImage>)
}

impl specs::Component for VisualType {
    type Storage = specs::VecStorage<VisualType>;
}
