use bevy::prelude::*;

pub struct RigPlugin;

impl Plugin for RigPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KiRoot>()
            .register_type::<KiRootDriver>()
            .register_type::<KiLoop>()
            .register_type::<KiBone>()
            .register_type::<KiEffector>()
            .register_type::<KiRevoluteJoint>()
            .register_type::<KiSphericalJoint>();
    }
}

#[derive(Component, Reflect)]
pub struct KiRoot;

#[derive(Component, Reflect)]
pub struct KiRootDriver;

#[derive(Component, Reflect)]
pub struct KiLoop;

#[derive(Component, Reflect)]
pub struct KiBone {
    pub length: f32,
}

impl KiBone {
    pub fn new(length: f32) -> Self {
        Self { length }
    }
}

#[derive(Component, Reflect)]
pub struct KiEffector;

#[derive(PartialEq, Eq, Reflect)]
pub enum KiJointType {
    Revolute,
    Spherical,
}

#[derive(Component, Reflect)]
pub struct KiRevoluteJoint {
    pub length: f32,
    pub start_dir: Vec3,
    pub show_mesh: bool,
}

impl KiRevoluteJoint {
    pub fn get_angle(&self, tr: &Transform) -> f32 {
        let sign = {
            let dir = self.start_dir.cross(tr.up());
            if dir.length() > 0.01 && dir.dot(tr.right()) < 0.01 {
                -1.
            } else {
                1.
            }
        };
        sign * self.start_dir.angle_between(tr.up())
    }
}

#[derive(Component, Reflect)]
pub struct KiSphericalJoint {
    pub start_rot: Quat,
    pub show_mesh: bool,
}
