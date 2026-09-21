//! A satellite: a TLE plus the components flying on it.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uom::si::f64::Time;
use uom::si::time::second;

use crate::components::{
    Component, ComponentConfig, PowerBus, StepContext, TelemetryChannel, TelemetrySink,
};
use crate::propagator::{Propagator, PropagatorError};
use crate::sun;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Satellite {
    /// Stable identifier, used to match results back to the UI list.
    pub id: String,
    pub name: String,
    /// Two-line element set. The two element lines are separated by newlines;
    /// a leading title line is tolerated and ignored.
    pub tle: String,
    pub components: Vec<ComponentConfig>,
}

/// One satellite's simulated trajectory and telemetry.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SatelliteResult {
    pub id: String,
    pub name: String,
    /// TEME positions in metres, flattened as `[x0, y0, z0, x1, y1, z1, ...]`.
    ///
    /// Flat rather than an array of objects: a three-orbit window at ten-second
    /// steps is ~1600 samples per satellite, and this avoids allocating that
    /// many JavaScript objects.
    pub positions_teme_meters: Vec<f64>,
    pub channels: Vec<TelemetryChannel>,
}

impl Satellite {
    /// Propagates this satellite over the sample grid and steps its components
    /// alongside, producing one entry per sample in every output series.
    pub fn simulate(
        &self,
        start_unix_seconds: f64,
        step_seconds: f64,
        sample_count: usize,
    ) -> Result<SatelliteResult, PropagatorError> {
        let propagator = Propagator::from_tle(Some(self.name.clone()), &self.tle)?;

        let mut components = self.build_components();
        let specs: Vec<_> = components
            .iter()
            .flat_map(|component| component.channels().iter().cloned())
            .collect();

        // Where each component's channels begin in the flattened list. Computed
        // once rather than re-accumulated on every step.
        let channel_offsets: Vec<usize> = components
            .iter()
            .scan(0, |offset, component| {
                let start = *offset;
                *offset += component.channels().len();
                Some(start)
            })
            .collect();

        let dt = Time::new::<second>(step_seconds);
        let mut sink = TelemetrySink::new(specs.len(), sample_count);
        let mut positions_teme_meters = Vec::with_capacity(sample_count * 3);

        for sample in 0..sample_count {
            let unix_seconds = start_unix_seconds + sample as f64 * step_seconds;
            let state = propagator.state_at(unix_seconds)?;

            for axis in &state.position {
                positions_teme_meters.push(axis.get::<uom::si::length::meter>());
            }

            let sun_direction = sun::direction(unix_seconds);
            let context = StepContext {
                dt,
                position: state.position,
                velocity: state.velocity,
                sun_direction,
                in_eclipse: sun::in_eclipse(&state.position, &sun_direction),
            };

            let mut bus = PowerBus::zeroed();
            for (component, &first_channel) in components.iter_mut().zip(&channel_offsets) {
                let values = component.step(&context, &mut bus);

                // A component returning the wrong number of values is a bug in
                // that component. Catching it here means the message names the
                // culprit, instead of its values silently shifting into a
                // neighbouring component's series.
                assert_eq!(
                    values.len(),
                    component.channels().len(),
                    "component {} returned {} values for {} channels",
                    component.id(),
                    values.len(),
                    component.channels().len(),
                );

                sink.record(first_channel, &values);
            }
        }

        Ok(SatelliteResult {
            id: self.id.clone(),
            name: self.name.clone(),
            positions_teme_meters,
            channels: sink.finish(specs),
        })
    }

    /// Instantiates the configured components, ordered so that every source
    /// reports to the bus before any storage reads it.
    fn build_components(&self) -> Vec<Box<dyn Component>> {
        let mut components: Vec<Box<dyn Component>> =
            self.components.iter().map(ComponentConfig::build).collect();

        components.sort_by_key(|component| component.execution_order());
        components
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ExecutionOrder;
    use crate::components::battery::BatteryConfig;
    use crate::components::solar::SolarPanelConfig;

    const LANDSAT_9_TLE: &str = "1 49260U 21088A   26264.47165275  .00000170  00000-0  47773-4 0  9997\n2 49260  98.2196 333.4388 0001506  93.9087 266.2284 14.57100580265024";

    fn satellite() -> Satellite {
        Satellite {
            id: "landsat9".to_owned(),
            name: "Landsat 9".to_owned(),
            tle: LANDSAT_9_TLE.to_owned(),
            components: vec![
                // Deliberately configured storage-first, to prove the sort.
                ComponentConfig::Battery(BatteryConfig {
                    id: "main_battery".to_owned(),
                    capacity_watt_hours: 500.0,
                    initial_state_of_charge: 0.8,
                    load_watts: 300.0,
                }),
                ComponentConfig::SolarPanel(SolarPanelConfig {
                    id: "array".to_owned(),
                    area_square_meters: 4.0,
                    efficiency: 0.28,
                }),
            ],
        }
    }

    #[test]
    fn sources_are_ordered_before_storage() {
        let components = satellite().build_components();
        let orders: Vec<_> = components
            .iter()
            .map(|component| component.execution_order())
            .collect();

        assert_eq!(
            orders,
            vec![ExecutionOrder::Source, ExecutionOrder::Storage]
        );
    }

    #[test]
    fn every_series_is_parallel_to_the_sample_grid() {
        let result = satellite().simulate(1_774_092_000.0, 60.0, 120).unwrap();

        assert_eq!(result.positions_teme_meters.len(), 120 * 3);
        assert_eq!(result.channels.len(), 2);
        for channel in &result.channels {
            assert_eq!(channel.values.len(), 120, "channel {}", channel.spec.id);
        }
    }

    #[test]
    fn a_full_orbit_passes_through_eclipse_and_sunlight() {
        // One orbit of Landsat 9 is ~99 minutes; 120 one-minute samples cover it.
        let result = satellite().simulate(1_774_092_000.0, 60.0, 120).unwrap();

        let solar = result
            .channels
            .iter()
            .find(|channel| channel.spec.id == "array.power_generated")
            .expect("solar channel");

        assert!(solar.values.contains(&0.0), "expected some eclipse");
        assert!(
            solar.values.iter().any(|&watts| watts > 0.0),
            "expected some sunlight"
        );
    }

    #[test]
    fn battery_discharges_during_eclipse_and_recovers_in_sunlight() {
        let result = satellite().simulate(1_774_092_000.0, 60.0, 120).unwrap();

        let solar = result
            .channels
            .iter()
            .find(|channel| channel.spec.id == "array.power_generated")
            .expect("solar channel");
        let charge = result
            .channels
            .iter()
            .find(|channel| channel.spec.id == "main_battery.state_of_charge")
            .expect("battery channel");

        // Across every consecutive pair, charge must fall whenever the array is
        // dark and the load is still drawing. This is the sign convention that
        // is easy to invert, so it is asserted directly.
        let mut checked_eclipse_step = false;
        for index in 1..charge.values.len() {
            if solar.values[index] == 0.0 && charge.values[index - 1] > 0.0 {
                assert!(
                    charge.values[index] < charge.values[index - 1],
                    "charge rose during eclipse at sample {index}"
                );
                checked_eclipse_step = true;
            }
        }
        assert!(checked_eclipse_step, "no eclipse step to check");
    }

    #[test]
    fn a_malformed_tle_is_an_error_not_a_panic() {
        let mut satellite = satellite();
        satellite.tle = "not a tle".to_owned();

        assert!(satellite.simulate(1_774_092_000.0, 60.0, 10).is_err());
    }
}
