use super::{resource, scene};
use crate::ecs;
use crate::graphics;

pub struct Simulator {
    condvar_pair: std::sync::Arc<(std::sync::Mutex<bool>, std::sync::Condvar)>,
    resize_event: std::sync::Arc<crossbeam::atomic::AtomicCell<Option<ResizeEvent>>>,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            condvar_pair: std::sync::Arc::new((
                std::sync::Mutex::new(false),
                std::sync::Condvar::new(),
            )),
            resize_event: std::sync::Arc::new(crossbeam::atomic::AtomicCell::new(None)),
        }
    }

    pub fn spawn(
        &self,
        mut world: bevy_ecs::world::World,
        scene_to_renderer_sender: crossbeam::channel::Sender<bevy_ecs::world::World>,
        renderer_to_scene_receiver: crossbeam::channel::Receiver<bevy_ecs::world::World>,
    ) -> std::thread::JoinHandle<()> {
        let mut update_schedule = schedule::update();
        let condvar_pair = self.condvar_pair.clone();
        let resize_event = self.resize_event.clone();

        std::thread::spawn(move || loop {
            let (lock, cvar) = &*condvar_pair;
            {
                let mut update_requested = lock.lock().unwrap();
                while !*update_requested {
                    update_requested = cvar.wait(update_requested).unwrap();
                }
                *update_requested = false;
            }

            let Ok(mut render_world) = renderer_to_scene_receiver.recv() else {
                // Channel disconnected.
                return;
            };

            if let Some(ResizeEvent { width, height }) = resize_event.swap(None) {
                if let Some(mut camera) = world.get_resource_mut::<ecs::resource::Camera>() {
                    camera.aspect_ratio = width as f32 / height as f32;
                }

                let mut gpu = render_world.get_resource_mut::<graphics::Gpu>().unwrap();
                gpu.resize(width, height);
                let new_depth_buffer = graphics::gpu::create_depth_buffer(
                    &gpu.device,
                    width,
                    height,
                    scene::MSAA_SAMPLE_COUNT,
                );
                let new_msaa_buffer = graphics::gpu::create_msaa_buffer(
                    &gpu.device,
                    width,
                    height,
                    gpu.config.format,
                    scene::MSAA_SAMPLE_COUNT,
                );
                render_world.insert_resource::<resource::DepthBuffer>(resource::DepthBuffer(
                    new_depth_buffer,
                ));
                render_world
                    .insert_resource::<resource::MsaaBuffer>(resource::MsaaBuffer(new_msaa_buffer));
            }

            extract_world(&mut world, &mut render_world);

            scene_to_renderer_sender.send(render_world).unwrap();

            update_schedule.run(&mut world);
            world.clear_trackers();
        })
    }

    pub fn request_update(&mut self) {
        let (lock, cvar) = &*self.condvar_pair;
        let mut update_requested = lock.lock().unwrap();
        *update_requested = true;
        cvar.notify_one();
    }

    pub fn request_resize(&mut self, width: u32, height: u32) {
        self.resize_event.store(Some(ResizeEvent { width, height }));
    }
}

struct ResizeEvent {
    width: u32,
    height: u32,
}

fn extract_world(
    main_world: &mut bevy_ecs::world::World,
    render_world: &mut bevy_ecs::world::World,
) {
    let camera = main_world.get_resource::<ecs::resource::Camera>().unwrap();
    render_world.insert_resource::<ecs::resource::Camera>(camera.clone());

    // TODO: extract only visible entities.

    let mut query = main_world.query::<(
        bevy_ecs::entity::Entity,
        &ecs::component::Mesh,
        &ecs::component::Material,
        &ecs::component::GlobalTransform,
    )>();
    render_world
        .insert_or_spawn_batch(query.iter(main_world).map(
            |(entity, mesh, material, global_transform)| {
                (entity, (mesh.clone(), material.clone(), *global_transform))
            },
        ))
        .unwrap();
}

mod schedule {
    use super::{run_condition, system};
    use bevy_ecs::schedule::IntoSystemConfigs;

    pub fn update() -> bevy_ecs::schedule::Schedule {
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(
            (
                system::update_time,
                (
                    (system::step_physics, system::sync_ecs_to_physics)
                        .chain()
                        .run_if(run_condition::should_step_physics),
                    system::move_camera,
                ),
            )
                .chain(),
        );

        schedule
    }
}

mod system {
    use super::super::resource::*;
    use crate::ecs;
    use crate::physics;
    use bevy_ecs::change_detection::{Res, ResMut};
    use bevy_ecs::system::Query;

    pub fn update_time(mut timestamp: ResMut<Timestamp>, mut delta_time: ResMut<DeltaTime>) {
        let now = std::time::Instant::now();
        **delta_time = now - **timestamp;
        **timestamp = now;
    }

    pub fn move_camera(mut camera: ResMut<ecs::resource::Camera>, delta_time: Res<DeltaTime>) {
        let rotation = glam::Quat::from_axis_angle(
            glam::f32::Vec3::Y.normalize(),
            delta_time.as_millis() as f32 * 0.0001,
        );
        camera.position = rotation * camera.position;
    }

    pub fn step_physics(
        mut physics_world: ResMut<physics::Physics>,
        timestamp: Res<Timestamp>,
        mut last_physics_step_timestamp: ResMut<physics::LastStepTimestamp>,
    ) {
        let timestep_duration = std::time::Duration::from_secs_f32(physics::TIMESTEP);
        let mut last_step = **last_physics_step_timestamp;
        while last_step + timestep_duration <= **timestamp {
            physics_world.step();
            last_step += timestep_duration;
        }
        last_physics_step_timestamp.0 = last_step;
    }

    pub fn sync_ecs_to_physics(
        physics_world: Res<physics::Physics>,
        mut query: Query<(&physics::RigidBody, &mut ecs::component::GlobalTransform)>,
    ) {
        for (physics::RigidBody(rigid_body_handle), mut global_transform) in query.iter_mut() {
            if let Some(rigid_body) = physics_world.rigid_body_set.get(*rigid_body_handle) {
                let (scale, _, _) = global_transform.to_scale_rotation_translation();
                let rigid_body_position = rigid_body.position();
                **global_transform = glam::Affine3A::from_scale_rotation_translation(
                    scale,
                    glam::Quat::from_slice(rigid_body_position.rotation.coords.data.as_slice()),
                    glam::f32::Vec3::from_slice(rigid_body_position.translation.vector.as_slice()),
                );
            }
        }
    }
}

mod run_condition {
    use super::super::resource::*;
    use crate::physics;
    use bevy_ecs::change_detection::Res;

    pub fn should_step_physics(
        timestamp: Res<Timestamp>,
        last_physics_step_timestamp: Res<physics::LastStepTimestamp>,
    ) -> bool {
        **timestamp - **last_physics_step_timestamp
            >= std::time::Duration::from_secs_f32(physics::TIMESTEP)
    }
}
