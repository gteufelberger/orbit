<script lang="ts">
  import { getWasm } from "./wasm";
  import { runSimulation, sim } from "$lib/simulation.svelte";

  async function call_demo(
    pick: (wasm: Awaited<ReturnType<typeof getWasm>>) => unknown,
  ) {
    alert(pick(await getWasm()));
  }
</script>

<button
  class="primary"
  disabled={sim.status === "running"}
  onclick={() => runSimulation()}
>
  {sim.status === "running" ? "Simulating…" : "Run simulation"}
</button>

{#if sim.error}
  <p class="error">{sim.error}</p>
{/if}

<hr />

<details>
  <summary>Debug</summary>
  <button onclick={() => call_demo((wasm) => wasm.add(2, 2))}>
    Call WASM add Function
  </button>
  <button onclick={() => call_demo((wasm) => wasm.hello_world())}>
    Call WASM hello_world Function
  </button>
</details>

<style>
  .primary {
    width: 100%;
    padding: 0.5em;
    font-weight: 600;
  }

  .error {
    margin: 0.5em 0;
    color: #b00020;
    font-size: 0.85em;
    overflow-wrap: anywhere;
  }

  hr {
    margin: 1em 0 0.5em;
    border: none;
    border-top: 1px solid #ddd;
  }

  summary {
    cursor: pointer;
    font-size: 0.85em;
    color: #666;
  }

  details button {
    margin-top: 0.4em;
    width: 100%;
  }
</style>
