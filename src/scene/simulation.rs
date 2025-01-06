use crate::ecs;

pub struct Simulator {
    condvar_pair: std::sync::Arc<(std::sync::Mutex<bool>, std::sync::Condvar)>,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            condvar_pair: std::sync::Arc::new((
                std::sync::Mutex::new(false),
                std::sync::Condvar::new(),
            )),
        }
    }

    pub fn spawn(
        &self,
        mut world: bevy_ecs::world::World,
        scene_to_renderer_sender: crossbeam_channel::Sender<bevy_ecs::world::World>,
        renderer_to_scene_receiver: crossbeam_channel::Receiver<bevy_ecs::world::World>,
    ) -> std::thread::JoinHandle<()> {
        let mut update_schedule = schedule::update();
        let condvar_pair = self.condvar_pair.clone();

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

            extract_world(&mut world, &mut render_world);

            scene_to_renderer_sender.send(render_world).unwrap();

            update_schedule.run(&mut world);
        })
    }

    pub fn request_update(&mut self) {
        let (lock, cvar) = &*self.condvar_pair;
        let mut update_requested = lock.lock().unwrap();
        *update_requested = true;
        cvar.notify_one();
    }
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
    use super::system;

    pub fn update() -> bevy_ecs::schedule::Schedule {
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(system::move_camera);

        schedule
    }
}

mod system {
    use super::super::resource::*;
    use crate::ecs;
    use bevy_ecs::change_detection::ResMut;

    pub fn move_camera(
        mut camera: ResMut<ecs::resource::Camera>,
        mut timestamp: ResMut<Timestamp>,
    ) {
        let now = std::time::Instant::now();
        let delta_time = now - **timestamp;

        let rotation = glam::Quat::from_axis_angle(
            glam::f32::Vec3::Y.normalize(),
            delta_time.as_millis() as f32 * 0.0001,
        );
        camera.position = rotation * camera.position;

        **timestamp = now;
    }
}
