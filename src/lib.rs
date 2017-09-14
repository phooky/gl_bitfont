//! gl_bitfont renders simple, old-school pixel fonts 
extern crate gl;

mod glutil;

use gl::types::*;

// Shader sources
static VS_SRC: &'static str = include_str!("font_vertex.glsl");
static FS_SRC: &'static str = include_str!("font_fragment.glsl");

static mut font_program : Option<GLuint> = None;

/// The BitFont trait includes all the information necessary to
/// load a bit font into the GL engine. All fonts are presumed
/// to be ASCII and within a contiguous range in [0,255].
pub trait BitFont<'a> {
	/// Character cell size, in pixels
	fn cell_size_px(&self) -> (u8,u8);
	/// Spacing between cells, vertically and horizontall, in pixels
	fn intercell_px(&self) -> (u8,u8);
	/// Index of lowest and highest ASCII values present in font.
	fn bounds(&self) -> (u8, u8);
	/// Raw bit texture data. This represented as a byte map, with a 
	/// width of (max-min)*cell width and a height equal to the cell 
	/// height.
	fn texture(&self) -> &'a [u8];
}

// All the information necessary to render this font.
pub struct LoadedFont {
	cell_size : (u8,u8),
	intercell : (u8,u8),
	bounds : (u8,u8),
	gl_texture : GLuint,
}

/// An area to render text into, along with the current contents
/// of the text
pub struct Terminal<'a> {
	pub dim : (u8,u8),  /// The dimensions, in characters, of this terminal
	pub data : Vec<u8>, /// The data to be displayed
	font : &'a LoadedFont,
	vao : GLuint,
	data_texture : GLuint,
	pub cursor : (u8, u8),
}

impl<'a> Terminal<'a> {
	pub fn new(dimensions : (u8,u8), font : &'a LoadedFont) -> Terminal {
		let mut vao : GLuint = 0;
		let mut vbo : GLuint = 0;
		let mut data : Vec<u8> = Vec::new();
		let d = (dimensions.0 as f32, dimensions.1 as f32);
		data.resize(dimensions.0 as usize * dimensions.1 as usize, 32);
		let vertices : [GLfloat; 4 * 4] = [
			1.0 , 1.0,   d.0, 0.0,
			-1.0, 1.0,   0.0, 0.0, 
			1.0 ,-1.0,   d.0, d.1,
			-1.0,-1.0,   0.0, d.1,
		];

		unsafe {
			gl::GenVertexArrays(1, &mut vao);
			gl::BindVertexArray(vao);
			gl::GenBuffers(1, &mut vbo);
			gl::BindBuffer(gl::ARRAY_BUFFER,vbo);
			gl::BufferData(gl::ARRAY_BUFFER, 4*4*4, 
				vertices.as_ptr() as *const _, gl::STATIC_DRAW);


			let program = font_program.unwrap();
			gl::UseProgram(program);
            gl::BindFragDataLocation(program, 0,
                                     std::ffi::CString::new("color").unwrap().as_ptr());
			let pos_attrib = glutil::attrib_loc(program,"position");
            gl::EnableVertexAttribArray(pos_attrib as GLuint);
            gl::VertexAttribPointer(pos_attrib as GLuint, 2, gl::FLOAT, gl::FALSE,
                                    4*4, std::ptr::null());
            let tex_attrib = glutil::attrib_loc(program,"tex_coords");
            gl::EnableVertexAttribArray(tex_attrib as GLuint);
            gl::VertexAttribPointer(tex_attrib as GLuint, 2, gl::FLOAT, gl::FALSE,
                                    4*4, (2*4) as *const _);
            gl::Uniform1i(glutil::uni_loc(program,"font_tex"), 0 as i32);
            gl::Uniform1i(glutil::uni_loc(program,"data_tex"), 1 as i32);
		}
		let data_texture = glutil::make_byte_tex(dimensions.0 as i32, 
			dimensions.1 as i32, data.as_slice());
		Terminal {
			dim : dimensions,
			data : data,
			font : font,
			vao : vao,
			data_texture : data_texture,
			cursor : (0,0),
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
		self.write_char_at(x, y, c);
		self.cursor.0 += 1;
		if self.cursor.0 >= self.dim.0 { 
			self.cursor.0 = 0; self.cursor.1 += 1;
			if self.cursor.1 >= self.dim.1 {
				self.scroll(1); self.cursor.1 -= 1;
			}
		}
	}

	pub fn render(&self) {
		unsafe {
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
			gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
		}
	}
}

struct EmbeddedFont {
	cell_size : (u8, u8),
	intercell : (u8, u8),
	bounds : (u8, u8),
	texture : &'static [u8],
}

impl<'a> BitFont<'a> for EmbeddedFont {
	fn cell_size_px(&self) -> (u8, u8) { self.cell_size }
	fn intercell_px(&self) -> (u8, u8) { self.intercell }
	fn bounds(&self) -> (u8,u8) { self.bounds }
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

pub fn osborne_font() -> LoadedFont {
	load_font(get_osborne_font())
}

pub fn load_font<'a, T : BitFont<'a> >(font : T) -> LoadedFont {
	unsafe {
		if font_program == None {
			font_program = Some(glutil::build_program(VS_SRC, FS_SRC)
				.expect("Failed to create font shader program"));
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
