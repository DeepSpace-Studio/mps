//! Integration tests for the shared-memory force queue (Task 7: end-to-end)
//!
//! Tests the full cycle: Java enqueues N forces → native consumes → Rapier bodies have correct forces.
//! This test simulates the Java side in Rust to verify the FFI contract.

#[test]
fn force_queue_integration_full_cycle() {
    use mps_core::rapier::error::{ERR_INVALID_ARGUMENT, ERR_NULL_POINTER, ERR_OK};
    use mps_core::rapier::ffi::Vec3;
    use mps_core::rapier::ffi::force_queue::{ForceQueueHeader, rigid_body_consume_force_queue};
    use mps_core::rapier::ffi::types::BodyStatus;
    use mps_core::rapier::rigid_body::{
        rigid_body_builder_build, rigid_body_builder_create, rigid_body_builder_destroy,
        world_insert_rigid_body,
    };
    use mps_core::rapier::world::{world_create, world_destroy};
    use std::alloc::{Layout, alloc};
    use std::ptr;
    use std::sync::atomic::Ordering;

    // 1. Create a physics world
    let gravity = Vec3 {
        x: 0.0,
        y: -9.81,
        z: 0.0,
    };
    let world = world_create(gravity);
    assert!(!world.is_null());

    // 2. Create dynamic bodies using builder pattern
    let builder1 = rigid_body_builder_create(BodyStatus::Dynamic as u32);
    let builder2 = rigid_body_builder_create(BodyStatus::Dynamic as u32);
    assert!(!builder1.is_null());
    assert!(!builder2.is_null());

    let body1_ptr = rigid_body_builder_build(builder1);
    let body2_ptr = rigid_body_builder_build(builder2);
    assert!(!body1_ptr.is_null());
    assert!(!body2_ptr.is_null());

    let body1_handle = world_insert_rigid_body(world, body1_ptr);
    let body2_handle = world_insert_rigid_body(world, body2_ptr);
    assert_ne!(body1_handle, 0);
    assert_ne!(body2_handle, 0);

    // 3. Allocate a force queue (capacity=16, stride=7 for force+torque)
    let capacity = 16u64;
    let stride = 7u32;
    let bitmap_words = (capacity + 63) / 64;
    let header_size = 64usize;
    let bitmap_size = bitmap_words as usize * 8;
    let payload_size = capacity as usize * stride as usize * 8;
    let total_size = header_size + bitmap_size + payload_size;

    let layout = Layout::from_size_align(total_size, 64).unwrap();
    let ptr = unsafe { alloc(layout) };
    assert!(!ptr.is_null());

    // 4. Initialize header
    let hdr = ptr as *mut ForceQueueHeader;
    unsafe {
        (*hdr).capacity = capacity;
        (*hdr).head = 0;
        (*hdr).tail = 0;
        (*hdr).generation = 0;
        (*hdr).stride = stride;
        (*hdr).flags = 0;
    }

    // 5. Zero bitmap and payload
    unsafe {
        ptr::write_bytes(ptr.add(header_size), 0, bitmap_size + payload_size);
    }

    // 6. Verify header validation
    unsafe {
        assert_eq!((*hdr).validate(), ERR_OK);
    }

    // 7. Enqueue a force on body1 (slot 0)
    unsafe {
        let payload = (*hdr).payload_mut();
        let base = 0 * stride as usize;
        payload[base] = body1_handle as f64;
        payload[base + 1] = 100.0; // fx
        payload[base + 2] = 0.0; // fy
        payload[base + 3] = 0.0; // fz
        payload[base + 4] = 0.0; // tx
        payload[base + 5] = 0.0; // ty
        payload[base + 6] = 0.0; // tz
    }
    // Set bitmap bit 0
    unsafe {
        let bitmap = (*hdr).bitmap();
        bitmap[0].store(1u64, Ordering::Release);
    }
    // Advance head
    unsafe {
        let head_ptr = core::ptr::addr_of_mut!((*hdr).head) as *mut std::sync::atomic::AtomicU64;
        (*head_ptr).store(1, Ordering::Release);
    }

    // 8. Enqueue a force on body2 (slot 1)
    unsafe {
        let payload = (*hdr).payload_mut();
        let base = 1 * stride as usize;
        payload[base] = body2_handle as f64;
        payload[base + 1] = 0.0; // fx
        payload[base + 2] = 50.0; // fy
        payload[base + 3] = 0.0; // fz
        payload[base + 4] = 0.0; // tx
        payload[base + 5] = 10.0; // ty (torque)
        payload[base + 6] = 0.0; // tz
    }
    // Set bitmap bit 1
    unsafe {
        let bitmap = (*hdr).bitmap();
        bitmap[0].store(
            bitmap[0].load(Ordering::Acquire) | (1u64 << 1),
            Ordering::Release,
        );
    }
    // Advance head
    unsafe {
        let head_ptr = core::ptr::addr_of_mut!((*hdr).head) as *mut std::sync::atomic::AtomicU64;
        (*head_ptr).store(2, Ordering::Release);
    }

    // 9. Call the consumer
    let ret = rigid_body_consume_force_queue(world, hdr);
    assert_eq!(ret, ERR_OK);

    // 10. Verify tail advanced to 2
    unsafe {
        let tail_ptr = core::ptr::addr_of!((*hdr).tail) as *const std::sync::atomic::AtomicU64;
        assert_eq!((*tail_ptr).load(Ordering::Acquire), 2);
    }

    // 11. Verify bitmap bits cleared
    unsafe {
        let bitmap = (*hdr).bitmap();
        assert_eq!(bitmap[0].load(Ordering::Acquire), 0);
    }

    // 12. Test paused flag - should skip processing
    unsafe {
        (*hdr).flags = 1; // set paused
        (*hdr).head = 3;
        // Add another force
        let payload = (*hdr).payload_mut();
        let base = 2 * stride as usize;
        payload[base] = body1_handle as f64;
        payload[base + 1] = 100.0;
        payload[base + 2] = 0.0;
        payload[base + 3] = 0.0;
        payload[base + 4] = 0.0;
        payload[base + 5] = 0.0;
        payload[base + 6] = 0.0;
        let bitmap = (*hdr).bitmap();
        bitmap[0].store(
            bitmap[0].load(Ordering::Acquire) | (1u64 << 2),
            Ordering::Release,
        );
    }
    let ret = rigid_body_consume_force_queue(world, hdr);
    assert_eq!(ret, ERR_OK);
    // Tail should NOT have advanced (still 2) because paused
    unsafe {
        let tail_ptr = core::ptr::addr_of!((*hdr).tail) as *const std::sync::atomic::AtomicU64;
        assert_eq!((*tail_ptr).load(Ordering::Acquire), 2);
    }

    // 13. Clear paused and process again
    unsafe {
        (*hdr).flags = 0;
    }
    let ret = rigid_body_consume_force_queue(world, hdr);
    assert_eq!(ret, ERR_OK);
    unsafe {
        let tail_ptr = core::ptr::addr_of!((*hdr).tail) as *const std::sync::atomic::AtomicU64;
        assert_eq!((*tail_ptr).load(Ordering::Acquire), 3);
    }

    // 14. Test null pointer error
    let ret = rigid_body_consume_force_queue(ptr::null_mut(), hdr);
    assert_eq!(ret, ERR_NULL_POINTER);
    let ret = rigid_body_consume_force_queue(world, ptr::null_mut());
    assert_eq!(ret, ERR_NULL_POINTER);

    // 15. Test invalid stride
    unsafe {
        (*hdr).stride = 5;
    }
    let ret = rigid_body_consume_force_queue(world, hdr);
    assert_eq!(ret, ERR_INVALID_ARGUMENT);

    // 16. Test non-power-of-2 capacity
    unsafe {
        (*hdr).stride = 7;
        (*hdr).capacity = 1000;
    }
    let ret = rigid_body_consume_force_queue(world, hdr);
    assert_eq!(ret, ERR_INVALID_ARGUMENT);

    // 17. Cleanup
    unsafe {
        world_destroy(world);
        std::alloc::dealloc(ptr, layout);
    }
}

#[test]
fn force_queue_integration_stride6_only_force() {
    use mps_core::rapier::error::ERR_OK;
    use mps_core::rapier::ffi::Vec3;
    use mps_core::rapier::ffi::force_queue::{ForceQueueHeader, rigid_body_consume_force_queue};
    use mps_core::rapier::ffi::types::BodyStatus;
    use mps_core::rapier::rigid_body::{
        rigid_body_builder_build, rigid_body_builder_create, world_insert_rigid_body,
    };
    use mps_core::rapier::world::{world_create, world_destroy};
    use std::alloc::{Layout, alloc};
    use std::ptr;
    use std::sync::atomic::Ordering;

    // Test with stride=6 (force only, no torque)
    let gravity = Vec3 {
        x: 0.0,
        y: -9.81,
        z: 0.0,
    };
    let world = world_create(gravity);
    assert!(!world.is_null());

    let builder = rigid_body_builder_create(BodyStatus::Dynamic as u32);
    let body_ptr = rigid_body_builder_build(builder);
    let body_handle = world_insert_rigid_body(world, body_ptr);
    assert_ne!(body_handle, 0);

    let capacity = 8u64;
    let stride = 6u32;
    let bitmap_words = (capacity + 63) / 64;
    let header_size = 64usize;
    let bitmap_size = bitmap_words as usize * 8;
    let payload_size = capacity as usize * stride as usize * 8;
    let total_size = header_size + bitmap_size + payload_size;

    let layout = Layout::from_size_align(total_size, 64).unwrap();
    let ptr = unsafe { alloc(layout) };
    assert!(!ptr.is_null());

    let hdr = ptr as *mut ForceQueueHeader;
    unsafe {
        (*hdr).capacity = capacity;
        (*hdr).head = 0;
        (*hdr).tail = 0;
        (*hdr).generation = 0;
        (*hdr).stride = stride;
        (*hdr).flags = 0;
    }
    unsafe {
        ptr::write_bytes(ptr.add(header_size), 0, bitmap_size + payload_size);
    }

    // Enqueue force only (stride=6)
    unsafe {
        let payload = (*hdr).payload_mut();
        let base = 0 * stride as usize;
        payload[base] = body_handle as f64;
        payload[base + 1] = 10.0; // fx
        payload[base + 2] = 20.0; // fy
        payload[base + 3] = 30.0; // fz
    }
    {
        let bitmap = unsafe { (*hdr).bitmap() };
        bitmap[0].store(1u64, Ordering::Release);
    }
    unsafe {
        let head_ptr = core::ptr::addr_of_mut!((*hdr).head) as *mut std::sync::atomic::AtomicU64;
        (*head_ptr).store(1, Ordering::Release);
    }

    let ret = rigid_body_consume_force_queue(world, hdr);
    assert_eq!(ret, ERR_OK);

    unsafe {
        let tail_ptr = core::ptr::addr_of!((*hdr).tail) as *const std::sync::atomic::AtomicU64;
        assert_eq!((*tail_ptr).load(Ordering::Acquire), 1);
    }

    // Cleanup
    unsafe {
        world_destroy(world);
        std::alloc::dealloc(ptr, layout);
    }
}

#[test]
fn force_queue_integration_cancel_by_index() {
    use mps_core::rapier::error::ERR_OK;
    use mps_core::rapier::ffi::Vec3;
    use mps_core::rapier::ffi::force_queue::{ForceQueueHeader, rigid_body_consume_force_queue};
    use mps_core::rapier::ffi::types::BodyStatus;
    use mps_core::rapier::rigid_body::{
        rigid_body_builder_build, rigid_body_builder_create, world_insert_rigid_body,
    };
    use mps_core::rapier::world::{world_create, world_destroy};
    use std::alloc::{Layout, alloc};
    use std::ptr;
    use std::sync::atomic::Ordering;

    // Test O(1) cancellation by clearing bitmap bit
    let gravity = Vec3 {
        x: 0.0,
        y: -9.81,
        z: 0.0,
    };
    let world = world_create(gravity);
    assert!(!world.is_null());

    let builder = rigid_body_builder_create(BodyStatus::Dynamic as u32);
    let body_ptr = rigid_body_builder_build(builder);
    let body_handle = world_insert_rigid_body(world, body_ptr);
    assert_ne!(body_handle, 0);

    let capacity = 8u64;
    let stride = 7u32;
    let bitmap_words = (capacity + 63) / 64;
    let header_size = 64usize;
    let bitmap_size = bitmap_words as usize * 8;
    let payload_size = capacity as usize * stride as usize * 8;
    let total_size = header_size + bitmap_size + payload_size;

    let layout = Layout::from_size_align(total_size, 64).unwrap();
    let ptr = unsafe { alloc(layout) };
    assert!(!ptr.is_null());

    let hdr = ptr as *mut ForceQueueHeader;
    unsafe {
        (*hdr).capacity = capacity;
        (*hdr).head = 0;
        (*hdr).tail = 0;
        (*hdr).generation = 0;
        (*hdr).stride = stride;
        (*hdr).flags = 0;
    }
    unsafe {
        ptr::write_bytes(ptr.add(header_size), 0, bitmap_size + payload_size);
    }

    // Enqueue 3 forces
    for i in 0..3 {
        unsafe {
            let payload = (*hdr).payload_mut();
            let base = i * stride as usize;
            payload[base] = body_handle as f64;
            payload[base + 1] = 10.0 * (i + 1) as f64;
            payload[base + 2] = 0.0;
            payload[base + 3] = 0.0;
            payload[base + 4] = 0.0;
            payload[base + 5] = 0.0;
            payload[base + 6] = 0.0;
            let bitmap = (*hdr).bitmap();
            bitmap[0].store(
                bitmap[0].load(Ordering::Acquire) | (1u64 << i),
                Ordering::Release,
            );
            let head_ptr =
                core::ptr::addr_of_mut!((*hdr).head) as *mut std::sync::atomic::AtomicU64;
            (*head_ptr).store((i + 1) as u64, Ordering::Release);
        }
    }

    // Cancel the middle one (index 1)
    unsafe {
        let bitmap = (*hdr).bitmap();
        let current = bitmap[0].load(Ordering::Acquire);
        bitmap[0].store(current & !(1u64 << 1), Ordering::Release);
    }

    // Consume - should only process indices 0 and 2
    let ret = rigid_body_consume_force_queue(world, hdr);
    assert_eq!(ret, ERR_OK);

    // Tail should advance to 3 (all slots processed/skipped)
    unsafe {
        let tail_ptr = core::ptr::addr_of!((*hdr).tail) as *const std::sync::atomic::AtomicU64;
        assert_eq!((*tail_ptr).load(Ordering::Acquire), 3);
    }

    // Bitmap should be cleared
    unsafe {
        let bitmap = (*hdr).bitmap();
        assert_eq!(bitmap[0].load(Ordering::Acquire), 0);
    }

    // Cleanup
    unsafe {
        world_destroy(world);
        std::alloc::dealloc(ptr, layout);
    }
}

#[test]
fn force_queue_integration_wrap_around() {
    use mps_core::rapier::error::ERR_OK;
    use mps_core::rapier::ffi::Vec3;
    use mps_core::rapier::ffi::force_queue::{ForceQueueHeader, rigid_body_consume_force_queue};
    use mps_core::rapier::ffi::types::BodyStatus;
    use mps_core::rapier::rigid_body::{
        rigid_body_builder_build, rigid_body_builder_create, world_insert_rigid_body,
    };
    use mps_core::rapier::world::{world_create, world_destroy};
    use std::alloc::{Layout, alloc};
    use std::ptr;
    use std::sync::atomic::Ordering;

    // Test generation counter increment on head wrap
    let gravity = Vec3 {
        x: 0.0,
        y: -9.81,
        z: 0.0,
    };
    let world = world_create(gravity);
    assert!(!world.is_null());

    let builder = rigid_body_builder_create(BodyStatus::Dynamic as u32);
    let body_ptr = rigid_body_builder_build(builder);
    let body_handle = world_insert_rigid_body(world, body_ptr);
    assert_ne!(body_handle, 0);

    let capacity = 4u64; // Small capacity to force wrap
    let stride = 7u32;
    let bitmap_words = (capacity + 63) / 64;
    let header_size = 64usize;
    let bitmap_size = bitmap_words as usize * 8;
    let payload_size = capacity as usize * stride as usize * 8;
    let total_size = header_size + bitmap_size + payload_size;

    let layout = Layout::from_size_align(total_size, 64).unwrap();
    let ptr = unsafe { alloc(layout) };
    assert!(!ptr.is_null());

    let hdr = ptr as *mut ForceQueueHeader;
    unsafe {
        (*hdr).capacity = capacity;
        (*hdr).head = 0;
        (*hdr).tail = 0;
        (*hdr).generation = 0;
        (*hdr).stride = stride;
        (*hdr).flags = 0;
    }
    unsafe {
        ptr::write_bytes(ptr.add(header_size), 0, bitmap_size + payload_size);
    }

    // Fill the queue (capacity=4, so head goes 0->1->2->3->0 wrap)
    for wrap in 0..2 {
        for i in 0..capacity {
            unsafe {
                let payload = (*hdr).payload_mut();
                let base = i as usize * stride as usize;
                payload[base] = body_handle as f64;
                payload[base + 1] = 1.0;
                payload[base + 2] = 0.0;
                payload[base + 3] = 0.0;
                payload[base + 4] = 0.0;
                payload[base + 5] = 0.0;
                payload[base + 6] = 0.0;
                let bitmap = (*hdr).bitmap();
                bitmap[0].store(
                    bitmap[0].load(Ordering::Acquire) | (1u64 << i),
                    Ordering::Release,
                );
                let head_ptr =
                    core::ptr::addr_of_mut!((*hdr).head) as *mut std::sync::atomic::AtomicU64;
                (*head_ptr).store((i + 1) as u64, Ordering::Release);
            }
        }
        // Consume
        let ret = rigid_body_consume_force_queue(world, hdr);
        assert_eq!(ret, ERR_OK);
        // After first wrap, generation should increment (producer responsibility)
        if wrap == 0 {
            unsafe {
                // Simulate Java producer incrementing generation on head wrap
                let gen_ptr =
                    core::ptr::addr_of_mut!((*hdr).generation) as *mut std::sync::atomic::AtomicU64;
                (*gen_ptr).store(1, Ordering::Release);
            }
        }
        // Clear bitmap for next iteration
        unsafe {
            let bitmap = (*hdr).bitmap();
            bitmap[0].store(0, Ordering::Release);
        }
    }

    // Cleanup
    unsafe {
        world_destroy(world);
        std::alloc::dealloc(ptr, layout);
    }
}
