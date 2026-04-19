export async function getWasm(): Promise<OrbitWasmModule> {
  const wasm = await import("./wasm/pkg/orbit_wasm.js");
  await wasm.default();
  return wasm as unknown as OrbitWasmModule;
}

export interface OrbitWasmModule {
  add: (a: number, b: number) => number;
  hello_world: () => string;
}
