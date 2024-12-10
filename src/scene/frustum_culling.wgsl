struct InstanceCullingInformation {
  batch_id: u32,
  bounding_box_min: vec3<f32>,
  bounding_box_max: vec3<f32>,
}

struct DrawIndexedIndirectArgs {
  index_count: u32,
  instance_count: atomic<u32>,
  first_index: u32,
  base_vertex: i32,
  first_instance: u32,
}

struct Plane {
  normal: vec3<f32>,
  distance: f32, // Distance from the origin along the normal.
}

struct Frustum {
  left: Plane,
  right: Plane,
  top: Plane,
  bottom: Plane,
  near: Plane,
  far: Plane,
  corners: array<vec3<f32>, 8>,
}

@group(0) @binding(0)
var<storage, read> instance_culling_information: array<InstanceCullingInformation>;
@group(0) @binding(1)
var<storage, read_write> indirect_draw_commands: array<DrawIndexedIndirectArgs>;
@group(0) @binding(2)
var<storage, read_write> instance_buffer: array<u32>;
@group(0) @binding(3)
var<uniform> frustum: Frustum;
@group(0) @binding(4)
var<uniform> instance_count: u32;

@compute @workgroup_size(64) fn cs_main (
  @builtin(global_invocation_id) id: vec3<u32>
) {
  let instance_id = id.x;

  if instance_id >= instance_count {
    return;
  }

  let instance = instance_culling_information[instance_id];

  if intersects_frustum(instance.bounding_box_min, instance.bounding_box_max) {
    let batch_instance_id = atomicAdd(&indirect_draw_commands[instance.batch_id].instance_count, 1u);
    let buffer_instance_id = indirect_draw_commands[instance.batch_id].first_instance + batch_instance_id;
    instance_buffer[buffer_instance_id] = instance_id;
  }
}

fn intersects_frustum(bounding_box_min: vec3<f32>, bounding_box_max: vec3<f32>) -> bool {
  return
    protrudes_plane(bounding_box_min, bounding_box_max, frustum.left)
    && protrudes_plane(bounding_box_min, bounding_box_max, frustum.right)
    && protrudes_plane(bounding_box_min, bounding_box_max, frustum.top)
    && protrudes_plane(bounding_box_min, bounding_box_max, frustum.bottom)
    && protrudes_plane(bounding_box_min, bounding_box_max, frustum.near)
    && protrudes_plane(bounding_box_min, bounding_box_max, frustum.far)
    // Extra check to cull large objects outside the frustum but still protruding all frustum planes.
    && bounding_box_contains_frustum(bounding_box_min, bounding_box_max);
}

fn protrudes_plane(bounding_box_min: vec3<f32>, bounding_box_max: vec3<f32>, plane: Plane) -> bool {
  // Farthest point in the direction of the plane normal.
  var p: vec3<f32>;
  if plane.normal.x > 0.0 { p.x = bounding_box_max.x; } else { p.x = bounding_box_min.x; }
  if plane.normal.y > 0.0 { p.y = bounding_box_max.y; } else { p.y = bounding_box_min.y; }
  if plane.normal.z > 0.0 { p.z = bounding_box_max.z; } else { p.z = bounding_box_min.z; }

  return dot(plane.normal, p) + plane.distance >= 0.0;
}

fn bounding_box_contains_frustum(bounding_box_min: vec3<f32>, bounding_box_max: vec3<f32>) -> bool {
  for (var i = 0; i < 8; i = i + 1) {
    let corner = frustum.corners[i];

    if (corner.x < bounding_box_min.x || corner.x > bounding_box_max.x ||
        corner.y < bounding_box_min.y || corner.y > bounding_box_max.y ||
        corner.z < bounding_box_min.z || corner.z > bounding_box_max.z) {
      return false;
    }
  }

  return true;
}
