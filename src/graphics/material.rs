use std::hash::Hash;

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, bytemuck::Pod, bytemuck::Zeroable, Hash, Debug)]
pub struct Material {
    // TODO: check alignment.
    base_color_texture_array_id: u32,
    base_color_texture_id: u32,
    normal_texture_array_id: u32,
    normal_texture_id: u32,
}

impl Material {
    pub fn new(
        base_color_texture_array_id: u32,
        base_color_texture_id: u32,
        normal_texture_array_id: u32,
        normal_texture_id: u32,
    ) -> Self {
        Self {
            base_color_texture_array_id,
            base_color_texture_id,
            normal_texture_array_id,
            normal_texture_id,
        }
    }
}
