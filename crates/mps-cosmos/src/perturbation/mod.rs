//! 环境扰动力 — 大气阻力、太阳光压等空间环境作用于航天器的力。
//!
//! 复用 `mps_formula::spaceflight` 的纯计算函数，把位置/速度/引力体参数
//! 翻译为对应扰动力（加速度 × 质量）。所有函数返回的是**力**（N），
//! 由 `CosmosWorld::step` 在推进前注入到刚体上。

use mps_formula::celestial_data::{CelestialBody, G, SOLAR_PRESSURE_AT_1AU};
use mps_formula::ffi::Vec3;
use mps_formula::galactic_dynamics as gd;
use mps_formula::heliophysics as hph;
use mps_formula::spaceflight;
use rapier3d::prelude::Vector;

/// 计算大气阻力产生的力（单位 N），返回世界坐标系向量。
///
/// - `body_velocity`：航天器相对惯性系的速度（m/s）
/// - `atmosphere_velocity`：大气在惯性系下的速度（一般为赤道自转分量，0 表示静止大气）
/// - `density`：当地大气密度（kg/m³），可由
///   [`spaceflight::atmospheric_density_scale_height`] 结合天体 `surface_density`/
///   `scale_height` 算出
/// - `drag_coefficient`：阻力系数 Cd（典型 2.2）
/// - `area`：迎风截面积（m²）
/// - `mass`：航天器质量（kg）
///
/// 返回 `None` 表示输入非法（密度、面积、质量非正等）。
pub fn atmospheric_drag_force(
    body_velocity: Vector,
    atmosphere_velocity: Vector,
    density: f64,
    drag_coefficient: f64,
    area: f64,
    mass: f64,
) -> Option<Vector> {
    // 早退：密度/面积/质量非正直接返回，避免 FFI 调用 + Vec3 装箱。
    if density <= 0.0 || area <= 0.0 || mass <= 0.0 {
        return None;
    }
    let accel = spaceflight::atmospheric_drag_acceleration(
        to_ffi(body_velocity),
        to_ffi(atmosphere_velocity),
        density,
        drag_coefficient,
        area,
        mass,
    )?;
    Some(scale(accel, mass))
}

/// 基于天体大气模型在给定高度采样的大气密度（kg/m³）。
///
/// 使用天体的 `surface_density`（参考高度处密度）与 `scale_height`，
/// 调用 [`spaceflight::atmospheric_density_scale_height`]。若天体无大气
///（`surface_density==0` 或 `scale_height==0`）则返回 0。
#[inline]
pub fn atmosphere_density_at(body: &CelestialBody, altitude_above_surface: f64) -> f64 {
    if body.surface_density <= 0.0 || body.scale_height <= 0.0 {
        return 0.0;
    }
    spaceflight::atmospheric_density_scale_height(
        body.surface_density,
        altitude_above_surface,
        0.0, // 参考高度取表面
        body.scale_height,
    )
    .unwrap_or(0.0)
}

/// 太阳光压产生的力（N）。
///
/// 模型：`F = -Cr · P · A · ŝ`，其中 `P` 是距太阳 1 AU 处的太阳光压
/// 常数（`SOLAR_PRESSURE_AT_1AU`），按距离平方反比衰减；
/// `ŝ` 是从航天器指向太阳的单位向量。
///
/// - `sun_to_body`：太阳到航天器的位置向量（用于按 1/AU² 衰减）
/// - `sun_direction`：从航天器指向太阳的单位向量
/// - `area`：受光截面积（m²）
/// - `reflectivity`：光压系数 Cr（吸收=1，镜面反射=2，典型 1.3）
/// - `au`：以米为单位的 1 AU（[`mps_formula::celestial_data::AU`]）
pub fn solar_pressure_force(
    sun_to_body: Vector,
    sun_direction: Vector,
    area: f64,
    reflectivity: f64,
    au: f64,
) -> Vector {
    // 早退：area/reflectivity 无意义时直接零，避免 normalize + length 的两次 sqrt。
    if area <= 0.0 || reflectivity <= 0.0 || au <= 0.0 {
        return Vector::ZERO;
    }
    let dir_len = sun_direction.length();
    if dir_len <= 1e-12 {
        return Vector::ZERO;
    }
    // sun_direction 在上游已是 `-sun_to_body/r`，这里再 normalize 容错
    // 非单位输入；用 length_squared 比再次 .length() 省一次 sqrt。
    let dir = sun_direction / dir_len;
    let r2 = sun_to_body.length_squared();
    if r2 < 1.0 {
        return Vector::ZERO;
    }
    // 距离平方反比衰减：实际光压 = P · (1AU / r)²；用 r2 省一次 sqrt。
    let pressure = SOLAR_PRESSURE_AT_1AU * (au * au) / r2;
    -dir * (pressure * area * reflectivity)
}

/// 太阳风等离子体动压产生的力（N），沿风向推航天器。
///
/// 模型：`F = P_sw · A_eff · d̂`，其中 `P_sw = ρ · v_rel²` 是太阳风（质子数
/// 密度 × m_p × 整体速度²）作用于物体的动压，由
/// [`heliophysics::solar_wind_dynamic_pressure`] 计算（返回 nPa，本函数换算到
/// Pa 后乘以有效面积以得 N）。`d̂` 取自太阳指向航天器的方向——即风自太阳
/// 向外辐射的世界方向，与 `solar_pressure_force` 共用同一几何约定。
///
/// 物体自身速度沿风向的分量被减去，得到相对风压；物体顺风而行
///（v_rel ≤ 0）感受不到力。
///
/// 接入 `mps-formula` 的 [`heliophysics::solar_wind_dynamic_pressure`]。
///
/// Inputs:
/// - `sun_to_body`        — 太阳指物体的位置向量（决定风向与距离）
/// - `body_velocity`      — 物体在惯性系的速度（m/s）
/// - `proton_density`     — 太阳风质子数密度（n / m³）
/// - `solar_wind_speed`   — 太阳风整体速度（m/s，世界系）
/// - `effective_area`     — 迎风有效面积（m²）
pub fn solar_wind_pressure_force(
    sun_to_body: Vector,
    body_velocity: Vector,
    proton_density: f64,
    solar_wind_speed: f64,
    effective_area: f64,
) -> Option<Vector> {
    // 早退：参数无意义时直接零。
    if proton_density <= 0.0 || solar_wind_speed <= 0.0 || effective_area <= 0.0 {
        return None;
    }
    let r2 = sun_to_body.length_squared();
    if r2 < 1.0 || !r2.is_finite() {
        return None;
    }
    let r = r2.sqrt();
    let dir = sun_to_body / r; // 太阳 → 物体 的单位向量（风自太阳辐射的方向）

    // 沿风向的相对速度：v_rel = v_sw - v_body·d̂。物体顺风而行则压为 0。
    let v_rel = solar_wind_speed - body_velocity.dot(dir);
    if v_rel <= 0.0 || !v_rel.is_finite() {
        return None;
    }
    // 公式要求 km/s。
    let v_rel_kms = v_rel * 1.0e-3;
    // nPa → Pa。
    let pressure_pa = hph::solar_wind_dynamic_pressure(proton_density, v_rel_kms)? * 1.0e-9;
    Some(dir * (pressure_pa * effective_area))
}

/// 日食（阴影锥）衰减因子，作用于光压 / 太阳风。纯几何，无状态、不依赖 `WorldHandle`。
///
/// 双锥模型（太阳与遮挡体均为有限半径圆盘）：
/// - 本影（umbra）：遮挡体完全遮住太阳 → 因子 0（光压/太阳风为 0）。
/// - 半影（penumbra）：部分遮挡 → 因子在 [0,1] 内按横向距离线性过渡。
/// - 无遮挡 → 因子 1（等于原输出，满足「原方法不变」铁律）。
///
/// 几何：光源在 `sun_pos`，遮挡体圆心在原点、半径 `occ_radius`（=`central_body` 的
/// `equatorial_radius`），被照体在 `pos`。沿「太阳 → 遮挡体」轴投影求被照体的轴向
/// 距离 `x`（遮挡体之后为正）与横向距离 `d_perp`；用太阳半径 `sun_radius`
/// （=`SUN_EQ_RADIUS`）做双锥：
/// - `r_umbra(x) = occ_radius + (occ_radius - sun_radius) * x / D`（`D` = 日-遮距离）。
/// - `r_pen(x)   = occ_radius + (occ_radius + sun_radius) * x / D`。
/// `d_perp ≤ r_umbra` → 0；`d_perp ≤ r_pen` → `(d_perp - r_umbra) / (r_pen - r_umbra)`
/// 夹 [0,1]；否则 1。任意阶退化（零半径 / 零距离 / NaN）返回 1。
///
/// 调用方应先用 `cfg.enable_eclipse` 门控，只在开启时调用（开启即默认路径行为改变，
/// 故必须显式 opt-in，且需 lock-down 证明终态仅日食场景变化）。
pub fn eclipse_attenuation(pos: Vector, sun_pos: Vector, occ_radius: f64, sun_radius: f64) -> f64 {
    if occ_radius <= 0.0 || sun_radius <= 0.0 || !occ_radius.is_finite() || !sun_radius.is_finite()
    {
        return 1.0;
    }
    let d_sun = sun_pos.length();
    if d_sun < 1e-9 {
        return 1.0;
    }
    let d_body = pos.length();
    if d_body < 1e-9 {
        return 1.0;
    }
    // 轴方向：太阳 → 遮挡体（遮挡体在原点）→ a = -sun_pos / |sun_pos|。
    let a = -sun_pos / d_sun;
    // 被照体相对太阳的轴向投影 s（s=0 在太阳，s=D 在遮挡体）。
    let s = (pos - sun_pos).dot(a);
    let x = s - d_sun; // 遮挡体之后为正
    if x <= 0.0 {
        return 1.0; // 在太阳与遮挡体之间 / 之前 → 无阴影
    }
    // 横向距离：被照体到轴的距离。
    let axis_point = sun_pos + a * s;
    let d_perp = (pos - axis_point).length();
    if !d_perp.is_finite() {
        return 1.0;
    }
    let r_umbra = occ_radius + (occ_radius - sun_radius) * x / d_sun;
    if d_perp <= r_umbra {
        return 0.0; // 本影
    }
    let r_pen = occ_radius + (occ_radius + sun_radius) * x / d_sun;
    if d_perp <= r_pen {
        // 半影：0（内缘，恰全遮）→ 1（外缘，恰初遮）线性过渡。
        let denom = r_pen - r_umbra;
        if denom <= 0.0 {
            return 1.0;
        }
        let f = (d_perp - r_umbra) / denom;
        if f < 0.0 {
            0.0
        } else if f > 1.0 {
            1.0
        } else {
            f
        }
    } else {
        1.0
    }
}

/// Hut (1981) 平衡潮自旋同步力矩（N·m），驱动卫星自转趋向轨道同步、并（在双体
/// 均可动时）圆化轨道。
///
/// 采用 Murray & Dermott《Solar System Dynamics》式 (4.159) 的圆形轨道主导项：
/// `Γ = (3/2) · G·M_p²·k2 / (Q·a⁶) · R⁵ · (n − ω_∥)`，
/// 其中 `M_p` 为伴星（潮汐施主）GM、`k2` Love 数、`Q` 潮汐品质因子、`R` 卫星半径、
/// `a` 轨道半径、`n = sqrt(G·M_p/a³)` 平均运动、`ω_∥ = ω·û` 为卫星自旋在轨道法向
/// `û = (r×v)/|r×v|` 上的分量。力矩沿 `û`，符号 `(n − ω_∥)`：欠同步时加速、过同步时减速。
///
/// 物理范围与限制（诚实标注，不伪造）：本实现仅施加**卫星自旋**力矩这一自洽、物理
/// 成立的主导项。`central_body` 在当前 `CosmosWorld` 中为静态 `CelestialBody`（无可变
/// 态），故「轨道圆化」所需的角动量反向交换（对伴星施加等大反向力矩）未实现——那需要
/// 伴星本身是动态 n-body 体。完整双体潮汐演化（含轨道半长轴/偏心率演化）留给后续把
/// `central_body` 升级为动态源时再做。默认关闭 → 不影响现有输出（原方法不变）。
///
/// 输入非法（a≤0 / R≤0 / k2≤0 / Q≤0 / 退化轨道）返回零向量。
pub fn tidal_torque(
    position: Vector,
    velocity: Vector,
    body_spin: Vector,
    primary_gm: f64,
    primary_radius: f64,
    body_radius: f64,
    k2: f64,
    q: f64,
) -> Vector {
    let a = position.length();
    if a <= 1e-9 || body_radius <= 0.0 || k2 <= 0.0 || q <= 0.0 || !primary_gm.is_finite() {
        return Vector::ZERO;
    }
    // 必须离开两体表面，避免表面内奇异。
    if a <= primary_radius + body_radius {
        return Vector::ZERO;
    }
    let h_vec = position.cross(velocity);
    let h_mag = (h_vec.x * h_vec.x + h_vec.y * h_vec.y + h_vec.z * h_vec.z).sqrt();
    if h_mag < 1e-20 {
        return Vector::ZERO; // 退化轨道（径向），无定义法向
    }
    let u_hat = h_vec / h_mag; // 轨道法向
    let n = (primary_gm / a.powi(3)).sqrt(); // 平均运动
    let omega_par = body_spin.dot(u_hat); // 自旋沿法向分量
    let a6 = a.powi(6);
    let r5 = body_radius.powi(5);
    let tau_mag = 1.5 * (G * primary_gm * primary_gm * k2 / (q * a6)) * r5 * (n - omega_par);
    u_hat * tau_mag
}

/// Chandrasekhar 动力学摩擦产生的力（N）。当一颗卫星穿过背景弥散介质（如星
/// 系盘中的暗物质晕或星际介质）时，介质粒子的引力拖尾对卫星施加反向减速力：
/// `F = -m_sat · a_df · v̂`，其中
/// `a_df = 4π G² M ρ ln Λ / v²` 由
/// [`galactic_dynamics::chandrasekhar_dynamical_friction`] 给出。
///
/// 与 mps-core 的 `DynamicalFrictionForceLaw` 等价，但 cosmos 自行复用
/// mps-formula 纯函数（不介入 mps-core 的 C ABI / 力律登记表）。
///
/// Inputs:
/// - `body_mass`          — 卫星质量（kg）；a_df 是**自身质量无关**，
///   故力 = m·a_df，质量进入合力系数
/// - `background_density` — 背景介质密度 ρ（kg/m³）
/// - `velocity`           — 卫星相对背景介质的速度（m/s）
/// - `coulomb_log`        — 库仑对数 ln Λ（典型 2–10）
pub fn dynamical_friction_force(
    body_mass: f64,
    background_density: f64,
    velocity: Vector,
    coulomb_log: f64,
) -> Option<Vector> {
    if body_mass <= 0.0 || background_density <= 0.0 || coulomb_log <= 0.0 {
        return None;
    }
    let speed = velocity.length();
    if speed <= 0.0 || !speed.is_finite() {
        // 速度为零 / 退化时摩擦无定义（公式 1/v² 发散），返回 None。
        return None;
    }
    let a_mag =
        gd::chandrasekhar_dynamical_friction(body_mass, background_density, speed, coulomb_log)?;
    // 反向于速度方向施加。
    Some(velocity / speed * (-a_mag) * body_mass)
}

#[inline]
fn to_ffi(v: Vector) -> Vec3 {
    Vec3 {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

#[inline]
fn scale(v: Vec3, s: f64) -> Vector {
    Vector::new(v.x * s, v.y * s, v.z * s)
}
