use super::{rendering, resource::*, simulation};
use crate::asset;
use crate::ecs;
use crate::graphics;
use crate::physics;

use bevy_hierarchy::BuildChildren;
use wgpu::util::DeviceExt;

pub const MSAA_SAMPLE_COUNT: u32 = 4;

pub struct Scene {
    simulator: simulation::Simulator,
}

impl Scene {
    pub fn setup(gpu: graphics::Gpu<'static>) -> Self {
        let (scene_to_renderer_sender, scene_to_renderer_receiver) = crossbeam::channel::bounded(1);
        let (renderer_to_scene_sender, renderer_to_scene_receiver) = crossbeam::channel::bounded(1);

        let mut world = bevy_ecs::world::World::new();
        let mut render_world = bevy_ecs::world::World::new();

        load_scene(&mut world, &mut render_world, &gpu);

        render_world.insert_resource(gpu);
        renderer_to_scene_sender.send(render_world).unwrap();

        let simulator = simulation::Simulator::new();
        simulator.spawn(world, scene_to_renderer_sender, renderer_to_scene_receiver);

        rendering::Renderer::spawn(renderer_to_scene_sender, scene_to_renderer_receiver);

        Self { simulator }
    }

    pub fn update(&mut self) {
        self.simulator.request_update();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.simulator.request_resize(width, height);
    }
}

fn load_scene(
    main_world: &mut bevy_ecs::world::World,
    render_world: &mut bevy_ecs::world::World,
    gpu: &graphics::Gpu<'static>,
) {
    main_world.clear_all();
    render_world.clear_all();

    let now = std::time::Instant::now();
    main_world.insert_resource(Timestamp(now));
    main_world.insert_resource(DeltaTime(std::time::Duration::ZERO));
    main_world.insert_resource(physics::LastStepTimestamp(
        now - std::time::Duration::from_secs_f32(physics::TIMESTEP),
    ));

    main_world.insert_resource(ecs::resource::Camera {
        position: (-1.2, 0.6, 1.2).into(),
        target: (0.0, 0.3, 0.0).into(),
        aspect_ratio: gpu.config.width as f32 / gpu.config.height as f32,
        ..Default::default()
    });

    let mut asset_loader = asset::AssetLoader::new();

    let model_id = asset_loader
        .load_gltf_model(asset::assets_path().join("flight_helmet/flight_helmet.gltf"))
        .unwrap();

    let cubemap = asset_loader
        .load_cubemap(
            asset::assets_path().join("cubemap").join("px.ktx2"),
            asset::assets_path().join("cubemap").join("nx.ktx2"),
            asset::assets_path().join("cubemap").join("py.ktx2"),
            asset::assets_path().join("cubemap").join("ny.ktx2"),
            asset::assets_path().join("cubemap").join("pz.ktx2"),
            asset::assets_path().join("cubemap").join("nz.ktx2"),
        )
        .unwrap();

    let asset::AssetLoader {
        mut mesh_map,
        texture_arrays,
        texture_dictionary: _texture_dictionary,
        mut material_map,
        model_map,
    } = asset_loader;

    let mut physics_world = physics::Physics::default();

    // Physics test
    {
        // Ground
        let collider_handle = physics_world.collider_set.insert(
            rapier3d::geometry::ColliderBuilder::cuboid(100.0, 0.1, 100.0)
                .translation([0.0, -0.1, 0.0].into())
                .restitution(0.8)
                .build(),
        );
        let _ = main_world.spawn((
            ecs::component::GlobalTransform(glam::Affine3A::default()),
            physics::Collider(collider_handle),
        ));

        // Bouncing ball
        let icosphere = graphics::mesh::primitive::Icosphere::with_subdivision_level(3);

        let icosphere_mesh_id = mesh_map.push(
            icosphere.canonic_name(),
            icosphere.vertices,
            icosphere.indices,
            graphics::BoundingBox::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]),
        );

        let icosphere_material_id = material_map.add(asset::Material {
            base_color: asset::BaseColor::Solid {
                r: 0.0.into(),
                g: 0.0.into(),
                b: 1.0.into(),
                a: 1.0.into(),
            },
            normal: None,
            occlusion: asset::Occlusion::Solid(1.0.into()),
            metallic: asset::Metallic::Solid(0.5.into()),
            roughness: asset::Roughness::Solid(0.5.into()),
        });

        let start_position = [0.0, 1.5, -0.5];
        let rigid_body = rapier3d::dynamics::RigidBodyBuilder::dynamic()
            .translation(start_position.into())
            .build();
        let rigid_body_handle = physics_world.rigid_body_set.insert(rigid_body);
        let collider = rapier3d::geometry::ColliderBuilder::ball(0.1)
            .restitution(0.8)
            .build();
        let collider_handle = physics_world.collider_set.insert_with_parent(
            collider,
            rigid_body_handle,
            &mut physics_world.rigid_body_set,
        );
        let _ = main_world.spawn((
            ecs::component::Mesh {
                mesh_id: icosphere_mesh_id,
            },
            ecs::component::Material {
                material_id: icosphere_material_id,
            },
            ecs::component::GlobalTransform(glam::Affine3A::from_scale_rotation_translation(
                glam::Vec3::new(0.05, 0.05, 0.05),
                glam::Quat::IDENTITY,
                glam::Vec3::from(start_position),
            )),
            physics::RigidBody(rigid_body_handle),
            physics::Collider(collider_handle),
        ));
    }

    let asset::MeshMap {
        vertices,
        indices,
        meshes,
        bounding_boxes,
        map: _meshes_map,
    } = mesh_map;

    let asset::MaterialMap {
        materials,
        map: _materials_map,
    } = material_map;

    render_world.insert_resource(Meshes(meshes));

    main_world.insert_resource(physics_world);

    let mut commands = main_world.commands();

    let model = model_map.index(model_id).unwrap();

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
                                    ecs::component::GlobalTransform::default(),
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
                                    ecs::component::GlobalTransform(
                                        glam::Affine3A::from_translation(glam::Vec3::new(
                                            0.5, 0.0, 0.0,
                                        )),
                                    ),
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
                                    ecs::component::GlobalTransform(
                                        glam::Affine3A::from_translation(glam::Vec3::new(
                                            -0.5, 0.0, 0.0,
                                        )),
                                    ),
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

    main_world.flush();

    let bounding_boxes_buffer = graphics::pipeline::render::skybox::create_bounding_boxes_buffer(
        &gpu.device,
        &bounding_boxes,
    );
    render_world.insert_resource(BoundingBoxesBuffer(bounding_boxes_buffer));

    let compute_pipeline_frustum_culling =
        graphics::pipeline::compute::FrustumCulling::new(&gpu.device);
    render_world.insert_resource(ComputePipelineFrustumCulling(
        compute_pipeline_frustum_culling,
    ));

    let vertex_buffer = gpu
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
        });
    render_world.insert_resource(VertexBuffer(vertex_buffer));

    let index_buffer = gpu
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index_buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::STORAGE,
        });
    render_world.insert_resource(IndexBuffer(index_buffer));

    let material_buffer = graphics::pipeline::render::pbr::create_material_buffer(
        &gpu.device,
        &materials
            .into_iter()
            .map(graphics::Material::from)
            .collect::<Vec<_>>(),
    );

    let render_pipeline_pbr =
        graphics::pipeline::render::Pbr::new(&gpu.device, gpu.config.format, MSAA_SAMPLE_COUNT);

    let texture_array_handles = graphics::pipeline::render::pbr::create_texture_arrays_init(
        &gpu.device,
        &gpu.queue,
        &graphics::pipeline::render::pbr::TextureArrays {
            rg_bc5_unorm_512: texture_arrays.rg_bc5_unorm_512,
            rg_bc5_unorm_1024: texture_arrays.rg_bc5_unorm_1024,
            rg_bc5_unorm_2048: texture_arrays.rg_bc5_unorm_2048,
            rg_bc5_unorm_4096: texture_arrays.rg_bc5_unorm_4096,
            rgb_bc7_unorm_512: texture_arrays.rgb_bc7_unorm_512,
            rgb_bc7_unorm_1024: texture_arrays.rgb_bc7_unorm_1024,
            rgb_bc7_unorm_2048: texture_arrays.rgb_bc7_unorm_2048,
            rgb_bc7_unorm_4096: texture_arrays.rgb_bc7_unorm_4096,
            rgba_bc7_srgb_512: texture_arrays.rgba_bc7_srgb_512,
            rgba_bc7_srgb_1024: texture_arrays.rgba_bc7_srgb_1024,
            rgba_bc7_srgb_2048: texture_arrays.rgba_bc7_srgb_2048,
            rgba_bc7_srgb_4096: texture_arrays.rgba_bc7_srgb_4096,
        },
    );

    let texture_array_views =
        graphics::pipeline::render::pbr::create_texture_array_views(texture_array_handles);

    let bind_group_bindless = render_pipeline_pbr.create_bind_group_bindless(
        &gpu.device,
        material_buffer.as_entire_binding(),
        texture_array_views,
        wgpu::BindingResource::Sampler(&gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("texture_array_sampler_base_color"),
            ..Default::default()
        })),
        wgpu::BindingResource::Sampler(&gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("texture_array_sampler_normal"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        })),
    );
    render_world.insert_resource(RenderPipelinePbr(render_pipeline_pbr));
    render_world.insert_resource(BindGroupBindless(bind_group_bindless));

    let depth_buffer_view = graphics::gpu::create_depth_buffer(
        &gpu.device,
        gpu.config.width,
        gpu.config.height,
        MSAA_SAMPLE_COUNT,
    );
    render_world.insert_resource(DepthBuffer(depth_buffer_view));

    let cubemap_size = cubemap.texture_array.size().0 as u32;
    let texture_skybox = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("texture_skybox"),
        size: wgpu::Extent3d {
            width: cubemap_size,
            height: cubemap_size,
            depth_or_array_layers: 6,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bc6hRgbFloat,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for (layer_index, &face_texture_id) in [
        cubemap.positive_x,
        cubemap.negative_x,
        cubemap.positive_y,
        cubemap.negative_y,
        cubemap.positive_z,
        cubemap.negative_z,
    ]
    .iter()
    .enumerate()
    {
        const BYTES_PER_BLOCK: u32 = 16;
        const BLOCK_SIZE: u32 = 4;

        let face_data = texture_arrays
            .no_mip_rgb_bc6h_sfloat_1024
            .get(face_texture_id, 0)
            .unwrap();

        gpu.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture_skybox,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: layer_index as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            face_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(BYTES_PER_BLOCK * cubemap_size / BLOCK_SIZE),
                rows_per_image: Some(cubemap_size / BLOCK_SIZE),
            },
            wgpu::Extent3d {
                width: cubemap_size,
                height: cubemap_size,
                depth_or_array_layers: 1,
            },
        );
    }

    let render_pipeline_skybox =
        graphics::pipeline::render::Skybox::new(&gpu.device, gpu.config.format, MSAA_SAMPLE_COUNT);
    let bind_group_skybox = render_pipeline_skybox.create_bind_group_skybox(
        &gpu.device,
        wgpu::BindingResource::TextureView(&texture_skybox.create_view(
            &wgpu::TextureViewDescriptor {
                label: Some("texture_skybox"),
                format: Some(wgpu::TextureFormat::Bc6hRgbFloat),
                dimension: Some(wgpu::TextureViewDimension::Cube),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            },
        )),
        wgpu::BindingResource::Sampler(&gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("texture_sampler_skybox"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        })),
    );
    render_world.insert_resource(RenderPipelineSkybox(render_pipeline_skybox));
    render_world.insert_resource(BindGroupSkybox(bind_group_skybox));

    let msaa_buffer_view = graphics::gpu::create_msaa_buffer(
        &gpu.device,
        gpu.config.width,
        gpu.config.height,
        gpu.config.format,
        MSAA_SAMPLE_COUNT,
    );
    render_world.insert_resource(MsaaBuffer(msaa_buffer_view));
}
