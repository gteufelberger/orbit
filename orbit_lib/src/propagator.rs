//! SGP4 propagation, wrapped so the rest of the crate only sees `uom` quantities.

use sgp4::chrono::{DateTime, NaiveDateTime};
use sgp4::{Constants, Elements, MinutesSinceEpoch};
use uom::si::f64::{Length, Velocity};
use uom::si::length::kilometer;
use uom::si::velocity::kilometer_per_second;

/// A satellite's position and velocity at one instant, in the TEME frame.
///
/// TEME is what SGP4 natively produces. It is deliberately *not* converted to
/// an Earth-fixed frame here: Cesium already ships
/// `Transforms.computeTemeToPseudoFixedMatrix`, so doing it in Rust would mean
/// reimplementing something the viewer does correctly.
pub struct State {
    pub position: [Length; 3],
    pub velocity: [Velocity; 3],
}

#[derive(Debug)]
pub enum PropagatorError {
    /// The TLE did not contain two usable element lines.
    MalformedTle { detail: String },
    /// SGP4 rejected the elements or could not propagate to the requested time.
    Propagation { detail: String },
}

impl core::fmt::Display for PropagatorError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MalformedTle { detail } => write!(formatter, "malformed TLE: {detail}"),
            Self::Propagation { detail } => write!(formatter, "propagation failed: {detail}"),
        }
    }
}

impl std::error::Error for PropagatorError {}

pub struct Propagator {
    elements: Elements,
    constants: Constants,
}

impl Propagator {
    /// Parses a TLE given as one string with the two element lines separated by
    /// newlines. An optional leading title line is ignored.
    pub fn from_tle(name: Option<String>, tle: &str) -> Result<Self, PropagatorError> {
        let lines: Vec<&str> = tle
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();

        let (line1, line2) = match lines.as_slice() {
            [line1, line2] => (*line1, *line2),
            // Tolerate the three-line form, where the first line is a title.
            [_, line1, line2] => (*line1, *line2),
            other => {
                return Err(PropagatorError::MalformedTle {
                    detail: format!("expected 2 element lines, found {}", other.len()),
                });
            }
        };

        let elements =
            Elements::from_tle(name, line1.as_bytes(), line2.as_bytes()).map_err(|error| {
                PropagatorError::MalformedTle {
                    detail: error.to_string(),
                }
            })?;

        let constants =
            Constants::from_elements(&elements).map_err(|error| PropagatorError::Propagation {
                detail: error.to_string(),
            })?;

        Ok(Self {
            elements,
            constants,
        })
    }

    /// Propagates to an absolute time given in Unix seconds.
    ///
    /// SGP4 works in minutes since the TLE's *own* epoch, which differs per
    /// satellite, so the conversion has to go through this propagator's
    /// elements rather than any shared reference time.
    pub fn state_at(&self, unix_seconds: f64) -> Result<State, PropagatorError> {
        let datetime = unix_seconds_to_naive_datetime(unix_seconds)?;

        let minutes_since_epoch = self
            .elements
            .datetime_to_minutes_since_epoch(&datetime)
            .map_err(|error| PropagatorError::Propagation {
                detail: error.to_string(),
            })?;

        self.state_at_minutes_since_epoch(minutes_since_epoch)
    }

    fn state_at_minutes_since_epoch(
        &self,
        minutes_since_epoch: MinutesSinceEpoch,
    ) -> Result<State, PropagatorError> {
        let prediction = self
            .constants
            .propagate(minutes_since_epoch)
            .map_err(|error| PropagatorError::Propagation {
                detail: error.to_string(),
            })?;

        Ok(State {
            position: prediction.position.map(Length::new::<kilometer>),
            velocity: prediction
                .velocity
                .map(Velocity::new::<kilometer_per_second>),
        })
    }
}

fn unix_seconds_to_naive_datetime(unix_seconds: f64) -> Result<NaiveDateTime, PropagatorError> {
    let seconds = unix_seconds.trunc();
    let nanoseconds = ((unix_seconds - seconds) * 1e9).round();

    DateTime::from_timestamp(seconds as i64, nanoseconds as u32)
        .map(|datetime| datetime.naive_utc())
        .ok_or_else(|| PropagatorError::Propagation {
            detail: format!("{unix_seconds} is not a representable timestamp"),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uom::si::length::meter;

    const LANDSAT_9_TLE: &str = "1 49260U 21088A   26264.47165275  .00000170  00000-0  47773-4 0  9997\n2 49260  98.2196 333.4388 0001506  93.9087 266.2284 14.57100580265024";

    fn altitude_m(state: &State) -> f64 {
        state
            .position
            .iter()
            .map(|axis| axis.get::<meter>().powi(2))
            .sum::<f64>()
            .sqrt()
            - crate::sun::EARTH_RADIUS_M
    }

    #[test]
    fn propagates_landsat_9_to_its_documented_altitude() {
        let propagator = Propagator::from_tle(Some("Landsat 9".to_owned()), LANDSAT_9_TLE).unwrap();

        // The TLE epoch is day 264.47 of 2026, i.e. 2026-09-21T11:19Z.
        let state = propagator.state_at(1_774_092_000.0).unwrap();

        // Landsat 9 flies a near-circular sun-synchronous orbit at ~705 km.
        let altitude_km = altitude_m(&state) / 1000.0;
        assert!(
            (690.0..720.0).contains(&altitude_km),
            "altitude was {altitude_km} km"
        );
    }

    #[test]
    fn altitude_stays_bounded_across_a_full_orbit() {
        let propagator = Propagator::from_tle(None, LANDSAT_9_TLE).unwrap();

        for minute in 0..100 {
            let state = propagator
                .state_at(1_774_092_000.0 + f64::from(minute) * 60.0)
                .unwrap();
            let altitude_km = altitude_m(&state) / 1000.0;
            assert!(
                (680.0..730.0).contains(&altitude_km),
                "altitude was {altitude_km} km at minute {minute}"
            );
        }
    }

    #[test]
    fn accepts_the_three_line_form() {
        let with_title = format!("LANDSAT 9\n{LANDSAT_9_TLE}");
        assert!(Propagator::from_tle(None, &with_title).is_ok());
    }

    #[test]
    fn rejects_a_single_line() {
        assert!(matches!(
            Propagator::from_tle(None, "1 49260U 21088A"),
            Err(PropagatorError::MalformedTle { .. })
        ));
    }
}
