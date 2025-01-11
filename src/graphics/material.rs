use crate::asset;

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, bytemuck::Pod, bytemuck::Zeroable, Default, Debug)]
pub struct MaterialBitmask(u32);

bitflags::bitflags! {
    impl MaterialBitmask: u32 {
        const CLEAR = 1 << 0;
        const BASE_COLOR_FLAG = 1 << 1;
        const NORMAL_FLAG = 1 << 2;
        const OCCLUSION_FLAG = 1 << 3;
        const ROUGHNESS_FLAG = 1 << 4;
        const METALLIC_FLAG = 1 << 5;
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Material {
    pub base_color_rgba: [f32; 4],
    pub normal_solid: [f32; 3],
    pub base_color_texture: TextureReference,
    pub normal_texture: TextureReference,
    pub occlusion_solid: f32,
    pub occlusion_texture: TextureReference,
    pub occlusion_texture_channel: u32,
    pub roughness_solid: f32,
    pub roughness_texture: TextureReference,
    pub roughness_texture_channel: u32,
    pub metallic_solid: f32,
    pub metallic_texture: TextureReference,
    pub metallic_texture_channel: u32,
    pub bitmask: MaterialBitmask,
}

impl From<asset::Material> for Material {
    fn from(material: asset::Material) -> Self {
        let (base_color_flag, base_color_rgba, base_color_texture) = match material.base_color {
            asset::BaseColor::Solid { r, g, b, a } => (
                MaterialBitmask::CLEAR,
                [*r, *g, *b, *a],
                TextureReference::default(),
            ),
            asset::BaseColor::Texture(texture_reference) => (
                MaterialBitmask::BASE_COLOR_FLAG,
                <[f32; 4]>::default(),
                texture_reference.into(),
            ),
        };

        let (normal_flag, normal_solid, normal_texture) =
            if let Some(normal_texture) = material.normal {
                (
                    MaterialBitmask::NORMAL_FLAG,
                    <[f32; 3]>::default(),
                    normal_texture.into(),
                )
            } else {
                (
                    MaterialBitmask::CLEAR,
                    [0.5, 0.5, 1.0],
                    TextureReference::default(),
                )
            };

        let (occlusion_flag, occlusion_solid, occlusion_texture, occlusion_texture_channel) =
            match material.occlusion {
                asset::Occlusion::Solid(occlusion_solid) => (
                    MaterialBitmask::CLEAR,
                    *occlusion_solid,
                    TextureReference::default(),
                    u32::default(),
                ),
                asset::Occlusion::Texture {
                    texture_reference,
                    channel,
                } => (
                    MaterialBitmask::OCCLUSION_FLAG,
                    f32::default(),
                    texture_reference.into(),
                    channel,
                ),
            };

        let (roughness_flag, roughness_solid, roughness_texture, roughness_texture_channel) =
            match material.roughness {
                asset::Roughness::Solid(roughness_solid) => (
                    MaterialBitmask::CLEAR,
                    *roughness_solid,
                    TextureReference::default(),
                    u32::default(),
                ),
                asset::Roughness::Texture {
                    texture_reference,
                    channel,
                } => (
                    MaterialBitmask::ROUGHNESS_FLAG,
                    f32::default(),
                    texture_reference.into(),
                    channel,
                ),
            };

        let (metallic_flag, metallic_solid, metallic_texture, metallic_texture_channel) =
            match material.metallic {
                asset::Metallic::Solid(metallic_solid) => (
                    MaterialBitmask::CLEAR,
                    *metallic_solid,
                    TextureReference::default(),
                    u32::default(),
                ),
                asset::Metallic::Texture {
                    texture_reference,
                    channel,
                } => (
                    MaterialBitmask::METALLIC_FLAG,
                    f32::default(),
                    texture_reference.into(),
                    channel,
                ),
            };

        Self {
            bitmask: base_color_flag
                | normal_flag
                | occlusion_flag
                | roughness_flag
                | metallic_flag,
            base_color_rgba,
            base_color_texture,
            normal_solid,
            normal_texture,
            occlusion_solid,
            occlusion_texture,
            occlusion_texture_channel,
            roughness_solid,
            roughness_texture,
            roughness_texture_channel,
            metallic_solid,
            metallic_texture,
            metallic_texture_channel,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Hash, bytemuck::Pod, bytemuck::Zeroable, Default, Debug)]
pub struct TextureReference {
    texture_array_id: u32,
    texture_id: u32,
}

impl From<asset::TextureReference> for TextureReference {
    fn from(texture_reference: asset::TextureReference) -> Self {
        Self {
            texture_array_id: texture_reference.texture_array_id,
            texture_id: texture_reference.texture_id,
        }
    }
}
