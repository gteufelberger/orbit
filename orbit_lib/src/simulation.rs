//! The simulation driver: one call covers a whole time window.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::propagator::PropagatorError;
use crate::satellite::{Satellite, SatelliteResult};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SimulationConfig {
    pub satellites: Vec<Satellite>,
    /// Start of the window, in Unix seconds.
    pub start_unix_seconds: f64,
    /// Length of the window, in seconds.
    pub duration_seconds: f64,
    /// Spacing between samples, in seconds.
    pub step_seconds: f64,
}

/// Simulated trajectories and telemetry for every satellite in the request.
///
/// Every series is parallel to the same sample grid: entry `i` is at
/// `start_unix_seconds + i * step_seconds`.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SimulationResult {
    pub start_unix_seconds: f64,
    pub step_seconds: f64,
    pub sample_count: u32,
    pub satellites: Vec<SatelliteResult>,
}

#[derive(Debug)]
pub enum SimulationError {
    /// A window or step size that does not describe a usable sample grid.
    InvalidWindow { detail: String },
    /// A satellite failed to propagate.
    Satellite { id: String, source: PropagatorError },
}

impl core::fmt::Display for SimulationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidWindow { detail } => write!(formatter, "invalid window: {detail}"),
            Self::Satellite { id, source } => write!(formatter, "satellite {id}: {source}"),
        }
    }
}

impl std::error::Error for SimulationError {}

impl SimulationConfig {
    /// Number of samples in the window, inclusive of the start.
    ///
    /// Validates the grid rather than saturating, so a bad window surfaces as
    /// an error message in the UI instead of an empty plot.
    pub fn sample_count(&self) -> Result<usize, SimulationError> {
        if !self.step_seconds.is_finite() || self.step_seconds <= 0.0 {
            return Err(SimulationError::InvalidWindow {
                detail: format!("step must be positive, got {}", self.step_seconds),
            });
        }
        if !self.duration_seconds.is_finite() || self.duration_seconds <= 0.0 {
            return Err(SimulationError::InvalidWindow {
                detail: format!("duration must be positive, got {}", self.duration_seconds),
            });
        }
        if !self.start_unix_seconds.is_finite() {
            return Err(SimulationError::InvalidWindow {
                detail: "start must be finite".to_owned(),
            });
        }

        // One million samples per satellite is already far past what the viewer
        // can usefully interpolate; beyond it we would just exhaust memory.
        const MAX_SAMPLES: f64 = 1_000_000.0;
        let count = (self.duration_seconds / self.step_seconds).floor() + 1.0;
        if count > MAX_SAMPLES {
            return Err(SimulationError::InvalidWindow {
                detail: format!("{count} samples exceeds the limit of {MAX_SAMPLES}"),
            });
        }

        Ok(count as usize)
    }
}

/// Runs every satellite in the configuration over the same sample grid.
pub fn run(config: &SimulationConfig) -> Result<SimulationResult, SimulationError> {
    let sample_count = config.sample_count()?;

    let satellites = config
        .satellites
        .iter()
        .map(|satellite| {
            satellite
                .simulate(config.start_unix_seconds, config.step_seconds, sample_count)
                .map_err(|source| SimulationError::Satellite {
                    id: satellite.id.clone(),
                    source,
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SimulationResult {
        start_unix_seconds: config.start_unix_seconds,
        step_seconds: config.step_seconds,
        sample_count: sample_count as u32,
        satellites,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(duration_seconds: f64, step_seconds: f64) -> SimulationConfig {
        SimulationConfig {
            satellites: Vec::new(),
            start_unix_seconds: 1_774_092_000.0,
            duration_seconds,
            step_seconds,
        }
    }

    #[test]
    fn sample_count_is_inclusive_of_the_start() {
        assert_eq!(config(100.0, 10.0).sample_count().unwrap(), 11);
    }

    #[test]
    fn rejects_a_zero_step() {
        assert!(config(100.0, 0.0).sample_count().is_err());
    }

    #[test]
    fn rejects_a_negative_duration() {
        assert!(config(-100.0, 10.0).sample_count().is_err());
    }

    #[test]
    fn rejects_a_grid_that_would_exhaust_memory() {
        assert!(config(1e12, 0.001).sample_count().is_err());
    }

    #[test]
    fn rejects_a_non_finite_step() {
        assert!(config(100.0, f64::NAN).sample_count().is_err());
    }
}
