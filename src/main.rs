#[macro_use]
extern crate glium;
extern crate rodio;
extern crate cgmath;
extern crate collision;
extern crate specs;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate time;
extern crate image;

mod components;
mod systems;
pub mod graphics;
pub mod state;
pub mod input;

fn main() {
    use glium::{DisplayBuild, Surface};
    use glium::backend::Facade;
    use cgmath::Transform;

    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(640, 480)
        .with_title("Reign of Aliens - Yes, the name is TEMPORARY")
        .build_glium().unwrap();

    env_logger::init().unwrap();

    // The ECS we use, SPECS, handles nearly EVERYTHING, from GUI to actors.
    // We just need to find a good way to do that.

    let mut planner = {
        let mut w = specs::World::new();
        // Register components
        w.register::<components::Spatial>();
        w.register::<components::VisualType>();

        // Create the Planner to run systems
        specs::Planner::new(w, 4)
    };
    let (swidth, sheight) = display.get_context().get_framebuffer_dimensions();
    debug!("Window dimensions: {}x{}", swidth, sheight);

    // Create drawing texture
    let game_render_texture = glium::Texture2d::empty(&display, swidth, sheight).unwrap();
    // let mut game_render_surface = game_render_texture.as_surface();
    game_render_texture.as_surface();
    let gui_render_texture = glium::Texture2d::empty(&display, swidth, sheight).unwrap();
    gui_render_texture.as_surface();

    let (render_in, render_out) = systems::create_render_channel();
    // Why clone render_in? Because it is how ANY rendering command gets to our renderer, and we happen
    // to need a separate Camera system that ALSO requires a render pipeline too (except it just adjusts
    // the View.)
    let render_sys = systems::RenderSystem::new(render_in.clone());
    let mut renderer = systems::Renderer::new(render_out, (swidth, sheight));

    use cgmath::{Rotation3, Basis3};

    // Setup entities
    planner.mut_world().create_now().with(
        components::Spatial {
            pos: cgmath::Point2::new(32.0, 32.0),
            origin: cgmath::Point2::new(16.0, 16.0),
            transform: cgmath::Decomposed {
                rot: Basis3::from_angle_z(cgmath::Deg(90.0)),
                ..cgmath::Decomposed::one()
            }
        }
    ).with(
        // TODO: Hide generating types
        components::VisualType::Still(vec![
            graphics::Vertex { position: [0.0, 0.0], color: [1.0,0.0,0.754,1.0], tex_coords: [0.0, 0.0] },
            graphics::Vertex { position: [32.0, 0.0], color: [1.0,0.0,0.0,1.0], tex_coords: [1.0, 0.0] },
            graphics::Vertex { position: [35.0, 35.0], color: [1.0,0.0,0.0,1.0], tex_coords: [1.0, 1.0] },
            graphics::Vertex { position: [0.0, 32.0], color: [1.0,0.0,0.0,1.0], tex_coords: [0.0, 1.0] }
        ], Some(vec![0, 1, 2, 2, 0, 3]), None)
    );

    planner.mut_world().create_now().with(
        components::Spatial {
            pos: cgmath::Point2::new(0.0, 0.0),
            origin: cgmath::Point2::new(16.0, 16.0),
            transform: cgmath::Decomposed {
                rot: Basis3::from_angle_z(cgmath::Deg(0.0)),
                ..cgmath::Decomposed::one()
            }
        }
    ).with(
        // TODO: Hide generating types
        components::VisualType::Still(vec![
            graphics::Vertex { position: [0.0, 0.0], color: [0.0,1.0,0.754,1.0], tex_coords: [0.0, 0.0] },
            graphics::Vertex { position: [32.0, 0.0], color: [0.0,1.0,0.0,1.0], tex_coords: [1.0, 0.0] },
            graphics::Vertex { position: [35.0, 35.0], color: [0.0,1.0,0.0,1.0], tex_coords: [1.0, 1.0] },
            graphics::Vertex { position: [0.0, 32.0], color: [0.0,1.0,0.0,1.0], tex_coords: [0.0, 1.0] }
        ], Some(vec![0, 1, 2, 2, 0, 3]), None)
    );

    // Register systems
    planner.add_system(render_sys, "render", 5);

    // <-- Do we use immediate mode GUI, because we want to use the drawing system all the way.
    let keyreader = input::KeyReader::new();
    // How the engine pauses when we're out of focus
    // TODO make a general Pause state (a.k.a. make a State system)
    let mut update = true;

    // Screen vertexbuffer and program
    //
    let vertices = vec![
        graphics::Vertex { position: [-1.0, -1.0], color: [0.0, 0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] },
        graphics::Vertex { position: [1.0, -1.0], color: [0.0, 0.0, 0.0, 0.0], tex_coords: [1.0, 0.0] },
        graphics::Vertex { position: [1.0, 1.0], color: [0.0, 0.0, 0.0, 0.0], tex_coords: [1.0, 1.0] },
        graphics::Vertex { position: [-1.0, 1.0], color: [0.0, 0.0, 0.0, 0.0], tex_coords: [0.0, 1.0] },
    ];
    let scr_vb = glium::VertexBuffer::new(&display, &vertices).unwrap();
    let indices = vec! [0u32, 1, 2, 0, 2, 3];
    let scr_ib = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap();
    let (vert_src, frag_src) = (include_str!("screen.vert"), include_str!("screen.frag"));
    let program = glium::Program::from_source(&display, &vert_src, &frag_src, None).unwrap();
    let mut last_tick = time::SteadyTime::now();
    loop {
        use std::{thread, time as stdtime};

        // listing the events produced by the window and waiting to be received
        for ev in display.poll_events() {
            use glium::glutin::Event;
            println!("{:?}", ev);
            println!("{:?}", keyreader.interpret_event(&ev));
            // TODO Create state machine and allow handling of Focused event
            match ev {
                Event::Closed => return,   // the window has been closed by the user
                Event::Focused(u) => update = u,
                _ => ()
            }
        }
        // Note about the controller
        //
        // There will be 12 buttons: A, B, C, X, Y, Z, START, SELECT, UP, DOWN, LEFT, and RIGHT
        // A B C - light med heavy bullet (Klay - bullet hell mode) punch (Wor - hack'n slash mode)
        // X Y Z - light med heavy beam (Klay) kick (Wor)
        // START - starts the game
        // SELECT - selects/confirms something (may merge with select)
        // Planned mappings:
        // UP DOWN LEFT RIGHT - corresponding arrow key
        // START - enter/return, SELECT - Space bar
        // A B C -> A S D, X Y Z -> Z X C

        // Move to system
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        renderer.draw(&display, &mut game_render_texture.as_surface());

        {
            use glium::Surface;

            target.draw(&scr_vb, &scr_ib, &program, &uniform! {
                game: game_render_texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear),
                gui: gui_render_texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
            }, &Default::default()).unwrap();
        }

        target.finish().unwrap();

        // Check planner
        let duration = time::SteadyTime::now() - last_tick;
        last_tick = time::SteadyTime::now();
        if update {
            planner.dispatch(duration);
        }

        thread::sleep(stdtime::Duration::from_millis(16));
    }
}
