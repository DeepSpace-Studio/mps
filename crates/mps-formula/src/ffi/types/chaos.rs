use super::core::*;
// ---------------------------------------------------------------------------
// Chaos theory / nonlinear dynamics structures
// ---------------------------------------------------------------------------

/// Lorenz attractor state at a single time step.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LorenzState {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Parameters for the Lorenz system: dx/dt = sigma*(y-x), dy/dt = x*(rho-z)-y, dz/dt = x*y - beta*z.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LorenzParams {
    pub sigma: f64,
    pub rho: f64,
    pub beta: f64,
    pub dt: f64,
}

impl Default for LorenzParams {
    fn default() -> Self {
        Self {
            sigma: 10.0,
            rho: 28.0,
            beta: 8.0 / 3.0,
            dt: 0.01,
        }
    }
}

/// Full Lorenz integration report at a step.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LorenzStepReport {
    pub state: LorenzState,
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
}

/// Lyapunov exponent estimation report for a single trajectory.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LyapunovReport {
    /// Largest Lyapunov exponent (bits/s or nats/s depending on log base)
    pub largest_exponent: f64,
    /// Convergence indicator: number of orbit steps used
    pub convergence_steps: u32,
    /// Whether the exponent is positive (chaotic) within the tolerance
    pub positive: Bool,
}

/// A single bifurcation point: parameter value vs. sampled state.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct BifurcationPoint {
    pub parameter: f64,
    pub sample: f64,
}

/// Double pendulum state (generalised coordinates and their derivatives).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DoublePendulumState {
    /// Angle of upper pendulum (radians)
    pub theta1: f64,
    /// Angle of lower pendulum (radians)
    pub theta2: f64,
    /// Angular velocity of upper pendulum (rad/s)
    pub omega1: f64,
    /// Angular velocity of lower pendulum (rad/s)
    pub omega2: f64,
}

/// Double pendulum parameters (geometry and integration step).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DoublePendulumParams {
    /// Mass of upper bob
    pub m1: f64,
    /// Mass of lower bob
    pub m2: f64,
    /// Length of upper rod
    pub l1: f64,
    /// Length of lower rod
    pub l2: f64,
    /// Gravitational acceleration
    pub g: f64,
    /// Integration time step
    pub dt: f64,
}

impl Default for DoublePendulumParams {
    fn default() -> Self {
        Self {
            m1: 1.0,
            m2: 1.0,
            l1: 1.0,
            l2: 1.0,
            g: 9.81,
            dt: 0.01,
        }
    }
}

/// Double-pendulum acceleration report (RK4 intermediate computation).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DoublePendulumAccel {
    pub alpha1: f64,
    pub alpha2: f64,
}

/// Report from a chaos detection analysis.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ChaosDetectionReport {
    /// Largest Lyapunov exponent estimate
    pub lyapunov_exponent: f64,
    /// Correlation dimension estimate (box-counting style)
    pub correlation_dimension: f64,
    /// Whether the system is classified as chaotic
    pub is_chaotic: Bool,
    /// Confidence metric between 0 and 1
    pub confidence: f64,
}

/// Parameters controlling chaos detection heuristics.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ChaosDetectionParams {
    /// Number of orbit steps to sample
    pub sample_steps: u32,
    /// Embedding dimension for delay-coordinate reconstruction
    pub embedding_dim: u32,
    /// Delay (in steps) for reconstruction
    pub embedding_delay: u32,
    /// Neighbourhood radius for correlation dimension
    pub neighbourhood_radius: f64,
    /// Threshold above which Lyapunov exponent is considered chaotic
    pub chaotic_threshold: f64,
}

impl Default for ChaosDetectionParams {
    fn default() -> Self {
        Self {
            sample_steps: 10_000,
            embedding_dim: 3,
            embedding_delay: 1,
            neighbourhood_radius: 0.1,
            chaotic_threshold: 0.001,
        }
    }
}

/// Logistic map state (classic 1D chaos example).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LogisticMapState {
    pub x: f64,
    pub r: f64,
}
