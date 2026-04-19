<script lang="ts">
  import { add, helloWorld } from "./wasm";

  async function callWasmAdd() {
    try {
      const result = await add(2, 2);
      alert(result);
    } catch (err) {
      console.error("WASM call failed:", err);
    }
  }
  async function callWasmHelloWorld() {
    try {
      const result = await helloWorld();
      alert(result);
    } catch (err) {
      console.error("WASM call failed:", err);
    }
  }
</script>

<button onclick={callWasmAdd}> Call WASM add Function </button>
<button onclick={callWasmHelloWorld}> Call WASM hello_world Function </button>
