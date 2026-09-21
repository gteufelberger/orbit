//! Battery: the only energy store in the model.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uom::si::energy::watt_hour;
use uom::si::f64::{Energy, Power, Ratio};
use uom::si::power::watt;
use uom::si::ratio::percent;

use super::{ChannelSpec, Component, ExecutionOrder, PowerBus, StepContext};

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BatteryConfig {
    /// Unique within a satellite.
    pub id: String,
    /// Usable capacity, in watt-hours.
    pub capacity_watt_hours: f64,
    /// Charge at the start of the simulation, as a fraction of capacity in `[0, 1]`.
    pub initial_state_of_charge: f64,
    /// Constant housekeeping load drawn by the rest of the spacecraft, in watts.
    pub load_watts: f64,
}

/// A battery with a constant bus load.
///
/// Runs with [`ExecutionOrder::Storage`], so by the time [`Component::step`] is
/// called every source for this step has already reported to the bus.
pub struct Battery {
    id: String,
    capacity: Energy,
    charge: Energy,
    load: Power,
    channels: Vec<ChannelSpec>,
}

impl Battery {
    pub fn new(config: &BatteryConfig) -> Self {
        let capacity = Energy::new::<watt_hour>(config.capacity_watt_hours);

        Self {
            id: config.id.clone(),
            capacity,
            charge: capacity * config.initial_state_of_charge.clamp(0.0, 1.0),
            load: Power::new::<watt>(config.load_watts),
            channels: vec![ChannelSpec {
                id: format!("{}.state_of_charge", config.id),
                label: "Battery charge".to_owned(),
                unit: "%".to_owned(),
            }],
        }
    }

    /// Charge as a fraction of capacity.
    pub fn state_of_charge(&self) -> Ratio {
        self.charge / self.capacity
    }
}

impl Component for Battery {
    fn id(&self) -> &str {
        &self.id
    }

    fn channels(&self) -> &[ChannelSpec] {
        &self.channels
    }

    fn execution_order(&self) -> ExecutionOrder {
        ExecutionOrder::Storage
    }

    fn step(&mut self, ctx: &StepContext, bus: &mut PowerBus) -> Vec<f64> {
        // The spacecraft's housekeeping load is drawn continuously, in sunlight
        // and in eclipse alike.
        bus.consume(self.load);

        // Power x time is an energy, so this is the whole of the physics.
        // Surplus on the bus charges, a deficit discharges.
        let empty = Energy::new::<watt_hour>(0.0);
        self.charge = (self.charge + bus.net() * ctx.dt)
            .max(empty)
            .min(self.capacity);

        vec![self.state_of_charge().get::<percent>()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uom::si::f64::{Length, Time, Velocity};
    use uom::si::length::meter;
    use uom::si::time::second;
    use uom::si::velocity::meter_per_second;

    fn battery(initial_state_of_charge: f64, load_watts: f64) -> Battery {
        Battery::new(&BatteryConfig {
            id: "main".to_owned(),
            capacity_watt_hours: 100.0,
            initial_state_of_charge,
            load_watts,
        })
    }

    /// One hour, so a watt of imbalance moves exactly one watt-hour.
    fn one_hour() -> StepContext {
        StepContext {
            dt: Time::new::<second>(3600.0),
            position: [Length::new::<meter>(7_000_000.0); 3],
            velocity: [Velocity::new::<meter_per_second>(0.0); 3],
            sun_direction: [1.0, 0.0, 0.0],
            in_eclipse: false,
        }
    }

    /// Steps the battery once with `generated` watts already on the bus,
    /// returning the state of charge it reported.
    fn step_once(battery: &mut Battery, generated_watts: f64) -> f64 {
        let mut bus = PowerBus::zeroed();
        bus.generate(Power::new::<watt>(generated_watts));

        let values = battery.step(&one_hour(), &mut bus);
        assert_eq!(values.len(), battery.channels().len());

        values[0]
    }

    #[test]
    fn discharges_under_load_with_no_generation() {
        let mut battery = battery(0.5, 10.0);
        step_once(&mut battery, 0.0);

        // 10 W for one hour out of 100 Wh: 50 Wh -> 40 Wh.
        assert!((battery.charge.get::<watt_hour>() - 40.0).abs() < 1e-9);
    }

    #[test]
    fn charges_when_generation_exceeds_load() {
        let mut battery = battery(0.5, 10.0);
        step_once(&mut battery, 30.0);

        // Net +20 W for one hour: 50 Wh -> 70 Wh.
        assert!((battery.charge.get::<watt_hour>() - 70.0).abs() < 1e-9);
    }

    #[test]
    fn never_charges_beyond_capacity() {
        let mut battery = battery(0.95, 10.0);
        step_once(&mut battery, 1000.0);

        assert!(battery.charge <= battery.capacity);
        assert!((battery.state_of_charge().get::<percent>() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn never_discharges_below_empty() {
        let mut battery = battery(0.01, 500.0);
        step_once(&mut battery, 0.0);

        assert!(battery.charge.get::<watt_hour>() >= 0.0);
        assert!((battery.state_of_charge().get::<percent>()).abs() < 1e-9);
    }

    #[test]
    fn reports_state_of_charge_as_a_percentage() {
        let mut battery = battery(0.5, 0.0);
        let reported = step_once(&mut battery, 0.0);

        assert!(
            (reported - 50.0).abs() < 1e-9,
            "expected ~50 (percent), got {reported}"
        );
    }
}
