use super::Scene;
use crate::asset;
use crate::ecs;
use crate::graphics;
use bevy_hierarchy::BuildChildren;
use wgpu::util::DeviceExt;

pub struct PlayScene {
    world: bevy_ecs::world::World,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    mesh_buffer: wgpu::Buffer,
    material_buffer: wgpu::Buffer,
    texture_array_unorm_srgb_512: wgpu::Texture,
    texture_array_unorm_srgb_1024: wgpu::Texture,
    texture_array_unorm_srgb_2048: wgpu::Texture,
    texture_array_unorm_srgb_4096: wgpu::Texture,
    texture_array_unorm_512: wgpu::Texture,
    texture_array_unorm_1024: wgpu::Texture,
    texture_array_unorm_2048: wgpu::Texture,
    texture_array_unorm_4096: wgpu::Texture,
    texture_array_hdr_512: wgpu::Texture,
    texture_array_hdr_1024: wgpu::Texture,
    texture_array_hdr_2048: wgpu::Texture,
    texture_array_hdr_4096: wgpu::Texture,
}

impl Scene for PlayScene {
    fn update(&mut self) {
        // TODO: implement.
    }

    fn render(&self, gpu: &graphics::Gpu) {
        // TODO: implement.
    }
}

impl PlayScene {
    pub fn setup(gpu: &graphics::Gpu) -> Self {
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

        let vertex_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertex_buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let index_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("index_buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let mesh_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mesh_buffer"),
                contents: bytemuck::cast_slice(&meshes),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let material_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("material_buffer"),
                contents: bytemuck::cast_slice(&materials),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let create_texture = |label: Option<&str>,
                              texture_map: &asset::TextureMap,
                              dimension: u32,
                              channel: wgpu::AstcChannel| {
            let texture_descriptor = wgpu::TextureDescriptor {
                label,
                size: wgpu::Extent3d {
                    width: dimension,
                    height: dimension,
                    depth_or_array_layers: (texture_map.map.len() as u32).max(1),
                },
                mip_level_count: dimension.ilog2() + 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Astc {
                    block: wgpu::AstcBlock::B4x4,
                    channel,
                },
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, // TODO: change usage
                view_formats: &[],
            };

            if texture_map.map.is_empty() {
                gpu.device.create_texture(&texture_descriptor)
            } else {
                gpu.device.create_texture_with_data(
                    &gpu.queue,
                    &texture_descriptor,
                    wgpu::util::TextureDataOrder::LayerMajor,
                    bytemuck::cast_slice(&texture_map.textures),
                )
            }
        };

        let texture_array_unorm_srgb_512 = create_texture(
            Some("2d_texture_array_unorm_srgb_512"),
            &texture_arrays.unorm_srgb_512,
            512,
            wgpu::AstcChannel::UnormSrgb,
        );

        let texture_array_unorm_srgb_1024 = create_texture(
            Some("2d_texture_array_unorm_srgb_1024"),
            &texture_arrays.unorm_srgb_1024,
            1024,
            wgpu::AstcChannel::UnormSrgb,
        );

        let texture_array_unorm_srgb_2048 = create_texture(
            Some("2d_texture_array_unorm_srgb_2048"),
            &texture_arrays.unorm_srgb_2048,
            2048,
            wgpu::AstcChannel::UnormSrgb,
        );

        let texture_array_unorm_srgb_4096 = create_texture(
            Some("2d_texture_array_unorm_srgb_4096"),
            &texture_arrays.unorm_srgb_4096,
            4096,
            wgpu::AstcChannel::UnormSrgb,
        );

        let texture_array_unorm_512 = create_texture(
            Some("2d_texture_array_unorm_512"),
            &texture_arrays.unorm_512,
            512,
            wgpu::AstcChannel::Unorm,
        );

        let texture_array_unorm_1024 = create_texture(
            Some("2d_texture_array_unorm_1024"),
            &texture_arrays.unorm_1024,
            1024,
            wgpu::AstcChannel::Unorm,
        );

        let texture_array_unorm_2048 = create_texture(
            Some("2d_texture_array_unorm_2048"),
            &texture_arrays.unorm_2048,
            2048,
            wgpu::AstcChannel::Unorm,
        );

        let texture_array_unorm_4096 = create_texture(
            Some("2d_texture_array_unorm_4096"),
            &texture_arrays.unorm_4096,
            4096,
            wgpu::AstcChannel::Unorm,
        );

        let texture_array_hdr_512 = create_texture(
            Some("2d_texture_array_hdr_512"),
            &texture_arrays.hdr_512,
            512,
            wgpu::AstcChannel::Hdr,
        );

        let texture_array_hdr_1024 = create_texture(
            Some("2d_texture_array_hdr_1024"),
            &texture_arrays.hdr_1024,
            1024,
            wgpu::AstcChannel::Hdr,
        );

        let texture_array_hdr_2048 = create_texture(
            Some("2d_texture_array_hdr_2048"),
            &texture_arrays.hdr_2048,
            2048,
            wgpu::AstcChannel::Hdr,
        );

        let texture_array_hdr_4096 = create_texture(
            Some("2d_texture_array_hdr_4096"),
            &texture_arrays.hdr_4096,
            4096,
            wgpu::AstcChannel::Hdr,
        );

        Self {
            world,
            vertex_buffer,
            index_buffer,
            mesh_buffer,
            material_buffer,
            texture_array_unorm_srgb_512,
            texture_array_unorm_srgb_1024,
            texture_array_unorm_srgb_2048,
            texture_array_unorm_srgb_4096,
            texture_array_unorm_512,
            texture_array_unorm_1024,
            texture_array_unorm_2048,
            texture_array_unorm_4096,
            texture_array_hdr_512,
            texture_array_hdr_1024,
            texture_array_hdr_2048,
            texture_array_hdr_4096,
        }
    }
}
