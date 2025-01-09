pub struct Renderer;

impl Renderer {
    pub fn spawn(
        renderer_to_scene_sender: crossbeam::channel::Sender<bevy_ecs::world::World>,
        scene_to_renderer_receiver: crossbeam::channel::Receiver<bevy_ecs::world::World>,
    ) -> std::thread::JoinHandle<()> {
        let mut render_schedule = schedule::rendering();

        std::thread::spawn(move || loop {
            let Ok(mut render_world) = scene_to_renderer_receiver.recv() else {
                // Channel disconnected.
                return;
            };

            render_schedule.run(&mut render_world);

            render_world.clear_entities();

            let send_result = renderer_to_scene_sender.send(render_world);
            if send_result.is_err() {
                // Channel disconnected.
                return;
            }
        })
    }
}

mod schedule {
    use super::system;

    pub fn rendering() -> bevy_ecs::schedule::Schedule {
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(system::render);

        schedule
    }
}

mod system {
    use super::super::resource::*;
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
        (bind_group_bindless, bind_group_skybox): (Res<BindGroupBindless>, Res<BindGroupSkybox>),
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
            instance_transforms.push(glam::Mat4::from(**global_transform).to_cols_array());

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
