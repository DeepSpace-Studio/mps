//! Unit tests for the pure Voronoi pre-fracture math (`mps-formula::voronoi`):
//! cell geometry, volume/centroid correctness, fragment box-fitting, and
//! input validation.

#[cfg(test)]
mod tests {
    use mps_formula::ffi::{FractureFragmentDesc, Vec3};
    use mps_formula::voronoi::{MAX_VORONOI_SEEDS, voronoi_cell, voronoi_fragments_from_seeds};

    fn v3(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    fn unit_box() -> (Vec3, Vec3) {
        (v3(-1.0, -1.0, -1.0), v3(1.0, 1.0, 1.0))
    }

    fn template() -> FractureFragmentDesc {
        FractureFragmentDesc {
            local_center: Vec3::default(),
            half_extents: Vec3::default(),
            initial_velocity: Vec3::default(),
            density: 1000.0,
            friction: 0.5,
            restitution: 0.1,
        }
    }

    #[test]
    fn single_seed_cell_equals_aabb() {
        let (lo, hi) = unit_box();
        let cell = voronoi_cell(v3(0.0, 0.0, 0.0), &[], lo, hi).unwrap();
        assert_eq!(cell.vertices.len(), 8);
        assert!((cell.volume - 8.0).abs() < 1.0e-9);
        assert!(cell.centroid.x.abs() < 1.0e-9);
        assert!(cell.centroid.y.abs() < 1.0e-9);
        assert!(cell.centroid.z.abs() < 1.0e-9);
    }

    #[test]
    fn two_seeds_split_volume_evenly() {
        let (lo, hi) = unit_box();
        let center = v3(-0.5, 0.0, 0.0);
        let neighbors = [v3(0.5, 0.0, 0.0)];
        let cell = voronoi_cell(center, &neighbors, lo, hi).unwrap();
        // The bisector plane is x = 0, so the cell is [-1,0]×[-1,1]² (volume 4).
        assert!((cell.volume - 4.0).abs() < 1.0e-9);
        assert!((cell.centroid.x - (-0.5)).abs() < 1.0e-9);
        // Every vertex lies on the x <= 0 side.
        assert!(cell.vertices.iter().all(|v| v.x <= 1.0e-9));
    }

    #[test]
    fn four_seeds_cells_fill_box() {
        let (lo, hi) = unit_box();
        let seeds = [
            v3(-0.5, 0.0, -0.5),
            v3(0.5, 0.0, -0.5),
            v3(-0.5, 0.0, 0.5),
            v3(0.5, 0.0, 0.5),
        ];
        let mut total = 0.0;
        for (index, seed) in seeds.iter().copied().enumerate() {
            let neighbors: Vec<Vec3> = seeds
                .iter()
                .copied()
                .enumerate()
                .filter_map(|(j, s)| if j != index { Some(s) } else { None })
                .collect();
            let cell = voronoi_cell(seed, &neighbors, lo, hi).unwrap();
            // A symmetric 2x2 quad grid quarters the box.
            assert!((cell.volume - 2.0).abs() < 1.0e-9);
            total += cell.volume;
        }
        assert!((total - 8.0).abs() < 1.0e-9);
    }

    #[test]
    fn fragments_match_cells_and_template() {
        let (lo, hi) = unit_box();
        let seeds = [v3(-0.5, 0.0, 0.0), v3(0.5, 0.0, 0.0)];
        let frags = voronoi_fragments_from_seeds(lo, hi, &seeds, template(), 0.0).unwrap();
        assert_eq!(frags.len(), 2);
        // First fragment box-fits the [-1,0]×[-1,1]² cell.
        assert!((frags[0].local_center.x - (-0.5)).abs() < 1.0e-9);
        assert!((frags[0].half_extents.x - 0.5).abs() < 1.0e-9);
        assert!((frags[0].half_extents.y - 1.0).abs() < 1.0e-9);
        // Template fields pass through.
        assert!((frags[0].density - 1000.0).abs() < 1.0e-12);
        assert!((frags[0].friction - 0.5).abs() < 1.0e-12);
        assert!((frags[0].restitution - 0.1).abs() < 1.0e-12);
        assert!(frags[0].initial_velocity.x == 0.0);
    }

    #[test]
    fn shrink_reduces_half_extents() {
        let (lo, hi) = unit_box();
        let seeds = [v3(-0.5, 0.0, 0.0), v3(0.5, 0.0, 0.0)];
        let frags = voronoi_fragments_from_seeds(lo, hi, &seeds, template(), 0.1).unwrap();
        // half 0.5 → 0.5 * (1 - 2*0.1) = 0.4
        assert!((frags[0].half_extents.x - 0.4).abs() < 1.0e-9);
        assert!((frags[0].half_extents.y - 0.8).abs() < 1.0e-9);
    }

    #[test]
    fn duplicate_seeds_are_merged() {
        let (lo, hi) = unit_box();
        let seeds = [v3(-0.5, 0.0, 0.0), v3(-0.5, 0.0, 0.0), v3(0.5, 0.0, 0.0)];
        let frags = voronoi_fragments_from_seeds(lo, hi, &seeds, template(), 0.0).unwrap();
        assert_eq!(frags.len(), 2);
    }

    #[test]
    fn invalid_inputs_rejected() {
        let (lo, hi) = unit_box();
        let seeds = [v3(-0.5, 0.0, 0.0), v3(0.5, 0.0, 0.0)];
        // No seeds.
        assert!(voronoi_fragments_from_seeds(lo, hi, &[], template(), 0.0).is_none());
        // Inverted / flat AABB.
        assert!(voronoi_fragments_from_seeds(hi, lo, &seeds, template(), 0.0).is_none());
        assert!(
            voronoi_fragments_from_seeds(
                v3(-1.0, 0.0, -1.0),
                v3(1.0, 0.0, 1.0),
                &seeds,
                template(),
                0.0
            )
            .is_none()
        );
        // Out-of-range shrink.
        assert!(voronoi_fragments_from_seeds(lo, hi, &seeds, template(), 0.5).is_none());
        assert!(voronoi_fragments_from_seeds(lo, hi, &seeds, template(), -0.1).is_none());
        assert!(voronoi_fragments_from_seeds(lo, hi, &seeds, template(), f64::NAN).is_none());
        // Non-finite seed.
        assert!(
            voronoi_fragments_from_seeds(
                lo,
                hi,
                &[v3(0.0, 0.0, 0.0), v3(f64::NAN, 0.0, 0.0)],
                template(),
                0.0
            )
            .is_none()
        );
        // Invalid cell input (flat box, non-finite).
        assert!(
            voronoi_cell(
                v3(0.0, 0.0, 0.0),
                &[],
                v3(-1.0, 0.0, -1.0),
                v3(1.0, 0.0, 1.0)
            )
            .is_none()
        );
        assert!(voronoi_cell(v3(f64::INFINITY, 0.0, 0.0), &[], lo, hi).is_none());
    }

    #[test]
    fn seed_count_cap_enforced() {
        let (lo, hi) = unit_box();
        let seeds: Vec<Vec3> = (0..=MAX_VORONOI_SEEDS)
            .map(|i| v3(-1.0 + 2.0 * i as f64 / MAX_VORONOI_SEEDS as f64, 0.0, 0.0))
            .collect();
        assert_eq!(seeds.len(), MAX_VORONOI_SEEDS + 1);
        assert!(voronoi_fragments_from_seeds(lo, hi, &seeds, template(), 0.0).is_none());
        // Exactly at the cap is accepted (degenerate slivers get skipped, but
        // a subset of cells remains valid).
        let seeds = &seeds[..MAX_VORONOI_SEEDS];
        let frags = voronoi_fragments_from_seeds(lo, hi, seeds, template(), 0.0).unwrap();
        assert!(!frags.is_empty());
    }
}
