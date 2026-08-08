
#[cfg(test)]
mod debug_tests {
    use mps_cosmos::bodies::satellite_builder;
    use rapier3d::prelude::{RigidBodySet, Vector};

    #[test]
    fn debug_body_mass() {
        let mut bodies = RigidBodySet::new();
        let handle = bodies.insert(
            satellite_builder(800.0, Vector::new(0.0, 0.0, 100.0), Vector::ZERO, 1.0).build(),
        );
        let body = bodies.get(handle).unwrap();
        eprintln!("body.mass() = {}", body.mass());
        let mp = body.mass_properties();
        eprintln!("local_mprops.mass = {}", mp.local_mprops.mass());
        let p = mp.local_mprops.principal_inertia();
        eprintln!("principal_inertia = {:?}", p);
    }
}
