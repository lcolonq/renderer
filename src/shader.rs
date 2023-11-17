use crate::{gl, utils, context};

use std::collections::HashMap;

#[allow(dead_code)]
pub struct Shader {
    pub prog: gl::types::GLuint,
    pub uniform_view: gl::types::GLint,
    pub uniform_position: gl::types::GLint,
    pub uniform_projection: gl::types::GLint,
    pub uniform_normal: gl::types::GLint,
    pub uniform_camera_pos: gl::types::GLint,

    pub uniform_joint_matrices: gl::types::GLint,

    pub uniform_expressions: HashMap<String, gl::types::GLint>,
}

impl Shader {
    fn check_compile_error(path: &str, shader: gl::types::GLuint) {
        unsafe {
            let mut success: gl::types::GLint = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success as *mut _);
            if success == 0 {
                let mut buf: [i8; 512] = [0; 512];
                gl::GetShaderInfoLog(shader, 512, 0 as _, &mut buf as *mut i8);
                let msg: &std::ffi::CStr = std::ffi::CStr::from_ptr(&buf as *const i8);
                let strmsg = String::from_utf8_lossy(msg.to_bytes()).to_string();
                panic!("shader compile error for {}: {}", path, strmsg);
            }
        }
    }

    fn check_link_error(prog: gl::types::GLuint) {
        unsafe {
            let mut success: gl::types::GLint = 0;
            gl::GetProgramiv(prog, gl::LINK_STATUS, &mut success as *mut _);
            if success == 0 {
                let mut buf: [i8; 512] = [0; 512];
                gl::GetProgramInfoLog(prog, 512, 0 as _, &mut buf as *mut i8);
                let msg: &std::ffi::CStr = std::ffi::CStr::from_ptr(&buf as *const i8);
                let strmsg = String::from_utf8_lossy(msg.to_bytes()).to_string();
                panic!("shader link error: {}", strmsg);
            }
        }
    }

    pub fn new(ctx: &context::Context, vtxpath: &str, fragpath: &str) -> Self {
        unsafe {
            let prog = gl::CreateProgram();

            let vtxsrc = std::fs::read_to_string(vtxpath).unwrap();
            let vtx = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(
                vtx,
                1,
                [vtxsrc.as_bytes().as_ptr() as *const gl::types::GLchar].as_ptr(),
                [vtxsrc.len() as gl::types::GLint - 1].as_ptr(),
            );
            gl::CompileShader(vtx);
            Self::check_compile_error(vtxpath, vtx);
            gl::AttachShader(prog, vtx);
            gl::DeleteShader(vtx);

            let fragsrc = std::fs::read_to_string(fragpath).unwrap();
            let frag = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(
                frag,
                1,
                [fragsrc.as_bytes().as_ptr() as *const gl::types::GLchar].as_ptr(),
                [fragsrc.len() as gl::types::GLint - 1].as_ptr(),
            );
            gl::CompileShader(frag);
            Self::check_compile_error(fragpath, frag);
            gl::AttachShader(prog, frag);
            gl::DeleteShader(frag);

            gl::BindAttribLocation(prog, utils::ATTRIB_VERTEX, b"vertex\0".as_ptr() as *const i8);
            gl::BindAttribLocation(prog, utils::ATTRIB_NORMAL, b"normal\0".as_ptr() as *const i8);
            gl::BindAttribLocation(prog, utils::ATTRIB_TEXCOORD, b"texcoord\0".as_ptr() as *const i8);
            gl::BindAttribLocation(prog, utils::ATTRIB_JOINT, b"joint\0".as_ptr() as *const i8);
            gl::BindAttribLocation(prog, utils::ATTRIB_WEIGHT, b"weight\0".as_ptr() as *const i8);

            gl::LinkProgram(prog);
            Self::check_link_error(prog);

            for (enm, attrib) in &ctx.attrib_expressions {
                gl::BindAttribLocation(prog, *attrib, format!("expression_{}_vertex\0", enm).as_ptr() as *const i8);
            }

            let uniform_expressions = ctx.attrib_expressions.iter().map(|(enm, _)| {
                (enm.clone(), gl::GetUniformLocation(prog, format!("expression_{}_weight\0", enm).as_ptr() as *const i8))
            }).collect();

            Self {
                prog,
                uniform_view: gl::GetUniformLocation(prog, b"view\0".as_ptr() as *const i8),
                uniform_position: gl::GetUniformLocation(prog, b"position\0".as_ptr() as *const i8),
                uniform_projection: gl::GetUniformLocation(prog, b"projection\0".as_ptr() as *const i8),
                uniform_normal: gl::GetUniformLocation(prog, b"normal_matrix\0".as_ptr() as *const i8),
                uniform_camera_pos: gl::GetUniformLocation(prog, b"camera_pos\0".as_ptr() as *const i8),
                uniform_joint_matrices: gl::GetUniformLocation(prog, b"joint_matrices\0".as_ptr() as *const i8),
                uniform_expressions,
            }
        }
    }

    pub fn bind(&self, _ctx: &context::Context) {
        unsafe {
            gl::UseProgram(self.prog);
        }
    }
}
