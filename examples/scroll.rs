extern crate gl_bitfont;
extern crate glfw;
extern crate gl;

use glfw::Context;
use std::time;
use gl_bitfont::glutil::Framebuffer;

const ww : i32 = 20*8*8;
const wh : i32 = 12*10*8;

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
		.expect("Failed to initialize GLFW.");
	let (mut window, events) = glfw.create_window(ww as u32,wh as u32, "gl_bitfont example",
		glfw::WindowMode::Windowed)
		.expect("Failed to create GLFW window.");
    gl::load_with(|s| window.get_proc_address(s) as *const _);
	window.make_current();
	let f = gl_bitfont::osborne_font();
	let mut t = gl_bitfont::Terminal::new((20,12),(ww, wh),&f);
	t.options.scan_coverage = 0.2;
	window.set_key_polling(true);
	let text = include_str!("jabberwocky.txt");
	let mut text_iter = text.chars();
	while !window.should_close() {
		for _ in 0..1 {
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