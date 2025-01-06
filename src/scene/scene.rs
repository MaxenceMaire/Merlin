use super::{rendering, resource::*, simulation};
use crate::asset;
use crate::ecs;
use crate::graphics;

use bevy_hierarchy::BuildChildren;
use wgpu::util::DeviceExt;

const MSAA_SAMPLE_COUNT: u32 = 4;

pub struct Scene {
    simulator: simulation::Simulator,
}

impl Scene {
    pub fn setup(gpu: graphics::Gpu<'static>) -> Self {
        let (scene_to_renderer_sender, scene_to_renderer_receiver) = crossbeam_channel::bounded(1);
        let (renderer_to_scene_sender, renderer_to_scene_receiver) = crossbeam_channel::bounded(1);

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
}

fn load_scene(
    main_world: &mut bevy_ecs::world::World,
    render_world: &mut bevy_ecs::world::World,
    gpu: &graphics::Gpu<'static>,
) {
    main_world.clear_all();
    render_world.clear_all();

    main_world.insert_resource(Timestamp(std::time::Instant::now()));

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
        mesh_map,
        texture_arrays,
        texture_dictionary: _texture_dictionary,
        material_map,
        model_map,
    } = asset_loader;

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

    let model = model_map.index(model_id).unwrap();

    let mut commands = main_world.commands();

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
                                        ))
                                        .to_cols_array(),
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
                                        ))
                                        .to_cols_array(),
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

    let material_buffer =
        graphics::pipeline::render::pbr::create_material_buffer(&gpu.device, &materials);

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

    let depth_buffer_view = create_depth_buffer(&gpu.device, gpu.config.width, gpu.config.height);
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

    let msaa_buffer_view = create_msaa_buffer(
        &gpu.device,
        gpu.config.width,
        gpu.config.height,
        gpu.config.format,
    );
    render_world.insert_resource(MsaaBuffer(msaa_buffer_view));
}

fn create_depth_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let depth_buffer = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth_buffer"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: MSAA_SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    depth_buffer.create_view(&wgpu::TextureViewDescriptor::default())
}

fn create_msaa_buffer(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    texture_format: wgpu::TextureFormat,
) -> wgpu::TextureView {
    let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("msaa_texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: MSAA_SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: texture_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    msaa_texture.create_view(&wgpu::TextureViewDescriptor::default())
}
