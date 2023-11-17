use std::collections::HashMap;

use gltf_derive::Validate;
use serde_derive::{Deserialize, Serialize};

/// The root object of a glTF 2.0 asset.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct Root {
    #[cfg(feature = "KHR_lights_punctual")]
    #[serde(
        default,
        rename = "KHR_lights_punctual",
        skip_serializing_if = "Option::is_none"
    )]
    pub khr_lights_punctual: Option<KhrLightsPunctual>,

    #[cfg(feature = "KHR_materials_variants")]
    #[serde(
        default,
        rename = "KHR_materials_variants",
        skip_serializing_if = "Option::is_none"
    )]
    pub khr_materials_variants: Option<KhrMaterialsVariants>,

    #[serde(
        default,
        rename = "VRMC_vrm",
        skip_serializing_if = "Option::is_none"
    )]
    pub vrmc_vrm: Option<VrmcVrm>,
}

#[cfg(feature = "KHR_lights_punctual")]
#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct KhrLightsPunctual {
    /// Lights at this node.
    pub lights: Vec<crate::extensions::scene::khr_lights_punctual::Light>,
}

#[cfg(feature = "KHR_lights_punctual")]
impl crate::root::Get<crate::extensions::scene::khr_lights_punctual::Light> for crate::Root {
    fn get(
        &self,
        id: crate::Index<crate::extensions::scene::khr_lights_punctual::Light>,
    ) -> Option<&crate::extensions::scene::khr_lights_punctual::Light> {
        if let Some(extensions) = self.extensions.as_ref() {
            if let Some(khr_lights_punctual) = extensions.khr_lights_punctual.as_ref() {
                khr_lights_punctual.lights.get(id.value())
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(feature = "KHR_materials_variants")]
#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct KhrMaterialsVariants {
    pub variants: Vec<crate::extensions::scene::khr_materials_variants::Variant>,
}

#[cfg(feature = "KHR_materials_variants")]
impl crate::root::Get<crate::extensions::scene::khr_materials_variants::Variant> for crate::Root {
    fn get(
        &self,
        id: crate::Index<crate::extensions::scene::khr_materials_variants::Variant>,
    ) -> Option<&crate::extensions::scene::khr_materials_variants::Variant> {
        self.extensions
            .as_ref()?
            .khr_materials_variants
            .as_ref()?
            .variants
            .get(id.value())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct VrmcVrm {
    #[serde(
        default,
        rename = "specVersion",
    )]
    pub spec_version: String,
    pub expressions: VrmExpressions,
    pub humanoid: VrmHumanoid,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct VrmExpressions {
    pub preset: HashMap<String, VrmExpression>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct VrmExpression {
    #[serde(
        default,
        rename = "isBinary",
    )]
    pub is_binary: bool,

    #[serde(
        default,
        rename = "morphTargetBinds",
    )]
    pub morph_target_binds: Vec<MorphTargetBind>,

    #[serde(
        default,
        rename = "materialColorBinds",
    )]
    pub material_color_binds: Vec<MaterialValueBind>,

    #[serde(
        default,
        rename = "textureTransformBinds",
    )]
    pub texture_transform_binds: Vec<TextureTransformBind>,

    #[serde(
        default,
        rename = "overrideMouth",
    )]
    pub override_mouth: String,

    #[serde(
        default,
        rename = "overrideBlink",
    )]
    pub override_blink: String,

    #[serde(
        default,
        rename = "overrideLookAt",
    )]
    pub override_look_at: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct MorphTargetBind {
    pub node: u32,
    pub index: u32,
    pub weight: f32,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct MaterialValueBind {
    pub material: u32,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "targetValue")]
    pub target_value: Vec<f32>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct TextureTransformBind {
    pub material: u32,
    pub scale: Vec<f32>,
    pub offset: Vec<f32>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct VrmHumanoid {
    #[serde(
        default,
        rename = "humanBones",
    )]
    pub human_bones: HashMap<String, HumanBone>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct HumanBone {
    pub node: u32,
}
