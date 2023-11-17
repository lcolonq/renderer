use crate::gl;

pub const ATTRIB_VERTEX: gl::types::GLuint = 0;
pub const ATTRIB_NORMAL: gl::types::GLuint = 1;
pub const ATTRIB_TEXCOORD: gl::types::GLuint = 2;

pub const ATTRIB_JOINT: gl::types::GLuint = 3;
pub const ATTRIB_WEIGHT: gl::types::GLuint = 4;

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub struct Dimensions {
    pub w: f32,
    pub h: f32,
}
