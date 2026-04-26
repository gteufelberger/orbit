export async function getWasm(): Promise<OrbitWasmModule> {
  try {
    const wasm = await import("./wasm/pkg/orbit_wasm.js");
    await wasm.default();
    return wasm as unknown as OrbitWasmModule;
  } catch (error) {
    alert("Failed to import WASM module");
    throw error;
  }
}

export interface OrbitWasmModule {
  add: (a: number, b: number) => number;
  hello_world: () => string;
  orbital_period: (semi_major_axis: number) => number;
  calculate_position_keplerian: (
    semi_major_axis: number,
    eccentricity: number,
    inclination: number,
    raan: number,
    arg_periapsis: number,
    mean_anomaly_epoch: number,
    epoch_unix: number,
    time_unix: number,
  ) => number[];
  calculate_orbit_path: (
    semi_major_axis: number,
    eccentricity: number,
    inclination: number,
    raan: number,
    arg_periapsis: number,
    mean_anomaly_epoch: number,
    epoch_unix: number,
    time_unix: number,
    num_points: number,
    inertial: boolean,
  ) => number[];
}
