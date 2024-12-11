struct Material {
  base_color_texture_array_id: u32,
  base_color_texture_id: u32,
  normal_texture_array_id: u32,
  normal_texture_id: u32,
}

struct Instance {
    material_id: u32,
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
};

@group(0) @binding(0)
var<storage, read> materials: array<Material>;
@group(0) @binding(1)
var<storage, read> instances: array<Instance>;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) tex_coords: vec2<f32>,
  @location(2) normal: vec3<f32>,
  @location(3) tangent: vec3<f32>,
  @location(4) bitangent: vec3<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) instance_index: u32,
  @location(1) tex_coords: vec2<f32>,
  @location(2) tangent_position: vec3<f32>,
  @location(3) tangent_light_position: vec3<f32>,
  @location(4) tangent_view_position: vec3<f32>,
};

@vertex
fn vs_main(
  model: VertexInput,
  @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
  // TODO: implement.
}

@group(1) @binding(0)
var texture_array_unorm_srgb_512: texture_2d_array<f32>;
@group(1) @binding(1)
var texture_array_unorm_srgb_1024: texture_2d_array<f32>;
@group(1) @binding(2)
var texture_array_unorm_srgb_2048: texture_2d_array<f32>;
@group(1) @binding(3)
var texture_array_unorm_srgb_4096: texture_2d_array<f32>;
@group(1) @binding(4)
var texture_array_unorm_512: texture_2d_array<f32>;
@group(1) @binding(5)
var texture_array_unorm_1024: texture_2d_array<f32>;
@group(1) @binding(6)
var texture_array_unorm_2048: texture_2d_array<f32>;
@group(1) @binding(7)
var texture_array_unorm_4096: texture_2d_array<f32>;
@group(1) @binding(8)
var texture_array_hdr_512: texture_2d_array<f32>;
@group(1) @binding(9)
var texture_array_hdr_1024: texture_2d_array<f32>;
@group(1) @binding(10)
var texture_array_hdr_2048: texture_2d_array<f32>;
@group(1) @binding(11)
var texture_array_hdr_4096: texture_2d_array<f32>;
@group(1) @binding(12)
var texture_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  // TODO: implement.
}
