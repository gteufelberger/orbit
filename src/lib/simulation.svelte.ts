import type { Satellite } from "$bindings/Satellite";
import type { SimulationResult } from "$bindings/SimulationResult";
import { getWasm } from "$lib/wasm";

/** Roughly one orbit for a satellite in low Earth orbit. */
const ORBIT_SECONDS = 99 * 60;

export type SimulationStatus = "idle" | "running" | "error";

interface SimulationState {
  satellites: Satellite[];
  /** Result of the most recent successful run, or null before the first one. */
  result: SimulationResult | null;
  /**
   * Cesium's current clock time, in Unix seconds, or null before a run.
   *
   * Published by CesiumViewer so the plots can mark where the globe is in the
   * window. The viewer owns the clock, so this only ever flows outwards.
   */
  current_unix_seconds: number | null;
  status: SimulationStatus;
  error: string | null;
}

/**
 * Simulation state shared across the whole app.
 *
 * Controls, SatelliteList, SatelliteHolder and CesiumViewer all need this, but
 * they are not in an ancestor relationship: Controls and CesiumViewer meet only
 * at the page, with the Splitter in between. Module-scope `$state` avoids
 * threading props through a component whose only job is dragging a divider.
 *
 * Exported as an object rather than a reassignable `let` because exported
 * bindings are not reactive across module boundaries; mutating fields is.
 */
export const sim: SimulationState = $state({
  satellites: [
    {
      name: "Landsat 9",
      id: "landsat9",
      tle: "1 49260U 21088A   26264.47165275  .00000170  00000-0  47773-4 0  9997\n2 49260  98.2196 333.4388 0001506  93.9087 266.2284 14.57100580265024",
      components: [
        {
          kind: "SolarPanel",
          id: "array",
          area_square_meters: 11,
          efficiency: 0.28,
        },
        {
          kind: "Battery",
          id: "main_battery",
          capacity_watt_hours: 4000,
          initial_state_of_charge: 0.9,
          load_watts: 1550,
        },
      ],
    },
    {
      name: "FLEX",
      id: "flex",
      tle: "1 A0690U 26209B   26264.59238340  .00000241  00000-0  12522-3 0  9990\n2 A0690  98.6262 330.6436 0001976  88.7260 271.4146 14.23497854   911",
      components: [
        {
          kind: "SolarPanel",
          id: "array",
          area_square_meters: 4,
          efficiency: 0.28,
        },
        {
          kind: "Battery",
          id: "main_battery",
          capacity_watt_hours: 1000,
          initial_state_of_charge: 0.9,
          load_watts: 500,
        },
      ],
    },
    {
      name: "ISS",
      id: "iss",
      tle: "1 25544U 98067A   26264.49719541  .00007274  00000-0  13903-3 0  9999\n2 25544  51.6310 183.4357 0004788 164.7753 195.3381 15.49212079586682",
      components: [
        {
          kind: "SolarPanel",
          id: "array",
          area_square_meters: 4,
          efficiency: 0.28,
        },
        {
          kind: "Battery",
          id: "main_battery",
          capacity_watt_hours: 1000,
          initial_state_of_charge: 0.9,
          load_watts: 500,
        },
      ],
    },
  ],
  result: null,
  current_unix_seconds: null,
  status: "idle",
  error: null,
});

/**
 * Simulates every satellite over one window and publishes the result.
 *
 * One call covers the whole window rather than stepping per frame: Cesium
 * interpolates between the samples on its own clock, and the plots need the
 * full series anyway.
 */
export async function runSimulation(
  durationSeconds = 3 * ORBIT_SECONDS,
  stepSeconds = 10,
): Promise<void> {
  sim.status = "running";
  sim.error = null;

  try {
    const wasm = await getWasm();

    sim.result = wasm.simulate({
      // Snapshot because serde reads this through plain property access, and
      // handing a reactive proxy to non-Svelte code is asking for trouble.
      satellites: $state.snapshot(sim.satellites),
      start_unix_seconds: Date.now() / 1000,
      duration_seconds: durationSeconds,
      step_seconds: stepSeconds,
    });
    sim.status = "idle";
  } catch (error) {
    sim.error = error instanceof Error ? error.message : String(error);
    sim.status = "error";
  }
}

/** The Unix timestamp, in seconds, of sample `index`. */
export function sampleTime(result: SimulationResult, index: number): number {
  return result.start_unix_seconds + index * result.step_seconds;
}

/**
 * Where Cesium's clock sits within the simulated window, in elapsed minutes.
 *
 * Returns null when there is no result yet or the clock is outside the window,
 * so the plots can simply omit the marker. Reads reactive state, so calling it
 * from a `$derived` tracks the clock.
 */
export function playheadMinutes(): number | null {
  const result = sim.result;
  const now = sim.current_unix_seconds;
  if (!result || now === null) return null;

  const elapsed_seconds = now - result.start_unix_seconds;
  const window_seconds = (result.sample_count - 1) * result.step_seconds;
  if (elapsed_seconds < 0 || elapsed_seconds > window_seconds) return null;

  return elapsed_seconds / 60;
}
