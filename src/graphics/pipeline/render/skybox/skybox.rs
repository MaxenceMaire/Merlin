use crate::graphics;
use wgpu::util::DeviceExt;

pub struct Skybox {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group_layout_inverse_view_projection: wgpu::BindGroupLayout,
    bind_group_layout_skybox: wgpu::BindGroupLayout,
}

impl Skybox {
    pub fn new(
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
        msaa_sample_count: u32,
    ) -> Self {
        const VERTEX_POSITIONS: [[f32; 2]; 6] = [
            [-1.0, 1.0],  // Top-left
            [-1.0, -1.0], // Bottom-left
            [1.0, 1.0],   // Top-right
            [-1.0, -1.0], // Bottom-left
            [1.0, -1.0],  // Bottom-right
            [1.0, 1.0],   // Top-right
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("skybox_vertex_buffer"),
            contents: bytemuck::cast_slice(&VERTEX_POSITIONS),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
        });

        let bind_group_layout_inverse_view_projection =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader_skybox"),
            source: wgpu::ShaderSource::Wgsl(include_str!("skybox.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout_skybox"),
                bind_group_layouts: &[
                    &bind_group_layout_inverse_view_projection,
                    &bind_group_layout_skybox,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline_skybox"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
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
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
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
                count: msaa_sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            bind_group_layout_inverse_view_projection,
            bind_group_layout_skybox,
        }
    }

    pub fn prepare(
        &self,
        render_pass: &mut wgpu::RenderPass,
        bind_group_inverse_view_projection: &wgpu::BindGroup,
        bind_group_skybox: &wgpu::BindGroup,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, bind_group_inverse_view_projection, &[]);
        render_pass.set_bind_group(1, bind_group_skybox, &[]);
    }

    pub fn draw(render_pass: &mut wgpu::RenderPass) {
        render_pass.draw(0..6, 0..1);
    }

    pub fn create_bind_group_inverse_view_projection(
        &self,
        device: &wgpu::Device,
        binding_resource_inverse_view_projection_buffer: wgpu::BindingResource,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group_inverse_view_projection"),
            layout: &self.bind_group_layout_inverse_view_projection,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: binding_resource_inverse_view_projection_buffer,
            }],
        })
    }

    pub fn create_bind_group_skybox(
        &self,
        device: &wgpu::Device,
        binding_resource_texture_skybox: wgpu::BindingResource,
        binding_resource_texture_sampler_skybox: wgpu::BindingResource,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group_skybox"),
            layout: &self.bind_group_layout_skybox,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: binding_resource_texture_skybox,
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: binding_resource_texture_sampler_skybox,
                },
            ],
        })
    }
}

pub fn create_bounding_boxes_buffer(
    device: &wgpu::Device,
    bounding_boxes: &[graphics::BoundingBox],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("bounding_boxes_buffer"),
        contents: bytemuck::cast_slice(bounding_boxes),
        usage: wgpu::BufferUsages::STORAGE,
    })
}

pub fn create_inverse_view_projection_buffer(
    device: &wgpu::Device,
    inverse_view_projection: &[f32; 16],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("inverse_view_projection_buffer"),
        contents: bytemuck::cast_slice(inverse_view_projection),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}
