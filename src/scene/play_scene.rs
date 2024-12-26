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
    bind_group_bindless: wgpu::BindGroup,
    bind_group_skybox: wgpu::BindGroup,
    compute_pipeline_frustum_culling: graphics::pipeline::compute::FrustumCulling,
    render_pipeline_pbr: graphics::pipeline::render::Pbr,
    render_pipeline_skybox: graphics::pipeline::render::Skybox,
    depth_buffer_view: wgpu::TextureView,
    msaa_buffer_view: wgpu::TextureView,
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
        let output_texture = &output.texture;

        let output_texture_view =
            output_texture.create_view(&wgpu::TextureViewDescriptor::default());

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

            instance_culling_information.push(
                graphics::pipeline::compute::frustum_culling::InstanceCullingInformation {
                    batch_id,
                },
            );
        }

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

            let bind_group_frustum_culling = self
                .compute_pipeline_frustum_culling
                .create_bind_group_frustum_culling(
                    &gpu.device,
                    self.bounding_boxes_buffer.as_entire_binding(),
                    instance_culling_information_buffer.as_entire_binding(),
                    indirect_draw_commands_buffer.as_entire_binding(),
                    indirect_instances_buffer.as_entire_binding(),
                    frustum_buffer.as_entire_binding(),
                    instance_count_buffer.as_entire_binding(),
                );

            self.compute_pipeline_frustum_culling
                .prepare(&mut compute_pass, &bind_group_frustum_culling);

            compute_pass.dispatch_workgroups(instances_len.div_ceil(64) as u32, 1, 1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.msaa_buffer_view,
                    resolve_target: Some(&output_texture_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
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
            let ambient_light_buffer = graphics::pipeline::render::pbr::create_ambient_light_buffer(
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
            let point_lights_buffer = graphics::pipeline::render::pbr::create_point_lights_buffer(
                &gpu.device,
                &point_lights,
            );
            let point_lights_length_buffer =
                graphics::pipeline::render::pbr::create_point_lights_length_buffer(
                    &gpu.device,
                    point_lights.len() as u32,
                );

            let bind_group_variable = self.render_pipeline_pbr.create_bind_group_variable(
                &gpu.device,
                camera_buffer.as_entire_binding(),
                instance_transforms_buffer.as_entire_binding(),
                indirect_instances_buffer.as_entire_binding(),
                instance_materials_buffer.as_entire_binding(),
            );

            let bind_group_lights = self.render_pipeline_pbr.create_bind_group_lights(
                &gpu.device,
                ambient_light_buffer.as_entire_binding(),
                point_lights_buffer.as_entire_binding(),
                point_lights_length_buffer.as_entire_binding(),
            );

            self.render_pipeline_pbr.prepare(
                &mut render_pass,
                self.vertex_buffer.slice(..),
                self.index_buffer.slice(..),
                &bind_group_variable,
                &self.bind_group_bindless,
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

            let bind_group_inverse_view_projection = self
                .render_pipeline_skybox
                .create_bind_group_inverse_view_projection(
                    &gpu.device,
                    inverse_view_projection_buffer.as_entire_binding(),
                );

            self.render_pipeline_skybox.prepare(
                &mut render_pass,
                &bind_group_inverse_view_projection,
                &self.bind_group_skybox,
            );

            graphics::pipeline::render::Skybox::draw(&mut render_pass);
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

        world.flush();

        let bounding_boxes_buffer =
            graphics::pipeline::render::skybox::create_bounding_boxes_buffer(
                &gpu.device,
                &bounding_boxes,
            );

        let compute_pipeline_frustum_culling =
            graphics::pipeline::compute::FrustumCulling::new(&gpu.device);

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

        let material_buffer =
            graphics::pipeline::render::pbr::create_material_buffer(&gpu.device, &materials);

        let create_texture_array = |label: Option<&str>, texture_map: &asset::TextureMap| {
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

        let texture_array_rg_bc5_unorm_512 = create_texture_array(
            Some("2d_texture_array_rg_bc5_unorm_512"),
            &texture_arrays.rg_bc5_unorm_512,
        );

        let texture_array_rg_bc5_unorm_1024 = create_texture_array(
            Some("2d_texture_array_rg_bc5_unorm_1024"),
            &texture_arrays.rg_bc5_unorm_1024,
        );

        let texture_array_rg_bc5_unorm_2048 = create_texture_array(
            Some("2d_texture_array_rg_bc5_unorm_2048"),
            &texture_arrays.rg_bc5_unorm_2048,
        );

        let texture_array_rg_bc5_unorm_4096 = create_texture_array(
            Some("2d_texture_array_rg_bc5_unorm_4096"),
            &texture_arrays.rg_bc5_unorm_4096,
        );

        let texture_array_rgb_bc7_unorm_512 = create_texture_array(
            Some("2d_texture_array_rgb_bc7_unorm_512"),
            &texture_arrays.rgb_bc7_unorm_512,
        );

        let texture_array_rgb_bc7_unorm_1024 = create_texture_array(
            Some("2d_texture_array_rgb_bc7_unorm_1024"),
            &texture_arrays.rgb_bc7_unorm_1024,
        );

        let texture_array_rgb_bc7_unorm_2048 = create_texture_array(
            Some("2d_texture_array_rgb_bc7_unorm_2048"),
            &texture_arrays.rgb_bc7_unorm_2048,
        );

        let texture_array_rgb_bc7_unorm_4096 = create_texture_array(
            Some("2d_texture_array_rgb_bc7_unorm_4096"),
            &texture_arrays.rgb_bc7_unorm_4096,
        );

        let texture_array_rgba_bc7_srgb_512 = create_texture_array(
            Some("2d_texture_array_rgba_bc7_srgb_512"),
            &texture_arrays.rgba_bc7_srgb_512,
        );

        let texture_array_rgba_bc7_srgb_1024 = create_texture_array(
            Some("2d_texture_array_rgba_bc7_srgb_1024"),
            &texture_arrays.rgba_bc7_srgb_1024,
        );

        let texture_array_rgba_bc7_srgb_2048 = create_texture_array(
            Some("2d_texture_array_rgba_bc7_srgb_2048"),
            &texture_arrays.rgba_bc7_srgb_2048,
        );

        let texture_array_rgba_bc7_srgb_4096 = create_texture_array(
            Some("2d_texture_array_rgba_bc7_srgb_4096"),
            &texture_arrays.rgba_bc7_srgb_4096,
        );

        gpu.queue.submit([]);

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

        const MSAA_SAMPLE_COUNT: u32 = 4;

        let render_pipeline_pbr =
            graphics::pipeline::render::Pbr::new(&gpu.device, gpu.config.format, MSAA_SAMPLE_COUNT);

        let bind_group_bindless = render_pipeline_pbr.create_bind_group_bindless(
            &gpu.device,
            material_buffer.as_entire_binding(),
            wgpu::BindingResource::TextureView(&create_texture_view(
                Some("texture_array_rg_bc5_unorm_512"),
                &texture_array_rg_bc5_unorm_512,
                wgpu::TextureFormat::Bc5RgUnorm,
            )),
            wgpu::BindingResource::TextureView(&create_texture_view(
                Some("texture_array_rg_bc5_unorm_1024"),
                &texture_array_rg_bc5_unorm_1024,
                wgpu::TextureFormat::Bc5RgUnorm,
            )),
            wgpu::BindingResource::TextureView(&create_texture_view(
                Some("texture_array_rg_bc5_unorm_2048"),
                &texture_array_rg_bc5_unorm_2048,
                wgpu::TextureFormat::Bc5RgUnorm,
            )),
            wgpu::BindingResource::TextureView(&create_texture_view(
                Some("texture_array_rg_bc5_unorm_4096"),
                &texture_array_rg_bc5_unorm_4096,
                wgpu::TextureFormat::Bc5RgUnorm,
            )),
            wgpu::BindingResource::TextureView(&create_texture_view(
                Some("texture_array_rgb_bc7_unorm_512"),
                &texture_array_rgb_bc7_unorm_512,
                wgpu::TextureFormat::Bc7RgbaUnorm,
            )),
            wgpu::BindingResource::TextureView(&create_texture_view(
                Some("texture_array_rgb_bc7_unorm_1024"),
                &texture_array_rgb_bc7_unorm_1024,
                wgpu::TextureFormat::Bc7RgbaUnorm,
            )),
            wgpu::BindingResource::TextureView(&create_texture_view(
                Some("texture_array_rgb_bc7_unorm_2048"),
                &texture_array_rgb_bc7_unorm_2048,
                wgpu::TextureFormat::Bc7RgbaUnorm,
            )),
            wgpu::BindingResource::TextureView(&create_texture_view(
                Some("texture_array_rgb_bc7_unorm_4096"),
                &texture_array_rgb_bc7_unorm_4096,
                wgpu::TextureFormat::Bc7RgbaUnorm,
            )),
            wgpu::BindingResource::TextureView(&create_texture_view(
                Some("texture_array_rgba_bc7_srgb_512"),
                &texture_array_rgba_bc7_srgb_512,
                wgpu::TextureFormat::Bc7RgbaUnormSrgb,
            )),
            wgpu::BindingResource::TextureView(&create_texture_view(
                Some("texture_array_rgba_bc7_srgb_1024"),
                &texture_array_rgba_bc7_srgb_1024,
                wgpu::TextureFormat::Bc7RgbaUnormSrgb,
            )),
            wgpu::BindingResource::TextureView(&create_texture_view(
                Some("texture_array_rgba_bc7_srgb_2048"),
                &texture_array_rgba_bc7_srgb_2048,
                wgpu::TextureFormat::Bc7RgbaUnormSrgb,
            )),
            wgpu::BindingResource::TextureView(&create_texture_view(
                Some("texture_array_rgba_bc7_srgb_4096"),
                &texture_array_rgba_bc7_srgb_4096,
                wgpu::TextureFormat::Bc7RgbaUnormSrgb,
            )),
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

        let depth_buffer = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_buffer"),
            size: wgpu::Extent3d {
                width: gpu.config.width,
                height: gpu.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: MSAA_SAMPLE_COUNT,
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

        let render_pipeline_skybox = graphics::pipeline::render::Skybox::new(
            &gpu.device,
            gpu.config.format,
            MSAA_SAMPLE_COUNT,
        );

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

        let msaa_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("msaa_texture"),
            size: wgpu::Extent3d {
                width: gpu.config.width,
                height: gpu.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: MSAA_SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: gpu.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let msaa_buffer_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            world,
            meshes,
            vertex_buffer,
            index_buffer,
            bounding_boxes_buffer,
            bind_group_bindless,
            bind_group_skybox,
            compute_pipeline_frustum_culling,
            render_pipeline_pbr,
            render_pipeline_skybox,
            depth_buffer_view,
            msaa_buffer_view,
            instances_query_state,
        }
    }
}
