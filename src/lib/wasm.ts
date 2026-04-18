let wasmInitialized = false;

async function initWasm() {
  if (wasmInitialized) {
    return;
  }

  try {
    // Import the WASM module and call its default init function
    const { default: init } = await import("./wasm/pkg/orbit_wasm.js");
    await init();
    wasmInitialized = true;
  } catch (error) {
    console.error("Failed to load WASM module:", error);
    throw error;
  }
}

export async function helloWorld(): Promise<string> {
  await initWasm();
  const { hello_world } = await import("./wasm/pkg/orbit_wasm.js");
  return hello_world();
}

export async function add(a: number, b: number): Promise<number> {
  await initWasm();
  const { add: addFn } = await import("./wasm/pkg/orbit_wasm.js");
  return addFn(a, b);
}
