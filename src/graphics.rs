///! Just anything about graphics

pub type Index = u32;

#[derive(Copy, Clone, Debug)]
// A Vec<Vertex> is stored for each combination of action and frame
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4], // RGBA tint for this vertex
    pub tex_coords: [f32; 2], // Texture position
}

implement_vertex!(Vertex, position, color, tex_coords);
