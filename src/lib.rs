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
	dim : (u8,u8),  /// The dimensions, in characters, of this terminal
	data : Vec<u8>, /// The data to be displayed
	font : &'a LoadedFont,
	vao : GLuint,
}

impl<'a> Terminal<'a> {
	pub fn new(dimensions : (u8,u8), font : &'a LoadedFont) -> Terminal {
		let mut vao : GLuint = 0;
		let mut vbo : GLuint = 0;
		let mut data : Vec<u8> = Vec::new();
		let d = (dimensions.0 as f32, dimensions.1 as f32);
		data.resize(dimensions.0 as usize * dimensions.1 as usize, 0);
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

		}
		Terminal {
			dim : dimensions,
			data : data,
			font : font,
			vao : vao,
		}
	}
	pub fn write(&mut self, x : usize, y : usize, text : &str) {
		let mut idx = x + y*self.dim.0 as usize;
		for c in text.bytes() {
			self.data[idx] = c as u8;
			idx = idx + 1;
		}
	}

	pub fn render(&self) {
		unsafe {
			let p = font_program.unwrap();
			gl::UseProgram(p);
			gl::ActiveTexture(gl::TEXTURE0);
			gl::BindTexture(gl::TEXTURE_2D,self.font.gl_texture);
			gl::BindVertexArray(self.vao);
			// Set uniform for text?
			let c_str = std::ffi::CString::new("data".as_bytes()).unwrap();
			let loc = gl::GetUniformLocation(p,c_str.as_ptr());
			gl::Uniform1uiv(loc,80*24,self.data.as_ptr() as *const _);
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
	let mut texture : GLuint = 0;
	unsafe {
		// Create and bind texture object
		gl::GenTextures(1, &mut texture);
		gl::ActiveTexture(gl::TEXTURE0);
		gl::BindTexture(gl::TEXTURE_2D,texture);
		// Set texture data and eschew mipmaps
		gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
		gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
		gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
		gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
		gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8UI as GLint,
			texture_size.0, texture_size.1, 0,
			gl::RED_INTEGER, gl::UNSIGNED_BYTE,
			font.texture().as_ptr() as *const _);
		// Unbind texture object
		gl::BindTexture(gl::TEXTURE_2D,0);
	}
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
