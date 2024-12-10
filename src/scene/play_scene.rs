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
    compute_pipeline_frustum_culling: wgpu::ComputePipeline,
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

        let primitives_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("primitives_bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let primitives_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("primitives_bind_group"),
            layout: &primitives_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: vertex_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: index_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: mesh_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: material_buffer.as_entire_binding(),
                },
            ],
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

        let texture_arrays_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("texture_arrays_bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                multisampled: false,
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
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                    ],
                });

        let create_texture_view =
            |label: Option<&str>, texture: &wgpu::Texture, channel: wgpu::AstcChannel| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    label,
                    format: Some(wgpu::TextureFormat::Astc {
                        block: wgpu::AstcBlock::B4x4,
                        channel,
                    }),
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                })
            };

        let texture_arrays_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_arrays_bind_group"),
            layout: &texture_arrays_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_unorm_srgb_512_view"),
                        &texture_array_unorm_srgb_512,
                        wgpu::AstcChannel::UnormSrgb,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_unorm_srgb_1024_view"),
                        &texture_array_unorm_srgb_1024,
                        wgpu::AstcChannel::UnormSrgb,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_unorm_srgb_2048_view"),
                        &texture_array_unorm_srgb_2048,
                        wgpu::AstcChannel::UnormSrgb,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_unorm_srgb_4096_view"),
                        &texture_array_unorm_srgb_4096,
                        wgpu::AstcChannel::UnormSrgb,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_unorm_512_view"),
                        &texture_array_unorm_512,
                        wgpu::AstcChannel::Unorm,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_unorm_1024_view"),
                        &texture_array_unorm_1024,
                        wgpu::AstcChannel::Unorm,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_unorm_2048_view"),
                        &texture_array_unorm_2048,
                        wgpu::AstcChannel::Unorm,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_unorm_4096_view"),
                        &texture_array_unorm_4096,
                        wgpu::AstcChannel::Unorm,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_hdr_512_view"),
                        &texture_array_hdr_512,
                        wgpu::AstcChannel::Hdr,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_hdr_1024_view"),
                        &texture_array_hdr_1024,
                        wgpu::AstcChannel::Hdr,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_hdr_2048_view"),
                        &texture_array_hdr_2048,
                        wgpu::AstcChannel::Hdr,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: wgpu::BindingResource::TextureView(&create_texture_view(
                        Some("texture_array_hdr_4096_view"),
                        &texture_array_hdr_4096,
                        wgpu::AstcChannel::Hdr,
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: wgpu::BindingResource::Sampler(&gpu.device.create_sampler(
                        &wgpu::SamplerDescriptor {
                            label: Some("texture_array_sampler"),
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

        let frustum_culling_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("frustum_culling_bind_group_layout"),
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
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
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
                                ty: wgpu::BufferBindingType::Uniform,
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
                    ],
                });

        let pipeline_layout_frustum_culling =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("pipeline_layout_frustum_culling"),
                    bind_group_layouts: &[&frustum_culling_bind_group_layout],
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

        /*
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shader_main"),
                source: wgpu::ShaderSource::Wgsl(include_str!("render.wgsl").into()),
            });

        let main_render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("main_render_pipeline_layout"),
                    bind_group_layouts: &[
                        &primitives_bind_group_layout,
                        &texture_arrays_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let main_render_pipeline =
            gpu.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("main_render_pipeline"),
                    layout: Some(&main_render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[graphics::Vertex::buffer_layout()],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: gpu.config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
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
        */

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
            compute_pipeline_frustum_culling,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct FrustumCullingInformation {
    instance_count: u32,
}
