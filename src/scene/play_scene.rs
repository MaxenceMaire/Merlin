use super::Scene;
use crate::asset;
use crate::ecs;
use crate::graphics;
use bevy_hierarchy::BuildChildren;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub struct PlayScene {
    world: bevy_ecs::world::World,
    meshes: Vec<graphics::Mesh>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    bounding_boxes_buffer: wgpu::Buffer,
    bind_group_layout_frustum_culling: wgpu::BindGroupLayout,
    bind_group_layout_variable: wgpu::BindGroupLayout,
    bind_group_layout_inverse_view_projection: wgpu::BindGroupLayout,
    bind_group_bindless: wgpu::BindGroup,
    bind_group_skybox: wgpu::BindGroup,
    skybox_vertex_buffer: wgpu::Buffer,
    compute_pipeline_frustum_culling: wgpu::ComputePipeline,
    render_pipeline_main: wgpu::RenderPipeline,
    render_pipeline_skybox: wgpu::RenderPipeline,
    depth_buffer: wgpu::Texture,
    depth_buffer_view: wgpu::TextureView,
    instances_query_state: bevy_ecs::query::QueryState<(
        &'static ecs::component::Mesh,
        &'static ecs::component::Material,
        &'static ecs::component::GlobalTransform,
    )>,
}

impl Scene for PlayScene {
    fn update(&mut self) {
        // TODO: implement.
    }

    fn render(&mut self, gpu: &graphics::Gpu) {
        let output = gpu.surface.get_current_texture().unwrap();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        let camera = self.world.get_resource::<ecs::resource::Camera>().unwrap();

        let view_projection = camera.perspective() * camera.view_matrix();

        let instances = self.instances_query_state.iter(&self.world);
        let instances_len = instances.len();

        let mut instance_culling_information = Vec::with_capacity(instances_len);
        let mut instance_transforms = Vec::with_capacity(instances_len);
        let mut instance_materials = Vec::with_capacity(instances_len);
        let mut batches_map = HashMap::new();
        let mut batches: Vec<(u32, usize)> = Vec::new();
        for (mesh, material, global_transform) in instances {
            instance_transforms.push(
                glam::Mat4::from(glam::Affine3A::from_cols_array(global_transform)).to_cols_array(),
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

            instance_culling_information.push(InstanceCullingInformation { batch_id });
        }

        let instance_culling_information_buffer =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("instance_culling_information_buffer"),
                    contents: bytemuck::cast_slice(&instance_culling_information),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                });

        let mut indirect_draw_commands = Vec::with_capacity(batches.len());
        let mut cumulative_count = 0;
        for (mesh_id, instance_count) in batches {
            let mesh = self.meshes[mesh_id as usize];
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

        let indirect_instances = vec![0_u32; instances_len];
        let indirect_instances_buffer =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("indirect_instances_buffer"),
                    contents: bytemuck::cast_slice(&indirect_instances),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute_pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.compute_pipeline_frustum_culling);

            let frustum = ecs::resource::Frustum::from_view_projection_matrix(&view_projection);
            let frustum_buffer = gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("frustum_buffer"),
                    contents: bytemuck::cast_slice(&[frustum]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

            let instance_count_buffer =
                gpu.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("instance_count_buffer"),
                        contents: bytemuck::cast_slice(&[instances_len as u32]),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });

            let bind_group_frustum_culling =
                gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("bind_group_frustum_culling"),
                    layout: &self.bind_group_layout_frustum_culling,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.bounding_boxes_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: instance_culling_information_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: indirect_draw_commands_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: indirect_instances_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: frustum_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: instance_count_buffer.as_entire_binding(),
                        },
                    ],
                });
            compute_pass.set_bind_group(0, &bind_group_frustum_culling, &[]);

            compute_pass.dispatch_workgroups(instances_len.div_ceil(64) as u32, 1, 1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_buffer_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline_main);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            let camera_buffer = gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("camera_buffer"),
                    contents: bytemuck::cast_slice(&[CameraMatrix {
                        position: camera.position.extend(1.0).into(),
                        view_projection: view_projection.to_cols_array(),
                    }]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

            let instance_transforms_buffer =
                gpu.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("instance_transforms_buffer"),
                        contents: bytemuck::cast_slice(&instance_transforms),
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    });

            let instance_materials_buffer =
                gpu.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("instance_materials_buffer"),
                        contents: bytemuck::cast_slice(&instance_materials),
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    });

            let bind_group_variable = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bind_group_variable"),
                layout: &self.bind_group_layout_variable,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: instance_transforms_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: indirect_instances_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: instance_materials_buffer.as_entire_binding(),
                    },
                ],
            });
            render_pass.set_bind_group(0, &bind_group_variable, &[]);

            render_pass.set_bind_group(1, &self.bind_group_bindless, &[]);

            render_pass.multi_draw_indexed_indirect(
                &indirect_draw_commands_buffer,
                0,
                indirect_draw_commands.len() as u32,
            );

            render_pass.set_pipeline(&self.render_pipeline_skybox);

            render_pass.set_vertex_buffer(0, self.skybox_vertex_buffer.slice(..));

            let inverse_view_projection_buffer =
                gpu.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("inverse_view_projection_buffer"),
                        contents: bytemuck::cast_slice(&view_projection.inverse().to_cols_array()),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });

            let bind_group_inverse_view_projection =
                gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("bind_group_inverse_view_projection"),
                    layout: &self.bind_group_layout_inverse_view_projection,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: inverse_view_projection_buffer.as_entire_binding(),
                    }],
                });
            render_pass.set_bind_group(0, &bind_group_inverse_view_projection, &[]);

            render_pass.set_bind_group(1, &self.bind_group_skybox, &[]);

            render_pass.draw(0..6, 0..1);
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl PlayScene {
    pub fn setup(gpu: &graphics::Gpu) -> Self {
        let mut world = bevy_ecs::world::World::new();

        // TODO: aspect ratio.
        world.insert_resource(ecs::resource::Camera::default());

        let mut asset_loader = asset::AssetLoader::new();

        let model_id = asset_loader
            .load_gltf_model("assets/flight_helmet.gltf")
            .unwrap();

        let cubemap = asset_loader
            .load_cubemap(
                "assets/cubemap/px.ktx2",
                "assets/cubemap/nx.ktx2",
                "assets/cubemap/py.ktx2",
                "assets/cubemap/ny.ktx2",
                "assets/cubemap/pz.ktx2",
                "assets/cubemap/nz.ktx2",
            )
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
            bounding_boxes,
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

        world.flush();

        let vertex_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertex_buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            });

        let index_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("index_buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::STORAGE,
            });

        let bounding_boxes_buffer =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("bounding_boxes_buffer"),
                    contents: bytemuck::cast_slice(&bounding_boxes),
                    usage: wgpu::BufferUsages::STORAGE,
                });

        let bind_group_layout_variable =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("bind_group_layout_variable"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let material_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("material_buffer"),
                contents: bytemuck::cast_slice(&materials),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let create_texture = |label: Option<&str>, texture_map: &asset::TextureMap| {
            let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label,
                size: wgpu::Extent3d {
                    width: texture_map.dimension,
                    height: texture_map.dimension,
                    depth_or_array_layers: (texture_map.map.len() as u32).max(1),
                },
                mip_level_count: texture_map.mip_level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: texture_map.format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            // Holds true for BC5 and BC7.
            const BYTES_PER_BLOCK: u32 = 16;
            const BLOCK_SIZE: u32 = 4;

            for layer_index in 0..texture_map.count() {
                for mip_level_index in 0..texture_map.mip_level_count {
                    let mip_level_dimension =
                        (texture_map.dimension >> mip_level_index).max(BLOCK_SIZE);
                    let (mip_offset, mip_len) = texture_map.mip_levels[layer_index
                        * texture_map.mip_level_count as usize
                        + mip_level_index as usize];
                    let mip_level = &texture_map.data[mip_offset..(mip_offset + mip_len)];

                    gpu.queue.write_texture(
                        wgpu::ImageCopyTexture {
                            texture: &texture,
                            mip_level: mip_level_index,
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: 0,
                                z: layer_index as u32,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        mip_level,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(BYTES_PER_BLOCK * mip_level_dimension / BLOCK_SIZE),
                            rows_per_image: Some(mip_level_dimension / BLOCK_SIZE),
                        },
                        wgpu::Extent3d {
                            width: mip_level_dimension,
                            height: mip_level_dimension,
                            depth_or_array_layers: 1,
                        },
                    );
                }
            }

            texture
        };

        let texture_array_rg_bc5_unorm_512 = create_texture(
            Some("2d_texture_array_rg_bc5_unorm_512"),
            &texture_arrays.rg_bc5_unorm_512,
        );

        let texture_array_rg_bc5_unorm_1024 = create_texture(
            Some("2d_texture_array_rg_bc5_unorm_1024"),
            &texture_arrays.rg_bc5_unorm_1024,
        );

        let texture_array_rg_bc5_unorm_2048 = create_texture(
            Some("2d_texture_array_rg_bc5_unorm_2048"),
            &texture_arrays.rg_bc5_unorm_2048,
        );

        let texture_array_rg_bc5_unorm_4096 = create_texture(
            Some("2d_texture_array_rg_bc5_unorm_4096"),
            &texture_arrays.rg_bc5_unorm_4096,
        );

        let texture_array_rgb_bc7_unorm_512 = create_texture(
            Some("2d_texture_array_rgb_bc7_unorm_512"),
            &texture_arrays.rgb_bc7_unorm_512,
        );

        let texture_array_rgb_bc7_unorm_1024 = create_texture(
            Some("2d_texture_array_rgb_bc7_unorm_1024"),
            &texture_arrays.rgb_bc7_unorm_1024,
        );

        let texture_array_rgb_bc7_unorm_2048 = create_texture(
            Some("2d_texture_array_rgb_bc7_unorm_2048"),
            &texture_arrays.rgb_bc7_unorm_2048,
        );

        let texture_array_rgb_bc7_unorm_4096 = create_texture(
            Some("2d_texture_array_rgb_bc7_unorm_4096"),
            &texture_arrays.rgb_bc7_unorm_4096,
        );

        let texture_array_rgba_bc7_srgb_512 = create_texture(
            Some("2d_texture_array_rgba_bc7_srgb_512"),
            &texture_arrays.rgba_bc7_srgb_512,
        );

        let texture_array_rgba_bc7_srgb_1024 = create_texture(
            Some("2d_texture_array_rgba_bc7_srgb_1024"),
            &texture_arrays.rgba_bc7_srgb_1024,
        );

        let texture_array_rgba_bc7_srgb_2048 = create_texture(
            Some("2d_texture_array_rgba_bc7_srgb_2048"),
            &texture_arrays.rgba_bc7_srgb_2048,
        );

        let texture_array_rgba_bc7_srgb_4096 = create_texture(
            Some("2d_texture_array_rgba_bc7_srgb_4096"),
            &texture_arrays.rgba_bc7_srgb_4096,
        );

        gpu.queue.submit([]);

        let bind_group_layout_bindless =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("bind_group_layout_bindless"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 6,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 7,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 8,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 9,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 10,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 11,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 12,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 13,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 14,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let create_texture_view =
            |label: Option<&str>, texture: &wgpu::Texture, format: wgpu::TextureFormat| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    label,
                    format: Some(format),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                })
            };

        let bind_group_bindless = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group_bindless"),
            layout: &bind_group_layout_bindless,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: material_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_rg_bc5_unorm_512"),
                        &texture_array_rg_bc5_unorm_512,
                        wgpu::TextureFormat::Bc5RgUnorm,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_rg_bc5_unorm_1024"),
                        &texture_array_rg_bc5_unorm_1024,
                        wgpu::TextureFormat::Bc5RgUnorm,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_rg_bc5_unorm_2048"),
                        &texture_array_rg_bc5_unorm_2048,
                        wgpu::TextureFormat::Bc5RgUnorm,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_rg_bc5_unorm_4096"),
                        &texture_array_rg_bc5_unorm_4096,
                        wgpu::TextureFormat::Bc5RgUnorm,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_rgb_bc7_unorm_512"),
                        &texture_array_rgb_bc7_unorm_512,
                        wgpu::TextureFormat::Bc7RgbaUnorm,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_rgb_bc7_unorm_1024"),
                        &texture_array_rgb_bc7_unorm_1024,
                        wgpu::TextureFormat::Bc7RgbaUnorm,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_rgb_bc7_unorm_2048"),
                        &texture_array_rgb_bc7_unorm_2048,
                        wgpu::TextureFormat::Bc7RgbaUnorm,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_rgb_bc7_unorm_4096"),
                        &texture_array_rgb_bc7_unorm_4096,
                        wgpu::TextureFormat::Bc7RgbaUnorm,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_rgba_bc7_srgb_512"),
                        &texture_array_rgba_bc7_srgb_512,
                        wgpu::TextureFormat::Bc7RgbaUnormSrgb,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_rgba_bc7_srgb_1024"),
                        &texture_array_rgba_bc7_srgb_1024,
                        wgpu::TextureFormat::Bc7RgbaUnormSrgb,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_rgba_bc7_srgb_2048"),
                        &texture_array_rgba_bc7_srgb_2048,
                        wgpu::TextureFormat::Bc7RgbaUnormSrgb,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_rgba_bc7_srgb_4096"),
                        &texture_array_rgba_bc7_srgb_4096,
                        wgpu::TextureFormat::Bc7RgbaUnormSrgb,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 13,
                    resource: wgpu::BindingResource::Sampler(&gpu.device.create_sampler(
                        &wgpu::SamplerDescriptor {
                            label: Some("texture_array_sampler_base_color"),
                            ..Default::default()
                        },
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 14,
                    resource: wgpu::BindingResource::Sampler(&gpu.device.create_sampler(
                        &wgpu::SamplerDescriptor {
                            label: Some("texture_array_sampler_normal"),
                            ..Default::default()
                        },
                    )),
                },
            ],
        });

        let shader_frustum_culling =
            gpu.device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("shader_frustum_culling"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("frustum_culling.wgsl").into()),
                });

        let bind_group_layout_frustum_culling =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("bind_group_layout_frustum_culling"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let pipeline_layout_frustum_culling =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("pipeline_layout_frustum_culling"),
                    bind_group_layouts: &[&bind_group_layout_frustum_culling],
                    push_constant_ranges: &[],
                });

        let compute_pipeline_frustum_culling =
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("compute_pipeline_descriptor_frustum_culling"),
                    layout: Some(&pipeline_layout_frustum_culling),
                    module: &shader_frustum_culling,
                    entry_point: Some("cs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                });

        let shader_main = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shader_main"),
                source: wgpu::ShaderSource::Wgsl(include_str!("render.wgsl").into()),
            });

        let render_pipeline_layout_main =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("render_pipeline_layout_main"),
                    bind_group_layouts: &[&bind_group_layout_variable, &bind_group_layout_bindless],
                    push_constant_ranges: &[],
                });

        let render_pipeline_main =
            gpu.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("render_pipeline_main"),
                    layout: Some(&render_pipeline_layout_main),
                    vertex: wgpu::VertexState {
                        module: &shader_main,
                        entry_point: Some("vs_main"),
                        buffers: &[graphics::Vertex::buffer_layout()],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_main,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: gpu.config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        let depth_buffer = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_buffer"),
            size: wgpu::Extent3d {
                width: gpu.config.width,
                height: gpu.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_buffer_view = depth_buffer.create_view(&wgpu::TextureViewDescriptor::default());

        let instances_query_state = world.query::<(
            &ecs::component::Mesh,
            &ecs::component::Material,
            &ecs::component::GlobalTransform,
        )>();

        let vertex_positions: [[f32; 2]; 6] = [
            [-1.0, 1.0],  // Top-left
            [-1.0, -1.0], // Bottom-left
            [1.0, 1.0],   // Top-right
            [-1.0, -1.0], // Bottom-left
            [1.0, -1.0],  // Bottom-right
            [1.0, 1.0],   // Top-right
        ];
        let skybox_vertex_buffer =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("skybox_vertex_buffer"),
                    contents: bytemuck::cast_slice(&vertex_positions),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
                });

        let bind_group_layout_inverse_view_projection =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("bind_group_layout_inverse_view_projection"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let bind_group_layout_skybox =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("bind_group_layout_skybox"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::Cube,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

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

        let bind_group_skybox = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group_skybox"),
            layout: &bind_group_layout_skybox,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_skybox.create_view(
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
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&gpu.device.create_sampler(
                        &wgpu::SamplerDescriptor {
                            label: Some("texture_sampler_skybox"),
                            mag_filter: wgpu::FilterMode::Linear,
                            min_filter: wgpu::FilterMode::Linear,
                            ..Default::default()
                        },
                    )),
                },
            ],
        });

        let shader_skybox = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shader_skybox"),
                source: wgpu::ShaderSource::Wgsl(include_str!("skybox.wgsl").into()),
            });

        let render_pipeline_layout_skybox =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("render_pipeline_layout_skybox"),
                    bind_group_layouts: &[
                        &bind_group_layout_inverse_view_projection,
                        &bind_group_layout_skybox,
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline_skybox =
            gpu.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("render_pipeline_skybox"),
                    layout: Some(&render_pipeline_layout_skybox),
                    vertex: wgpu::VertexState {
                        module: &shader_skybox,
                        entry_point: Some("vs_main"),
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x2,
                            }],
                        }],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_skybox,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: gpu.config.format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        Self {
            world,
            meshes,
            vertex_buffer,
            index_buffer,
            bounding_boxes_buffer,
            bind_group_bindless,
            bind_group_skybox,
            bind_group_layout_frustum_culling,
            bind_group_layout_variable,
            bind_group_layout_inverse_view_projection,
            skybox_vertex_buffer,
            compute_pipeline_frustum_culling,
            render_pipeline_main,
            render_pipeline_skybox,
            depth_buffer,
            depth_buffer_view,
            instances_query_state,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceCullingInformation {
    batch_id: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraMatrix {
    position: [f32; 4],
    view_projection: [f32; 16],
}
