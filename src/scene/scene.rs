use crate::asset;
use crate::ecs;
use crate::graphics;

use std::ops::{Deref, DerefMut};

use bevy_hierarchy::BuildChildren;
use wgpu::util::DeviceExt;

const MSAA_SAMPLE_COUNT: u32 = 4;

pub struct Scene {
    world: bevy_ecs::world::World,
    render_thread: std::thread::JoinHandle<()>,
    scene_to_renderer_sender: crossbeam_channel::Sender<bevy_ecs::world::World>,
    renderer_to_scene_receiver: crossbeam_channel::Receiver<bevy_ecs::world::World>,
    update_schedule: bevy_ecs::schedule::Schedule,
}

impl Scene {
    pub fn setup(gpu: graphics::Gpu<'static>) -> Self {
        let (scene_to_renderer_sender, scene_to_renderer_receiver) = crossbeam_channel::bounded(1);
        let (renderer_to_scene_sender, renderer_to_scene_receiver) = crossbeam_channel::bounded(1);

        let mut world = bevy_ecs::world::World::new();
        let mut render_world = bevy_ecs::world::World::new();

        load_scene(&mut world, &mut render_world, &gpu);

        render_world.insert_resource(gpu);

        let render_thread = renderer::spawn(renderer_to_scene_sender, scene_to_renderer_receiver);

        extract_world(&mut world, &mut render_world);
        scene_to_renderer_sender.send(render_world).unwrap();

        let update_schedule = schedule::update();

        Self {
            world,
            render_thread,
            scene_to_renderer_sender,
            renderer_to_scene_receiver,
            update_schedule,
        }
    }

    pub fn update(&mut self) {
        let Ok(mut render_world) = self.renderer_to_scene_receiver.try_recv() else {
            return;
        };
        extract_world(&mut self.world, &mut render_world);
        self.scene_to_renderer_sender.send(render_world).unwrap();

        self.update_schedule.run(&mut self.world);
    }
}

fn extract_world(
    main_world: &mut bevy_ecs::world::World,
    render_world: &mut bevy_ecs::world::World,
) {
    let camera = main_world.get_resource::<ecs::resource::Camera>().unwrap();
    render_world.insert_resource::<ecs::resource::Camera>(camera.clone());

    // TODO: extract only visible entities.

    let mut query = main_world.query::<(
        bevy_ecs::entity::Entity,
        &ecs::component::Mesh,
        &ecs::component::Material,
        &ecs::component::GlobalTransform,
    )>();
    render_world
        .insert_or_spawn_batch(query.iter(main_world).map(
            |(entity, mesh, material, global_transform)| {
                (entity, (mesh.clone(), material.clone(), *global_transform))
            },
        ))
        .unwrap();
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

#[derive(bevy_ecs::prelude::Resource)]
struct Timestamp(std::time::Instant);

impl Deref for Timestamp {
    type Target = std::time::Instant;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Timestamp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::prelude::Resource)]
struct Meshes(Vec<graphics::Mesh>);

impl Deref for Meshes {
    type Target = Vec<graphics::Mesh>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Meshes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::prelude::Resource)]
struct VertexBuffer(wgpu::Buffer);

impl Deref for VertexBuffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VertexBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::prelude::Resource)]
struct IndexBuffer(wgpu::Buffer);

impl Deref for IndexBuffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for IndexBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::prelude::Resource)]
struct BoundingBoxesBuffer(wgpu::Buffer);

impl Deref for BoundingBoxesBuffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BoundingBoxesBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::prelude::Resource)]
struct DepthBuffer(wgpu::TextureView);

impl Deref for DepthBuffer {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DepthBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::prelude::Resource)]
struct MsaaBuffer(wgpu::TextureView);

impl Deref for MsaaBuffer {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MsaaBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::prelude::Resource)]
struct ComputePipelineFrustumCulling(graphics::pipeline::compute::FrustumCulling);

impl Deref for ComputePipelineFrustumCulling {
    type Target = graphics::pipeline::compute::FrustumCulling;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ComputePipelineFrustumCulling {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::prelude::Resource)]
struct RenderPipelinePbr(graphics::pipeline::render::Pbr);

impl Deref for RenderPipelinePbr {
    type Target = graphics::pipeline::render::Pbr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RenderPipelinePbr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::prelude::Resource)]
struct RenderPipelineSkybox(graphics::pipeline::render::Skybox);

impl Deref for RenderPipelineSkybox {
    type Target = graphics::pipeline::render::Skybox;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RenderPipelineSkybox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::prelude::Resource)]
struct BindGroupBindless(wgpu::BindGroup);

impl Deref for BindGroupBindless {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BindGroupBindless {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(bevy_ecs::prelude::Resource)]
struct BindGroupSkybox(wgpu::BindGroup);

impl Deref for BindGroupSkybox {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BindGroupSkybox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
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

mod schedule {
    use super::system;

    pub fn update() -> bevy_ecs::schedule::Schedule {
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(system::move_camera);

        schedule
    }
}

mod system {
    use super::*;
    use bevy_ecs::change_detection::ResMut;

    pub fn move_camera(
        mut camera: ResMut<ecs::resource::Camera>,
        mut timestamp: ResMut<Timestamp>,
    ) {
        let now = std::time::Instant::now();
        let delta_time = now - **timestamp;

        let rotation = glam::Quat::from_axis_angle(
            glam::f32::Vec3::Y.normalize(),
            delta_time.as_millis() as f32 * 0.0001,
        );
        camera.position = rotation * camera.position;

        **timestamp = now;
    }
}

mod renderer {
    pub fn spawn(
        renderer_to_scene_sender: crossbeam_channel::Sender<bevy_ecs::world::World>,
        scene_to_renderer_receiver: crossbeam_channel::Receiver<bevy_ecs::world::World>,
    ) -> std::thread::JoinHandle<()> {
        let mut render_schedule = schedule::rendering();

        std::thread::spawn(move || loop {
            let Ok(mut render_world) = scene_to_renderer_receiver.recv() else {
                // Channel disconnected.
                return;
            };

            render_schedule.run(&mut render_world);

            // TODO: clean up render world.
            render_world.clear_entities();

            let send_result = renderer_to_scene_sender.send(render_world);
            if send_result.is_err() {
                // Channel disconnected.
                return;
            }
        })
    }

    // TODO: remove pub
    pub mod schedule {
        use super::system;

        pub fn rendering() -> bevy_ecs::schedule::Schedule {
            let mut schedule = bevy_ecs::schedule::Schedule::default();
            schedule.add_systems(system::render);

            schedule
        }
    }

    mod system {
        use super::super::*;
        use crate::ecs;
        use crate::graphics;
        use bevy_ecs::change_detection::Res;
        use bevy_ecs::system::Query;
        use wgpu::util::DeviceExt;

        pub fn render(
            (gpu, camera, meshes): (
                Res<graphics::Gpu<'static>>,
                Res<ecs::resource::Camera>,
                Res<Meshes>,
            ),
            (vertex_buffer, index_buffer, bounding_boxes_buffer): (
                Res<VertexBuffer>,
                Res<IndexBuffer>,
                Res<BoundingBoxesBuffer>,
            ),
            (bind_group_bindless, bind_group_skybox): (
                Res<BindGroupBindless>,
                Res<BindGroupSkybox>,
            ),
            (compute_pipeline_frustum_culling, render_pipeline_pbr, render_pipeline_skybox): (
                Res<ComputePipelineFrustumCulling>,
                Res<RenderPipelinePbr>,
                Res<RenderPipelineSkybox>,
            ),
            (depth_buffer, msaa_buffer): (Res<DepthBuffer>, Res<MsaaBuffer>),
            query: Query<(
                &ecs::component::Mesh,
                &ecs::component::Material,
                &ecs::component::GlobalTransform,
            )>,
        ) {
            let output = gpu.surface.get_current_texture().unwrap();
            let output_texture = &output.texture;

            let output_texture_view =
                output_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render_encoder"),
                });

            let view_projection = camera.perspective() * camera.view_matrix();

            let instances = query.iter();
            let instances_len = instances.len();

            let mut instance_culling_information = Vec::with_capacity(instances_len);
            let mut instance_transforms = Vec::with_capacity(instances_len);
            let mut instance_materials = Vec::with_capacity(instances_len);
            let mut batches_map = std::collections::HashMap::new();
            let mut batches: Vec<(u32, usize)> = Vec::new();
            for (mesh, material, global_transform) in instances {
                instance_transforms.push(
                    glam::Mat4::from(glam::Affine3A::from_cols_array(global_transform))
                        .to_cols_array(),
                );

                instance_materials.push(material.material_id);

                let batch_id = if let Some(&batch_id) = batches_map.get(&mesh.mesh_id) {
                    batches[batch_id as usize].1 += 1;
                    batch_id
                } else {
                    let batch_id = batches_map.len() as u32;
                    batches_map.insert(mesh.mesh_id, batch_id);
                    batches.push((mesh.mesh_id, 1));
                    batch_id
                };

                instance_culling_information.push(
                    graphics::pipeline::compute::frustum_culling::InstanceCullingInformation {
                        batch_id,
                    },
                );
            }

            let mut indirect_draw_commands = Vec::with_capacity(batches.len());
            let mut cumulative_count = 0;
            for (mesh_id, instance_count) in batches {
                let mesh = meshes[mesh_id as usize];
                indirect_draw_commands.push(wgpu::util::DrawIndexedIndirectArgs {
                    index_count: mesh.index_count,
                    instance_count: 0,
                    first_index: mesh.index_offset,
                    base_vertex: mesh.vertex_offset as i32,
                    first_instance: cumulative_count,
                });
                cumulative_count += instance_count as u32;
            }

            let indirect_draw_commands_buffer =
                gpu.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("indirect_draw_commands_buffer"),
                        contents: &indirect_draw_commands
                            .iter()
                            .flat_map(|cmd| cmd.as_bytes())
                            .copied()
                            .collect::<Vec<_>>(),
                        usage: wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::INDIRECT,
                    });

            let indirect_instances_buffer =
                graphics::pipeline::compute::frustum_culling::create_indirect_instances_buffer(
                    &gpu.device,
                    &vec![0; instances_len],
                );

            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("compute_pass"),
                    timestamp_writes: None,
                });

                let instance_culling_information_buffer =
                    graphics::pipeline::compute::frustum_culling::create_instance_culling_information_buffer(
                        &gpu.device,
                        &instance_culling_information
                    );

                let frustum = ecs::resource::Frustum::from_view_projection_matrix(&view_projection);
                let frustum_buffer =
                    graphics::pipeline::compute::frustum_culling::create_frustum_buffer(
                        &gpu.device,
                        frustum,
                    );

                let instance_count_buffer =
                    graphics::pipeline::compute::frustum_culling::create_instance_count_buffer(
                        &gpu.device,
                        instances_len as u32,
                    );

                let bind_group_frustum_culling = compute_pipeline_frustum_culling
                    .create_bind_group_frustum_culling(
                        &gpu.device,
                        bounding_boxes_buffer.as_entire_binding(),
                        instance_culling_information_buffer.as_entire_binding(),
                        indirect_draw_commands_buffer.as_entire_binding(),
                        indirect_instances_buffer.as_entire_binding(),
                        frustum_buffer.as_entire_binding(),
                        instance_count_buffer.as_entire_binding(),
                    );

                compute_pipeline_frustum_culling
                    .prepare(&mut compute_pass, &bind_group_frustum_culling);

                compute_pass.dispatch_workgroups(instances_len.div_ceil(64) as u32, 1, 1);
            }

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("render_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &msaa_buffer,
                        resolve_target: Some(&output_texture_view),
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_buffer,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                let camera_buffer = graphics::pipeline::render::pbr::create_camera_buffer(
                    &gpu.device,
                    graphics::pipeline::render::pbr::CameraMatrix {
                        position: camera.position.extend(1.0).into(),
                        view_projection: view_projection.to_cols_array(),
                    },
                );

                let instance_transforms_buffer =
                    graphics::pipeline::render::pbr::create_instance_transforms_buffer(
                        &gpu.device,
                        instance_transforms,
                    );

                let instance_materials_buffer =
                    graphics::pipeline::render::pbr::create_instance_materials_buffer(
                        &gpu.device,
                        instance_materials,
                    );

                // TODO: query dynamically from world.
                let ambient_light_buffer =
                    graphics::pipeline::render::pbr::create_ambient_light_buffer(
                        &gpu.device,
                        graphics::pipeline::render::pbr::AmbientLight {
                            color: [1.0, 1.0, 1.0],
                            strength: 0.7,
                        },
                    );

                // TODO: query dynamically from world.
                let point_lights = [graphics::pipeline::render::pbr::PointLight {
                    color: [1.0, 1.0, 1.0],
                    strength: 0.7,
                    position: [0.0, 3.0, 2.0],
                    range: 4.0,
                }];
                let point_lights_buffer =
                    graphics::pipeline::render::pbr::create_point_lights_buffer(
                        &gpu.device,
                        &point_lights,
                    );
                let point_lights_length_buffer =
                    graphics::pipeline::render::pbr::create_point_lights_length_buffer(
                        &gpu.device,
                        point_lights.len() as u32,
                    );

                let bind_group_variable = render_pipeline_pbr.create_bind_group_variable(
                    &gpu.device,
                    camera_buffer.as_entire_binding(),
                    instance_transforms_buffer.as_entire_binding(),
                    indirect_instances_buffer.as_entire_binding(),
                    instance_materials_buffer.as_entire_binding(),
                );

                let bind_group_lights = render_pipeline_pbr.create_bind_group_lights(
                    &gpu.device,
                    ambient_light_buffer.as_entire_binding(),
                    point_lights_buffer.as_entire_binding(),
                    point_lights_length_buffer.as_entire_binding(),
                );

                render_pipeline_pbr.prepare(
                    &mut render_pass,
                    vertex_buffer.slice(..),
                    index_buffer.slice(..),
                    &bind_group_variable,
                    &bind_group_bindless,
                    &bind_group_lights,
                );

                render_pass.multi_draw_indexed_indirect(
                    &indirect_draw_commands_buffer,
                    0,
                    indirect_draw_commands.len() as u32,
                );

                let inverse_view_projection_buffer =
                    graphics::pipeline::render::skybox::create_inverse_view_projection_buffer(
                        &gpu.device,
                        &view_projection.inverse().to_cols_array(),
                    );

                let bind_group_inverse_view_projection = render_pipeline_skybox
                    .create_bind_group_inverse_view_projection(
                        &gpu.device,
                        inverse_view_projection_buffer.as_entire_binding(),
                    );

                render_pipeline_skybox.prepare(
                    &mut render_pass,
                    &bind_group_inverse_view_projection,
                    &bind_group_skybox,
                );

                graphics::pipeline::render::Skybox::draw(&mut render_pass);
            }

            gpu.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }
    }
}
