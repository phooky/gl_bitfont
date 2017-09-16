//! gl_bitfont renders simple, old-school pixel fonts 
extern crate gl;

pub mod glutil;

use glutil::Framebuffer;

use gl::types::*;

// Shader sources
static FONT_VS_SRC: &'static str = include_str!("font_vertex.glsl");
static FONT_FS_SRC: &'static str = include_str!("font_fragment.glsl");
static CRT_VS_SRC: &'static str = include_str!("crt_vertex.glsl");
static CRT_FS_SRC: &'static str = include_str!("crt_fragment.glsl");

static mut font_program : Option<GLuint> = None;
static mut crt_program : Option<GLuint> = None;

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
			scan_coverage : 0.1,
		}
	}
}

/// An area to render text into, along with the current contents
/// of the text
pub struct Terminal<'a> {
	pub dim : (u8,u8),  /// The dimensions, in characters, of this terminal
	pub render_dim : (i32, i32),
	pub data : Vec<u8>, /// The data to be displayed
	font : &'a LoadedFont,
	vao : GLuint, /// The vertex array object
	vao2 : GLuint,
	data_texture : GLuint,
	pub cursor : (u8, u8),
	pub options : DisplayOptions,
	fbs : [Framebuffer;2],
	fb_beam : Framebuffer,
	phase : usize,	
}

impl<'a> Terminal<'a> {
	pub fn new(char_dim : (u8,u8), render_dim : (i32, i32), font : &'a LoadedFont) -> Terminal {
		let mut vao : GLuint = 0;
		let mut vao2 : GLuint = 0;
		let mut vbo : GLuint = 0;
		let mut data : Vec<u8> = Vec::new();
		let d = (char_dim.0 as f32, char_dim.1 as f32);
		data.resize(char_dim.0 as usize * char_dim.1 as usize, 32);
		let vertices : [GLfloat; 4 * 4] = [
			1.0 , 1.0,   d.0, 0.0,
			-1.0, 1.0,   0.0, 0.0, 
			1.0 ,-1.0,   d.0, d.1,
			-1.0,-1.0,   0.0, d.1,
		];
		let vertices2 : [GLfloat; 4 * 4] = [
			1.0 , 1.0,   1.0, 1.0,
			-1.0, 1.0,   0.0, 1.0, 
			1.0 ,-1.0,   1.0, 0.0,
			-1.0,-1.0,   0.0, 0.0,
		];
		let fbs = [Framebuffer::new(render_dim).unwrap(), Framebuffer::new(render_dim).unwrap()];
		let fb_beam = Framebuffer::new(render_dim).unwrap();
		unsafe {
			gl::GenVertexArrays(1, &mut vao);
			gl::BindVertexArray(vao);
			gl::GenBuffers(1, &mut vbo);
			gl::BindBuffer(gl::ARRAY_BUFFER,vbo);
			gl::BufferData(gl::ARRAY_BUFFER, 4*4*4, 
				vertices.as_ptr() as *const _, gl::STATIC_DRAW);

			let program = font_program.unwrap();
			gl::UseProgram(program);
			let pos_attrib = glutil::attrib_loc(program,"position");
            gl::EnableVertexAttribArray(pos_attrib as GLuint);
            gl::VertexAttribPointer(pos_attrib as GLuint, 2, gl::FLOAT, gl::FALSE,
                                    4*4, std::ptr::null());
            let tex_attrib = glutil::attrib_loc(program,"tex_coords");
            gl::EnableVertexAttribArray(tex_attrib as GLuint);
            gl::VertexAttribPointer(tex_attrib as GLuint, 2, gl::FLOAT, gl::FALSE,
                                    4*4, (2*4) as *const _);

			gl::GenVertexArrays(1, &mut vao2);
			gl::BindVertexArray(vao2);
			gl::GenBuffers(1, &mut vbo);
			gl::BindBuffer(gl::ARRAY_BUFFER,vbo);
			gl::BufferData(gl::ARRAY_BUFFER, 4*4*4, 
				vertices2.as_ptr() as *const _, gl::STATIC_DRAW);

			let program2 = crt_program.unwrap();
			gl::UseProgram(program2);
			let pos_attrib2 = glutil::attrib_loc(program,"position");
            gl::EnableVertexAttribArray(pos_attrib2 as GLuint);
            gl::VertexAttribPointer(pos_attrib2 as GLuint, 2, gl::FLOAT, gl::FALSE,
                                    4*4, std::ptr::null());
            let tex_attrib2 = glutil::attrib_loc(program,"tex_coords");
            gl::EnableVertexAttribArray(tex_attrib2 as GLuint);
            gl::VertexAttribPointer(tex_attrib2 as GLuint, 2, gl::FLOAT, gl::FALSE,
                                    4*4, (2*4) as *const _);

		}
		let data_texture = glutil::make_byte_tex(char_dim.0 as i32, 
			char_dim.1 as i32, data.as_slice());
		Terminal {
			dim : char_dim,
			render_dim : render_dim,
			data : data,
			font : font,
			vao : vao,
			vao2 : vao2,
			data_texture : data_texture,
			cursor : (0,0),
			options : DisplayOptions::new(),
			fbs : fbs,
			fb_beam : fb_beam,
			phase : 0,
		}
	}

	pub fn copy_line(&mut self, from : i8, to : i8) {
		let to_idx = self.dim.0 as usize * to as usize;
		let from_idx = self.dim.0 as usize * from as usize;
		for n in 0..self.dim.0 as usize{
			self.data[to_idx + n] = self.data[from_idx + n];
		}
	}

	pub fn blank_line(&mut self, line_no : i8) {
		let idx = self.dim.0 as usize * line_no as usize;
		for n in 0..self.dim.0 as usize{
			self.data[idx + n] = 32;
		}
	}

	pub fn scroll(&mut self, lines : i8) {
		match lines {
			0 => {},
			x if x > self.dim.1 as i8 => {},
			x if x < -(self.dim.1 as i8) => {},
			x if x > 0 => {
				for n in 0..(self.dim.1 as i8) {
					if n < self.dim.1 as i8-x {
						self.copy_line(n+x,n);
					} else {
						self.blank_line(n);
					}
				}
			},
			x if x < 0 => {
				for n in (0..(self.dim.1 as i8)).rev() {
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
		let mut idx = x + y*self.dim.0 as usize;
		for c in text.bytes() {
			self.data[idx] = c as u8;
			idx = idx + 1;
		}
	}

	pub fn write_char_at(&mut self, x : usize, y : usize, c : char) {
		let mut idx = x + y*self.dim.0 as usize;
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
		if (self.cursor.0 >= self.dim.0) || lf { 
			self.cursor.0 = 0; self.cursor.1 += 1;
			if self.cursor.1 >= self.dim.1 {
				self.scroll(1); self.cursor.1 -= 1;
			}
		}
	}

	pub fn flip_phase(&mut self) {
		let new_phase = if self.phase == 0 { 1 } else { 0 };
		self.phase = new_phase;
	}

	pub fn render(&self) {
		let ph1 = self.phase;
		let ph2 = if self.phase == 0 { 1 } else { 0 };

		unsafe {
			self.fb_beam.bind();
			let p = font_program.unwrap();
			gl::UseProgram(p);
			gl::ActiveTexture(gl::TEXTURE0);
			gl::BindTexture(gl::TEXTURE_2D,self.font.gl_texture);
			gl::ActiveTexture(gl::TEXTURE1);
			gl::BindTexture(gl::TEXTURE_2D,self.data_texture);
			glutil::update_byte_tex(self.dim.0 as i32, 
			self.dim.1 as i32, self.data.as_slice());
			gl::BindVertexArray(self.vao);
			// Set uniforms
            gl::Uniform1f(glutil::uni_loc(p,"scan_coverage"), self.options.scan_coverage);
            gl::Uniform1f(glutil::uni_loc(p,"scan_height"), 1.0 / self.font.cell_size.1 as f32);
            gl::Uniform1f(glutil::uni_loc(p,"font_char_count"), (self.font.bounds.1 - self.font.bounds.0) as f32);
            gl::Uniform1f(glutil::uni_loc(p,"font_first_char"), self.font.bounds.0 as f32);
            let ref fg = self.options.fg_color;
            let ref bg = self.options.bg_color;
            gl::Uniform4f(glutil::uni_loc(p,"fg_color"), 
            	fg.r, fg.g, fg.b, fg.a);
            gl::Uniform4f(glutil::uni_loc(p,"bg_color"), 
            	0.0,0.0,0.0,0.0);
            // Draw triangles
			gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
			self.fb_beam.unbind();
			self.fbs[ph2].bind();
			let p2 = crt_program.unwrap();
			gl::UseProgram(p2);
			gl::ActiveTexture(gl::TEXTURE0);
			gl::BindTexture(gl::TEXTURE_2D,self.fbs[ph1].texture_obj());
			gl::ActiveTexture(gl::TEXTURE1);
			gl::BindTexture(gl::TEXTURE_2D,self.fb_beam.texture_obj());
			gl::BindVertexArray(self.vao2);
			// Set uniforms
            gl::Uniform1f(glutil::uni_loc(p2,"decay_factor"), 0.15);
            gl::Uniform4f(glutil::uni_loc(p2,"bg_color"), 
            	bg.r, bg.g, bg.b, bg.a);
            // Draw triangles
			gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
			self.fbs[ph2].unbind();
			// Blit to window
			gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER,0);
			gl::BindFramebuffer(gl::READ_FRAMEBUFFER,self.fbs[ph2].fbo);
			gl::BlitFramebuffer(0,0,self.render_dim.0, self.render_dim.1,
				0,0,self.render_dim.0, self.render_dim.1,
				gl::COLOR_BUFFER_BIT,gl::LINEAR);
			gl::BindFramebuffer(gl::READ_FRAMEBUFFER,self.fb_beam.fbo);
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

pub fn load_font<'a, T : BitFont<'a> >(font : T) -> LoadedFont {
	unsafe {
		if font_program == None {
			font_program = Some(glutil::build_program(FONT_VS_SRC, FONT_FS_SRC)
				.expect("Failed to create font shader program"));
			let program = font_program.unwrap();
			gl::UseProgram(program);
            gl::Uniform1i(glutil::uni_loc(program,"font_tex"), 0 as i32);
            gl::Uniform1i(glutil::uni_loc(program,"data_tex"), 1 as i32);
            gl::BindFragDataLocation(program, 0,
                                     std::ffi::CString::new("color").unwrap().as_ptr());
		}
		if crt_program == None {
			crt_program = Some(glutil::build_program(CRT_VS_SRC, CRT_FS_SRC)
				.expect("Failed to create crt shader program"));
			let program = crt_program.unwrap();
			gl::UseProgram(program);
            gl::Uniform1i(glutil::uni_loc(program,"last_frame_tex"), 0 as i32);
            gl::Uniform1i(glutil::uni_loc(program,"new_beam_tex"), 1 as i32);
            gl::BindFragDataLocation(program, 0,
                                     std::ffi::CString::new("color").unwrap().as_ptr());
		}
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
