use bevy::prelude::*;
use bevy_rapier3d::{na::Isometry, prelude::*};

pub struct KinematicRigPlugin;

impl Plugin for KinematicRigPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KinematicRigCollider>()
            .register_type::<KinematicRigMesh>()
            .add_system(update_kinematic_colliders);
    }
}

#[derive(Component, Reflect)]
pub struct KinematicRigCollider {
    pub mesh: Entity,
    pub is_root: bool,
}

#[derive(Component, Reflect)]
pub struct KinematicRigMesh {
    pub collider: Entity,
}

fn update_kinematic_colliders(
    mut rapier: ResMut<RapierContext>,
    q_object: Query<&GlobalTransform, With<RigidBody>>,
    mut q_coll: Query<(Entity, &KinematicRigCollider, &mut Transform)>,
    q_mesh: Query<&GlobalTransform, (Without<RigidBody>, With<KinematicRigMesh>)>,
    q_parent: Query<&Parent>,
) {
    for (coll_ent, kcoll, mut coll_tr) in &mut q_coll {
        if !kcoll.is_root {
            let obj_ent = q_parent.iter_ancestors(coll_ent).next().unwrap();
            if let Ok(obj_gtr) = q_object.get(obj_ent) {
                if let Ok(mesh_gtr) = q_mesh.get(kcoll.mesh) {
                    let mesh_gpos = mesh_gtr.transform_point(Vec3::ZERO);
                    let obj_inv = obj_gtr.affine().inverse();
                    let mesh_obj_pos = obj_inv.transform_point3(mesh_gpos);
                    coll_tr.translation = mesh_obj_pos;

                    let (dir_x, dir_y, dir_z) = (mesh_gtr.right(), mesh_gtr.up(), mesh_gtr.back());
                    let (dir_x, dir_y, dir_z) = (
                        obj_inv.transform_vector3(dir_x).normalize(),
                        obj_inv.transform_vector3(dir_y).normalize(),
                        obj_inv.transform_vector3(dir_z).normalize(),
                    );
                    coll_tr.rotation = Quat::from_mat3(&Mat3::from_cols(dir_x, dir_y, dir_z));

                    // fix for bevy_rapier not auto syncing with the above
                    let h = { rapier.entity2collider().get(&coll_ent).unwrap().clone() };
                    let c = rapier.colliders.get_mut(h).unwrap();
                    c.set_position_wrt_parent(Isometry::from_parts(
                        mesh_obj_pos.into(),
                        coll_tr.rotation.into(),
                    ));
                }
            }
        }
    }
}
