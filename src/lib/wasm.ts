import type { SimulationConfig } from "$bindings/SimulationConfig";
import type { SimulationResult } from "$bindings/SimulationResult";

/**
 * The functions `orbit_lib` exposes through `wasm_bindgen`.
 *
 * wasm-bindgen generates `simulate(config: any): any` because `JsValue` carries
 * no shape information, so the signature is asserted here while the payload
 * shapes come from the ts-rs bindings. Cross-check against
 * `wasm/pkg/orbit_wasm.d.ts` after changing the Rust boundary.
 */
export interface OrbitWasmModule {
  add: (a: number, b: number) => number;
  hello_world: () => string;
  simulate: (config: SimulationConfig) => SimulationResult;
}

let pending: Promise<OrbitWasmModule> | null = null;

/**
 * Loads and initializes the WASM module, at most once per page load.
 *
 * Memoized because `wasm.default()` re-initializes the instance on every call,
 * and more than one caller reaches for the module.
 */
export function getWasm(): Promise<OrbitWasmModule> {
  return (pending ??= load());
}

async function load(): Promise<OrbitWasmModule> {
  try {
    const wasm = await import("./wasm/pkg/orbit_wasm.js");
    await wasm.default();
    return wasm as unknown as OrbitWasmModule;
  } catch (error) {
    // Drop the rejected promise so a retry can try again rather than replaying
    // the same failure forever.
    pending = null;
    throw error;
  }
}
