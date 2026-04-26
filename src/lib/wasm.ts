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
  calculate_satellite_position: (
    time_seconds: number,
    orbital_period: number,
    orbital_radius: number,
  ) => number[];
}
