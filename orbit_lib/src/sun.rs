//! Solar geometry: where the Sun is, and whether the Earth is in the way.

use uom::si::f64::Length;
use uom::si::length::meter;

/// Mean equatorial radius of the Earth (WGS84 semi-major axis), in metres.
pub const EARTH_RADIUS_M: f64 = 6_378_137.0;

/// Solar irradiance at 1 AU, in watts per square metre.
pub const SOLAR_CONSTANT_W_PER_M2: f64 = 1361.0;

/// Julian date of the Unix epoch.
const UNIX_EPOCH_JULIAN_DATE: f64 = 2_440_587.5;

/// Julian date of J2000.0.
const J2000_JULIAN_DATE: f64 = 2_451_545.0;

/// Converts Unix seconds to a Julian date.
pub fn julian_date(unix_seconds: f64) -> f64 {
    UNIX_EPOCH_JULIAN_DATE + unix_seconds / 86_400.0
}

/// Unit vector from the centre of the Earth towards the Sun.
///
/// Uses the low-precision almanac series, accurate to roughly 0.01 degrees.
/// That is far beyond what a power budget needs, so the result is used as if it
/// were TEME: the two frames differ by the equation of the equinoxes, under
/// 0.005 degrees.
pub fn direction(unix_seconds: f64) -> [f64; 3] {
    let days_since_j2000 = julian_date(unix_seconds) - J2000_JULIAN_DATE;

    let mean_longitude = (280.460 + 0.985_647_4 * days_since_j2000).to_radians();
    let mean_anomaly = (357.528 + 0.985_600_3 * days_since_j2000).to_radians();

    // Equation of the centre, applied in degrees before converting.
    let ecliptic_longitude = mean_longitude
        + (1.915 * mean_anomaly.sin() + 0.020 * (2.0 * mean_anomaly).sin()).to_radians();
    let obliquity = (23.439 - 0.000_000_4 * days_since_j2000).to_radians();

    [
        ecliptic_longitude.cos(),
        obliquity.cos() * ecliptic_longitude.sin(),
        obliquity.sin() * ecliptic_longitude.sin(),
    ]
}

/// Whether the satellite is inside the Earth's shadow.
///
/// Uses a cylindrical shadow: the Sun is treated as infinitely distant, so the
/// umbra is a cylinder of one Earth radius extending anti-sunward. This ignores
/// the penumbra, which a satellite in low Earth orbit crosses in a few seconds.
pub fn in_eclipse(position: &[Length; 3], sun_direction: &[f64; 3]) -> bool {
    let position_m = position.map(|axis| axis.get::<meter>());

    let distance_along_sun_axis: f64 = (0..3).map(|i| position_m[i] * sun_direction[i]).sum();

    // Anywhere on the sunward side of the terminator plane is lit.
    if distance_along_sun_axis >= 0.0 {
        return false;
    }

    let distance_from_shadow_axis = (0..3)
        .map(|i| (position_m[i] - distance_along_sun_axis * sun_direction[i]).powi(2))
        .sum::<f64>()
        .sqrt();

    distance_from_shadow_axis < EARTH_RADIUS_M
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-09-21T00:00:00Z
    const REFERENCE_TIME: f64 = 1_774_051_200.0;

    fn meters(x: f64, y: f64, z: f64) -> [Length; 3] {
        [
            Length::new::<meter>(x),
            Length::new::<meter>(y),
            Length::new::<meter>(z),
        ]
    }

    #[test]
    fn sun_direction_is_a_unit_vector() {
        let direction = direction(REFERENCE_TIME);
        let norm = direction.iter().map(|c| c * c).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 1e-12, "norm was {norm}");
    }

    #[test]
    fn sun_stays_near_the_ecliptic_plane() {
        // The Sun's declination never exceeds the obliquity, ~23.44 degrees,
        // so the z component is bounded by sin(23.44 deg).
        for day in 0..365 {
            let direction = direction(REFERENCE_TIME + f64::from(day) * 86_400.0);
            assert!(direction[2].abs() <= 23.44_f64.to_radians().sin() + 1e-6);
        }
    }

    #[test]
    fn directly_behind_the_earth_is_eclipse() {
        let sun = [1.0, 0.0, 0.0];
        assert!(in_eclipse(&meters(-7_000_000.0, 0.0, 0.0), &sun));
    }

    #[test]
    fn sunward_side_is_never_eclipse() {
        let sun = [1.0, 0.0, 0.0];
        assert!(!in_eclipse(&meters(7_000_000.0, 0.0, 0.0), &sun));
    }

    #[test]
    fn beside_the_shadow_cylinder_is_lit() {
        let sun = [1.0, 0.0, 0.0];
        // Anti-sunward, but further off-axis than the Earth is wide.
        assert!(!in_eclipse(&meters(-7_000_000.0, 7_000_000.0, 0.0), &sun));
    }
}
