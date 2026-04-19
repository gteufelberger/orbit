<script lang="ts">
  // import { add, helloWorld } from "./wasm";
  import { getWasm, type OrbitWasmModule } from "./wasm";

  let wasm_module: OrbitWasmModule | null;
  $effect(() => {
    (async () => {
      wasm_module = await getWasm();
    })();
  });
</script>

<button onclick={() => alert(wasm_module!.add(2, 2))}>
  Call WASM add Function
</button>
<button onclick={() => alert(wasm_module!.hello_world())}>
  Call WASM hello_world Function
</button>
