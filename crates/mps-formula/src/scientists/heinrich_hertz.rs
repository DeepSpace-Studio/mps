//! Heinrich Hertz —— 贡献目录与公式实现。
//!
//! 本文件收录该科学家的里程碑式贡献，并承载其名下的公式实现
//! （从原 `mps-formula` 域模块迁移而来；实现在此，域模块仅作
//! `pub use` 重导出以保持 FFI/ABI 不变）。不引入 Rapier / `WorldHandle`。

use super::ScientistRecord;

/// 本科学家的贡献记录。
#[allow(dead_code)]
pub const SCIENTIST: ScientistRecord = ScientistRecord {
    id: "heinrich_hertz",
    name: "Heinrich Hertz",
    birth_year: Some(1857),
    death_year: Some(1894),
    field_id: "electromagnetism",
    nationality: "German",
    contribution: "Experimental proof of EM waves",
    key_constants: "hertz",
};

/// 该科学家名下的公式实现（从各域模块迁移而来）。
pub mod formulas {
    use crate::error::*;
    use crate::ffi::*;
    use crate::math::*;
    const PI: f64 = std::f64::consts::PI;
    fn finite_3(a: f64, b: f64, c: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite()
    }
    fn finite_4(a: f64, b: f64, c: f64, d: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite()
    }
    fn finite_5(a: f64, b: f64, c: f64, d: f64, e: f64) -> bool {
        a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite() && e.is_finite()
    }
    fn finite_6(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> bool {
        a.is_finite()
            && b.is_finite()
            && c.is_finite()
            && d.is_finite()
            && e.is_finite()
            && f.is_finite()
    }

    /// Active sonar equation: SL - 2·TL + TS = NL - DI + DT
    /// Returns the received echo level (dB).

    pub fn active_sonar_echo_level(
        source_level: f64,
        transmission_loss: f64,
        target_strength: f64,
        noise_level: f64,
        directivity_index: f64,
        detection_threshold: f64,
    ) -> Option<f64> {
        if !finite_6(
            source_level,
            transmission_loss,
            target_strength,
            noise_level,
            directivity_index,
            detection_threshold,
        ) {
            return None;
        }
        Some(
            source_level - 2.0 * transmission_loss + target_strength
                - (noise_level - directivity_index + detection_threshold),
        )
    }

    /// Passive sonar equation: SL - TL = NL - DI + DT
    /// Returns the signal excess (dB).

    pub fn passive_sonar_signal_excess(
        source_level: f64,
        transmission_loss: f64,
        noise_level: f64,
        directivity_index: f64,
        detection_threshold: f64,
    ) -> Option<f64> {
        if !finite_5(
            source_level,
            transmission_loss,
            noise_level,
            directivity_index,
            detection_threshold,
        ) {
            return None;
        }
        Some(
            source_level
                - transmission_loss
                - (noise_level - directivity_index + detection_threshold),
        )
    }

    /// Spherical spreading loss: TL = 20·log₁₀(r)

    pub fn spherical_spreading_loss(range: f64) -> Option<f64> {
        if !range.is_finite() || range <= 0.0 {
            return None;
        }
        Some(20.0 * range.log10())
    }

    /// Cylindrical spreading loss: TL = 10·log₁₀(r)

    pub fn cylindrical_spreading_loss(range: f64) -> Option<f64> {
        if !range.is_finite() || range <= 0.0 {
            return None;
        }
        Some(10.0 * range.log10())
    }

    /// Thorp absorption coefficient (dB/km) for seawater.
    /// α = 0.11·f²/(1+f²) + 44·f²/(4100+f²) + 2.75e-4·f² + 0.003

    pub fn thorp_absorption(frequency_khz: f64) -> Option<f64> {
        if !frequency_khz.is_finite() || frequency_khz < 0.0 {
            return None;
        }
        let f = frequency_khz;
        Some(
            0.11 * f * f / (1.0 + f * f)
                + 44.0 * f * f / (4100.0 + f * f)
                + 2.75e-4 * f * f
                + 0.003,
        )
    }

    /// Sabine reverberation time: RT₆₀ = 0.161 · V / (S·ᾱ)

    pub fn sabine_rt60(volume: f64, surface_area: f64, mean_absorption: f64) -> Option<f64> {
        if !finite_3(volume, surface_area, mean_absorption)
            || volume <= 0.0
            || surface_area <= 0.0
            || mean_absorption <= 0.0
        {
            return None;
        }
        Some(0.161 * volume / (surface_area * mean_absorption))
    }

    /// Eyring reverberation time: RT₆₀ = 0.161 · V / (-S·ln(1-ᾱ))

    pub fn eyring_rt60(volume: f64, surface_area: f64, mean_absorption: f64) -> Option<f64> {
        if !finite_3(volume, surface_area, mean_absorption)
            || volume <= 0.0
            || surface_area <= 0.0
            || mean_absorption <= 0.0
            || mean_absorption >= 1.0
        {
            return None;
        }
        Some(0.161 * volume / (-surface_area * (1.0 - mean_absorption).ln()))
    }

    /// Characteristic impedance: Z = ρ·c

    pub fn acoustic_impedance(density: f64, sound_speed: f64) -> Option<f64> {
        if !density.is_finite() || density <= 0.0 || !sound_speed.is_finite() || sound_speed <= 0.0
        {
            return None;
        }
        Some(density * sound_speed)
    }

    /// Normal incidence transmission coefficient: τ = 4·Z₁·Z₂ / (Z₁+Z₂)²

    pub fn transmission_coefficient(z1: f64, z2: f64) -> Option<f64> {
        if !z1.is_finite() || z1 <= 0.0 || !z2.is_finite() || z2 <= 0.0 {
            return None;
        }
        Some(4.0 * z1 * z2 / ((z1 + z2) * (z1 + z2)))
    }

    /// Mass law transmission loss: TL = 20·log₁₀(f·m) - 47 (dB)

    pub fn mass_law_tl(frequency: f64, surface_density: f64) -> Option<f64> {
        if !frequency.is_finite()
            || frequency <= 0.0
            || !surface_density.is_finite()
            || surface_density <= 0.0
        {
            return None;
        }
        Some(20.0 * (frequency * surface_density).log10() - 47.0)
    }

    /// Helmholtz resonator frequency: f = (c/(2π))·√(A/(V·L))

    pub fn helmholtz_resonance_frequency(
        sound_speed: f64,
        neck_area: f64,
        cavity_volume: f64,
        neck_length: f64,
    ) -> Option<f64> {
        if !finite_4(sound_speed, neck_area, cavity_volume, neck_length)
            || sound_speed <= 0.0
            || neck_area <= 0.0
            || cavity_volume <= 0.0
            || neck_length <= 0.0
        {
            return None;
        }
        Some(
            sound_speed / (2.0 * std::f64::consts::PI)
                * (neck_area / (cavity_volume * neck_length)).sqrt(),
        )
    }

    /// Doppler shift: f' = f · (c ± v_r) / (c ∓ v_s)

    pub fn doppler_shift(
        source_frequency: f64,
        sound_speed: f64,
        receiver_velocity: f64,
        source_velocity: f64,
        approach: bool,
    ) -> Option<f64> {
        if !finite_4(
            source_frequency,
            sound_speed,
            receiver_velocity,
            source_velocity,
        ) || source_frequency <= 0.0
            || sound_speed <= 0.0
        {
            return None;
        }
        let num = sound_speed
            + (if approach {
                receiver_velocity
            } else {
                -receiver_velocity
            });
        let den = sound_speed
            - (if approach {
                source_velocity
            } else {
                -source_velocity
            });
        if den <= 0.0 {
            return None;
        }
        Some(source_frequency * num / den)
    }

    /// Barrier attenuation (Maekawa): ΔL = 5 + 20·log₁₀(√(2π·N)/tanh(√(2π·N)))

    pub fn maekawa_barrier_attenuation(fresnel_number: f64) -> Option<f64> {
        if !fresnel_number.is_finite() || fresnel_number < 0.0 {
            return None;
        }
        if fresnel_number < 1e-6 {
            return Some(5.0);
        }
        let arg = (2.0 * std::f64::consts::PI * fresnel_number).sqrt();
        Some(5.0 + 20.0 * (arg / arg.tanh()).log10())
    }
}
