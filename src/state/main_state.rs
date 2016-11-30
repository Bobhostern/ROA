use super::{Update, State, EventUpdate, CreateState};
use time::Duration;
use glium::backend::Context;
use glium::{Frame, Program, IndexBuffer, VertexBuffer, index};
use std::rc::Rc;
use specs::{Planner, World};
use glium::{Surface, Texture2d};
use {components, systems};
use super::super::graphics::Vertex;
use systems::{Renderer, RenderPipeIn};
use std::cell::RefCell;
use super::super::input::KeyReader;
use slog::Logger;
use glium::glutin::Event;
use super::pause::PauseState;
use components::{RectangleProvider, Movement, MoveSender, MovementCommand, Physical, Friction};
use palette::Rgba;
use std::any::Any;
use util::KeyTracker;

type CLikePointer<T> = Rc<RefCell<T>>;

pub struct MainGameState {
    planner: Planner<Duration>,
    // Using an array of Textures? Why? So that we can freely create textures and store them on the go.
    render_in: RenderPipeIn, // To init stuff on setup
    renderer: Renderer,

    // A cache for a buncha stuff
    game_tex: Option<Texture2d>,
    gui_tex: Option<Texture2d>,
    programs: Vec<Program>,
    vertexbuffers: Vec<VertexBuffer<Vertex>>,
    indexbuffers: Vec<IndexBuffer<u32>>,
    keyreader: KeyReader,
    grabbag: Vec<MoveSender>,
    keytracker: KeyTracker,
}

impl MainGameState {
    pub fn new() -> Box<MainGameState> {
        let (render_in, render_out) = systems::create_render_channel();
        let planner = {
            let mut w = World::new();
            // Register components
            w.register::<components::Spatial>();
            w.register::<components::VisualType>();
            w.register::<components::Movement>();
            w.register::<components::Physical>();
            w.register::<components::Friction>();

            // Create the Planner to run systems
            Planner::new(w, 4)
        };

        let state = MainGameState {
            planner: planner,
            render_in: render_in,
            renderer: Renderer::new(render_out, (1, 1)), // resize at setup
            game_tex: None,
            gui_tex: None,
            programs: vec![],
            indexbuffers: vec![],
            vertexbuffers: vec![],
            keyreader: KeyReader::new(),
            grabbag: vec![],
            keytracker: KeyTracker::new(),
        };
        Box::new(state)
    }
}

// TODO Convert MainGameState into a CreateState
// impl CreateState for MainGameState {
//     fn name() -> &'static str { "MainGame" }
//     fn create(context: &Rc<Context>, log: Logger) -> MainGameState {
//
//     }
// }

impl State for MainGameState {
    fn name(&self) -> &'static str { "MainGame" }

    fn setup(&mut self, c: &Rc<Context>, log: Logger) {
        use {nalgebra, image};
        use std::fs::File;

        info!(log, "Main Game is being initialized! Yay!");
        // Resize renderer to actual dimensions
        let (swidth, sheight) = c.get_framebuffer_dimensions();
        self.renderer.size_and_center(swidth, sheight);

        self.game_tex = Some(Texture2d::empty(c, swidth, sheight).unwrap());
        self.gui_tex = Some(Texture2d::empty(c, swidth, sheight).unwrap());
        // Link program
        let vertices = vec![
            Vertex { position: [-1.0, -1.0], color: [0.0, 0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] },
            Vertex { position: [1.0, -1.0], color: [0.0, 0.0, 0.0, 0.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [1.0, 1.0], color: [0.0, 0.0, 0.0, 0.0], tex_coords: [1.0, 1.0] },
            Vertex { position: [-1.0, 1.0], color: [0.0, 0.0, 0.0, 0.0], tex_coords: [0.0, 1.0] },
        ];
        let scr_vb = VertexBuffer::new(c, &vertices).unwrap();
        self.vertexbuffers.push(scr_vb);
        let indices = vec! [0u32, 1, 2, 0, 2, 3];
        let scr_ib = IndexBuffer::new(c, index::PrimitiveType::TrianglesList, &indices).unwrap();
        self.indexbuffers.push(scr_ib);
        let (vert_src, frag_src) = (include_str!("../screen.vert"), include_str!("../screen.frag"));
        let program = Program::from_source(c, &vert_src, &frag_src, None).unwrap();
        self.programs.push(program);

        let render_sys = systems::RenderSystem::new(self.render_in.clone());

        let (mov_1, msnd_1) = Movement::new();
        // Load the file containing the texture
        // TODO Make a single loading  service
        let image = match File::open("data/textures/Orbeus.png") {
            Ok(file) => {
                use std::io::BufReader;
                image::load(BufReader::new(file), image::PNG).ok()
            }
            Err(e) => {
                error!(log, "resource"=>"texture"; "Failed to load texture: {}", e);
                panic!("Failed to load {}: {}", "data/textures/Orbeus.png", e)
            }
        };
        // Setup entities
        self.planner.mut_world().create_now().with(
            components::Spatial {
                pos: nalgebra::Point2::new(32.0, 32.0),
                origin: nalgebra::Point2::new(16.0, 16.0),
                rotation: nalgebra::Rotation2::new(nalgebra::Vector1::new(45.0f32.to_radians()))
            }
        ).with(
            // TODO: Hide generating types
            components::VisualType::Still(
                RectangleProvider::new_from_size_components(32.0, 32.0, Rgba::new(1.0, 0.1, 0.1, 1.0).into()),
                image, 0.0)
        ).with(mov_1).with(Physical::new(1)).with(Friction::new(0.7, 0.7));
        self.grabbag.push(msnd_1);

        self.planner.mut_world().create_now().with(
            components::Spatial {
                pos: nalgebra::Point2::new(0.0, 0.0),
                origin: nalgebra::Point2::new(16.0, 16.0),
                rotation: nalgebra::one()
            }
        ).with(
            // TODO: Hide generating types
            components::VisualType::Still(
                RectangleProvider::new_from_size_components(32.0, 32.0, Rgba::new(0.1, 1.0, 0.1, 1.0).into()),
                None, -0.9)
        ).with(Physical::new(0));

        self.planner.add_system(systems::MovementSystem::new(), "movement", 7);
        self.planner.add_system(systems::PhysicsSystem::new(), "physics", 6);
        self.planner.add_system(render_sys, "render", 5);
    }

    fn update(&mut self, dura: Duration, _: Logger) -> Update {
        use time::SteadyTime;
        use input::Key;

        for k in self.keytracker.held_keys() {
            match k {
                Key::Up => self.grabbag[0].send(MovementCommand::Up).unwrap(),
                Key::Down => self.grabbag[0].send(MovementCommand::Down).unwrap(),
                Key::Left => self.grabbag[0].send(MovementCommand::Left).unwrap(),
                Key::Right => self.grabbag[0].send(MovementCommand::Right).unwrap(),
                _ => ()
            }
        }

        self.planner.dispatch(dura);
        let start = SteadyTime::now();
        self.planner.wait();
        let dura = SteadyTime::now() - start;
        println!("Time: {}", dura);
        self.renderer.buffer();
        Update::Nothing
    }

    fn draw(&mut self, target: &mut Frame, context: &Rc<Context>, _: Logger) {
        use glium::texture::DepthTexture2d;
        use glium::framebuffer::SimpleFrameBuffer;

        // Are we wasteful? hell yes, but what ev er.
        let ref scr_vb = self.vertexbuffers[0];
        let ref scr_ib = self.indexbuffers[0];
        let ref program = self.programs[0];
        let (ww, wh) = context.get_framebuffer_dimensions();

        target.clear_color_and_depth((0.0, 0.1, 0.1, 1.0), 1.0);

        let depth_game = DepthTexture2d::empty(context, ww, wh).unwrap();
        {
            let mut game_surface = SimpleFrameBuffer::with_depth_buffer(context, self.game_tex.as_ref().unwrap(), &depth_game).unwrap();
            game_surface.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
            self.renderer.draw(context, &mut game_surface);
        }

        {
            use glium;

            let ref game_tex = self.game_tex.as_ref().unwrap();
            let ref gui_tex = self.gui_tex.as_ref().unwrap();
            use glium::{Surface, uniforms};

            target.draw(scr_vb, scr_ib, program, &uniform! {
                game: game_tex.sampled().magnify_filter(uniforms::MagnifySamplerFilter::Linear),
                gui: gui_tex.sampled().magnify_filter(uniforms::MagnifySamplerFilter::Linear),
            }, &Default::default()).unwrap();
        }
    }

    fn event(&mut self, ev: Event, l: Logger) -> EventUpdate {
        use glium::glutin::{VirtualKeyCode, ElementState};
        use input::Key;
        // debug!(log, "{:?}", ev);
        // debug!(log, "{:?}", self.keyreader.interpret_event(&ev));

        match ev {
            // Event::Focused(false) => EventUpdate::Halt, // TODO change this to push a state
            Event::Focused(false) | Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::P)) => EventUpdate::Update(Update::Push(PauseState::new())),
             // the window has been closed by the user
            Event::KeyboardInput(ElementState::Pressed, _, Some(key)) => {match self.keyreader.interpret_code(&key) {
                Some(a) => self.keytracker.pressed(a),
                None => ()
            }; EventUpdate::Halt},
            Event::KeyboardInput(ElementState::Released, _, Some(key)) => {match self.keyreader.interpret_code(&key) {
                Some(a) => self.keytracker.released(a),
                None => ()
            }; EventUpdate::Halt},
            _ => EventUpdate::Halt
        }
    }
}
