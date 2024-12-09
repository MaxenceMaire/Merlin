use std::sync::Arc;

pub struct Gpu<'a> {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'a>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl<'a> Gpu<'a> {
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let mut required_features = wgpu::Features::empty();

        match texture_compression(&adapter) {
            TextureCompression::Astc => {
                required_features.set(wgpu::Features::TEXTURE_COMPRESSION_ASTC, true);
            }
            TextureCompression::Bc => {
                required_features.set(wgpu::Features::TEXTURE_COMPRESSION_BC, true);
            }
            _ => {}
        }

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features,
                    label: None,
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        Self {
            instance,
            surface,
            adapter,
            device,
            queue,
        }
    }

    fn test(device: &wgpu::Device, queue: &wgpu::Queue) {
        use wgpu::util::DeviceExt;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buffer"), // TODO: change label.
            contents: &[], // TODO: load all vertex data in an array. Bonus point if compressed.
            usage: wgpu::BufferUsages::STORAGE,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index_buffer"), // TODO: change label.
            contents: &[], // TODO: load all vertex data in an array. Bonus point if compressed.
            usage: wgpu::BufferUsages::STORAGE,
        });

        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct Mesh {
            // TODO: check alignment.
            vertex_offset: u32,
            vertex_count: u32,
            index_offset: u32,
            index_count: u32,
        }

        let entity_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("entity_buffer"), // TODO: change label.
            contents: &[], // TODO: load all vertex data in an array. Bonus point if compressed.
            usage: wgpu::BufferUsages::STORAGE,
        });

        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct Instance {
            // TODO: check alignment.
            texture_base_color_size: u32,
            texture_base_color_index: u32,
        }

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance_buffer"), // TODO: change label.
            contents: &[], // TODO: load all vertex data in an array. Bonus point if compressed.
            usage: wgpu::BufferUsages::STORAGE,
        });

        wgpu::TextureDescriptor {
            label: Some("texture_array_2048"),
            size: wgpu::Extent3d {
                width: 2048,
                height: 2048,
                depth_or_array_layers: 1, // TODO: number of textures.
            },
            mip_level_count: 1 + 2048_u32.ilog2(),
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bc3RgbaUnormSrgb, // TODO: change format.
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        // TODO: use BC2 for solid textures, BC3 for transparent textures.
        wgpu::TextureDescriptor {
            label: Some("texture_array_1024"),
            size: wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 1, // TODO: number of textures.
            },
            mip_level_count: 1 + 1024_u32.ilog2(),
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bc3RgbaUnormSrgb, // TODO: change format.
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"), // TODO: change label.
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let vertex_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("vertex_bind_group_layout"), // TODO: change label.
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let vertex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &vertex_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: vertex_buffer.as_entire_binding(),
            }],
            label: None,
        });

        // TODO: test 2d texture array creation.
        let texture_array = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("text_2d_texture_array"),
            size: wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 20, // number of textures in array.
            },
            mip_level_count: 11,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bc2RgbaUnormSrgb, // TODO: check features to
            // enable.
            usage: wgpu::TextureUsages::TEXTURE_BINDING, // TODO: change usage
            view_formats: &[],
        });

        device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("test_2d_texture_array"),
                size: wgpu::Extent3d {
                    width: 1024,
                    height: 1024,
                    depth_or_array_layers: 20, // number of textures in array.
                },
                mip_level_count: 11,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bc2RgbaUnormSrgb, // TODO: check features to
                // enable.
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, // TODO: change usage
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::MipMajor,
            &[],
        );

        /*
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"), // TODO: change label.
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[model::ModelVertex::descriptor(), InstanceRaw::descriptor()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE.
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL.
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION.
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });
        */
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum TextureCompression {
    Astc,
    Bc,
    None,
}

pub fn texture_compression(adapter: &wgpu::Adapter) -> TextureCompression {
    let features = adapter.features();
    if features.contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC) {
        TextureCompression::Astc
    } else if features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC) {
        TextureCompression::Bc
    } else {
        TextureCompression::None
    }
}
