use crate::asset;
use bevy_ecs::component::Component;

#[derive(Component)]
pub struct Material {
    material_id: asset::MaterialId,
}
