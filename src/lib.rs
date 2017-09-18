//! gl_bitfont renders simple, old-school pixel fonts 
extern crate gl;

pub mod glutil;

use glutil::Framebuffer;

use gl::types::*;

// Shader sources
static FONT_VS_SRC: &'static str = include_str!("font_vertex.glsl");
static FONT_FS_SRC: &'static str = include_str!("font_fragment.glsl");
static BLOOM_VS_SRC: &'static str = include_str!("bloom_vertex.glsl");
static BLOOM_FS_SRC: &'static str = include_str!("bloom_fragment.glsl");
static CRT_VS_SRC: &'static str = include_str!("crt_vertex.glsl");
static CRT_FS_SRC: &'static str = include_str!("crt_fragment.glsl");

static mut beam_program : GLuint = 0;
static mut bloom_program : GLuint = 0;
static mut crt_program : GLuint = 0;
static mut programs_loaded : bool = false;

/// The BitFont trait includes all the information necessary to
/// load a bit font into the GL engine. All fonts are presumed
/// to be ASCII and within a contiguous range in [0,255].
pub trait BitFont<'a> {
	/// Character cell size, in pixels
	fn cell_size_px(&self) -> (u8,u8);
	/// Spacing between cells, vertically and horizontall, in pixels
	fn intercell_px(&self) -> (u8,u8);
	/// Index of lowest and highest ASCII values present in font.
	fn bounds(&self) -> (i16, i16);
	/// Raw bit texture data. This represented as a byte map, with a 
	/// width of (max-min)*cell width and a height equal to the cell 
	/// height.
	fn texture(&self) -> &'a [u8];
}

// All the information necessary to render this font.
pub struct LoadedFont {
	cell_size : (u8,u8),
	intercell : (u8,u8),
	bounds : (i16,i16),
	gl_texture : GLuint,
}

pub struct Color {
	r : f32,
	g : f32, 
	b : f32,
	a : f32,
}

pub struct DisplayOptions {
	pub fg_color : Color,
	pub bg_color : Color,
	pub scan_coverage : f32,
}

impl DisplayOptions {
	pub fn new() -> DisplayOptions {
		DisplayOptions {
			fg_color : Color { r:0.0, g:1.0, b:0.0, a:1.0 },
			bg_color : Color { r:0.0, g:0.0, b:0.1, a:1.0 },
			scan_coverage : 0.5,
		}
	}
}

/// The TerminalGLState contains all the information necessary to render a terminal
/// display.
pub struct TerminalGLState {
	beam_vao : GLuint, //< The vertex array object used to render characters
	crt_vao : GLuint, //< The vertex array object used to blit a rectangle
	data_texture : GLuint, //< The texture representing the characters to be displayed
	crt_fb : [Framebuffer;2], //< Two framebuffers to ping-pong the phosphor state of the CRT
	beam_fb : [Framebuffer;2], //< Two framebuffers to ping-pong the bloomed beam trace
	crt_phase : usize, //< The index of the CRT framebuffer that is currently displayed.
}

/// An area to render text into, along with the current contents
/// of the text
pub struct Terminal<'a> {
	pub term_dim : (i32, i32),  /// The dimensions, in characters, of this terminal
	pub render_dim : (i32, i32),
	pub data : Vec<u8>, /// The data to be displayed
	font : &'a LoadedFont,
	pub cursor : (i32, i32),
	pub options : DisplayOptions,
	gl : TerminalGLState,
}

impl<'a> Terminal<'a> {
	/// Creating a new terminal presumes that you have already set up a valid GL context
	pub fn new(term_dim : (i32, i32), render_dim : (i32, i32), font : &'a LoadedFont) -> Terminal {
		let mut gl = TerminalGLState {
			beam_vao : 0,
			crt_vao : 0,
			data_texture : 0,
			crt_fb : [
				Framebuffer::new(render_dim).unwrap(), 
				Framebuffer::new(render_dim).unwrap()
			],
			beam_fb : [
				Framebuffer::new(render_dim).unwrap(), 
				Framebuffer::new(render_dim).unwrap()
			],
			crt_phase : 0,
		};

		let mut data : Vec<u8> = Vec::new();
		let d = (term_dim.0 as f32, term_dim.1 as f32);
		data.resize(term_dim.0 as usize * term_dim.1 as usize, 32);

		// Set up VAO/VBO for beam
		let beam_vertices : [GLfloat; 4 * 4] = [
			1.0 , 1.0,   d.0, 0.0,
			-1.0, 1.0,   0.0, 0.0, 
			1.0 ,-1.0,   d.0, d.1,
			-1.0,-1.0,   0.0, d.1,
		];
		let mut beam_vbo : GLuint = 0;
		unsafe {
			gl::GenVertexArrays(1, &mut gl.beam_vao);
			gl::BindVertexArray(gl.beam_vao);
			gl::GenBuffers(1, &mut beam_vbo);
			gl::BindBuffer(gl::ARRAY_BUFFER,beam_vbo);
			gl::BufferData(gl::ARRAY_BUFFER, 4*4*4, 
				beam_vertices.as_ptr() as *const _, gl::STATIC_DRAW);
			gl::UseProgram(beam_program);
			let pos_attrib = glutil::attrib_loc(beam_program,"position");
            gl::EnableVertexAttribArray(pos_attrib as GLuint);
            gl::VertexAttribPointer(pos_attrib as GLuint, 2, gl::FLOAT, gl::FALSE,
                                    4*4, std::ptr::null());
            let tex_attrib = glutil::attrib_loc(beam_program,"tex_coords");
            gl::EnableVertexAttribArray(tex_attrib as GLuint);
            gl::VertexAttribPointer(tex_attrib as GLuint, 2, gl::FLOAT, gl::FALSE,
                                    4*4, (2*4) as *const _);
        }

        // Set up VAO/VBO for crt
		let crt_vertices : [GLfloat; 4 * 4] = [
			1.0 , 1.0,   1.0, 1.0,
			-1.0, 1.0,   0.0, 1.0, 
			1.0 ,-1.0,   1.0, 0.0,
			-1.0,-1.0,   0.0, 0.0,
		];
		let mut crt_vbo : GLuint = 0;
		unsafe {
			gl::GenVertexArrays(1, &mut gl.crt_vao);
			gl::BindVertexArray(gl.crt_vao);
			gl::GenBuffers(1, &mut crt_vbo);
			gl::BindBuffer(gl::ARRAY_BUFFER,crt_vbo);
			gl::BufferData(gl::ARRAY_BUFFER, 4*4*4, 
				crt_vertices.as_ptr() as *const _, gl::STATIC_DRAW);
			gl::UseProgram(crt_program);
			let pos_attrib = glutil::attrib_loc(crt_program,"position");
            gl::EnableVertexAttribArray(pos_attrib as GLuint);
            gl::VertexAttribPointer(pos_attrib as GLuint, 2, gl::FLOAT, gl::FALSE,
                                    4*4, std::ptr::null());
            let tex_attrib = glutil::attrib_loc(crt_program,"tex_coords");
            gl::EnableVertexAttribArray(tex_attrib as GLuint);
            gl::VertexAttribPointer(tex_attrib as GLuint, 2, gl::FLOAT, gl::FALSE,
                                    4*4, (2*4) as *const _);
		}

		// Set up data texture
		gl.data_texture = glutil::make_byte_tex(term_dim.0 as i32, 
			term_dim.1 as i32, data.as_slice());

		Terminal {
			term_dim : term_dim,
			render_dim : render_dim,
			data : data,
			font : font,
			cursor : (0,0),
			options : DisplayOptions::new(),
			gl : gl,
		}
	}

	pub fn copy_line(&mut self, from : i8, to : i8) {
		let to_idx = self.term_dim.0 as usize * to as usize;
		let from_idx = self.term_dim.0 as usize * from as usize;
		for n in 0..self.term_dim.0 as usize{
			self.data[to_idx + n] = self.data[from_idx + n];
		}
	}

	pub fn blank_line(&mut self, line_no : i8) {
		let idx = self.term_dim.0 as usize * line_no as usize;
		for n in 0..self.term_dim.0 as usize{
			self.data[idx + n] = 32;
		}
	}

	pub fn scroll(&mut self, lines : i8) {
		match lines {
			0 => {},
			x if x > self.term_dim.1 as i8 => {},
			x if x < -(self.term_dim.1 as i8) => {},
			x if x > 0 => {
				for n in 0..(self.term_dim.1 as i8) {
					if n < self.term_dim.1 as i8-x {
						self.copy_line(n+x,n);
					} else {
						self.blank_line(n);
					}
				}
			},
			x if x < 0 => {
				for n in (0..(self.term_dim.1 as i8)).rev() {
					if n >= -x { 
						self.copy_line(n+x,n);
					} else {
						self.blank_line(n);
					}
				}				
			},
			_ => {},
		}
	}

	pub fn write_str_at(&mut self, x : usize, y : usize, text : &str) {
		let mut idx = x + y*self.term_dim.0 as usize;
		for c in text.bytes() {
			self.data[idx] = c as u8;
			idx = idx + 1;
		}
	}

	pub fn write_char_at(&mut self, x : usize, y : usize, c : char) {
		let mut idx = x + y*self.term_dim.0 as usize;
		self.data[idx] = c as u8;
	}

	pub fn write_char(&mut self, c : char) {
		let x = self.cursor.0 as usize;
		let y = self.cursor.1 as usize;
		let lf : bool = c == '\n';
		if !lf {
			self.write_char_at(x, y, c);
		}
		self.cursor.0 += 1;
		if (self.cursor.0 >= self.term_dim.0) || lf { 
			self.cursor.0 = 0; self.cursor.1 += 1;
			if self.cursor.1 >= self.term_dim.1 {
				self.scroll(1); self.cursor.1 -= 1;
			}
		}
	}

	pub fn flip_phase(&mut self) {
		let new_phase = if self.gl.crt_phase == 0 { 1 } else { 0 };
		self.gl.crt_phase = new_phase;
	}

	pub fn render(&self) {
		// Set up CRT phases (we blend with previous render, which decays phosphor-style)
		let ph1 = self.gl.crt_phase;
		let ph2 = if self.gl.crt_phase == 0 { 1 } else { 0 };

		unsafe {
			// Render initial CRT beam
			self.gl.beam_fb[0].bind();
			gl::UseProgram(beam_program);
			gl::ActiveTexture(gl::TEXTURE0);
			gl::BindTexture(gl::TEXTURE_2D,self.font.gl_texture);
			gl::ActiveTexture(gl::TEXTURE1);
			gl::BindTexture(gl::TEXTURE_2D,self.gl.data_texture);
			glutil::update_byte_tex(self.term_dim.0, self.term_dim.1, self.data.as_slice());
			gl::BindVertexArray(self.gl.beam_vao);
			// Set uniforms
            gl::Uniform1f(glutil::uni_loc(beam_program,"scan_coverage"), self.options.scan_coverage);
            gl::Uniform1f(glutil::uni_loc(beam_program,"scan_height"), 1.0 / self.font.cell_size.1 as f32);
            gl::Uniform1f(glutil::uni_loc(beam_program,"font_char_count"), (self.font.bounds.1 - self.font.bounds.0) as f32);
            gl::Uniform1f(glutil::uni_loc(beam_program,"font_first_char"), self.font.bounds.0 as f32);
            let ref fg = self.options.fg_color;
            let ref bg = self.options.bg_color;
            gl::Uniform4f(glutil::uni_loc(beam_program,"fg_color"), 
            	fg.r, fg.g, fg.b, fg.a);
            gl::Uniform4f(glutil::uni_loc(beam_program,"bg_color"), 
            	0.0,0.0,0.0,0.0);
			gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
			self.gl.beam_fb[0].unbind();
			// Bloom on beam

			self.gl.crt_fb[ph2].bind();
			gl::UseProgram(crt_program);
			gl::ActiveTexture(gl::TEXTURE0);
			gl::BindTexture(gl::TEXTURE_2D,self.gl.crt_fb[ph1].texture_obj());
			gl::ActiveTexture(gl::TEXTURE1);
			gl::BindTexture(gl::TEXTURE_2D,self.gl.beam_fb[0].texture_obj());
			gl::BindVertexArray(self.gl.crt_vao);
			// Set uniforms
            gl::Uniform1f(glutil::uni_loc(crt_program,"decay_factor"), 0.15);
            gl::Uniform4f(glutil::uni_loc(crt_program,"bg_color"), 
            	bg.r, bg.g, bg.b, bg.a);
            // Draw triangles
			gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
			self.gl.crt_fb[ph2].unbind();
			// Blit to window
			gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER,0);
			gl::BindFramebuffer(gl::READ_FRAMEBUFFER,self.gl.crt_fb[ph2].fbo);
			// gl::BindFramebuffer(gl::READ_FRAMEBUFFER,self.gl.beam_fb[0].fbo);
			gl::BlitFramebuffer(0,0,self.render_dim.0, self.render_dim.1,
				0,0,self.render_dim.0, self.render_dim.1,
				gl::COLOR_BUFFER_BIT,gl::LINEAR);
			//gl::BindFramebuffer(gl::READ_FRAMEBUFFER,self.fb_beam.fbo);
		}
	}
}

struct EmbeddedFont {
	cell_size : (u8, u8),
	intercell : (u8, u8),
	bounds : (i16, i16),
	texture : &'static [u8],
}

impl<'a> BitFont<'a> for EmbeddedFont {
	fn cell_size_px(&self) -> (u8, u8) { self.cell_size }
	fn intercell_px(&self) -> (u8, u8) { self.intercell }
	fn bounds(&self) -> (i16, i16) { self.bounds }
	fn texture(&self) -> &'a [u8] { self.texture }
}

fn get_osborne_font() -> EmbeddedFont {
	EmbeddedFont {
		cell_size : (8,10),
		intercell : (0,0),
		bounds : (0,128),
		texture : include_bytes!("fonts/Osborne_I.charrom"),
	}
}

fn get_waters_w600e_font() -> EmbeddedFont {
	EmbeddedFont {
		cell_size : (8,16),
		intercell : (0,0),
		bounds : (32,137),
		texture : include_bytes!("fonts/W600E.charrom"),
	}	
}

fn get_kaypro_2_font() -> EmbeddedFont {
	EmbeddedFont {
		cell_size : (8,16),
		intercell : (0,0),
		bounds : (0,256),
		texture : include_bytes!("fonts/Kaypro2.charrom"),
	}	
}

pub fn osborne_font() -> LoadedFont {
	load_font(get_osborne_font())
}

pub fn kaypro_2_font() -> LoadedFont {
	load_font(get_kaypro_2_font())
}

pub fn waters_w600e_font() -> LoadedFont {
	load_font(get_waters_w600e_font())
}

fn check_programs() {
	unsafe {
		if !programs_loaded {
			beam_program = glutil::build_program(FONT_VS_SRC, FONT_FS_SRC)
				.expect("Failed to create font shader program");
			bloom_program = glutil::build_program(BLOOM_VS_SRC, BLOOM_FS_SRC)
				.expect("Failed to create bloom shader program");
			crt_program = glutil::build_program(CRT_VS_SRC, CRT_FS_SRC)
				.expect("Failed to create crt shader program");
			programs_loaded = true;
		}
	}
}
pub fn load_font<'a, T : BitFont<'a> >(font : T) -> LoadedFont {
	check_programs();
	unsafe {
		gl::UseProgram(beam_program);
        gl::Uniform1i(glutil::uni_loc(beam_program,"font_tex"), 0 as i32);
        gl::Uniform1i(glutil::uni_loc(beam_program,"data_tex"), 1 as i32);
        gl::BindFragDataLocation(beam_program, 0,
        	std::ffi::CString::new("color").unwrap().as_ptr());
		gl::UseProgram(crt_program);
        gl::Uniform1i(glutil::uni_loc(crt_program,"last_frame_tex"), 0 as i32);
        gl::Uniform1i(glutil::uni_loc(crt_program,"new_beam_tex"), 1 as i32);
        gl::BindFragDataLocation(crt_program, 0,
        	std::ffi::CString::new("color").unwrap().as_ptr());
		gl::UseProgram(bloom_program);
        gl::Uniform1i(glutil::uni_loc(bloom_program,"in_tex"), 0 as i32);
        gl::BindFragDataLocation(bloom_program, 0,
        	std::ffi::CString::new("color").unwrap().as_ptr());
	}
	let cell_size = font.cell_size_px();
	let bounds = font.bounds();
	let char_count = bounds.1 - bounds.0;
	let texture_size = (cell_size.0 as i32 * char_count as i32, cell_size.1 as i32);
	let texture = glutil::make_byte_tex(texture_size.0,texture_size.1,font.texture());
	LoadedFont {
		cell_size : cell_size,
		intercell : font.intercell_px(),
		bounds : bounds,
		gl_texture : texture,
	}
}

#[cfg(test)]
mod tests {
    #[test]
    fn has_basic_fonts() {
    }
}
