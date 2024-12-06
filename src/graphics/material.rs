use std::collections::HashMap;
use std::hash::Hash;

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, bytemuck::Pod, bytemuck::Zeroable, Hash, Debug)]
pub struct Material {
    // TODO: check alignment.
    base_color_texture_array_id: usize,
    base_color_texture_id: usize,
    normal_texture_array_id: usize,
    normal_texture_id: usize,
}

impl Material {
    pub fn new(
        base_color_texture_array_id: usize,
        base_color_texture_id: usize,
        normal_texture_array_id: usize,
        normal_texture_id: usize,
    ) -> Self {
        Self {
            base_color_texture_array_id,
            base_color_texture_id,
            normal_texture_array_id,
            normal_texture_id,
        }
    }
}

#[derive(Default, Debug)]
pub struct MaterialMap {
    materials: Vec<Material>,
    map: HashMap<Material, usize>,
}

impl MaterialMap {
    pub fn add(&mut self, material: Material) -> usize {
        if let Some(&material_index) = self.map.get(&material) {
            return material_index;
        }

        let material_index = self.materials.len();

        self.materials.push(material);
        self.map.insert(material, material_index);

        material_index
    }
}
