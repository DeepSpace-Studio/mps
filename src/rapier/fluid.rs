use rapier3d::prelude::Vector;

use crate::rapier::error::{ERR_INVALID_ARGUMENT, ERR_NOT_FOUND, ERR_NULL_POINTER, clear_error, set_error};
use crate::rapier::ffi::{
    Bool, FluidForceReport, FluidVolume, RigidBodyHandleRaw, Vec3, WorldHandle,
    unpack_rigid_body_handle, vec3_finite, vec3_from_rapier, vec3_to_rapier,
};

fn fluid_valid(fluid: FluidVolume) -> bool {
    vec3_finite(fluid.center)
        && vec3_finite(fluid.half_extents)
        && vec3_finite(fluid.flow_velocity)
        && vec3_finite(fluid.gravity)
        && fluid.half_extents.x >= 0.0
        && fluid.half_extents.y >= 0.0
        && fluid.half_extents.z >= 0.0
        && fluid.density.is_finite()
        && fluid.density >= 0.0
        && fluid.linear_drag.is_finite()
        && fluid.linear_drag >= 0.0
        && fluid.quadratic_drag.is_finite()
        && fluid.quadratic_drag >= 0.0
        && fluid.angular_drag.is_finite()
        && fluid.angular_drag >= 0.0
}

fn finite_positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn submerged_fraction_aabb(body_center: Vec3, body_half_extents: Vec3, fluid: FluidVolume) -> f64 {
    let body_min_y = body_center.y - body_half_extents.y;
    let body_max_y = body_center.y + body_half_extents.y;
    let fluid_min_y = fluid.center.y - fluid.half_extents.y;
    let fluid_max_y = fluid.center.y + fluid.half_extents.y;
    let body_height = body_max_y - body_min_y;
    if body_height <= 0.0 {
        return 0.0;
    }
    let overlap = body_max_y.min(fluid_max_y) - body_min_y.max(fluid_min_y);
    clamp01(overlap / body_height)
}

fn compute_fluid_forces(
    fluid: FluidVolume,
    body_center: Vec3,
    body_half_extents: Vec3,
    body_volume: f64,
    body_linvel: Vec3,
    body_angvel: Vec3,
) -> Option<FluidForceReport> {
    if !fluid_valid(fluid)
        || !vec3_finite(body_center)
        || !vec3_finite(body_half_extents)
        || !vec3_finite(body_linvel)
        || !vec3_finite(body_angvel)
        || !finite_positive(body_volume)
        || body_half_extents.x < 0.0
        || body_half_extents.y < 0.0
        || body_half_extents.z < 0.0
    {
        return None;
    }

    let submerged_fraction = submerged_fraction_aabb(body_center, body_half_extents, fluid);
    if submerged_fraction <= 0.0 || fluid.density <= 0.0 {
        return Some(FluidForceReport::default());
    }

    let displaced_volume = body_volume * submerged_fraction;
    let gravity = vec3_to_rapier(fluid.gravity);
    let relative_velocity = vec3_to_rapier(fluid.flow_velocity) - vec3_to_rapier(body_linvel);
    let speed = relative_velocity.length_squared().sqrt();
    let drag_force = if speed > 1.0e-12 {
        let drag_scale = submerged_fraction
            * (fluid.linear_drag * speed + fluid.quadratic_drag * speed * speed);
        relative_velocity / speed * drag_scale
    } else {
        Vector::ZERO
    };
    let buoyancy_force = -gravity * (fluid.density * displaced_volume);
    let angular_damping_torque = -vec3_to_rapier(body_angvel) * (fluid.angular_drag * submerged_fraction);
    let total_force = buoyancy_force + drag_force;

    Some(FluidForceReport {
        buoyancy_force: vec3_from_rapier(buoyancy_force),
        drag_force: vec3_from_rapier(drag_force),
        angular_damping_torque: vec3_from_rapier(angular_damping_torque),
        total_force: vec3_from_rapier(total_force),
        total_torque: vec3_from_rapier(angular_damping_torque),
        submerged_fraction,
        displaced_volume,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn fluid_estimate_aabb_forces(
    fluid: FluidVolume,
    body_center: Vec3,
    body_half_extents: Vec3,
    body_volume: f64,
    body_linvel: Vec3,
    body_angvel: Vec3,
    out_report: *mut FluidForceReport,
) -> Bool {
    let Some(report) = compute_fluid_forces(
        fluid,
        body_center,
        body_half_extents,
        body_volume,
        body_linvel,
        body_angvel,
    ) else {
        set_error(ERR_INVALID_ARGUMENT, "invalid fluid force parameters");
        return Bool::FALSE;
    };

    if let Some(out_report) = unsafe { out_report.as_mut() } {
        *out_report = report;
    }
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn fluid_apply_aabb_forces(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    fluid: FluidVolume,
    body_half_extents: Vec3,
    body_volume: f64,
    wake_up: Bool,
    out_report: *mut FluidForceReport,
) -> Bool {
    let Some(world) = (unsafe { world.as_mut() }) else {
        set_error(ERR_NULL_POINTER, "world is null");
        return Bool::FALSE;
    };
    let Some(body) = world
        .inner
        .bodies
        .get_mut(unpack_rigid_body_handle(body_handle))
    else {
        set_error(ERR_NOT_FOUND, "body was not found");
        return Bool::FALSE;
    };

    let body_center = vec3_from_rapier(body.center_of_mass());
    let body_linvel = vec3_from_rapier(body.linvel());
    let body_angvel = vec3_from_rapier(body.angvel());
    let Some(report) = compute_fluid_forces(
        fluid,
        body_center,
        body_half_extents,
        body_volume,
        body_linvel,
        body_angvel,
    ) else {
        set_error(ERR_INVALID_ARGUMENT, "invalid fluid force parameters");
        return Bool::FALSE;
    };

    body.add_force(vec3_to_rapier(report.total_force), wake_up.0 != 0);
    body.add_torque(vec3_to_rapier(report.total_torque), wake_up.0 != 0);
    if let Some(out_report) = unsafe { out_report.as_mut() } {
        *out_report = report;
    }
    clear_error();
    Bool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn fluid_apply_aabb_forces_flag(
    world: *mut WorldHandle,
    body_handle: RigidBodyHandleRaw,
    fluid: FluidVolume,
    body_half_extents: Vec3,
    body_volume: f64,
    wake_up: Bool,
    out_report: *mut FluidForceReport,
) -> u8 {
    fluid_apply_aabb_forces(
        world,
        body_handle,
        fluid,
        body_half_extents,
        body_volume,
        wake_up,
        out_report,
    )
    .0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rapier::ffi::BodyStatus;

    fn water() -> FluidVolume {
        FluidVolume {
            center: Vec3::default(),
            half_extents: Vec3 {
                x: 10.0,
                y: 10.0,
                z: 10.0,
            },
            density: 1000.0,
            linear_drag: 2.0,
            quadratic_drag: 0.5,
            angular_drag: 0.2,
            flow_velocity: Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            gravity: Vec3 {
                x: 0.0,
                y: -9.81,
                z: 0.0,
            },
        }
    }

    #[test]
    fn estimates_buoyancy_and_drag() {
        let mut report = FluidForceReport::default();
        assert_eq!(
            fluid_estimate_aabb_forces(
                water(),
                Vec3::default(),
                Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                },
                1.0,
                Vec3::default(),
                Vec3::default(),
                &mut report,
            ),
            Bool::TRUE
        );
        assert_eq!(report.submerged_fraction, 1.0);
        assert!(report.buoyancy_force.y > 0.0);
        assert!(report.drag_force.x > 0.0);
    }

    #[test]
    fn applies_fluid_force_to_body() {
        let world = crate::rapier::world::world_create(Vec3::default());
        let builder = crate::rapier::rigid_body::rigid_body_builder_create(
            BodyStatus::Dynamic as u32,
        );
        crate::rapier::rigid_body::rigid_body_builder_set_additional_mass(builder, 1.0);
        let body = crate::rapier::rigid_body::rigid_body_builder_build(builder);
        let handle = crate::rapier::rigid_body::world_insert_rigid_body(world, body);
        let mut report = FluidForceReport::default();

        assert_eq!(
            fluid_apply_aabb_forces(
                world,
                handle,
                water(),
                Vec3 {
                    x: 0.5,
                    y: 0.5,
                    z: 0.5,
                },
                1.0,
                Bool::TRUE,
                &mut report,
            ),
            Bool::TRUE
        );
        assert!(report.total_force.y > 0.0);
        crate::rapier::world::world_step(world, 1.0 / 60.0);
        let velocity = crate::rapier::rigid_body::rigid_body_get_linvel(world, handle);
        assert!(velocity.y > 0.0);
        crate::rapier::world::world_destroy(world);
    }
}
