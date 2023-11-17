use crate::{gl, utils, context, shader};

use std::{io::Read, collections::HashMap};

#[allow(dead_code)]
pub struct Primitive {
    pub vao: gl::types::GLuint,
    pub mode: gl::types::GLenum,
    pub count: i32,
    pub index_type: gl::types::GLenum,
    pub index_offset: i32,
    pub material_index: usize,
}

#[allow(dead_code)]
pub struct Mesh {
    pub primitives: Vec<Primitive>,
}

#[allow(dead_code)]
pub struct Skin {
    pub inverse_bind_matrices: Vec<glam::Mat4>,
    pub joints: Vec<usize>,
}

#[allow(dead_code)]
pub struct Texture {
    pub tid: gl::types::GLuint,
}

impl Texture {
    pub fn bind(&self, _ctx: &context::Context) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.tid);
        }
    }
}

#[allow(dead_code)]
pub struct Material {
    pub base_color_factor: glam::Vec4,
    pub base_color_texture: Option<Texture>,

    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture: Option<Texture>,

    pub normal_texture: Option<Texture>,

    pub occlusion_texture: Option<Texture>,

    pub emissive_factor: glam::Vec3,
    pub emissive_texture: Option<Texture>,
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub struct Node {
    pub child_indices: Vec<usize>,
    pub mesh_index: Option<usize>,
    pub skin_index: Option<usize>,
    pub transform: glam::Mat4,
    
}

#[allow(dead_code)]
pub struct Scene {
    pub meshes: Vec<Mesh>,
    pub skins: Vec<Skin>,
    pub materials: Vec<Material>,
    pub nodes: Vec<Node>,
    pub bone_node_indices: HashMap<String, usize>,
    pub scene_node_indices: Vec<usize>,
}

impl Scene {
    fn initialize_attrib(attrib: gl::types::GLuint, accessor: &gltf::Accessor, bufs: &Vec<(gl::types::GLuint, &gltf::buffer::Data)> ) {
        let data_type = match accessor.data_type() {
            gltf::accessor::DataType::I8 => gl::BYTE,
            gltf::accessor::DataType::U8 => gl::UNSIGNED_BYTE,
            gltf::accessor::DataType::I16 => gl::SHORT,
            gltf::accessor::DataType::U16 => gl::UNSIGNED_SHORT,
            gltf::accessor::DataType::U32 => gl::UNSIGNED_INT,
            gltf::accessor::DataType::F32 => gl::FLOAT,
        };
        if let Some(_sparse) = accessor.sparse() {
            let get_buffer_data = |buffer: gltf::Buffer| bufs.get(buffer.index()).map(|x| &*x.1.0);
            let iter = gltf::accessor::Iter::<[f32; 3]>::new(accessor.clone(), get_buffer_data).unwrap();
            let mut b: Vec<[f32; 3]> = Vec::new();
            for item in iter {
                b.push(item);
            }
            
            log::info!("attrib {}: length {}", attrib, (b.len() * accessor.dimensions().multiplicity() * accessor.data_type().size()));
            unsafe {
                let mut buf: gl::types::GLuint = 0;
                gl::GenBuffers(1, &mut buf as *mut gl::types::GLuint);
                gl::BindBuffer(gl::ARRAY_BUFFER, buf);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (b.len() * accessor.dimensions().multiplicity() * accessor.data_type().size()) as _,
                    b.as_ptr() as *const std::ffi::c_void,
                    gl::STATIC_DRAW,
                );
                gl::VertexAttribPointer(
                    attrib,
                    accessor.dimensions().multiplicity() as _,
                    data_type,
                    accessor.normalized() as _,
                    0,
                    0 as _,
                );
                gl::EnableVertexAttribArray(attrib);
            }
        } else {
            let view = accessor.view().unwrap();
            let buf = bufs.get(view.buffer().index()).unwrap().0;
            log::info!("attrib {}: length {}", attrib, view.length());
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, buf);
                gl::VertexAttribPointer(
                    attrib,
                    accessor.dimensions().multiplicity() as _,
                    data_type,
                    accessor.normalized() as _,
                    match view.stride() { Some(s) => s, _ => 0 } as _,
                    view.offset() as _
                );
                gl::EnableVertexAttribArray(attrib);
            }
        }
    }

    pub fn new(ctx: &context::Context, path: &str) -> Self {
        let file = std::fs::File::open(path).unwrap();
        let mut reader = std::io::BufReader::new(file);
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).unwrap();
        let (gltf, buffers, images) = gltf::import_slice(bytes.as_slice()).unwrap();
        let json = gltf.clone().into_json();
        let vrm = json.extensions.unwrap().vrmc_vrm.unwrap();

        log::info!("specVersion: {}", vrm.spec_version);
        let mut max: i32 = 0;
        unsafe {
            gl::GetIntegerv(gl::MAX_VERTEX_ATTRIBS, &mut max as _);
        }
        log::info!("max: {}", max);

        let expressions: HashMap<String, (gltf::json::extensions::root::MorphTargetBind, gltf::Mesh, gl::types::GLuint)> =
            ctx.attrib_expressions.iter().map(|(enm, attrib)| {
                let expr = &vrm.expressions.preset.get(enm).unwrap().morph_target_binds[0];
                (enm.clone(), (expr.clone(), gltf.nodes().nth(expr.node as _).unwrap().mesh().unwrap(), *attrib))
            }).collect();

        let bufs: Vec<(gl::types::GLuint, &gltf::buffer::Data)> = buffers.iter().map(|b| {
            unsafe {
                let mut buf: gl::types::GLuint = 0;
                gl::GenBuffers(1, &mut buf as *mut gl::types::GLuint);
                gl::BindBuffer(gl::ARRAY_BUFFER, buf);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    b.len() as _,
                    b.as_ptr() as *const std::ffi::c_void,
                    gl::STATIC_DRAW,
                );
                (buf, b)
            }
        }).collect();

        let tids: Vec<gl::types::GLuint> = images.iter().map(|i| {
            unsafe {
                let mut texture: gl::types::GLuint = 0;
                gl::GenTextures(1, &mut texture as *mut gl::types::GLuint);
                gl::BindTexture(gl::TEXTURE_2D, texture);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as i32,
                    i.width as i32,
                    i.height as i32,
                    0,
                    match i.format {
                        gltf::image::Format::R8 => gl::RED,
                        gltf::image::Format::R8G8 => gl::RG,
                        gltf::image::Format::R8G8B8 => gl::RGB,
                        gltf::image::Format::R8G8B8A8 => gl::RGBA,
                        gltf::image::Format::R16 => gl::RED,
                        gltf::image::Format::R16G16 => gl::RG,
                        gltf::image::Format::R16G16B16 => gl::RGB,
                        gltf::image::Format::R16G16B16A16 => gl::RGBA,
                        gltf::image::Format::R32G32B32FLOAT => gl::RGB,
                        gltf::image::Format::R32G32B32A32FLOAT => gl::RGBA,
                    },
                    match i.format {
                        gltf::image::Format::R8 => gl::UNSIGNED_BYTE,
                        gltf::image::Format::R8G8 => gl::UNSIGNED_BYTE,
                        gltf::image::Format::R8G8B8 => gl::UNSIGNED_BYTE,
                        gltf::image::Format::R8G8B8A8 => gl::UNSIGNED_BYTE,
                        gltf::image::Format::R16 => gl::UNSIGNED_SHORT,
                        gltf::image::Format::R16G16 => gl::UNSIGNED_SHORT,
                        gltf::image::Format::R16G16B16 => gl::UNSIGNED_SHORT,
                        gltf::image::Format::R16G16B16A16 => gl::UNSIGNED_SHORT,
                        gltf::image::Format::R32G32B32FLOAT => gl::FLOAT,
                        gltf::image::Format::R32G32B32A32FLOAT => gl::FLOAT,
                    },
                    i.pixels.as_ptr() as *const std::ffi::c_void
                );
                gl::GenerateMipmap(gl::TEXTURE_2D);
                texture
            }
        }).collect();

        let meshes = gltf.meshes().map(|m| {
            let primitives = m.primitives().filter_map(|p| {
                log::info!("begin primitive");
                let mode = match p.mode() {
                    gltf::mesh::Mode::Points => gl::POINTS,
                    gltf::mesh::Mode::Lines => gl::LINES,
                    gltf::mesh::Mode::LineLoop => gl::LINE_LOOP,
                    gltf::mesh::Mode::LineStrip => gl::LINE_STRIP,
                    gltf::mesh::Mode::Triangles => gl::TRIANGLES,
                    gltf::mesh::Mode::TriangleStrip => gl::TRIANGLE_STRIP,
                    gltf::mesh::Mode::TriangleFan => gl::TRIANGLE_FAN,
                };
                unsafe {
                    let mut vao: gl::types::GLuint = 0;
                    gl::GenVertexArrays(1, &mut vao as *mut gl::types::GLuint);
                    gl::BindVertexArray(vao);

                    let indices_accessor = p.indices()?;
                    let indices_view = indices_accessor.view()?;
                    let indices_buf = bufs.get(indices_view.buffer().index())?.0;
                    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indices_buf);

                    for (semantic, accessor) in p.attributes() {
                        let mattrib = match semantic {
                            gltf::Semantic::Positions => Some(utils::ATTRIB_VERTEX),
                            gltf::Semantic::Normals => Some(utils::ATTRIB_NORMAL),
                            gltf::Semantic::TexCoords(0) => Some(utils::ATTRIB_TEXCOORD),
                            gltf::Semantic::Joints(0) => Some(utils::ATTRIB_JOINT),
                            gltf::Semantic::Weights(0) => Some(utils::ATTRIB_WEIGHT),
                            _ => None,
                        };
                        if let Some(attrib) = mattrib {
                            Self::initialize_attrib(attrib, &accessor, &bufs);
                        }
                    }

                    for (_enm, (expr, mesh, attrib)) in &expressions {
                        if m.index() == mesh.index() {
                            let target = p.morph_targets().nth(expr.index as _).unwrap();
                            Self::initialize_attrib(*attrib, &target.positions().unwrap(), &bufs);
                        }
                    }

                    Some(Primitive {
                        vao,
                        mode,
                        count: indices_accessor.count() as _,
                        index_type: match indices_accessor.data_type() {
                            gltf::accessor::DataType::I8 => gl::BYTE,
                            gltf::accessor::DataType::U8 => gl::UNSIGNED_BYTE,
                            gltf::accessor::DataType::I16 => gl::SHORT,
                            gltf::accessor::DataType::U16 => gl::UNSIGNED_SHORT,
                            gltf::accessor::DataType::U32 => gl::UNSIGNED_INT,
                            gltf::accessor::DataType::F32 => gl::FLOAT,
                        },
                        index_offset: indices_view.offset() as _,
                        material_index: p.material().index().unwrap(),
                    })
                }
            }).collect();
            Mesh {
                primitives,
            }
        }).collect();

        let skins = gltf.skins().map(|s| {
            let get_buffer_data = |buffer: gltf::Buffer| bufs.get(buffer.index()).map(|x| &*x.1.0);
            Skin {
                inverse_bind_matrices: s.reader(get_buffer_data).read_inverse_bind_matrices().unwrap().map(|m| glam::Mat4::from_cols_array_2d(&m)).collect(),
                joints: s.joints().map(|j| j.index()).collect(),
            }
        }).collect();

        let materials = gltf.materials().map(|m| {
            let pbr = m.pbr_metallic_roughness();
            let [bcr, bcg, bcb, bca] = pbr.base_color_factor();
            let [emx, emy, emz] = m.emissive_factor();
            Material {
                base_color_factor: glam::Vec4::new(bcr, bcg, bcb, bca),
                base_color_texture: pbr.base_color_texture().and_then(|t| {
                    let sampler = t.texture().sampler();
                    let tid = *tids.get(t.texture().source().index())?;
                    unsafe {
                        gl::BindTexture(gl::TEXTURE_2D, tid);
                        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, sampler.wrap_s().as_gl_enum() as i32);
                        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, sampler.wrap_t().as_gl_enum() as i32);
                        if let Some(min_filter) = sampler.min_filter() {
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter.as_gl_enum() as i32);
                        }
                        if let Some(mag_filter) = sampler.mag_filter() {
                            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter.as_gl_enum() as i32);
                        }
                    }
                    Some(Texture {
                        tid,
                    })
                }),
                metallic_factor: pbr.metallic_factor(),
                roughness_factor: pbr.roughness_factor(),
                metallic_roughness_texture: None,
                normal_texture: None,
                occlusion_texture: None,
                emissive_factor: glam::Vec3::new(emx, emy, emz),
                emissive_texture: None,
            }
        }).collect();

        let nodes = gltf.nodes().map(|n| {
            log::info!("node {} {:?}: {:?}", n.index(), n.name(), n.skin());
            Node {
                child_indices: n.children().map(|c| c.index()).collect(),
                mesh_index: n.mesh().map(|m| m.index()),
                skin_index: n.skin().map(|s| s.index()),
                transform: glam::Mat4::from_cols_array_2d(&n.transform().matrix()),
            }
        }).collect();

        let bone_node_indices = vrm.humanoid.human_bones.iter().map(|(nm, b)| (nm.clone(), b.node as _)).collect();

        let scene_node_indices = gltf.default_scene().unwrap().nodes().map(|n| n.index()).collect();

        Self {
            meshes,
            skins,
            materials,
            nodes,
            bone_node_indices,
            scene_node_indices,
        }
    }

    fn compute_global_transforms_from(&self, transforms: &mut Vec<glam::Mat4>, nodes: &Vec<Node>, mat: &glam::Mat4, node_index: usize) {
        let node = &nodes[node_index];
        let newmat = mat.mul_mat4(&node.transform);
        transforms[node_index] = newmat;
        for ci in &node.child_indices {
            self.compute_global_transforms_from(transforms, nodes, &newmat, *ci);
        }
    }

    pub fn compute_global_transforms(&self, nodes: &Vec<Node>, mat: &glam::Mat4) -> Vec<glam::Mat4> {
        let mut global_transforms = vec![glam::Mat4::IDENTITY; self.nodes.len()];
        for ni in &self.scene_node_indices {
            self.compute_global_transforms_from(&mut global_transforms, &nodes, &mat, *ni);
        }
        global_transforms
    }

    pub fn render_node(&self, ctx: &context::Context, shader: &shader::Shader, global_transforms: &Vec<glam::Mat4>, node_index: usize) {
        let node = &self.nodes[node_index];
        let transform = &global_transforms[node_index];
        let mut joint_matrices = vec![glam::Mat4::IDENTITY; 256];
        if let Some(skin) = node.skin_index.and_then(|i| self.skins.get(i)) {
            for (idx, ni) in skin.joints.iter().enumerate() {
                let jointnode_transform = &global_transforms[*ni];
                let ibm = skin.inverse_bind_matrices[idx];
                joint_matrices[idx] = jointnode_transform.mul_mat4(&ibm);
            }
        }
        if let Some(m) = node.mesh_index.and_then(|i| self.meshes.get(i)) {
            for p in &m.primitives {
                if let Some(tex) = self.materials.get(p.material_index).and_then(|m| m.base_color_texture.as_ref()) {
                    tex.bind(ctx);
                }
                unsafe {
                    gl::UniformMatrix4fv(shader.uniform_position, 1, false as u8, transform.to_cols_array().as_ptr());
                    gl::UniformMatrix4fv(
                        shader.uniform_joint_matrices, 256, false as u8,
                        joint_matrices.iter().map(|m| m.to_cols_array()).flatten().collect::<Vec<f32>>().as_ptr());
                    gl::BindVertexArray(p.vao);
                    gl::DrawElements(p.mode, p.count, p.index_type, p.index_offset as _);
                }
            }
        }
        for ci in &node.child_indices {
            self.render_node(ctx, shader, global_transforms, *ci);
        }
    }

    pub fn render(&self, ctx: &context::Context, shader: &shader::Shader, global_transforms: &Vec<glam::Mat4>) {
        for ni in &self.scene_node_indices {
            self.render_node(ctx, shader, global_transforms, *ni);
        }
    }
}
