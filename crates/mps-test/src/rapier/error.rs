#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use mps_core::rapier::error::{
        ERR_INVALID_ARGUMENT, ERR_NULL_POINTER, ERR_OK, ERR_CAPACITY, last_error_clear,
        last_error_code, last_error_message,
    };
    use mps_core::rapier::ffi::{Bool, QueryFilterDesc, Vec3};
    use mps_core::rapier::query::query_cast_rays;
    use mps_core::rapier::world::{
        world_create, world_destroy, world_get_integration_parameters,
        world_set_integration_parameters,
    };

    fn last_message() -> String {
        let ptr = last_error_message();
        assert!(!ptr.is_null());
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned()
    }

    #[test]
    fn initial_state_is_ok() {
        // Each test runs on a fresh thread, so the thread-local error slot
        // starts in the default state.
        assert_eq!(last_error_code(), ERR_OK);
        assert_eq!(last_message(), "ok");
    }

    #[test]
    fn null_world_reports_null_pointer() {
        let written = query_cast_rays(
            std::ptr::null(),
            std::ptr::null(),
            0,
            1.0,
            Bool::FALSE,
            QueryFilterDesc::default(),
            std::ptr::null_mut(),
            0,
        );
        assert_eq!(written, 0);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        assert!(last_message().contains("world"));
    }

    #[test]
    fn null_ray_buffer_reports_null_pointer() {
        let world = world_create(Vec3::default());
        let written = query_cast_rays(
            world,
            std::ptr::null(),
            1,
            1.0,
            Bool::FALSE,
            QueryFilterDesc::default(),
            std::ptr::null_mut(),
            1,
        );
        assert_eq!(written, 0);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);
        world_destroy(world);
    }

    #[test]
    fn small_output_capacity_reports_capacity() {
        let world = world_create(Vec3::default());
        let mut out = [0.0f64; 2];
        let written = world_get_integration_parameters(world, out.as_mut_ptr(), 2);
        assert_eq!(written, 0);
        assert_eq!(last_error_code(), ERR_CAPACITY);
        world_destroy(world);
    }

    #[test]
    fn invalid_argument_reports_invalid_argument() {
        let world = world_create(Vec3::default());
        let ok = world_set_integration_parameters(world, -1.0, 4, 1);
        assert_eq!(ok, Bool::FALSE);
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        world_destroy(world);
    }

    #[test]
    fn success_clears_previous_error() {
        let world = world_create(Vec3::default());
        // Trigger an error first.
        assert_eq!(
            world_set_integration_parameters(world, -1.0, 4, 1),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        // A successful call resets the slot.
        assert_eq!(
            world_set_integration_parameters(world, 1.0 / 60.0, 4, 1),
            Bool::TRUE
        );
        assert_eq!(last_error_code(), ERR_OK);
        assert_eq!(last_message(), "ok");
        world_destroy(world);
    }

    #[test]
    fn clear_resets_state() {
        let world = world_create(Vec3::default());
        assert_eq!(
            world_set_integration_parameters(world, -1.0, 4, 1),
            Bool::FALSE
        );
        assert_ne!(last_error_code(), ERR_OK);

        last_error_clear();
        assert_eq!(last_error_code(), ERR_OK);
        assert_eq!(last_message(), "ok");
        world_destroy(world);
    }

    #[test]
    fn error_state_is_thread_local() {
        let world = world_create(Vec3::default());
        assert_eq!(
            world_set_integration_parameters(world, -1.0, 4, 1),
            Bool::FALSE
        );
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);

        // Another thread must observe a fresh, untouched error slot.
        let handle = std::thread::spawn(|| {
            assert_eq!(last_error_code(), ERR_OK);
            assert_eq!(last_message(), "ok");
        });
        handle.join().expect("thread panicked");

        // The spawning thread still sees its own error.
        assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT);
        world_destroy(world);
    }
}
