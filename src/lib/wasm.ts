let wasmModule: OrbitWasmModule | null = null;

export async function initWasm(): Promise<void> {
  // Return early if already set up
  if (wasmModule) return;

  const wasm = await import("./wasm/pkg/orbit_wasm.js");
  await wasm.default();
  wasmModule = wasm as unknown as OrbitWasmModule;
}

export async function helloWorld(): Promise<string> {
  await initWasm();
  return wasmModule!.hello_world();
}

export async function add(a: number, b: number): Promise<number> {
  await initWasm();
  return wasmModule!.add(a, b);
}

interface OrbitWasmModule {
  add: (a: number, b: number) => number;
  hello_world: () => string;
}
