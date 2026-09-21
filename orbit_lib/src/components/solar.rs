//! Solar array: the only power source in the model.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uom::si::area::square_meter;
use uom::si::f64::{Area, Power, Ratio};
use uom::si::power::watt;
use uom::si::ratio::ratio;

use super::{ChannelSpec, Component, ExecutionOrder, PowerBus, StepContext};
use crate::sun::SOLAR_CONSTANT_W_PER_M2;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SolarPanelConfig {
    /// Unique within a satellite.
    pub id: String,
    /// Total array area, in square metres.
    pub area_square_meters: f64,
    /// Conversion efficiency, as a fraction in `[0, 1]`.
    pub efficiency: f64,
}

/// A sun-tracking solar array.
///
/// The array is assumed to be articulated and always pointed at the Sun, which
/// is how the satellites this models (Landsat 9, FLEX) actually fly. That makes
/// the incidence angle zero whenever the satellite is lit, so no attitude model
/// is needed — output is either full or nothing.
pub struct SolarPanel {
    id: String,
    area: Area,
    efficiency: Ratio,
    channels: Vec<ChannelSpec>,
}

impl SolarPanel {
    pub fn new(config: &SolarPanelConfig) -> Self {
        Self {
            id: config.id.clone(),
            area: Area::new::<square_meter>(config.area_square_meters),
            efficiency: Ratio::new::<ratio>(config.efficiency),
            channels: vec![ChannelSpec {
                id: format!("{}.power_generated", config.id),
                label: "Solar power".to_owned(),
                unit: "W".to_owned(),
            }],
        }
    }

    fn output(&self, in_eclipse: bool) -> Power {
        if in_eclipse {
            Power::new::<watt>(0.0)
        } else {
            // Irradiance x area x efficiency. Writing the irradiance as a bare
            // f64 multiplier keeps this dimensionally honest: W/m^2 x m^2 = W.
            Power::new::<watt>(SOLAR_CONSTANT_W_PER_M2 * self.area.get::<square_meter>())
                * self.efficiency
        }
    }
}

impl Component for SolarPanel {
    fn id(&self) -> &str {
        &self.id
    }

    fn channels(&self) -> &[ChannelSpec] {
        &self.channels
    }

    fn execution_order(&self) -> ExecutionOrder {
        ExecutionOrder::Source
    }

    fn step(&mut self, ctx: &StepContext, bus: &mut PowerBus) -> Vec<f64> {
        let generated = self.output(ctx.in_eclipse);
        bus.generate(generated);
        vec![generated.get::<watt>()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel() -> SolarPanel {
        SolarPanel::new(&SolarPanelConfig {
            id: "array".to_owned(),
            area_square_meters: 10.0,
            efficiency: 0.3,
        })
    }

    #[test]
    fn generates_nothing_in_eclipse() {
        assert_eq!(panel().output(true).get::<watt>(), 0.0);
    }

    #[test]
    fn generates_irradiance_times_area_times_efficiency_when_lit() {
        let expected = SOLAR_CONSTANT_W_PER_M2 * 10.0 * 0.3;
        assert!((panel().output(false).get::<watt>() - expected).abs() < 1e-9);
    }
}
