//! gl_bitfont renders simple, old-school pixel fonts 
extern crate gl;

use gl::types::*;

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
struct Terminal<'a> {
	dim : (u8,u8),  /// The dimensions, in characters, of this terminal
	data : Vec<u8>, /// The data to be displayed
	font : &'a LoadedFont,
}

pub fn load_font<'a, T : BitFont<'a> >(font : T) -> LoadedFont {
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
