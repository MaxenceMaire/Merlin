use std::ops::{Deref, DerefMut};

pub const TIMESTEP: f32 = 1.0 / 64.0;
pub const GRAVITY: rapier3d::math::Vector<f32> = rapier3d::math::Vector::new(0.0, -9.81, 0.0);

#[derive(bevy_ecs::system::Resource)]
pub struct Physics {
    pub physics_pipeline: rapier3d::pipeline::PhysicsPipeline,
    pub query_pipeline: rapier3d::pipeline::QueryPipeline,
    pub broad_phase: rapier3d::geometry::BroadPhaseMultiSap,
    pub narrow_phase: rapier3d::geometry::NarrowPhase,
    pub collider_set: rapier3d::geometry::ColliderSet,
    pub integration_parameters: rapier3d::dynamics::IntegrationParameters,
    pub rigid_body_set: rapier3d::dynamics::RigidBodySet,
    pub impulse_joint_set: rapier3d::dynamics::ImpulseJointSet,
    pub multibody_joint_set: rapier3d::dynamics::MultibodyJointSet,
    pub island_manager: rapier3d::dynamics::IslandManager,
    pub ccd_solver: rapier3d::dynamics::CCDSolver,
    pub gravity: rapier3d::math::Vector<f32>,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            physics_pipeline: rapier3d::pipeline::PhysicsPipeline::new(),
            query_pipeline: rapier3d::pipeline::QueryPipeline::new(),
            broad_phase: rapier3d::geometry::BroadPhaseMultiSap::new(),
            narrow_phase: rapier3d::geometry::NarrowPhase::new(),
            collider_set: rapier3d::geometry::ColliderSet::new(),
            integration_parameters: rapier3d::dynamics::IntegrationParameters {
                dt: TIMESTEP,
                ..Default::default()
            },
            rigid_body_set: rapier3d::dynamics::RigidBodySet::new(),
            impulse_joint_set: rapier3d::dynamics::ImpulseJointSet::new(),
            multibody_joint_set: rapier3d::dynamics::MultibodyJointSet::new(),
            island_manager: rapier3d::dynamics::IslandManager::new(),
            ccd_solver: rapier3d::dynamics::CCDSolver::new(),
            gravity: GRAVITY,
        }
    }
}

impl Physics {
    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }
}

#[derive(bevy_ecs::component::Component, Debug)]
pub struct RigidBody(pub rapier3d::dynamics::RigidBodyHandle);

#[derive(bevy_ecs::component::Component, Debug)]
pub struct Collider(pub rapier3d::geometry::ColliderHandle);

#[derive(bevy_ecs::system::Resource, Debug)]
pub struct LastStepTimestamp(pub std::time::Instant);

impl Deref for LastStepTimestamp {
    type Target = std::time::Instant;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LastStepTimestamp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
