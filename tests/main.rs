extern crate roa;
extern crate glium;

mod input {
    use glium::glutin::{Event, VirtualKeyCode};
    use roa::input::{KeyReader, Key};

    #[test]
    fn key_reader_test_interpreting_events() {
        let kr = KeyReader::new();
        // TODO Work on LOADING key settings, so that they are customizable
        // When we begin to LOAD the key settings, we will have to load it here too, to test it.
        assert_eq!(kr.interpret_code(&VirtualKeyCode::Z), Some(Key::X));
        assert_eq!(kr.interpret_code(&VirtualKeyCode::X), Some(Key::Y));
        assert_eq!(kr.interpret_code(&VirtualKeyCode::C), Some(Key::Z));
        assert_eq!(kr.interpret_code(&VirtualKeyCode::A), Some(Key::A));
        assert_eq!(kr.interpret_code(&VirtualKeyCode::S), Some(Key::B));
        assert_eq!(kr.interpret_code(&VirtualKeyCode::D), Some(Key::C));
        assert_eq!(kr.interpret_code(&VirtualKeyCode::Up), Some(Key::Up));
        assert_eq!(kr.interpret_code(&VirtualKeyCode::Down), Some(Key::Down));
        assert_eq!(kr.interpret_code(&VirtualKeyCode::Left), Some(Key::Left));
        assert_eq!(kr.interpret_code(&VirtualKeyCode::Right), Some(Key::Right));
        assert_eq!(kr.interpret_code(&VirtualKeyCode::Return), Some(Key::Start));
        assert_eq!(kr.interpret_code(&VirtualKeyCode::Space), Some(Key::Select));
        assert_eq!(kr.interpret_code(&VirtualKeyCode::W), None);
        assert_eq!(kr.interpret_code(&VirtualKeyCode::Q), None);
    }
}
