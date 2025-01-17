struct Camera {
  position: vec3<f32>,
  view_projection: mat4x4<f32>,
}

struct Material {
  base_color_rgba: vec4<f32>,
  normal_solid: vec3<f32>,
  base_color_texture: TextureReference,
  normal_texture: TextureReference,
  occlusion_solid: f32,
  occlusion_texture: TextureReference,
  occlusion_texture_channel: u32,
  roughness_solid: f32,
  roughness_texture: TextureReference,
  roughness_texture_channel: u32,
  metallic_solid: f32,
  metallic_texture: TextureReference,
  metallic_texture_channel: u32,
  bitmask: u32,
}

struct TextureReference {
  texture_array_id: u32,
  texture_id: u32,
}

struct InstanceMaterial {
  material_id: u32,
}

struct InstanceTransform {
  matrix_col_0: vec4<f32>,
  matrix_col_1: vec4<f32>,
  matrix_col_2: vec4<f32>,
  matrix_col_3: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;
@group(0) @binding(1)
var<storage, read> instance_transforms: array<InstanceTransform>;
@group(0) @binding(2)
var<storage, read> indirect_instances: array<u32>;
@group(0) @binding(3)
var<storage, read> instance_materials: array<InstanceMaterial>;

struct Vertex {
  @location(0) position: vec3<f32>,
  @location(1) tex_coords: vec2<f32>,
  @location(2) normal: vec3<f32>,
  @location(3) tangent: vec3<f32>,
  @location(4) bitangent: vec3<f32>,
}

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) object_index: u32,
  @location(1) tex_coords: vec2<f32>,
  @location(2) world_position: vec3<f32>,
  @location(3) normal: vec3<f32>,
  @location(4) tangent: vec3<f32>,
  @location(5) bitangent: vec3<f32>,
}

@vertex
fn vs_main(
  vertex: Vertex,
  @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
  let object_index = indirect_instances[instance_index];

  let transform = mat4x4<f32>(
    instance_transforms[object_index].matrix_col_0,
    instance_transforms[object_index].matrix_col_1,
    instance_transforms[object_index].matrix_col_2,
    instance_transforms[object_index].matrix_col_3,
  );

  let world_position = transform * vec4<f32>(vertex.position, 1.0);

  //let camera_space_position = world_position.xyz - camera.position;

  var vertex_output: VertexOutput;
  //vertex_output.clip_position = camera.view_projection * vec4<f32>(camera_space_position, 1.0);
  vertex_output.clip_position = camera.view_projection * world_position;
  vertex_output.object_index = object_index;
  vertex_output.tex_coords = vertex.tex_coords;
  vertex_output.world_position = world_position.xyz;
  vertex_output.normal = vertex.normal;
  vertex_output.tangent = vertex.tangent;
  vertex_output.bitangent = vertex.bitangent;

  return vertex_output;
}

@group(1) @binding(0)
var<storage, read> materials: array<Material>;
@group(1) @binding(1)
var texture_array_rg_bc5_unorm_512: texture_2d_array<f32>;
@group(1) @binding(2)
var texture_array_rg_bc5_unorm_1024: texture_2d_array<f32>;
@group(1) @binding(3)
var texture_array_rg_bc5_unorm_2048: texture_2d_array<f32>;
@group(1) @binding(4)
var texture_array_rg_bc5_unorm_4096: texture_2d_array<f32>;
@group(1) @binding(5)
var texture_array_rgb_bc7_unorm_512: texture_2d_array<f32>;
@group(1) @binding(6)
var texture_array_rgb_bc7_unorm_1024: texture_2d_array<f32>;
@group(1) @binding(7)
var texture_array_rgb_bc7_unorm_2048: texture_2d_array<f32>;
@group(1) @binding(8)
var texture_array_rgb_bc7_unorm_4096: texture_2d_array<f32>;
@group(1) @binding(9)
var texture_array_rgba_bc7_srgb_512: texture_2d_array<f32>;
@group(1) @binding(10)
var texture_array_rgba_bc7_srgb_1024: texture_2d_array<f32>;
@group(1) @binding(11)
var texture_array_rgba_bc7_srgb_2048: texture_2d_array<f32>;
@group(1) @binding(12)
var texture_array_rgba_bc7_srgb_4096: texture_2d_array<f32>;
@group(1) @binding(13)
var base_color_sampler: sampler;
@group(1) @binding(14)
var normal_sampler: sampler;

struct AmbientLight {
  color: vec3<f32>,
  strength: f32,
}

struct PointLight {
  color: vec3<f32>,
  strength: f32,
  position: vec3<f32>,
  range: f32,
}

@group(2) @binding(0)
var<uniform> ambient_light: AmbientLight;
@group(2) @binding(1)
var<storage, read> point_lights: array<PointLight>;
@group(2) @binding(2)
var<uniform> point_lights_length: u32;

@fragment
fn fs_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
  let material_id = instance_materials[vertex_output.object_index].material_id;
  let material = materials[material_id];

  var color = vec3<f32>(0.0, 0.0, 0.0);

  let object_color: vec4<f32> = base_color(material, vertex_output.tex_coords);
  let ambient_occlusion: f32 = occlusion(material, vertex_output.tex_coords);
  let roughness: f32 = roughness(material, vertex_output.tex_coords);
  let metalness: f32 = metallic(material, vertex_output.tex_coords);

  let ambient_color = ambient_light.strength * ambient_light.color * object_color.xyz * ambient_occlusion;
  color += ambient_color;

  let tbn = mat3x3<f32>(vertex_output.tangent, vertex_output.bitangent, vertex_output.normal);
  let object_normal: vec3<f32> = normal(material, vertex_output.tex_coords, tbn);

  let view_direction = normalize(camera.position - vertex_output.world_position);

  for (var i: u32 = 0; i < point_lights_length; i++) {
    let point_light = point_lights[i];
    let point_light_direction = point_light.position - vertex_output.world_position;
    let distance = length(point_light_direction);
    let attenuation = max(0.0, 1.0 - pow(distance / point_light.range, 2.0));
    let point_light_direction_normalized = normalize(point_light_direction);

    let diffuse_factor = max(dot(object_normal, point_light_direction_normalized), 0.0);
    let diffuse_color = attenuation * diffuse_factor * point_light.strength * point_light.color * object_color.xyz;
    color += diffuse_color;

    let halfway_vector = normalize(point_light_direction_normalized + view_direction);
    let specular_factor = max(dot(object_normal, halfway_vector), 0.0);
    let specular_intensity = pow(specular_factor, (1.0 - roughness) * 128.0);
    var specular_color: vec3<f32>;
    if (metalness > 0.5) {
      // For metals, specular color is the color of the material.
      specular_color = object_color.xyz * specular_intensity;
    } else {
      // For non-metals, specular color is typically white.
      specular_color = specular_intensity * vec3<f32>(1.0, 1.0, 1.0);
    }
    specular_color = specular_color * attenuation * point_light.strength * point_light.color;
    color += specular_color;
  }

  return vec4<f32>(color, object_color.w);
}

fn base_color(material: Material, tex_coords: vec2<f32>) -> vec4<f32> {
  const BASE_COLOR_FLAG: u32 = 1u << 1u;

  if (material.bitmask & BASE_COLOR_FLAG) != 0u {
    return sample_texture_2d_array(
      material.base_color_texture.texture_array_id,
      material.base_color_texture.texture_id,
      base_color_sampler,
      tex_coords
    );
  } else {
    return material.base_color_rgba;
  }
}

fn normal(material: Material, tex_coords: vec2<f32>, tbn: mat3x3<f32>) -> vec3<f32> {
  const NORMAL_FLAG: u32 = 1u << 2u;

  if (material.bitmask & NORMAL_FLAG) != 0u {
    let sampled_texture = sample_texture_2d_array(
      material.normal_texture.texture_array_id,
      material.normal_texture.texture_id,
      normal_sampler,
      tex_coords
    );

    let object_normal_xy = sampled_texture.xy * 2.0 - 1.0;
    let object_normal_z = sqrt(1.0 - dot(object_normal_xy, object_normal_xy));
    let object_normal = normalize(tbn * vec3<f32>(object_normal_xy, object_normal_z));

    return object_normal;
  } else {
    return material.normal_solid;
  }
}

fn occlusion(material: Material, tex_coords: vec2<f32>) -> f32 {
  const OCCLUSION_FLAG: u32 = 1u << 3u;

  if (material.bitmask & OCCLUSION_FLAG) != 0u {
    let sampled_texture = sample_texture_2d_array(
      material.occlusion_texture.texture_array_id,
      material.occlusion_texture.texture_id,
      base_color_sampler,
      tex_coords
    );
    return sample_texture_channel(sampled_texture, material.occlusion_texture_channel);
  } else {
    return material.occlusion_solid;
  }
}

fn roughness(material: Material, tex_coords: vec2<f32>) -> f32 {
  const ROUGHNESS_FLAG: u32 = 1u << 4u;

  if (material.bitmask & ROUGHNESS_FLAG) != 0u {
    let sampled_texture = sample_texture_2d_array(
      material.roughness_texture.texture_array_id,
      material.roughness_texture.texture_id,
      base_color_sampler,
      tex_coords
    );
    return sample_texture_channel(sampled_texture, material.roughness_texture_channel);
  } else {
    return material.roughness_solid;
  }
}

fn metallic(material: Material, tex_coords: vec2<f32>) -> f32 {
  const METALLIC_FLAG: u32 = 1u << 5u;

  if (material.bitmask & METALLIC_FLAG) != 0u {
    let sampled_texture = sample_texture_2d_array(
      material.metallic_texture.texture_array_id,
      material.metallic_texture.texture_id,
      base_color_sampler,
      tex_coords
    );
    return sample_texture_channel(sampled_texture, material.metallic_texture_channel);
  } else {
    return material.metallic_solid;
  }
}

fn sample_texture_2d_array(texture_array_id: u32, texture_id: u32, s: sampler, tex_coords: vec2<f32>) -> vec4<f32> {
  switch texture_array_id {
    case 0u, default: {
      return textureSample(texture_array_rg_bc5_unorm_512, s, tex_coords, texture_id);
    }
    case 1u: {
      return textureSample(texture_array_rg_bc5_unorm_1024, s, tex_coords, texture_id);
    }
    case 2u: {
      return textureSample(texture_array_rg_bc5_unorm_2048, s, tex_coords, texture_id);
    }
    case 3u: {
      return textureSample(texture_array_rg_bc5_unorm_4096, s, tex_coords, texture_id);
    }
    case 4u: {
      return textureSample(texture_array_rgb_bc7_unorm_512, s, tex_coords, texture_id);
    }
    case 5u: {
      return textureSample(texture_array_rgb_bc7_unorm_1024, s, tex_coords, texture_id);
    }
    case 6u: {
      return textureSample(texture_array_rgb_bc7_unorm_2048, s, tex_coords, texture_id);
    }
    case 7u: {
      return textureSample(texture_array_rgb_bc7_unorm_4096, s, tex_coords, texture_id);
    }
    case 8u: {
      return textureSample(texture_array_rgba_bc7_srgb_512, s, tex_coords, texture_id);
    }
    case 9u: {
      return textureSample(texture_array_rgba_bc7_srgb_1024, s, tex_coords, texture_id);
    }
    case 10u: {
      return textureSample(texture_array_rgba_bc7_srgb_2048, s, tex_coords, texture_id);
    }
    case 11u: {
      return textureSample(texture_array_rgba_bc7_srgb_4096, s, tex_coords, texture_id);
    }
  }
}

fn sample_texture_channel(sampled_texture: vec4<f32>, channel: u32) -> f32 {
  switch channel {
    case 0u, default: {
      return sampled_texture.r;
    }
    case 1u: {
      return sampled_texture.g;
    }
    case 2u: {
      return sampled_texture.b;
    }
    case 3u: {
      return sampled_texture.a;
    }
  }
}
