use bevy::prelude::*;
use bevy_rapier3d::{na::Isometry, prelude::*};

pub struct AutoColliderPlugin;

impl Plugin for AutoColliderPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AutoCollider>()
            .register_type::<AutoColliderMesh>()
            .register_type::<AutoColliderRoot>()
            .add_system(update_auto_colliders);
    }
}

#[derive(Component, Reflect)]
pub struct AutoCollider {
    pub mesh: Entity,
    pub update_transform: bool,
}

#[derive(Component, Reflect)]
pub struct AutoColliderMesh {
    pub collider: Entity,
}

#[derive(Component, Reflect)]
pub struct AutoColliderRoot;

fn update_auto_colliders(
    mut rapier: ResMut<RapierContext>,
    q_root: Query<&GlobalTransform, With<AutoColliderRoot>>,
    mut q_coll: Query<(Entity, &Parent, &AutoCollider, &mut Transform)>,
    q_mesh: Query<&GlobalTransform, (Without<AutoColliderRoot>, With<AutoColliderMesh>)>,
    mut cmd: Commands,
) {
    for (coll_ent, coll_parent, auto_coll, mut coll_tr) in &mut q_coll {
        if auto_coll.update_transform {
            let root_ent = coll_parent.get();
            if let (Ok(root_gtr), Ok(mesh_gtr)) = (q_root.get(root_ent), q_mesh.get(auto_coll.mesh))
            {
                let mesh_gpos = mesh_gtr.transform_point(Vec3::ZERO);
                let obj_inv = root_gtr.affine().inverse();
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

                cmd.entity(coll_ent).remove::<ColliderDisabled>();
            }
        }
    }
}
