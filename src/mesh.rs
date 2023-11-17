use std::convert::TryInto;
use std::{ffi::c_void, path::Path};

use crate::gl;
use crate::utils;

#[derive(Debug)]
pub struct Mesh {
    pub mesh: tobj::Mesh,
    pub vao: gl::types::GLuint,
}

impl Mesh {
    pub fn build(mesh: &tobj::Mesh) -> gl::types::GLuint {
        unsafe {
            let mut vao: gl::types::GLuint = 0;
            gl::GenVertexArrays(1, &mut vao as *mut gl::types::GLuint);
            gl::BindVertexArray(vao);

            let mut vertices: gl::types::GLuint = 0;
            gl::GenBuffers(1, &mut vertices as *mut gl::types::GLuint);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertices);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (4 * mesh.positions.len()) as _,
                mesh.positions.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, vertices);
            gl::VertexAttribPointer(utils::ATTRIB_VERTEX, 3, gl::FLOAT, false as u8, 0, 0 as *const c_void);
            gl::EnableVertexAttribArray(utils::ATTRIB_VERTEX);

            let mut normals: gl::types::GLuint = 0;
            gl::GenBuffers(1, &mut normals as *mut gl::types::GLuint);
            gl::BindBuffer(gl::ARRAY_BUFFER, normals);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (4 * mesh.normals.len()) as _,
                mesh.normals.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, normals);
            gl::VertexAttribPointer(utils::ATTRIB_NORMAL, 3, gl::FLOAT, false as u8, 0, 0 as *const c_void);
            gl::EnableVertexAttribArray(utils::ATTRIB_NORMAL);

            let mut texcoords: gl::types::GLuint = 0;
            gl::GenBuffers(1, &mut texcoords as *mut gl::types::GLuint);
            gl::BindBuffer(gl::ARRAY_BUFFER, texcoords);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (4 * mesh.texcoords.len()) as _,
                mesh.texcoords.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, texcoords);
            gl::VertexAttribPointer(utils::ATTRIB_TEXCOORD, 2, gl::FLOAT, false as u8, 0, 0 as *const c_void);
            gl::EnableVertexAttribArray(utils::ATTRIB_TEXCOORD);

            let mut indices: gl::types::GLuint = 0;
            gl::GenBuffers(1, &mut indices as *mut gl::types::GLuint);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indices);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (4 * mesh.indices.len()) as _,
                mesh.indices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            vao
        }
    }

    pub fn new(p: &Path) -> Option<Self> {
        let lopts = tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        };
        let (meshes, _materials) = tobj::load_obj(p, &lopts).ok()?;
        let mesh = meshes.into_iter().next()?.mesh;
        let vao = Self::build(&mesh);
        Some(Self { mesh, vao })
    }

    pub fn render(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, self.mesh.indices.len() as i32, gl::UNSIGNED_INT, 0 as *const c_void);
        }
    }
}
