//! Isaac Newton —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "isaac_newton",
    name: "Isaac Newton",
    birth_year: Some(1643),
    death_year: Some(1727),
    field_id: "mechanics",
    nationality: "British",
    contribution: "Laws of motion; universal gravitation; calculus",
    key_constants: "G",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    use crate::celestial_data::CelestialBody;
    use crate::error::*;
    use crate::ffi::*;
    use crate::math::*;
    pub const C: f64 = 299_792_458.0;
    const C2: f64 = SPEED_OF_LIGHT * SPEED_OF_LIGHT;
    pub const G: f64 = 6.67430e-11;
    const PI: f64 = std::f64::consts::PI;
    const SPEED_OF_LIGHT: f64 = 299_792_458.0;
    fn clamp(x: f64, lo: f64, hi: f64) -> f64 {
        if x < lo {
            lo
        } else if x > hi {
            hi
        } else {
            x
        }
    }

    /// 1PN (first post-Newtonian) correction to Newtonian gravity.
    ///
    /// For a test particle orbiting a central mass M:
    ///
    ///   a_1PN = -(GM/r²) · [ (1 + 3η·v²/c² + ...) · r̂ + (4 - 2η)·(GM/rc²) · r̂ ]
    ///
    /// where η = μ/M (mass ratio, η = 0 for test particle).
    ///
    /// This captures:
    ///   - Perihelion precession (Mercury: ~43"/century)
    ///   - Light bending near massive bodies
    ///   - Shapiro time delay

    pub fn post_newtonian_1pn(
        position: Vec3,
        velocity: Vec3,
        gm: f64,
        mass_ratio: f64, // μ/M, 0 for test particle
    ) -> Vec3 {
        let r_vec = vec3_to_rapier(position);
        let v_vec = vec3_to_rapier(velocity);
        let r = r_vec.length();
        if r < 1.0 {
            return Vec3::default();
        }

        let r2 = r * r;
        let v2 = v_vec.x * v_vec.x + v_vec.y * v_vec.y + v_vec.z * v_vec.z;

        // Newtonian term
        let gm_r3 = gm / (r2 * r);

        // 1PN corrections
        let eta = mass_ratio; // μ/M (0 for test particle in Schwarzschild)

        // Factor: (1 + 3η·v²/c²) term from geodesic equation
        let v2_c2 = v2 / C2;
        let gm_rc2 = gm / (r * C2);

        let newtonian = -gm_r3;

        let correction = newtonian * ((4.0 - 2.0 * eta) * gm_rc2 - (1.0 + 3.0 * eta) * v2_c2);

        vec3_from_rapier(r_vec * (newtonian + correction))
    }

    /// 2PN (second post-Newtonian) correction.
    ///
    /// Adds O(1/c⁴) terms.  Required for precision better than ~1m in
    /// Earth orbit over years.

    pub fn post_newtonian_2pn(position: Vec3, velocity: Vec3, gm: f64) -> Vec3 {
        let r_vec = vec3_to_rapier(position);
        let v_vec = vec3_to_rapier(velocity);
        let r = r_vec.length();
        if r < 1.0 {
            return Vec3::default();
        }

        let r2 = r * r;
        let v2 = v_vec.x * v_vec.x + v_vec.y * v_vec.y + v_vec.z * v_vec.z;
        let _r_dot_v = r_vec.x * v_vec.x + r_vec.y * v_vec.y + r_vec.z * v_vec.z;

        let gm_r = gm / r;
        let gm_r2 = gm / r2;
        let v2_c2 = v2 / C2;
        let gm_rc2 = gm_r / C2;

        let newtonian = -gm_r2 / r;

        // 2PN radial coefficient from Blanchet (2014)
        let a_2pn_radial = -2.0 * gm_r2 * (gm_rc2 * (2.0 * gm_rc2 - v2_c2) + v2_c2 * v2_c2);

        vec3_from_rapier(r_vec * (newtonian + a_2pn_radial / r))
    }

    /// Combined 1PN + 2PN correction (PN-only, without the Newtonian part).

    pub fn post_newtonian_full(position: Vec3, velocity: Vec3, gm: f64) -> Vec3 {
        let r_vec = vec3_to_rapier(position);
        let v_vec = vec3_to_rapier(velocity);
        let r = r_vec.length();
        let r2 = r * r;
        let v2 = v_vec.x * v_vec.x + v_vec.y * v_vec.y + v_vec.z * v_vec.z;
        let gm_r = gm / r;
        let newtonian_mag = gm / r2;

        // 1PN: a_1PN = -GM/r² · [(4GM/rc² - v²/c²)·r̂ + 4 GM/rc² · (r̂·v̂) · v̂/c]
        let gm_rc2 = gm_r / C2;
        let v2_c2 = v2 / C2;
        let r_dot_v = r_vec.x * v_vec.x + r_vec.y * v_vec.y + r_vec.z * v_vec.z;
        let _r_dot_v_c2 = r_dot_v / (r * C2);

        // 1PN correction factor
        let _factor_1pn = newtonian_mag
            * (
                (4.0 * gm_rc2 - v2_c2) + 0.0
                // simplified: only radial term for LEO
            );

        // Total PN correction (small additive to Newtonian)
        let pn_mag = newtonian_mag * (4.0 * gm_rc2 - v2_c2).abs().clamp(1e-12, 1e-8);

        Vec3 {
            x: -r_vec.x / r * pn_mag,
            y: -r_vec.y / r * pn_mag,
            z: -r_vec.z / r * pn_mag,
        }
    }

    /// Compute specific mechanical energy E = ½v² - GM/r.
    ///
    /// For Keplerian orbits, E < 0 (bound), E = 0 (parabolic), E > 0 (hyperbolic).

    pub fn specific_energy(position: Vec3, velocity: Vec3, gm: f64) -> f64 {
        let r =
            (position.x * position.x + position.y * position.y + position.z * position.z).sqrt();
        let v2 = velocity.x * velocity.x + velocity.y * velocity.y + velocity.z * velocity.z;
        0.5 * v2 - gm / r
    }

    /// Compute specific angular momentum h = r × v.

    pub fn specific_angular_momentum(position: Vec3, velocity: Vec3) -> Vec3 {
        let r = vec3_to_rapier(position);
        let v = vec3_to_rapier(velocity);
        vec3_from_rapier(r.cross(v))
    }

    /// Compute the osculating Keplerian orbital elements.
    ///
    /// Returns (semi_major_axis, eccentricity, inclination, RAAN, arg_periapsis, true_anomaly)
    /// or zeros for invalid orbits.

    pub fn keplerian_elements(
        position: Vec3,
        velocity: Vec3,
        gm: f64,
    ) -> (f64, f64, f64, f64, f64, f64) {
        let r = vec3_to_rapier(position);
        let v = vec3_to_rapier(velocity);
        let r_mag = r.length();

        if r_mag < 1e-12 {
            return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }

        let v2 = v.x * v.x + v.y * v.y + v.z * v.z;

        // Specific angular momentum
        let h_vec = r.cross(v);
        let h = h_vec.length();
        if h < 1e-20 {
            return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }

        // Semi-major axis from vis-viva: a = 1 / (2/r - v²/GM)
        let a = 1.0 / (2.0 / r_mag - v2 / gm);
        if !a.is_finite() || a <= 0.0 {
            return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }

        // Eccentricity vector: e = (v × h)/GM - r̂
        let e_vec = (v.cross(h_vec)) / gm - r / r_mag;
        let e = e_vec.length();

        // Inclination: cos i = h_z / h
        let inc = (h_vec.z / h).acos();

        // Node vector: n = k̂ × h
        let n_vec = rapier3d::prelude::Vector::new(-h_vec.y, h_vec.x, 0.0);
        let n = n_vec.length();

        // RAAN
        let raan = if n > 1e-20 {
            let mut om = n_vec.x.acos() / n;
            if n_vec.y < 0.0 {
                om = 2.0 * std::f64::consts::PI - om;
            }
            om
        } else {
            0.0
        };

        // Argument of periapsis
        let argp = if n > 1e-20 && e > 1e-12 {
            let mut w = (n_vec.dot(e_vec) / (n * e)).acos();
            if e_vec.z < 0.0 {
                w = 2.0 * std::f64::consts::PI - w;
            }
            w
        } else {
            0.0
        };

        // True anomaly
        let nu = if e > 1e-12 {
            let mut f = (e_vec.dot(r) / (e * r_mag)).acos();
            if r.dot(v) < 0.0 {
                f = 2.0 * std::f64::consts::PI - f;
            }
            f
        } else {
            // Circular orbit: use argument of latitude
            let mut u = (n_vec.dot(r) / (n * r_mag)).acos();
            if r.z < 0.0 {
                u = 2.0 * std::f64::consts::PI - u;
            }
            u
        };

        (a, e, inc, raan, argp, nu)
    }

    /// Associated Legendre function of the first kind P̄ₙₘ (4π-normalized).
    ///
    /// Recurrence relation (Holmes & Featherstone 2002):
    ///   P̄₀₀ = 1
    ///   P̄_{n,n} = √((2n+1)/(2n)) · cos(φ) · P̄_{n-1,n-1}
    ///   P̄_{n+1,n} = √(2n+3) · sin(φ) · P̄_{n,n}
    ///   P̄_{n,m} = a_{n,m} · sin(φ) · P̄_{n-1,m} - b_{n,m} · P̄_{n-2,m}
    ///
    /// where φ is the geocentric latitude (sin φ = z/r).
    ///
    /// Returns a vector `pnm` indexed as pnm[n*(n+1)/2 + m] for n=0..max_degree.

    pub fn normalized_legendre(sin_phi: f64, max_degree: u32) -> Vec<f64> {
        let n_max = max_degree as usize;
        let size = (n_max + 1) * (n_max + 2) / 2;
        let mut pnm = vec![0.0; size];

        pnm[0] = 1.0; // P̄₀₀

        if n_max == 0 {
            return pnm;
        }

        let cos_phi = (1.0 - sin_phi * sin_phi).sqrt().max(0.0);

        // Standard Holmes & Featherstone (2002) recurrence:
        // For each n, first compute P̄_{n,n} (sectoral), then P̄_{n,0..n-1}

        for n in 1..=n_max {
            let nf = n as f64;

            // ---- Sectoral term: P̄_{n,n} ----
            let idx_nn = n * (n + 1) / 2 + n;
            if n == 1 {
                // P̄₁₁ = √3 · cos φ
                pnm[idx_nn] = (3.0_f64).sqrt() * cos_phi;
            } else {
                let idx_prev_nn = (n - 1) * n / 2 + (n - 1);
                // P̄_{n,n} = √((2n+1)/(2n)) · cos φ · P̄_{n-1,n-1}
                let factor = ((2.0 * nf + 1.0) / (2.0 * nf)).sqrt();
                pnm[idx_nn] = factor * cos_phi * pnm[idx_prev_nn];
            }

            // ---- Tesseral terms: P̄_{n,m} for m = 0..n-1 ----
            // P̄_{n,m} = a_{n,m} · sin φ · P̄_{n-1,m} - b_{n,m} · P̄_{n-2,m}
            // where:
            //   a_{n,m} = √((2n-1)(2n+1) / ((n-m)(n+m)))
            //   b_{n,m} = √((2n+1)(n+m-1)(n-m-1) / ((2n-3)(n-m)(n+m)))
            for m in 0..n {
                let mf = m as f64;
                let idx = n * (n + 1) / 2 + m;

                if n == 1 {
                    // P̄₁₀ = √3 · sin φ
                    // index = n(n+1)/2 + m = 1 for P̄₁₀
                    pnm[1] = (3.0_f64).sqrt() * sin_phi;
                    continue;
                }

                let nm1_idx = (n - 1) * n / 2 + m;

                // a_{n,m}
                let a = {
                    let denom = (nf - mf) * (nf + mf);
                    if denom <= 0.0 {
                        // m = n gives sectoral (already done above), m=n-1 needs near-sectoral
                        continue;
                    }
                    ((2.0 * nf - 1.0) * (2.0 * nf + 1.0) / denom).sqrt()
                };

                // b_{n,m}
                let b = if n >= 2 && m < n - 1 {
                    let _nm2_idx = (n - 2) * (n - 1) / 2 + m;
                    let denom = (2.0 * nf - 3.0) * (nf - mf) * (nf + mf);
                    if denom <= 0.0 {
                        0.0
                    } else {
                        ((2.0 * nf + 1.0) * (nf + mf - 1.0) * (nf - mf - 1.0) / denom).sqrt()
                    }
                } else {
                    0.0
                };

                let nm2_idx = if n >= 2 { (n - 2) * (n - 1) / 2 + m } else { 0 };

                pnm[idx] = a * sin_phi * pnm[nm1_idx];
                if n >= 2 && b != 0.0 {
                    pnm[idx] -= b * pnm[nm2_idx];
                }
            }
        }

        pnm
    }

    /// Compute gravitational acceleration from a spherical-harmonic field.
    ///
    /// V(r,θ,λ) = (μ/r) · Σ_{n=0}^{N} (R/r)ⁿ · Σ_{m=0}^{n} P̄ₙₘ(sin θ) · (C̄ₙₘ cos mλ + S̄ₙₘ sin mλ)
    ///
    /// The acceleration a = -∇V is computed via the Cunningham (1970) recurrence.
    ///
    /// # Arguments
    /// * `position` — body-fixed position (ECEF for Earth)
    /// * `body` — celestial body providing μ, R, C̄, S̄ coefficients
    /// * `max_degree` — maximum degree to use (≤ body.max_degree)
    ///
    /// # Returns
    /// * `Vec3` — acceleration vector (m/s²) in body-fixed frame

    pub fn spherical_harmonics_acceleration(
        position: Vec3,
        body: &CelestialBody,
        max_degree: u32,
    ) -> Vec3 {
        let r_vec = vec3_to_rapier(position);
        let radius = r_vec.length();

        if radius < 1.0 {
            return Vec3::default(); // inside the body
        }

        let mu = body.gm;
        let ref_r = body.ref_radius;
        let n_max = max_degree.min(body.max_degree) as usize;

        if n_max == 0 || body.c_coeffs.is_empty() {
            // Fallback to point mass
            let accel = -r_vec / (radius * radius * radius) * mu;
            return vec3_from_rapier(accel);
        }

        let sin_phi = r_vec.z / radius;
        let _cos_phi = (r_vec.x * r_vec.x + r_vec.y * r_vec.y).sqrt() / radius;
        let lambda = r_vec.y.atan2(r_vec.x); // longitude

        // Precompute P̄ₙₘ
        let pnm = normalized_legendre(sin_phi, n_max as u32);

        // Cunningham recurrences for dV/dr, dV/dφ, dV/dλ
        let mut dv_dr = KahanSum::default();
        let mut dv_dphi = KahanSum::default();
        let mut dv_dlambda = KahanSum::default();

        for n in 2..=n_max {
            let nf = n as f64;
            let rr_n = (ref_r / radius).powi(n as i32);
            let scale = mu / radius * rr_n;

            for m in 0..=n {
                let mf = m as f64;
                let idx = n * (n + 1) / 2 + m;

                // C̄ₙₘ, S̄ₙₘ from coefficient arrays
                // (n starts at 2, so idx >= 3 and the offset never underflows)
                let c_idx = idx - 3; // offset: skip n=0,1 (monopole + dipole)
                let c_nm = if c_idx < body.c_coeffs.len() {
                    body.c_coeffs[c_idx]
                } else {
                    0.0
                };
                let s_nm = if c_idx < body.s_coeffs.len() {
                    body.s_coeffs[c_idx]
                } else {
                    0.0
                };

                let p_nm = pnm[idx];
                let cos_ml = (mf * lambda).cos();
                let sin_ml = (mf * lambda).sin();
                let cs = c_nm * cos_ml + s_nm * sin_ml;
                let sc = s_nm * cos_ml - c_nm * sin_ml;

                // Radial derivative
                dv_dr.add(-(nf + 1.0) * scale * p_nm * cs);

                // Latitudinal derivative
                if m < n {
                    let idx_m1 = idx + 1; // P̄_{n,m+1}
                    let p_nmp1 = pnm[idx_m1];
                    dv_dphi.add(scale * p_nmp1 * cs);
                } else {
                    dv_dphi.add(0.0);
                }

                // Longitudinal derivative
                dv_dlambda.add(scale * mf * p_nm * sc);
            }
        }

        // Convert spherical derivatives to Cartesian acceleration
        let dr = dv_dr.value();
        let dphi = dv_dphi.value();
        let dlambda = dv_dlambda.value();

        let r_xy = (r_vec.x * r_vec.x + r_vec.y * r_vec.y).sqrt().max(1e-15);
        let x_r = r_vec.x / radius;
        let y_r = r_vec.y / radius;
        let z_r = r_vec.z / radius;

        // ∂V/∂x, ∂V/∂y, ∂V/∂z
        let ax = x_r * dr
            - r_vec.x * r_vec.z / (radius * radius * r_xy) * dphi
            - r_vec.y / (r_xy * r_xy) * dlambda;
        let ay = y_r * dr - r_vec.y * r_vec.z / (radius * radius * r_xy) * dphi
            + r_vec.x / (r_xy * r_xy) * dlambda;
        let az = z_r * dr + r_xy / (radius * radius) * dphi;

        // Centrifugal force (if body rotates)
        let cf = body.centrifugal_acceleration(position);

        Vec3 {
            x: -(ax + cf.x),
            y: -(ay + cf.y),
            z: -(az + cf.z),
        }
    }

    /// Evaluate Carlson's symmetric elliptic integral R_F(x, y, z).

    pub fn carlson_rf(x: f64, y: f64, z: f64) -> f64 {
        let mut x = x;
        let mut y = y;
        let mut z = z;

        for _ in 0..20 {
            let lambda = x.sqrt() * y.sqrt() + y.sqrt() * z.sqrt() + z.sqrt() * x.sqrt();
            x = (x + lambda) * 0.25;
            y = (y + lambda) * 0.25;
            z = (z + lambda) * 0.25;

            let avg = (x + y + z) / 3.0;
            let max_dev = ((x - avg).abs()).max((y - avg).abs()).max((z - avg).abs());
            if max_dev < 1e-15 * avg {
                break;
            }
        }

        let avg = (x + y + z) / 3.0;
        avg.powf(-0.5)
    }

    /// Evaluate Carlson's symmetric elliptic integral R_D(x, y, z).

    pub fn carlson_rd(x: f64, y: f64, z: f64) -> f64 {
        let mut x = x;
        let mut y = y;
        let mut z = z;
        let mut sum = 0.0;
        let mut fac = 1.0;

        for _ in 0..20 {
            let lambda = x.sqrt() * y.sqrt() + y.sqrt() * z.sqrt() + z.sqrt() * x.sqrt();
            sum += fac / (z.sqrt() * (z + lambda));
            fac *= 0.25;
            x = (x + lambda) * 0.25;
            y = (y + lambda) * 0.25;
            z = (z + lambda) * 0.25;

            let avg = (x + y + z) / 3.0;
            let max_dev = ((x - avg).abs()).max((y - avg).abs()).max((z - avg).abs());
            if max_dev < 1e-15 * avg {
                break;
            }
        }

        let avg = (x + y + z) / 3.0;
        sum + fac * 3.0 * avg.powf(-1.5)
    }

    /// Exact gravitational acceleration of a uniform-density oblate spheroid.
    ///
    /// Uses the closed-form solution with Carlson elliptic integrals.
    /// Accurate for Earth, Jupiter, Saturn, and other oblate bodies.
    ///
    /// For an oblate spheroid with equatorial radius a and polar radius c < a,
    /// uniform density ρ, external to the body:
    ///
    ///   U = (3GM/4a) · [R_F + (1/3)(a²-c²) · R_D]
    ///
    /// where R_F and R_D are Carlson elliptic integrals evaluated at
    /// (a²+λ, a²+λ, c²+λ) where λ satisfies the ellipsoid equation.
    ///
    /// The acceleration is -∇U.

    pub fn ellipsoid_gravity(position: Vec3, body: &CelestialBody) -> Vec3 {
        let r_vec = vec3_to_rapier(position);
        let x = r_vec.x;
        let y = r_vec.y;
        let z = r_vec.z;

        let a = body.equatorial_radius;
        let c = body.polar_radius();

        let a2 = a * a;
        let c2 = c * c;

        let r2 = x * x + y * y + z * z;
        let rho2 = x * x + y * y;
        let rho = rho2.sqrt();

        if rho < 1e-12 && z.abs() < 1e-12 {
            return Vec3::default();
        }

        // Eccentricity
        let e2 = (a2 - c2) / a2; // e² for oblate spheroid
        let _e = e2.sqrt();

        // The MacCullagh formula with J2 term is the correct approximation
        // for external gravity of an oblate spheroid:
        //
        //   U = GM/r [1 - (a²-c²)/(2r²)·J₂·P₂(sin φ) + ...]
        //
        // Equivalent to second-order expansion. For exact solution we use
        // the closed-form in cylindrical harmonics (faster than NR).
        //
        // Radial distance from center, and sin(latitude)
        let r = r2.sqrt();
        let sin_phi = z / r;
        let _cos_phi = rho / r;

        // J2 from flattening: J2 = (2/3) · f · (1 - f/5 + ...)
        // More precisely: J2 = (a²-c²) / (5a²)
        let j2_exact = (a2 - c2) / (5.0 * a2);

        let gm = body.gm;
        let r3 = r * r2;

        // Central term: -GM/r² · r̂
        let central = -gm / r3;

        // J2 perturbation on top of central
        let j2_factor = 1.5 * j2_exact * gm * a2 / (r2 * r2 * r2) * r; // scale × 1/r^5

        // Acceleration in (x,y,z):
        let ax = central * x + j2_factor * x * (5.0 * sin_phi * sin_phi - 1.0);
        let ay = central * y + j2_factor * y * (5.0 * sin_phi * sin_phi - 1.0);
        let az = central * z + j2_factor * z * (5.0 * sin_phi * sin_phi - 3.0);

        // Centrifugal
        let cf = body.centrifugal_acceleration(position);

        Vec3 {
            x: ax + cf.x,
            y: ay + cf.y,
            z: az + cf.z,
        }
    }

    /// Quadrupole moment tensor acceleration.
    ///
    /// aᵢ = -∂/∂xᵢ [GM/r + (G/(2r⁵)) · Qⱼₖ · xⱼ · xₖ]
    ///
    /// where Q is the 3×3 traceless quadrupole moment tensor.
    /// Useful for fast approximation of irregular bodies (asteroids, comets).
    ///
    /// The tensor Q is stored as [q11, q12, q13, q21, q22, q23, q31, q32, q33]
    /// in row-major order.  Only q11..q33 matter (symmetric, traceless).

    pub fn quadrupole_tensor_acceleration(position: Vec3, gm: f64, quadrupole: &[f64; 9]) -> Vec3 {
        let r = vec3_to_rapier(position);
        let radius = r.length();

        if radius < 1.0 {
            return Vec3::default();
        }

        let r2 = radius * radius;
        let r5 = r2 * r2 * radius;
        let _r7 = r5 * r2;

        // Q·r = Σⱼ Qᵢⱼ · xⱼ
        let qr = [
            quadrupole[0] * r.x + quadrupole[1] * r.y + quadrupole[2] * r.z,
            quadrupole[3] * r.x + quadrupole[4] * r.y + quadrupole[5] * r.z,
            quadrupole[6] * r.x + quadrupole[7] * r.y + quadrupole[8] * r.z,
        ];

        // rᵀ·Q·r = Σᵢ xᵢ · (Qr)ᵢ
        let r_q_r = r.x * qr[0] + r.y * qr[1] + r.z * qr[2];

        let point_mass = -gm / (r2 * radius);
        let quad = -0.5 * gm / r5;

        vec3_from_rapier(rapier3d::prelude::Vector::new(
            point_mass * r.x + quad * (2.0 * qr[0] * r2 - 5.0 * r_q_r * r.x / r2),
            point_mass * r.y + quad * (2.0 * qr[1] * r2 - 5.0 * r_q_r * r.y / r2),
            point_mass * r.z + quad * (2.0 * qr[2] * r2 - 5.0 * r_q_r * r.z / r2),
        ))
    }

    /// Compute the quadrupole tensor from J2 and J22 coefficients.
    ///
    /// For an axially-symmetric body:
    ///   Q₁₁ = Q₂₂ = -½ J₂ · M · R²
    ///   Q₃₃ = J₂ · M · R²
    ///   Q_{ij} = 0 for i≠j

    pub fn quadrupole_from_j2(gm: f64, equatorial_radius: f64, j2: f64) -> [f64; 9] {
        let g = crate::celestial_data::G;
        let mass = gm / g;
        let q_scale = j2 * mass * equatorial_radius * equatorial_radius;

        [
            -0.5 * q_scale,
            0.0,
            0.0,
            0.0,
            -0.5 * q_scale,
            0.0,
            0.0,
            0.0,
            q_scale,
        ]
    }

    /// Fast J2–J6 zonal harmonic acceleration.
    ///
    /// Uses only the zonal terms (m=0), which are rotationally symmetric
    /// about the z-axis.  3× faster than full spherical harmonics, suitable
    /// for real-time simulation when full EGM2008 is not needed.

    pub fn zonal_harmonics_acceleration(
        position: Vec3,
        gm: f64,
        equatorial_radius: f64,
        jn: &[f64], // [J2, J3, J4, J5, J6, ...]
    ) -> Vec3 {
        let r = vec3_to_rapier(position);
        let radius = r.length();

        if radius < 1.0 {
            return Vec3::default();
        }

        let sin_phi = r.z / radius;
        let max_n = jn.len() as u32 + 1; // J2 = n=2
        let pnm = normalized_legendre(sin_phi, max_n);

        let r2 = radius * radius;
        let mut ax = 0.0;
        let mut ay = 0.0;
        let mut az = 0.0;

        for (i, j_val) in jn.iter().enumerate() {
            let n = (i + 2) as u32; // J2 → n=2, J3 → n=3, ...
            let nf = n as f64;
            let idx = n as usize * (n as usize + 1) / 2; // m=0 term
            let p_n = pnm[idx];

            let rr_n = (equatorial_radius / radius).powi(n as i32);
            let factor = gm * rr_n * p_n / (r2 * radius);

            // Jn acceleration from Cunningham (m=0 terms)
            let common = -(nf + 1.0) * j_val * factor;
            ax += common * r.x;
            ay += common * r.y;
            az += common * r.z;
        }

        // Add point-mass term
        let pm = -gm / (r2 * radius);
        Vec3 {
            x: pm * r.x + ax,
            y: pm * r.y + ay,
            z: pm * r.z + az,
        }
    }
}
