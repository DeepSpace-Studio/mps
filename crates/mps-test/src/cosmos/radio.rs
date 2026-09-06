#[cfg(test)]
mod tests {
    use mps_cosmos::radio::{ActiveSignal, RadioNode, RadioWorld, ReflectorState, SIGNAL_TTL_MS};
    use rapier3d::prelude::Vector;

    fn node(id: u64, pos: Vector) -> RadioNode {
        RadioNode {
            id,
            pos,
            vel: Vector::ZERO,
            dir: Vector::new(0.0, 0.0, 1.0),
            frequency: 2.4e9,
            power: 50.0,
            sensitivity: 1.0e-15,
            rx_gain: 1.0,
            tx_gain: 1.0,
            beam_angle: std::f64::consts::PI,
            owner_body: None,
        }
    }

    fn signal(id: u64, tx: u64, origin: Vector) -> ActiveSignal {
        ActiveSignal {
            id,
            tx_node_id: tx,
            birth_ms: 0,
            origin,
            origin_vel: Vector::ZERO,
            origin_dir: Vector::new(0.0, 0.0, 1.0),
            frequency: 2.4e9,
            energy: 10.0,
            tx_gain: 1.0,
            beam_angle: std::f64::consts::PI,
            owner_body: None,
        }
    }

    #[test]
    fn registers_and_unregisters_nodes() {
        let mut world = RadioWorld::new();
        assert_eq!(world.node_count(), 0);
        assert_eq!(world.signal_count(), 0);
        assert_eq!(world.reflector_count(), 0);

        world.register_node(node(1, Vector::ZERO));
        world.register_node(node(2, Vector::new(100.0, 0.0, 0.0)));
        assert_eq!(world.node_count(), 2);

        // Re-registering the same id overwrites rather than duplicates.
        world.register_node(node(1, Vector::new(5.0, 0.0, 0.0)));
        assert_eq!(world.node_count(), 2);

        world.unregister_node(1);
        assert_eq!(world.node_count(), 1);
        assert_eq!(world.node_ids().collect::<Vec<_>>(), vec![2]);
    }

    #[test]
    fn delivers_signal_on_clear_line_of_sight() {
        let mut world = RadioWorld::new();
        world.register_node(node(1, Vector::ZERO));
        world.register_node(node(2, Vector::new(1000.0, 0.0, 0.0)));
        world.submit_signal(signal(7, 1, Vector::ZERO));

        world.step(&[], 0);
        let results = world.take_results();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].signal_id, 7);
        assert_eq!(results[0].rx_node_id, 2);
        assert!((results[0].received_frequency - 2.4e9).abs() < 1.0e-6);
        assert!(results[0].received_power > 0.0);
        // Delivered signals leave the active pool.
        assert_eq!(world.signal_count(), 0);
    }

    #[test]
    fn reflector_between_nodes_blocks_direct_path() {
        let mut world = RadioWorld::new();
        world.register_node(node(1, Vector::ZERO));
        world.register_node(node(2, Vector::new(1000.0, 0.0, 0.0)));
        // A planet-sized reflector sitting right between the two nodes.
        let bodies = [ReflectorState {
            handle_raw: 10,
            pos: Vector::new(500.0, 0.0, 0.0),
            radius: 100.0,
        }];
        world.submit_signal(signal(7, 1, Vector::ZERO));

        world.step(&bodies, 0);
        // No third body to reflect off → the signal is undelivered this tick
        // and stays buffered until its TTL expires.
        assert!(world.take_results().is_empty());
        assert_eq!(world.signal_count(), 1);

        world.step(&bodies, SIGNAL_TTL_MS + 1);
        assert!(world.take_results().is_empty());
        assert_eq!(world.signal_count(), 0);
    }
}
