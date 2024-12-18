use bevy_ecs::system::Resource;

// +X is right, +Y is forward, +Z is up.
#[derive(Resource, Debug)]
pub struct Camera {
    pub position: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub aspect_ratio: f32,
    pub fov: f32, // In radians.
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(
            glam::Vec3::new(-self.position.x, -self.position.y, self.position.z),
            glam::Vec3::new(-self.target.x, -self.target.y, self.target.z),
            self.up,
        )
    }

    pub fn perspective(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: (-0.9, -0.9, 0.6).into(),
            target: (0.0, 0.0, 0.3).into(),
            up: glam::Vec3::Z,
            aspect_ratio: 16.0 / 9.0,
            fov: 40.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Frustum {
    pub left_plane: Plane,
    pub right_plane: Plane,
    pub bottom_plane: Plane,
    pub top_plane: Plane,
    pub near_plane: Plane,
    pub far_plane: Plane,
    pub corners: [[f32; 4]; 8], // Array of padded [f32; 3] corners.
}

impl Frustum {
    pub fn from_view_projection_matrix(view_projection_matrix: &glam::Mat4) -> Self {
        let left_plane = Plane {
            normal: glam::Vec3::new(
                view_projection_matrix.row(3)[0] + view_projection_matrix.row(0)[0],
                view_projection_matrix.row(3)[1] + view_projection_matrix.row(0)[1],
                view_projection_matrix.row(3)[2] + view_projection_matrix.row(0)[2],
            )
            .normalize()
            .into(),
            distance: view_projection_matrix.row(3)[3] + view_projection_matrix.row(0)[3],
        };

        let right_plane = Plane {
            normal: glam::Vec3::new(
                view_projection_matrix.row(3)[0] - view_projection_matrix.row(0)[0],
                view_projection_matrix.row(3)[1] - view_projection_matrix.row(0)[1],
                view_projection_matrix.row(3)[2] - view_projection_matrix.row(0)[2],
            )
            .normalize()
            .into(),
            distance: view_projection_matrix.row(3)[3] - view_projection_matrix.row(0)[3],
        };

        let bottom_plane = Plane {
            normal: glam::Vec3::new(
                view_projection_matrix.row(3)[0] + view_projection_matrix.row(1)[0],
                view_projection_matrix.row(3)[1] + view_projection_matrix.row(1)[1],
                view_projection_matrix.row(3)[2] + view_projection_matrix.row(1)[2],
            )
            .normalize()
            .into(),
            distance: view_projection_matrix.row(3)[3] + view_projection_matrix.row(1)[3],
        };

        let top_plane = Plane {
            normal: glam::Vec3::new(
                view_projection_matrix.row(3)[0] - view_projection_matrix.row(1)[0],
                view_projection_matrix.row(3)[1] - view_projection_matrix.row(1)[1],
                view_projection_matrix.row(3)[2] - view_projection_matrix.row(1)[2],
            )
            .normalize()
            .into(),
            distance: view_projection_matrix.row(3)[3] - view_projection_matrix.row(1)[3],
        };

        let near_plane = Plane {
            normal: glam::Vec3::new(
                view_projection_matrix.row(3)[0] + view_projection_matrix.row(2)[0],
                view_projection_matrix.row(3)[1] + view_projection_matrix.row(2)[1],
                view_projection_matrix.row(3)[2] + view_projection_matrix.row(2)[2],
            )
            .normalize()
            .into(),
            distance: view_projection_matrix.row(3)[3] + view_projection_matrix.row(2)[3],
        };

        let far_plane = Plane {
            normal: glam::Vec3::new(
                view_projection_matrix.row(3)[0] - view_projection_matrix.row(2)[0],
                view_projection_matrix.row(3)[1] - view_projection_matrix.row(2)[1],
                view_projection_matrix.row(3)[2] - view_projection_matrix.row(2)[2],
            )
            .normalize()
            .into(),
            distance: view_projection_matrix.row(3)[3] - view_projection_matrix.row(2)[3],
        };

        let mut corners: [[f32; 4]; 8] = Default::default();
        let inverse_view_projection_matrix = view_projection_matrix.inverse();

        for (i, corner) in corners.iter_mut().enumerate() {
            let x = if (i & 1) == 0 { -1.0 } else { 1.0 };
            let y = if (i & 2) == 0 { -1.0 } else { 1.0 };
            let z = if (i & 4) == 0 { -1.0 } else { 1.0 };

            let clip_space_corner = glam::Vec4::new(x, y, z, 1.0);
            let world_space_corner = inverse_view_projection_matrix * clip_space_corner;

            let unpadded_corner = glam::Vec3::new(
                world_space_corner.x,
                world_space_corner.y,
                world_space_corner.z,
            ) / world_space_corner.w;

            *corner = [unpadded_corner.x, unpadded_corner.y, unpadded_corner.z, 0.0];
        }

        Self {
            left_plane,
            right_plane,
            bottom_plane,
            top_plane,
            near_plane,
            far_plane,
            corners,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Plane {
    pub normal: [f32; 3],
    pub distance: f32,
}
