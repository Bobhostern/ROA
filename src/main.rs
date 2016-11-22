#[macro_use]
extern crate glium;
extern crate rodio;
extern crate cgmath;
extern crate collision;
extern crate specs;
extern crate palette;

extern crate nalgebra;
extern crate ncollide;

#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_stream;
extern crate slog_json;
extern crate slog_extra;

extern crate time;
extern crate image;
extern crate fontae;

mod graphics;
mod state;
mod input;
mod components;
mod systems;
mod font;
// TODO: Create tests for all NEW systems (go to the tests crate!)

fn main() {
    use glium::DisplayBuild;

    use std::fs::OpenOptions;
    use slog::DrainExt;
    use std::fs::DirBuilder;

    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(640, 480)
        .with_title("Spiritus [Name in Flux]")
        // .with_gl_robustness(glium::glutin::Robustness::RobustNoResetNotification)
        // .with_vsync()
        .build_glium().unwrap();

    DirBuilder::new().recursive(true).create("logs").unwrap();
    let log_path = format!("logs/{}.log", time::now().strftime("%Y-%m-%d_%H-%M-%S").unwrap());
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path).unwrap();

    let file = slog_stream::stream(file, slog_json::default());
    let console = slog_term::streamer().build();
    let drain = slog_extra::Async::new(slog::duplicate(file, console).fuse()).fuse();

    let root = slog::Logger::root(drain, o!());
    info!(root, "LoggingSystem initialized"; "build_ver" => env!("CARGO_PKG_VERSION"));
    // TDummy! We just need to use an Rc<Context> as our context, and we pass in a frame every update
    // and we good to go!

    let mut state_machine = state::StateMachine::new(&display, root.new(o!("service"=>"states")));
    state_machine.push_state(state::MainGameState::new());

    // Musika!
    use rodio::Source;
    let endpoint = rodio::get_default_endpoint().unwrap();
    let sink = rodio::Sink::new(&endpoint);

    let s1 = rodio::source::SineWave::new(440);
    let s2 = rodio::source::SineWave::new(880);
    let s3 = rodio::source::SineWave::new(220);
    sink.append(s1.mix(s2).mix(s3).amplify(0.5));

    while state_machine.stack_size() > 0 {
        // use std::{thread, time as stdtime};

        // listing the events produced by the window and waiting to be received
        for ev in display.poll_events() {
            use glium::glutin::{VirtualKeyCode, Event, ElementState};

            match ev {
                Event::Closed | Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::Escape)) => state_machine.quit(),
                ev => state_machine.event(ev)
            };
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

        state_machine.draw(&mut target);

        target.finish().unwrap();

        state_machine.update();

        // thread::sleep(stdtime::Duration::from_millis(16));
    }
}
