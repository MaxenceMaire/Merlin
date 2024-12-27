use crate::asset;
use crate::graphics;
use wgpu::util::DeviceExt;

pub struct Pbr {
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout_variable: wgpu::BindGroupLayout,
    bind_group_layout_bindless: wgpu::BindGroupLayout,
    bind_group_layout_lights: wgpu::BindGroupLayout,
}

impl Pbr {
    pub fn new(
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
        msaa_sample_count: u32,
    ) -> Self {
        let bind_group_layout_variable =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("bind_group_layout_variable"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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

        let bind_group_layout_bindless =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let bind_group_layout_lights =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("bind_group_layout_lights"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader_pbr"),
            source: wgpu::ShaderSource::Wgsl(include_str!("pbr.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout_pbr"),
                bind_group_layouts: &[
                    &bind_group_layout_variable,
                    &bind_group_layout_bindless,
                    &bind_group_layout_lights,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline_pbr"),
            layout: Some(&render_pipeline_layout),
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
                    format: texture_format,
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
                count: msaa_sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
            bind_group_layout_variable,
            bind_group_layout_bindless,
            bind_group_layout_lights,
        }
    }

    pub fn prepare(
        &self,
        render_pass: &mut wgpu::RenderPass,
        vertex_buffer: wgpu::BufferSlice,
        index_buffer: wgpu::BufferSlice,
        bind_group_variable: &wgpu::BindGroup,
        bind_group_bindless: &wgpu::BindGroup,
        bind_group_lights: &wgpu::BindGroup,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer);
        render_pass.set_index_buffer(index_buffer, wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, bind_group_variable, &[]);
        render_pass.set_bind_group(1, bind_group_bindless, &[]);
        render_pass.set_bind_group(2, bind_group_lights, &[]);
    }

    pub fn create_bind_group_variable(
        &self,
        device: &wgpu::Device,
        binding_resource_camera_buffer: wgpu::BindingResource,
        binding_resource_instance_transforms_buffer: wgpu::BindingResource,
        binding_resource_indirect_instances_buffer: wgpu::BindingResource,
        binding_resource_instance_materials_buffer: wgpu::BindingResource,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group_variable"),
            layout: &self.bind_group_layout_variable,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: binding_resource_camera_buffer,
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: binding_resource_instance_transforms_buffer,
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: binding_resource_indirect_instances_buffer,
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: binding_resource_instance_materials_buffer,
                },
            ],
        })
    }

    pub fn create_bind_group_bindless(
        &self,
        device: &wgpu::Device,
        binding_resource_material_buffer: wgpu::BindingResource,
        texture_array_views: TextureArrays<wgpu::TextureView>,
        binding_resource_texture_array_sampler_base_color: wgpu::BindingResource,
        binding_resource_texture_array_sampler_normal: wgpu::BindingResource,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group_bindless"),
            layout: &self.bind_group_layout_bindless,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: binding_resource_material_buffer,
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &texture_array_views.rg_bc5_unorm_512,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &texture_array_views.rg_bc5_unorm_1024,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        &texture_array_views.rg_bc5_unorm_2048,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(
                        &texture_array_views.rg_bc5_unorm_4096,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(
                        &texture_array_views.rgb_bc7_unorm_512,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(
                        &texture_array_views.rgb_bc7_unorm_1024,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(
                        &texture_array_views.rgb_bc7_unorm_2048,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(
                        &texture_array_views.rgb_bc7_unorm_4096,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(
                        &texture_array_views.rgba_bc7_srgb_512,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: wgpu::BindingResource::TextureView(
                        &texture_array_views.rgba_bc7_srgb_1024,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: wgpu::BindingResource::TextureView(
                        &texture_array_views.rgba_bc7_srgb_2048,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: wgpu::BindingResource::TextureView(
                        &texture_array_views.rgba_bc7_srgb_4096,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 13,
                    resource: binding_resource_texture_array_sampler_base_color,
                },
                wgpu::BindGroupEntry {
                    binding: 14,
                    resource: binding_resource_texture_array_sampler_normal,
                },
            ],
        })
    }

    pub fn create_bind_group_lights(
        &self,
        device: &wgpu::Device,
        resource_binding_ambient_light_buffer: wgpu::BindingResource,
        resource_binding_point_lights_buffer: wgpu::BindingResource,
        resource_binding_point_lights_length_buffer: wgpu::BindingResource,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group_lights"),
            layout: &self.bind_group_layout_lights,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: resource_binding_ambient_light_buffer,
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: resource_binding_point_lights_buffer,
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: resource_binding_point_lights_length_buffer,
                },
            ],
        })
    }
}

pub fn create_camera_buffer(device: &wgpu::Device, camera_matrix: CameraMatrix) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("camera_buffer"),
        contents: bytemuck::cast_slice(&[camera_matrix]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

pub fn create_material_buffer(
    device: &wgpu::Device,
    materials: &[graphics::Material],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("material_buffer"),
        contents: bytemuck::cast_slice(materials),
        usage: wgpu::BufferUsages::STORAGE,
    })
}

pub fn create_instance_transforms_buffer(
    device: &wgpu::Device,
    instance_transforms: Vec<[f32; 16]>,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("instance_transforms_buffer"),
        contents: bytemuck::cast_slice(&instance_transforms),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

pub fn create_instance_materials_buffer(
    device: &wgpu::Device,
    instance_materials: Vec<u32>,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("instance_materials_buffer"),
        contents: bytemuck::cast_slice(&instance_materials),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

pub fn create_ambient_light_buffer(
    device: &wgpu::Device,
    ambient_light: AmbientLight,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ambient_light_buffer"),
        contents: bytemuck::cast_slice(&[ambient_light]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

pub fn create_point_lights_buffer(
    device: &wgpu::Device,
    point_lights: &[PointLight],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("point_lights_buffer"),
        contents: bytemuck::cast_slice(point_lights),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

pub fn create_point_lights_length_buffer(
    device: &wgpu::Device,
    point_lights_length: u32,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("point_lights_length_buffer"),
        contents: bytemuck::cast_slice(&[point_lights_length]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

pub fn create_texture_array(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: Option<&str>,
    texture_map: &asset::TextureMap,
) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
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
            let mip_level_dimension = (texture_map.dimension >> mip_level_index).max(BLOCK_SIZE);
            let (mip_offset, mip_len) = texture_map.mip_levels
                [layer_index * texture_map.mip_level_count as usize + mip_level_index as usize];
            let mip_level = &texture_map.data[mip_offset..(mip_offset + mip_len)];

            queue.write_texture(
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
}

pub struct TextureArrays<T> {
    pub rg_bc5_unorm_512: T,
    pub rg_bc5_unorm_1024: T,
    pub rg_bc5_unorm_2048: T,
    pub rg_bc5_unorm_4096: T,
    pub rgb_bc7_unorm_512: T,
    pub rgb_bc7_unorm_1024: T,
    pub rgb_bc7_unorm_2048: T,
    pub rgb_bc7_unorm_4096: T,
    pub rgba_bc7_srgb_512: T,
    pub rgba_bc7_srgb_1024: T,
    pub rgba_bc7_srgb_2048: T,
    pub rgba_bc7_srgb_4096: T,
}

pub fn create_texture_arrays_init(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_maps: &TextureArrays<asset::TextureMap>,
) -> TextureArrays<wgpu::Texture> {
    let texture_array_rg_bc5_unorm_512 = create_texture_array(
        device,
        queue,
        Some("2d_texture_array_rg_bc5_unorm_512"),
        &texture_maps.rg_bc5_unorm_512,
    );

    let texture_array_rg_bc5_unorm_1024 = create_texture_array(
        device,
        queue,
        Some("2d_texture_array_rg_bc5_unorm_1024"),
        &texture_maps.rg_bc5_unorm_1024,
    );

    let texture_array_rg_bc5_unorm_2048 = create_texture_array(
        device,
        queue,
        Some("2d_texture_array_rg_bc5_unorm_2048"),
        &texture_maps.rg_bc5_unorm_2048,
    );

    let texture_array_rg_bc5_unorm_4096 = create_texture_array(
        device,
        queue,
        Some("2d_texture_array_rg_bc5_unorm_4096"),
        &texture_maps.rg_bc5_unorm_4096,
    );

    let texture_array_rgb_bc7_unorm_512 = create_texture_array(
        device,
        queue,
        Some("2d_texture_array_rgb_bc7_unorm_512"),
        &texture_maps.rgb_bc7_unorm_512,
    );

    let texture_array_rgb_bc7_unorm_1024 = create_texture_array(
        device,
        queue,
        Some("2d_texture_array_rgb_bc7_unorm_1024"),
        &texture_maps.rgb_bc7_unorm_1024,
    );

    let texture_array_rgb_bc7_unorm_2048 = create_texture_array(
        device,
        queue,
        Some("2d_texture_array_rgb_bc7_unorm_2048"),
        &texture_maps.rgb_bc7_unorm_2048,
    );

    let texture_array_rgb_bc7_unorm_4096 = create_texture_array(
        device,
        queue,
        Some("2d_texture_array_rgb_bc7_unorm_4096"),
        &texture_maps.rgb_bc7_unorm_4096,
    );

    let texture_array_rgba_bc7_srgb_512 = create_texture_array(
        device,
        queue,
        Some("2d_texture_array_rgba_bc7_srgb_512"),
        &texture_maps.rgba_bc7_srgb_512,
    );

    let texture_array_rgba_bc7_srgb_1024 = create_texture_array(
        device,
        queue,
        Some("2d_texture_array_rgba_bc7_srgb_1024"),
        &texture_maps.rgba_bc7_srgb_1024,
    );

    let texture_array_rgba_bc7_srgb_2048 = create_texture_array(
        device,
        queue,
        Some("2d_texture_array_rgba_bc7_srgb_2048"),
        &texture_maps.rgba_bc7_srgb_2048,
    );

    let texture_array_rgba_bc7_srgb_4096 = create_texture_array(
        device,
        queue,
        Some("2d_texture_array_rgba_bc7_srgb_4096"),
        &texture_maps.rgba_bc7_srgb_4096,
    );

    queue.submit([]);

    TextureArrays {
        rg_bc5_unorm_512: texture_array_rg_bc5_unorm_512,
        rg_bc5_unorm_1024: texture_array_rg_bc5_unorm_1024,
        rg_bc5_unorm_2048: texture_array_rg_bc5_unorm_2048,
        rg_bc5_unorm_4096: texture_array_rg_bc5_unorm_4096,
        rgb_bc7_unorm_512: texture_array_rgb_bc7_unorm_512,
        rgb_bc7_unorm_1024: texture_array_rgb_bc7_unorm_1024,
        rgb_bc7_unorm_2048: texture_array_rgb_bc7_unorm_2048,
        rgb_bc7_unorm_4096: texture_array_rgb_bc7_unorm_4096,
        rgba_bc7_srgb_512: texture_array_rgba_bc7_srgb_512,
        rgba_bc7_srgb_1024: texture_array_rgba_bc7_srgb_1024,
        rgba_bc7_srgb_2048: texture_array_rgba_bc7_srgb_2048,
        rgba_bc7_srgb_4096: texture_array_rgba_bc7_srgb_4096,
    }
}

pub fn create_texture_array_view(
    label: Option<&str>,
    texture: &wgpu::Texture,
    format: wgpu::TextureFormat,
) -> wgpu::TextureView {
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
}

pub fn create_texture_array_views(
    texture_arrays: TextureArrays<wgpu::Texture>,
) -> TextureArrays<wgpu::TextureView> {
    TextureArrays {
        rg_bc5_unorm_512: create_texture_array_view(
            Some("texture_view_texture_array_rg_bc5_unorm_512"),
            &texture_arrays.rg_bc5_unorm_512,
            wgpu::TextureFormat::Bc5RgUnorm,
        ),
        rg_bc5_unorm_1024: create_texture_array_view(
            Some("texture_view_texture_array_rg_bc5_unorm_1024"),
            &texture_arrays.rg_bc5_unorm_1024,
            wgpu::TextureFormat::Bc5RgUnorm,
        ),
        rg_bc5_unorm_2048: create_texture_array_view(
            Some("texture_view_texture_array_rg_bc5_unorm_2048"),
            &texture_arrays.rg_bc5_unorm_2048,
            wgpu::TextureFormat::Bc5RgUnorm,
        ),
        rg_bc5_unorm_4096: create_texture_array_view(
            Some("texture_view_texture_array_rg_bc5_unorm_4096"),
            &texture_arrays.rg_bc5_unorm_4096,
            wgpu::TextureFormat::Bc5RgUnorm,
        ),
        rgb_bc7_unorm_512: create_texture_array_view(
            Some("texture_view_texture_array_rgb_bc7_unorm_512"),
            &texture_arrays.rgb_bc7_unorm_512,
            wgpu::TextureFormat::Bc7RgbaUnorm,
        ),
        rgb_bc7_unorm_1024: create_texture_array_view(
            Some("texture_view_texture_array_rgb_bc7_unorm_1024"),
            &texture_arrays.rgb_bc7_unorm_1024,
            wgpu::TextureFormat::Bc7RgbaUnorm,
        ),
        rgb_bc7_unorm_2048: create_texture_array_view(
            Some("texture_view_texture_array_rgb_bc7_unorm_2048"),
            &texture_arrays.rgb_bc7_unorm_2048,
            wgpu::TextureFormat::Bc7RgbaUnorm,
        ),
        rgb_bc7_unorm_4096: create_texture_array_view(
            Some("texture_view_texture_array_rgb_bc7_unorm_4096"),
            &texture_arrays.rgb_bc7_unorm_4096,
            wgpu::TextureFormat::Bc7RgbaUnorm,
        ),
        rgba_bc7_srgb_512: create_texture_array_view(
            Some("texture_view_texture_array_rgba_bc7_srgb_512"),
            &texture_arrays.rgba_bc7_srgb_512,
            wgpu::TextureFormat::Bc7RgbaUnormSrgb,
        ),
        rgba_bc7_srgb_1024: create_texture_array_view(
            Some("texture_view_texture_array_rgba_bc7_srgb_1024"),
            &texture_arrays.rgba_bc7_srgb_1024,
            wgpu::TextureFormat::Bc7RgbaUnormSrgb,
        ),
        rgba_bc7_srgb_2048: create_texture_array_view(
            Some("texture_view_texture_array_rgba_bc7_srgb_2048"),
            &texture_arrays.rgba_bc7_srgb_2048,
            wgpu::TextureFormat::Bc7RgbaUnormSrgb,
        ),
        rgba_bc7_srgb_4096: create_texture_array_view(
            Some("texture_view_texture_array_rgba_bc7_srgb_4096"),
            &texture_arrays.rgba_bc7_srgb_4096,
            wgpu::TextureFormat::Bc7RgbaUnormSrgb,
        ),
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraMatrix {
    pub position: [f32; 4],
    pub view_projection: [f32; 16],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AmbientLight {
    pub color: [f32; 3],
    pub strength: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLight {
    pub color: [f32; 3],
    pub strength: f32,
    pub position: [f32; 3],
    pub range: f32,
}
