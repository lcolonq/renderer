use std::f32::consts::PI;

use crate::{shader, mesh, context, gl, texture};

pub struct Hat {
    pub shader: shader::Shader,
    pub mesh: mesh::Mesh,
    pub texture: Option<texture::Texture>,
    pub position: glam::Mat4,
}

impl Hat {
    pub fn cone(ctx: &context::Context) -> Self {
        Self {
            shader: shader::Shader::new(&ctx, "../assets/shadercone.vert", "../assets/shadercone.frag"),
            mesh: mesh::Mesh::new(std::path::Path::new("../assets/cone.obj")).unwrap(),
            texture: None,
            position: glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::new(0.07, 0.1, 0.07),
                glam::Quat::from_euler(
                    glam::EulerRot::XYZ,
                    0.8,
                    0.0,
                    0.5,
                ),
                glam::Vec3::new(-0.1, 1.80, 0.15)
            ),
        }
    }

    pub fn render(&self, ctx: &context::Context, view: &glam::Mat4, projection: &glam::Mat4, avatar: &super::Avatar, global_transforms: &Vec<glam::Mat4>) {
        self.shader.bind(&ctx);
        unsafe {
            gl::UniformMatrix4fv(self.shader.uniform_view, 1, false as u8, view.to_cols_array().as_ptr());
            gl::UniformMatrix4fv(self.shader.uniform_projection, 1, false as u8, projection.to_cols_array().as_ptr());

            let head_ni = *avatar.scene.bone_node_indices.get("head").unwrap();
            let ni = 83;
            let transform = global_transforms[ni];
            let node = &avatar.nodes[ni];
            let skin = avatar.scene.skins.get(
                node.skin_index.unwrap()
            ).unwrap();
            let idx = 10;
            // for (i, s) in skin.joints.iter().enumerate() {
            //     if *s == head_ni {
            //         log::info!("match: {}", i);
            //         idx = i;
            //     }
            // }
            let jointnode_transform = &global_transforms[head_ni];
            let ibm = skin.inverse_bind_matrices[idx];
            let skin_matrix = jointnode_transform.mul_mat4(&ibm);

            let final_pos = transform
                .mul_mat4(&skin_matrix)
                .mul_mat4(&self.position);
            gl::UniformMatrix4fv(self.shader.uniform_position, 1, false as u8, final_pos.to_cols_array().as_ptr());
        }
        if let Some(t) = &self.texture { t.bind(); }
        self.mesh.render();
    }
}

pub struct Pumpkin {
    pub shader: shader::Shader,
    pub mesh: mesh::Mesh,
    pub texture: Option<texture::Texture>,
    pub position: glam::Mat4,
}

impl Pumpkin {
    pub fn new(ctx: &context::Context) -> Self {
        Self {
            shader: shader::Shader::new(&ctx, "../assets/shaderpumpkin.vert", "../assets/shaderpumpkin.frag"),
            mesh: mesh::Mesh::new(std::path::Path::new("../assets/cube.obj")).unwrap(),
            texture: Some(texture::Texture::new(&ctx, "/home/llll/src/pumpkin/table/pumpkin.png")),
            position: glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::new(0.15, 0.15, 0.15),
                glam::Quat::from_rotation_z(PI / 2.0),
                // glam::Quat::from_euler(
                //     glam::EulerRot::XYZ,
                //     0.8,
                //     0.0,
                //     0.5,
                // ),
                glam::Vec3::new(0.0, 1.65, 0.0)
            ),
        }
    }

    pub fn reload(&self, ctx: &context::Context) {
        if let Some(t) = &self.texture {
            t.reload(ctx, "/home/llll/src/pumpkin/table/pumpkin.png");
        }
    }

    pub fn render(&self, ctx: &context::Context, view: &glam::Mat4, projection: &glam::Mat4, avatar: &super::Avatar, global_transforms: &Vec<glam::Mat4>) {
        self.shader.bind(&ctx);
        unsafe {
            gl::UniformMatrix4fv(self.shader.uniform_view, 1, false as u8, view.to_cols_array().as_ptr());
            gl::UniformMatrix4fv(self.shader.uniform_projection, 1, false as u8, projection.to_cols_array().as_ptr());

            let head_ni = *avatar.scene.bone_node_indices.get("head").unwrap();
            let ni = 83;
            let transform = global_transforms[ni];
            let node = &avatar.nodes[ni];
            let skin = avatar.scene.skins.get(
                node.skin_index.unwrap()
            ).unwrap();
            let idx = 10;
            // for (i, s) in skin.joints.iter().enumerate() {
            //     if *s == head_ni {
            //         log::info!("match: {}", i);
            //         idx = i;
            //     }
            // }
            let jointnode_transform = &global_transforms[head_ni];
            let ibm = skin.inverse_bind_matrices[idx];
            let skin_matrix = jointnode_transform.mul_mat4(&ibm);

            let final_pos = transform
                .mul_mat4(&skin_matrix)
                .mul_mat4(&self.position);
            gl::UniformMatrix4fv(self.shader.uniform_position, 1, false as u8, final_pos.to_cols_array().as_ptr());
            // gl::Disable(gl::CULL_FACE);
        }
        if let Some(t) = &self.texture { t.bind(); }
        self.mesh.render();
    }
}
