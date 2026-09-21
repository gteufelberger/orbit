//! Onboard components and the machinery that steps them.
//!
//! A component observes the spacecraft through a [`StepContext`], exchanges
//! power through a [`PowerBus`], and emits telemetry through a
//! [`TelemetrySink`]. Nothing here knows about any specific subsystem, so
//! adding one means implementing [`Component`] and extending
//! [`ComponentConfig`] — no changes to the driver loop.

pub mod battery;
pub mod solar;

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uom::si::f64::{Length, Power, Time, Velocity};
use uom::si::power::watt;

use battery::{Battery, BatteryConfig};
use solar::{SolarPanel, SolarPanelConfig};

/// Everything a component can observe about the spacecraft at one instant.
///
/// Holds `uom` quantities and deliberately derives neither `Serialize` nor
/// `TS`: it is core-only machinery and must never reach JavaScript.
pub struct StepContext {
    /// Length of the step being simulated.
    pub dt: Time,
    /// TEME position.
    pub position: [Length; 3],
    /// TEME velocity.
    pub velocity: [Velocity; 3],
    /// Unit vector towards the Sun.
    pub sun_direction: [f64; 3],
    pub in_eclipse: bool,
}

/// The spacecraft's electrical bus within a single step.
///
/// Lets a solar panel feed a battery without either component holding a
/// reference to the other. Sources run before storage (see
/// [`ExecutionOrder`]), so by the time a battery reads [`PowerBus::net`],
/// every generator and load for that step has already reported.
pub struct PowerBus {
    generated: Power,
    consumed: Power,
}

impl PowerBus {
    pub fn zeroed() -> Self {
        Self {
            generated: Power::new::<watt>(0.0),
            consumed: Power::new::<watt>(0.0),
        }
    }

    /// Reports power put onto the bus, e.g. by a solar array.
    pub fn generate(&mut self, power: Power) {
        self.generated += power;
    }

    /// Reports power drawn from the bus, e.g. by a payload or radio.
    pub fn consume(&mut self, power: Power) {
        self.consumed += power;
    }

    pub fn generated(&self) -> Power {
        self.generated
    }

    pub fn consumed(&self) -> Power {
        self.consumed
    }

    /// Surplus power: positive charges storage, negative discharges it.
    pub fn net(&self) -> Power {
        self.generated - self.consumed
    }
}

/// When a component runs within a step, so that producers report before
/// consumers read.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ExecutionOrder {
    /// Puts power onto the bus. Runs first.
    Source,
    /// Draws power from the bus.
    Load,
    /// Absorbs whatever is left over. Runs last.
    Storage,
}

/// Static description of one telemetry channel, used to label plots.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChannelSpec {
    /// Unique within a satellite, e.g. `"main_battery.state_of_charge"`.
    pub id: String,
    /// Human-readable name for the plot title.
    pub label: String,
    /// Unit symbol for the axis, e.g. `"W"` or `"%"`.
    pub unit: String,
}

/// One channel's full time series over the simulated window.
///
/// `values` is parallel to the sample grid: entry `i` is at
/// `start_unix_seconds + i * step_seconds`.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TelemetryChannel {
    pub spec: ChannelSpec,
    pub values: Vec<f64>,
}

/// Collects the per-step values into one series per channel.
///
/// Storage is column-major because that is the output shape: one contiguous
/// time series per channel. The driver scatters each component's returned
/// values into its own channel span, so a component can only ever write to the
/// channels it declared.
pub struct TelemetrySink {
    series: Vec<Vec<f64>>,
}

impl TelemetrySink {
    pub fn new(channel_count: usize, sample_count: usize) -> Self {
        Self {
            series: vec![Vec::with_capacity(sample_count); channel_count],
        }
    }

    /// Appends one component's values for the current step, starting at the
    /// channel index reserved for that component.
    ///
    /// # Panics
    ///
    /// If the values would run past the end of the sink. The driver checks
    /// arity first so it can name the offending component.
    pub fn record(&mut self, first_channel: usize, values: &[f64]) {
        for (offset, value) in values.iter().enumerate() {
            self.series[first_channel + offset].push(*value);
        }
    }

    /// Pairs the collected series with the specs they belong to.
    pub fn finish(self, specs: Vec<ChannelSpec>) -> Vec<TelemetryChannel> {
        specs
            .into_iter()
            .zip(self.series)
            .map(|(spec, values)| TelemetryChannel { spec, values })
            .collect()
    }
}

/// An onboard subsystem that evolves over the simulation and reports telemetry.
pub trait Component {
    fn id(&self) -> &str;

    /// The channels this component emits, in the order it pushes them.
    fn channels(&self) -> &[ChannelSpec];

    fn execution_order(&self) -> ExecutionOrder;

    /// Advances the component by `ctx.dt`, exchanging power through `bus` and
    /// returning one value per channel, in the order given by
    /// [`Component::channels`].
    ///
    /// Returning the values rather than writing them into a shared cursor keeps
    /// the arity checkable: the driver compares the length against
    /// `channels().len()` and can name the component when they disagree.
    fn step(&mut self, ctx: &StepContext, bus: &mut PowerBus) -> Vec<f64>;
}

/// Serializable description of a component, as authored in the UI.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind")]
pub enum ComponentConfig {
    SolarPanel(SolarPanelConfig),
    Battery(BatteryConfig),
}

impl ComponentConfig {
    pub fn build(&self) -> Box<dyn Component> {
        match self {
            Self::SolarPanel(config) => Box::new(SolarPanel::new(config)),
            Self::Battery(config) => Box::new(Battery::new(config)),
        }
    }
}
