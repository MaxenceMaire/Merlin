struct Camera {
  view_projection: mat4x4<f32>,
  position: vec3<f32>,
};

struct Material {
  base_color_texture_array_id: u32,
  base_color_texture_id: u32,
  normal_texture_array_id: u32,
  normal_texture_id: u32,
}

struct InstanceMaterial {
    material_id: u32,
};

struct InstanceTransform {
    matrix_0: vec4<f32>,
    matrix_1: vec4<f32>,
    matrix_2: vec4<f32>,
    matrix_3: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;
@group(0) @binding(1)
var<storage, read> instance_transform: array<InstanceTransform>;
@group(0) @binding(2)
var<storage, read> materials: array<Material>;
@group(0) @binding(3)
var<storage, read> instance_materials: array<InstanceMaterial>;

struct Vertex {
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
};

@vertex
fn vs_main(
  vertex: Vertex,
  @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
  let transform = mat4x4<f32>(
    instance_transform[instance_index].matrix_0,
    instance_transform[instance_index].matrix_1,
    instance_transform[instance_index].matrix_2,
    instance_transform[instance_index].matrix_3,
  );

  let world_position = transform * vec4<f32>(vertex.position, 1.0);

  let camera_space_position = world_position.xyz - camera.position;

  var vertex_output: VertexOutput;
  vertex_output.clip_position = camera.view_projection * vec4<f32>(camera_space_position, 1.0);
  vertex_output.instance_index = instance_index;
  vertex_output.tex_coords = vertex.tex_coords;

  return vertex_output;
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
var base_color_sampler: sampler;
@group(1) @binding(13)
var normal_sampler: sampler;

@fragment
fn fs_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
  let material_id = instance_materials[vertex_output.instance_index].material_id;
  let material = materials[material_id];

  let object_color: vec4<f32> = sample_texture_2d_array(
    material.base_color_texture_array_id,
    material.base_color_texture_id,
    base_color_sampler,
    vertex_output.tex_coords
  );

  let object_normal: vec4<f32> = sample_texture_2d_array(
    material.normal_texture_array_id,
    material.normal_texture_id,
    normal_sampler,
    vertex_output.tex_coords
  );

  let result = object_color.xyz;

  return vec4<f32>(result, object_color.a);
}

fn sample_texture_2d_array(texture_array_id: u32, texture_id: u32, s: sampler, tex_coords: vec2<f32>) -> vec4<f32> {
  switch texture_array_id {
    case 0u, default: {
      return textureSample(texture_array_unorm_srgb_512, s, tex_coords, texture_id);
    }
    case 1u: {
      return textureSample(texture_array_unorm_srgb_1024, s, tex_coords, texture_id);
    }
    case 2u: {
      return textureSample(texture_array_unorm_srgb_2048, s, tex_coords, texture_id);
    }
    case 3u: {
      return textureSample(texture_array_unorm_srgb_4096, s, tex_coords, texture_id);
    }
    case 4u: {
      return textureSample(texture_array_unorm_512, s, tex_coords, texture_id);
    }
    case 5u: {
      return textureSample(texture_array_unorm_1024, s, tex_coords, texture_id);
    }
    case 6u: {
      return textureSample(texture_array_unorm_2048, s, tex_coords, texture_id);
    }
    case 7u: {
      return textureSample(texture_array_unorm_4096, s, tex_coords, texture_id);
    }
    case 8u: {
      return textureSample(texture_array_hdr_512, s, tex_coords, texture_id);
    }
    case 9u: {
      return textureSample(texture_array_hdr_1024, s, tex_coords, texture_id);
    }
    case 10u: {
      return textureSample(texture_array_hdr_2048, s, tex_coords, texture_id);
    }
    case 11u: {
      return textureSample(texture_array_hdr_4096, s, tex_coords, texture_id);
    }
  }
}
