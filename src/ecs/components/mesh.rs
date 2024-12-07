use crate::asset;
use bevy_ecs::component::Component;

#[derive(Component)]
pub struct Mesh {
    mesh_id: asset::MeshId,
}
