<script lang="ts">
  import { getWasm, type OrbitWasmModule } from "./wasm";

  let {
    inertialMode,
    onToggleInertialMode,
  }: {
    inertialMode: boolean;
    onToggleInertialMode: () => void;
  } = $props();

  let wasm_module: OrbitWasmModule | null = null;
  $effect(() => {
    (async () => {
      wasm_module = await getWasm();
    })();
  });
</script>

<button class:active={inertialMode} onclick={onToggleInertialMode}>
  {inertialMode ? "ICRF" : "ECEF"}
</button>
<button onclick={() => alert(wasm_module!.add(2, 2))}>
  Call WASM add Function
</button>
<button onclick={() => alert(wasm_module!.hello_world())}>
  Call WASM hello_world Function
</button>

<style>
  button.active {
    color: #fff;
    background: rgba(20, 60, 100, 0.85);
  }
</style>
