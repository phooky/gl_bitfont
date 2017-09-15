use gl;
use gl::types::*;

use std;
use std::str;

#[derive(Debug)]


/// A Framebuffer object and attached texture
pub struct Framebuffer {
    pub fbo : GLuint,
    pub txo : GLuint,
}

impl Framebuffer {
    pub fn new(size : (i32, i32)) -> Result<Framebuffer,String> {
        let mut fbo : GLuint = 0;
        let mut txo : GLuint = 0;
        unsafe {
            // Create and bind texture object
            gl::GenTextures(1, &mut txo);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, txo);
            // Create texture in memory
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA8 as i32, size.0, size.1, 0, 
                gl::RGBA, gl::UNSIGNED_BYTE, 0 as *const _);
            // Create fbo and bind texture
            gl::GenFramebuffers(1,&mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER,gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D, txo, 0);
            let fbstat = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if fbstat != gl::FRAMEBUFFER_COMPLETE {
                return Err(String::from("Incomplete framebuffer!"));
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        Ok(Framebuffer {
            fbo : fbo,
            txo : txo,
        })
    }

    pub fn bind(&self) { unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo); } }
    pub fn unbind(&self) { unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); } }
    pub fn texture_obj(&self) -> GLuint { self.txo }
}

pub fn attrib_loc(program : GLuint , name : &str) -> GLint {
    let c_str = std::ffi::CString::new(name.as_bytes()).unwrap();
    let loc = unsafe { gl::GetAttribLocation(program, c_str.as_ptr()) };
    loc
}

pub fn uni_loc(program : GLuint, name : &str) -> GLint {
    let c_str = std::ffi::CString::new(name.as_bytes()).unwrap();
    let loc = unsafe { gl::GetUniformLocation(program, c_str.as_ptr()) };
    loc
}

pub fn build_shader(src : &str, shader_type : GLenum) -> Option<GLuint> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        let src_cstr = std::ffi::CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &src_cstr.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
        let mut compiled : GLint = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compiled);
        if compiled == gl::TRUE as GLint {
            Some(shader)
        } else {
            gl::DeleteShader(shader);
            None
        }
    }
}

pub fn build_program(vertex_shader_src : &str, fragment_shader_src : &str) -> Option<GLuint> {
    unsafe {
        let program = gl::CreateProgram();
        match (build_shader(vertex_shader_src, gl::VERTEX_SHADER),
               build_shader(fragment_shader_src, gl::FRAGMENT_SHADER)) {
            (Some(vs), Some(fs)) => {
                gl::AttachShader(program, vs);
                gl::AttachShader(program, fs);
                gl::LinkProgram(program);
                gl::DeleteShader(vs);
                gl::DeleteShader(fs);
                let mut linked : GLint = 0;
                gl::GetProgramiv(program, gl::LINK_STATUS, &mut linked);
                if linked == gl::TRUE as GLint {
                    Some(program)
                } else {
                    gl::DeleteProgram(program);
                    None
                }
            },
            _ => {
                gl::DeleteProgram(program);
                None
            }
        }
    }
}

/// Assumes texture is already bound
pub fn update_byte_tex<'a>(w : i32, h : i32, data : &'a [u8] ) {
    unsafe {
        // Set texture data and eschew mipmaps
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8UI as GLint,
            w, h, 0,
            gl::RED_INTEGER, gl::UNSIGNED_BYTE,
            data.as_ptr() as *const _);
    }
}

pub fn make_byte_tex<'a>(w : i32, h : i32, data : &'a [u8] ) -> GLuint {
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
            w, h, 0,
            gl::RED_INTEGER, gl::UNSIGNED_BYTE,
            data.as_ptr() as *const _);
        // Unbind texture object
        gl::BindTexture(gl::TEXTURE_2D,0);
    }
    texture
}