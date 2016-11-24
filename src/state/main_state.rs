use super::{Update, State, EventUpdate};
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
use components::RectangleProvider;
use palette::{Colora, Rgba};

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
}

impl MainGameState {
    pub fn new() -> Box<MainGameState> {
        let (render_in, render_out) = systems::create_render_channel();
        let planner = {
            let mut w = World::new();
            // Register components
            w.register::<components::Spatial>();
            w.register::<components::VisualType>();

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
            keyreader: KeyReader::new()
        };
        Box::new(state)
    }
}

impl State for MainGameState {
    fn name(&self) -> &'static str { "MainGame" }

    fn setup(&mut self, c: &Rc<Context>, log: Logger) {
        use nalgebra;

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
                None)
        );

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
                None)
        );

        self.planner.add_system(render_sys, "render", 5);
    }

    fn update(&mut self, dura: Duration, _: Logger) -> Update {
        self.planner.dispatch(dura);
        Update::Nothing
    }

    fn draw(&mut self, target: &mut Frame, context: &Rc<Context>, _: Logger) {
        // Are we wasteful? hell yes, but what ev er.
        let ref scr_vb = self.vertexbuffers[0];
        let ref scr_ib = self.indexbuffers[0];
        let ref program = self.programs[0];

        target.clear_color(0.0, 0.1, 0.1, 1.0);
        self.game_tex.as_mut().unwrap().as_surface().clear_color(0.0, 0.0, 0.0, 0.0);

        self.renderer.draw(context, &mut self.game_tex.as_mut().unwrap().as_surface());

        {
            let ref game_tex = self.game_tex.as_ref().unwrap();
            let ref gui_tex = self.gui_tex.as_ref().unwrap();
            use glium::{Surface, uniforms};

            target.draw(scr_vb, scr_ib, program, &uniform! {
                game: game_tex.sampled().magnify_filter(uniforms::MagnifySamplerFilter::Linear),
                gui: gui_tex.sampled().magnify_filter(uniforms::MagnifySamplerFilter::Linear)
            }, &Default::default()).unwrap();
        }
    }

    fn event(&mut self, ev: Event, _: Logger) -> EventUpdate {
        use glium::glutin::{VirtualKeyCode, ElementState};
        // debug!(log, "{:?}", ev);
        // debug!(log, "{:?}", self.keyreader.interpret_event(&ev));

        match ev {
            // Event::Focused(false) => EventUpdate::Halt, // TODO change this to push a state
            Event::Focused(false) | Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::P)) => EventUpdate::Update(Update::Push(PauseState::new())),
             // the window has been closed by the user
            _ => EventUpdate::Halt
        }
    }
}
