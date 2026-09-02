#[cfg(test)]
mod tests {
    use mps_core::rapier::balloon::{BALLOON_MAX_PARTICLES, BalloonDesc, soft_balloon_create};
    use mps_core::rapier::error::{
        ERR_CAPACITY, ERR_INVALID_ARGUMENT, ERR_NULL_POINTER, ERR_OK, last_error_code,
    };
    use mps_core::rapier::ffi::{Bool, Vec3, WorldHandle};
    use mps_core::rapier::soft_body::{
        soft_body_particle_count, soft_body_read_particles, soft_body_read_surface_triangle_count,
        soft_body_set_damping, soft_body_set_pressure,
    };
    use mps_core::rapier::world::{world_create, world_destroy, world_step};

    const SENTINEL: u32 = u32::MAX;

    fn make_world() -> *mut WorldHandle {
        world_create(Vec3 {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        })
    }

    fn desc() -> BalloonDesc {
        BalloonDesc {
            rings: 4,
            segments: 8,
            center: Vec3 {
                x: 0.0,
                y: 3.0,
                z: 0.0,
            },
            radius: 0.5,
            particle_mass: 0.02,
            edge_compliance: 2e-3,
            pressure: 0.0,
            iterations: 8,
        }
    }

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    fn read_positions(world: *const WorldHandle, id: u32) -> Vec<Vec3> {
        let n = soft_body_particle_count(world, id);
        assert_ne!(n, u32::MAX, "balloon must exist");
        let mut pos = vec![Vec3::default(); n as usize];
        let read = soft_body_read_particles(world, id, pos.as_mut_ptr(), std::ptr::null_mut(), n);
        assert_eq!(read, n);
        pos
    }

    #[test]
    fn balloon_create_builds_closed_shell() {
        let world = make_world();
        let d = desc();
        let id = soft_balloon_create(world, d);
        assert_ne!(id, SENTINEL);
        assert_eq!(last_error_code(), ERR_OK);

        // Particle and triangle bookkeeping: rings·segments + 2 particles,
        // 2·(rings−1)·segments quad triangles + 2·segments pole cap triangles.
        let n = soft_body_particle_count(world, id);
        assert_eq!(n, d.rings * d.segments + 2);
        let tris = soft_body_read_surface_triangle_count(world, id);
        assert_eq!(
            tris,
            2 * (d.rings - 1) * d.segments + 2 * d.segments,
            "closed UV-sphere triangle count"
        );

        // Every particle starts exactly on the shell sphere.
        let pos = read_positions(world, id);
        for p in &pos {
            let dx = p.x - d.center.x;
            let dy = p.y - d.center.y;
            let dz = p.z - d.center.z;
            let r = (dx * dx + dy * dy + dz * dz).sqrt();
            assert!(
                (r - d.radius).abs() < 1e-9,
                "particle radius {r} != {}",
                d.radius
            );
        }
        world_destroy(world);
    }

    /// Mean shell radius measured from the shell's *current* centroid —
    /// translation-invariant, so free fall does not pollute the metric.
    fn mean_radius(world: *const WorldHandle, id: u32) -> f64 {
        let pos = read_positions(world, id);
        let n = pos.len();
        let mut c = v(0.0, 0.0, 0.0);
        for p in &pos {
            c = v(
                c.x + p.x / n as f64,
                c.y + p.y / n as f64,
                c.z + p.z / n as f64,
            );
        }
        let mut mean = 0.0;
        for p in &pos {
            let dx = p.x - c.x;
            let dy = p.y - c.y;
            let dz = p.z - c.z;
            mean += (dx * dx + dy * dy + dz * dz).sqrt() / n as f64;
        }
        mean
    }

    #[test]
    fn balloon_pressure_inflates_sphere() {
        let world = make_world();
        let mut d = desc();
        d.pressure = 300.0;
        let id = soft_balloon_create(world, d);
        assert_ne!(id, SENTINEL);
        assert_eq!(soft_body_set_damping(world, id, 0.05), Bool::TRUE);

        let r0 = mean_radius(world, id);
        for _ in 0..240 {
            world_step(world, 1.0 / 60.0);
        }
        let r1 = mean_radius(world, id);

        // All positions stay finite/bounded throughout the inflation.
        for p in read_positions(world, id) {
            assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
            assert!(p.y.abs() < 100.0);
        }
        assert!(
            r1 > r0 * 1.02,
            "pressurized balloon should inflate: mean radius {r0} → {r1}"
        );
        world_destroy(world);
    }

    #[test]
    fn balloon_unpressurized_free_fall_stays_coherent() {
        let world = make_world();
        let mut d = desc();
        d.pressure = 0.0; // no internal pressure
        let id = soft_balloon_create(world, d);
        assert_ne!(id, SENTINEL);
        assert_eq!(soft_body_set_damping(world, id, 0.05), Bool::TRUE);

        // A free-floating unpressurized shell in uniform gravity is stress-free
        // (equivalence principle): it free-falls intact — the shape spread must
        // stay at the spawn radius (no crumple without an ambient-pressure or
        // ground model) while the whole shell falls.
        let spread0 = mean_radius(world, id);
        for _ in 0..120 {
            world_step(world, 1.0 / 60.0);
        }
        let pos = read_positions(world, id);
        let spread1 = mean_radius(world, id);
        let mean_y = pos.iter().map(|p| p.y).sum::<f64>() / pos.len() as f64;
        for p in &pos {
            assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
            assert!(p.y.abs() < 200.0);
        }
        assert!(
            (spread1 - spread0).abs() < 0.02,
            "stress-free shell must keep its shape: spread {spread0} → {spread1}"
        );
        assert!(
            mean_y < d.center.y - 4.0,
            "shell must actually free-fall (damped): mean y {mean_y}"
        );
        world_destroy(world);
    }

    #[test]
    fn balloon_pressure_pump_vents_at_runtime() {
        let world = make_world();
        let mut d = desc();
        d.pressure = 0.0; // spawn deflated
        let id = soft_balloon_create(world, d);
        assert_ne!(id, SENTINEL);
        assert_eq!(soft_body_set_damping(world, id, 0.05), Bool::TRUE);

        let r_deflated = mean_radius(world, id);

        // Pump up mid-flight: soft skin + internal pressure → inflate.
        assert_eq!(soft_body_set_pressure(world, id, 300.0), Bool::TRUE);
        for _ in 0..240 {
            world_step(world, 1.0 / 60.0);
        }
        let r_pumped = mean_radius(world, id);
        assert!(
            r_pumped > r_deflated * 1.02,
            "pumping must inflate: {r_deflated} → {r_pumped}"
        );

        // Vent: the stretched edges pull the shell back toward rest.
        assert_eq!(soft_body_set_pressure(world, id, 0.0), Bool::TRUE);
        for _ in 0..240 {
            world_step(world, 1.0 / 60.0);
        }
        let r_vented = mean_radius(world, id);
        for p in read_positions(world, id) {
            assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
            assert!(p.y.abs() < 200.0);
        }
        assert!(
            r_vented < r_pumped - 0.005,
            "venting must let the stretched shell contract: {r_pumped} → {r_vented}"
        );
        world_destroy(world);
    }

    #[test]
    fn balloon_create_rejects_bad_params() {
        // Null world.
        assert_eq!(soft_balloon_create(std::ptr::null_mut(), desc()), SENTINEL);
        assert_eq!(last_error_code(), ERR_NULL_POINTER);

        let world = make_world();
        let cases: [(fn(&mut BalloonDesc), &str); 7] = [
            (|d| d.rings = 1, "rings < 2"),
            (|d| d.segments = 2, "segments < 3"),
            (|d| d.iterations = 0, "zero iterations"),
            (|d| d.radius = 0.0, "zero radius"),
            (|d| d.particle_mass = 0.0, "zero mass"),
            (|d| d.edge_compliance = -1e-6, "negative compliance"),
            (|d| d.pressure = -1.0, "negative pressure"),
        ];
        for (mutate, label) in cases {
            let mut d = desc();
            mutate(&mut d);
            assert_eq!(soft_balloon_create(world, d), SENTINEL, "case: {label}");
            assert_eq!(last_error_code(), ERR_INVALID_ARGUMENT, "case: {label}");
        }

        // Oversized shell → ERR_CAPACITY.
        let rings = BALLOON_MAX_PARTICLES / 4 + 1;
        let mut d = desc();
        d.rings = rings;
        d.segments = 4;
        assert_eq!(soft_balloon_create(world, d), SENTINEL);
        assert_eq!(last_error_code(), ERR_CAPACITY);
        world_destroy(world);
    }
}
