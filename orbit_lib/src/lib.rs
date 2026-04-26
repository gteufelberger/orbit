use std::f64::consts::PI;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn hello_world() -> String {
    "Hello from WebAssembly! 🚀".to_string()
}

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

const GM_EARTH: f64 = 3.986004418e14; // m³/s²
const EARTH_ROTATION_RATE: f64 = 7.2921150e-5; // rad/s (sidereal)
// GMST at J2000.0 (Jan 1.5, 2000 UTC) in radians
const GMST_J2000: f64 = 4.894961212823756;
// Seconds from Unix epoch (Jan 1.0, 1970) to J2000.0 (Jan 1.5, 2000)
const UNIX_TO_J2000: f64 = 946_728_000.0;

/// Compute orbital period from semi-major axis using Kepler's third law: T = 2π√(a³/GM)
#[wasm_bindgen]
pub fn orbital_period(semi_major_axis: f64) -> f64 {
    2.0 * PI * (semi_major_axis.powi(3) / GM_EARTH).sqrt()
}

/// Solve Kepler's equation M = E - e·sin(E) via Newton-Raphson
fn solve_kepler(mean_anomaly: f64, eccentricity: f64) -> f64 {
    let m = mean_anomaly.rem_euclid(2.0 * PI);
    let mut e = m;
    for _ in 0..50 {
        let de = (m - e + eccentricity * e.sin()) / (1.0 - eccentricity * e.cos());
        e += de;
        if de.abs() < 1e-12 {
            break;
        }
    }
    e
}

/// Compute ECI position from Keplerian orbital elements at a given Unix time
fn keplerian_to_eci(
    semi_major_axis: f64,
    eccentricity: f64,
    inclination: f64,
    raan: f64,
    arg_periapsis: f64,
    mean_anomaly_epoch: f64,
    epoch_unix: f64,
    time_unix: f64,
) -> [f64; 3] {
    let n = (GM_EARTH / semi_major_axis.powi(3)).sqrt();
    let mean_anomaly = mean_anomaly_epoch + n * (time_unix - epoch_unix);

    let ecc_anomaly = solve_kepler(mean_anomaly, eccentricity);
    let true_anomaly = 2.0 * f64::atan2(
        (1.0 + eccentricity).sqrt() * (ecc_anomaly / 2.0).sin(),
        (1.0 - eccentricity).sqrt() * (ecc_anomaly / 2.0).cos(),
    );

    let r = semi_major_axis * (1.0 - eccentricity * ecc_anomaly.cos());
    let x_pf = r * true_anomaly.cos();
    let y_pf = r * true_anomaly.sin();

    // Rotate perifocal → ECI using rotation matrix R(-Ω) · R(-i) · R(-ω)
    let (si, ci) = inclination.sin_cos();
    let (sra, cra) = raan.sin_cos();
    let (sa, ca) = arg_periapsis.sin_cos();

    [
        (cra * ca - sra * sa * ci) * x_pf + (-cra * sa - sra * ca * ci) * y_pf,
        (sra * ca + cra * sa * ci) * x_pf + (-sra * sa + cra * ca * ci) * y_pf,
        sa * si * x_pf + ca * si * y_pf,
    ]
}

fn eci_to_ecef(eci: [f64; 3], time_unix: f64) -> [f64; 3] {
    let gmst = GMST_J2000 + EARTH_ROTATION_RATE * (time_unix - UNIX_TO_J2000);
    let (sg, cg) = gmst.sin_cos();
    [
        eci[0] * cg + eci[1] * sg,
        -eci[0] * sg + eci[1] * cg,
        eci[2],
    ]
}

fn keplerian_to_ecef(
    semi_major_axis: f64,
    eccentricity: f64,
    inclination: f64,
    raan: f64,
    arg_periapsis: f64,
    mean_anomaly_epoch: f64,
    epoch_unix: f64,
    time_unix: f64,
) -> [f64; 3] {
    eci_to_ecef(
        keplerian_to_eci(semi_major_axis, eccentricity, inclination, raan, arg_periapsis, mean_anomaly_epoch, epoch_unix, time_unix),
        time_unix,
    )
}

/// Calculate satellite position in ECEF from Keplerian elements.
/// Returns [x, y, z] in meters.
///
/// # Arguments
/// * `semi_major_axis` - Semi-major axis in meters
/// * `eccentricity` - Orbital eccentricity (0 = circular)
/// * `inclination` - Inclination in radians
/// * `raan` - Right ascension of ascending node in radians
/// * `arg_periapsis` - Argument of periapsis in radians
/// * `mean_anomaly_epoch` - Mean anomaly at epoch in radians
/// * `epoch_unix` - Epoch as Unix timestamp in seconds
/// * `time_unix` - Current time as Unix timestamp in seconds
#[wasm_bindgen]
pub fn calculate_position_keplerian(
    semi_major_axis: f64,
    eccentricity: f64,
    inclination: f64,
    raan: f64,
    arg_periapsis: f64,
    mean_anomaly_epoch: f64,
    epoch_unix: f64,
    time_unix: f64,
) -> Vec<f64> {
    let pos = keplerian_to_ecef(
        semi_major_axis,
        eccentricity,
        inclination,
        raan,
        arg_periapsis,
        mean_anomaly_epoch,
        epoch_unix,
        time_unix,
    );
    vec![pos[0], pos[1], pos[2]]
}

/// Compute orbit path positions as a flat [x,y,z,...] array.
/// Samples `num_points + 1` points over one period (last = first, closed loop).
///
/// `inertial`: if true, returns ECI coordinates (ICRF frame); the caller is
/// responsible for applying a single ECI→ECEF rotation for the current time.
/// If false, each point is converted to ECEF at its own simulation time,
/// which produces the correct ground-track shape in Earth-fixed coordinates.
#[wasm_bindgen]
pub fn calculate_orbit_path(
    semi_major_axis: f64,
    eccentricity: f64,
    inclination: f64,
    raan: f64,
    arg_periapsis: f64,
    mean_anomaly_epoch: f64,
    epoch_unix: f64,
    time_unix: f64,
    num_points: usize,
    inertial: bool,
) -> Vec<f64> {
    let period = orbital_period(semi_major_axis);
    let mut result = Vec::with_capacity((num_points + 1) * 3);

    for i in 0..=num_points {
        let t = time_unix + (i as f64 / num_points as f64) * period;
        let pos = if inertial {
            keplerian_to_eci(semi_major_axis, eccentricity, inclination, raan, arg_periapsis, mean_anomaly_epoch, epoch_unix, t)
        } else {
            keplerian_to_ecef(semi_major_axis, eccentricity, inclination, raan, arg_periapsis, mean_anomaly_epoch, epoch_unix, t)
        };
        result.push(pos[0]);
        result.push(pos[1]);
        result.push(pos[2]);
    }

    result
}
