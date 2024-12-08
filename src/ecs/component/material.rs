use crate::asset;
use bevy_ecs::component::Component;

#[derive(Component, Debug)]
pub struct Material {
    pub material_id: asset::MaterialId,
}
