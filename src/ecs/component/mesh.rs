use crate::asset;
use bevy_ecs::component::Component;

#[derive(Component, Debug)]
pub struct Mesh {
    pub mesh_id: asset::MeshId,
}
