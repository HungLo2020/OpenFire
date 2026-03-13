use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct CollisionBox {
    pub half_extents: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub struct RaycastHit {
    pub world_position: Vec3,
    pub distance: f32,
}

pub fn raycast_collision_boxes<'w, 's>(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    ignored_entity: Option<Entity>,
    colliders: impl Iterator<Item = (Entity, &'w CollisionBox, &'s GlobalTransform)>,
) -> Option<RaycastHit> {
    if max_distance <= 0.0 {
        return None;
    }

    let world_direction = direction.normalize_or_zero();
    if world_direction == Vec3::ZERO {
        return None;
    }

    let mut best_hit: Option<RaycastHit> = None;

    for (entity, collider, transform) in colliders {
        if ignored_entity.is_some_and(|ignored| ignored == entity) {
            continue;
        }

        let world_from_local = transform.affine();
        let local_from_world = world_from_local.inverse();

        let local_origin = local_from_world.transform_point3(origin);
        let local_direction = local_from_world.transform_vector3(world_direction);

        let Some(local_t) = intersect_ray_aabb(local_origin, local_direction, collider.half_extents) else {
            continue;
        };

        if !(0.0..=max_distance).contains(&local_t) {
            continue;
        }

        let hit_world = origin + world_direction * local_t;

        match best_hit {
            Some(hit) if local_t >= hit.distance => {}
            _ => {
                best_hit = Some(RaycastHit {
                    world_position: hit_world,
                    distance: local_t,
                });
            }
        }
    }

    best_hit
}

fn intersect_ray_aabb(origin: Vec3, direction: Vec3, half_extents: Vec3) -> Option<f32> {
    let min = -half_extents;
    let max = half_extents;

    let mut t_min: f32 = 0.0;
    let mut t_max: f32 = f32::INFINITY;
    let epsilon = 1e-6;

    for axis in 0..3 {
        let origin_axis = origin[axis];
        let direction_axis = direction[axis];
        let min_axis = min[axis];
        let max_axis = max[axis];

        if direction_axis.abs() <= epsilon {
            if origin_axis < min_axis || origin_axis > max_axis {
                return None;
            }
            continue;
        }

        let inv_direction = 1.0 / direction_axis;
        let mut t1 = (min_axis - origin_axis) * inv_direction;
        let mut t2 = (max_axis - origin_axis) * inv_direction;

        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }

        t_min = t_min.max(t1);
        t_max = t_max.min(t2);

        if t_min > t_max {
            return None;
        }
    }

    Some(t_min)
}