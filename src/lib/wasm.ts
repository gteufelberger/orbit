export async function getWasm(): Promise<OrbitWasmModule> {
  try {
    const wasm = await import("./wasm/pkg/orbit_wasm.js");
    await wasm.default();
    return wasm as OrbitWasmModule;
  } catch (error) {
    alert("Failed to import WASM module");
    throw error;
  }
}

export interface OrbitWasmModule {
  add: (a: number, b: number) => number;
  hello_world: () => string;
}
