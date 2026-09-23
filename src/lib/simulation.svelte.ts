import type { Satellite } from "$bindings/Satellite";
import type { SimulationResult } from "$bindings/SimulationResult";
import { getWasm } from "$lib/wasm";

const ORBIT_SECONDS = 99 * 60;

/**
 * A satellite plus the model file in `static/models/` to draw it with, and the
 * across-track width of its imager, omitted when it does not carry one.
 */
export type DisplaySatellite = Satellite & {
  model: string;
  swath_meters?: number;
};

/**
 * Shared simulation state. Controls and CesiumViewer are not in an ancestor
 * relationship, so props cannot carry this between them.
 *
 * `result.satellites[i]` belongs to `satellites[i]` — same length, same order.
 */
export const sim = $state({
  satellites: [
    {
      name: "Landsat 9",
      id: "landsat9",
      model: "landsat",
      swath_meters: 185_000,
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
      model: "generic",
      swath_meters: 150_000,
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
      model: "iss",
      tle: "1 25544U 98067A   26264.49719541  .00007274  00000-0  13903-3 0  9999\n2 25544  51.6310 183.4357 0004788 164.7753 195.3381 15.49212079586682",
      components: [
        {
          kind: "SolarPanel",
          id: "array",
          area_square_meters: 2500,
          efficiency: 0.047,
        },
        {
          kind: "Battery",
          id: "main_battery",
          capacity_watt_hours: 96000,
          initial_state_of_charge: 0.9,
          load_watts: 80000,
        },
      ],
    },
    {
      name: "Sentinel 3B",
      id: "sentinel-3b",
      model: "generic",
      swath_meters: 1_270_000,
      tle: "1 43437U 18039A   26198.00000000  .00000000  00000+0 -28164-1 0 00014\n2 43437  98.6222 265.1030 0001295 123.2206 263.1193 14.26747824428387",
      components: [
        {
          kind: "SolarPanel",
          id: "array",
          area_square_meters: 10,
          efficiency: 0.155,
        },
        {
          kind: "Battery",
          id: "main_battery",
          capacity_watt_hours: 4480,
          initial_state_of_charge: 0.9,
          load_watts: 1050,
        },
      ],
    },
    {
      name: "Sentinel 2B",
      id: "sentinel-2b",
      model: "generic",
      swath_meters: 290_000,
      tle: "1 42063U 17013A   26266.31938991  .00000098  00000-0  54015-4 0  9998\n2 42063  98.5710 339.6817 0001173  87.9132 272.2185 14.30815998498675",
      components: [
        {
          kind: "SolarPanel",
          id: "array",
          area_square_meters: 7.1,
          efficiency: 0.179,
        },
        {
          kind: "Battery",
          id: "main_battery",
          capacity_watt_hours: 2940,
          initial_state_of_charge: 0.9,
          load_watts: 900,
        },
      ],
    },
  ] as DisplaySatellite[],
  result: null as SimulationResult | null,
  /** Cesium's clock, published by the viewer so the plots can mark it. */
  current_unix_seconds: null as number | null,
  status: "idle" as "idle" | "running" | "error",
  error: null as string | null,
});

export async function runSimulation(
  durationSeconds = 3 * ORBIT_SECONDS,
  stepSeconds = 10,
): Promise<void> {
  sim.status = "running";
  sim.error = null;

  try {
    const wasm = await getWasm();

    // Snapshot: serde reads plain properties, not reactive proxies. The extra
    // `model` key is an unknown field, which serde ignores.
    sim.result = wasm.simulate({
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

/** Cesium's clock as elapsed minutes, or null when outside the window. */
export function playheadMinutes(): number | null {
  const result = sim.result;
  const now = sim.current_unix_seconds;
  if (!result || now === null) return null;

  const elapsed = now - result.start_unix_seconds;
  if (
    elapsed < 0 ||
    elapsed > (result.sample_count - 1) * result.step_seconds
  ) {
    return null;
  }

  return elapsed / 60;
}
