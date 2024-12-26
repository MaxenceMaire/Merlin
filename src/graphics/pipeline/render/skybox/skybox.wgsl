struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) @interpolate(linear, centroid) position: vec4<f32>,
};

@vertex
fn vs_main(
  @location(0) vertex_position: vec2<f32>,
) -> VertexOutput {
  var vertex_output: VertexOutput;
  let position = vec4<f32>(vertex_position, 1.0, 1.0);
  vertex_output.clip_position = position;
  vertex_output.position = position;
  return vertex_output;
}

@group(0) @binding(0)
var<uniform> inverse_view_projection: mat4x4<f32>;

@group(1) @binding(0)
var cubemap: texture_cube<f32>;
@group(1) @binding(1)
var cubemap_sampler: sampler;

@fragment
fn fs_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
  let world_space_position = inverse_view_projection * vertex_output.position;
  let world_direction = normalize(world_space_position.xyz / world_space_position.w);
  return textureSample(cubemap, cubemap_sampler, world_direction * vec3f(1, 1, -1));
}
