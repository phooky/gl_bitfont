extern crate gl_bitfont;
extern crate glfw;
extern crate gl;

use glfw::Context;
use std::time;
use gl_bitfont::glutil::Framebuffer;

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
		.expect("Failed to initialize GLFW.");
	let (mut window, events) = glfw.create_window(20*8*4, 12*10*4, "gl_bitfont example",
		glfw::WindowMode::Windowed)
		.expect("Failed to create GLFW window.");
    gl::load_with(|s| window.get_proc_address(s) as *const _);
	window.make_current();
	let f = gl_bitfont::osborne_font();
	let mut t = gl_bitfont::Terminal::new((20,12),(20*8*4, 12*10*4),&f);
	t.options.scan_coverage = 0.3;
	window.set_key_polling(true);
	let text = include_str!("perec.txt");
	let mut text_iter = text.chars();
	while !window.should_close() {
		for _ in 0..2 {
			match text_iter.next() {
				Some(c) => t.write_char(c),
				None => text_iter = text.chars(),
			}
		}
		t.render();
		t.flip_phase();
		window.swap_buffers();

		glfw.poll_events();
		use glfw::Action;
		use glfw::Key::*;

        for (x, event) in glfw::flush_messages(&events) {
			match event {
				glfw::WindowEvent::Key(key, _, Action::Press, _) => match key {
					Escape => window.set_should_close(true),
					_ => {},
				},
				_ => {},
			}
		}
	}
}