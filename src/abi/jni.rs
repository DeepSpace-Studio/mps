use std::ffi::c_void;

use crate::ffi::{
    RcBodyStatus, RcRigidBodyBuilderHandle, RcRigidBodyHandle, RcVec3, RcWorldHandle,
};

type JNIEnv = *mut c_void;
type JClass = *mut c_void;
type JDouble = f64;
type JInt = i32;
type JLong = i64;

fn ptr_to_jlong<T>(value: *mut T) -> JLong {
    value as isize as JLong
}

fn jlong_to_mut<T>(value: JLong) -> *mut T {
    value as isize as *mut T
}

fn jlong_to_const<T>(value: JLong) -> *const T {
    value as isize as *const T
}

fn vec3(x: JDouble, y: JDouble, z: JDouble) -> RcVec3 {
    RcVec3 { x, y, z }
}

fn body_status(value: JInt) -> RcBodyStatus {
    match value {
        0 => RcBodyStatus::Dynamic,
        1 => RcBodyStatus::Fixed,
        2 => RcBodyStatus::KinematicPositionBased,
        3 => RcBodyStatus::KinematicVelocityBased,
        _ => RcBodyStatus::Fixed,
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_abiVersion(
    _env: JNIEnv,
    _class: JClass,
) -> JInt {
    crate::abi::ffm::rc_abi_version() as JInt
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_worldCreate(
    _env: JNIEnv,
    _class: JClass,
    gravity_x: JDouble,
    gravity_y: JDouble,
    gravity_z: JDouble,
) -> JLong {
    ptr_to_jlong(crate::world::rc_world_create(vec3(
        gravity_x, gravity_y, gravity_z,
    )))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_worldDestroy(
    _env: JNIEnv,
    _class: JClass,
    world: JLong,
) {
    crate::world::rc_world_destroy(jlong_to_mut::<RcWorldHandle>(world));
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_worldStep(
    _env: JNIEnv,
    _class: JClass,
    world: JLong,
    delta_seconds: JDouble,
) {
    crate::world::rc_world_step(jlong_to_mut::<RcWorldHandle>(world), delta_seconds);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_worldSetGravity(
    _env: JNIEnv,
    _class: JClass,
    world: JLong,
    gravity_x: JDouble,
    gravity_y: JDouble,
    gravity_z: JDouble,
) {
    crate::world::rc_world_set_gravity(
        jlong_to_mut::<RcWorldHandle>(world),
        vec3(gravity_x, gravity_y, gravity_z),
    );
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_worldGetGravityX(
    _env: JNIEnv,
    _class: JClass,
    world: JLong,
) -> JDouble {
    crate::world::rc_world_get_gravity(jlong_to_const::<RcWorldHandle>(world)).x
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_worldGetGravityY(
    _env: JNIEnv,
    _class: JClass,
    world: JLong,
) -> JDouble {
    crate::world::rc_world_get_gravity(jlong_to_const::<RcWorldHandle>(world)).y
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_worldGetGravityZ(
    _env: JNIEnv,
    _class: JClass,
    world: JLong,
) -> JDouble {
    crate::world::rc_world_get_gravity(jlong_to_const::<RcWorldHandle>(world)).z
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_worldDynamicBodySnapshotCount(
    _env: JNIEnv,
    _class: JClass,
    world: JLong,
) -> JInt {
    crate::world::rc_world_dynamic_body_snapshot_count(jlong_to_const::<RcWorldHandle>(world))
        as JInt
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_rigidBodyBuilderCreate(
    _env: JNIEnv,
    _class: JClass,
    status: JInt,
) -> JLong {
    ptr_to_jlong(crate::rigid_body::rc_rigid_body_builder_create(
        body_status(status),
    ))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_rigidBodyBuilderDestroy(
    _env: JNIEnv,
    _class: JClass,
    builder: JLong,
) {
    crate::rigid_body::rc_rigid_body_builder_destroy(jlong_to_mut::<RcRigidBodyBuilderHandle>(
        builder,
    ));
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_rigidBodyBuilderSetTranslation(
    _env: JNIEnv,
    _class: JClass,
    builder: JLong,
    x: JDouble,
    y: JDouble,
    z: JDouble,
) {
    crate::rigid_body::rc_rigid_body_builder_set_translation(
        jlong_to_mut::<RcRigidBodyBuilderHandle>(builder),
        vec3(x, y, z),
    );
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_worldInsertRigidBody(
    _env: JNIEnv,
    _class: JClass,
    world: JLong,
    builder: JLong,
) -> JLong {
    crate::rigid_body::rc_world_insert_rigid_body(
        jlong_to_mut::<RcWorldHandle>(world),
        jlong_to_mut::<RcRigidBodyBuilderHandle>(builder),
    ) as JLong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_rigidBodyGetTranslationX(
    _env: JNIEnv,
    _class: JClass,
    world: JLong,
    body: JLong,
) -> JDouble {
    crate::rigid_body::rc_rigid_body_get_translation(
        jlong_to_const::<RcWorldHandle>(world),
        body as RcRigidBodyHandle,
    )
    .x
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_rigidBodyGetTranslationY(
    _env: JNIEnv,
    _class: JClass,
    world: JLong,
    body: JLong,
) -> JDouble {
    crate::rigid_body::rc_rigid_body_get_translation(
        jlong_to_const::<RcWorldHandle>(world),
        body as RcRigidBodyHandle,
    )
    .y
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_team_dove_rigidbody_RigidBodyNative_rigidBodyGetTranslationZ(
    _env: JNIEnv,
    _class: JClass,
    world: JLong,
    body: JLong,
) -> JDouble {
    crate::rigid_body::rc_rigid_body_get_translation(
        jlong_to_const::<RcWorldHandle>(world),
        body as RcRigidBodyHandle,
    )
    .z
}
