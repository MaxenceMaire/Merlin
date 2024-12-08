use crate::asset;
use crate::ecs;
use bevy_hierarchy::BuildChildren;

pub struct Scene {
    world: bevy_ecs::world::World,
}

impl Scene {
    pub fn new() -> Self {
        let mut world = bevy_ecs::world::World::new();

        world.insert_resource(ecs::resource::Camera::default());

        let mut asset_loader = asset::AssetLoader::new();
        let model_id = asset_loader
            .load_gltf_model("assets/FlightHelmet.gltf")
            .unwrap();

        let asset::AssetLoader {
            mesh_map,
            texture_arrays,
            texture_dictionary,
            material_map,
            model_map,
        } = asset_loader;

        let asset::MeshMap {
            vertices,
            indices,
            meshes,
            map: meshes_map,
        } = mesh_map;

        let asset::MaterialMap {
            materials,
            map: materials_map,
        } = material_map;

        let model = model_map.index(model_id).unwrap();

        let mut commands = world.commands();

        let root = commands.spawn(()).id();
        let mut stack: Vec<(usize, bevy_ecs::entity::Entity)> = model
            .root_nodes
            .iter()
            .map(|&node_index| (node_index, root))
            .collect();

        while let Some((node_index, parent_entity)) = stack.pop() {
            let node = model.nodes.get(node_index).unwrap();

            let objects = node
                .object_group
                .as_ref()
                .map(|object_group| {
                    object_group
                        .objects
                        .iter()
                        .map(
                            |&asset::Object {
                                 mesh_id,
                                 material_id,
                             }| {
                                commands
                                    .spawn((
                                        ecs::component::Mesh { mesh_id },
                                        ecs::component::Material { material_id },
                                    ))
                                    .id()
                            },
                        )
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let mut entity_commands = commands.spawn(());
            entity_commands.add_children(&objects);
            let entity = entity_commands.id();

            commands.entity(parent_entity).add_child(entity);

            for &child_index in &node.children {
                stack.push((child_index, entity));
            }
        }

        world.flush();

        println!("{:#?}", world.get_resource::<ecs::resource::Camera>());

        let mut query = world.query::<(&ecs::component::Mesh, &ecs::component::Material)>();
        for (mesh, material) in query.iter(&world) {
            println!("{:?} {:?}", mesh, material);
        }

        Self { world }
    }
}
