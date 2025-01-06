use crate::asset;
use bevy_ecs::component::Component;

#[derive(Component, Clone, Debug)]
pub struct Material {
    pub material_id: asset::MaterialId,
}
