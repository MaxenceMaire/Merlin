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
        binding_resource_texture_array_rg_bc5_unorm_512: wgpu::BindingResource,
        binding_resource_texture_array_rg_bc5_unorm_1024: wgpu::BindingResource,
        binding_resource_texture_array_rg_bc5_unorm_2048: wgpu::BindingResource,
        binding_resource_texture_array_rg_bc5_unorm_4096: wgpu::BindingResource,
        binding_resource_texture_array_rgb_bc7_unorm_512: wgpu::BindingResource,
        binding_resource_texture_array_rgb_bc7_unorm_1024: wgpu::BindingResource,
        binding_resource_texture_array_rgb_bc7_unorm_2048: wgpu::BindingResource,
        binding_resource_texture_array_rgb_bc7_unorm_4096: wgpu::BindingResource,
        binding_resource_texture_array_rgba_bc7_srgb_512: wgpu::BindingResource,
        binding_resource_texture_array_rgba_bc7_srgb_1024: wgpu::BindingResource,
        binding_resource_texture_array_rgba_bc7_srgb_2048: wgpu::BindingResource,
        binding_resource_texture_array_rgba_bc7_srgb_4096: wgpu::BindingResource,
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
                    resource: binding_resource_texture_array_rg_bc5_unorm_512,
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: binding_resource_texture_array_rg_bc5_unorm_1024,
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: binding_resource_texture_array_rg_bc5_unorm_2048,
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: binding_resource_texture_array_rg_bc5_unorm_4096,
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: binding_resource_texture_array_rgb_bc7_unorm_512,
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: binding_resource_texture_array_rgb_bc7_unorm_1024,
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: binding_resource_texture_array_rgb_bc7_unorm_2048,
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: binding_resource_texture_array_rgb_bc7_unorm_4096,
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: binding_resource_texture_array_rgba_bc7_srgb_512,
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: binding_resource_texture_array_rgba_bc7_srgb_1024,
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: binding_resource_texture_array_rgba_bc7_srgb_2048,
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: binding_resource_texture_array_rgba_bc7_srgb_4096,
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
