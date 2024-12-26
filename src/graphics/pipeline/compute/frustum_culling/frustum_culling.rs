use crate::ecs;
use wgpu::util::DeviceExt;

pub struct FrustumCulling {
    compute_pipeline: wgpu::ComputePipeline,
    bind_group_layout_frustum_culling: wgpu::BindGroupLayout,
}

impl FrustumCulling {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout_frustum_culling =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader_frustum_culling"),
            source: wgpu::ShaderSource::Wgsl(include_str!("frustum_culling.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout_frustum_culling"),
            bind_group_layouts: &[&bind_group_layout_frustum_culling],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute_pipeline_descriptor_frustum_culling"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            compute_pipeline,
            bind_group_layout_frustum_culling,
        }
    }

    pub fn prepare(
        &self,
        compute_pass: &mut wgpu::ComputePass,
        bind_group_frustum_culling: &wgpu::BindGroup,
    ) {
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, bind_group_frustum_culling, &[]);
    }

    pub fn create_bind_group_frustum_culling(
        &self,
        device: &wgpu::Device,
        binding_resource_bounding_boxes_buffer: wgpu::BindingResource,
        binding_resource_instance_culling_information_buffer: wgpu::BindingResource,
        binding_resource_indirect_draw_commands_buffer: wgpu::BindingResource,
        binding_resource_indirect_instances_buffer: wgpu::BindingResource,
        binding_resource_frustum_buffer: wgpu::BindingResource,
        binding_resource_instance_count_buffer: wgpu::BindingResource,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group_frustum_culling"),
            layout: &self.bind_group_layout_frustum_culling,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: binding_resource_bounding_boxes_buffer,
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: binding_resource_instance_culling_information_buffer,
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: binding_resource_indirect_draw_commands_buffer,
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: binding_resource_indirect_instances_buffer,
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: binding_resource_frustum_buffer,
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: binding_resource_instance_count_buffer,
                },
            ],
        })
    }
}

pub fn create_instance_culling_information_buffer(
    device: &wgpu::Device,
    instance_culling_information: &[InstanceCullingInformation],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("instance_culling_information_buffer"),
        contents: bytemuck::cast_slice(&instance_culling_information),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

pub fn create_indirect_instances_buffer(
    device: &wgpu::Device,
    indirect_instances: &[u32],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("indirect_instances_buffer"),
        contents: bytemuck::cast_slice(indirect_instances),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

pub fn create_frustum_buffer(
    device: &wgpu::Device,
    frustum: ecs::resource::Frustum,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("frustum_buffer"),
        contents: bytemuck::cast_slice(&[frustum]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

pub fn create_instance_count_buffer(device: &wgpu::Device, instance_count: u32) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("instance_count_buffer"),
        contents: bytemuck::cast_slice(&[instance_count]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceCullingInformation {
    pub batch_id: u32,
}
