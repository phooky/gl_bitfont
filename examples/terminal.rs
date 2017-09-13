extern crate gl_bitfont;
extern crate glfw;
extern crate gl;

use glfw::Context;

fn main() {
	println!("Starting example");
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
		.expect("Failed to initialize GLFW.");
	let (mut window, events) = glfw.create_window(80*8, 24*10, "gl_bitfont example",
		glfw::WindowMode::Windowed)
		.expect("Failed to create GLFW window.");
    gl::load_with(|s| window.get_proc_address(s) as *const _);
	window.make_current();
	let f = gl_bitfont::osborne_font();
	let mut t = gl_bitfont::Terminal::new((80,24),&f);
	window.set_key_polling(true);
	window.set_char_polling(true);
	let mut cursor : (usize,usize) = (0,0);
	glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
	while !window.should_close() {
		unsafe {
			gl::ClearColor(0.8,0.0,0.0,1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);
		}
		t.render();

		glfw.wait_events();
		use glfw::Action;
		use glfw::Key::*;

        for (x, event) in glfw::flush_messages(&events) {
			match event {
				glfw::WindowEvent::Key(key, _, Action::Press, _) => match key {
					Escape => window.set_should_close(true),
					Down => cursor.1 = cursor.1.saturating_add(1),
					Up => cursor.1 = cursor.1.saturating_sub(1),
					Right  => cursor.0 = cursor.0.saturating_add(1),
					Left => cursor.0 = cursor.0.saturating_sub(1),
					_ => {},
				},
				glfw::WindowEvent::Char(c) => { 
					t.write_char_at(cursor.0,cursor.1,c);
					cursor.0 = cursor.0 + 1;
				},
				_ => {},
			}
		}
		window.swap_buffers();
	}
}