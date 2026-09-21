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
  button {
    width: 100%;
    padding: 0.45em;
    border: 1px solid var(--line);
    border-radius: 4px;
    background: var(--raised);
    color: inherit;
    font: inherit;
    cursor: pointer;
  }

  button:hover:not(:disabled) {
    border-color: var(--accent);
  }

  button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .primary {
    font-weight: 600;
  }

  .error {
    margin: 0.5em 0;
    color: #ff7a7a;
    font-size: 0.85em;
    overflow-wrap: anywhere;
  }

  hr {
    margin: 1em 0 0.5em;
    border: none;
    border-top: 1px solid var(--line);
  }

  summary {
    cursor: pointer;
    font-size: 0.85em;
    color: var(--fg-dim);
  }

  details button {
    margin-top: 0.4em;
  }
</style>
