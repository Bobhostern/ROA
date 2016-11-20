extern crate cgmath;
#[macro_use]
extern crate glium;
extern crate rusttype;
extern crate unicode_normalization;

use std::convert::AsRef;
use std::borrow::Cow;
use std::path::Path;
use glium::backend::{Facade, Context};
use std::rc::Rc;
use glium::Texture2d;

///! This crate is for the production of a nice pure-Rust font type for glium
// Allotta code stolen from glium_text
fn get_nearest_po2(mut x: u32) -> u32 {
    assert!(x > 0);
    x -= 1;
    x = x | (x >> 1);
    x = x | (x >> 2);
    x = x | (x >> 4);
    x = x | (x >> 8);
    x = x | (x >> 16);
    x + 1
}


struct Font<'a> {
    context: Rc<Context>,
    font: rusttype::Font<'a>,
    // TODO move cache to Text object, rather than Font
    cache: rusttype::gpu_cache::Cache,
    cache_texture: Texture2d
} // Store information and the font texture

impl<'a> Font<'a> {
    pub fn new<F: Facade, S: AsRef<Path>>(f: F, s: S, cs: (u32, u32)) -> Font<'a> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(s.as_ref()).expect(&format!("Failed to open: {}", s.as_ref().display()));
        let mut contents = vec![];
        file.read_to_end(&mut contents).unwrap();
        let font = rusttype::FontCollection::from_bytes(contents).into_font().expect("Font file is not of single font");
        let (tw, th) = (get_nearest_po2(cs.0), get_nearest_po2(cs.1));
        Font {
            context: f.get_context().clone(),
            font: font,
            cache: rusttype::gpu_cache::Cache::new(tw, th, 0.1, 0.1),
            cache_texture: glium::texture::Texture2d::with_format(
                f.get_context(),
                glium::texture::RawImage2d {
                    data: Cow::Owned(vec![128u8; tw as usize * th as usize]),
                    width: tw,
                    height: th,
                    format: glium::texture::ClientFormat::U8
                },
                glium::texture::UncompressedFloatFormat::U8,
                glium::texture::MipmapsOption::NoMipmap).unwrap()
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 4]
}

implement_vertex!(Vertex, position, tex_coords, color);

// TODO Make laying out text a bit more modular.
trait Layout {
    // Method ripped straight from gpu_cache example
    fn layout_text<'a>(font: &'a Font, scale: rusttype::Scale, width: u32, text: &str) -> Vec<rusttype::PositionedGlyph<'a>>;
}

struct SimpleText {
    color: [f32; 4], // Color of text
    text: String, // Text to layout

} // Holds the actual text we want to draw, and various settings we want to associate with the text
// Lays out the text directly

// Notice we DON'T provide a Program to draw it. We simply will tell you how we produce everything, and
// it's your job to implement a proper shader to draw it. Sorry, but we don't want to require anything
// that needs a Facade, or Context
// NOTE Changed. We will provide a Program, and we WILL require a Context
