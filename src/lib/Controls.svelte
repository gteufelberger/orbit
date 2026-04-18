<script lang="ts">
  import { add } from "./wasm";

  async function callWasm() {
    try {
      const result = await add(2, 2);
      alert(result);
    } catch (err) {
      console.error("WASM call failed:", err);
    }
  }
</script>

<button onclick={callWasm}> Call WASM Function </button>
